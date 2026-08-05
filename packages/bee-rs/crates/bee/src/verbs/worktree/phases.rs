// the merge phases P1/P3
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

// ─── P1 / P3 ──────────────────────────────────────────────────────────────

/// Everything P3 needs, carried across the released lock.
pub(crate) struct Staged {
    pub(crate) id: String,
    pub(crate) branch: String,
    pub(crate) worktree_root: PathBuf,
    pub(crate) merge_message: String,
    pub(crate) pre_merge_head: String,
    pub(crate) merge_head_file: PathBuf,
    pub(crate) staged_tree_hash: String,
    /// teardownCompanionIfPresent's `{ended, sessionId, warning?}`, carried
    /// across the released lock so P3's results can spread it too.
    pub(crate) companion: Option<Value>,
}

pub(crate) enum StageOut {
    /// A TERMINAL outcome already fully resolved inside the P1 lock.
    Done(MergeAnswer),
    Staged(Box<Staged>),
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn merge_stage(
    main_root: &Path,
    id: &str,
    companion_end_command: Option<&str>,
) -> MR<StageOut> {
    // `typeof id !== 'string' || !id` is already enforced by run_merge's
    // requireFlag gate, so WORKTREE_MERGE_INVALID_ID is unreachable here.
    if !is_ordinary_checkout(main_root) {
        return Err(refuse_merge(
            "WORKTREE_MERGE_CALLER_NOT_ORDINARY",
            format!(
                "\"bee worktree merge\" must be run from the MAIN checkout, not a linked worktree ({} is not an ordinary checkout) — a worktree, including the one being merged, cannot merge itself.",
                p(main_root)
            ),
        ));
    }

    let main_store_root = main_root.join(".bee");
    let grants = read_grants_strict(&main_store_root).ok_or(MErr::Ex)?;
    let id_json = jsjson::stringify(&Value::String(id.to_string()));
    if grants.get(id) != Some(&Value::Bool(true)) {
        return Err(refuse_merge(
            "WORKTREE_MERGE_UNKNOWN_ID",
            format!(
                "no granted worktree found for id {id_json} — run \"bee worktree list\" to see granted ids."
            ),
        ));
    }

    let resolved = resolve_worktree_by_id(main_root, id).filter(|r| r.exists());
    let Some(worktree_root) = resolved else {
        return Err(refuse_merge(
            "WORKTREE_MERGE_UNKNOWN_ID",
            format!(
                "id {id_json} is granted but no matching, bidirectionally-valid git worktree link was found under {} (or the worktree no longer exists on disk) — run \"git worktree prune\" and \"bee worktree unregister --id {id}\" if it was removed by hand.",
                p(main_root)
            ),
        ));
    };

    // worktree-companion-hook: READ (never delete) the marker up front so its
    // mountPath can be excluded from the worktree dirty-check right below via
    // a git pathspec. Actual teardown is deferred until every zero-mutation
    // refusal in this function has cleared — see teardown_companion_if_present
    // for the full ordering rationale.
    let companion_marker = read_companion_marker(&worktree_root);

    if is_tree_dirty(main_root).map_err(MErr::Thrown)? {
        return Err(refuse_merge(
            "WORKTREE_MERGE_MAIN_DIRTY",
            format!(
                "the MAIN checkout at {} has uncommitted changes (\"git status --porcelain\" is non-empty) — commit or stash before merging.",
                p(main_root)
            ),
        ));
    }
    // A present companion mount AND its marker file are both untracked (and
    // the marker, unlike the rest of a bootstrapped `.bee` store, is not
    // gitignored either) — either alone would trip this check, so both are
    // excluded by git pathspec rather than by deletion-before-check.
    let worktree_dirty = match &companion_marker {
        Some(marker) => is_tree_dirty_excluding(
            &worktree_root,
            &[companion_mount_path(marker)?, companion_marker_rel()],
        ),
        None => is_tree_dirty(&worktree_root),
    };
    if worktree_dirty.map_err(MErr::Thrown)? {
        return Err(refuse_merge(
            "WORKTREE_MERGE_WORKTREE_DIRTY",
            format!(
                "the worktree at {} has uncommitted changes (\"git status --porcelain\" is non-empty) — commit or stash before merging. (A bootstrapped, gitignored .bee store alone is NOT dirty, per decision D8a.)",
                p(&worktree_root)
            ),
        ));
    }

    let Some(branch) = current_branch(&worktree_root) else {
        return Err(refuse_merge(
            "WORKTREE_MERGE_DETACHED_HEAD",
            format!(
                "the worktree at {} is on a detached HEAD — check out its branch before merging.",
                p(&worktree_root)
            ),
        ));
    };

    let identity = resolve_worktree_feature(&worktree_root);
    let expected_branch = identity.feature.as_ref().map(|f| format!("wt/{f}"));
    let branch_ok = match &expected_branch {
        Some(expected) => branch == *expected,
        None => wt_branch_shaped(&branch),
    };
    if !branch_ok {
        // issues-46-53 D4 (#46): name the field that actually drifted.
        let expected_disp = expected_branch.clone().unwrap_or_default();
        let drift = match (&identity.created, &identity.state_feature) {
            (Some(created), Some(state)) if created != state => format!(
                " This worktree was CREATED as feature {} (its immutable creation slug, which \"{expected_disp}\" comes from), while its .bee/state.json now records feature {} — the FEATURE FIELD drifted after creation (a rename, \"bee state set --feature\", or a new feature started in this worktree); the branch did not. Do NOT rename the branch to match: check \"{expected_disp}\" back out in the worktree, or merge the branch you actually want by hand.",
                jsjson::stringify(&Value::String(created.clone())),
                jsjson::stringify(&Value::String(state.clone())),
            ),
            (None, Some(state)) => format!(
                " \"{expected_disp}\" is derived from this worktree's MUTABLE .bee/state.json \"feature\" field ({}) because the worktree predates bee's immutable creation-slug record — if the feature was renamed after the worktree was created, that FIELD is what drifted, not the branch. The branch name is fixed at creation; do not rename it to match.",
                jsjson::stringify(&Value::String(state.clone())),
            ),
            _ => String::new(),
        };
        let expected_phrase = match &expected_branch {
            Some(e) => format!("\"{e}\""),
            None => "\"wt/<slug>\"-style".to_string(),
        };
        return Err(refuse_merge(
            "WORKTREE_MERGE_BRANCH_MISMATCH",
            format!(
                "the worktree at {} is checked out to \"{branch}\", not its expected {expected_phrase} branch — merge refuses to guess which branch to consume.{drift}",
                p(&worktree_root)
            ),
        ));
    }

    // worktree-companion-hook: every zero-mutation refusal above (both
    // dirty-tree checks, detached-HEAD, branch-mismatch) has now cleared, so
    // it is safe to tear the companion down. It cannot run any earlier (that
    // would destroy the mount even for a merge about to be refused) or any
    // later (the companion session must not outlive a merge attempt that is
    // actually proceeding). On a COMPANION worktree this — not the staged
    // merge — is the first real mutation, so NOTHING from this line on may
    // return MErr::Ex; run_merge's read-only probes are what keep that true.
    let companion = teardown_companion_if_present(
        main_root,
        &worktree_root,
        companion_end_command,
        companion_marker.as_ref(),
    );

    // ── every REFUSAL above is zero-mutation; the staged merge below is the
    // first write to MAIN. ────────────────────────────────────────────────
    let pre_merge_head =
        js_trim(&run_git(main_root, &["rev-parse", "HEAD"]).stdout.unwrap_or_default()).to_string();
    let merge_head_file = main_root.join(".git").join("MERGE_HEAD");
    let merge_message = format!("Merge worktree {id} (branch {branch}) via bee worktree merge");

    let merge_result = run_git(main_root, &["merge", "--no-ff", "--no-commit", "--", &branch]);
    if merge_result.status != Some(0) {
        run_git(main_root, &["merge", "--abort"]);
        if let Err(reason) = main_untouched_proof(main_root, &pre_merge_head, &merge_head_file) {
            return Err(refuse_merge(
                "WORKTREE_MERGE_ABORT_FAILED",
                format!(
                    "\"git merge --no-ff --no-commit {branch}\" failed and \"git merge --abort\" did NOT fully restore {} to its pre-merge state ({reason}) — main may be left mid-merge; inspect it by hand before retrying.",
                    p(main_root)
                ),
            ));
        }
        let mut result = Map::new();
        result.insert("ok".into(), Value::Bool(false));
        result.insert("code".into(), json!("MERGE_CONFLICT"));
        result.insert("id".into(), json!(id));
        result.insert("branch".into(), json!(branch));
        result.insert("worktreeRoot".into(), json!(p(&worktree_root)));
        result.insert("message".into(), json!(format!(
            "\"git merge --no-ff {branch}\" hit a textual conflict — the merge was aborted and {} was left byte-untouched (HEAD unchanged, no MERGE_HEAD, clean tracked status); bee does not auto-resolve a textual conflict.",
            p(main_root)
        )));
        result.insert("output".into(), json!(format!(
            "{}{}",
            merge_result.stdout.clone().unwrap_or_default(),
            merge_result.stderr.clone().unwrap_or_default()
        )));
        // `...(companion ? { companion } : {})` — last, after `output`.
        if let Some(companion) = companion {
            result.insert("companion".into(), companion);
        }
        return Ok(StageOut::Done(MergeAnswer { result, ok: false }));
    }

    if !merge_head_file.exists() {
        // Zero exit but nothing staged: "Already up to date".
        let mut result = Map::new();
        result.insert("ok".into(), Value::Bool(true));
        result.insert("merged".into(), Value::Bool(false));
        result.insert("id".into(), json!(id));
        result.insert("branch".into(), json!(branch));
        result.insert("worktreeRoot".into(), json!(p(&worktree_root)));
        result.insert("code".into(), json!("ALREADY_UP_TO_DATE"));
        result.insert("verify".into(), json!("skipped"));
        result.insert("message".into(), json!(format!(
            "\"{branch}\" is already up to date with {} — nothing to merge.",
            p(main_root)
        )));
        // `...(companion ? { companion } : {})` — after `message`, BEFORE the
        // cleanup keys attachCleanupOutcome appends next.
        if let Some(companion) = companion {
            result.insert("companion".into(), companion);
        }
        // verifySkipped is deliberately FALSE here (see the .mjs comment).
        //
        // D1a: cleanup-by-default fires only on a merge that merged
        // something. This arm merged nothing (`merge_head_file` never
        // existed — the branch was already fully contained in `base`), so
        // cleanup is hardcoded FALSE here regardless of the caller's
        // `cleanup` value — never the passed-through flag/config decision.
        // Otherwise re-running merge to check status would delete the
        // worktree, and a tree holding uncommitted gitignored work would
        // read as clean-and-trivially-merged.
        attach_cleanup_outcome(
            &mut result,
            main_root,
            &worktree_root,
            &branch,
            id,
            false,
            false,
        );
        return Ok(StageOut::Done(MergeAnswer { result, ok: true }));
    }

    let staged_tree_hash =
        js_trim(&run_git(main_root, &["write-tree"]).stdout.unwrap_or_default()).to_string();
    Ok(StageOut::Staged(Box::new(Staged {
        id: id.to_string(),
        branch,
        worktree_root,
        merge_message,
        pre_merge_head,
        merge_head_file,
        staged_tree_hash,
        companion,
    })))
}

/// P3: verify-red first, then the two-line fence, then commit + post-commit
/// guard + cleanup. `lease_drift` is the caller-supplied FIRST fence line
/// (integration-queue's `checkProcessorLeaseEpoch`), evaluated here so it runs
/// inside the re-acquired hold exactly as the .mjs's `await
/// checkProcessorLease()` does.
pub(crate) fn merge_finish(
    main_root: &Path,
    state: &Staged,
    cleanup: bool,
    verify: &VerifyOutcome,
    lease_fence: &dyn Fn() -> Option<String>,
) -> MR<MergeAnswer> {
    let Staged {
        id,
        branch,
        worktree_root,
        merge_message,
        pre_merge_head,
        merge_head_file,
        staged_tree_hash,
        companion,
    } = state;

    let mut committed = false;
    let outcome = (|| -> MR<MergeAnswer> {
        if verify.ran && verify.status != Some(0) {
            let lines: Vec<&str> = verify.combined.split('\n').collect();
            let tail = lines[lines.len().saturating_sub(30)..].join("\n");
            run_git(main_root, &["merge", "--abort"]);
            if let Err(reason) = main_untouched_proof(main_root, pre_merge_head, merge_head_file) {
                return Err(refuse_merge(
                    "WORKTREE_MERGE_ABORT_FAILED",
                    format!(
                        "verify failed and \"git merge --abort\" did NOT fully restore {} to its pre-merge state ({reason}) — main may be left mid-merge; inspect it by hand before retrying.",
                        p(main_root)
                    ),
                ));
            }
            let mut result = Map::new();
            result.insert("ok".into(), Value::Bool(false));
            result.insert("code".into(), json!("MERGE_VERIFY_RED"));
            result.insert("id".into(), json!(id));
            result.insert("branch".into(), json!(branch));
            result.insert("worktreeRoot".into(), json!(p(worktree_root)));
            result.insert("merged".into(), Value::Bool(false));
            result.insert("verify".into(), json!("red"));
            result.insert("message".into(), json!(format!(
                "the merge was textually clean but the post-merge verify failed against the merged-but-uncommitted tree — this is the semantic-conflict alarm: behavior broke even though git found no textual conflict. The merge was aborted and {} was left byte-untouched (HEAD unchanged, no MERGE_HEAD, clean tracked status); no merge commit exists. Fix-first before release.",
                p(main_root)
            )));
            result.insert("output_tail".into(), Value::String(tail));
            // `...(companion ? { companion } : {})` — last, after output_tail.
            if let Some(companion) = companion {
                result.insert("companion".into(), companion.clone());
            }
            return Ok(MergeAnswer { result, ok: false });
        }

        // The P3 fence: the processor-lease epoch is the FIRST line, and
        // `||` short-circuits, so checkMergeFence's git reads never run when
        // the lease already drifted.
        let fence_drift = lease_fence().or_else(|| {
            check_merge_fence(main_root, id, pre_merge_head, merge_head_file, staged_tree_hash)
        });
        if let Some(fence_drift) = fence_drift {
            run_git(main_root, &["merge", "--abort"]);
            if let Err(reason) = main_untouched_proof(main_root, pre_merge_head, merge_head_file) {
                return Err(refuse_merge(
                    "WORKTREE_MERGE_ABORT_FAILED",
                    format!(
                        "the P3 fence detected drift ({fence_drift}) and \"git merge --abort\" did NOT fully restore {} to its pre-merge state ({reason}) — main may be left mid-merge; inspect it by hand before retrying.",
                        p(main_root)
                    ),
                ));
            }
            return Err(refuse_merge(
                "WORKTREE_MERGE_FENCE_DRIFT",
                format!(
                    "the staged merge was aborted before commit because the P3 re-check (advisor condition C2) detected drift while the 'worktree-admin' lock was released around the verify child: {fence_drift}. {} was left byte-untouched (HEAD unchanged, no MERGE_HEAD, clean tracked status); no merge commit exists.",
                    p(main_root)
                ),
            ));
        }

        let commit_result = run_git(main_root, &["commit", "-m", merge_message]);
        if commit_result.status != Some(0) {
            run_git(main_root, &["merge", "--abort"]);
            return Err(refuse_merge(
                "WORKTREE_MERGE_COMMIT_FAILED",
                format!(
                    "\"git commit\" failed for the staged merge of {branch} ({}) — the staged merge was aborted; {} was left untouched.",
                    commit_result.fail_text(),
                    p(main_root)
                ),
            ));
        }
        committed = true;

        let mut result = Map::new();
        result.insert("ok".into(), Value::Bool(true));
        result.insert("merged".into(), Value::Bool(true));
        result.insert("id".into(), json!(id));
        result.insert("branch".into(), json!(branch));
        result.insert("worktreeRoot".into(), json!(p(worktree_root)));
        let verify_field = if verify.ran { "green" } else { "skipped" };
        result.insert("verify".into(), json!(verify_field));
        // `...(companion ? { companion } : {})` — after `verify`, BEFORE the
        // post-commit `warning` and the cleanup keys.
        if let Some(companion) = companion {
            result.insert("companion".into(), companion.clone());
        }

        // Post-commit guard (D2-REVISED).
        let post_commit_status = run_git(
            main_root,
            &["status", "--porcelain", "--untracked-files=no"],
        )
        .stdout
        .unwrap_or_default();
        if !js_trim(&post_commit_status).is_empty() {
            result.insert("warning".into(), json!({
                "code": "verify_mutated_tracked_files",
                "message": "the post-merge verify command left tracked files modified after the merge commit landed (\"git status --porcelain --untracked-files=no\" is non-empty) — the merge commit itself is clean, but verify mutated the working tree afterward; inspect and commit/discard those changes separately. Recovery if a LATER independent verify goes red on this merge: \"git revert -m 1 <merge-commit>\".",
                "status": post_commit_status,
            }));
        }

        attach_cleanup_outcome(
            &mut result,
            main_root,
            worktree_root,
            branch,
            id,
            cleanup,
            verify_field == "skipped",
        );
        Ok(MergeAnswer { result, ok: true })
    })();

    // The `finally` safety net: an unexpected exit that never committed and
    // never aborted must not strand a staged merge on main.
    if !committed && merge_head_file.exists() {
        run_git(main_root, &["merge", "--abort"]);
    }
    outcome
}

/// mergeFeatureWorktree — P1 / (P2) / P3 with the lock released across the
/// verify child. `Err(MErr::Ex)` is only ever produced by P1, before any
/// mutation; the caller has already taken the queue lock by then, so it is the
/// documented late-delegation residual, never an ordinary shape.
pub(crate) fn merge_feature_worktree(
    main_root: &Path,
    id: &str,
    cleanup: bool,
    verify_command: Option<&str>,
    companion_end_command: Option<&str>,
    hooks: Option<&crate::integration_queue::Hooks<'_>>,
) -> MR<MergeAnswer> {
    let mut guard = lock::acquire_store_lock(main_root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS)
        .map_err(|b| MErr::Thrown(b.message()))?;
    let staged = merge_stage(main_root, id, companion_end_command);
    guard.release();
    let staged = match staged? {
        StageOut::Done(answer) => return Ok(answer),
        StageOut::Staged(s) => s,
    };

    let no_lease_drift = || None;
    let lease_fence: &dyn Fn() -> Option<String> = match hooks {
        Some(h) => &move || h.check_processor_lease(),
        None => &no_lease_drift,
    };

    let Some(command) = verify_command else {
        // Nothing to release the lock around — stage and finish inside the
        // SAME shape as the pre-hardening-4c single-lock behavior, which is
        // still TWO acquires (the .mjs re-enters withStoreLock here).
        let mut guard = lock::acquire_store_lock(main_root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS)
            .map_err(|b| MErr::Thrown(b.message()))?;
        let out = merge_finish(
            main_root,
            &staged,
            cleanup,
            &VerifyOutcome { ran: false, status: None, combined: String::new() },
            lease_fence,
        );
        guard.release();
        return out;
    };

    // P2 — UNLOCKED.
    let tick_ms = hooks
        .map(|h| h.verify_tick_interval_ms)
        .unwrap_or(crate::integration_queue::DEFAULT_RENEW_INTERVAL_MS);
    let no_tick = || {};
    let tick: &dyn Fn() = match hooks {
        Some(h) => &move || h.on_verify_tick(),
        None => &no_tick,
    };
    let verify = run_verify_child(command, main_root, tick, tick_ms);

    // P3 — re-acquire and re-check the fence before ever committing.
    let mut guard = lock::acquire_store_lock(main_root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS)
        .map_err(|b| MErr::Thrown(b.message()))?;
    let out = merge_finish(main_root, &staged, cleanup, &verify, lease_fence);
    guard.release();
    out
}
