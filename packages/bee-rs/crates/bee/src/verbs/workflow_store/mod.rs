// workflow_store — the lane/workflow store shared by the `state` verb group.
// LIBRARY module (no `try_native`, no probe line in verbs/mod.rs).
//
// This is the Rust home of everything the Node runtime keeps in
// lib/workflow-store.mjs, the lanes half of lib/state.mjs, the handoff
// mailbox half of lib/state.mjs, and lib/state-projection.mjs. Porting it is
// what lets verbs/state_group.rs drop its "C1 gate" (native only in a repo
// with no --lane, no lane-bound session, and zero `.bee/runtime/workflows/`
// records) and serve those verbs in EVERY repo shape.
//
// Provenance, function by function (source → this file):
//   lib/workflow-store.mjs
//     runtimeDir/workflowsDir/workflowDir/workflowStatePath → workflows_dir /
//       workflow_dir / workflow_state_path
//     requireWorkflowId              → require_workflow_id
//     defaultGateEntry/defaultGates  → default_gate_entry / default_wf_gates
//     mergeGates                     → merge_gates
//     baseWorkflowDefaults           → base_workflow_defaults
//     readWorkflowRecord             → read_workflow_record
//     listWorkflows                  → list_workflows
//     updateWorkflowAssumingLock     → update_workflow_assuming_lock(_with)
//     updateWorkflow                 → update_workflow
//     withWorkflowLock               → acquire_workflow_lock
//   lib/state.mjs (lanes)
//     lanesDir/requireLaneFeature/lanePath/defaultLaneRecord/laneRecordFrom →
//       lanes_dir / require_lane_feature / lane_path / default_lane_record /
//       lane_record_from
//     readLane / readLaneStrict / writeLane / listLanes → read_lane_display /
//       read_lane_strict / write_lane / list_lanes
//   lib/state.mjs (handoff mailbox)
//     requireHandoffWorkflowId/normalizeTargetRole/handoffMailboxDir/
//     handoffRecordPath/listHandoffMailbox/newestOpenHandoffMailboxRecord/
//     nextHandoffSeq/writeMailboxHandoff/adoptMailboxHandoff →
//       require_handoff_workflow_id / normalize_target_role /
//       handoff_mailbox_dir / handoff_record_path / list_handoff_mailbox /
//       newest_open_handoff_mailbox_record / write_mailbox_handoff /
//       adopt_mailbox_handoff
//     normalizeHandoffKind           → normalize_handoff_kind
//   lib/state-projection.mjs
//     workflowGatesToApprovedGates   → workflow_gates_to_approved_gates
//     pickNewestActiveWorkflow       → pick_newest_active_workflow
//     rebuildStateProjection         → rebuild_state_projection
//     rebuildLaneProjection          → rebuild_lane_projection
//     rebuildHandoffProjection       → rebuild_handoff_projection
//   bee.mjs
//     findGateStamp                  → find_gate_stamp
//     laneLockName/projectionLockName→ lane_lock_name / projection_lock_name
//     workflowsListSort              → workflows_list_sort
//
// SECOND-PORT NOTE (required by the campaign rule "keep one behavior, not
// two"): src/hooks/state_sync.rs already carries a faithful port of
// listWorkflows / readWorkflowRecord / mergeGates / baseWorkflowDefaults /
// workflowGatesToApprovedGates / pickNewestActiveWorkflow /
// rebuildStateProjection for the state-sync hook. Those functions are MODULE-
// PRIVATE there (`fn`, not `pub(crate)`) and state_sync.rs is outside this
// cell's touchable file set, so they are re-derived here from the same .mjs
// sources rather than imported. `agrees_with_state_sync_port_on_shared_fixtures`
// below pins the two against the exact fixtures state_sync.rs's own tests use,
// so a future divergence fails a test instead of drifting silently. Two
// deliberate refinements over the hook's copy are called out inline:
//   * a workflow record whose key is ABSENT projects as an absent key (JS
//     `{...current, feature: undefined}` is dropped by JSON.stringify), where
//     the hook writes `null`. Unreachable for any bee-created record.
//   * list_workflows REPRODUCES the skip warn natively (the hook's copy still
//     delegates on any skip). CUTOVER (2026-08-01): the last two arms — the
//     one whose Node reason embedded a V8 parse message and the one whose
//     reason embedded a libuv errno string — are native too, so NO skip
//     routes back to Node and the pre-pass that kept a delegating scan silent
//     is deleted. Same sentence, same skip-and-continue, one warn per bad
//     record. readLane / listHandoffMailbox / writeMailboxHandoff's
//     previous-cell peek likewise fail open on a corrupt file (one
//     `crate::fsutil::warn_corrupt_json` line, then readJson's own fallback),
//     and readLaneStrict's unreadable refusal now names an engine-free
//     category instead of an errno code.
//
// Locking: identical lock-name strings to Node so both runtimes interoperate
// mid-campaign — `workflow:<id>` (workflow-store.mjs withWorkflowLock),
// `lane:<feature>` (bee.mjs laneLockName), `handoff:<workflow-id>`
// (state.mjs writeMailboxHandoff/adoptMailboxHandoff), and plain `state` for
// the default projection record. crate::lock's sanitizeLockName twin hashes
// the ':' forms into distinct lock files exactly as lock.mjs does.
//
// Control root: every caller reaches here through verbs/reservations.rs's
// `prelude`, whose resolve_store_root answers NeedsNode for a linked
// worktree — so on the native path controlRootFor(root) === root and the
// msn-18c re-rooting is the identity. Callers pass plain `root`.








pub(crate) const GATE_NAMES: [&str; 4] = ["context", "shape", "execution", "review"];

const STATUS_VALUES: [&str; 3] = ["active", "paused", "closed"];

const HANDOFF_SEQ_WIDTH: usize = 4;

// ─── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests;

mod record;
mod lanes;
mod projections;
mod handoff;
pub(crate) use self::record::*;
pub(crate) use self::lanes::*;
pub(crate) use self::projections::*;
pub(crate) use self::handoff::*;
