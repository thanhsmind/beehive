// status_full — FULL `bee status` (default and --lanes-full, --json and
// text) and `bee orient` (--json and text).
//
// Rules honored here:
//   - try_native accepts ONLY the six argv shapes below; --brief is handled
//     upstream by status_brief; anything else -> None before any output.
//   - Corrupt JSON anywhere on the snapshot path FAILS OPEN: `rj` buffers one
//     `bee: could not parse JSON at …` line and hands back the `null`
//     fallback every caller's `!x` / `?? null` guard already handles. The
//     snapshot, its exit code and its --json payload are unchanged; a
//     corrupt lane record still emits BOTH lines (the generic one, then the
//     lane-specific one).
//   - JS-exotic input (truthy non-object approved_gates spread, non-string
//     git args, ...) -> Ex::Bail: a live delegation signal, out of scope for
//     this sweep.
//   - Throw sites CAUGHT locally (build_review_block / build_recovery_block /
//     orient_worktree_context try/catch) are modeled as Ex::Thrown and
//     caught at the same spots; a Thrown that would escape further instead
//     bails.
//   - Handler-time stderr warnings are BUFFERED in order and printed only at
//     emit (before the drift line), so a bail can never leak partial output.



use crate::roots::LinkedRoots;

use serde_json::{Map, Value};

use std::cell::RefCell;




use std::path::{Path, PathBuf};



type JMap = Map<String, Value>;

// ─── constants (state.mjs / bee.mjs) ───────────────────────────────────────


const GATE_NAMES: [&str; 4] = ["context", "shape", "execution", "review"];

const PHASES: [&str; 8] = [
    "idle", "exploring", "planning", "swarming", "reviewing", "scribing", "compounding", "grooming",
];

const KNOWN_PHASES: [&str; 9] = [
    "idle", "exploring", "planning", "swarming", "reviewing", "scribing", "compounding", "grooming",
    "compounding-complete",
];

const COMMAND_KEYS: [&str; 4] = ["setup", "start", "test", "verify"];

const WORKTREE_COMPANION_COMMAND_KEYS: [&str; 3] = [
    "worktree_companion_start", "worktree_companion_end", "worktree_companion_mount",
];

const MODEL_TIERS: [&str; 3] = ["extraction", "generation", "ceiling"];

const EFFORT_LEVELS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];

const RUNTIMES: [&str; 2] = ["claude", "codex"];

const MODEL_NORMALIZE_SLOTS: [&str; 4] = ["extraction", "generation", "review", "advisor"];

const MODEL_VALIDATE_SLOTS: [&str; 4] = ["extraction", "generation", "review", "advisor"];

const ADVICE_CLASS_SLOTS: [&str; 2] = ["advisor", "review"];

const UNSAFE_CLI_FLAGS: [&str; 6] = [
    "--yolo",
    "--dangerously-skip-permissions",
    "--dangerously-bypass-approvals-and-sandbox",
    "--full-auto",
    "-s danger-full-access",
    "--sandbox danger-full-access",
];

const ADVICE_CLASS_WRITABLE_TOKENS: [&str; 4] = [
    "-s workspace-write",
    "--sandbox workspace-write",
    "--sandbox=workspace-write",
    "danger-full-access",
];

const STALE_ADVISOR_KEY_WARNING: &str = "advisor mode was removed in 0.1.23; the top-level advisor key in .bee/config.json is ignored — delete it. (This does not affect the models.<runtime>.advisor slot, which is separate and still valid.)";

// bee.mjs ~425-432
const STALE_HANDOFF_MS: f64 = 7.0 * 24.0 * 60.0 * 60.0 * 1000.0;

const POST_EXECUTION_REVIEW_PHASES: [&str; 3] = ["scribing", "compounding", "compounding-complete"];

/// How many un-retired finished features it takes before the retirement nudge
/// is worth a line. One or two is noise a reader learns to skip; five is a
/// real drag on every orientation, and one command clears it.
const ARCHIVABLE_NUDGE_FLOOR: f64 = 5.0;

/// How many reclaimable worktrees it takes before the leak is worth a line
/// (D4, D4a) — plan.md's own words: "one stale worktree is not news."
/// `reclaimable_worktree_ids().len()` must exceed this before either surface
/// shows the block.
pub(crate) const RECLAIMABLE_WORKTREES_SHOWN_FLOOR: usize = 1;

// bee.mjs ~819-821
const CONTENTION_TAIL_MAX_BYTES: u64 = 65536;

const CONTENTION_RECENT_BUSY_LIMIT: usize = 5;

const CONTENTION_TOP_LOCKS_LIMIT: usize = 5;

// cells.mjs ~2280-2281
const CEILING_MAX_SHARE: f64 = 0.4;

const SCARCITY_MIN_TIERED: i64 = 3;

// claims.mjs
const DEFAULT_HEARTBEAT_STALE_SECONDS: f64 = 900.0;

// recovery.mjs
const DEFAULT_TAIL_MAX_BYTES: u64 = 262144;

const TERMINAL_LANE_PHASES: [&str; 2] = ["idle", "compounding-complete"];

// backlog.mjs
const BACKLOG_STATUSES: [&str; 3] = ["proposed", "in-flight", "done"];

const PBI_STATUSES: [&str; 5] = ["proposed", "in-flight", "parked", "done", "declined"];

// bee.mjs ~1229-1235
const ORIENT_PHASE_SKILL: [(&str, &str); 5] = [
    ("exploring", "bee-shaping"),
    ("planning", "bee-planning"),
    ("swarming", "bee-swarming"),
    ("scribing", "bee-capturing"),
    ("compounding", "bee-capturing"),
];

// ─── error plumbing ────────────────────────────────────────────────────────

/// Bail = delegate to Node before any output — still constructed directly at
/// JS-exotic input sites (a truthy non-object approved_gates spread; see
/// lane_record_from / store.rs's own spread). Thrown = a JS exception Node
/// CATCHES locally (review/recovery/orient-worktree fail-open wrappers); one
/// escaping to the top level also bails (the Node re-run reproduces it).
#[derive(Debug)]
pub(crate) enum Ex {
    Bail,
    Thrown,
}

type R<T> = Result<T, Ex>;

pub(crate) struct Ctx {
    root: PathBuf,
    cwd: PathBuf,
    /// `resolveRoots(process.cwd()).linked` — `None` for an ORDINARY
    /// checkout, which is every main-checkout run and every unit fixture, so
    /// the pre-flip behavior is reached by exactly the same code path.
    ///
    /// bee.mjs re-runs `resolveRoots(process.cwd())` inside each of
    /// ungrantedWorktreeNotice / grantedWorktreeContext /
    /// orientWorktreeContext because `root` alone cannot tell an ordinary
    /// checkout apart from an ungranted worktree quietly sharing main's
    /// store (its own comment at ungrantedWorktreeNotice, GH #30). Resolving
    /// it once here and threading it is equivalent — the walk is a pure read
    /// and nothing in a status/orient run mutates `.git` or the registry.
    linked: Option<LinkedRoots>,
    /// Buffered stderr lines (console.warn / process.stderr.write) in Node's
    /// emission order; printed at emit time, before the drift line.
    ///
    /// A `RefCell` so the READ helpers can warn through a shared `&Ctx`:
    /// `readJson`'s corrupt-file warning is now native (see `rj`), and
    /// threading `&mut Ctx` through every reader would have rippled into a
    /// dozen signatures for no behavioral gain. Buffering still matters —
    /// a run that bails to Node after a partial read must have emitted
    /// nothing, or the re-run would print those lines a second time.
    stderr: RefCell<Vec<String>>,
}

impl Ctx {
    fn warn(&self, line: String) {
        self.stderr.borrow_mut().push(line);
    }

    /// The linked classification, only when the current checkout is a GRANTED
    /// worktree (bee.mjs grantedWorktreeContext's own test).
    fn granted_worktree(&self) -> Option<&LinkedRoots> {
        self.linked.as_ref().filter(|l| l.granted())
    }
}

// ─── promote proposals nobody has applied (D3, kf-2) ───────────────────────
//
// `bee close` writes docs/history/<feature>/promote-proposals.md on every
// green close (kl-3); nothing has ever read it back. A proposal counts as
// APPLIED once its feature carries a recorded compounding run — the SAME
// best-scribing-stamp reading `scribing_debt` already takes here (ledger,
// then lane, then state; latest wins) — at or after the proposal file's own
// mtime; otherwise it is open work. `bee orient` (orient.rs) and the session
// preamble (hooks::session_preamble::render) both call this ONE scan —
// neither re-parses docs/history/ its own way.

pub(crate) struct UnappliedPromoteProposal {
    pub(crate) feature: String,
    pub(crate) counts: String,
    pub(crate) path: String,
}

/// The proposal file's own opening line already carries its count
/// (`… — N capped cell(s): id, id, …`, promote.rs's `promote_text`) — lift
/// just the count clause; the id list belongs to the file, not a one-line
/// surface.
fn promote_proposal_counts(text: &str) -> String {
    let first_line = text.lines().next().unwrap_or("");
    let Some(idx) = first_line.rfind(" — ") else { return "unknown count".to_string() };
    let after = &first_line[idx + " — ".len()..];
    match after.find(':') {
        Some(cut) => after[..cut].trim().to_string(),
        None => after.trim().to_string(),
    }
}

pub(crate) fn unapplied_promote_proposals(root: &Path) -> Vec<UnappliedPromoteProposal> {
    let history_dir = root.join("docs").join("history");
    let Ok(rd) = std::fs::read_dir(&history_dir) else { return Vec::new() };
    let mut features: Vec<String> = rd
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    features.sort();

    // Read once, reused across every feature — the ledger/state helpers
    // already used by `scribing_debt` (cells.rs) and its preamble twin
    // (hooks::session_preamble::store), never re-derived a third way.
    let state = crate::hooks::session_preamble::read_state(root);
    let ledger = crate::hooks::session_preamble::read_scribing_ledger(root);

    let mut out = Vec::new();
    for feature in features {
        let file = history_dir.join(&feature).join("promote-proposals.md");
        let Ok(meta) = std::fs::metadata(&file) else { continue };
        let Ok(modified) = meta.modified() else { continue };
        let mtime_ms = modified
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as f64)
            .unwrap_or(0.0);
        let feature_value = Value::String(feature.clone());
        let threshold = crate::hooks::session_preamble::best_scribing_stamp_ms(
            root,
            &feature_value,
            &ledger,
            &state,
        );
        if threshold.map(|t| t >= mtime_ms).unwrap_or(false) {
            continue; // a compounding run at or after the proposal — applied
        }
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        out.push(UnappliedPromoteProposal {
            counts: promote_proposal_counts(&text),
            path: format!("docs/history/{feature}/promote-proposals.md"),
            feature,
        });
    }
    out
}

// ─── reclaimable worktrees nobody merged or pruned (D4, D4a) ──────────────
//
// `bee worktree merge` only reaches the worktree it is called on (D1), and
// `bee worktree prune` (verbs/worktree/prune.rs) only runs when a user asks
// for it — so a worktree abandoned outside `merge` sits forever unless
// something announces it. This is that announcement, kept cheap on purpose:
// the grants file is one small JSON read (no git, ever) and every candidate
// gets exactly one `metadata()` call — no `read_dir` walk of the worktree
// itself, no directory-size sum (that cost is `prune`'s to pay, D4a, once
// the user actually asks). A grant whose directory is gone already fails the
// single `metadata()` call, so "directory exists" and "old enough" are one
// check, not two. `bee orient` (orient.rs) and the session preamble
// (hooks::session_preamble::render) both call this ONE scan — the same
// single-scan discipline `unapplied_promote_proposals` holds just above.
//
// Age reuses `verbs::worktree::DEFAULT_PRUNE_AGE_DAYS` rather than a second
// number: a worktree this line calls "reclaimable" is the same one
// `bee worktree prune --dry-run` — the command the line names — would judge
// old enough, modulo prune's own further mergedness/liveness/clean-tree
// checks (which DO need git, and stay entirely inside `prune`, never here).
pub(crate) fn reclaimable_worktree_ids(root: &Path) -> Vec<String> {
    let grants_file = root.join(".bee").join("runtime").join("worktree-grants.json");
    let Ok(bytes) = std::fs::read(&grants_file) else { return Vec::new() };
    let Ok(Value::Object(grants)) = serde_json::from_slice::<Value>(&bytes) else {
        return Vec::new();
    };
    let Some(worktree_parent) = root.parent() else { return Vec::new() };
    let threshold = std::time::Duration::from_secs_f64(
        crate::verbs::worktree::DEFAULT_PRUNE_AGE_DAYS * 24.0 * 60.0 * 60.0,
    );
    let now = std::time::SystemTime::now();
    let mut ids: Vec<String> = grants
        .iter()
        .filter(|(_, v)| **v == Value::Bool(true))
        .filter_map(|(id, _)| {
            let meta = std::fs::metadata(worktree_parent.join(id)).ok()?;
            let modified = meta.modified().ok()?;
            let age = now.duration_since(modified).ok()?;
            (age >= threshold).then(|| id.clone())
        })
        .collect();
    ids.sort();
    ids
}

#[cfg(test)]
mod tests;

mod jsval;
mod store;
mod cells;
mod records;
mod recovery;
mod topology;
mod build;
mod render;
mod orient;
pub(crate) use self::jsval::*;
pub(crate) use self::store::*;
pub(crate) use self::cells::*;
pub(crate) use self::records::*;
pub(crate) use self::recovery::*;
pub(crate) use self::topology::*;
pub(crate) use self::build::*;
pub(crate) use self::render::*;
pub(crate) use self::orient::*;
