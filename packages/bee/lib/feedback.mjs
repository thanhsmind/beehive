// feedback.mjs — the dogfood feedback collector (P18, evolving loop, slice A).
//
// Locked design: decision 8cd4c84e (D2, allowlist — supersedes 20784de8). The
// digest is an ALLOWLIST of structured fields; there is NO free-text field.
// Measurement (reports/validation-slice-a.md) showed friction prose routinely
// names functions, files, and config keys (readBacklogCounts, COMMAND_KEYS,
// approved_gates.shape, internal call graphs) that no code-block strip and no
// secret regex removes — so the free-text surface was REMOVED, not filtered.
// Never collect a detail / text / outcome / deviations prose field.
//
// READ-SCOPE INVARIANT (D2): every filesystem access routes through the single
// exported resolveInScope(root, relPath). It path.resolve()s the target,
// realpath()s the target AND the repo root, and throws unless the real target
// sits under realpath(root)/.bee/ or realpath(root)/docs/history/. path.resolve
// normalizes ".." but does NOT resolve symlinks — a symlinked .bee/cells/evil.json
// pointing outside the repo is rejected by realpath containment.
//
// resolveInScope VALIDATES AND RETURNS AN ABSOLUTE PATH — never bytes. Every
// content read goes through an fsutil wrapper (readJson / readJsonl / readText)
// fed a resolved path; those wrappers hold the only content-read primitive and
// live in fsutil.mjs, not here. Directory enumeration goes through the sibling
// guard listInScope. A source-level test greps this file and asserts it contains
// no bare filesystem-read call — like the COMMAND_KEYS cross-file guard, this is
// a no-accidental-drift check, NOT a sandbox: a determined worker can defeat it.
// It exists to catch accident, not malice.

import fs from 'node:fs';
import path from 'node:path';
import { readJson, readText } from './fsutil.mjs';
import { SECRET_CONTENT_PATTERNS, INJECTION_PATTERNS, datamark } from './decisions.mjs';
import { readConfig } from './state.mjs';

/** Digest schema version. Pinned by a drift test, like BACKLOG_STATUSES. */
export const SCHEMA_VERSION = '1.0';

/**
 * The allowlist (decision 8cd4c84e). Each digest entry is EXACTLY these fields
 * and nothing more — there is no detail/text/outcome/deviations field. Exported
 * so the consumer (evolving-3) imports this one constant rather than hardcoding a
 * second copy that would silently drift.
 *
 * ROUND 3 (P18, D2/D2b). The list is NO LONGER a bare literal. Three rounds of
 * the same class of defect (title-only, then title+layer+source, then a
 * Date.parse-lenient first_seen) all shared one root cause: ENTRY_FIELDS was a
 * list of NAMES and nothing forced a name to own a validator, so forgetting a
 * field was natural, silent, and untested. ENTRY_FIELDS is now DERIVED from
 * `Object.keys(ENTRY_FIELD_SPEC)` (declared below, once the field validators
 * exist), so a field with no spec cannot be emitted and a new field cannot be
 * added without declaring how it is validated. See ENTRY_FIELD_SPEC and the
 * `export const ENTRY_FIELDS = Object.keys(ENTRY_FIELD_SPEC)` derivation below.
 */

/**
 * Drop reasons. Category only — a dropped record NEVER carries the matched text
 * (a bare integer cannot distinguish a careless worker from a repo probing bee
 * every close). Pinned by a drift test.
 */
export const DROP_REASONS = ['secret', 'injection', 'oversize', 'unknown_type'];

/**
 * Raw entry.type is unconstrained by contract and repos have already diverged
 * (anphabe-gogl carries 11 types and does not use 'finding' at all — the same
 * concept is spelled 'review-finding', its largest single class). This map
 * normalizes a raw type into the closed kind enum. A type absent from this map
 * is NOT silently dropped — it goes to dropped[] with reason 'unknown_type'.
 */
export const KIND_ALIASES = {
  friction: 'friction',
  finding: 'finding',
  'review-finding': 'finding',
  proposal: 'proposal',
  'kill-proposal': 'proposal',
  outcome: 'outcome',
  'kill-outcome': 'outcome',
  'kill-approval': 'approval',
  'backlog-closed': 'closed',
  'entropy-audit': 'audit',
  'harness-issue': 'harness-issue',
  debt: 'debt',
  'migrate-on-touch': 'debt',
  'scope-correction': 'correction',
  // derived kinds (built directly from cells / learnings) normalize to themselves
  blocked: 'blocked',
  deviation: 'deviation',
  learning: 'learning',
};

/**
 * The closed set of NORMALIZED kinds — the alias VALUES, e.g. 'finding',
 * 'approval', 'closed', 'audit', 'correction' — derived from KIND_ALIASES so
 * the two can never drift into two separate literals. This is what makes
 * normalizeKind idempotent: a digest bee already wrote carries values from
 * this set (never alias KEYS like 'review-finding'), and the consumer's
 * re-normalization (a D2b security control — see mergeDigests) must accept
 * its own producer's vocabulary unchanged while still rejecting anything
 * genuinely unrecognized (a foreign `kind: {}` or `kind: "<script>"`).
 */
export const NORMALIZED_KINDS = new Set(Object.values(KIND_ALIASES));

const MAX_TITLE = 200;
const PAIN_SEVERITY = { P1: 3, P2: 2, P3: 1 };
const PAIN_LMH = { low: 1, medium: 2, high: 3 };

// Source labels — bee-owned meta, stable across runs, used for `source` and for
// the scanned/absent tally. Never a repo's own free-text `source` field.
const SRC_BACKLOG = '.bee/backlog.jsonl';
const SRC_DECISIONS = '.bee/decisions.jsonl';
const SRC_CELLS = '.bee/cells';
const SRC_LEARNINGS = 'docs/history/learnings';

/**
 * Validate a repo-relative path and return its real absolute location, or null
 * if the path does not exist (an ABSENT source is skipped and counted, never a
 * scope violation and never a throw). Throws for any other realpath error and
 * for any target whose real location escapes .bee/ or docs/history/.
 *
 * Only realpath / lstat are used here — never a content-read primitive — so the
 * source-level drift guard stays at zero matches with no per-function exclusion.
 *
 * @param {string} root - repo root
 * @param {string} relPath - path relative to the repo root
 * @returns {string|null} absolute real path inside scope, or null if absent
 */
export function resolveInScope(root, relPath) {
  let realRoot;
  try {
    realRoot = fs.realpathSync(root);
  } catch (err) {
    throw new Error(`resolveInScope: cannot resolve repo root "${root}": ${err && err.code ? err.code : err}`);
  }
  const target = path.resolve(realRoot, relPath);
  let realTarget;
  try {
    realTarget = fs.realpathSync(target);
  } catch (err) {
    if (err && err.code === 'ENOENT') return null; // absent — the caller counts it, never throws
    throw new Error(`resolveInScope: cannot resolve "${relPath}": ${err && err.code ? err.code : err}`);
  }
  const beeRoot = path.join(realRoot, '.bee');
  const historyRoot = path.join(realRoot, 'docs', 'history');
  const contained =
    realTarget === beeRoot ||
    realTarget === historyRoot ||
    realTarget.startsWith(beeRoot + path.sep) ||
    realTarget.startsWith(historyRoot + path.sep);
  if (!contained) {
    throw new Error(
      `resolveInScope: "${relPath}" resolves to "${realTarget}", outside .bee/ and docs/history/ — rejected by realpath containment`,
    );
  }
  return realTarget;
}

/**
 * Resolve a directory in scope and return its sorted entry names, [] if it
 * exists but is not a directory, or null if it is absent. Enumeration is gated
 * behind resolveInScope's realpath containment and uses opendir (not a
 * directory-listing content read) so the source-level drift guard stays clean.
 *
 * @param {string} root - repo root
 * @param {string} relDir - directory path relative to the repo root
 * @returns {string[]|null} sorted entry names, or null if absent
 */
export function listInScope(root, relDir) {
  const dir = resolveInScope(root, relDir);
  if (dir === null) return null;
  let stat;
  try {
    stat = fs.lstatSync(dir);
  } catch {
    return null;
  }
  if (!stat.isDirectory()) return [];
  const names = [];
  const handle = fs.opendirSync(dir);
  try {
    let entry = handle.readSync();
    while (entry !== null) {
      names.push(entry.name);
      entry = handle.readSync();
    }
  } finally {
    handle.closeSync();
  }
  names.sort();
  return names;
}

/**
 * Map a raw backlog `type` (an alias KEY, e.g. 'review-finding') to its
 * normalized kind (the VALUE, e.g. 'finding'). IDEMPOTENT: a value already in
 * NORMALIZED_KINDS (i.e. already-normalized — exactly what a digest bee
 * already wrote carries) is returned unchanged, so re-running this on the
 * consumer path (D2b) never rejects the producer's own vocabulary. A
 * genuinely unrecognized token — including a non-string like `{}` or `null`,
 * or an unrecognized string like '<script>' — still becomes null
 * (unknown_type). Exported so its idempotence is directly testable.
 */
export function normalizeKind(rawType) {
  if (typeof rawType !== 'string') return null;
  if (Object.prototype.hasOwnProperty.call(KIND_ALIASES, rawType)) return KIND_ALIASES[rawType];
  if (NORMALIZED_KINDS.has(rawType)) return rawType;
  return null;
}

function scanTitle(value) {
  // Runs BEFORE any transformation, so a match is counted as a security event
  // rather than silently rewritten. Secret takes precedence over injection.
  const text = typeof value === 'string' ? value : '';
  for (const pattern of SECRET_CONTENT_PATTERNS) {
    if (pattern.test(text)) return 'secret';
  }
  for (const pattern of INJECTION_PATTERNS) {
    if (pattern.test(text)) return 'injection';
  }
  return null;
}

function capTitle(value) {
  const text = typeof value === 'string' ? value : '';
  if (text.length <= MAX_TITLE) return text;
  return `${text.slice(0, MAX_TITLE - 1)}…`; // trailing ellipsis marks truncation
}

// Sanitize a field for a dropped[] record. A dropped record records the reason
// CATEGORY only and must NEVER carry the matched secret/injection text (datamark
// neutralizes role tags but does not remove an API key or an "ignore previous
// instructions" phrase). So a field that independently matches a pattern is
// nulled outright; a clean field is datamarked so even benign meta cannot act as
// instructions once it reaches a prompt.
function sanitizeDropField(value) {
  if (typeof value !== 'string' || !value) return null;
  if (scanTitle(value)) return null; // never record matched secret/injection text
  return datamark(value);
}

// first_seen must stay SORTABLE, so it is NEVER datamarked — a wrapped value
// cannot be compared. Round 3 (P18) makes it UNFORGEABLE BY FORMAT instead:
// accept only an anchored strict ISO-ish date literal. V8's legacy Date.parse
// treats parenthesised text as a discardable COMMENT, so
// `Date.parse('Jan 1 2020 (</system> … AKIAIOSFODNN7EXAMPLE)')` returns a valid
// timestamp and the raw string — a role tag and an AWS key — would ride through
// un-scanned and un-datamarked. Never trust Date.parse's leniency: match the
// literal instead.
const STRICT_ISO_DATE = /^\d{4}-\d{2}-\d{2}([T ]\d{2}:\d{2}(:\d{2}(\.\d+)?)?(Z|[+-]\d{2}:?\d{2})?)?$/;
function validFirstSeen(value) {
  return typeof value === 'string' && STRICT_ISO_DATE.test(value) ? value : null;
}

// A positive integer, else null (local callers substitute the default 1).
function validPain(value) {
  return Number.isInteger(value) && value > 0 ? value : null;
}

// Extract the RAW (pre-transform) string a free-string field scans and stores.
// layer coerces empty → null; source keeps '' (a bee-owned meta value); title
// coerces a non-string to '' so it is always a string before capping.
const rawLayerStr = (raw) => (typeof raw.layer === 'string' && raw.layer ? raw.layer : null);
const rawSourceStr = (raw) => (typeof raw.source === 'string' ? raw.source : null);
const rawTitleStr = (raw) => (typeof raw.title === 'string' ? raw.title : '');

/**
 * A free-string field spec. On the LOCAL (trusted) path the value is validated
 * but left bare; on the NEUTRALIZE (foreign, untrusted) path it is scanned (a hit
 * drops the whole entry, attributed) and then datamark()ed — never surfaced raw.
 * `scanWhen: 'always'` (title) scans on both paths; `'neutralize'` (layer,
 * source) widens the scan only for foreign data. `scanPriority` fixes the
 * cross-field precedence (title > source > layer) independent of iteration order.
 * `validator` is a function so the structural guard can assert every field owns
 * one — a field without a validator cannot exist.
 */
function freeStringSpec({ rawOf, scanWhen, scanPriority, cap = false, inDropped }) {
  return {
    inDropped,
    scanPriority,
    validator: rawOf,
    keep(raw, neutralize) {
      const rawVal = rawOf(raw);
      if (cap) return neutralize ? datamark(capTitle(rawVal)) : capTitle(rawVal);
      if (neutralize) return typeof rawVal === 'string' && rawVal ? datamark(rawVal) : null;
      return rawVal;
    },
    dropped(raw, neutralize) {
      const rawVal = rawOf(raw);
      return neutralize ? sanitizeDropField(rawVal) : rawVal;
    },
    scan(raw, neutralize) {
      if (scanWhen === 'always' || (scanWhen === 'neutralize' && neutralize)) return scanTitle(rawOf(raw));
      return null;
    },
  };
}

/**
 * ENTRY_FIELD_SPEC — the SINGLE source of truth (P18 round 3, D2/D2b). Maps every
 * allowlist field to its validator/neutralizer. buildEntry (both trust levels)
 * and the dropped[] record builder iterate THIS map, never a raw name list, so a
 * field with no spec cannot be emitted and a new field cannot be added without a
 * validator. ENTRY_FIELDS is derived from Object.keys below, so the two can never
 * drift into separate literals. `inDropped` marks the fields a dropped[] record
 * carries ({kind, layer, source, first_seen} + reason — never title/pain).
 * Insertion order fixes ENTRY_FIELDS at kind,layer,source,title,first_seen,pain.
 */
export const ENTRY_FIELD_SPEC = {
  // `kind` after normalizeKind is a closed-enum literal, already safe and bare.
  // resolveKind also supplies the dropped-record value: the sanitised raw type on
  // an unknown_type drop, the normalized kind on a scan-hit drop.
  kind: {
    inDropped: true,
    validator: normalizeKind,
    resolveKind(raw, neutralize) {
      const value = normalizeKind(raw.type);
      if (value === null) {
        const rawKind = typeof raw.type === 'string' ? raw.type : null;
        return { value: null, isUnknown: true, dropped: neutralize ? sanitizeDropField(rawKind) : rawKind };
      }
      return { value, isUnknown: false, dropped: value };
    },
  },
  layer: freeStringSpec({ rawOf: rawLayerStr, scanWhen: 'neutralize', scanPriority: 2, inDropped: true }),
  source: freeStringSpec({ rawOf: rawSourceStr, scanWhen: 'neutralize', scanPriority: 1, inDropped: true }),
  title: freeStringSpec({ rawOf: rawTitleStr, scanWhen: 'always', scanPriority: 0, cap: true, inDropped: false }),
  first_seen: {
    inDropped: true,
    validator: validFirstSeen,
    keep(raw) {
      return validFirstSeen(raw.first_seen);
    },
    dropped(raw) {
      return validFirstSeen(raw.first_seen);
    },
    scan() {
      return null;
    },
  },
  pain: {
    inDropped: false,
    validator: validPain,
    keep(raw, neutralize) {
      const p = validPain(raw.pain);
      return p === null ? (neutralize ? null : 1) : p;
    },
    scan() {
      return null;
    },
  },
};

/**
 * The allowlist field names, DERIVED from the spec so the two can never drift.
 * Exported for the consumer (evolving-3). A source-level structural test asserts
 * this is exactly Object.keys(ENTRY_FIELD_SPEC) and that every field owns a
 * validator — so a future field added without a spec turns the suite red instead
 * of opening a hole.
 */
export const ENTRY_FIELDS = Object.keys(ENTRY_FIELD_SPEC);

/**
 * A raw candidate is { type, title, layer, first_seen, pain, source }. Turn it
 * into an allowlist entry, or push a { kind, layer, source, first_seen, reason }
 * record onto dropped[]. Order is fixed: unknown-type check, then the scan
 * (before any transformation), then the entry. Returns the entry or null.
 *
 * Both the entry AND the dropped record are built by ITERATING ENTRY_FIELD_SPEC,
 * never a hardcoded field list — the round-3 fix. This is the SINGLE construction
 * path for BOTH producers: buildDigest (local, trusted) and mergeDigests
 * (foreign, untrusted). With `neutralize: true` (D2b) the consumer path is
 * strictly stronger than the producer it backstops: the scan widens beyond title
 * to source and layer; every surviving string field is datamark()ed, not title
 * alone; first_seen must be a strict date literal; a non-integer pain becomes
 * null; and a dropped[] record's own source/layer/kind are sanitised so the
 * attacker's raw text never lands even there.
 */
function buildEntry(raw, dropped, { neutralize = false } = {}) {
  const kindResult = ENTRY_FIELD_SPEC.kind.resolveKind(raw, neutralize);

  // Build a dropped[] record from the SAME spec loop, so it too can never carry
  // an unspecced field. kind's dropped value comes from resolveKind (unknown_type
  // → sanitised raw type; scan hit → the normalized kind).
  const makeDropped = (reason) => {
    const record = {};
    for (const field of ENTRY_FIELDS) {
      const spec = ENTRY_FIELD_SPEC[field];
      if (!spec.inDropped) continue;
      record[field] = field === 'kind' ? kindResult.dropped : spec.dropped(raw, neutralize);
    }
    record.reason = reason;
    return record;
  };

  if (kindResult.isUnknown) {
    dropped.push(makeDropped('unknown_type'));
    return null;
  }

  // Scan the RAW values BEFORE any transformation. title scans on both paths;
  // source/layer widen the scan on the foreign path so an attacker who moves the
  // payload out of title cannot walk through clean. Lowest scanPriority wins
  // (title > source > layer); scanTitle keeps secret > injection within a field.
  let hit = null;
  let hitPriority = Infinity;
  for (const field of ENTRY_FIELDS) {
    const spec = ENTRY_FIELD_SPEC[field];
    if (typeof spec.scan !== 'function') continue;
    const h = spec.scan(raw, neutralize);
    if (h && spec.scanPriority < hitPriority) {
      hit = h;
      hitPriority = spec.scanPriority;
    }
  }
  if (hit) {
    dropped.push(makeDropped(hit));
    return null;
  }

  const entry = {};
  for (const field of ENTRY_FIELDS) {
    const spec = ENTRY_FIELD_SPEC[field];
    entry[field] = field === 'kind' ? kindResult.value : spec.keep(raw, neutralize);
  }
  return entry;
}

/**
 * Parse a learnings *.md frontmatter block into { date, severity, title }, or
 * null when there is no leading `---` frontmatter block. Text-only, no code,
 * no body prose — just the three allowlist-relevant frontmatter fields plus the
 * first H1 as the title.
 */
function parseLearningFrontmatter(text) {
  const lines = String(text || '').split(/\r?\n/);
  if (lines[0] !== '---') return null;
  let end = -1;
  for (let i = 1; i < lines.length; i += 1) {
    if (lines[i] === '---') {
      end = i;
      break;
    }
  }
  if (end === -1) return null;
  let date = null;
  let severity = null;
  for (let i = 1; i < end; i += 1) {
    const m = lines[i].match(/^([A-Za-z_]+)\s*:\s*(.*)$/);
    if (!m) continue;
    const key = m[1].toLowerCase();
    const val = m[2].trim();
    if (key === 'date') date = val || null;
    else if (key === 'severity') severity = val.toLowerCase() || null;
  }
  let title = '';
  for (let i = end + 1; i < lines.length; i += 1) {
    const h1 = lines[i].match(/^#\s+(.*)$/);
    if (h1) {
      title = h1[1].trim();
      break;
    }
  }
  return { date, severity, title };
}

/**
 * Gather raw candidates from every in-scope source, plus the scanned/absent
 * tally and a skipped count (malformed JSONL lines and trace-less/invalid cells
 * are skipped and counted, never thrown). Warnings (e.g. a symlink escaping
 * scope) are surfaced on console.warn and never read.
 *
 * @param {string} root - repo root
 */
export function collectFeedback(root) {
  const raw = [];
  const scanned = [];
  const absent = [];
  let skipped = 0;

  // ── .bee/backlog.jsonl ────────────────────────────────────────────────────
  // Read ONLY allowlist-relevant fields (type, title, ts, severity, layer).
  // NEVER `detail` / `predicted_impact` — that is the removed free-text surface.
  {
    const p = resolveInScope(root, SRC_BACKLOG);
    if (p === null) {
      absent.push(SRC_BACKLOG);
    } else {
      scanned.push(SRC_BACKLOG);
      for (const line of readText(p).split(/\r?\n/)) {
        const trimmed = line.trim();
        if (!trimmed) continue;
        let row;
        try {
          row = JSON.parse(trimmed);
        } catch {
          skipped += 1; // a malformed JSONL line is skipped and counted, never thrown
          continue;
        }
        if (!row || typeof row !== 'object') {
          skipped += 1;
          continue;
        }
        // backlog-unification D1: kind:'pbi' rows are event-sourced PBI
        // records (add/status/amend) — a wholly different shape from the
        // friction/proposal rows this loop otherwise reads. They carry no
        // `type` field at all, so without this skip every one would fall
        // through to resolveKind and get bucketed as an unknown_type drop.
        // Explicitly skipped instead: never merged, never dropped, never
        // counted as skipped either (a recognized shape, not malformed).
        if (row.kind === 'pbi') continue;
        const pain = typeof row.severity === 'string' && PAIN_SEVERITY[row.severity] ? PAIN_SEVERITY[row.severity] : 1;
        raw.push({ type: row.type, title: row.title, layer: row.layer, first_seen: row.ts, pain, source: SRC_BACKLOG });
      }
    }
  }

  // ── .bee/decisions.jsonl ──────────────────────────────────────────────────
  // A scope-guarded, counted source that emits NO entries in slice A. Decision
  // `decision`/`rationale` text is unbounded prose that names functions, files,
  // and config keys (this repo's own 8cd4c84e decision names readBacklogCounts,
  // COMMAND_KEYS, approved_gates.shape) — exactly the identifier leak the D2
  // allowlist removed. There is no allowlist field mapping for a decision event,
  // so it contributes nothing. It is still routed through resolveInScope so its
  // absence is a genuinely exercised skip-and-count path (must-have).
  {
    const p = resolveInScope(root, SRC_DECISIONS);
    if (p === null) absent.push(SRC_DECISIONS);
    else scanned.push(SRC_DECISIONS);
  }

  // ── .bee/cells/*.json ─────────────────────────────────────────────────────
  // From a cell trace we read ONLY blocked_reason PRESENCE and deviations LENGTH
  // — never their text, never trace.worker (free-form; may hold a human name).
  {
    const names = listInScope(root, SRC_CELLS);
    if (names === null) {
      absent.push(SRC_CELLS);
    } else {
      scanned.push(SRC_CELLS);
      for (const name of names) {
        if (!name.endsWith('.json')) continue;
        let resolved;
        try {
          resolved = resolveInScope(root, `${SRC_CELLS}/${name}`);
        } catch (err) {
          // A symlink (or anything) escaping scope is rejected, warned, NOT read.
          console.warn(`feedback: skipping ${SRC_CELLS}/${name} — ${err && err.message ? err.message : err}`);
          continue;
        }
        if (resolved === null) continue;
        const cell = readJson(resolved, null);
        const trace = cell && typeof cell === 'object' ? cell.trace : null;
        if (!trace || typeof trace !== 'object') {
          skipped += 1; // a cell without a trace is skipped and counted, never thrown
          continue;
        }
        const source = typeof cell.id === 'string' && cell.id ? cell.id : `${SRC_CELLS}/${name}`;
        const firstSeen =
          (typeof trace.capped_at === 'string' && trace.capped_at) ||
          (typeof trace.claimed_at === 'string' && trace.claimed_at) ||
          null;
        const title = typeof cell.title === 'string' ? cell.title : '';
        if (trace.blocked_reason) {
          raw.push({ type: 'blocked', title, layer: null, first_seen: firstSeen, pain: 1, source });
        }
        if (Array.isArray(trace.deviations) && trace.deviations.length > 0) {
          raw.push({ type: 'deviation', title, layer: null, first_seen: firstSeen, pain: 1, source });
        }
      }
    }
  }

  // ── docs/history/learnings/*.md frontmatter ───────────────────────────────
  // Read frontmatter only: the `date`, `severity` (low/medium/high scale), and
  // the H1 title. Never the body prose.
  {
    const names = listInScope(root, SRC_LEARNINGS);
    if (names === null) {
      absent.push(SRC_LEARNINGS);
    } else {
      scanned.push(SRC_LEARNINGS);
      for (const name of names) {
        if (!name.endsWith('.md') || name === 'critical-patterns.md') continue;
        let resolved;
        try {
          resolved = resolveInScope(root, `${SRC_LEARNINGS}/${name}`);
        } catch (err) {
          console.warn(`feedback: skipping ${SRC_LEARNINGS}/${name} — ${err && err.message ? err.message : err}`);
          continue;
        }
        if (resolved === null) continue;
        const parsed = parseLearningFrontmatter(readText(resolved));
        if (!parsed) {
          skipped += 1;
          continue;
        }
        const pain = PAIN_LMH[parsed.severity] || 1;
        raw.push({
          type: 'learning',
          title: parsed.title,
          layer: null,
          first_seen: parsed.date,
          pain,
          source: `${SRC_LEARNINGS}/${name}`,
        });
      }
    }
  }

  return { raw, scanned: scanned.sort(), absent: absent.sort(), skipped };
}

// Deterministic sort key so buildDigest is byte-identical across runs regardless
// of filesystem enumeration order. generated_at is the ONLY volatile field.
function sortKey(o) {
  // Separator must be a printable, non-C0 sentinel — a raw C0 control byte
  // (e.g. NUL) here makes grep/rg treat this whole file as BINARY and print
  // nothing, not even a zero count, silently defeating any grep-based drift
  // guard over this file (critical-patterns.md 20260710). U+241F (SYMBOL FOR
  // UNIT SEPARATOR, a printable Control Pictures glyph, not itself a control
  // character) is vanishingly unlikely to appear in these fields and keeps
  // the file plain UTF-8 text.
  return [o.first_seen ?? '', o.kind ?? '', o.source ?? '', o.title ?? '', o.reason ?? ''].join('\u241F');
}

/**
 * Build the feedback digest — a SNAPSHOT rebuilt from scratch each call, never
 * appended. The injected clock {now} (a Date or ISO string) pins generated_at so
 * the test can prove byte-identical output. Shape:
 *   { schema_version, generated_at, repo_label, counts, dropped, entries }
 *
 * @param {string} root - repo root
 * @param {{now?: (Date|string)}} [opts]
 */
export function buildDigest(root, { now } = {}) {
  const generatedAt = now instanceof Date ? now.toISOString() : typeof now === 'string' ? now : new Date().toISOString();
  let repoLabel;
  try {
    repoLabel = path.basename(fs.realpathSync(root));
  } catch {
    repoLabel = path.basename(root);
  }

  const { raw, scanned, absent, skipped } = collectFeedback(root);
  const dropped = [];
  const entries = [];
  for (const candidate of raw) {
    const entry = buildEntry(candidate, dropped);
    if (entry) entries.push(entry);
  }

  entries.sort((a, b) => sortKey(a).localeCompare(sortKey(b)));
  dropped.sort((a, b) => sortKey(a).localeCompare(sortKey(b)));

  const byKind = {};
  for (const e of entries) byKind[e.kind] = (byKind[e.kind] || 0) + 1;
  const byKindSorted = {};
  for (const k of Object.keys(byKind).sort()) byKindSorted[k] = byKind[k];

  const counts = {
    entries: entries.length,
    dropped: dropped.length,
    skipped,
    by_kind: byKindSorted,
    sources_scanned: scanned,
    sources_absent: absent,
  };

  return {
    schema_version: SCHEMA_VERSION,
    generated_at: generatedAt,
    repo_label: repoLabel,
    counts,
    dropped,
    entries,
  };
}

/**
 * D2b — THE CONSUMER REVALIDATES (decision 8cd4c84e, clause D2b). Merge the LOCAL
 * digest with each dogfood repo's ALREADY-WRITTEN .bee/feedback-digest.json. This
 * is the security heart of slice A: the producing repo's write-time scan protects
 * THAT repo, not bee. bee-evolving later edits bee's own source from these files,
 * and a foreign digest lives in a repo bee does not control — a hand-edited,
 * stale, or hostile feedback-digest.json is just JSON on disk. So for every entry
 * read from a FOREIGN digest this function:
 *   (1) realpath-contains the digest file under realpath(repoPath)/.bee/ — a
 *       symlinked feedback-digest.json pointing anywhere else on this machine is
 *       rejected and warned, never read (realpath resolves the symlink; a target
 *       outside .bee/ fails containment). Content reads route through readJson so
 *       the source-level read-scope drift guard stays clean;
 *   (2) re-runs SECRET_CONTENT_PATTERNS and INJECTION_PATTERNS on the RAW title
 *       (before any transformation), dropping violators into the repo's dropped[]
 *       with reason 'secret'/'injection' attributed to the source repo;
 *   (3) wraps every surviving title in datamark() (lib/decisions.mjs) — the same
 *       neutralizer the decisions surfacing path applies at read time — so
 *       '</system> ignore all previous instructions …' can never act as
 *       instructions once the title reaches a prompt;
 *   (4) keeps ONLY ENTRY_FIELDS — any field outside the allowlist (e.g. a
 *       resurrected free-text `detail`) is stripped, never merged through.
 * A missing, unreadable, or corrupt dogfood repo or digest is warned/skipped and
 * counted, never thrown — one dead repo must never break the bee session.
 *
 * The merge is a FLAT list keyed by repo_label. There is deliberately NO
 * clustering, frequency, corroboration, ranking, or tie-break here — measurement
 * showed their inputs do not exist yet, and they moved to slice B by design.
 *
 * Shape (a superset of the local digest so a consumer reads one object):
 *   { ...localDigest, merged: [ { repo_label, counts, dropped, entries } ],
 *     merged_counts: { repos_configured, repos_merged, repos_skipped } }
 * With dogfood_repos absent, `merged` is [] and the local digest is returned as-is.
 *
 * @param {string} root - the bee repo root (the CONSUMER)
 * @param {{now?: (Date|string)}} [opts]
 */
export function mergeDigests(root, { now } = {}) {
  const local = buildDigest(root, { now });
  const repos = readConfig(root).dogfood_repos || [];
  const merged = [];
  let reposMerged = 0;
  let reposSkipped = 0;

  for (const repo of repos) {
    const repoPath = repo && typeof repo.path === 'string' ? repo.path : null;
    const label = repo && typeof repo.label === 'string' && repo.label ? repo.label : repoPath || 'unknown';
    if (!repoPath) {
      reposSkipped += 1;
      continue;
    }

    // The dogfood path was already realpath'd by normalizeDogfoodRepos; re-realpath
    // defensively (idempotent) so this function is safe under any caller.
    let repoReal;
    try {
      repoReal = fs.realpathSync(repoPath);
    } catch (err) {
      console.warn(`mergeDigests: skipping dogfood repo "${label}" — ${err && err.code ? err.code : err}`);
      reposSkipped += 1;
      continue;
    }

    // Read ONLY the repo's written digest — never its raw .bee/ or docs/.
    const digestPath = path.resolve(repoReal, '.bee', 'feedback-digest.json');
    let realDigest;
    try {
      realDigest = fs.realpathSync(digestPath);
    } catch (err) {
      // No digest yet (a real repo that has never closed) or unreadable → skip+count.
      if (!(err && err.code === 'ENOENT')) {
        console.warn(`mergeDigests: skipping "${label}" digest — ${err && err.code ? err.code : err}`);
      }
      reposSkipped += 1;
      continue;
    }

    // Realpath containment (D2b, clause 1): the resolved digest must be a REGULAR
    // FILE under realpath(repoPath)/.bee/. A symlinked feedback-digest.json whose
    // real target sits anywhere else is rejected, warned, and never read.
    const beeDir = path.join(repoReal, '.bee');
    let stat;
    try {
      stat = fs.lstatSync(realDigest);
    } catch {
      reposSkipped += 1;
      continue;
    }
    const contained = realDigest.startsWith(beeDir + path.sep);
    if (!contained || !stat.isFile()) {
      console.warn(
        `mergeDigests: rejecting "${label}" feedback-digest.json — real path "${realDigest}" is outside realpath(${label})/.bee/ or is not a regular file (D2b realpath containment)`,
      );
      reposSkipped += 1;
      continue;
    }

    const foreign = readJson(realDigest, null);
    if (!foreign || typeof foreign !== 'object' || !Array.isArray(foreign.entries)) {
      // A corrupt or shapeless digest is skipped and counted, never thrown.
      reposSkipped += 1;
      continue;
    }

    const entries = [];
    const dropped = [];
    for (const raw of foreign.entries) {
      if (!raw || typeof raw !== 'object') continue;
      // Route EVERY foreign entry through the SAME construction path the local
      // producer uses (buildEntry), with neutralize:true. This is the fix for
      // review P1-1: the old ad-hoc copy loop scanned/datamarked title alone and
      // copied source/layer/kind/pain/first_seen raw, so an attacker moved the
      // payload out of title and walked through clean. A foreign entry spells its
      // normalized type as `kind`; buildEntry re-normalizes it through
      // KIND_ALIASES exactly as the local `type` path does — an unknown kind lands
      // in dropped[] as 'unknown_type' rather than being merged raw. Any field
      // outside ENTRY_FIELDS (e.g. a resurrected `detail`) is dropped by
      // construction, because the returned entry is built field by field.
      const entry = buildEntry(
        {
          type: raw.kind,
          title: raw.title,
          layer: raw.layer,
          source: raw.source,
          first_seen: raw.first_seen,
          pain: raw.pain,
        },
        dropped,
        { neutralize: true },
      );
      if (entry) entries.push(entry);
    }

    entries.sort((a, b) => sortKey(a).localeCompare(sortKey(b)));
    dropped.sort((a, b) => sortKey(a).localeCompare(sortKey(b)));
    merged.push({
      repo_label: label,
      counts: { entries: entries.length, dropped: dropped.length },
      dropped,
      entries,
    });
    reposMerged += 1;
  }

  merged.sort((a, b) => String(a.repo_label).localeCompare(String(b.repo_label)));

  return {
    ...local,
    merged,
    merged_counts: {
      repos_configured: repos.length,
      repos_merged: reposMerged,
      repos_skipped: reposSkipped,
    },
  };
}

// ─── slice B — ranking (P18, plan.md revision 3) ───────────────────────────
//
// The cluster key defuses TWO traps in one pass:
//   (1) the datamark asymmetry trap — a foreign title is stored wrapped in
//       «…» (mergeDigests, D2b) while an identical LOCAL title is stored bare
//       (buildDigest never wraps its own trusted output). A naive string
//       comparison never unifies them.
//   (2) datamark's own double-wrap non-idempotence — datamark(datamark(t))
//       produces «« t »», not «t» — so a title that has been merged twice (or
//       whose source repo itself already carried a wrapped title) needs a
//       FIXED-POINT strip, not a single strip.
//
// normalizeTitle strips the wrapper to fixed point, then re-applies the SAME
// cleaning transforms datamark() itself runs (decisions.mjs:145-149 — fence
// strip, role-tag strip, C0 control-char strip, trim) so the invariant
// normalizeTitle(datamark(t)) === normalizeTitle(t) holds even when t already
// carries a fence or a role tag (plan-checker finding W4): a bare local title
// must match its datamarked foreign twin. It is intentionally NOT exported for
// rendering — key is an INTERNAL clustering handle; every surface that shows a
// title uses the stored (still-wrapped-for-foreign) entry.title, never this.
export function normalizeTitle(title) {
  let text = String(title ?? '');

  // (1) Strip the «…» datamark wrapper to fixed point.
  for (;;) {
    const trimmed = text.trim();
    if (trimmed.length >= 2 && trimmed.startsWith('«') && trimmed.endsWith('»')) {
      text = trimmed.slice(1, -1);
    } else {
      text = trimmed;
      break;
    }
  }

  // (2) Apply datamark's OWN cleaning transforms (decisions.mjs:145-149) so a
  // title carrying a fence, a role tag, or a control char normalizes the same
  // way whether it arrived bare (local) or wrapped (foreign, already cleaned
  // once by datamark).
  const cleaned = text
    .replace(/```+/g, '')
    .replace(/<\/?\s*(?:system|assistant|user|developer|tool)\b[^>]*>/gi, '')
    .replace(/[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F]/g, '')
    .trim();

  // (3) Casefold + collapse whitespace so purely-cosmetic differences (case,
  // repeated spaces) never split one friction into two clusters.
  return cleaned.toLowerCase().replace(/\s+/g, ' ').trim();
}

/**
 * Group every entry in a merged view into clusters keyed by normalizeTitle.
 * Iterates BOTH the local `.entries` (repo = view.repo_label) and each
 * `merged[i].entries` (repo = merged[i].repo_label) — the shape mergeDigests
 * returns. Each cluster is { key, entries, pain, frequency, corroboration }:
 *   - pain          = max entry.pain in the cluster (default 1 for a
 *                      malformed/missing pain, matching buildEntry's own
 *                      local default so a cluster is never rank-starved by a
 *                      hole in the data)
 *   - frequency      = cluster size (total contributing entries)
 *   - corroboration  = count of DISTINCT repos contributing (the local repo
 *                      counts as one repo, same as every foreign repo)
 *
 * Never throws on an empty or malformed view — an absent `.entries` /
 * `.merged` is treated as [], so `clusterEntries({})` returns [].
 *
 * @param {object} mergedView - the shape returned by mergeDigests (or
 *   buildDigest, which has no `.merged`)
 * @returns {Array<{key: string, entries: object[], pain: number, frequency: number, corroboration: number}>}
 */
export function clusterEntries(mergedView) {
  const view = mergedView && typeof mergedView === 'object' ? mergedView : {};
  const localEntries = Array.isArray(view.entries) ? view.entries : [];
  const mergedRepos = Array.isArray(view.merged) ? view.merged : [];
  const localLabel = typeof view.repo_label === 'string' && view.repo_label ? view.repo_label : 'local';

  const buckets = new Map();
  const order = [];

  const addEntry = (entry, repoLabel) => {
    if (!entry || typeof entry !== 'object') return;
    const key = normalizeTitle(entry.title);
    let bucket = buckets.get(key);
    if (!bucket) {
      bucket = { key, entries: [], pain: 0, repos: new Set() };
      buckets.set(key, bucket);
      order.push(key);
    }
    bucket.entries.push(entry);
    bucket.repos.add(repoLabel);
    const pain = Number.isInteger(entry.pain) && entry.pain > 0 ? entry.pain : 1;
    if (pain > bucket.pain) bucket.pain = pain;
  };

  for (const entry of localEntries) addEntry(entry, localLabel);
  for (const repo of mergedRepos) {
    if (!repo || typeof repo !== 'object') continue;
    const label = typeof repo.repo_label === 'string' && repo.repo_label ? repo.repo_label : 'unknown';
    const entries = Array.isArray(repo.entries) ? repo.entries : [];
    for (const entry of entries) addEntry(entry, label);
  }

  return order.map((key) => {
    const bucket = buckets.get(key);
    return {
      key: bucket.key,
      entries: bucket.entries,
      pain: bucket.pain,
      frequency: bucket.entries.length,
      corroboration: bucket.repos.size,
    };
  });
}

// Earliest first_seen among a cluster's entries — the tie-break input. A
// cluster with no valid first_seen anywhere sorts as '' (lexicographically
// earliest), which is deterministic even though it is not chronological.
function clusterFirstSeen(cluster) {
  let earliest = null;
  const entries = Array.isArray(cluster.entries) ? cluster.entries : [];
  for (const entry of entries) {
    const firstSeen = entry && typeof entry.first_seen === 'string' ? entry.first_seen : null;
    if (firstSeen && (earliest === null || firstSeen < earliest)) earliest = firstSeen;
  }
  return earliest;
}

/**
 * Rank clusters by rank = pain * frequency * corroboration, descending.
 * Deterministic tie-break: earliest first_seen ascending, then cluster key
 * lexicographic — so two runs over the SAME pinned digest are byte-identical
 * (the same idempotence discipline buildDigest's clock injection proves).
 * No non-deterministic input (clock, LLM, filesystem order) reaches this
 * computation — it is a pure sort over its argument.
 *
 * @param {Array<{key: string, entries: object[], pain: number, frequency: number, corroboration: number}>} clusters
 * @returns {Array<{key: string, entries: object[], pain: number, frequency: number, corroboration: number, rank: number, first_seen: string|null}>}
 */
export function rankClusters(clusters) {
  const list = Array.isArray(clusters) ? clusters : [];
  const ranked = list.map((cluster) => ({
    ...cluster,
    rank: cluster.pain * cluster.frequency * cluster.corroboration,
    first_seen: clusterFirstSeen(cluster),
  }));
  ranked.sort((a, b) => {
    if (b.rank !== a.rank) return b.rank - a.rank;
    const aFirstSeen = a.first_seen ?? '';
    const bFirstSeen = b.first_seen ?? '';
    if (aFirstSeen !== bFirstSeen) return aFirstSeen < bFirstSeen ? -1 : 1;
    if (a.key < b.key) return -1;
    if (a.key > b.key) return 1;
    return 0;
  });
  return ranked;
}
