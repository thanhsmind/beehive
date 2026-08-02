// status_full — native port of FULL `bee status` (default and --lanes-full,
// --json and text) and `bee orient` (--json and text).
//
// Provenance: bee.mjs handleStatus/buildStatus/renderStatusText (~874-1206),
// handleOrient/buildOrient/renderOrientText (~1229-1373), plus exactly the
// lib functions those consume:
//   lib/state.mjs        readState/readConfig/readOnboarding/readHandoff/
//                        bypassLevel/bypassBanner/shipVisibility/
//                        hasStaleAdvisorKey/validateModelsConfig/
//                        validateAgentFilesDrift/listLanes/readLane/
//                        resolveContext/controlRootFor/resolveProductRoot
//   lib/cells.mjs        listCells/readCell/readyCells/archivedTotals/
//                        scribingDebt/globalScribingDebt/bestScribingStampMs/
//                        tierMix/ceilingScarcityWarning
//   lib/claims.mjs       listSessionRecords/readSession/resolveSessionId/
//                        heartbeatStale/activeWorkers/readClaim/isClaimActive
//   lib/reservations.mjs listReservations (over lib/lease-store.mjs listLeases)
//   lib/decisions.mjs    activeDecisions/datamark (+ tag overlay)
//   lib/backlog.mjs      readBacklogCounts (fold + legacy table)
//   lib/reviews.mjs      listReviews/listCandidates/deriveCandidateStatus
//   lib/recovery.mjs     detectCrashCandidates/scanTranscriptRoots/
//                        readTranscriptTail/hasCleanEndTrio/lastDurableSettlement
//   lib/perf.mjs         claudeProjectsRoot/encodeProjectDir/resolveTranscript
//   lib/capture.mjs      captureQueue/pendingCaptureStubs
//   lib/worktree-store.mjs readGrants/findGrantedWorktreeForFeature
//   lib/source-identity.mjs classifySource
//   lib/fsutil.mjs       hashFile (sha256 of the lossy-utf8 STRING content)
//
// Strangler rules honored here:
//   - try_native accepts ONLY the six argv shapes below; --brief is handled
//     upstream by status_brief; anything else -> None before any output.
//   - Corrupt JSON anywhere on the snapshot path used to be Ex::Bail -> None
//     BEFORE any output. CUTOVER (2026-08-01): it FAILS OPEN instead, exactly
//     as `readJson(file, fallback)` did — `rj` buffers one
//     `bee: could not parse JSON at …` line (our wording in place of V8's)
//     and hands back the `null` fallback every caller's `!x` / `?? null`
//     guard already handled. The snapshot, its exit code and its --json
//     payload are unchanged; a corrupt lane record still emits BOTH lines
//     Node emitted (readJson's, then readLane's own).
//   - JS-exotic input (truthy non-object approved_gates spread, non-string
//     git args, ...) -> Ex::Bail as well: the Node re-run owns the edge.
//   - JS throw sites that Node CATCHES locally (buildReviewBlock /
//     buildRecoveryBlock / orientWorktreeContext try/catch) are modeled as
//     Ex::Thrown and caught at the same spots; a Thrown that would escape to
//     main()'s emitError instead bails to Node, which reproduces the error.
//   - Handler-time stderr warnings are BUFFERED in order and printed only at
//     emit (before the drift line), so a bail can never leak partial output.



use crate::roots::LinkedRoots;

use crate::state::Bail;


use serde_json::{Map, Value};

use std::cell::RefCell;




use std::path::PathBuf;



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

/// Bail = delegate to Node before any output. Thrown = a JS exception Node
/// CATCHES locally (review/recovery/orient-worktree fail-open wrappers); one
/// escaping to the top level also bails (the Node re-run reproduces it).
#[derive(Debug)]
pub(crate) enum Ex {
    Bail,
    Thrown,
}

impl From<Bail> for Ex {
    fn from(_: Bail) -> Self {
        Ex::Bail
    }
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
