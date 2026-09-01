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
use crate::uat::{uat_gate_applies_to_lane, uat_gate_approved, uat_stop_config, UatStop};
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
    /// D4 (docs/history/proof-strength-and-expiry/CONTEXT.md): the capped
    /// cells whose recorded proof is OLDER than the tree this merge lands —
    /// computed pre-merge (the merge base only exists before the merge) and
    /// carried across the released lock exactly like `proof`, so P3 can
    /// report it. Empty on every ordinary merge; an ADVISORY, never a
    /// refusal — this door checks recorded proof and runs nothing
    /// (verbs/cells/proof.rs), and D4 adds no new door.
    pub(crate) proof_stale: Vec<String>,
    /// uat-stop-placement D4.1: the placement `uat_stop_config` already
    /// resolved for the merge precondition above, carried across the
    /// released lock so P3's lane write (D4.2) and cleanup suppression
    /// (D4.3) branch on the exact same resolved value.
    pub(crate) uat_stop: UatStop,
}

pub(crate) enum StageOut {
    /// A TERMINAL outcome already fully resolved inside the P1 lock.
    Done(MergeAnswer),
    Staged(Box<Staged>),
}

/// D4: the capped cells of this feature whose recorded proof predates the
/// tree being merged — the rows the `proof-stale` advisory names.
///
/// A cap records the commit its proof was taken at (`ProofCheck.commits`,
/// verbs/cells/proof.rs). The merge base is where `branch` and main last
/// agreed, so it is the newest state both sides of this merge already
/// contain: a cap whose commit DESCENDS from the base was proven against
/// that state, and one that does not was proven before it — main moved (or
/// the branch was rewritten) after the proof was taken, which is the exact
/// event D4 exists to make visible.
///
/// It FAILS OPEN in every uncertain case, because a warning that is wrong
/// is worse than no warning: no merge base, a git call that did not run, or
/// a recorded sha this repo does not have (`--is-ancestor` exits 128, not
/// 1) all say nothing. Only a definite "not a descendant" — exit 1 — is
/// reported stale. Caps with no recorded commit never reach here at all;
/// `feature_proof_check` drops the `"none"` sentinel at the read.
///
/// The merge base is computed HERE rather than taken from
/// `branch_changed_files` (merge.rs): that helper's own base is a local,
/// and its only call site runs far below this one.
fn stale_proof_cells(
    main_root: &Path,
    branch: &str,
    proof: &crate::verbs::cells::ProofCheck,
) -> Vec<String> {
    if proof.commits.is_empty() {
        return Vec::new();
    }
    let base = run_git(main_root, &["merge-base", "HEAD", branch]);
    if base.status != Some(0) {
        return Vec::new();
    }
    let base = js_trim(&base.stdout.unwrap_or_default()).to_string();
    if base.is_empty() {
        return Vec::new();
    }
    proof
        .commits
        .iter()
        .filter(|(_, commit)| {
            run_git(main_root, &["merge-base", "--is-ancestor", &base, commit]).status == Some(1)
        })
        .map(|(id, _)| id.clone())
        .collect()
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

    // slp-dissent-stop-and-ask sd-5 (a2affcba, 4b7aa303): the merge door's
    // OWN dissent precondition. a2affcba names BOTH doors — the branch must
    // not land while a worker's blocker question is unanswered — and merge
    // had no cell-debt precondition but proof debt, so this is new code
    // rather than a copied close-door arm.
    //
    // Both readers are the SAME two functions `bee close`'s dissent-debt
    // door calls (verbs/cells/dissent.rs, sd-4), never a second reading of
    // the rule: one obligation, two doors. The escape is consulted ONLY
    // when there is debt to escape, exactly as the close door consults it.
    //
    // Placed HERE — after the proof door, before the main-dirty bookkeeping
    // auto-commit below — because everything above that auto-commit is the
    // zero-mutation zone: a dissent-debt merge is refused, never staged. A
    // proof-debt refusal therefore MASKS a dissent refusal on the same
    // feature; that matches the close door's own ordering (its judge arm
    // precedes its dissent arm) and is deliberate.
    //
    // A worktree whose feature cannot be resolved has no cell record to
    // check, so the helper is never called at all — the same `None` posture
    // the proof door above and `uat_merge_precheck` below both take.
    if let Some(feature) = identity.feature.as_deref() {
        let dissent = crate::verbs::cells::feature_dissent_debt(main_root, feature)
            .map_err(|_: crate::verbs::drivers::Delegate| MErr::Ex)?;
        if dissent.count > 0 {
            let deferred = crate::verbs::cells::has_dissent_deferral_decision(main_root, feature)
                .map_err(|_: crate::verbs::drivers::Delegate| MErr::Ex)?;
            if !deferred {
                // `DebtSummary.ids` is `Vec<Value>`, not `Vec<String>` — the
                // close door renders it through `js_join` and so does this.
                return Err(refuse_merge(
                    "WORKTREE_MERGE_DISSENT_DEBT",
                    format!(
                        "worktree {id}'s feature \"{feature}\" has {} cell(s) carrying a dissent with no recorded verdict ({}) — \"bee worktree merge\" refuses until each one is answered: bee cells dissent-verdict --id <id> --verdict accept|reject|escalate --reason \"<why>\". To defer instead, log a decision tagged dissent-deferral naming \"{feature}\" — the same escape \"bee close\"'s dissent-debt door reads.",
                        dissent.count,
                        crate::verbs::drivers::js_join(&dissent.ids, ", "),
                    ),
                ));
            }
        }
    }

    // slp-advisor-nudge an-3 (9e5eda5b): the merge door's advisor-nudge
    // precondition — a branch must not land while the supervisor's advisor
    // recommendation about that work is unanswered.
    //
    // Copied from the dissent precondition right above, including its two
    // structural choices: the SAME function the close door and the cap path
    // both call (verbs/supervisor.rs `feature_advisor_nudge_debt`) — one
    // obligation, three doors, never a second reading of the rule — and the
    // placement in the zero-mutation zone above the bookkeeping auto-commit,
    // so a nudged merge is refused, never staged. A dissent-debt refusal
    // therefore MASKS this one on the same feature, matching the close
    // door's own ordering.
    //
    // Two differences from the dissent arm, both because 9e5eda5b puts the
    // obligation on each nudge rather than on the feature: there is no
    // feature-level escape to consult (a cleared row simply stops being
    // counted), and the ids named are MAILBOX ROW ids, not cell ids.
    //
    // A worktree whose feature cannot be resolved has no row to check, so the
    // helper is never called — the same `None` posture every door here takes.
    if let Some(feature) = identity.feature.as_deref() {
        let nudge = crate::verbs::supervisor::feature_advisor_nudge_debt(main_root, feature)
            .map_err(|_: crate::verbs::drivers::Delegate| MErr::Ex)?;
        if nudge.count > 0 {
            return Err(refuse_merge(
                "WORKTREE_MERGE_ADVISOR_NUDGE_DEBT",
                format!(
                    "worktree {id}'s feature \"{feature}\" has {} advisor nudge(s) with no consult and no recorded decline ({}) — \"bee worktree merge\" refuses until each one is answered: run the consult, then record what came of it with bee decisions log --tags advisor-nudge, naming that row id in the text. A reasoned decline is the same record. One decision answers ONE row — the same per-row escape \"bee close\"'s advisor-nudge door reads.",
                    nudge.count,
                    crate::verbs::drivers::js_join(&nudge.ids, ", "),
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
    // wider) and proceed; dirt OUTSIDE them still refuses, named by path —
    // narrowed by mdp-1 to the dirt this merge can actually collide with
    // (see the split right below).
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
        let roots = main_bookkeeping_roots(main_root, identity.feature.as_deref());
        let dirt = main_dirt_excluding(main_root, &roots).map_err(MErr::Thrown)?;
        // mdp-1: the leftover dirt is no longer one undifferentiated pile.
        //
        // A TRACKED modification outside the bookkeeping roots refuses
        // exactly as it always did — this narrowing never reaches it. An
        // UNTRACKED path is different: the merge can only harm it by
        // WRITING it, so it blocks only when it is in the merging branch's
        // own changed-file set (`branch_changed_files`, read against the
        // MERGE BASE — main's HEAD would name a file main deleted after the
        // fork as "changed" and refuse for a collision that cannot happen).
        // Anything else is a peer session's in-flight file this merge will
        // never touch; refusing on it made the steady state with several
        // live sessions "nobody merges until everybody commits" — a
        // coordination cost paid daily, four refusals in two days, for a
        // collision that cannot happen. Exempting it from the REFUSAL is
        // not exempting it from anything else: `main_bookkeeping_roots` is
        // untouched below, so no untracked path is ever swept into the
        // auto-commit, and git's own "untracked working tree file would be
        // overwritten by merge" stays the backstop for whatever the branch
        // does touch.
        let blocking_untracked: Vec<String> = if dirt.untracked.is_empty() {
            Vec::new()
        } else {
            match current_branch(&worktree_root).and_then(|b| branch_changed_files(main_root, &b)) {
                Some(changed) => dirt.untracked.iter().filter(|u| changed.contains(*u)).cloned().collect(),
                // Fail closed: a detached HEAD or an unresolvable merge base
                // leaves the collision question unanswerable, so every
                // untracked path blocks, exactly as before the narrowing.
                None => dirt.untracked.clone(),
            }
        };
        if !dirt.tracked.is_empty() || !blocking_untracked.is_empty() {
            let offending = dirt
                .tracked
                .iter()
                .cloned()
                .chain(blocking_untracked.iter().map(|u| format!("?? {u}")))
                .collect::<Vec<String>>()
                .join("\n");
            let scope = match &identity.feature {
                Some(f) => format!(".bee/, docs/decisions/, docs/knowledge/ paths this feature recorded, or docs/history/{f}/"),
                None => ".bee/ or docs/decisions/".to_string(),
            };
            return Err(refuse_merge(
                "WORKTREE_MERGE_MAIN_DIRTY",
                format!(
                    "the MAIN checkout at {} has uncommitted changes that \"bee worktree merge\" will not auto-commit — commit or stash them before merging:\n{offending}\nWhat still blocks: any TRACKED change outside {scope}, plus any UNTRACKED file this branch itself writes (the merge would overwrite it). An untracked file the branch never touches cannot collide, so it no longer blocks — it is left uncommitted and untouched.",
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
                "Auto-commit .bee, docs/decisions, {f}'s recorded docs/knowledge, and docs/history/{f} bookkeeping before merging worktree {id}"
            ),
            None => format!("Auto-commit .bee and docs/decisions bookkeeping before merging worktree {id}"),
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

    // uat-stop-placement D1/D4.1: the LAST zero-mutation precondition — a
    // standard/high-risk feature whose "uat" gate is not yet approved
    // refuses here, before the companion is torn down or anything else
    // mutates, but ONLY under `uat_stop: "merge"` (today's default). Under
    // `"close"` and `"off"` this precondition never refuses at all — the
    // door has either moved to `bee close` or is off entirely. `identity`
    // was already resolved above (line ~97) for the bookkeeping
    // auto-commit's scope; reused here rather than re-resolved.
    // `uat_stop_config`'s `None` (a typo'd string, a non-string `uat_stop`,
    // or a non-boolean `uat_before_merge`) refuses UNCONDITIONALLY, before
    // the lane/gate reads even run and before the placement is even known —
    // a typo'd config must never silently resolve to any outcome, on ANY
    // merge, not just the ones that would otherwise be gated. `uat_stop` is
    // resolved exactly ONCE, here, and carried in `Staged` so P3's lane
    // write (D4.2) and cleanup suppression (D4.3) act on the SAME resolved
    // value this precondition already checked — never a second, possibly-
    // drifted read.
    let Some(uat_stop) = uat_stop_config(main_root) else {
        return Err(refuse_merge(
            "WORKTREE_MERGE_UAT_CONFIG_INVALID",
            format!(
                "{}/.bee/config.json's uat-stop-placement config is invalid \u{2014} \"uat_stop\" must be \"merge\", \"close\", or \"off\" when present, and its back-compat alias \"uat_before_merge\" must be a boolean (true or false) when present \u{2014} merge refuses rather than guess which way to resolve it. FIX: set \"uat_stop\" to one of those three values, set \"uat_before_merge\" to true or false, or remove both keys entirely to use the default (\"merge\", uat-stop-placement D1).",
                p(main_root)
            ),
        ));
    };
    if uat_stop == UatStop::Merge && !skip_uat {
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

    // D4: the staleness advisory's own read — still inside the zero-mutation
    // zone, and read-only git. It cannot sit beside the proof check further
    // up (which is where the merge door READS proof) because `branch` is not
    // resolved until the detached-HEAD/branch-mismatch checks above, and the
    // merge base needs it; it cannot sit after the merge either, because the
    // merge is what destroys the base. Nothing here can refuse — the rows
    // are carried to P3 and reported beside the merge that landed.
    let proof_stale = proof
        .as_ref()
        .map(|p| stale_proof_cells(main_root, &branch, p))
        .unwrap_or_default();

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
        proof_stale,
        feature: identity.feature.clone(),
        uat_stop,
    })))
}

/// mcl-2 (R1) / uat-stop-placement D4.2: what `close_the_lane_on_merge`
/// answers with — the `next_action` text it wrote, empty when the feature
/// has no lane file at all (nothing to rewrite). usp-5: `merge_finish` no
/// longer reads a `uat_wait_set` bit off this outcome to drive D4.3's
/// cleanup suppression — whether the lane record exists is a bookkeeping
/// fact this type carries, and whether the user still needs their worktree
/// is a DIFFERENT, precheck-only fact `merge_finish` now computes itself
/// directly (see its own `uat_wait_set` local), so a missing lane file can
/// never silently un-suppress cleanup.
struct CloseLaneOutcome {
    next_action: String,
    /// merge-ready-fact D2: did this call WRITE to the feature's record —
    /// the lane rewrite itself, or the `merge_ready` clear alone? The caller
    /// commits the tracked lane file on either, so a merge whose only lane
    /// change was dropping the merge-ready fact still lands that change in
    /// its own bookkeeping commit instead of leaving it as dirt in main.
    changed: bool,
}

/// mcl-2 (R1) / uat-stop-placement D4.2: the merge-time half of "a shipped
/// feature stops reading waiting on you" — now branching on the SAME
/// resolved `uat_stop` the merge precondition above already checked (D4.1).
///
/// Under `Merge`/`Off`, or under `Close` when the lane is exempt or uat is
/// already approved: today's behavior, kept BYTE-FOR-BYTE — clears the
/// merging feature's stranded `waiting_on` + `run_state` pair through
/// mcl-1's shared helper (`clear_lane_waiting_on_pair`,
/// `state_group/waiting_on.rs` — read, never re-implemented here) and
/// rewrites the lane's `next_action` to name the close road.
///
/// Under `Close`, when the lane cares (`uat_gate_applies_to_lane`) and uat
/// is still unapproved (the SAME fail-closed reads `uat_merge_precheck`
/// already uses under `Merge`, reused here rather than re-derived): the
/// INVERSE — delegates to `set_lane_uat_wait_on_merge`, which SETS
/// `waiting_on` instead of clearing it.
///
/// Every failure surfaces as `Err` so the one caller can warn and keep the
/// merge green; this function itself never decides warn-vs-fail.
fn close_the_lane_on_merge(
    main_root: &Path,
    feature: &str,
    merge_commit_sha: &str,
    uat_stop: UatStop,
) -> Result<CloseLaneOutcome, String> {
    // merge-ready-fact D2: a merged feature is no longer WAITING to be
    // merged, so the stored fact goes away with the merge — in this same
    // lane-closing mutation, on BOTH branches below (the merge is what ends
    // the wait, whatever `uat_stop` does to `waiting_on`). FAIL-OPEN:
    // `clear` never throws and never turns this green merge red; its answer
    // only tells the caller whether the record was written at all.
    let merge_ready_cleared = crate::verbs::workflow_store::merge_ready::clear(main_root, feature);
    if uat_stop == UatStop::Close {
        let precheck = uat_merge_precheck(main_root, Some(feature));
        if precheck.lane_applies && !precheck.gate_approved {
            let mut outcome = set_lane_uat_wait_on_merge(main_root, feature, merge_commit_sha)?;
            outcome.changed = outcome.changed || merge_ready_cleared;
            return Ok(outcome);
        }
    }
    crate::verbs::state_group::clear_lane_waiting_on_pair(main_root, feature)
        .map_err(lane_write_err_text)?;
    let Some(mut lane) =
        crate::verbs::workflow_store::read_lane_strict(main_root, feature).map_err(lane_write_err_text)?
    else {
        // No lane file to rewrite — but the clear above may still have
        // written the fact off the DEFAULT record, so `changed` reports it.
        return Ok(CloseLaneOutcome { next_action: String::new(), changed: merge_ready_cleared });
    };
    let short_sha = &merge_commit_sha[..merge_commit_sha.len().min(7)];
    let next_action = format!(
        "Merged into main at {short_sha}; capture what settled, then bee close --feature {feature}."
    );
    lane.insert("next_action".into(), json!(next_action.clone()));
    crate::verbs::workflow_store::write_lane(main_root, &lane).map_err(lane_write_err_text)?;
    Ok(CloseLaneOutcome { next_action, changed: true })
}

/// uat-stop-placement D4.2's SET branch: under `uat_stop: "close"`, a merge
/// whose merged feature's "uat" gate is still pending INVERTS
/// `close_the_lane_on_merge`'s usual clear — it SETS `waiting_on` to a
/// `"gate"` mark naming `"uat: <feature>"` instead of clearing it, and
/// points `next_action` at the reload-test-approve-or-fix road rather than
/// at `bee close`. The mark is written straight onto the lane record (not
/// through a live workflow's session-scoped `set_workflow_waiting_on` —
/// mirroring `clear_lane_waiting_on_pair`'s own lane-file-direct write, the
/// only copy a merged-and-possibly-torn-down worktree's feature reliably
/// still has), so `session` names the merge itself
/// (`WORKTREE_MERGE_SESSIONLESS_ID`) rather than a live interactive
/// session. Returns an empty, unset outcome when the feature has no lane
/// file at all — nothing to rewrite, not an error.
fn set_lane_uat_wait_on_merge(
    main_root: &Path,
    feature: &str,
    merge_commit_sha: &str,
) -> Result<CloseLaneOutcome, String> {
    let Some(mut lane) =
        crate::verbs::workflow_store::read_lane_strict(main_root, feature).map_err(lane_write_err_text)?
    else {
        return Ok(CloseLaneOutcome { next_action: String::new(), changed: false });
    };
    let short_sha = &merge_commit_sha[..merge_commit_sha.len().min(7)];
    lane.insert(
        "waiting_on".into(),
        json!({
            "kind": "gate",
            "subject": format!("uat: {feature}"),
            "asked_at": now_iso(),
            "session": WORKTREE_MERGE_SESSIONLESS_ID,
        }),
    );
    let next_action = format!(
        "Merged into main at {short_sha}; reload the product and test \"{feature}\", then either approve uat (\"bee gate --name uat --approved true --lane {feature}\") or fix it in the worktree and merge again."
    );
    lane.insert("next_action".into(), json!(next_action.clone()));
    crate::verbs::workflow_store::write_lane(main_root, &lane).map_err(lane_write_err_text)?;
    Ok(CloseLaneOutcome { next_action, changed: true })
}

/// `Err2`'s two shapes, rendered as one warn-line message —
/// `close_the_lane_on_merge`/`set_lane_uat_wait_on_merge`'s only consumers.
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
        proof_stale,
        feature,
        uat_stop,
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
        // D4: the `proof-stale` advisory, beside the `verify` field it
        // qualifies. Present only when there is something to say, and the
        // merge it rides on has already landed — this names an age, never a
        // fault, and refuses nothing.
        if !proof_stale.is_empty() {
            result.insert("proof_stale".into(), json!({
                "code": "proof_stale",
                "cells": proof_stale,
                "message": format!(
                    "{} capped cell(s) recorded their proof before the merge base this branch is landing against ({}) — the tree moved after those proofs were taken, so they were never run against what is merging now. The merge is NOT refused (bee worktree merge checks recorded proof and runs nothing); re-run their proof if the change is one main could have broken.",
                    proof_stale.len(),
                    proof_stale.join(", "),
                ),
            }));
        }
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
        // usp-5: whether THIS merge's feature still owes an unapproved
        // "uat" under `uat_stop: "close"` is a precheck fact — the SAME
        // fail-closed reads `uat_merge_precheck` already takes for the
        // merge-time precondition above, and the same call
        // `close_the_lane_on_merge` makes internally to pick its own
        // branch. It is computed HERE, directly, rather than read back off
        // `CloseLaneOutcome::uat_wait_set` (which `set_lane_uat_wait_on_merge`
        // only sets when a `.bee/lanes/<feature>.json` record exists to
        // rewrite): whether the lane record exists on disk is a bookkeeping
        // fact, and whether the user still needs their worktree is not — the
        // second must never be decided by the first. So `uat_wait_set` below
        // stays true for a pending uat even when the lane write below finds
        // no lane file to touch (`Ok` with an empty `next_action`) or fails
        // outright (`Err`) — the D4.3 cleanup suppression it feeds must fail
        // closed exactly like the precondition already does.
        //
        // usp-7: `uat_merge_precheck` is called with `feature.as_deref()`
        // directly — the SAME `Option<&str>` the merge-time precondition
        // (line ~893) passes it — rather than gated behind
        // `feature.as_deref().is_some_and(...)`. `is_some_and` short-
        // circuited to `false` the instant `Staged.feature` was `None` (an
        // unresolvable feature), which skipped `uat_merge_precheck`
        // entirely and left `uat_wait_set` `false` — UNsuppressed cleanup —
        // even though `uat_merge_precheck(main_root, None)` itself already
        // fails closed (`lane_applies: true, gate_approved: false`, mirror
        // of `uat_gate_applies_to_lane(None) == true` at the precondition).
        // Calling it directly, the way the precondition already does, means
        // an unresolvable feature under `uat_stop: "close"` now suppresses
        // cleanup exactly like a resolved-but-pending one — nobody could
        // check whether a uat was owed, so the worktree is kept rather than
        // torn down out from under whichever fix depends on it.
        let precheck = uat_merge_precheck(main_root, feature.as_deref());
        let uat_wait_set =
            *uat_stop == UatStop::Close && precheck.lane_applies && !precheck.gate_approved;
        if let Some(feature) = feature.as_deref() {
            match close_the_lane_on_merge(main_root, feature, &merge_commit_sha, *uat_stop) {
                // mrf-2: `|| outcome.changed` is the second door into this
                // same commit — a lane whose ONLY change was the
                // `merge_ready` clear carries no new `next_action`, and
                // would otherwise leave a modified TRACKED lane file
                // uncommitted in main (exactly the dirt mct-1 closed).
                Ok(outcome) if !outcome.next_action.is_empty() || outcome.changed => {
                    let next_action = outcome.next_action;
                    if !next_action.is_empty() {
                        result.insert("next_action".into(), json!(next_action));
                    }
                    // mct-1: the lane rewrite just above landed a TRACKED
                    // file (`.bee/lanes/<feature>.json`) modified-but-
                    // uncommitted in main — dirt indistinguishable from any
                    // OTHER post-commit drift until some unrelated
                    // bookkeeping auto-commit happened to sweep it up. One
                    // path-scoped commit of exactly that one file, through
                    // the SAME mechanism trun-4's pre-merge bookkeeping
                    // commit already owns (`commit_main_bookkeeping` —
                    // `git add -A -- <lane path>` then `git commit --
                    // <lane path>`, never a bare `git add -A`, never
                    // `--amend` the merge commit that already landed).
                    // Runs strictly AFTER the post-commit dirty guard above
                    // has taken its reading (mcl-2 R2 ordering, unchanged)
                    // and after the lane write itself, so it commits
                    // exactly the rewrite this merge just made — never a
                    // moving target. BEST-EFFORT, same warn-never-block
                    // contract as the lane write and the pre-merge
                    // bookkeeping commit: a failure here warns on its own
                    // line and never turns this green merge red.
                    let lane_pathspec = format!(".bee/lanes/{feature}.json");
                    let lane_commit = commit_main_bookkeeping(
                        main_root,
                        &format!("Commit {feature}'s lane rewrite after merging worktree {id}"),
                        &[lane_pathspec],
                    );
                    if let MainBookkeepingCommit::Skipped { reason, .. } = &lane_commit {
                        if reason != "clean" {
                            eprintln!(
                                "bee worktree merge: could not commit {feature}'s lane rewrite after this green merge ({reason}) — the lane file will show as modified in \"git status\" until committed by hand."
                            );
                        }
                    }
                    result.insert("lane_bookkeeping_commit".into(), lane_commit.value());
                }
                Ok(_) => {} // no lane file for this feature: nothing to rewrite; `uat_wait_set` above already carries the precheck's own fail-closed answer regardless
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
        // uat-stop-placement D4.3: while the uat wait THIS merge just set is
        // live (only reachable when `uat_wait_set` is true — `uat_stop:
        // "close"`, the lane cares, and uat is still unapproved),
        // `--cleanup` and `worktree_cleanup_on_merge: true` are both
        // ignored — the worktree is the only place a failed uat can be
        // fixed, and tearing it down now would drop the grant the SECOND
        // merge needs (the no-granted-worktree refusal, WORKTREE_MERGE_UNKNOWN_ID,
        // above). `effective_cleanup` collapses to exactly `cleanup` when
        // `uat_wait_set` is false (Merge, Off, or Close-exempt/-approved),
        // so that path is untouched. Only override the result's `cleanup`
        // reporting when a real request (`cleanup == true`) was actually
        // suppressed — an unrequested keep needs no explanation beyond the
        // existing `cleanup_suggested_command` line `attach_cleanup_outcome`
        // already writes.
        let effective_cleanup = cleanup && !uat_wait_set;
        if cleanup && uat_wait_set {
            result.insert("cleanup".into(), json!({
                "ok": false,
                "code": "WORKTREE_MERGE_CLEANUP_SUPPRESSED_UAT_PENDING",
                "reason": "the merged feature's \"uat\" gate is still pending under uat_stop \"close\" \u{2014} the worktree is the only place a failed uat can be fixed, and tearing it down now would drop the grant a second merge needs (WORKTREE_MERGE_UNKNOWN_ID). Reload the product and test it, then either approve uat (\"bee gate --name uat --approved true\") or fix in the worktree and merge again.",
            }));
        } else {
            attach_cleanup_outcome(
                &mut result,
                main_root,
                worktree_root,
                branch,
                id,
                effective_cleanup,
                proof_unproven,
            );
        }
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
        // own arm above) never reaches merge_finish at all. Uses
        // `effective_cleanup` (D4.3): a merge suppressed by a live uat wait
        // kept the worktree too, so it queues the same cross-check entry.
        if !effective_cleanup {
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

// uat-stop-placement D1: the placement read (`uat_stop_config`, `.bee/config.json`
// "uat_stop" with the "uat_before_merge" back-compat alias) now lives once,
// in `crate::uat` (imported above) — `uat_before_merge_config` (formerly
// here) is retired; nothing calls it anymore.

// uat-stop-placement D2 (docs/history/uat-stop-placement/CONTEXT.md): the
// lane rule now lives once, in `crate::uat` (imported above), so the merge
// side and the close side never carry two copies of it.

/// The merge-time uat precondition's two fail-closed reads.
struct UatPrecheck {
    lane_applies: bool,
    gate_approved: bool,
}

/// uat-gate-before-merge D1, as revised by two features that met here.
/// `lane_applies` reads through `crate::uat::uat_lane_mode` (uat-stop-placement
/// usp-6: the one lane-classification read, shared with `close`'s uat door), which
/// prefers the live workflow record's own `mode` field — present regardless of
/// whether the feature was ever bound to an explicit `--as-lane` file — falling
/// back to `.bee/lanes/<feature>.json` (`read_lane_display`, the same fail-open
/// display read `close`'s own scoping already reuses) when no live workflow names
/// the feature.
///
/// `gate_approved` reads through `crate::uat::uat_gate_approved`
/// (docs/history/uat-approval-reaches-the-door/plan.md R1-R3, decision
/// uat-approval-reaches-the-door D1): the live workflow record's own
/// `gates.uat.approved` first and alone — a live record saying `false` beats any
/// fallback — then, only when no live record stands, an OR of the LANE record's
/// `approved_gates.uat` and the default `.bee/state.json`'s, the latter only when
/// that record is presently tracking THIS feature, so a foreign feature's approval
/// never leaks through. The lane source is the one this door used to miss: it is
/// where `bee gate --lane <f>` writes once the live-record lookup comes up empty.
///
/// A feature that could not even be resolved (`feature` is `None`) fails closed on
/// both: an unclassifiable lane (standard) and an unapprovable gate (false). Every
/// read is fail-open by construction (`list_workflows`, `read_lane_display`,
/// `read_state_peek` never throw for an ordinary missing/corrupt shape), so an
/// unreadable store reads as "not approved" — the safe direction.
fn uat_merge_precheck(main_root: &Path, feature: Option<&str>) -> UatPrecheck {
    let Some(feature) = feature else {
        return UatPrecheck { lane_applies: true, gate_approved: false };
    };
    let mode = crate::uat::uat_lane_mode(main_root, feature);
    let lane_applies = uat_gate_applies_to_lane(mode.as_deref());

    let gate_approved = uat_gate_approved(main_root, feature);

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
