// schedule.mjs — plan-time computed schedule (D1): pure functions over a
// feature's declared cells, no disk I/O. Cells arrive as plain objects
// ({ id, deps, files, status, ... }); this module never imports cells.mjs
// (one-directional: cells.mjs -> schedule.mjs, plan import-cycle
// constraint) and never re-implements overlap — pathsOverlap from
// reservations.mjs is the ONE overlap semantics (D3), matching exactly what
// the runtime write-guard/reservation hold enforces.

import { pathsOverlap } from './reservations.mjs';

const TERMINAL_UNSATISFIABLE_STATUSES = new Set(['blocked', 'dropped']);
const SCHEDULABLE_STATUSES = new Set(['open', 'claimed']);

function idsById(cells) {
  const byId = new Map();
  for (const cell of Array.isArray(cells) ? cells : []) {
    if (cell && typeof cell.id === 'string' && cell.id) byId.set(cell.id, cell);
  }
  return byId;
}

function depsOf(cell) {
  return Array.isArray(cell && cell.deps) ? cell.deps.filter((d) => typeof d === 'string' && d) : [];
}

function filesOf(cell) {
  return Array.isArray(cell && cell.files) ? cell.files.filter((f) => typeof f === 'string' && f) : [];
}

/**
 * detectCycles(cells) — structural check over ALL cells regardless of
 * status (D2: a dependency cycle is illegal no matter whether its members
 * are open, claimed, or already capped). Returns an array of cycles, each a
 * sorted array of member cell ids. A self-dependency (`a` lists `a` in its
 * own `deps`) is its own single-member cycle. Deps pointing at an id not
 * present in `cells` are ignored here — an unknown id can never close a
 * cycle within this cell set; that case is a computeSchedule diagnostic
 * (`unsatisfiable_deps`), not a cycle.
 *
 * Implementation: Tarjan's strongly-connected-components over the
 * cell -> dep edge direction. A component with more than one member is a
 * cycle; a single-member component is a cycle only if the cell depends on
 * itself.
 */
export function detectCycles(cells) {
  const byId = idsById(cells);
  let index = 0;
  const indices = new Map();
  const lowlink = new Map();
  const onStack = new Map();
  const stack = [];
  const sccs = [];

  function strongconnect(v) {
    indices.set(v, index);
    lowlink.set(v, index);
    index += 1;
    stack.push(v);
    onStack.set(v, true);

    for (const w of depsOf(byId.get(v))) {
      if (!byId.has(w)) continue; // unknown dep target: cannot close a cycle here
      if (!indices.has(w)) {
        strongconnect(w);
        lowlink.set(v, Math.min(lowlink.get(v), lowlink.get(w)));
      } else if (onStack.get(w)) {
        lowlink.set(v, Math.min(lowlink.get(v), indices.get(w)));
      }
    }

    if (lowlink.get(v) === indices.get(v)) {
      const component = [];
      let w;
      do {
        w = stack.pop();
        onStack.set(w, false);
        component.push(w);
      } while (w !== v);
      sccs.push(component);
    }
  }

  for (const id of byId.keys()) {
    if (!indices.has(id)) strongconnect(id);
  }

  const cycles = [];
  for (const component of sccs) {
    if (component.length > 1) {
      cycles.push([...component].sort());
      continue;
    }
    const [id] = component;
    if (depsOf(byId.get(id)).includes(id)) cycles.push([id]);
  }
  cycles.sort((a, b) => (a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0));
  return cycles;
}

/** True when any file in `filesA` overlaps any file in `filesB` per pathsOverlap (D3). Empty on either side never overlaps (CONTEXT: empty files = overlaps-nothing). */
function filesOverlapAny(filesA, filesB) {
  for (const a of filesA) {
    for (const b of filesB) {
      if (pathsOverlap(a, b)) return true;
    }
  }
  return false;
}

/**
 * classifyDep — how a dependency target resolves against the node-set
 * contract (plan.md Approach): 'satisfied' (capped — the dep is done),
 * 'blocked' | 'dropped' (terminal, can never cap), 'missing' (id not in
 * this cell set at all), or 'pending' (open/claimed — a real schedule
 * edge: the dependent waits for the dep to be placed in an earlier wave).
 */
function classifyDep(byId, depId) {
  const depCell = byId.get(depId);
  if (!depCell) return 'missing';
  if (depCell.status === 'capped') return 'satisfied';
  if (TERMINAL_UNSATISFIABLE_STATUSES.has(depCell.status)) return depCell.status;
  return 'pending';
}

/**
 * computeSchedule(cells) — D1: overlap matrix + dependency graph over a
 * feature's declared cells -> numbered waves, advisory-but-default dispatch
 * order for swarming, feasibility check for validating.
 *
 * Node-set contract: waves contain only `open`/`claimed` cells. A dep on a
 * `capped` cell is satisfied (no schedule edge). A dep on an unknown id, or
 * on a `blocked`/`dropped` cell, makes the dependent cell UNSCHEDULABLE —
 * excluded from every wave and reported in `diagnostics.unsatisfiable_deps`
 * as `{ cell, dep, reason }`. Exclusion propagates: a cell that depends
 * (even transitively) on an unschedulable cell is unschedulable too, but
 * only the direct cause is reported as a diagnostic row — the transitive
 * fallout is simply absent from every wave, never a crash.
 *
 * Waves are built by Kahn topological layering over the remaining
 * schedulable node-set, then greedy overlap packing per D2/D3: within a
 * ready layer (deterministic ascending id order), a cell whose `files`
 * overlap (pathsOverlap, any pair) a cell already placed in the current
 * wave defers to the next wave instead of being refused. A dependency
 * cycle simply never resolves in Kahn's layering (in-degree never reaches
 * zero) — its members never appear in any wave, matching the same
 * never-crash contract as unsatisfiable deps; `diagnostics.cycles` names
 * them explicitly via detectCycles.
 *
 * Pure: same input array always yields identical output. No disk I/O.
 */
export function computeSchedule(cells) {
  const list = Array.isArray(cells) ? cells : [];
  const byId = idsById(list);
  const cycles = detectCycles(list);

  const emptyFiles = list
    .filter((cell) => cell && typeof cell.id === 'string')
    .filter((cell) => filesOf(cell).length === 0)
    .map((cell) => cell.id)
    .sort();

  const schedulable = list.filter((cell) => cell && SCHEDULABLE_STATUSES.has(cell.status));

  // Direct unsatisfiable deps: missing / blocked / dropped targets.
  const unsatisfiable = [];
  const excluded = new Set();
  for (const cell of schedulable) {
    for (const dep of depsOf(cell)) {
      const kind = classifyDep(byId, dep);
      if (kind === 'missing' || kind === 'blocked' || kind === 'dropped') {
        unsatisfiable.push({ cell: cell.id, dep, reason: kind });
        excluded.add(cell.id);
      }
    }
  }
  unsatisfiable.sort((a, b) => (a.cell === b.cell ? (a.dep < b.dep ? -1 : a.dep > b.dep ? 1 : 0) : a.cell < b.cell ? -1 : 1));

  // Propagate exclusion: a cell depending (directly) on an already-excluded
  // schedulable cell can never schedule either, even without its own
  // direct unsatisfiable-dep row.
  let changed = true;
  while (changed) {
    changed = false;
    for (const cell of schedulable) {
      if (excluded.has(cell.id)) continue;
      for (const dep of depsOf(cell)) {
        if (excluded.has(dep)) {
          excluded.add(cell.id);
          changed = true;
          break;
        }
      }
    }
  }

  const nodes = schedulable.filter((cell) => !excluded.has(cell.id));
  const nodeIds = new Set(nodes.map((cell) => cell.id));

  const inDegree = new Map();
  const dependents = new Map();
  for (const cell of nodes) {
    inDegree.set(cell.id, 0);
    dependents.set(cell.id, []);
  }
  for (const cell of nodes) {
    for (const dep of depsOf(cell)) {
      if (!nodeIds.has(dep)) continue; // satisfied (capped) or excluded — no schedule edge
      inDegree.set(cell.id, inDegree.get(cell.id) + 1);
      dependents.get(dep).push(cell.id);
    }
  }

  const remaining = new Map(inDegree);
  const placed = new Set();
  const waves = [];
  for (;;) {
    const ready = nodes
      .filter((cell) => !placed.has(cell.id) && remaining.get(cell.id) === 0)
      .map((cell) => cell.id)
      .sort();
    if (ready.length === 0) break;

    const wave = [];
    for (const id of ready) {
      const cell = byId.get(id);
      const overlapsPlaced = wave.some((placedId) => filesOverlapAny(filesOf(byId.get(placedId)), filesOf(cell)));
      if (!overlapsPlaced) wave.push(id);
    }
    waves.push(wave);
    for (const id of wave) placed.add(id);
    for (const id of wave) {
      for (const dependent of dependents.get(id)) {
        remaining.set(dependent, remaining.get(dependent) - 1);
      }
    }
  }

  return {
    waves,
    diagnostics: {
      cycles,
      unsatisfiable_deps: unsatisfiable,
      empty_files: emptyFiles,
    },
  };
}
