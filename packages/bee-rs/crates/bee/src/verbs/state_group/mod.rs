// bee state — the `state` verb group.
//
// R6 coverage debt "the lane/workflow world" — CLOSED for the record-mutating
// verbs. Through R3 wave 2 these verbs served natively ONLY in the "C1 world"
// (no --lane flag, no session-bound lane, zero records under
// .bee/runtime/workflows/). That gate is GONE: verbs/workflow_store.rs now
// carries the lane store, the workflow store, the handoff mailbox, and the
// projection builders, so lane-targeted and workflow-carrying repos take the
// same native path a bare repo does — same lock names, same projection
// write-through, same bytes.
//
// Ported argv shapes (everything else returns None BEFORE any output and the
// whole command re-runs under Node):
//   state set / gate / scribing-run / compounding-run / plan-rev bump
//     — native in EVERY repo shape (explicit --lane, a session-bound lane, or
//       the default record; with or without live workflow records). The full
//       Node seam is reproduced: resolveMutationLockScope's fail-open peek,
//       withMutationLock's `workflow:<id>` → {'state' | lane:<f>} nesting,
//       resolveMutationTarget's strict reads, and
//       writeLaneRecordThroughProjection / writeStateRecordThroughProjection
//       (updateWorkflowAssumingLock + rebuild). Deterministic refusals — arg
//       validation, the chain-integrity gate doors, owner checks,
//       readStateStrict/readLaneStrict's typed errors, the LANE_MISSING
//       refusals, the scribing-run/compounding-run phase doors, plan-rev
//       bump's four refusals — are all native.
//       Still delegated INSIDE these verbs (each is a different R6 debt, not
//       this one): a PASSING `--phase compounding-complete` close (its
//       scribing-debt door + waiver decision-logging live in cells.mjs /
//       decisions.mjs — default AND lane branches), and a high-risk
//       execution/merge approval (advisorRefStale, lib/state.mjs).
//   state worker add / update / remove / clear / prune — always native for
//     known flag shapes (they never consult lanes/sessions/workflows).
//   state lanes / session list / session bind / session unbind / session
//     release — native. `session release` marks an OPEN session
//     `status: "closed"`, `released: true` (instant lock release, no lane
//     change) — `state_sync.rs`'s heartbeat leaves a `released` mark intact
//     while `prompt_context.rs`'s UserPromptSubmit revival clears it.
//   state scribing-run --show — native (read-only ledger/lane/state query).
//   state handoff write / adopt / show — native in every repo shape: the
//     legacy .bee/HANDOFF.json path when resolveHandoffWorkflowId answers
//     null (C1), and the per-workflow MAILBOX path
//     (.bee/runtime/handoffs/<workflow-id>/NNNN.json + the legacy-file
//     projection rebuild) when a workflow resolves.
//   state workflows list / close — native (listWorkflowRecords /
//     closeWorkflowsForFeature + the three mutually exclusive close modes).
//
//   state rebuild-projections — NATIVE (R6). The one seam that holds more
//     than one projection lock: 'state' then every active workflow's
//     `lane:<feature>`, sorted + de-duplicated, then rebuildAllProjections
//     (state/handoff/reservations/lanes). reservations.mjs's
//     rebuildReservationsProjection now lives in verbs/reservations.rs beside
//     its own `list_reservations`.
//   state route — NATIVE (R6) for `--show` in every repo, and for `--set`
//     in every repo whose worktree-grants registry has no `true` entry (see
//     the `state route` section header: buildRouteWorktreeBlock's
//     findGrantedWorktreeForFeature walk lives in verbs/status_full.rs and is
//     not reachable from here, so a repo that HAS a granted worktree AND
//     records a code-touching lane returns None before any lock or write).
//
//   state start-feature — NATIVE (R6), BOTH branches: the default pipeline
//     and `--as-lane`. seedLegacyWorkflows (C1), applyWritePolicy's three
//     modes (observe / shared-disjoint / isolated — owner, refusal, and the
//     CONSENTED isolate-create's real `git worktree add`), startLane's five
//     preconditions, the default body's six, the shared workflow-precondition
//     layer, ensureWorkflowRecordForFeature, closeWorkflowsForFeature and
//     bee.mjs's own projection rebuild all run here. Both recorded blockers
//     have dissolved — see the `state start-feature` section header below for
//     which port retired each, and for the delegation gates that keep the
//     "nothing after the first write can fall back" property true.
//
//   state advisor-ref record / show — HELPERS AND VERBS ARE NATIVE
//     (advisor_ref.rs, cell agp-1): advisorRefAnchors/advisorRefStale (AO13,
//     no TTL) and both handlers, ported verbatim from lib/state.mjs /
//     bee.mjs. NOT YET REACHABLE from `try_native`'s routing table below,
//     though — that match arm lands with the Gate 3 precondition in agp-2
//     (set_gate.rs, a separate cell by explicit scope boundary), so the
//     registry keeps advertising both as `unavailable` until that wiring
//     lands too. `show` widens the deleted JS's target resolution (explicit
//     `--lane` only) to the standard selector every other read here already
//     gives a caller: `--lane` wins, else the session's bound lane, else the
//     default record; `--no-lane` forces the default.
//
// DELEGATED whole verbs (unprovable here, by design):
//   * state.compact-*.
//
// Provenance: bee.mjs handleStateSet/handleStateGate/handleStatePlanRevBump/
// stateWorkerMutate + worker handlers/readPruneKeepSet/keptByPruneKeepSet/
// handleStateScribingRun/handleStateCompoundingRun/handleStateLanes/
// handleStateSessionList/Bind/Unbind/handleStateHandoffWrite/Adopt/Show/
// resolveHandoffWorkflowId/mutationLaneSelector/optionalLaneFlag/
// resolveMutationTarget/resolveMutationLockScope/requireFlag/requireFlags/
// exampleFor/splitList/WORKER_TRANSIENT_SUFFIX; lib/state.mjs readStateStrict/
// writeState/defaultState/coerceLegacyPhase/checkPhaseTransition/
// checkScribingRunPhase/checkCompoundingRunPhase/isKnownPhase/readState/
// readHandoff/writeHandoff/adoptHandoff/normalizeHandoffKind/readLane/
// laneRecordFrom/defaultLaneRecord/listLanes; lib/claims.mjs requireId/
// sessionsDir/readSession/listSessionRecords/heartbeatStale/resolveSessionId/
// bindSessionLane/unbindSessionLane/readClaim/adoptClaim (gate file +
// fence_epoch); lib/cells.mjs scribingLedgerPath/readScribingLedger/
// appendScribingLedger/bestScribingStampMs/scribingRunStampMs.
//
// Locking: identical lock-name strings — the mutation verbs follow bee.mjs's
// global order `workflow:<id>` → {'state' | lane:<feature>} (withMutationLock),
// falling back to a single "state" hold when no live workflow names the
// target; the worker verbs hold "state" alone; the handoff mailbox holds
// `handoff:<workflow-id>`; session bind/unbind/release hold "sessions" through
// claims.mjs's bounded 15×20ms acquire-once loop; adoptClaim uses the
// per-claim `<cell>.adopting` gate file (no store lock). worker prune takes
// no lock (read-only on state). All waits are lock.rs's 100×50ms
// withStoreLock, with LockBusyError's bytes reproduced natively.
//
// CUTOVER (2026-08-01) — the corrupt-JSON delegations are GONE. Contract C2
// required byte-identical output with Node, so every read whose warning or
// refusal would have interpolated a V8 `JSON.parse` message or a libuv errno
// string returned "ask Node" instead of doing the work. Node is being
// deleted, so those arms are native now, with our own wording and the SAME
// semantics:
//   * readState / readSession / readClaim / readHandoff / listAllCellsForStart
//     and writeHandoff's previous-cell peek all FAIL OPEN exactly as
//     `readJson(file, fallback)` did — one `bee: could not parse JSON at …`
//     warning per read (crate::fsutil::warn_corrupt_json), then the same
//     fallback (defaultState / null / skip-the-record).
//   * readStateStrict still REFUSES with the same typed message and exit
//     code; only the `(EISDIR)`-style parenthetical is now an engine-free
//     category (`io_read_reason`), and every errno class gets one instead of
//     half of them delegating.
//   * parse_json_v8 no longer delegates on lone-surrogate "\u" escapes —
//     nothing in this process can decode them, so such input is CORRUPT and
//     takes each caller's corrupt branch.
//
// Known accepted approximations (documented, delegation guards the rest):
// prune's mid-loop rmSync failure message is reconstructed from the errno
// class; the scribing-ledger append-failure warning (embeds a Node error
// message) is not replicated — the append virtually never fails and the
// verb's own success output is unaffected. `approved_gates` holding a string
// or array is CLOSED (D2, docs/history/js-parity-cleanup/CONTEXT.md) —
// spread_gates merges only the object shape and falls back to defaults for
// everything else, natively, no delegate. STILL delegating and out of this
// cutover's scope: the passing-close / feature-swap / high-risk approval
// doors, and fs WRITE failures after the preflight.








use serde_json::Value;





// ─── enums (state.mjs) ─────────────────────────────────────────────────────

const KNOWN_PHASES: [&str; 9] = [
    "idle", "exploring", "planning", "swarming", "reviewing", "scribing", "compounding",
    "grooming", "compounding-complete",
];

const KNOWN_PHASES_JOINED: &str =
    "idle, exploring, planning, swarming, reviewing, scribing, compounding, grooming, compounding-complete";
const GATE_NAMES: [&str; 5] = ["context", "shape", "execution", "review", "uat"];

const MODEL_TIERS: [&str; 3] = ["extraction", "generation", "ceiling"];

const SCRIBING_RUN_FROM: [&str; 3] = ["swarming", "reviewing", "scribing"];

const HEARTBEAT_STALE_SECONDS: f64 = 900.0;

fn is_known_phase(p: &str) -> bool {
    KNOWN_PHASES.contains(&p)
}

// exampleFor(command) — registry examples[0] for the requireFlags callers.
const EXAMPLE_GATE: &str = "bee state gate --name execution --approved true --json";

const EXAMPLE_SCRIBING: &str =
    "bee state scribing-run --feature newf --areas auth --next-action bee-capturing --json";
const EXAMPLE_WORKFLOWS_CLOSE: &str = "bee state workflows close --feature stale-feature --json";

const EXAMPLE_COMPOUNDING: &str =
    "bee state compounding-run --feature newf --learnings docs/history/newf/reports/learnings.md --json";

// ─── tiny JS-semantics helpers ─────────────────────────────────────────────

/// `a === b` where either side may be undefined (None). undefined === undefined
/// is true in JS.
fn opt_strict_eq(a: Option<&Value>, b: Option<&Value>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

// ─── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;

mod flags;
mod store;
mod ledger;
mod set_gate;
mod workers;
mod sessions;
mod workflows;
mod feature;
mod policy;
mod advisor_ref;
mod waiting_on;
mod plan_conflicts;
pub(crate) use self::flags::*;
pub(crate) use self::store::*;
pub(crate) use self::ledger::*;
pub(crate) use self::set_gate::*;
pub(crate) use self::workers::*;
pub(crate) use self::sessions::*;
pub(crate) use self::workflows::*;
pub(crate) use self::feature::*;
pub(crate) use self::policy::*;
pub(crate) use self::advisor_ref::*;
pub(crate) use self::waiting_on::*;
pub(crate) use self::plan_conflicts::*;
