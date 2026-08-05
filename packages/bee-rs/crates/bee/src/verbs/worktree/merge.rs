// mergeFeatureWorktree and the verify child
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

// ─── worktree-store.mjs mergeFeatureWorktree ──────────────────────────────
//
// The three-phase staged transaction, whole (see the module header for the
// two delegation gates that keep every V8-worded arm out of reach):
//   P1  mergeFeatureWorktreeStage   — LOCKED ('worktree-admin' on mainRoot)
//   P2  runVerifyChild              — UNLOCKED, only when a verify command
//                                     is configured (hardening-4c)
//   P3  mergeFeatureWorktreeFinish  — RE-LOCKED
// Node acquires 'worktree-admin' TWICE on every non-terminal merge (P1 then
// P3) even when no verify runs, so this port does too — a single hold would
// drop one `result: "acquired"` row from .bee/logs/contention.jsonl.

/// `WorktreeMergeError` — `[CODE] message`, the only observable byte.
pub(crate) fn refuse_merge(code: &str, message: String) -> MErr {
    MErr::Thrown(format!("[{code}] {message}"))
}

/// The merge's failure channel. `Thrown` is a message bee.mjs's dispatcher
/// surfaces through emitError AND processAsOwner persists into the queue
/// record's `error` field — every arm that produces one is deterministic.
pub(crate) enum MErr {
    Thrown(String),
    /// Only ever returned BEFORE `git merge --no-ff --no-commit` runs.
    Ex,
}

pub(crate) type MR<T> = Result<T, MErr>;

/// The merge's own `{ok, ...}` answer: the result object plus the exit code
/// bee.mjs derives from it and the queue status processAsOwner writes.
pub(crate) struct MergeAnswer {
    pub(crate) result: Map<String, Value>,
    pub(crate) ok: bool,
}

/// gitStatusPorcelain — deliberately WITHOUT `--ignored` (decision D8a). A
/// git failure is a plain (untyped) Error whose bytes are still fully
/// deterministic, including the literal "exit null" a never-launched spawn
/// renders.
pub(crate) fn git_status_porcelain(cwd: &Path) -> Result<String, String> {
    let r = run_git(cwd, &["status", "--porcelain"]);
    if r.status != Some(0) {
        return Err(format!(
            "\"git status --porcelain\" failed in {}: {}",
            p(cwd),
            r.fail_text()
        ));
    }
    Ok(r.stdout.unwrap_or_default())
}

pub(crate) fn is_tree_dirty(cwd: &Path) -> Result<bool, String> {
    Ok(!js_trim(&git_status_porcelain(cwd)?).is_empty())
}

/// gitStatusPorcelainExcluding — `git status --porcelain -- :(exclude)<p> …`.
///
/// Deliberately NOT post-hoc text filtering: porcelain COLLAPSES an untracked
/// directory (or a symlink-to-directory, which is exactly what a companion
/// mount is) to one summary line for its top-level name, so a mount at
/// `vendor/companion` shows only as `?? vendor/` and a text filter for the
/// mount path would never match — the merge would refuse forever. Asking git
/// itself never to report those paths removes them at the source, at any
/// depth and under any quoting. Multiple `:(exclude)` pathspecs with no
/// positive pathspec among them still mean "everything else in the tree"
/// (git's own pathspec-magic contract), so excluding two is not a narrowing.
///
/// Pathspecs are `/`-only even on Windows, hence the `\` → `/` rewrite.
pub(crate) fn git_status_porcelain_excluding(cwd: &Path, exclude_paths: &[String]) -> Result<String, String> {
    let pathspecs: Vec<String> = exclude_paths
        .iter()
        .map(|p| format!(":(exclude){}", p.replace('\\', "/")))
        .collect();
    let mut args: Vec<&str> = vec!["status", "--porcelain", "--"];
    args.extend(pathspecs.iter().map(String::as_str));
    let r = run_git(cwd, &args);
    if r.status != Some(0) {
        return Err(format!(
            "\"git status --porcelain -- {}\" failed in {}: {}",
            pathspecs.join(" "),
            p(cwd),
            r.fail_text()
        ));
    }
    Ok(r.stdout.unwrap_or_default())
}

pub(crate) fn is_tree_dirty_excluding(cwd: &Path, exclude_paths: &[String]) -> Result<bool, String> {
    Ok(!js_trim(&git_status_porcelain_excluding(cwd, exclude_paths)?).is_empty())
}

/// The three-part "main was left byte-untouched" proof (decision D2-REVISED)
/// required after EVERY `git merge --abort` this module runs. `Ok(())` is
/// `{ok:true}`; `Err(reason)` is the `{ok:false, reason}` the caller folds
/// into a SPECIFIC typed refusal.
///
/// Both `runGit(...).stdout.trim()` sites here are un-null-guarded in Node (a
/// TypeError whose V8 text would be persisted into the queue record). They are
/// unreachable by construction: `mergeFeatureWorktreeStage`'s very first git
/// call is `isTreeDirty(mainRoot)`, which throws the deterministic
/// `"git status --porcelain" failed in <root>: exit null` message — with ZERO
/// mutations — before any merge is staged. So by the time this function can
/// run, git has provably launched at least once from this same cwd. The
/// residual (git becoming unlaunchable mid-merge) is the same race class
/// verbs/workspace_store.rs documents for a record going unreadable between
/// its probe and its in-lock read.
pub(crate) fn main_untouched_proof(main_root: &Path, pre_merge_head: &str, merge_head_file: &Path) -> Result<(), String> {
    let head_now = js_trim(
        &run_git(main_root, &["rev-parse", "HEAD"]).stdout.unwrap_or_default(),
    )
    .to_string();
    if head_now != pre_merge_head {
        return Err(format!("HEAD moved from {pre_merge_head} to {head_now}"));
    }
    if merge_head_file.exists() {
        return Err(".git/MERGE_HEAD is still present".to_string());
    }
    let status = run_git(main_root, &["status", "--porcelain", "--untracked-files=no"])
        .stdout
        .unwrap_or_default();
    if !js_trim(&status).is_empty() {
        return Err(format!(
            "\"git status --porcelain --untracked-files=no\" is not clean:\n{status}"
        ));
    }
    Ok(())
}

/// currentBranch — `null` on detached HEAD (or no HEAD ref at all).
pub(crate) fn current_branch(cwd: &Path) -> Option<String> {
    let r = run_git(cwd, &["symbolic-ref", "-q", "--short", "HEAD"]);
    if r.status != Some(0) {
        return None;
    }
    Some(js_trim(&r.stdout.unwrap_or_default()).to_string())
}

/// The two never-throwing `feature` reads behind resolveWorktreeFeature. A
/// missing/corrupt/foreign file is simply "unknown" in BOTH runtimes (the read
/// is a bare `JSON.parse(readFileSync(...))` in a `try`, never fsutil's
/// warning `readJson`), so a parse failure needs no delegation here.
pub(crate) fn read_json_feature(file: &Path) -> Option<String> {
    let raw = std::fs::read(file).ok()?;
    let parsed: Value = serde_json::from_slice(&raw).ok()?;
    match parsed.get("feature") {
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        _ => None,
    }
}

pub(crate) struct WorktreeFeature {
    pub(crate) feature: Option<String>,
    pub(crate) created: Option<String>,
    pub(crate) state_feature: Option<String>,
}

/// resolveWorktreeFeature — the IMMUTABLE creation slug wins over the mutable
/// `state.feature` (issues-46-53 D4), degrading exactly to the pre-fix
/// behavior when no creation record exists.
pub(crate) fn resolve_worktree_feature(worktree_root: &Path) -> WorktreeFeature {
    let created = read_json_feature(
        &worktree_root
            .join(".bee")
            .join("runtime")
            .join("worktree-identity.json"),
    );
    let state_feature = read_json_feature(&worktree_root.join(".bee").join("state.json"));
    WorktreeFeature {
        feature: created.clone().or_else(|| state_feature.clone()),
        created,
        state_feature,
    }
}

/// WT_BRANCH_RE = /^wt\/[a-z0-9][a-z0-9-]*$/.
pub(crate) fn wt_branch_shaped(branch: &str) -> bool {
    branch.strip_prefix("wt/").is_some_and(feature_slug_ok)
}

/// resolveWorktreeById — the same BIDIRECTIONAL gitdir validation
/// resolveRoots uses, keyed by id instead of by walking up from a cwd. `None`
/// on ANY mismatch, missing file or unreadable content, so "no such id" and
/// "id's link is broken" fold into one typed refusal.
///
/// The reverse comparison goes through path_identity's `canonical_paths_equal`
/// (windows-path-identity wpi-1), NOT a byte compare — the shared fix.
pub(crate) fn resolve_worktree_by_id(main_root: &Path, id: &str) -> Option<PathBuf> {
    let git_worktree_dir = main_root.join(".git").join("worktrees").join(id);
    if !std::fs::metadata(&git_worktree_dir).map(|m| m.is_dir()).unwrap_or(false) {
        return None;
    }
    let forward_raw = std::fs::read_to_string(git_worktree_dir.join("gitdir")).ok()?;
    let forward_raw = js_trim(&forward_raw);
    if forward_raw.is_empty() {
        return None;
    }
    let resolved_git_file = js_path_resolve(
        &git_worktree_dir,
        &forward_raw.replace('\\', &MAIN_SEPARATOR.to_string()),
    );
    let worktree_root = resolved_git_file.parent()?.to_path_buf();

    let reverse_raw = std::fs::read_to_string(worktree_root.join(".git")).ok()?;
    let captured = parse_gitdir_pointer(js_trim(&reverse_raw))?;
    let reverse_resolved = js_path_resolve(
        &worktree_root,
        &js_trim(captured).replace('\\', &MAIN_SEPARATOR.to_string()),
    );
    if !crate::path_identity::canonical_paths_equal(&reverse_resolved, &git_worktree_dir) {
        return None;
    }
    Some(worktree_root)
}

/// worktree-holds.mjs releaseAllForHolder — every unreleased hold for `id`,
/// marked released under the shared `cross-worktree-holds` lock on mainRoot.
/// BEST-EFFORT at its one call site (performCleanup wraps it in `try/catch`),
/// so every failure here is swallowed exactly as Node swallows the throw.
pub(crate) fn release_all_for_holder(main_root: &Path, id: &str) {
    let Ok(mut guard) =
        lock::acquire_store_lock(main_root, "cross-worktree-holds", lock::MAX_ATTEMPTS)
    else {
        return;
    };
    let file = main_root
        .join(".bee")
        .join("runtime")
        .join("cross-worktree-holds.json");
    let mut store: Value = match std::fs::read(&file) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        Err(_) => Value::Null,
    };
    let mut released = 0usize;
    let released_at = now_iso();
    if let Some(Value::Array(holds)) = store.get_mut("holds") {
        for hold in holds.iter_mut() {
            let unreleased = matches!(hold.get("released_at"), None | Some(Value::Null));
            if !unreleased {
                continue;
            }
            if !matches!(hold.get("holder"), Some(Value::String(s)) if s == id) {
                continue;
            }
            if let Value::Object(m) = hold {
                m.insert("released_at".into(), Value::String(released_at.clone()));
            }
            released += 1;
        }
    }
    if released > 0 {
        let _ = crate::fsutil::write_json_atomic(&file, &store);
    }
    guard.release();
}

/// The self-delete guard `teardown_worktree` runs before ever removing a
/// directory: `worktree_root` must never be `cwd`, nor an ancestor of it.
/// Pulled out as a pure predicate over an injected `cwd` so it is testable
/// without touching the real process directory — the one production call
/// site always supplies `std::env::current_dir()`.
pub(crate) fn assert_directory_removal_is_safe(cwd: Option<&Path>, worktree_root: &Path) {
    if let Some(cwd) = cwd {
        assert!(
            !cwd.starts_with(worktree_root),
            "teardown_worktree: refusing to remove {} — it contains the current directory ({})",
            p(worktree_root),
            p(cwd)
        );
    }
}

/// The two ways `teardown_worktree`'s directory half can refuse. Only
/// produced when `remove` is `Some` — the registry-only call `run_unregister`
/// makes never reaches either arm.
pub(crate) enum TeardownFailure {
    /// `git worktree remove --force` failed; nothing else ran.
    RemoveFailed(String),
    /// The directory is gone but `git branch -d` (NEVER -D) refused; the
    /// registry half below did not run.
    BranchDeleteFailed(String),
}

/// The five removal steps `performCleanup` used to run inline, lifted into
/// one shared helper (decisions D3, D3a): `git worktree remove --force`,
/// `git branch -d`, grant drop, `ws::unregister_workspace`,
/// `release_all_for_holder`.
///
/// `remove` is the explicit, non-default directory-removal parameter —
/// `Some((worktree_root, branch))` runs the directory and branch steps first
/// and only falls through to the registry half (grant, workspace record,
/// holds) on success; `None` reaches the registry half alone, which is how
/// `run_unregister` wires in without ever touching a directory.
pub(crate) fn teardown_worktree(
    main_root: &Path,
    id: &str,
    remove: Option<(&Path, &str)>,
) -> Result<(), TeardownFailure> {
    if let Some((worktree_root, branch)) = remove {
        assert_directory_removal_is_safe(std::env::current_dir().ok().as_deref(), worktree_root);

        let worktree_root_s = p(worktree_root);
        let remove_result = run_git(
            main_root,
            &["worktree", "remove", "--force", "--", &worktree_root_s],
        );
        if remove_result.status != Some(0) {
            return Err(TeardownFailure::RemoveFailed(remove_result.fail_text()));
        }

        let branch_delete = run_git(main_root, &["branch", "-d", "--", branch]);
        if branch_delete.status != Some(0) {
            return Err(TeardownFailure::BranchDeleteFailed(branch_delete.fail_text()));
        }
    }

    // The three best-effort ledger drops, in Node's order. Each is wrapped in
    // its own `try/catch` there, so every failure — including the ones this
    // port would otherwise call Exotic — is swallowed, not delegated.
    let main_store_root = main_root.join(".bee");
    if let Some(existing) = read_grants_strict(&main_store_root) {
        if existing.contains_key(id) {
            let mut next = existing;
            next.remove(id);
            let _ = write_grants_file_atomic(&main_store_root, &next);
        }
    }
    let _ = ws::unregister_workspace(main_root, id);
    release_all_for_holder(main_root, id);
    Ok(())
}

/// performCleanup (decision D8b): re-check freshness, then run the shared
/// teardown with directory removal on. Never throws — every outcome is the
/// `{ok, code?}` object folded into the merge result's `.cleanup` field, in
/// Node's exact key order. The `Map` construction stays here; only the
/// side-effect calls moved into `teardown_worktree`.
pub(crate) fn perform_cleanup(
    main_root: &Path,
    worktree_root: &Path,
    branch: &str,
    id: &str,
    verify_skipped: bool,
) -> Map<String, Value> {
    let mut out = Map::new();
    let status = match git_status_porcelain(worktree_root) {
        Ok(s) => s,
        Err(message) => {
            out.insert("ok".into(), Value::Bool(false));
            out.insert("code".into(), json!("WORKTREE_MERGE_CLEANUP_CHECK_FAILED"));
            out.insert("reason".into(), Value::String(message));
            return out;
        }
    };
    if !js_trim(&status).is_empty() {
        out.insert("ok".into(), Value::Bool(false));
        out.insert("code".into(), json!("WORKTREE_MERGE_CLEANUP_DIRTY"));
        out.insert("reason".into(), json!(format!(
            "{} has tracked-modified or untracked files at tracked paths — cleanup refuses. Remove them (a bootstrapped, gitignored .bee store alone does not block cleanup) and retry, or clean up manually.",
            p(worktree_root)
        )));
        out.insert("status".into(), Value::String(status));
        return out;
    }

    if let Err(failure) = teardown_worktree(main_root, id, Some((worktree_root, branch))) {
        match failure {
            TeardownFailure::RemoveFailed(reason) => {
                out.insert("ok".into(), Value::Bool(false));
                out.insert("code".into(), json!("WORKTREE_MERGE_CLEANUP_REMOVE_FAILED"));
                out.insert("reason".into(), Value::String(reason));
            }
            TeardownFailure::BranchDeleteFailed(reason) => {
                out.insert("ok".into(), Value::Bool(false));
                out.insert(
                    "code".into(),
                    json!("WORKTREE_MERGE_CLEANUP_BRANCH_DELETE_FAILED"),
                );
                out.insert("removed".into(), Value::Bool(true));
                out.insert("reason".into(), Value::String(reason));
            }
        }
        return out;
    }

    out.insert("ok".into(), Value::Bool(true));
    out.insert("removed".into(), Value::Bool(true));
    out.insert("branch_deleted".into(), Value::Bool(true));
    if verify_skipped {
        out.insert(
            "warning".into(),
            json!("verify skipped — no commands.test recorded; cleaned up unchecked."),
        );
    }
    out
}

/// attachCleanupOutcome — runs cleanup, or attaches the suggested command
/// (decision D8b: "never prompt").
pub(crate) fn attach_cleanup_outcome(
    result: &mut Map<String, Value>,
    main_root: &Path,
    worktree_root: &Path,
    branch: &str,
    id: &str,
    cleanup: bool,
    verify_skipped: bool,
) {
    if !cleanup {
        result.insert(
            "cleanup_suggested_command".into(),
            json!(format!("bee worktree merge --id {id} --cleanup --json")),
        );
        return;
    }
    result.insert(
        "cleanup".into(),
        Value::Object(perform_cleanup(
            main_root,
            worktree_root,
            branch,
            id,
            verify_skipped,
        )),
    );
}

/// checkMergeFence — P3's SECOND line of defense (the processor-lease epoch is
/// the first). A short drift description, or `None` when the fence is clean.
/// The two `.stdout.trim()` sites here carry the same unreachability argument
/// `main_untouched_proof` documents.
pub(crate) fn check_merge_fence(
    main_root: &Path,
    id: &str,
    pre_merge_head: &str,
    merge_head_file: &Path,
    staged_tree_hash: &str,
) -> Option<String> {
    let head_now = js_trim(&run_git(main_root, &["rev-parse", "HEAD"]).stdout.unwrap_or_default())
        .to_string();
    if head_now != pre_merge_head {
        return Some(format!(
            "HEAD moved from {pre_merge_head} to {head_now} while the lock was released for verify"
        ));
    }
    if !merge_head_file.exists() {
        return Some(
            ".git/MERGE_HEAD disappeared while the lock was released for verify — the staged merge was cleared out from under this operation"
                .to_string(),
        );
    }
    let tree_now =
        js_trim(&run_git(main_root, &["write-tree"]).stdout.unwrap_or_default()).to_string();
    if tree_now != staged_tree_hash {
        return Some(format!(
            "the staged tree changed from {staged_tree_hash} to {tree_now} while the lock was released for verify — the index was mutated mid-verify"
        ));
    }
    // readGrants swallows a parse error and reads `{}`; read_grants_strict
    // delegates instead — but `run_merge` already probed the registry before
    // any lock, so `None` here is a mid-merge race, treated as "revoked"
    // exactly like an absent entry would be.
    let granted = read_grants_strict(&main_root.join(".bee"))
        .map(|g| g.get(id) == Some(&Value::Bool(true)))
        .unwrap_or(false);
    if !granted {
        return Some(format!(
            "the grant for worktree id {} was revoked while the lock was released for verify",
            jsjson::stringify(&Value::String(id.to_string()))
        ));
    }
    None
}

// ─── the verify child (P2) ────────────────────────────────────────────────

/// Node's `spawn(command, { shell: true })` file/args, faithfully: on win32
/// `process.env.comspec || 'cmd.exe'` with `/d /s /c "<command>"` passed
/// VERBATIM; elsewhere `/bin/sh -c <command>`. Deliberately NOT
/// verbs/cells.rs's `spawn_declared`, which prefers Git Bash on win32 — that
/// is `runDeclaredTests`' shape, not `runVerifyChild`'s.
pub(crate) fn shell_child(command: &str) -> std::process::Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let file = std::env::var("comspec").unwrap_or_else(|_| "cmd.exe".to_string());
        let mut c = std::process::Command::new(file);
        c.raw_arg(format!("/d /s /c \"{command}\""));
        c
    }
    #[cfg(not(windows))]
    {
        let mut c = std::process::Command::new("/bin/sh");
        c.args(["-c", command]);
        c
    }
}

/// The pre-check that retires blocker (b). `runVerifyChild`'s ONLY V8/libuv
/// byte is the `error` event's message (`spawn cmd.exe ENOENT`,
/// `spawn /bin/sh ENOENT`), concatenated into `verifyOutcome.combined` and
/// surfaced verbatim in MERGE_VERIFY_RED's `output_tail` — a byte reached
/// AFTER the merge is staged, where nothing can fall back. That event fires
/// only when the SHELL ITSELF cannot be started, so probing the shell BEFORE
/// P1 ever stages anything makes the arm unreachable: a failed probe returns
/// None with zero mutations and zero locks taken, and a passing probe proves
/// the real spawn will launch.
pub(crate) fn shell_launchable() -> bool {
    shell_child("exit 0")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

pub(crate) struct VerifyOutcome {
    pub(crate) ran: bool,
    pub(crate) status: Option<i32>,
    pub(crate) combined: String,
}

/// runVerifyChild — the async-`spawn` verify, run UNLOCKED against the
/// merged-but-UNCOMMITTED tree, with `on_tick` firing on `tick_interval_ms`
/// for as long as the child is still running (integration-queue's processor-
/// lease renewal in production). A throwing tick is swallowed.
///
/// ONE DOCUMENTED DIVERGENCE, in the same class as verbs/state_group.rs's
/// prune approximation: Node resolves on the child's `exit` event, so output
/// still sitting in a pipe when the process exits can be LOST; this port joins
/// its reader threads, so it always captures the full stream. The difference
/// is observable only in a race Node does not reproduce run-to-run, and it can
/// only ever ADD trailing bytes to `output_tail` — never change `status`, the
/// red/green verdict, or any `.bee/` record.
pub(crate) fn run_verify_child(
    command: &str,
    cwd: &Path,
    on_tick: &dyn Fn(),
    tick_interval_ms: f64,
) -> VerifyOutcome {
    use std::io::Read;
    use std::sync::mpsc;

    let mut child = match shell_child(command)
        .current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        // Unreachable after `shell_launchable` (see its doc comment); if a
        // race gets here anyway, Node's `status: null` verdict is reproduced
        // and the error text is Rust's — the same narrow approximation
        // node_fs_error_message already makes elsewhere in this file.
        Err(e) => {
            return VerifyOutcome {
                ran: true,
                status: None,
                combined: format!("{e}"),
            }
        }
    };

    let drain = |mut pipe: Option<std::process::ChildStdout>| {
        let (tx, rx) = mpsc::channel::<String>();
        let handle = std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(pipe) = pipe.as_mut() {
                let _ = pipe.read_to_end(&mut buf);
            }
            let _ = tx.send(String::from_utf8_lossy(&buf).into_owned());
        });
        (handle, rx)
    };
    let (out_handle, out_rx) = drain(child.stdout.take());
    let (err_handle, err_rx) = {
        let mut pipe = child.stderr.take();
        let (tx, rx) = mpsc::channel::<String>();
        let handle = std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(pipe) = pipe.as_mut() {
                let _ = pipe.read_to_end(&mut buf);
            }
            let _ = tx.send(String::from_utf8_lossy(&buf).into_owned());
        });
        (handle, rx)
    };

    // `setInterval(tickIntervalMs)` for as long as the child is running. The
    // .mjs unref()s the timer so it can never keep the process alive; here the
    // poll is inline, so there is nothing to unref.
    let interval = std::time::Duration::from_millis(tick_interval_ms.max(1.0) as u64);
    let mut last_tick = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s.code(),
            Ok(None) => {}
            Err(_) => break None,
        }
        if last_tick.elapsed() >= interval {
            last_tick = Instant::now();
            on_tick();
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    };
    let stdout = out_rx.recv().unwrap_or_default();
    let stderr = err_rx.recv().unwrap_or_default();
    let _ = out_handle.join();
    let _ = err_handle.join();
    VerifyOutcome {
        ran: true,
        status,
        combined: format!("{stdout}{stderr}"),
    }
}
