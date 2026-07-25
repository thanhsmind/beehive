// intent.mjs — the INTENT ANCHOR: the one durable record of what the user
// actually asked for, kept on disk so it survives every clear and every
// autocompact (feature intent-anchor, CONTEXT.md D1-D6).
//
// The defect this exists to fix, in one sentence: today the most-durable
// content in a long session is bee's own workflow scaffolding (on disk,
// re-injected in full at every compaction) and the LEAST-durable is the
// user's request (it lives only in the conversation, which is exactly what
// compaction compresses first). After ~2 compactions the scaffolding is at
// full strength and the objective is gone, so the agent optimises for
// "finish the workflow" instead of "answer the user". This module inverts
// that: the objective goes on disk, VERBATIM, is re-asserted at PreCompact,
// and is read FIRST on a compact/resume start.
//
// Three disciplines hold its shape:
//
//   * SMALL. A fixed set of fields, nothing that grows per task. A large
//     anchor is bloat, not durability — the spike measured that intent's
//     *share* of the context is the wrong axis; PRESENCE and FIDELITY are
//     what drive post-compact behaviour (.bee/spikes/intent-anchor/FINDINGS.md).
//   * VERBATIM. `request` is the user's own bytes. A paraphrase is the first
//     step of the drift the anchor exists to prevent, so nothing in this
//     pipeline ever truncates, re-wraps, or summarises it — and `advanceIntent`
//     structurally cannot touch it (D1: only `next_action` advances).
//   * FAIL-OPEN. Every reader returns null rather than throwing. A missing or
//     corrupt anchor must leave every existing surface byte-identical to what
//     it printed before this shipped (D5) — a repo that never writes one must
//     not be able to tell this feature exists.

import fs from 'node:fs';
import path from 'node:path';
import { readJson, writeJsonAtomic } from './fsutil.mjs';
import { readState } from './state.mjs';

export const INTENT_SCHEMA_VERSION = '1.0';

// The last-resort key for work that has neither an active feature nor a
// session id (D2: a direct question or a tiny fix must still be anchorable —
// those have no CONTEXT.md and no work item, so nothing holds their intent
// at all today). The CLI runs as a bare process with no session id, so this
// is the key it lands on for featureless work; the hooks, which DO receive a
// session id, still find it because lookup walks every candidate.
export const DEFAULT_INTENT_KEY = 'default';

// The two phases where no work is open. A stale `feature` string outlives
// both, which is why the PHASE — not the feature field — decides whether a
// feature is "active" (the same predicate inject.mjs uses for its own
// no-work branches).
const NO_WORK_PHASES = new Set(['idle', 'compounding-complete']);

export function intentDir(root) {
  return path.join(root, '.bee', 'intent');
}

// Keys become filenames, so they are constrained to a safe charset here
// rather than trusted. Never throws: an unusable key degrades to the default
// one instead of failing a write the user cannot recover.
export function sanitizeIntentKey(key) {
  const raw = typeof key === 'string' ? key.trim() : '';
  if (!raw) return DEFAULT_INTENT_KEY;
  const safe = raw
    .replace(/[^A-Za-z0-9._-]+/g, '-')
    .replace(/^[-.]+/, '')
    .replace(/-+$/, '')
    .slice(0, 120);
  return safe || DEFAULT_INTENT_KEY;
}

export function intentPath(root, key) {
  return path.join(intentDir(root), `${sanitizeIntentKey(key)}.json`);
}

/** The active feature slug, or null. Fail-open: any read problem => null. */
export function activeFeature(root) {
  try {
    const state = readState(root);
    if (NO_WORK_PHASES.has(state.phase)) return null;
    return typeof state.feature === 'string' && state.feature.trim() ? state.feature.trim() : null;
  } catch {
    return null;
  }
}

/**
 * Key resolution (D2), in priority order: an explicit feature, the active
 * feature, the session id, then the shared default. writeIntent lands on
 * candidates[0]; every reader walks the whole list, so an anchor written by
 * the CLI (no session id) is still found by a hook (session id present), and
 * one written under a feature is still found after that feature closes.
 */
export function intentKeyCandidates(root, { sessionId = null, feature = null, key = null } = {}) {
  if (typeof key === 'string' && key.trim()) return [sanitizeIntentKey(key)];
  const candidates = [];
  const explicit = typeof feature === 'string' && feature.trim() ? feature.trim() : null;
  const resolved = explicit || activeFeature(root);
  if (resolved) candidates.push(sanitizeIntentKey(resolved));
  if (typeof sessionId === 'string' && sessionId.trim()) candidates.push(sanitizeIntentKey(sessionId));
  candidates.push(DEFAULT_INTENT_KEY);
  return [...new Set(candidates)];
}

function normalizeList(value) {
  if (Array.isArray(value)) return value.map((v) => String(v).trim()).filter(Boolean);
  if (typeof value === 'string' && value.trim()) {
    return value
      .split(',')
      .map((v) => v.trim())
      .filter(Boolean);
  }
  return [];
}

function optionalString(value) {
  return typeof value === 'string' && value.trim() ? value.trim() : null;
}

// A record missing the one field that matters is not an anchor. Corrupt,
// half-written, or hand-mangled files read as absent (D5) rather than as a
// half-anchor that could hand a summarizer a truncated objective.
function normalizeAnchor(raw, key) {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return null;
  if (typeof raw.request !== 'string' || !raw.request.trim()) return null;
  return {
    schema_version: typeof raw.schema_version === 'string' ? raw.schema_version : INTENT_SCHEMA_VERSION,
    key: typeof raw.key === 'string' && raw.key ? raw.key : key,
    written_at: typeof raw.written_at === 'string' ? raw.written_at : null,
    // VERBATIM — never trimmed, never truncated, never re-wrapped.
    request: raw.request,
    acceptance: typeof raw.acceptance === 'string' ? raw.acceptance : '',
    next_action: optionalString(raw.next_action),
    feature: optionalString(raw.feature),
    lane: optionalString(raw.lane),
    cell: optionalString(raw.cell),
    do_not_reverse: normalizeList(raw.do_not_reverse),
    stop_conditions: normalizeList(raw.stop_conditions),
    ...(typeof raw.advanced_at === 'string' ? { advanced_at: raw.advanced_at } : {}),
  };
}

/**
 * Read the anchor at the first candidate key that holds one. Returns null
 * when nothing is stored, when the file is unreadable, and when its content
 * is not a usable anchor. NEVER throws — this is called from hooks whose
 * whole contract is fail-open.
 */
export function readIntent(root, options = {}) {
  try {
    for (const key of intentKeyCandidates(root, options)) {
      const anchor = normalizeAnchor(readJson(intentPath(root, key), null), key);
      if (anchor) return anchor;
    }
  } catch {
    // fail-open: a missing/corrupt anchor is silence, never a crash.
  }
  return null;
}

/** Which key currently HOLDS an anchor (null when none does). */
export function locateIntentKey(root, options = {}) {
  try {
    for (const key of intentKeyCandidates(root, options)) {
      if (normalizeAnchor(readJson(intentPath(root, key), null), key)) return key;
    }
  } catch {
    // fail-open
  }
  return null;
}

/**
 * Write the anchor. `request` is stored EXACTLY as given.
 *
 * Immutability (D1): request and acceptance are immutable once set. Writing a
 * DIFFERENT request/acceptance over a live anchor throws unless `force` is
 * passed — an objective is replaced deliberately (a new task), never drifted
 * into. Re-writing the same request is idempotent and always allowed, so a
 * re-run of "anchor this task" is never a failure.
 */
export function writeIntent(root, fields = {}, options = {}) {
  const { request, acceptance } = fields;
  if (typeof request !== 'string' || !request.trim()) {
    throw new Error('writeIntent: `request` is required and must be the user\'s VERBATIM words.');
  }
  if (typeof acceptance !== 'string' || !acceptance.trim()) {
    throw new Error('writeIntent: `acceptance` is required — an anchor with no "done means" cannot detect drift.');
  }
  const key = intentKeyCandidates(root, options)[0];
  const existing = normalizeAnchor(readJson(intentPath(root, key), null), key);
  if (existing && options.force !== true) {
    if (existing.request !== request) {
      throw new Error(
        `writeIntent: an anchor already exists at "${key}" with a different request — request is immutable once set (D1). ` +
          'Advance it (`bee intent advance`), clear it (`bee intent clear`), or pass --force to replace the objective deliberately.',
      );
    }
    if (existing.acceptance !== acceptance) {
      throw new Error(
        `writeIntent: an anchor already exists at "${key}" with different acceptance criteria — acceptance is immutable once set (D1). ` +
          'Clear it (`bee intent clear`) or pass --force to replace the objective deliberately.',
      );
    }
  }
  const anchor = {
    schema_version: INTENT_SCHEMA_VERSION,
    key,
    written_at: new Date().toISOString(),
    request,
    acceptance,
    next_action: optionalString(fields.next_action),
    feature: optionalString(fields.feature) || activeFeature(root),
    lane: optionalString(fields.lane),
    cell: optionalString(fields.cell),
    do_not_reverse: normalizeList(fields.do_not_reverse),
    stop_conditions: normalizeList(fields.stop_conditions),
  };
  writeJsonAtomic(intentPath(root, key), anchor);
  return anchor;
}

/**
 * A segment finishes; the through-line (request + acceptance) is untouched
 * and only `next_action` moves. This is what a clear-and-continue boundary
 * writes before the clear, so the next session resumes the SAME intent at the
 * next step instead of re-routing to a workflow. The signature makes the
 * immutability structural: there is no parameter that could carry a new
 * request or acceptance into the stored record.
 */
export function advanceIntent(root, nextAction, options = {}) {
  const key = locateIntentKey(root, options);
  if (!key) return null;
  const anchor = normalizeAnchor(readJson(intentPath(root, key), null), key);
  if (!anchor) return null;
  const advanced = {
    ...anchor,
    next_action: optionalString(nextAction),
    advanced_at: new Date().toISOString(),
  };
  writeJsonAtomic(intentPath(root, key), advanced);
  return advanced;
}

/** Remove the anchor. Returns {cleared, key} — never throws. */
export function clearIntent(root, options = {}) {
  const key = locateIntentKey(root, options) || intentKeyCandidates(root, options)[0];
  try {
    const file = intentPath(root, key);
    if (!fs.existsSync(file)) return { cleared: false, key };
    fs.rmSync(file, { force: true });
    return { cleared: true, key };
  } catch {
    return { cleared: false, key };
  }
}

// ─── renderers ─────────────────────────────────────────────────────────────
//
// Two blocks, one job each. Both take an anchor OBJECT (never a key) so they
// stay pure and testable, and both return '' for a null anchor — the silence
// D5 requires. Neither ever reflows `request`: it is emitted on its own
// line(s), byte for byte, under a label.

function contextLines(anchor) {
  const lines = [];
  if (anchor.do_not_reverse.length > 0) {
    lines.push(`DO NOT REVERSE: ${anchor.do_not_reverse.join(' | ')}`);
  }
  if (anchor.stop_conditions.length > 0) {
    lines.push(`STOP IF: ${anchor.stop_conditions.join(' | ')}`);
  }
  const where = [
    anchor.feature ? `feature=${anchor.feature}` : null,
    anchor.lane ? `lane=${anchor.lane}` : null,
    anchor.cell ? `cell=${anchor.cell}` : null,
  ].filter(Boolean);
  if (where.length > 0) lines.push(`CONTEXT: ${where.join(' ')}`);
  return lines;
}

export const INTENT_PRECOMPACT_HEADER =
  '=== BEE INTENT ANCHOR — VERBATIM · DO NOT SUMMARIZE · DO NOT PARAPHRASE ===';
export const INTENT_PRECOMPACT_FOOTER = '=== END BEE INTENT ANCHOR ===';
export const INTENT_RESUME_HEADER =
  '## INTENT ANCHOR — read this FIRST (the objective; bee workflow state follows below)';

/**
 * D3 — what PreCompact pushes into the preserved context. Labelled top and
 * bottom precisely so a summarizer cannot treat it as ordinary prose: the
 * label is the mechanism that makes the block survive the summary, and the
 * spike's honest correction was that this labelling — not the token share —
 * is what changes post-compact behaviour.
 */
export function precompactBlock(anchor) {
  if (!anchor || typeof anchor !== 'object' || typeof anchor.request !== 'string') return '';
  return [
    INTENT_PRECOMPACT_HEADER,
    'This block is the OBJECTIVE and outranks every phase/gate/workflow detail in this',
    'context. Carry it through the compaction unchanged, word for word.',
    'ORIGINAL REQUEST (verbatim):',
    anchor.request,
    `DONE MEANS: ${anchor.acceptance}`,
    ...(anchor.next_action ? [`NEXT ACTION: ${anchor.next_action}`] : []),
    ...contextLines(anchor),
    INTENT_PRECOMPACT_FOOTER,
  ].join('\n');
}

/**
 * D4 — what a compact/resume session start LEADS with. The ordering is the
 * whole fix: the objective is stated first and the phase is a detail below
 * it, because the defect being corrected is bee re-anchoring a compacted
 * session on its own bookkeeping.
 */
export function resumeBlock(anchor) {
  if (!anchor || typeof anchor !== 'object' || typeof anchor.request !== 'string') return '';
  return [
    INTENT_RESUME_HEADER,
    'ORIGINAL REQUEST (verbatim):',
    anchor.request,
    `DONE MEANS: ${anchor.acceptance}`,
    ...(anchor.next_action ? [`NEXT ACTION: ${anchor.next_action}`] : []),
    ...contextLines(anchor),
    'Everything below is workflow state — it serves the request above, it never replaces it.',
  ].join('\n');
}
