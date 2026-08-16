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
// wr-3 wires the subcommand (`run_prune`, below) into `handlers.rs`'s
// `try_native`, calling `classify_worktree` per enumerated id and wr-1's
// `teardown_worktree` for each dead one. Nothing here bypasses the
// classifier: `run_prune_core` reads a `Verdict` for every id and only ever
// removes the ones that came back `Dead`.
#![allow(unused_imports)]
#![allow(dead_code)]

use super::*;
use crate::jsjson;
use crate::lock;
use crate::verbs::reservations::{js_trim, FlagV, Flags};
use crate::verbs::workspace_store as ws;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

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
///
/// `Dead.orphan` marks the one Dead shape that never had a directory or a
/// branch to remove in the first place (the orphan verdict, below) — the
/// removal step in `run_prune_core` reads this flag to skip `git worktree
/// remove`/`git branch -d` entirely rather than run them against artifacts
/// that were never there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Verdict {
    Dead { reason: String, orphan: bool },
    Kept { reason: String },
}

impl Verdict {
    pub(crate) fn is_dead(&self) -> bool {
        matches!(self, Verdict::Dead { .. })
    }
    pub(crate) fn reason(&self) -> &str {
        match self {
            Verdict::Dead { reason, .. } | Verdict::Kept { reason } => reason,
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
pub(crate) fn live_session_holds(
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

        if matches!(record.get("status"), Some(Value::String(s)) if s == "closed" || s == "dead") {
            continue; // a closed/dead session never holds a worktree.
        }

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

    // (0a) orphan: no directory AND no branch. Evaluated BEFORE the merge
    // test (1) on purpose — `git merge-base --is-ancestor` can never exit 0
    // for a branch that does not exist, so condition (1) alone would keep a
    // record like this FOREVER, misreading a real absence as "not provably
    // merged" and asking an ancestry question that has no answer. The two
    // conditions are a conjunction, each checked on its own: a missing
    // directory with a branch that still exists keeps below at (1) or (2) —
    // the branch may carry commits no ref elsewhere protects; a standing
    // directory with a branch that is gone keeps below at (1) — the tree may
    // hold uncommitted or ignored work the branch never saw. Only when
    // NEITHER artifact remains does this fire: the workspace record is the
    // only thing left, so there is no directory to remove, no commits to
    // strand, and no ignored files to lose. `run_prune_core`'s removal step
    // reads `Verdict::Dead.orphan` to skip `git worktree remove`/`git branch
    // -d` entirely for this verdict — neither target exists to run them on.
    if !worktree_root.exists() && !branch_exists(check.main_root, &branch) {
        return Verdict::Dead {
            reason: format!(
                "neither {} nor branch {branch} exists — the workspace record is the only artifact left, so nothing could be lost by dropping it.",
                p(&worktree_root)
            ),
            orphan: true,
        };
    }

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
        orphan: false,
    }
}

// ─── `bee worktree prune` (wr-3, D2/D5) ────────────────────────────────────

/// Days, not the classifier's own `PRUNE_LIVENESS_SECONDS` (six hours) — a
/// merged, clean, session-free worktree can still be one its owner means to
/// come back to tomorrow. A week gives real headroom past the liveness
/// window while still catching CONTEXT.md's leak (worktrees idle for
/// months, not hours). `--older-than-days` overrides it per run; no config
/// key backs it (this cell touches no config file).
pub(crate) const DEFAULT_PRUNE_AGE_DAYS: f64 = 7.0;

const MS_PER_DAY: f64 = 24.0 * 60.0 * 60.0 * 1000.0;

/// The permanent, git-enforced opt-out (D2/D5): `teardown_worktree`'s own
/// first step, `git worktree remove --force`, dies outright on a locked
/// tree, so naming it once is the whole promise — prune needs no lock
/// bookkeeping of its own to honour it.
const PRUNE_LOCK_HINT: &str = "Lock a worktree to opt it out of prune permanently: \"git worktree lock <path>\" — \"git worktree remove --force\" refuses on a locked tree.";

fn dir_size_bytes(path: &Path) -> u64 {
    let Ok(md) = std::fs::symlink_metadata(path) else { return 0 };
    if md.file_type().is_symlink() {
        return 0;
    }
    if !md.is_dir() {
        return md.len();
    }
    let Ok(entries) = std::fs::read_dir(path) else { return 0 };
    entries.flatten().map(|e| dir_size_bytes(&e.path())).sum()
}

/// Base-2, one decimal place past bytes — CONTEXT.md's own "4.5 GB" shape.
fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Grants **union** workspace records (CONTEXT.md's out-of-scope line,
/// closed here): `worktree unregister` drops the grant but — until D3/D3a —
/// never the record, so 13 orphan `.bee/runtime/workspaces/*.json` files
/// stayed behind with an empty grants file. A grant-only scan never reaches
/// one. Every `type: "worktree"` record enters the run even with no
/// matching grant; `"main"`'s own record never does.
fn enumerate_worktree_ids(main_root: &Path) -> Result<BTreeSet<String>, String> {
    let main_store_root = main_root.join(".bee");
    let grants = read_grants_strict(&main_store_root).ok_or_else(|| {
        format!(
            "the worktree grants registry at {} does not parse as JSON — refusing rather than guess which ids are granted.",
            p(&grants_file(&main_store_root))
        )
    })?;
    let mut ids: BTreeSet<String> = grants
        .iter()
        .filter(|(_, v)| **v == Value::Bool(true))
        .map(|(k, _)| k.clone())
        .collect();
    let (workspaces, _skipped) = ws::list_workspaces(main_root).unwrap_or_else(|_| (vec![], vec![]));
    for record in workspaces {
        if record.get("type") != Some(&Value::String("worktree".to_string())) {
            continue;
        }
        if let Some(Value::String(id)) = record.get("id") {
            ids.insert(id.clone());
        }
    }
    Ok(ids)
}

/// The `root`/`branch` pair a DEAD verdict's removal needs, re-read fresh
/// rather than threaded out of `classify_worktree` — the classifier's job is
/// an answer, not a side channel. A record that has gone missing or lost
/// either field BETWEEN classification and this read is a race, not a
/// removal: `None` here keeps the worktree for this run rather than
/// guessing where it lives.
fn dead_worktree_target(main_root: &Path, id: &str) -> Option<(PathBuf, String)> {
    let record = ws::read_workspace(main_root, id).ok()?;
    let root = match record.get("root") {
        Some(Value::String(s)) if !s.is_empty() => PathBuf::from(s),
        _ => return None,
    };
    let branch = match record.get("branch") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => return None,
    };
    Some((root, branch))
}

/// The whole run's answer: `run_prune`'s flag-parsing/emission wrapper reads
/// this to build both the JSON result and the text report; a test reads it
/// directly, with no CLI dispatch or `std::env::current_dir()` in the way.
pub(crate) struct PruneOutcome {
    pub(crate) base_ref: String,
    pub(crate) base_commit: String,
    pub(crate) dry_run: bool,
    pub(crate) age_threshold_days: f64,
    pub(crate) entries: Vec<Value>,
    pub(crate) removed_ids: Vec<String>,
    pub(crate) kept_ids: Vec<String>,
    pub(crate) reclaimed_bytes: u64,
    pub(crate) lines: Vec<String>,
}

/// The whole prune run, minus argv parsing and emission (`run_prune` below
/// wraps this). `main_root` is the caller's own already-resolved ordinary
/// checkout; this function runs no root resolution of its own, so a test can
/// hand it any tempdir main root without touching the process's real cwd.
pub(crate) fn run_prune_core(
    main_root: &Path,
    dry_run: bool,
    age_threshold_days: f64,
) -> Result<PruneOutcome, String> {
    // The base ref: the MAIN checkout's own current branch — the ref every
    // merged `wt/*` branch is measured against (CONTEXT.md's own evidence:
    // "every wt/* branch measured ahead=0 against main"). Prune carries no
    // `--base-ref` flag (plan.md's signature is dry-run/older-than-days/json
    // only) and no config key names one either, so a detached main HEAD
    // refuses the whole run rather than guessing a ref (D2a's own shape:
    // a base ref that does not resolve refuses outright).
    let base_ref = current_branch(main_root).ok_or_else(|| {
        format!(
            "{} is on a detached HEAD — prune needs a named branch to measure mergedness against; checkout a branch there and retry.",
            p(main_root)
        )
    })?;
    let base_commit = resolve_prune_base(main_root, &base_ref)?;
    let ids = enumerate_worktree_ids(main_root)?;

    let now_ms = crate::verbs::reservations::now_ms();
    let age_threshold_ms = age_threshold_days * MS_PER_DAY;

    let mut entries = Vec::new();
    let mut lines = Vec::new();
    let mut removed_ids = Vec::new();
    let mut kept_ids = Vec::new();
    let mut reclaimed_bytes: u64 = 0;

    for id in &ids {
        let verdict = classify_worktree(&PruneCheck {
            main_root,
            id,
            base_commit: &base_commit,
            now_ms,
            liveness_seconds: PRUNE_LIVENESS_SECONDS,
            age_threshold_ms,
        });

        match verdict {
            Verdict::Kept { reason } => {
                kept_ids.push(id.clone());
                lines.push(format!("{id}: kept — {reason}"));
                entries.push(json!({"id": id, "verdict": "kept", "removed": false, "reason": reason}));
            }
            Verdict::Dead { reason, orphan: true } => {
                // Neither a directory nor a branch exists — nothing for
                // `git worktree remove` or `git branch -d` to run against.
                // Skip both and drop straight to the registry-only teardown
                // `run_unregister` already uses (`teardown_worktree(..,
                // None)`, registry.rs) — it runs no git mutation, so it
                // cannot fail the way the directory/branch removal below can.
                if dry_run {
                    lines.push(format!("{id}: would remove, 0 B reclaimable — {reason}"));
                    entries.push(json!({
                        "id": id, "verdict": "dead", "removed": false,
                        "reclaimable_bytes": 0, "reason": reason,
                    }));
                    continue;
                }
                let mut guard = match lock::acquire_store_lock(main_root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS) {
                    Ok(g) => g,
                    Err(busy) => {
                        kept_ids.push(id.clone());
                        let kept_reason = format!(
                            "was classified dead ({reason}) but the worktree-admin lock is busy ({}) — kept this run, retry.",
                            busy.message()
                        );
                        lines.push(format!("{id}: kept — {kept_reason}"));
                        entries.push(json!({"id": id, "verdict": "kept", "removed": false, "reason": kept_reason}));
                        continue;
                    }
                };
                let _ = teardown_worktree(main_root, id, None);
                guard.release();
                removed_ids.push(id.clone());
                lines.push(format!(
                    "{id}: removed (record only — no directory or branch existed), reclaimed 0 B — {reason}"
                ));
                entries.push(json!({
                    "id": id, "verdict": "dead", "removed": true,
                    "reclaimed_bytes": 0, "reason": reason,
                }));
            }
            Verdict::Dead { reason, orphan: false } => {
                let Some((worktree_root, branch)) = dead_worktree_target(main_root, id) else {
                    // Lost between classification and removal — a race, not
                    // a reason to guess; keep for this run.
                    kept_ids.push(id.clone());
                    let kept_reason = format!(
                        "was classified dead ({reason}) but its workspace record could not be re-read for removal — keeping rather than guessing."
                    );
                    lines.push(format!("{id}: kept — {kept_reason}"));
                    entries.push(json!({"id": id, "verdict": "kept", "removed": false, "reason": kept_reason}));
                    continue;
                };
                let size = dir_size_bytes(&worktree_root);

                if dry_run {
                    // `--dry-run` classifies and removes NOTHING: no lock, no
                    // git mutation, no registry write — the size below is
                    // reclaimABLE, never reclaimed.
                    reclaimed_bytes += size;
                    lines.push(format!("{id}: would remove, {} reclaimable — {reason}", format_bytes(size)));
                    entries.push(json!({
                        "id": id, "verdict": "dead", "removed": false,
                        "reclaimable_bytes": size, "reason": reason,
                    }));
                    continue;
                }

                let mut guard = match lock::acquire_store_lock(main_root, WORKTREE_ADMIN_LOCK, lock::MAX_ATTEMPTS) {
                    Ok(g) => g,
                    Err(busy) => {
                        kept_ids.push(id.clone());
                        let kept_reason = format!(
                            "was classified dead ({reason}) but the worktree-admin lock is busy ({}) — kept this run, retry.",
                            busy.message()
                        );
                        lines.push(format!("{id}: kept — {kept_reason}"));
                        entries.push(json!({"id": id, "verdict": "kept", "removed": false, "reason": kept_reason}));
                        continue;
                    }
                };
                let teardown = teardown_worktree(main_root, id, Some((&worktree_root, &branch)));
                guard.release();

                match teardown {
                    Ok(()) => {
                        removed_ids.push(id.clone());
                        reclaimed_bytes += size;
                        lines.push(format!("{id}: removed, reclaimed {} — {reason}", format_bytes(size)));
                        entries.push(json!({
                            "id": id, "verdict": "dead", "removed": true,
                            "reclaimed_bytes": size, "reason": reason,
                        }));
                    }
                    Err(TeardownFailure::RemoveFailed(why)) => {
                        kept_ids.push(id.clone());
                        let kept_reason =
                            format!("was dead ({reason}) but \"git worktree remove --force\" failed: {why}");
                        lines.push(format!("{id}: kept — {kept_reason}"));
                        entries.push(json!({"id": id, "verdict": "kept", "removed": false, "reason": kept_reason}));
                    }
                    Err(TeardownFailure::BranchDeleteFailed(why)) => {
                        // The directory (and its reflog) is already gone —
                        // this is a removal with a caveat, not a keep.
                        removed_ids.push(id.clone());
                        reclaimed_bytes += size;
                        lines.push(format!(
                            "{id}: removed (directory only — \"git branch -d\" failed: {why}), reclaimed {} — {reason}",
                            format_bytes(size)
                        ));
                        entries.push(json!({
                            "id": id, "verdict": "dead", "removed": true, "branch_deleted": false,
                            "reclaimed_bytes": size, "reason": reason,
                        }));
                    }
                }
            }
        }
    }

    if ids.is_empty() {
        lines.push("No granted or recorded worktrees to classify.".to_string());
    }
    lines.push(PRUNE_LOCK_HINT.to_string());

    Ok(PruneOutcome {
        base_ref,
        base_commit,
        dry_run,
        age_threshold_days,
        entries,
        removed_ids,
        kept_ids,
        reclaimed_bytes,
        lines,
    })
}

/// `bee worktree prune [--dry-run] [--older-than-days N] [--json]` — the
/// subcommand over `classify_worktree` (wr-2), routed at `handlers.rs`'s
/// `try_native`. Every dead-or-kept decision is `run_prune_core`'s; this
/// wrapper only parses argv and renders the answer.
pub(crate) fn run_prune(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !crate::verbs::reservations::keys_known(&flags, &["dry-run", "older-than-days"]) {
        return None;
    }
    if !bool_flag_ok(&flags, "dry-run") {
        return None;
    }
    let dry_run = bool_flag_true(&flags, "dry-run");

    let mut age_threshold_days = DEFAULT_PRUNE_AGE_DAYS;
    match flags.get("older-than-days") {
        None => {}
        Some(FlagV::Present) => return None, // a bare flag is not a number
        Some(FlagV::S(raw)) => {
            if js_trim(raw).is_empty() {
                return None;
            }
            let n = js_string_to_number(raw);
            if !n.is_finite() || n < 0.0 {
                return None; // a non-finite or negative age makes no sense; refuse rather than guess
            }
            age_threshold_days = n;
        }
    }

    let ctx = match prelude("worktree prune", use_json, t0)? {
        Pre::Go(c) => c,
        Pre::Emitted(code) => return Some(code),
    };
    if ctx.kind != "ordinary" {
        return Some(ctx.fail(&format!(
            "\"bee worktree prune\" must be run from the MAIN checkout, not a \"{}\" checkout — it enumerates and removes OTHER worktrees, and a linked worktree cannot prune itself.",
            ctx.kind
        )));
    }
    let main_root = ctx.work_root.clone();

    let outcome = match run_prune_core(&main_root, dry_run, age_threshold_days) {
        Ok(o) => o,
        Err(message) => return Some(ctx.fail(&message)),
    };

    let mut result = Map::new();
    result.insert("main_root".into(), json!(p(&main_root)));
    result.insert("base_ref".into(), json!(outcome.base_ref));
    result.insert("base_commit".into(), json!(outcome.base_commit));
    result.insert("dry_run".into(), json!(outcome.dry_run));
    result.insert("age_threshold_days".into(), json!(outcome.age_threshold_days));
    result.insert("worktrees".into(), Value::Array(outcome.entries.clone()));
    result.insert("removed".into(), json!(outcome.removed_ids));
    result.insert("kept".into(), json!(outcome.kept_ids));
    if dry_run {
        result.insert("reclaimable_bytes".into(), json!(outcome.reclaimed_bytes));
    } else {
        result.insert("reclaimed_bytes".into(), json!(outcome.reclaimed_bytes));
    }

    let text = outcome.lines.join("\n");
    Some(ctx.emit(&Value::Object(result), &text))
}
