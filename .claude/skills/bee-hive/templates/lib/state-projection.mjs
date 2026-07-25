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
// C5 residual seam — CLOSED by multisession-native-10 (documented here for
// history; the seam itself no longer exists). Through multisession-native-9,
// the DEFAULT (non-lane) state.json mutation path — bee.mjs's
// resolveMutationTarget default branch, and every verb built on it (state
// set/gate/scribing-run/advisor-ref record without --lane) — called
// writeState(root, record) directly, never touching the matching workflow
// record. multisession-native-10 rewires that default branch through
// writeStateRecordThroughProjection (bee.mjs, mirroring writeLaneRecordThrough
// Projection) whenever a LIVE workflow already names the default record's
// current feature (the narrow exception: a `--feature` SWAP that changes
// identity mid-call keeps writing state.json directly, since a workflow
// record's `feature` is immutable identity — see writeStateRecordThrough
// Projection's own comment in bee.mjs).
//
// Consequence for rebuildStateProjection specifically (see its own doc
// comment below for the full reasoning): now that the default path keeps its
// live workflow record in sync at every mutation (same discipline lanes have
// had since msn-7), this module ADOPTS a workflow record's fields into
// .bee/state.json whenever a LIVE workflow names the CURRENT default
// feature — idle or not. The idle gate survives ONLY for the different,
// still-real case it was never about protecting against C5 in the first
// place for: bootstrapping the very first feature a fresh/idle repo starts
// (state.json knows no feature yet, so there is nothing to match by name —
// "newest active workflow" is a deliberate heuristic for that one moment,
// see rebuildStateProjection's own comment).

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
 * D1-owned fields (phase/feature/mode/approved_gates/summary/next_action).
 * Two sources, tried in order:
 *
 * (1) FEATURE-MATCHED (multisession-native-10): when the current default
 * record already names a feature (`current.feature`) AND a LIVE (non-closed)
 * workflow record names that SAME feature, that record is authoritative —
 * REGARDLESS of phase, idle or not. This is safe now (it was not, before
 * msn-10): bee.mjs's writeStateRecordThroughProjection keeps this exact
 * workflow record in sync at every default-path mutation (state set/gate/
 * scribing-run/advisor-ref record), the same discipline rebuildLaneProjection
 * has relied on for lanes since msn-7. Matched strictly BY FEATURE NAME,
 * mirroring rebuildLaneProjection — never "the newest active workflow"
 * once we know which feature state.json is actually tracking; guessing at
 * a DIFFERENT feature's (or an active lane's) workflow record here would
 * silently misattribute real, current default-pipeline state, which is
 * exactly the danger the ORIGINAL idle-only gate (below) existed to prevent
 * back when this branch did not exist.
 *
 * (2) IDLE BOOTSTRAP (unchanged since msn-7): when the default record names
 * NO feature at all (idle, or the terminal alias compounding-complete) and
 * at least one workflow record exists, adopt the newest ACTIVE workflow —
 * this is the ONE moment "which workflow" cannot be decided by feature-name
 * match (there is no name yet), so a heuristic is unavoidable: bootstrapping
 * the very first feature a fresh/idle repo starts, or right after
 * `startFeature`'s own dual-write just created both the legacy record and
 * its workflow record together.
 *
 * Neither source fires (pure no-op on the D1 fields) when: zero workflow
 * records exist anywhere (C1); OR a feature is set but NO live workflow
 * names it — a pre-msn-6 legacy record, or the narrow `--feature` SWAP
 * carve-out (writeStateRecordThroughProjection's own comment: a workflow
 * record's `feature` is immutable identity, so a swap that changes
 * state.json's feature mid-call is deliberately left writing state.json
 * directly, never routed through this rebuild).
 *
 * Every other field already on the file (`workers`, `schema_version`, and
 * any ad hoc field a future cell adds) passes through unchanged in every
 * branch. `overrides.cellCounts`/`overrides.lastActivity`, when present, are
 * written into `cells`/`last_activity` in this SAME write REGARDLESS of
 * which branch fired (or none) — the seam bee-state-sync's F8 full rebuild
 * uses so its counts/timestamp refresh (unrelated to D1, needed on every
 * hook tick whether or not a feature is active) and this module's D1-field
 * rebuild land in one write, never two.
 *
 * `authoritative` always reflects whether the D1 fields specifically were
 * sourced from a workflow record on THIS call.
 */
export function rebuildStateProjection(root, overrides = {}) {
  const { workflows } = listWorkflows(root);
  const current = readState(root);
  const hasOverrides =
    Object.prototype.hasOwnProperty.call(overrides, 'cellCounts') ||
    Object.prototype.hasOwnProperty.call(overrides, 'lastActivity');

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

  if (workflows.length === 0) {
    return applyOverridesOnly();
  }

  const applyOverrides = (next) => {
    if (Object.prototype.hasOwnProperty.call(overrides, 'cellCounts')) next.cells = overrides.cellCounts;
    if (Object.prototype.hasOwnProperty.call(overrides, 'lastActivity')) next.last_activity = overrides.lastActivity;
    return next;
  };

  // Branch (1) — feature-matched, msn-10.
  if (current.feature) {
    const wf = workflows.find((w) => w.feature === current.feature && w.status !== 'closed');
    if (wf) {
      const next = applyOverrides({
        ...current,
        phase: wf.phase,
        feature: wf.feature,
        mode: wf.mode,
        approved_gates: workflowGatesToApprovedGates(wf.gates, wf.plan_rev),
        summary: wf.summary,
        next_action: wf.next_action,
      });
      writeState(root, next);
      return { authoritative: true, source: wf.id, state: next };
    }
    // A feature is set but no LIVE workflow names it — falls through to the
    // idle-bootstrap branch below, which requires `!current.feature` and so
    // is always skipped here too; the net effect is applyOverridesOnly(),
    // unchanged from before this cell for this specific case.
  }

  // Branch (2) — idle bootstrap, unchanged since msn-7.
  const currentIsIdle = !current.feature && (current.phase === 'idle' || current.phase === 'compounding-complete' || !current.phase);
  if (!currentIsIdle) {
    return applyOverridesOnly();
  }

  const active = pickNewestActiveWorkflow(workflows);
  const next = applyOverrides({
    ...current,
    phase: active ? active.phase : 'idle',
    feature: active ? active.feature : null,
    mode: active ? active.mode : null,
    approved_gates: active ? workflowGatesToApprovedGates(active.gates, active.plan_rev) : defaultState().approved_gates,
    summary: active ? active.summary : defaultState().summary,
    next_action: active ? active.next_action : defaultState().next_action,
  });
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
 * the current default feature's own live workflow record when one names it,
 * else the newest active workflow while state.json is idle — see
 * rebuildStateProjection's own comment for the full two-branch contract,
 * multisession-native-10) and EVERY active workflow's lane projection ("one
 * per active workflow", msn-7 cell contract). This is what proves invariants
 * 13/14 (deleting a projection loses nothing; overview rebuilds fully from
 * records) for whichever projections have a corresponding LIVE workflow
 * record kept in sync by every write path that touches them — true for
 * every lane since msn-7, and true for state.json's live default feature
 * too since msn-10 closed the former C5 residual seam. Cell-counts/
 * last_activity are left untouched here (refreshing them is bee-state-sync's
 * job, not recovery's) — pass `{cellCounts, lastActivity}` to
 * rebuildStateProjection directly for that.
 */
export function rebuildAllProjections(root) {
  const state = rebuildStateProjection(root);
  const { workflows } = listWorkflows(root);
  const active = workflows.filter((wf) => wf.status === 'active');
  const lanes = active.map((wf) => rebuildLaneProjection(root, wf.feature));
  return { state, lanes };
}
