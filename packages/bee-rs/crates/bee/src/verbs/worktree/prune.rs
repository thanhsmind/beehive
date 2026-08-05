// the fail-closed dead-worktree classifier (decisions D2, D2a, D2b).
//
// New code, not a Node port: `bee worktree prune` never existed there.
// Given a granted worktree id, `classify_worktree` answers dead-with-reason
// or kept-with-reason, and NO subcommand reaches it yet (that is wr-3). The
// governing rule, everywhere below: a missing file, an unreadable record, a
// non-zero git exit, or an unparseable value is never permission to delete —
// it is a reason to keep. Deletion has no retry, so the existing fail-OPEN
// convention this codebase uses for guards (git.rs:113's `branch_exists`,
// whose failure mode is "the real gate is `git worktree add`, try again") is
// the wrong shape here on purpose. `perform_cleanup` (merge.rs:355) already
// demonstrates the fail-closed shape for a single dirty check; this module
// is that shape carried through all seven conditions.
//
// No production caller lands until wr-3 wires `bee worktree prune` at
// handlers.rs — same shape as workspace_store.rs's own
// `#![allow(dead_code)]` while it waited for start-feature.
#![allow(unused_imports)]
#![allow(dead_code)]

use super::*;
use crate::jsjson;
use crate::verbs::reservations::js_trim;
use crate::verbs::workspace_store as ws;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

/// Hours, not `HEARTBEAT_STALE_SECONDS`' 15 minutes (claims.rs, leases.rs,
/// write_guard/store.rs, status_full/mod.rs — all `900.0`). That window
/// exists for a claim, where a stale read costs the loser a retry; this
/// window guards a deletion, which has no retry, and a gate answer or a
/// closed laptop lid outlasts 15 minutes on any ordinary day. Six hours.
pub(crate) const PRUNE_LIVENESS_SECONDS: f64 = 6.0 * 60.0 * 60.0;

/// The interrupted-operation markers under a worktree's OWN git admin dir,
/// `<main>/.git/worktrees/<id>/` — never the worktree's working tree, which
/// only ever sees the pointer file.
const INTERRUPTED_OP_MARKERS: [&str; 5] =
    ["rebase-merge", "rebase-apply", "CHERRY_PICK_HEAD", "MERGE_HEAD", "BISECT_LOG"];

/// One line naming why a worktree was judged dead or kept. `prune`'s
/// one-line-per-worktree report (wr-3) reads `reason()` directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Verdict {
    Dead { reason: String },
    Kept { reason: String },
}

impl Verdict {
    pub(crate) fn is_dead(&self) -> bool {
        matches!(self, Verdict::Dead { .. })
    }
    pub(crate) fn reason(&self) -> &str {
        match self {
            Verdict::Dead { reason } | Verdict::Kept { reason } => reason,
        }
    }
}

fn kept(reason: impl Into<String>) -> Verdict {
    Verdict::Kept { reason: reason.into() }
}

/// The inputs one classification needs. `base_commit` is the ALREADY
/// RESOLVED sha a whole prune run shares (see `resolve_prune_base`) — this
/// function never re-resolves a ref itself, so it can never race a moving
/// branch mid-run.
pub(crate) struct PruneCheck<'a> {
    pub(crate) main_root: &'a Path,
    pub(crate) id: &'a str,
    pub(crate) base_commit: &'a str,
    pub(crate) now_ms: f64,
    pub(crate) liveness_seconds: f64,
    pub(crate) age_threshold_ms: f64,
}

/// Resolves the base ref ONCE for a whole prune run. The base ref name lives
/// nowhere in the store (a workspace record's `base_sha` is a sha pinned at
/// CREATE time, not the ref — `workspace_store.rs:387`), so a caller supplies
/// the ref (e.g. its configured default branch) and this function turns it
/// into the single sha every worktree in the run is measured against.
///
/// A ref that does not resolve REFUSES THE WHOLE RUN rather than classifying
/// anything: `git rev-list --count base..branch` (the shape this replaces)
/// fails OPEN — any git failure reads as `0`, i.e. "merged", for every
/// worktree at once. `resolve_base_ref_commit` (git.rs:99) already returns
/// `None` on a bad ref; this is that call, made a hard refusal.
pub(crate) fn resolve_prune_base(main_root: &Path, base_ref: &str) -> Result<String, String> {
    resolve_base_ref_commit(main_root, base_ref).ok_or_else(|| {
        format!(
            "base ref {} does not resolve to a commit in {} (\"git rev-parse --verify\" found nothing) — refusing the whole prune run rather than guessing at mergedness.",
            jsjson::stringify(&Value::String(base_ref.to_string())),
            p(main_root)
        )
    })
}

/// `git merge-base --is-ancestor refs/heads/<branch> <base_commit>`,
/// EXIT CODE ONLY. Never `rev-list --count`: a parsed count reads a git
/// failure as the literal string `""`, which parses to `0` — "merged", for
/// every worktree the count is asked about, at once. `status != Some(0)` is
/// "not provably merged", full stop, whether that is a real divergence or
/// git refusing to run at all.
fn branch_is_merged(main_root: &Path, branch: &str, base_commit: &str) -> bool {
    let branch_ref = format!("refs/heads/{branch}");
    run_git(main_root, &["merge-base", "--is-ancestor", &branch_ref, base_commit]).status == Some(0)
}

/// D8a's blind spot, closed: `git status --porcelain` (no `--ignored`) never
/// sees a gitignored `.bee/HANDOFF.json` or `.bee/capture-queue.jsonl`, so a
/// paused handoff or an unpromoted capture stub reads as a clean tree and
/// would be deleted forever. This checks both directly, off the filesystem,
/// never through git.
fn precious_state_present(worktree_root: &Path) -> Option<String> {
    let handoff = worktree_root.join(".bee").join("HANDOFF.json");
    if handoff.exists() {
        return Some(format!(
            "{} is present — a paused handoff is precious and invisible to the clean-tree check (D8a); keeping.",
            p(&handoff)
        ));
    }
    let queue = worktree_root.join(".bee").join("capture-queue.jsonl");
    match std::fs::metadata(&queue) {
        Ok(meta) if meta.len() > 0 => Some(format!(
            "{} is non-empty — unpromoted capture stubs are precious and invisible to the clean-tree check (D8a); keeping.",
            p(&queue)
        )),
        Ok(_) => None, // present but empty: not precious.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(_) => Some(format!(
            "{} could not be checked — keeping rather than assuming it is empty.",
            p(&queue)
        )),
    }
}

/// Any of the five interrupted-operation markers under the worktree's git
/// admin dir keeps, hard — an in-progress rebase/cherry-pick/merge/bisect has
/// no business being deleted out from under it.
fn interrupted_operation_present(main_root: &Path, id: &str) -> Option<String> {
    let admin_dir = main_root.join(".git").join("worktrees").join(id);
    for marker in INTERRUPTED_OP_MARKERS {
        let path = admin_dir.join(marker);
        match std::fs::symlink_metadata(&path) {
            Ok(_) => {
                return Some(format!(
                    "{marker} is present under {} — an interrupted git operation; keeping.",
                    p(&admin_dir)
                ))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => {
                return Some(format!(
                    "{marker} under {} could not be checked — keeping rather than assuming it is absent.",
                    p(&admin_dir)
                ))
            }
        }
    }
    None
}

/// A minimal RFC3339 reader for the two timestamp shapes this module reads
/// (a session's `last_heartbeat`, `git log --format=%cI`'s commit date) —
/// both already ISO 8601 with an offset, so no JS `Date.parse` fallback
/// chain is needed the way claims.rs's ported `date_parse_val` needs one.
fn parse_iso_ms(s: &str) -> Option<f64> {
    chrono::DateTime::parse_from_rfc3339(s.trim())
        .ok()
        .map(|dt| dt.timestamp_millis() as f64)
}

/// D2b: liveness comes from session records under the MAIN checkout, never
/// the workspace record's `write_owner_session`/`attached_sessions` — the
/// only writer of those hardcodes `workspace_id = "main"`
/// (state_group/policy.rs:128), so they are null/empty for every worktree,
/// including one with a session live inside it.
///
/// A session HOLDS this worktree when its heartbeat is fresh within
/// `liveness_seconds` AND either it names this workspace directly
/// (`workspace_id == id`) or its `workspace_id` resolves (through the
/// workspace store) to a root that sits under this worktree's root. An
/// UNREADABLE session record — unparseable JSON, an unreadable file — counts
/// as LIVE: this scan cannot rule it out, so it holds every worktree still
/// being classified in the same run rather than silently skipping it the way
/// `list_session_records`'s fail-open scan does elsewhere in this codebase.
/// A missing/unparseable heartbeat is the same fail-closed direction: treated
/// as fresh, never as stale.
fn live_session_holds(
    main_root: &Path,
    id: &str,
    worktree_root: &Path,
    now_ms: f64,
    liveness_seconds: f64,
) -> Option<String> {
    let dir = main_root.join(".bee").join("sessions");
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return None, // no sessions dir at all: nothing to hold on.
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".json") {
            continue;
        }
        let file = dir.join(&name);
        let record: Map<String, Value> = match std::fs::read(&file) {
            Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
                Ok(Value::Object(m)) => m,
                _ => {
                    return Some(format!(
                        "session record {} does not parse as an object — an unreadable session record counts as live; keeping.",
                        p(&file)
                    ))
                }
            },
            Err(_) => {
                return Some(format!(
                    "session record {} could not be read — an unreadable session record counts as live; keeping.",
                    p(&file)
                ))
            }
        };

        let fresh = match record.get("last_heartbeat") {
            Some(Value::String(s)) => match parse_iso_ms(s) {
                Some(ms) => now_ms - ms <= liveness_seconds * 1000.0,
                None => true, // unparseable timestamp: fail closed as fresh.
            },
            _ => true, // no heartbeat at all: fail closed as fresh.
        };
        if !fresh {
            continue;
        }

        let session_name = record.get("id").and_then(Value::as_str).unwrap_or(name.as_str());
        if matches!(record.get("workspace_id"), Some(Value::String(s)) if s == id) {
            return Some(format!(
                "session {session_name} names this workspace with a heartbeat inside the {liveness_seconds}s liveness window; keeping."
            ));
        }
        if let Some(Value::String(ws_id)) = record.get("workspace_id") {
            if let Ok(ws_record) = ws::read_workspace(main_root, ws_id) {
                if let Some(Value::String(root_s)) = ws_record.get("root") {
                    if Path::new(root_s).starts_with(worktree_root) {
                        return Some(format!(
                            "session {session_name}'s resolved root sits under this worktree; keeping."
                        ));
                    }
                }
            }
        }
    }
    None
}

/// The last-commit age gate. `git log -1 --format=%cI refs/heads/<branch>`;
/// a non-zero exit or an unparseable date keeps — never guessed as "old
/// enough".
fn too_young(main_root: &Path, branch: &str, now_ms: f64, age_threshold_ms: f64) -> Option<String> {
    let branch_ref = format!("refs/heads/{branch}");
    let result = run_git(main_root, &["log", "-1", "--format=%cI", &branch_ref]);
    if result.status != Some(0) {
        return Some(format!(
            "the last commit date on {branch} could not be read ({}) — keeping rather than guessing its age.",
            result.fail_text()
        ));
    }
    let stdout = result.stdout.unwrap_or_default();
    let trimmed = js_trim(&stdout);
    let Some(commit_ms) = parse_iso_ms(trimmed) else {
        return Some(format!(
            "the last commit date on {branch} (\"{trimmed}\") did not parse — keeping rather than guessing its age."
        ));
    };
    if now_ms - commit_ms < age_threshold_ms {
        return Some(format!("the last commit on {branch} is younger than the age threshold; keeping."));
    }
    None
}

/// The classifier, whole: every condition from D2/D2a/D2b, in the order
/// plan.md's table lists them, each one keeping on ANY doubt. Reaching the
/// end with no keep-reason is the only way to answer `Verdict::Dead`.
pub(crate) fn classify_worktree(check: &PruneCheck<'_>) -> Verdict {
    // (0) the record everything else reads through. Unreadable or missing —
    // `WORKSPACE_MISSING`, `WORKSPACE_CORRUPT` — keeps before any other probe
    // runs; a classifier that cannot even find the worktree cannot pronounce
    // it dead.
    let record = match ws::read_workspace(check.main_root, check.id) {
        Ok(r) => r,
        Err(e) => {
            return kept(format!(
                "the workspace record for {} could not be read ({}) — keeping rather than guessing.",
                check.id,
                e.message()
            ))
        }
    };

    // A re-registered worktree carries `branch: null` (session_init.rs:436,
    // registerWorkspace's idempotent-create path passing `branch: None`) —
    // null keeps, same as any other "cannot confirm" shape here.
    let branch = match record.get("branch") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => {
            return kept(format!(
                "the workspace record for {} carries no branch (null) — a re-registered worktree looks exactly like this; keeping.",
                check.id
            ))
        }
    };

    let worktree_root = match record.get("root") {
        Some(Value::String(s)) if !s.is_empty() => PathBuf::from(s),
        _ => {
            return kept(format!(
                "the workspace record for {} carries no root — keeping rather than guessing where it lives.",
                check.id
            ))
        }
    };

    // (1) merged into base.
    if !branch_is_merged(check.main_root, &branch, check.base_commit) {
        return kept(format!(
            "{branch} is not provably merged into the resolved base ({}) — \"git merge-base --is-ancestor\" did not exit 0 (a real divergence or a git failure read the same way); keeping.",
            check.base_commit
        ));
    }

    // (2) branch is real. `None` is a detached HEAD — the condition that
    // stands between prune and permanent commit loss, since a detached
    // HEAD's only reflog is `.git/worktrees/<id>/logs/HEAD`, gone with the
    // directory and immediately gc-eligible. A mismatch is logged as the
    // keep-reason itself, never silently resolved either way.
    match current_branch(&worktree_root) {
        None => {
            return kept(format!(
                "{} is on a detached HEAD — its only reflog dies with the directory; keeping.",
                p(&worktree_root)
            ))
        }
        Some(actual) if actual != branch => {
            return kept(format!(
                "{} is on branch {actual}, but the workspace record says {branch} — disagreement; keeping.",
                p(&worktree_root)
            ))
        }
        Some(_) => {}
    }

    // (3) clean. A failed `git status --porcelain` keeps, exactly like
    // `perform_cleanup` (merge.rs:363) already does for the merge path.
    match git_status_porcelain(&worktree_root) {
        Err(message) => return kept(format!("{message} — keeping rather than guessing the tree is clean.")),
        Ok(status) if !js_trim(&status).is_empty() => {
            return kept(format!(
                "{} has tracked-modified or untracked files; keeping.",
                p(&worktree_root)
            ))
        }
        Ok(_) => {}
    }

    // (4) nothing precious ignored (D8a's blind spot, closed).
    if let Some(reason) = precious_state_present(&worktree_root) {
        return kept(reason);
    }

    // (5) no interrupted operation.
    if let Some(reason) = interrupted_operation_present(check.main_root, check.id) {
        return kept(reason);
    }

    // (6) no live session (D2b).
    if let Some(reason) = live_session_holds(
        check.main_root,
        check.id,
        &worktree_root,
        check.now_ms,
        check.liveness_seconds,
    ) {
        return kept(reason);
    }

    // (7) old enough.
    if let Some(reason) = too_young(check.main_root, &branch, check.now_ms, check.age_threshold_ms) {
        return kept(reason);
    }

    Verdict::Dead {
        reason: format!(
            "{branch} is merged into the resolved base, {} is clean with nothing precious ignored, no interrupted operation, no live session, and old enough.",
            p(&worktree_root)
        ),
    }
}
