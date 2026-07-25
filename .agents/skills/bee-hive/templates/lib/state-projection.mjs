// state-projection.mjs — read-only compatibility projections of workflow
// records onto the legacy single-pipeline stores (.bee/state.json,
// .bee/lanes/<feature>.json), per CONTEXT.md D1 (multisession-native-7,
// advisor consult slice 2 conditions C1/C5, finding F8).
//
// A workflow record (workflow-store.mjs) is the unit of coordination state
// from multisession-native-6 (msn-6) onward — every startFeature call
// (default or lane) creates one. This module is the ONE place that turns the
// current set of workflow records back into the two legacy shapes every
// existing reader (status, prompt-context, guards, resolvePipeline/
// resolveMutationTarget) still expects, so those readers keep working
// unchanged (D1 must_have: "existing readers ... see unchanged shapes").
//
// C1 fallback (advisor consult slice 2, binding — the read-side half of
// msn-6's own seedLegacyWorkflows, which is the WRITE-side seed): when ZERO
// workflow records exist anywhere in the repo, every rebuild function below
// is a NO-OP — it never writes, and returns `authoritative: false` plus
// whatever the legacy file already holds untouched. The projection layer
// only takes authority once at least one workflow record exists. A repo
// that has never called startFeature since msn-6 landed keeps behaving
// exactly as it did before this cell: state.json/lanes stay hand-written by
// their existing callers, byte-identical. This module never deletes or
// rewrites a legacy record that has no corresponding workflow record.
//
// F8 (binding): every rebuild here is FULL, never a partial patch — each
// call recomputes every D1-owned field (phase/feature/mode/approved_gates/
// summary/next_action) together from the SAME source workflow record, never
// one field at a time. Fields this module does not yet own — state.json's
// `workers` array (D6/multisession-native-8 retires it) and its ad hoc
// `cells`/`last_activity` hook fields — pass through UNCHANGED from
// whatever is already on disk, or via an explicit override (bee-state-sync
// uses this to refresh counts/timestamp in the SAME write as the D1 rebuild,
// never a second one).
//
// F8 self-heal (also binding): a workflow record and its lane projection can
// diverge only through a crash inside msn-6's own documented three-
// transaction window (seed / legacy write / workflow create are separate
// transactions). Every rebuild here re-derives the projection ENTIRELY from
// the CURRENT record — "record wins" — so calling rebuildLaneProjection or
// rebuildStateProjection again always self-heals any such divergence; there
// is no separate repair path to keep in sync.
//
// C5 residual seam (documented, binding — scoped honestly per the advisor
// consult, not swept under the rug): the DEFAULT (non-lane) state.json
// mutation path — bee.mjs's resolveMutationTarget default branch, which
// still calls writeState(root, record) directly, and every verb built on it
// (state set/gate/scribing-run/advisor-ref record without --lane) — is NOT
// rerouted through this module in this cell. That is multisession-native-10.
// Only LANE projections are rerouted in this cell: bee.mjs's lane branches
// of resolveMutationTarget, and the lane path of `bee state start-feature
// --as-lane` (handleStateStartFeature) — see those call sites' own comments
// for exactly how each routes through rebuildLaneProjection below.
//
// Consequence for rebuildStateProjection specifically (see its own doc
// comment for the full reasoning): because C5 leaves the default path's
// workflow record stale from the moment of creation onward, this module
// only ever ADOPTS a workflow record's fields into .bee/state.json while it
// is genuinely IDLE — never overwrites a LIVE default record. Lane
// projections carry no such caveat: every lane mutation in this cell is
// rerouted through its workflow record, so a lane's record is always kept
// in sync and safe to treat as fully authoritative.

import { readState, writeState, defaultState, readLane, writeLane, GATE_NAMES } from './state.mjs';
import { listWorkflows } from './workflow-store.mjs';

/** True once at least one workflow record exists anywhere in the repo — the C1 authority switch. */
export function projectionsAuthoritative(root) {
  return listWorkflows(root).workflows.length > 0;
}

// workflowGatesToApprovedGates — the D1 workflow gates map (per-name
// {approved, approved_for_plan_rev}) -> the legacy boolean approved_gates
// shape, in the FIXED GATE_NAMES key order (deterministic JSON output —
// never derived from Object.entries' insertion order on the workflow
// record, which is not guaranteed stable across records).
//
// multisession-native-9 (D7, C2 — advisor consult slice 2, binding): the
// projected boolean is now the PLAN-REV-EFFECTIVE approval, not the bare
// `approved` flag — `approved && (approved_for_plan_rev == null ||
// approved_for_plan_rev === planRev)`. `approved_for_plan_rev === null` (or
// the field being absent — a hand-written/legacy record predating this
// cell, or msn-6's own seedLegacyWorkflows) means "not rev-scoped" and is
// ALWAYS effective, independent of `planRev` — this is what keeps the
// context/shape/review gates immune to a plan_rev bump by construction
// (CONTEXT.md D7 default: only the execution gate is ever stamped with a
// real rev number — see bee.mjs's handleStateGate/writeLaneRecordThroughProjection).
// `planRev` is OPTIONAL and defaults to `undefined`: a caller that omits it
// (e.g. a bare structural translation with no live workflow record's
// plan_rev in hand) gets the SAME rev-immune behavior only for `null`/absent
// revs; a gate stamped with an explicit rev number never matches `undefined`,
// so it reads as ineffective — callers that DO have a plan_rev (every
// production caller below) must pass it for a stamped gate to project true.
export function workflowGatesToApprovedGates(gates, planRev) {
  const approved = {};
  for (const name of GATE_NAMES) {
    const entry = gates && gates[name];
    const isApproved = Boolean(entry && entry.approved === true);
    const rev = entry ? entry.approved_for_plan_rev : undefined;
    const revEffective = rev === null || rev === undefined || rev === planRev;
    approved[name] = isApproved && revEffective;
  }
  return approved;
}

// pickNewestActiveWorkflow — "the newest ACTIVE workflow" (msn-7 cell
// contract): status === 'active' only (paused/closed workflows never become
// the default projection's source); ties broken by created_at descending,
// then id descending, so two records created within the same millisecond
// still resolve deterministically.
export function pickNewestActiveWorkflow(workflows) {
  const active = (workflows || []).filter((wf) => wf && wf.status === 'active');
  if (active.length === 0) return null;
  return active.slice().sort((a, b) => {
    const ta = Date.parse(a.created_at) || 0;
    const tb = Date.parse(b.created_at) || 0;
    if (tb !== ta) return tb - ta;
    if (a.id === b.id) return 0;
    return a.id < b.id ? 1 : -1;
  })[0];
}

/**
 * rebuildStateProjection(root, overrides) — full rebuild of .bee/state.json's
 * D1-owned fields (phase/feature/mode/approved_gates/summary/next_action)
 * from the newest ACTIVE workflow record — but ONLY while the current
 * default record is itself IDLE (no feature, phase 'idle'/
 * 'compounding-complete'/absent). Every other field already on the file
 * (`workers`, `schema_version`, and any ad hoc field a future cell adds)
 * passes through unchanged. `overrides.cellCounts`/`overrides.lastActivity`,
 * when present, are written into `cells`/`last_activity` in this SAME write
 * REGARDLESS of the idle gate — the seam bee-state-sync's F8 full rebuild
 * uses so its counts/timestamp refresh (unrelated to D1, and needed on
 * every hook tick whether or not a feature is active) and this module's
 * D1-field rebuild land in one write, never two.
 *
 * Why the idle gate (a deliberate narrowing of the literal "newest ACTIVE
 * workflow" contract, documented honestly rather than silently): C5 (this
 * cell's own binding scope) leaves the DEFAULT (non-lane) mutation path —
 * `state set`/`gate`/`scribing-run`/`advisor-ref record` without `--lane` —
 * writing state.json directly, WITHOUT updating the matching workflow
 * record (msn-10's job). That workflow record is therefore only ever as
 * fresh as the moment `state start-feature` created it — genuinely stale
 * the instant any further default-path mutation lands. Rebuilding a LIVE
 * default record's D1 fields from ANY workflow record — its own stale one,
 * or worse, a DIFFERENT feature's (e.g. an active lane, which workflow
 * records cannot be told apart from by kind — msn-6 creates one uniformly
 * for both start paths) — would silently regress or misattribute real,
 * current default-pipeline state. That is unsafe at ANY call rate, and
 * actively dangerous at bee-state-sync's (every hook tick). The idle gate
 * makes this safe by construction: state.json is only ever ADOPTED (never
 * overwritten) from a workflow record while there is nothing live to
 * protect — bootstrapping the very first feature a fresh/idle repo starts,
 * or after `startFeature`'s own dual-write just created both the legacy
 * record and its workflow record together (see handleStateStartFeature's
 * lane-only wiring in bee.mjs, which calls rebuildLaneProjection, never
 * rebuildStateProjection, for exactly this reason on the default path).
 * Full default-record losslessness across arbitrary mutation history is
 * D8's later-stage guarantee (msn-10, or 7+10 landing atomically) — this
 * cell's own C5 comment says so — not a claim made here.
 *
 * C1 fallback is scoped to the D1-owned fields ONLY, never to `overrides`:
 * zero workflow records anywhere in the repo leaves phase/feature/mode/
 * approved_gates/summary/next_action exactly as they already are (the
 * projection layer has no authority over them yet) — but `cells`/
 * `last_activity` still get written when `overrides` supplies them, because
 * that refresh is bee-state-sync's own pre-existing job, unrelated to D1,
 * and must keep working in every repo that has not yet started a workflow-
 * creating feature (i.e. every repo before its first startFeature call since
 * msn-6 landed). With no overrides AND (zero workflow records OR a live
 * non-idle default record) this is a pure no-op read (nothing written).
 * `authoritative` always reflects whether the D1 fields specifically were
 * sourced from a workflow record on THIS call.
 */
export function rebuildStateProjection(root, overrides = {}) {
  const { workflows } = listWorkflows(root);
  const current = readState(root);
  const hasOverrides =
    Object.prototype.hasOwnProperty.call(overrides, 'cellCounts') ||
    Object.prototype.hasOwnProperty.call(overrides, 'lastActivity');
  const currentIsIdle = !current.feature && (current.phase === 'idle' || current.phase === 'compounding-complete' || !current.phase);

  const applyOverridesOnly = () => {
    if (!hasOverrides) {
      return { authoritative: false, source: null, state: current };
    }
    const next = { ...current };
    if (Object.prototype.hasOwnProperty.call(overrides, 'cellCounts')) next.cells = overrides.cellCounts;
    if (Object.prototype.hasOwnProperty.call(overrides, 'lastActivity')) next.last_activity = overrides.lastActivity;
    writeState(root, next);
    return { authoritative: false, source: null, state: next };
  };

  if (workflows.length === 0 || !currentIsIdle) {
    return applyOverridesOnly();
  }

  const active = pickNewestActiveWorkflow(workflows);
  const next = {
    ...current,
    phase: active ? active.phase : 'idle',
    feature: active ? active.feature : null,
    mode: active ? active.mode : null,
    approved_gates: active ? workflowGatesToApprovedGates(active.gates, active.plan_rev) : defaultState().approved_gates,
    summary: active ? active.summary : defaultState().summary,
    next_action: active ? active.next_action : defaultState().next_action,
  };
  if (Object.prototype.hasOwnProperty.call(overrides, 'cellCounts')) next.cells = overrides.cellCounts;
  if (Object.prototype.hasOwnProperty.call(overrides, 'lastActivity')) next.last_activity = overrides.lastActivity;
  writeState(root, next);
  return { authoritative: true, source: active ? active.id : null, state: next };
}

/**
 * rebuildLaneProjection(root, feature) — full rebuild of
 * .bee/lanes/<feature>.json from the live (non-closed) workflow record
 * naming that feature. The six baseline D1 fields (schema_version/feature/
 * mode/phase/approved_gates/summary/next_action) are fully recomputed from
 * the record every time. Everything else already on the existing lane file
 * — `created_at` (so a feature's lane identity keeps its original timestamp
 * across rebuilds; falls back to the workflow record's own created_at when
 * there is no existing file to read one from) AND any ad hoc field a lane
 * mutation verb has stamped directly onto the record (`last_scribing_run`,
 * `gate_revoked_at`, `advisor_ref` — none of these are D1/workflow-record
 * fields; workflow-store.mjs's schema does not carry them, and migrating
 * them is out of this cell's scope) — passes through UNCHANGED, same
 * discipline as rebuildStateProjection's own `workers` pass-through above.
 * A genuinely deleted lane file loses those ad hoc fields on rebuild (there
 * is no other source of truth for them yet) — an honest, documented gap,
 * not a silent one; D1's own baseline fields are never affected by it.
 *
 * No-op (authoritative: false) when zero workflow records exist anywhere,
 * OR when no LIVE (non-closed) workflow record names this feature — never
 * guesses, never deletes an existing lane file it cannot derive.
 */
export function rebuildLaneProjection(root, feature) {
  const { workflows } = listWorkflows(root);
  if (workflows.length === 0) {
    return { authoritative: false, source: null, lane: readLane(root, feature) };
  }
  const wf = workflows.find((w) => w.feature === feature && w.status !== 'closed');
  if (!wf) {
    return { authoritative: false, source: null, lane: readLane(root, feature) };
  }
  const existing = readLane(root, feature);
  const next = {
    ...existing,
    schema_version: '1.0',
    feature: wf.feature,
    mode: wf.mode,
    phase: wf.phase,
    approved_gates: workflowGatesToApprovedGates(wf.gates, wf.plan_rev),
    summary: wf.summary,
    next_action: wf.next_action,
    created_at: (existing && existing.created_at) || wf.created_at || new Date().toISOString(),
  };
  writeLane(root, next);
  return { authoritative: true, source: wf.id, lane: next };
}

/**
 * rebuildAllProjections(root) — the recovery entry point (must_have: "A
 * rebuild verb/function callable for recovery"): rebuilds state.json (from
 * the newest active workflow, only while state.json is itself idle — see
 * rebuildStateProjection's own comment for why) and EVERY active workflow's
 * lane projection ("one per active workflow", msn-7 cell contract). This is
 * what proves invariants 13/14 (deleting a projection loses nothing;
 * overview rebuilds fully from records) for whichever projections have a
 * corresponding workflow record kept in sync by every write path that
 * touches them — true for every lane in this cell, true for state.json only
 * while it is idle (its live/non-idle case is C5's residual seam, msn-10).
 * Cell-counts/last_activity are left untouched here
 * (refreshing them is bee-state-sync's job, not recovery's) — pass
 * `{cellCounts, lastActivity}` to rebuildStateProjection directly for that.
 */
export function rebuildAllProjections(root) {
  const state = rebuildStateProjection(root);
  const { workflows } = listWorkflows(root);
  const active = workflows.filter((wf) => wf.status === 'active');
  const lanes = active.map((wf) => rebuildLaneProjection(root, wf.feature));
  return { state, lanes };
}
