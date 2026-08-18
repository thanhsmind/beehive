// the merge phases P1/P3
//
// Split out of the single 4.2k-line verbs/worktree.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_roots_core, Resolution};
use crate::verbs::reservations::{js_numberify, js_trim, now_iso, parse_flags, Err2, FlagV, Flags};
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
    /// trun-4: the pre-merge `.bee` (+ `docs/history/<feature>`) bookkeeping
    /// auto-commit's outcome, if it ran — carried across the released lock
    /// the same way `companion` is, so P3's results can attach it too.
    pub(crate) bookkeeping_commit: Option<Value>,
    /// D7/D8 (td-3): the D8 proof-check verdict `merge_stage` already read
    /// as a zero-mutation precondition (never blocking here — a blocking
    /// verdict refused before `Staged` was ever built), carried across the
    /// released lock so `merge_finish` can report it honestly instead of a
    /// `VerifyOutcome` from a test run that no longer happens. `None` only
    /// when the worktree's feature could not be resolved.
    pub(crate) proof: Option<crate::verbs::cells::ProofCheck>,
    /// mcl-2: `identity.feature`, carried the same way `proof` is, so P3 can
    /// close the merged feature's lane without re-resolving the identity a
    /// second time. `None` only when the worktree's feature could not be
    /// resolved — matches `proof`'s own `None` condition exactly.
    pub(crate) feature: Option<String>,
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
    skip_uat: bool,
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
    let id_json = jsjson::stringify(&Value::String(id.to_string()));

    // staging-lane D0 (teeth #1): the staging branch/worktree can NEVER land
    // into main via `bee worktree merge` — checked before the grant lookup
    // below (staging is never granted, so "no granted worktree" would
    // otherwise be the misleading message a staging merge attempt hits) and
    // before ANY mutation, zero-mutation, NO escape flag: staging is a
    // disposable mixing ground, never a source of truth (D0/D0a); its
    // history is garbage that must never land. Matched by branch name
    // and/or worktree root — either alone is enough, since `id` on this CLI
    // surface is the git-internal worktree dir name, not always the branch.
    if let Ok(Some(staging_record)) = crate::verbs::staging::read_staging_record(main_root) {
        let by_branch = id == staging_record.branch;
        let by_worktree_root = resolve_worktree_by_id(main_root, id).is_some_and(|root| {
            crate::path_identity::canonical_paths_equal(&root, &staging_record.worktree_root)
        });
        if by_branch || by_worktree_root {
            return Err(refuse_merge(
                "WORKTREE_MERGE_STAGING_FORBIDDEN",
                format!(
                    "{id_json} names the staging worktree/branch \u{2014} \"bee worktree merge\" refuses to land it into main, with no escape flag (staging-lane D0: staging is a disposable mixing ground, never a source of truth; its history is garbage that never lands). FIX: merge the FEATURE branch itself after its \"uat\" gate is approved \u{2014} never staging."
                ),
            ));
        }
    }

    let grants = read_grants_strict(&main_store_root).ok_or(MErr::Ex)?;
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

    // resolveWorktreeFeature, computed here (not just for the branch check
    // further down) because trun-4's pre-merge bookkeeping auto-commit needs
    // the MERGING feature's slug to scope its docs/history/<feature>
    // pathspec.
    let identity = resolve_worktree_feature(&worktree_root);

    // D7/D8 (docs/history/test-doctrine/CONTEXT.md, td-3): `bee worktree
    // merge` no longer spawns `commands.test` itself — this is the tests
    // door's own zero-mutation precondition, reading whether every capped
    // cell for the merging feature already carries a recorded D8 proof
    // line (verbs/cells/proof.rs `feature_proof_check`, the SAME helper
    // `bee close`'s tests door reads, td-2). Checked here, before the
    // main-dirty bookkeeping auto-commit below, so a proof-less merge
    // never touches main at all — the merge is refused, never staged. A
    // worktree whose feature cannot be resolved has no cell record to
    // check (the same posture `uat_merge_precheck` takes for a `None`
    // feature further down) and proceeds ungated.
    let proof = match identity.feature.as_deref() {
        Some(feature) => Some(
            crate::verbs::cells::feature_proof_check(main_root, feature)
                .map_err(|_: crate::verbs::drivers::Delegate| MErr::Ex)?,
        ),
        None => None,
    };
    if let Some(proof) = &proof {
        if proof.blocking {
            let feature_disp = identity.feature.as_deref().unwrap_or("(unresolved)");
            return Err(refuse_merge(
                "WORKTREE_MERGE_PROOF_DEBT",
                format!(
                    "worktree {id}'s feature \"{feature_disp}\" has {} capped cell(s) that carry a report with no valid proof line ({}) — \"bee worktree merge\" refuses until each is re-capped with a real proof line: \"<command> — <result> — <scope reason>\" (bee cells finish --id <id> --report '{{...}}').",
                    proof.bad_ids.len(),
                    proof.bad_ids.join(", "),
                ),
            ));
        }
    }

    // trun-4: `bee worktree merge` used to refuse on ANY dirty path in main,
    // but the dirt is routinely bee's OWN bookkeeping (cell traces,
    // decisions, backlog under .bee/, and this feature's own
    // docs/history/<feature>/ artifacts) written by the orchestrator's
    // normal state calls during the slice — committing exactly that dirt BY
    // HAND from main was refused by the worktree-first guard, deadlocking a
    // green slice against its own landing step. Mirrors bee close's own
    // bee-store bookkeeping auto-commit (drivers/close.rs,
    // commit_close_bookkeeping): before the refusal fires, auto-commit ONLY
    // the two allowed roots (merge.rs's main_bookkeeping_roots — never
    // wider) and proceed; any dirt OUTSIDE them still refuses, named by
    // path, exactly as before.
    let mut bookkeeping_commit: Option<Value> = None;
    if is_tree_dirty(main_root).map_err(MErr::Thrown)? {
        if !worktree_merge_commit_bookkeeping_enabled(main_root) {
            return Err(refuse_merge(
                "WORKTREE_MERGE_MAIN_DIRTY",
                format!(
                    "the MAIN checkout at {} has uncommitted changes (\"git status --porcelain\" is non-empty) — commit or stash before merging.",
                    p(main_root)
                ),
            ));
        }
        let roots = main_bookkeeping_roots(identity.feature.as_deref());
        if is_tree_dirty_excluding(main_root, &roots).map_err(MErr::Thrown)? {
            let offending = git_status_porcelain_excluding_untracked_all(main_root, &roots).map_err(MErr::Thrown)?;
            let scope = match &identity.feature {
                Some(f) => format!(".bee/, docs/decisions/, docs/knowledge/, or docs/history/{f}/"),
                None => ".bee/, docs/decisions/, or docs/knowledge/".to_string(),
            };
            return Err(refuse_merge(
                "WORKTREE_MERGE_MAIN_DIRTY",
                format!(
                    "the MAIN checkout at {} has uncommitted changes outside {scope} that \"bee worktree merge\" will not auto-commit — commit or stash them before merging:\n{offending}",
                    p(main_root)
                ),
            ));
        }
        // Warn-never-block (same contract close's own bookkeeping commit
        // keeps): whatever this returns, the merge proceeds — a leftover
        // .bee/docs-history dirt that could not be tidied is not a reason to
        // keep refusing forever.
        let message = match &identity.feature {
            Some(f) => format!(
                "Auto-commit .bee, docs/decisions, docs/knowledge, and docs/history/{f} bookkeeping before merging worktree {id}"
            ),
            None => format!("Auto-commit .bee, docs/decisions, and docs/knowledge bookkeeping before merging worktree {id}"),
        };
        bookkeeping_commit = Some(commit_main_bookkeeping(main_root, &message, &roots).value());
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

    // `identity` was already resolved above, before the main-dirty check,
    // for the bookkeeping auto-commit's docs/history/<feature> scope.
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

    // uat-gate-before-merge D1: the LAST zero-mutation precondition — a
    // standard/high-risk feature whose "uat" gate is not yet approved
    // refuses here, before the companion is torn down or anything else
    // mutates. `identity` was already resolved above (line ~97) for the
    // bookkeeping auto-commit's scope; reused here rather than re-resolved.
    // `uat_before_merge_config`'s `None` (a present-but-non-boolean config
    // value) refuses UNCONDITIONALLY, before the lane/gate reads even run —
    // a typo'd config must never silently resolve to either outcome, on ANY
    // merge, not just the ones that would otherwise be gated.
    let Some(uat_before_merge) = uat_before_merge_config(main_root) else {
        return Err(refuse_merge(
            "WORKTREE_MERGE_UAT_CONFIG_INVALID",
            format!(
                "\"uat_before_merge\" in {}/.bee/config.json must be a boolean (true or false) \u{2014} merge refuses rather than guess which way to resolve it. FIX: set it to true or false, or remove the key entirely to use the default (true, uat-gate-before-merge D1).",
                p(main_root)
            ),
        ));
    };
    if uat_before_merge && !skip_uat {
        let precheck = uat_merge_precheck(main_root, identity.feature.as_deref());
        if precheck.lane_applies && !precheck.gate_approved {
            let feature_disp = identity.feature.as_deref().unwrap_or("(unresolved)");
            let lane_tail = identity
                .feature
                .as_deref()
                .map(|f| format!(" --lane {f}"))
                .unwrap_or_default();
            return Err(refuse_merge(
                "WORKTREE_MERGE_UAT_PENDING",
                format!(
                    "worktree {id}'s feature \"{feature_disp}\" has not been approved for the \"uat\" gate, and its lane is standard/high-risk (a missing or unrecognized lane classification fails closed the same way, uat-gate-before-merge D1) \u{2014} \"bee worktree merge\" refuses until the user accepts it, exactly once, or the repo opts out. FIX: approve it (\"bee gate --name uat --approved true{lane_tail}\"), or skip uat for JUST this merge (\"bee worktree merge --id {id} --skip-uat\"), or turn the door off repo-wide (set \"uat_before_merge\": false in {}/.bee/config.json).",
                    p(main_root)
                ),
            ));
        }
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
        // trun-4: the pre-merge bookkeeping auto-commit, if it ran, already
        // landed on main before this merge was even attempted — report it
        // regardless of how the merge attempt itself came out.
        if let Some(bookkeeping_commit) = bookkeeping_commit.clone() {
            result.insert("bookkeeping_commit".into(), bookkeeping_commit);
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
        // trun-4: same as the MERGE_CONFLICT arm above — the bookkeeping
        // auto-commit, if it ran, already landed before this arm was even
        // reached.
        if let Some(bookkeeping_commit) = bookkeeping_commit.clone() {
            result.insert("bookkeeping_commit".into(), bookkeeping_commit);
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
        bookkeeping_commit,
        proof,
        feature: identity.feature.clone(),
    })))
}

/// mcl-2 (R1): the merge-time half of "a shipped feature stops reading
/// waiting on you". Clears the merging feature's stranded `waiting_on` +
/// `run_state` pair through mcl-1's shared helper (`clear_lane_waiting_on_pair`,
/// `state_group/waiting_on.rs` — read, never re-implemented here) and
/// rewrites the lane's `next_action` to name the close road. Returns the
/// written `next_action` text, or an empty string when the feature has no
/// lane file at all (nothing to rewrite — not an error). Every failure
/// surfaces as `Err` so the one caller can warn and keep the merge green;
/// this function itself never decides warn-vs-fail.
fn close_the_lane_on_merge(main_root: &Path, feature: &str, merge_commit_sha: &str) -> Result<String, String> {
    crate::verbs::state_group::clear_lane_waiting_on_pair(main_root, feature)
        .map_err(lane_write_err_text)?;
    let Some(mut lane) =
        crate::verbs::workflow_store::read_lane_strict(main_root, feature).map_err(lane_write_err_text)?
    else {
        return Ok(String::new());
    };
    let short_sha = &merge_commit_sha[..merge_commit_sha.len().min(7)];
    let next_action = format!(
        "Merged into main at {short_sha}; capture what settled, then bee close --feature {feature}."
    );
    lane.insert("next_action".into(), json!(next_action.clone()));
    crate::verbs::workflow_store::write_lane(main_root, &lane).map_err(lane_write_err_text)?;
    Ok(next_action)
}

/// `Err2`'s two shapes, rendered as one warn-line message — `close_the_lane_on_merge`'s only consumer.
fn lane_write_err_text(e: Err2) -> String {
    match e {
        Err2::Msg(m) => m,
        Err2::Ex => "an unrecognized lane-record shape".to_string(),
    }
}

/// P3: the two-line fence, then commit + post-commit guard + cleanup.
/// `lease_drift` is the caller-supplied FIRST fence line (integration-
/// queue's `checkProcessorLeaseEpoch`), evaluated here so it runs inside the
/// re-acquired hold exactly as the .mjs's `await checkProcessorLease()`
/// does. D7/D8 (td-3): there is no more "verify-red first" arm — the D8
/// proof check already ran as a zero-mutation precondition in `merge_stage`
/// (P1), before `git merge` was ever attempted, so a blocking verdict never
/// reaches here; `state.proof` carries only the CLEARED verdict, for
/// reporting.
pub(crate) fn merge_finish(
    main_root: &Path,
    state: &Staged,
    cleanup: bool,
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
        bookkeeping_commit,
        proof,
        feature,
    } = state;

    let mut committed = false;
    let outcome = (|| -> MR<MergeAnswer> {
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
                    "the staged merge was aborted before commit because the P3 re-check (advisor condition C2) detected drift while the 'worktree-admin' lock was released between staging and finishing: {fence_drift}. {} was left byte-untouched (HEAD unchanged, no MERGE_HEAD, clean tracked status); no merge commit exists.",
                    p(main_root)
                ),
            ));
        }

        // `--no-gpg-sign`: this merge commit runs while the worktree-admin
        // lock is held, so a repo with `commit.gpgsign true` and a tty
        // pinentry configured must never be able to block the merge on a
        // signing prompt — same defense `close.rs`'s bee-store bookkeeping
        // commit keeps (B-P1-2), now through the one shared helper both call
        // (B-P2-1). `commit_unsigned` always spawns `git` with stdin null,
        // so even an unexpected prompt has nowhere to read from.
        let commit_result = commit_unsigned(main_root, merge_message, &[]);
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
        // wkm-1 (D1): the sha the keep-path deferred-queue entry names, read
        // right after the merge commit lands.
        let merge_commit_sha =
            js_trim(&run_git(main_root, &["rev-parse", "HEAD"]).stdout.unwrap_or_default())
                .to_string();

        let mut result = Map::new();
        result.insert("ok".into(), Value::Bool(true));
        result.insert("merged".into(), Value::Bool(true));
        result.insert("id".into(), json!(id));
        result.insert("branch".into(), json!(branch));
        result.insert("worktreeRoot".into(), json!(p(worktree_root)));
        let verify_field = proof_report_field(proof.as_ref());
        result.insert("verify".into(), json!(verify_field));
        // `...(companion ? { companion } : {})` — after `verify`, BEFORE the
        // post-commit `warning` and the cleanup keys.
        if let Some(companion) = companion {
            result.insert("companion".into(), companion.clone());
        }
        // trun-4: the bookkeeping auto-commit, if it ran, already landed
        // before P1 ever staged this merge — report it on the success path
        // too.
        if let Some(bookkeeping_commit) = bookkeeping_commit {
            result.insert("bookkeeping_commit".into(), bookkeeping_commit.clone());
        }

        // Post-commit guard (D2-REVISED). Runs BEFORE the lane write below
        // (mcl-2 R2): the guard measures the whole tree honestly, and the
        // lane write is itself a tracked-file mutation that lands right
        // after the merge commit — reading `git status` first keeps this
        // warning about genuine post-commit drift (a hook, a concurrent
        // process), not about the lane rewrite this function is about to
        // make on its own behalf.
        let post_commit_status = run_git(
            main_root,
            &["status", "--porcelain", "--untracked-files=no"],
        )
        .stdout
        .unwrap_or_default();
        if !js_trim(&post_commit_status).is_empty() {
            result.insert("warning".into(), json!({
                "code": "verify_mutated_tracked_files",
                "message": "tracked files are modified after the merge commit landed (\"git status --porcelain --untracked-files=no\" is non-empty) — the merge commit itself is clean, but something (a git hook, a concurrent process) touched the working tree afterward; inspect and commit/discard those changes separately. D7/D8: this is no longer the post-merge verify command (bee worktree merge does not run one), so this warning is now defensive rather than expected.",
                "status": post_commit_status,
            }));
        }

        // mcl-2 (R1): this is a real, green merge (the ALREADY_UP_TO_DATE
        // arm returns long before `Staged` is ever built, so every path
        // that reaches here actually merged something) — the event that
        // answers the merged feature's stranded uat mark. Clear the pair
        // mcl-1's shared helper owns and rewrite the lane's `next_action` to
        // name the close road. BEST-EFFORT and post-commit, same
        // warn-never-block contract the bookkeeping auto-commit above keeps
        // (and `commit_close_bookkeeping` keeps for `bee close`): a failure
        // here warns on its own line and never turns this green merge red.
        // NEVER a phase write — a merge can land one slice of many, so
        // `phase` stays close's word alone (mcl-3). Placed AFTER the
        // post-commit dirty guard above (mcl-2 R2): the guard must take its
        // reading before this write touches the tracked lane file, or every
        // green merge would trip `verify_mutated_tracked_files` on its own
        // work.
        if let Some(feature) = feature.as_deref() {
            match close_the_lane_on_merge(main_root, feature, &merge_commit_sha) {
                Ok(next_action) if !next_action.is_empty() => {
                    result.insert("next_action".into(), json!(next_action));
                }
                Ok(_) => {} // no lane file for this feature: nothing to rewrite
                Err(reason) => {
                    eprintln!(
                        "bee worktree merge: could not close {feature}'s lane after this green merge ({reason}) — run \"bee close --feature {feature}\" when ready; its waiting_on/next_action may be stale until then."
                    );
                    result.insert("lane_close_warning".into(), json!(reason));
                }
            }
        }

        // D7/D8: "unproven" now means the proof check cleared with nothing
        // actually PROVEN — no capped cells, an unresolved feature, or only
        // legacy caps with no D8 proof line — rather than "no commands.test
        // configured".
        let proof_unproven = proof.as_ref().map(|p| p.proven_count == 0).unwrap_or(true);
        attach_cleanup_outcome(
            &mut result,
            main_root,
            worktree_root,
            branch,
            id,
            cleanup,
            proof_unproven,
        );
        // staging-lane D0a (trigger 3): a successful merge just moved main,
        // which is exactly what makes any existing staging record stale —
        // nudge, never force, a rebuild. Only reachable here (a REAL commit
        // landed); ALREADY_UP_TO_DATE (merge_stage's own early-return arm)
        // never reaches merge_finish, so it never nudges — nothing on main
        // moved for staging to be stale against.
        if crate::verbs::staging::read_staging_record(main_root).ok().flatten().is_some() {
            result.insert("staging_rebuild_suggested".into(), json!("bee staging rebuild"));
        }
        // wkm-1 (D1): the keep path (cleanup == false) queues its one
        // cross-check entry AFTER attach_cleanup_outcome runs — this is the
        // real-merge success path only; ALREADY_UP_TO_DATE (merge_stage's
        // own arm above) never reaches merge_finish at all.
        if !cleanup {
            enqueue_worktree_cleanup_deferral(main_root, worktree_root, id, branch, &merge_commit_sha);
        }
        Ok(MergeAnswer { result, ok: true })
    })();

    // The `finally` safety net: an unexpected exit that never committed and
    // never aborted must not strand a staged merge on main.
    if !committed && merge_head_file.exists() {
        run_git(main_root, &["merge", "--abort"]);
    }
    outcome
}

/// uat-gate-before-merge D1: `uat_before_merge` in `.bee/config.json` — ON
/// by default (absent \u{2192} true, so the door protects every repo until
/// someone opts it off), an explicit `false` disables it repo-wide, and any
/// other shape refuses (`None`) rather than guessing which way to resolve a
/// typo. Models `worktree_cleanup_on_merge_config`'s fail-closed shape
/// (handlers.rs:415-421) with the default flipped to ON.
pub(crate) fn uat_before_merge_config(main_root: &Path) -> Option<bool> {
    match crate::state::read_config_raw(main_root).get("uat_before_merge") {
        None => Some(true),
        Some(Value::Bool(b)) => Some(*b),
        Some(_) => None,
    }
}

/// uat-gate-before-merge D1: does `mode` (a record's risk-lane
/// classification) require uat approval before merge? Only the known
/// LOW-risk lanes are exempt (`tiny`/`small`/`docs`/`spike`, i.e.
/// `ROUTE_LANE_VALUES` minus `standard`/`high-risk`) — a missing record, a
/// null mode, or any value this port does not recognize fails CLOSED as
/// "standard", because an unclassified feature is exactly the case a silent
/// skip would be most dangerous for.
fn uat_gate_applies_to_lane(mode: Option<&str>) -> bool {
    !matches!(mode, Some("tiny") | Some("small") | Some("docs") | Some("spike"))
}

/// The merge-time uat precondition's two fail-closed reads.
struct UatPrecheck {
    lane_applies: bool,
    gate_approved: bool,
}

/// uat-gate-before-merge D1: `lane_applies` prefers the live workflow
/// record's own `mode` field (present regardless of whether the feature was
/// ever bound to an explicit `--as-lane` file), falling back to
/// `.bee/lanes/<feature>.json` (`read_lane_display` — the same fail-open
/// display read `close`'s own scoping already reuses, drivers/close.rs:305)
/// when no live workflow names the feature. `gate_approved` reads the live
/// workflow record's own `gates.uat.approved` (GATE_NAMES-driven, written by
/// `bee state gate --name uat`), falling back to the plain default
/// `.bee/state.json` record's `approved_gates.uat` ONLY when that record is
/// presently tracking THIS feature — a foreign feature's approval must
/// never leak through as "approved" for a different one. A feature that
/// could not even be resolved (`feature` is `None`) fails closed on both: an
/// unclassifiable lane (standard) and an unapprovable gate (false). Every
/// read here is fail-open/fail-closed by construction (`list_workflows`,
/// `read_lane_display`, `read_state_peek` never throw for an ordinary
/// missing/corrupt shape), so an unreadable store never delegates — it
/// reads as "not approved", the safe direction.
fn uat_merge_precheck(main_root: &Path, feature: Option<&str>) -> UatPrecheck {
    let Some(feature) = feature else {
        return UatPrecheck { lane_applies: true, gate_approved: false };
    };
    let workflows = crate::verbs::workflow_store::list_workflows(main_root).unwrap_or_default();
    let live = crate::verbs::workflow_store::find_live_workflow(&workflows, feature);

    let mode = live
        .and_then(|wf| wf.get("mode"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            crate::verbs::workflow_store::read_lane_display(main_root, feature)
                .ok()
                .flatten()
                .and_then(|rec| rec.get("mode").and_then(Value::as_str).map(str::to_string))
        });
    let lane_applies = uat_gate_applies_to_lane(mode.as_deref());

    let gate_approved = if let Some(wf) = live {
        matches!(
            wf.get("gates").and_then(|g| g.get("uat")).and_then(|e| e.get("approved")),
            Some(Value::Bool(true))
        )
    } else {
        crate::verbs::state_group::read_state_peek(main_root)
            .ok()
            .filter(|state| matches!(state.get("feature"), Some(Value::String(f)) if f == feature))
            .is_some_and(|state| {
                matches!(
                    state.get("approved_gates").and_then(|g| g.get("uat")),
                    Some(Value::Bool(true))
                )
            })
    };

    UatPrecheck { lane_applies, gate_approved }
}

/// mergeFeatureWorktree — P1 / P3. D7/D8 (td-3): the former "(P2) — the lock
/// released across the verify child" is RETIRED — `merge_stage` (P1) already
/// checked the merging feature's D8 proof lines as a zero-mutation
/// precondition, so there is nothing left to run unlocked between staging
/// and finishing. `Err(MErr::Ex)` is only ever produced by P1, before any
/// mutation; the caller has already taken the queue lock by then, so it is
/// the documented late-delegation residual, never an ordinary shape.
#[allow(clippy::too_many_arguments)]
pub(crate) fn merge_feature_worktree(
    main_root: &Path,
    id: &str,
    cleanup: bool,
    companion_end_command: Option<&str>,
    skip_uat: bool,
    hooks: Option<&crate::integration_queue::Hooks<'_>>,
) -> MR<MergeAnswer> {
    let mut guard = lock::acquire_store_lock(main_root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS)
        .map_err(|b| MErr::Thrown(b.message()))?;
    let staged = merge_stage(main_root, id, companion_end_command, skip_uat);
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

    // Nothing to release the lock around anymore (D7/D8 retired the P2
    // verify-child window) — stage and finish inside the SAME shape as the
    // pre-hardening-4c single-lock behavior, which is still TWO acquires
    // (the .mjs re-enters withStoreLock here, and this port keeps that shape
    // so a delegation never drops one `result: "acquired"` row from
    // .bee/logs/contention.jsonl).
    let mut guard = lock::acquire_store_lock(main_root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS)
        .map_err(|b| MErr::Thrown(b.message()))?;
    let out = merge_finish(main_root, &staged, cleanup, lease_fence);
    guard.release();
    out
}
