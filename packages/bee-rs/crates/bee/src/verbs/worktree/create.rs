// createFeatureWorktree
//
// Split out of the single 4.2k-line verbs/worktree.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_roots_core, Resolution};
use crate::verbs::reservations::{js_numberify, js_trim, now_iso, parse_flags, FlagV, Flags};
use crate::verbs::workspace_store as ws;
use crate::verbs::{emit_no_root_error, record_timing};
use crate::{jsjson, lock};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::process::ExitCode;
use std::time::Instant;

// ─── worktree-store.mjs createFeatureWorktree ─────────────────────────────

/// `refuse(code, message)` throws a WorktreeCreateError whose `.message` is
/// `[CODE] message` — bee.mjs's dispatcher surfaces only `.message`, so the
/// bracket prefix is part of every observable byte.
pub(crate) fn refuse(code: &str, message: String) -> CErr {
    CErr::Refuse(format!("[{code}] {message}"))
}

pub(crate) enum CErr {
    /// A shape whose Node bytes embed a V8 message — delegate. Only ever
    /// returned BEFORE `git worktree add` runs (nothing has mutated).
    Ex,
    Refuse(String),
}

pub(crate) struct Created {
    pub(crate) id: String,
    pub(crate) worktree_root: PathBuf,
    pub(crate) branch: String,
    pub(crate) base_ref: Option<String>,
    pub(crate) base_ref_sha: Option<String>,
    pub(crate) bootstrap: Map<String, Value>,
    /// `null`, or runCompanionStart's `{sessionId, worktreePath, mountPath}`.
    pub(crate) companion: Value,
    pub(crate) skills_sync: Value,
}

/// The `--with-companion` pair, both-or-neither. Node passes the two as
/// separate options and re-checks the pairing inside
/// createFeatureWorktreeLocked; this keeps that check reachable (see
/// WORKTREE_COMPANION_CONFIG_INCOMPLETE there) by carrying them as two
/// independent `Option`s rather than collapsing them into one.
#[derive(Default, Clone, Copy)]
pub(crate) struct CompanionSpec<'a> {
    pub(crate) start_command: Option<&'a str>,
    pub(crate) mount_path: Option<&'a str>,
}

/// FEATURE_SLUG_RE = /^[a-z0-9][a-z0-9-]*$/.
pub(crate) fn feature_slug_ok(feature: &str) -> bool {
    let mut chars = feature.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// createFeatureWorktree, whole. The entire body — every validation, every
/// refusal, the `git worktree add`, the post-add block and the rollback
/// ladder — runs inside ONE `worktree-admin` hold on mainRoot, exactly as
/// Node's `withStoreLock(mainRoot, 'worktree-admin', ...)` wrapper does.
///
/// `companion` carries `--with-companion`'s two config strings (both-or-
/// neither, re-checked below exactly as Node re-checks them): with it present
/// the post-add block also runs `runCompanionStart`, whose failure enters the
/// SAME rollback ladder as any other post-add failure.
pub(crate) fn create_feature_worktree(
    main_root: &Path,
    feature: &str,
    base_ref: Option<&str>,
    companion: CompanionSpec<'_>,
    lock_busy: &mut Option<String>,
) -> Result<Created, CErr> {
    let main_store_root = main_root.join(".bee");
    // Pre-probe BEFORE the lock: an unparseable grants registry delegates
    // here rather than from inside the hold (campaign rule 2 — a delegation
    // after an acquire would double contention.jsonl's telemetry).
    read_grants_strict(&main_store_root).ok_or(CErr::Ex)?;

    let mut guard = match lock::acquire_store_lock(main_root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS)
    {
        Ok(g) => g,
        Err(busy) => {
            *lock_busy = Some(busy.message());
            return Err(CErr::Ex); // signalled to the caller through lock_busy
        }
    };
    let out = create_feature_worktree_locked(main_root, feature, base_ref, companion);
    guard.release();
    out
}

pub(crate) fn create_feature_worktree_locked(
    main_root: &Path,
    feature: &str,
    base_ref: Option<&str>,
    companion: CompanionSpec<'_>,
) -> Result<Created, CErr> {
    // (1) slug.
    if !feature_slug_ok(feature) {
        return Err(refuse(
            "WORKTREE_INVALID_SLUG",
            format!(
                "feature slug {} must match /^[a-z0-9][a-z0-9-]*$/ (lowercase letters/digits, starting with a letter or digit, hyphens allowed after that).",
                jsjson::stringify(&Value::String(feature.to_string()))
            ),
        ));
    }

    // (2) companion both-or-neither. The CLI handler already refuses each
    // half's absence when --with-companion is passed; this is the defensive
    // invariant for any OTHER caller, and it is a zero-mutation refusal.
    // JS truthiness: an empty string counts as absent on both sides.
    let start_command = companion.start_command.filter(|s| !s.is_empty());
    let mount_path_raw = companion.mount_path.filter(|s| !s.is_empty());
    let mut companion_mount: Option<String> = None;
    if start_command.is_some() || mount_path_raw.is_some() {
        let (Some(_), Some(mount)) = (start_command, mount_path_raw) else {
            return Err(refuse(
                "WORKTREE_COMPANION_CONFIG_INCOMPLETE",
                "commands.worktree_companion_start and commands.worktree_companion_mount must both be configured to use --with-companion — only one was found."
                    .to_string(),
            ));
        };
        companion_mount = Some(validate_companion_mount_path(mount)?);
    }

    // (3) base ref. `baseRef !== undefined && !== null && !== ''`, so an
    // empty --base-ref is treated as absent, exactly like Node.
    let mut base_ref_sha: Option<String> = None;
    if let Some(r) = base_ref.filter(|s| !s.is_empty()) {
        match resolve_base_ref_commit(main_root, r) {
            Some(sha) => base_ref_sha = Some(sha),
            None => {
                return Err(refuse(
                    "WORKTREE_BASE_NOT_FOUND",
                    format!(
                        "--base-ref {} does not resolve to a commit in {} (\"git rev-parse --verify\" found nothing) — check the ref/sha/tag exists (and isn't just a syntax typo).",
                        jsjson::stringify(&Value::String(r.to_string())),
                        p(main_root)
                    ),
                ))
            }
        }
    }

    // (4) the belt-and-braces ordinary-checkout guard.
    if !is_ordinary_checkout(main_root) {
        return Err(refuse(
            "WORKTREE_CALLER_NOT_ORDINARY",
            format!(
                "\"bee worktree new\" must be run from the main checkout, not a linked worktree ({} is not an ordinary checkout).",
                p(main_root)
            ),
        ));
    }

    // (5) derivation.
    let repo_basename = main_root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let sibling_dir_name = format!("{repo_basename}--wt--{feature}");
    let worktree_root = js_path_resolve(&js_path_resolve(main_root, ".."), &sibling_dir_name);
    let branch = format!("wt/{feature}");
    let main_store_root = main_root.join(".bee");

    // (6) target path.
    if worktree_root.exists() {
        return Err(refuse(
            "WORKTREE_TARGET_EXISTS",
            format!("{} already exists.", p(&worktree_root)),
        ));
    }

    // (7) branch.
    if branch_exists(main_root, &branch) {
        return Err(refuse(
            "WORKTREE_BRANCH_EXISTS",
            format!("branch \"{branch}\" already exists in {}.", p(main_root)),
        ));
    }

    // (8) advisory grant collision (strict `=== true`).
    let grants = read_grants_strict(&main_store_root).ok_or(CErr::Ex)?;
    let likely_id = sibling_dir_name;
    if grants.get(&likely_id) == Some(&Value::Bool(true)) {
        return Err(refuse(
            "WORKTREE_GRANT_EXISTS",
            format!(
                "a worktree grant already exists for id \"{likely_id}\" — run \"bee worktree unregister --id {likely_id}\" (or \"git worktree prune\") before retrying."
            ),
        ));
    }

    // (9) THE MUTATION. The RESOLVED SHA is what git receives, never the
    // original ref string.
    let worktree_root_s = p(&worktree_root);
    let mut add_args: Vec<&str> = vec!["worktree", "add", "-b", &branch, "--", &worktree_root_s];
    if let Some(sha) = &base_ref_sha {
        add_args.push(sha);
    }
    let add_result = run_git(main_root, &add_args);
    if add_result.status != Some(0) {
        return Err(refuse(
            "WORKTREE_ADD_FAILED",
            format!("git worktree add failed: {}", add_result.fail_text()),
        ));
    }

    // (10) the post-add block. Every failure below enters the rollback
    // ladder; NONE of them may delegate (the worktree already exists).
    let mut id: Option<String> = None;
    let attempt = post_add(
        main_root,
        &main_store_root,
        &worktree_root,
        feature,
        &branch,
        base_ref,
        base_ref_sha.as_deref(),
        grants,
        start_command.zip(companion_mount.as_deref()),
        &mut id,
    );
    match attempt {
        Ok(created) => Ok(created),
        Err(post_add_message) => Err(rollback(
            main_root,
            &main_store_root,
            &worktree_root,
            feature,
            &branch,
            id.as_deref(),
            &post_add_message,
        )),
    }
}

/// Steps 10.1-10.7, in Node's exact order. `Err(String)` carries the message
/// the ladder interpolates as `postAddMessage`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn post_add(
    main_root: &Path,
    main_store_root: &Path,
    worktree_root: &Path,
    feature: &str,
    branch: &str,
    base_ref: Option<&str>,
    base_ref_sha: Option<&str>,
    grants: Map<String, Value>,
    companion: Option<(&str, &str)>,
    id_out: &mut Option<String>,
) -> Result<Created, String> {
    // 10.1 — the authoritative id. `id` stays None if this throws, so the
    // ladder's `if (id)` rungs are skipped, exactly as in Node.
    let id = read_worktree_git_verified_id(worktree_root)?;
    *id_out = Some(id.clone());

    // 10.2 — writeGrantCore (the UNLOCKED core: withStoreLock is not
    // reentrant and this whole body already holds 'worktree-admin').
    let mut next = grants;
    next.insert(id.clone(), Value::Bool(true));
    write_grants_file_atomic(main_store_root, &next)
        .map_err(|e| node_fs_error_message(&e, "open", &grants_file(main_store_root)))?;

    // 10.3 — bootstrap the worktree's own store.
    let bootstrap = bootstrap_worktree_store(worktree_root, main_store_root, feature)
        .ok_or_else(|| "EIO: i/o error, open".to_string())?;

    // 10.4 — the workspace base sha. `git worktree add` has just succeeded,
    // so git is provably launchable and `.stdout` is never null here — the
    // one Node shape (`TypeError: Cannot read properties of null`) that this
    // line can raise is unreachable by construction.
    let workspace_base_sha: Option<String> = match base_ref_sha {
        Some(sha) => Some(sha.to_string()),
        None => {
            let head = run_git(worktree_root, &["rev-parse", "HEAD"]);
            let trimmed = js_trim(head.stdout.as_deref().unwrap_or("")).to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
    };

    // 10.5 — registerWorkspace: the WRITE-OWNERSHIP ledger, alongside the
    // grant's STORE-TOPOLOGY ledger. Takes its own `workspace:<id>` lock
    // NESTED inside the 'worktree-admin' hold — a different lock name, so
    // never a self-deadlock, exactly as Node nests it.
    ws::register_workspace(
        main_root,
        ws::RegisterSpec {
            id: &id,
            kind: "worktree",
            root: &p(worktree_root),
            branch: Some(branch),
            base_sha: workspace_base_sha.as_deref(),
        },
        &now_iso(),
    )
    .map_err(|e| match e {
        // A WorkspaceStoreError / LockBusyError message, reproduced natively
        // by verbs/workspace_store.rs — this is the arm that used to be the
        // reason the whole verb delegated.
        ws::WsErr::Err { message, .. } => message,
        ws::WsErr::Ex => "EIO: i/o error, open".to_string(),
    })?;

    // 10.6 — companion. Deliberately INSIDE this fallible block: a companion
    // start failure folds into the exact same post-add rollback as any other
    // failure after `git worktree add` succeeded, so a worktree is never left
    // created-and-registered but silently missing the companion it asked for.
    let companion = match companion {
        Some((command, mount)) => {
            run_companion_start(main_root, worktree_root, command, mount)?
        }
        None => Value::Null,
    };

    // 10.7 — skills: best-effort, never fatal, never in the ladder.
    let skills_sync = sync_worktree_skills(main_root, worktree_root);

    Ok(Created {
        id,
        worktree_root: worktree_root.to_path_buf(),
        branch: branch.to_string(),
        base_ref: base_ref.filter(|s| !s.is_empty()).map(str::to_string),
        base_ref_sha: base_ref_sha.map(str::to_string),
        bootstrap,
        companion,
        skills_sync,
    })
}

/// The ROLLBACK LADDER, in Node's exact order. Order is load-bearing: a
/// different unwind leaves a different tree behind, which is the C1 breach
/// the campaign forbids.
///
///   R1  removeGrantCore(mainStoreRoot, id)        only if id, best-effort
///   R2  unregisterWorkspace(mainRoot, id)         only if id, best-effort
///   R3  git worktree remove --force <worktreeRoot>   unconditional
///   R4  fs.existsSync(worktreeRoot) -> stillPresent
///   R5  git branch -D <branch>                    only if R3 ok && !R4
///   R6  refuse WORKTREE_POST_ADD_FAILED           same gate as R5
///   R7  refuse WORKTREE_POST_ADD_ROLLBACK_FAILED  otherwise
pub(crate) fn rollback(
    main_root: &Path,
    main_store_root: &Path,
    worktree_root: &Path,
    feature: &str,
    branch: &str,
    id: Option<&str>,
    post_add_message: &str,
) -> CErr {
    if let Some(id) = id {
        // R1 — removeGrantCore: a no-op (no write at all) when the id is
        // absent. Best-effort; the typed refusal below fires either way.
        if let Some(existing) = read_grants_strict(main_store_root) {
            if existing.contains_key(id) {
                let mut next = existing;
                next.remove(id);
                let _ = write_grants_file_atomic(main_store_root, &next);
            }
        }
        // R2 — best-effort workspace unregister, always after R1.
        let _ = ws::unregister_workspace(main_root, id);
    }
    // R3 — unconditional; its status is the branch point.
    let remove_result = run_git(main_root, &["worktree", "remove", "--force", &p(worktree_root)]);
    // R4.
    let still_present = worktree_root.exists();
    if remove_result.status == Some(0) && !still_present {
        // R5 — only once the worktree is confirmed gone (git refuses to
        // delete a branch a live worktree still has checked out).
        let _ = run_git(main_root, &["branch", "-D", branch]);
        // R6.
        return refuse(
            "WORKTREE_POST_ADD_FAILED",
            format!(
                "{} was created but could not be registered ({post_add_message}); it has been rolled back (worktree and branch \"{branch}\" removed).",
                p(worktree_root)
            ),
        );
    }
    // R7.
    refuse(
        "WORKTREE_POST_ADD_ROLLBACK_FAILED",
        format!(
            "{} was created but could not be registered ({post_add_message}), and the rollback itself failed — the tree still exists on disk; run \"bee worktree register --feature {feature}\" from inside it to adopt it.",
            p(worktree_root)
        ),
    )
}
