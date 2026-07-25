// compaction.mjs — the ONE module every compaction surface calls (feature
// compaction-hardening, D3: the helper floor). Hooks and `bee.mjs` verbs are
// two thin caller classes of the functions below; no behavior lives only
// inside a hook, so a runtime whose hook execution is unconfirmed can still
// reach every surface by command.
//
// Six things live here and nothing else:
//
//   appendCompactionRecord  the durable telemetry write (D4/D5)
//   readCompactionCounts    the counting rule, read-only (D5)
//   survivalWarning         the D9 advisory string (never a verdict)
//   anchorMissing           the D10 nudge predicate + the exact command
//   compactCheck            the D12/D13 integrity sweep — reports, never repairs
//   buildCompactCapsule     the D6 compact-scoped orientation body (cz-5)
//
// THE COUNTING RULE IS THE LOAD-BEARING PART (D5). Only `precompact` records
// are counted, and only a `precompact` record counts itself:
//   * on a precompact, compact_index / cell_compact_count = prior precompact
//     records (for the session / for the (session, cell) pair) PLUS ONE;
//   * on a resume, both fields carry the PLAIN prior count — the same values
//     the preceding precompact recorded, never +1 again.
// Counting a resume inclusively makes one compaction read as two and fires
// D9's advisory a full cycle early, on the first compaction's resume.
//
// TWO EVENTS, NOT ONE, because they answer different questions: a `precompact`
// with no following `resume` is a session that never came back, which is
// itself the signal a later audit wants.
//
// FAIL-OPEN, LOCALLY (D4). The append sits in its own try/catch — precedent
// hooks/bee-model-guard.mjs:106-128 and lib/dispatch-prepare.mjs:172-183 — so
// a write failure never changes any caller's return value or exit code. The
// write goes through fsutil's appendJsonl, which calls ensureDir; a bare
// fs.appendFileSync would throw ENOENT on a repo whose .bee/logs/ is absent.
// No rotation and no size cap, matching every other .bee/logs/*.jsonl.
//
// READ-ONLY MEANS READ-ONLY (D13). compactCheck reports and never mutates: it
// never repairs, never releases, never blocks. Its guarantee is proven by
// hashing the .bee/ tree across two consecutive runs, not by a stable stdout.

import fs from 'node:fs';
import path from 'node:path';

import { appendJsonl, readJsonl, readJson } from './fsutil.mjs';
import {
  BEE_VERSION,
  COMMAND_KEYS,
  readState,
  readConfig,
  readHandoff,
  readOnboarding,
  resolvePipeline,
  gateApproved,
  bypassLevel,
  controlRootFor,
} from './state.mjs';
import { readIntent } from './intent.mjs';
import { claimsDir, sessionPath, readClaim } from './claims.mjs';
import { listReservations } from './reservations.mjs';
import { listCells, readCell } from './cells.mjs';
import { onboardingLine, bypassBannerLines, handoffBlockLines, firstOpenGate } from './inject.mjs';
import { bundleMode } from './knowledge.mjs';

/** The two events, and the only two (D5). */
export const COMPACT_EVENTS = ['precompact', 'resume'];

/** D9 — a unit that has survived this many compactions gets the advisory. */
export const SURVIVAL_WARNING_THRESHOLD = 2;

/** D11 — the shouldInject/markInjected cache key for the anchor nudge. */
export const ANCHOR_NUDGE_KEY = 'anchor-missing-nudge';

/** D10 — the exact command the nudge names. */
export const ANCHOR_NUDGE_COMMAND =
  'node .bee/bin/bee.mjs intent set --request "<the user\'s VERBATIM request>" --acceptance "<what done means>"';

// The terminal phases: the two where no work is open. There is no exported
// isTerminalPhase — the set is documented at state.mjs:1655 and in AGENTS.md's
// intake-gate rule, and intent.mjs keeps its own private copy for the same
// reason. Kept local rather than imported so this module never depends on
// another module's private constant.
const TERMINAL_PHASES = new Set(['idle', 'compounding-complete']);

function norm(value) {
  return typeof value === 'string' && value.trim() ? value.trim() : null;
}

export function compactionLogPath(root) {
  return path.join(root, '.bee', 'logs', 'compaction.jsonl');
}

/** Every readable record in the log. Fail-open: an absent log is zero records. */
export function readCompactionRecords(root) {
  try {
    return readJsonl(compactionLogPath(root));
  } catch {
    return [];
  }
}

/**
 * D5 — the PLAIN prior counts: how many `precompact` records this session has
 * already written, and how many of those were for this (session, cell) pair.
 * `resume` records are never counted, at all, by anything.
 *
 * These are exactly the values a `resume` record carries; a `precompact`
 * record carries them plus one. With no cell there is no unit, so
 * cell_compact_count is 0 rather than a session-wide count wearing a per-cell
 * name — D9's advisory is about a UNIT being oversized.
 */
export function readCompactionCounts(root, { sessionId = null, cell = null } = {}) {
  const session = norm(sessionId);
  const unit = norm(cell);
  let compactIndex = 0;
  let cellCompactCount = 0;
  for (const record of readCompactionRecords(root)) {
    if (!record || typeof record !== 'object') continue;
    if (record.event !== 'precompact') continue;
    if (norm(record.session) !== session) continue;
    compactIndex += 1;
    if (unit && norm(record.cell) === unit) cellCompactCount += 1;
  }
  return { compact_index: compactIndex, cell_compact_count: cellCompactCount };
}

// ─── cell ownership ─────────────────────────────────────────────────────────

// msn-18b (PLANE RULE): claims are control-plane — resolved through
// controlRootFor so a compaction/injection read from a linked worktree sees
// the SAME claim store main sees, never a worktree-local one. This is the
// single chokepoint every caller below (claimStoreCellIds, attributedToSession)
// goes through.
function safeReadClaim(root, cellId) {
  try {
    return readClaim(controlRootFor(root), cellId);
  } catch {
    return null;
  }
}

/** Cell ids the cross-session claims store attributes to this session. */
function claimStoreCellIds(root, session) {
  if (!session) return [];
  let entries;
  try {
    entries = fs.readdirSync(claimsDir(controlRootFor(root)));
  } catch {
    return [];
  }
  const ids = [];
  for (const entry of entries) {
    if (!entry.endsWith('.json')) continue;
    const id = entry.slice(0, -'.json'.length);
    const claim = safeReadClaim(root, id);
    if (claim && norm(claim.session) === session) ids.push(id);
  }
  return ids;
}

function attributedToSession(root, cell, session) {
  const traceSession = norm(cell?.trace?.claim_session);
  if (session) {
    if (traceSession === session) return true;
    const claim = safeReadClaim(root, cell.id);
    return Boolean(claim && norm(claim.session) === session);
  }
  // A caller with no session id owns only genuinely session-less claims —
  // never another session's work by default.
  if (traceSession) return false;
  if (cell?.status !== 'claimed') return false;
  const claim = safeReadClaim(root, cell.id);
  return !claim || !norm(claim.session);
}

/**
 * Every cell this session is on record as having claimed, in ANY status —
 * a cell that has since been capped, dropped or reopened is exactly what the
 * D12 sweep exists to surface, so filtering by status here would make that
 * check structurally unable to fail. Missing cell records surface as
 * `{ id, missing: true }` rather than being dropped.
 */
function sessionOwnedCells(root, session) {
  const owned = new Map();
  let cells = [];
  try {
    cells = listCells(root, {});
  } catch {
    cells = [];
  }
  for (const cell of cells) {
    if (!cell || typeof cell.id !== 'string') continue;
    if (attributedToSession(root, cell, session)) owned.set(cell.id, cell);
  }
  for (const id of claimStoreCellIds(root, session)) {
    if (owned.has(id)) continue;
    let cell = null;
    try {
      cell = readCell(root, id);
    } catch {
      cell = null;
    }
    owned.set(id, cell || { id, status: null, missing: true });
  }
  return [...owned.values()];
}

function claimedCells(root, session) {
  return sessionOwnedCells(root, session).filter((cell) => cell && cell.status === 'claimed');
}

/**
 * The single cell a compaction record names. Deterministic when a session
 * somehow holds more than one: most recently claimed first, id as tiebreak.
 */
export function claimedCellId(root, sessionId = null) {
  const cells = claimedCells(root, norm(sessionId));
  if (cells.length === 0) return null;
  cells.sort((a, b) => {
    const left = String(b?.trace?.claimed_at ?? '');
    const right = String(a?.trace?.claimed_at ?? '');
    if (left !== right) return left.localeCompare(right);
    return String(a.id).localeCompare(String(b.id));
  });
  return cells[0].id;
}

// ─── the record (D4/D5) ─────────────────────────────────────────────────────

/**
 * Resolve lane / feature / phase for a session. WITH the session id, exactly
 * as buildSessionPreamble (inject.mjs:301-305) and buildPromptReminder
 * (:477-479) both do — a lane-bound session must read its OWN lane, never the
 * default state.json.
 *
 * On a typed refusal (LANE_INVALID / LANE_MISSING / LANE_CORRUPT) the lane
 * NAME is still recorded — it is a fact — but feature and phase are left null
 * rather than filled from the default pipeline: reporting the default
 * pipeline's phase under a lane-bound session is the precise mistake
 * resolvePipeline's typed refusal exists to prevent. compactCheck is the
 * surface that explains the refusal; the log never guesses.
 */
function pipelineFields(root, sessionId) {
  let pipeline;
  try {
    pipeline = resolvePipeline(root, { sessionId });
  } catch {
    pipeline = null;
  }
  if (!pipeline) {
    const state = readState(root);
    return { lane: null, feature: state.feature ?? null, phase: state.phase ?? null };
  }
  if (!pipeline.ok) {
    return { lane: norm(pipeline.feature), feature: null, phase: null };
  }
  const record = pipeline.record || readState(root);
  return {
    lane: pipeline.source === 'lane' ? norm(pipeline.feature) : null,
    feature: record.feature ?? null,
    phase: record.phase ?? null,
  };
}

/**
 * Append one compaction record and return it (D4/D5).
 *
 * The return value is the caller's — the hook renders the D9 warning off
 * `cell_compact_count`, the verb prints the counts — and it is IDENTICAL
 * whether the append succeeded or failed. An unknown event is an argument
 * error and throws (matching reservations.mjs's posture on bad arguments):
 * writing a record for an event that does not exist would corrupt every later
 * count, which is worse than a loud refusal at the one call site that could
 * cause it.
 */
export function appendCompactionRecord(root, { event, sessionId = null, now = Date.now() } = {}) {
  if (!COMPACT_EVENTS.includes(event)) {
    throw new Error(
      `appendCompactionRecord: event must be one of ${COMPACT_EVENTS.join(' | ')} — got ${JSON.stringify(event)}.`,
    );
  }
  const session = norm(sessionId);
  const { lane, feature, phase } = pipelineFields(root, session);
  const cell = claimedCellId(root, session);
  const prior = readCompactionCounts(root, { sessionId: session, cell });
  // Only a precompact counts itself; a resume carries the plain prior count.
  const increment = event === 'precompact' ? 1 : 0;

  const record = {
    ts: new Date(now).toISOString(),
    event,
    session,
    lane,
    feature,
    phase,
    cell,
    compact_index: prior.compact_index + increment,
    cell_compact_count: cell ? prior.cell_compact_count + increment : 0,
    anchor_present: readIntent(root, { sessionId: session }) !== null,
  };

  try {
    appendJsonl(compactionLogPath(root), record);
  } catch {
    // D4 — a log failure never changes the caller's return value or exit code.
  }
  return record;
}

// ─── the D9 advisory ────────────────────────────────────────────────────────

/**
 * D9 — the survival advisory for a unit that has now survived
 * SURVIVAL_WARNING_THRESHOLD compactions. Returns the string, or null.
 *
 * It is ADVICE and structurally cannot be anything else: PreCompact output is
 * routed through encodeAdvisory and never encodeBlock (hooks/adapter.mjs:355-372,
 * the B2/R14 contract), so a blocking design here is unimplementable rather
 * than merely undesirable. Evaluated against the record for the compaction NOW
 * beginning, which is why it first appears on a cell's SECOND compaction.
 */
export function survivalWarning(count) {
  const n = typeof count === 'number' ? count : Number.NaN;
  if (!Number.isFinite(n) || n < SURVIVAL_WARNING_THRESHOLD) return null;
  return (
    `this unit has now survived ${n} compactions — it may be oversized; ` +
    'consider capping at the next green verify and handing off'
  );
}

// ─── the D10 anchor nudge ───────────────────────────────────────────────────

/**
 * D10 — fires when work is active and no anchor exists:
 *   the resolved phase is NON-terminal (idle / compounding-complete are the
 *   terminal set, state.mjs:1655)
 *   AND (a cell is claimed by this session OR approved_gates.execution)
 *   AND readIntent returns null.
 *
 * Returns null, or an object naming the exact command plus the fields D11's
 * dedup hash is built from (`<sessionId>:<feature>:<cell>`), so the two
 * surfaces that render it (UserPromptSubmit, PreCompact) share one truth
 * instead of two copies of the predicate.
 *
 * An unresolvable lane binding returns null rather than falling back to the
 * default pipeline: the sweep is what surfaces that, and a nudge computed off
 * the wrong pipeline would be noise on top of a real fault.
 */
export function anchorMissing(root, { sessionId = null } = {}) {
  const session = norm(sessionId);
  let pipeline;
  try {
    pipeline = resolvePipeline(root, { sessionId: session });
  } catch {
    return null;
  }
  if (!pipeline || !pipeline.ok) return null;
  const record = pipeline.record || readState(root);
  const phase = record.phase ?? null;
  if (!phase || TERMINAL_PHASES.has(phase)) return null;

  const cells = claimedCells(root, session);
  const executionApproved = gateApproved(record, 'execution');
  if (cells.length === 0 && !executionApproved) return null;

  if (readIntent(root, { sessionId: session }) !== null) return null;

  const cell = cells.length > 0 ? claimedCellId(root, session) : null;
  const feature = record.feature ?? null;
  return {
    command: ANCHOR_NUDGE_COMMAND,
    key: ANCHOR_NUDGE_KEY,
    hash: `${session ?? ''}:${feature ?? ''}:${cell ?? ''}`,
    session,
    feature,
    cell,
    phase,
    message:
      'bee: work is active and NO INTENT ANCHOR is stored' +
      `${feature ? ` (feature=${feature}` : ''}${feature && cell ? ` cell=${cell}` : ''}${feature ? ')' : ''}. ` +
      'The objective currently lives only in this conversation — the least durable place in the session, and the ' +
      'first thing a compaction compresses, while every workflow detail on disk comes back at full strength. ' +
      `Write it down VERBATIM now: ${ANCHOR_NUDGE_COMMAND}`,
  };
}

// ─── the D12/D13 integrity sweep ────────────────────────────────────────────

function reservationExpired(row, nowMs) {
  const ttl = row.ttl_seconds;
  if (!Number.isFinite(ttl) || ttl <= 0) return false;
  const reservedMs = Date.parse(row.reserved_at);
  if (!Number.isFinite(reservedMs)) return false;
  return reservedMs + ttl * 1000 <= nowMs;
}

/**
 * D12 — the read-only integrity sweep. Every check it makes is a question a
 * just-compacted session cannot answer from memory, and disk state overrides
 * conversational recollection on every one of them.
 *
 * D13 — it REPORTS. It never repairs, never releases, never blocks, and it
 * writes nothing at all: run it twice and the .bee/ tree hashes identically.
 *
 * Returns { ok, session, checks: [{name, ok, detail, code?, skipped?}],
 * mismatches } where `mismatches` is simply the failed checks, in order.
 */
export function compactCheck(root, { sessionId = null, now = Date.now() } = {}) {
  const session = norm(sessionId);
  const checks = [];
  const add = (name, ok, detail, extra = {}) => {
    checks.push({ name, ok, detail, ...extra });
  };

  // 1. The session record exists and its STORED id matches.
  if (!session) {
    add('session_record', true, 'no session id supplied — session-scoped checks are skipped, not failed.', {
      skipped: true,
    });
  } else {
    let file = null;
    try {
      // msn-18b (PLANE RULE): sessions are control-plane — see
      // safeReadClaim's own comment above.
      file = sessionPath(controlRootFor(root), session);
    } catch {
      file = null;
    }
    const stored = file ? readJson(file, null) : null;
    if (!stored) {
      add('session_record', false, `no readable session record for "${session}" under .bee/sessions/.`, {
        code: 'SESSION_MISSING',
      });
    } else if (norm(stored.id) !== session) {
      add(
        'session_record',
        false,
        `.bee/sessions/${session}.json stores id "${stored.id}" — the record does not describe this session.`,
        { code: 'SESSION_ID_MISMATCH' },
      );
    } else {
      add('session_record', true, `session record present and self-consistent (${session}).`);
    }
  }

  // 2. The lane binding resolves — the typed refusal is SURFACED, never
  //    swallowed into a silent fallback to the default pipeline.
  let pipeline = null;
  try {
    pipeline = resolvePipeline(root, { sessionId: session });
  } catch (error) {
    pipeline = null;
    add('lane_binding', false, `lane resolution threw: ${error instanceof Error ? error.message : error}`, {
      code: 'LANE_UNRESOLVABLE',
    });
  }
  if (pipeline && !pipeline.ok) {
    add('lane_binding', false, pipeline.reason, { code: pipeline.code, feature: pipeline.feature ?? null });
  } else if (pipeline) {
    add(
      'lane_binding',
      true,
      pipeline.source === 'lane'
        ? `bound to lane "${pipeline.feature}" and it resolves.`
        : 'no lane binding — the default pipeline applies.',
    );
  }
  const record = pipeline && pipeline.ok && pipeline.record ? pipeline.record : readState(root);

  // 3. Every cell this session claimed is still claimed and still owned by it.
  const owned = sessionOwnedCells(root, session);
  const ownershipProblems = [];
  for (const cell of owned) {
    if (!cell || cell.missing) {
      ownershipProblems.push(`${cell?.id ?? 'unknown'}: no cell record found`);
      continue;
    }
    if (cell.status !== 'claimed') {
      ownershipProblems.push(`${cell.id}: no longer claimed (status=${cell.status})`);
      continue;
    }
    const traceSession = norm(cell.trace?.claim_session);
    if (session && traceSession && traceSession !== session) {
      ownershipProblems.push(`${cell.id}: cell trace names session "${traceSession}"`);
    }
    const claim = safeReadClaim(root, cell.id);
    const claimSession = norm(claim?.session);
    if (session && claimSession && claimSession !== session) {
      ownershipProblems.push(`${cell.id}: claim record is owned by session "${claimSession}"`);
    }
  }
  const claimed = owned.filter((cell) => cell && cell.status === 'claimed');
  if (owned.length === 0) {
    add('claimed_cells', true, 'this session holds no cell claims.', { skipped: true });
  } else if (ownershipProblems.length > 0) {
    add('claimed_cells', false, ownershipProblems.join('; '), { code: 'CLAIM_DRIFT' });
  } else {
    add('claimed_cells', true, `still claimed and still owned: ${claimed.map((cell) => cell.id).join(', ')}.`);
  }

  // 4. approved_gates.execution is still true whenever a cell is claimed.
  if (claimed.length === 0) {
    add('execution_gate', true, 'no cell claimed — the execution gate is not in question.', { skipped: true });
  } else if (gateApproved(record, 'execution')) {
    add('execution_gate', true, 'execution gate is approved.');
  } else {
    add(
      'execution_gate',
      false,
      `execution gate is NOT approved while ${claimed.map((cell) => cell.id).join(', ')} ${
        claimed.length === 1 ? 'is' : 'are'
      } claimed — the claim outlived its authorization.`,
      { code: 'GATE_REVOKED' },
    );
  }

  // 5. The claimed cell's dependencies are still capped.
  const depProblems = [];
  for (const cell of claimed) {
    for (const dep of Array.isArray(cell.deps) ? cell.deps : []) {
      let depCell = null;
      try {
        depCell = readCell(root, dep);
      } catch {
        depCell = null;
      }
      if (!depCell) depProblems.push(`${cell.id}: dep "${dep}" has no cell record`);
      else if (depCell.status !== 'capped') depProblems.push(`${cell.id}: dep "${dep}" is ${depCell.status}, not capped`);
    }
  }
  if (claimed.length === 0) {
    add('deps_capped', true, 'no cell claimed — no dependencies to re-check.', { skipped: true });
  } else if (depProblems.length > 0) {
    add('deps_capped', false, depProblems.join('; '), { code: 'DEP_UNCAPPED' });
  } else {
    add('deps_capped', true, 'every dependency of every claimed cell is still capped.');
  }

  // 6. This session's reservations are still held by it. A row with NO
  //    `session` field is a legacy / intra-swarm row (reservations.mjs:92-98):
  //    it is REPORTED as unbound and is never counted as a mismatch (D13).
  // msn-18b: no change needed here — reservations.mjs's listReservations
  // self-resolves the control root internally (see that module's header),
  // so it is already correct regardless of which root this call passes.
  let rows = [];
  try {
    rows = listReservations(root, {});
  } catch {
    rows = [];
  }
  const claimedIds = new Set(claimed.map((cell) => cell.id));
  let held = 0;
  let unbound = 0;
  const expired = [];
  for (const row of rows) {
    if (!row || typeof row !== 'object' || row.released_at != null) continue;
    const rowSession = norm(row.session);
    if (session && rowSession === session) {
      if (reservationExpired(row, now)) expired.push(row.path);
      else held += 1;
    } else if (!rowSession && claimedIds.has(row.cell)) {
      unbound += 1;
    }
  }
  const reservationDetail =
    `${held} active hold(s) still owned by this session; ` +
    `${unbound} session-less (unbound) row(s) on this session's cells — legacy/intra-swarm rows, never a mismatch (D13)` +
    (expired.length > 0 ? `; EXPIRED and no longer held: ${expired.join(', ')}` : '.');
  if (expired.length > 0) {
    add('reservations', false, reservationDetail, { code: 'HOLD_EXPIRED', held, unbound, expired });
  } else {
    add('reservations', true, reservationDetail, { held, unbound, expired });
  }

  // 7. An anchor exists.
  if (readIntent(root, { sessionId: session }) !== null) {
    add('anchor', true, 'an intent anchor is stored.');
  } else {
    add('anchor', false, `no intent anchor is stored — write it verbatim: ${ANCHOR_NUDGE_COMMAND}`, {
      code: 'ANCHOR_MISSING',
      command: ANCHOR_NUDGE_COMMAND,
    });
  }

  const mismatches = checks.filter((entry) => entry.ok !== true);
  return { ok: mismatches.length === 0, session, checks, mismatches };
}

// ─── the D6 compact capsule ─────────────────────────────────────────────────

/**
 * The sweep checks the capsule NEVER renders.
 *
 * THIS SET IS THE LOAD-BEARING PART OF THE WHOLE CAPSULE (D19). compactCheck
 * reports "an anchor exists" as check 7, so a STATE MISMATCH line rendered off
 * the raw sweep varies with anchor presence BY CONSTRUCTION — and
 * hooks/test_hook_contracts.mjs:2740-2780 compares a with-anchor render
 * against a no-anchor control render, both through the same source=compact
 * path, asserting the bodies are byte-identical. An anchor-correlated capsule
 * reds a suite this feature is forbidden to edit.
 *
 * Muting it loses nothing: the hook already prefixes the anchor when one
 * exists (bee-session-init.mjs:44,148-150), so "no anchor" is visible from the
 * capsule's ABSENCE of a lead block, and the D10 nudge — reachable by command
 * through `state compact-check` — is the surface that names the fix.
 */
export const CAPSULE_MUTED_CHECKS = new Set(['anchor']);

/**
 * The handoff outcome a NON-ADOPTING source produces, or null.
 *
 * One truth for the string the SessionStart hook builds at
 * bee-session-init.mjs:113-121: a planned-next handoff never auto-adopts on
 * `compact`/`resume`, so the capsule must be able to say WHY it is asking the
 * session to wait (D26/D27). The `state compact-capsule` verb calls this so
 * the command-line surface renders the exact bytes the hook renders — that is
 * D3's helper floor, not a convenience.
 */
export function nonAdoptingHandoffOutcome(root, { source = 'compact' } = {}) {
  let handoff = null;
  try {
    handoff = readHandoff(root);
  } catch {
    return null;
  }
  if (!handoff || handoff.kind !== 'planned-next') return null;
  return {
    ok: false,
    code: 'WRONG_SOURCE',
    reason: `a planned-next handoff never auto-adopts on source "${source || 'unknown'}" — only "clear"/"startup" qualify (D1).`,
  };
}

function safeCell(root, id) {
  try {
    return readCell(root, id);
  } catch {
    return null;
  }
}

/**
 * D6 — the compact-scoped orientation body, emitted only on SessionStart with
 * source=compact. It REPLACES the full startup preamble; it does not extend
 * it, and it is not asserted to be a byte-subset of it (D15).
 *
 * IT NEVER RENDERS THE ANCHOR (D19). The hook keeps prefixing it, exactly as
 * it does today; rendering it here too would print the objective twice and
 * neither the anchor-first assertion nor an additivity assertion would notice.
 *
 * Item order is D6's, verbatim, items 2-12:
 *   2 STATE MISMATCH (the D12 sweep, anchor check muted — see above)
 *   3 the onboarding-MISSING line
 *   4 the HANDOFF block, wait-instruction verbatim, WITH its refusal reason
 *   5 the gate-bypass banner
 *   6 phase / mode / feature / lane  ← the `- Phase:` label is verbatim
 *   7 the claimed cell, its verify command, its dependency status
 *   8 the first open gate
 *   9 next_action
 *  10 the recorded standard commands
 *  11 the compaction survival count + the D9 advisory
 *  12 a POINTER to the critical patterns, never the digest (D7)
 *
 * handoffOutcome is MANDATORY at the call site (D27), not merely accepted:
 * every existing row exercising compact + a planned-next handoff regexes only
 * the WAIT heading, so a caller that drops it passes every suite while a
 * compacted session silently loses the explanation of why adoption was
 * refused.
 *
 * Every section is composed as its own line array and the non-empty ones are
 * joined by a blank line. That is what makes the blank lines deterministic —
 * and, with the anchor check muted, makes every byte of the output independent
 * of whether `.bee/intent/` holds anything.
 */
export function buildCompactCapsule(root, { sessionId = null, handoffOutcome = null } = {}) {
  const session = norm(sessionId);
  const sections = [];

  sections.push([`## bee v${BEE_VERSION} — compact capsule (source=compact)`]);

  // ── item 2: the integrity sweep, reported and never repaired (D12/D13).
  let sweep = null;
  try {
    sweep = compactCheck(root, { sessionId: session });
  } catch {
    sweep = null;
  }
  const mismatches = (sweep?.mismatches ?? []).filter((entry) => !CAPSULE_MUTED_CHECKS.has(entry.name));
  if (mismatches.length > 0) {
    const block = ['⚠ STATE MISMATCH — disk state overrides conversational recollection:'];
    for (const entry of mismatches) {
      block.push(`- ${entry.name}${entry.code ? ` (${entry.code})` : ''}: ${entry.detail}`);
    }
    sections.push(block);
  }

  // ── item 3: onboarding, but only the MISSING arm (D6 item 3 cites
  // inject.mjs:319 specifically). A version drift is startup housekeeping; a
  // MISSING onboarding means the vendored helpers this capsule was rendered by
  // may not exist at all, which a compacted session must know immediately.
  let onboarding = null;
  try {
    onboarding = readOnboarding(root);
  } catch {
    onboarding = null;
  }
  if (!onboarding) sections.push([onboardingLine(onboarding)]);

  // ── item 4: the HANDOFF block, wait instruction verbatim, refusal reason
  // included (D26/D27). The blank separating it is this caller's, per the
  // blank-line ownership note in inject.mjs.
  let handoff = null;
  try {
    handoff = readHandoff(root);
  } catch {
    handoff = null;
  }
  const handoffLines = handoffBlockLines(handoff, handoffOutcome);
  if (handoffLines.length > 0) sections.push(handoffLines);

  // ── item 5: the bypass banner — mandatory wherever orientation renders.
  const banner = bypassBannerLines(bypassLevel(root));
  if (banner.length > 0) sections.push(banner);

  // ── items 6-9: which pipeline, which cell, which gate, what next.
  let pipeline = null;
  try {
    pipeline = resolvePipeline(root, { sessionId: session });
  } catch {
    pipeline = null;
  }
  const record = pipeline && pipeline.ok && pipeline.record ? pipeline.record : readState(root);
  const lane = pipeline && pipeline.ok && pipeline.source === 'lane' ? norm(pipeline.feature) : null;

  const status = [
    `- Phase: ${record.phase ?? 'unknown'} | Mode: ${record.mode ?? 'none'} | Feature: ${record.feature ?? 'none'} | Lane: ${lane ?? 'none'}`,
  ];
  const cellId = claimedCellId(root, session);
  if (cellId) {
    const cell = safeCell(root, cellId);
    status.push(`- Cell: ${cellId}${cell?.title ? ` — ${cell.title}` : ''} (${cell?.status ?? 'unknown'}${cell?.lane ? `, ${cell.lane} lane` : ''})`);
    status.push(`- Verify: ${cell?.verify ? `\`${cell.verify}\`` : 'NONE RECORDED — a cell with no runnable verify cannot be capped'}`);
    const deps = Array.isArray(cell?.deps) ? cell.deps : [];
    status.push(
      deps.length === 0
        ? '- Deps: none'
        : `- Deps: ${deps.map((dep) => `${dep} (${safeCell(root, dep)?.status ?? 'no cell record'})`).join(', ')}`,
    );
  } else {
    status.push('- Cell: none claimed by this session');
  }
  const gate = firstOpenGate(record);
  if (gate) status.push(`- Gate pending: ${gate}`);
  const nextAction = norm(record.next_action);
  if (nextAction) status.push(`- Next action: ${nextAction}`);
  sections.push(status);

  // ── item 10: the recorded standard commands.
  let commands = {};
  try {
    commands = readConfig(root).commands || {};
  } catch {
    commands = {};
  }
  const recordedKeys = COMMAND_KEYS.filter((key) => commands[key]);
  if (recordedKeys.length > 0) {
    sections.push([
      '### Standard commands (host project)',
      ...recordedKeys.map((key) => `- ${key}: \`${commands[key]}\``),
    ]);
  }

  // ── item 11: the survival count and, when it applies, the D9 advisory.
  // Silent on a repo with no records at all (D15) — a first compaction has
  // nothing to report and must not manufacture a line saying so.
  const counts = readCompactionCounts(root, { sessionId: session, cell: cellId });
  if (counts.compact_index > 0 || counts.cell_compact_count > 0) {
    const block = [
      `- Compactions survived: ${counts.compact_index} in this session${cellId ? ` | ${counts.cell_compact_count} on ${cellId}` : ''}`,
    ];
    const warning = survivalWarning(counts.cell_compact_count);
    if (warning) block.push(`- ⚠ ${warning}.`);
    sections.push(block);
  }

  // ── item 12: the POINTER (D7). Never dropped, never expanded to the digest.
  let bundle = false;
  try {
    bundle = bundleMode(root) === true;
  } catch {
    bundle = false;
  }
  sections.push([
    `- Critical patterns: ${
      bundle ? 'docs/knowledge/index.md ("## Critical patterns")' : 'docs/history/learnings/critical-patterns.md'
    } — re-read the doctrine after a compaction (AGENTS.md startup step 1); this capsule carries the pointer, never the digest (D7).`,
  ]);

  return sections
    .filter((block) => block.length > 0)
    .map((block) => block.join('\n'))
    .join('\n\n');
}
