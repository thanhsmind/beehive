// knowledge.mjs — OKF v0.1 knowledge-bundle core (okf-foundation S1, cell
// okf-1): the emitter-first frontmatter codec, the Bee OKF Profile concept
// model, and the two-level bundle checker behind `bee knowledge check`.
//
// Ground rules, all locked in docs/history/okf-foundation/CONTEXT.md:
//   D12 — zero dependencies. This file ships its own frontmatter parser
//         covering EXACTLY the YAML subset emitFrontmatter() produces, and
//         fails loudly (typed {code,message,line}) on anything outside it.
//         The emitter is the subset's single source of truth: "what parses"
//         is defined as "what the emitter can produce", never as "what YAML
//         allows". Hand-edited files that still parse but would not re-emit
//         byte-identically are surfaced as a not_canonical profile warning
//         (advisor digest s1 items 1-2) — the silent-misparse class (colon in
//         an unquoted title, '#' mid-value, CRLF line endings) becomes a
//         detectable finding instead of a wrong-without-erroring parse.
//   D23 — a concept is any non-reserved .md INSIDE docs/knowledge/; every
//         concept-facing function here reads only the bundle directory. The
//         single deliberate exception is `promote` (D38 below), which also
//         READS .bee/cells/*.json — never writes it.
//   D2  — the bundle is the knowledge layer, never a write path into
//         .bee/*.json(l) runtime stores; check and list perform no writes at
//         all, and index writes ONLY generated index.md files inside
//         docs/knowledge/ — never a runtime store, never a file outside the
//         bundle.
//   D21 — `index` (cell okf-4, S3) generates index.md at every directory
//         level with concepts plus the root index.md (sole okf_version
//         carrier), byte-identically from concept frontmatter: path-sorted
//         ordering, LF endings, no wall-clock values. `index --check`
//         re-renders in memory and diffs against disk — the decisions.mjs
//         render --check idiom.
//   D15 — `list` emits one row per concept (path, id, type, lifecycle,
//         title), never content.
//   D27 — `context` (cell okf-7, S5) resolves a work item by bee.id, walks
//         required_context transitively with a cycle guard (a cycle is
//         deduped silently, never an error), adds the area decisions and
//         every bee.critical concept, ranks, and cuts at a token budget
//         estimated as bytes/4 — the estimator is NAMED in the output. The
//         result is an ordered manifest of paths, sizes and one-line reasons:
//         never file content.
//   G5  — `context` RANKS the critical concepts by relevance to the work item
//         and CUTS them (supersedes D27's include-every rule), keeping a small
//         guaranteed floor of the highest-scoring against the budget.
//   G11 — the cut is conserving and audited: every critical not in `entries`
//         is named in `truncated` or in `excluded` with its score and reason,
//         and a ranking where most of a real population scores zero FAILS the
//         run rather than shipping as a path sort wearing a relevance label.
//   D38 — `promote` (cell okf-9, S7) closes the learning loop the only way
//         D2 allows: it READS the bundle and the CAPPED cell traces in
//         .bee/cells/*.json (a read of the runtime store — permitted; never a
//         write) and returns three PROPOSALS — a delivery draft in canonical
//         emitter form, candidate area spec-sync bullets, and candidate
//         pitfall patterns. It is the one function in this module that looks
//         outside docs/knowledge/, and it still writes NOTHING, anywhere:
//         applying a proposal is a human or agent decision.
//   D4  — check reports two levels: OKF errors (the spec's own MUSTs) and
//         profile warnings (bee's SHOULD layer); --strict promotes warnings.
//   D13 — the CLI handler (bee.mjs) emits {okf:{errors},profile:{warnings},
//         counts} and exits non-zero only on OKF errors, or on any finding
//         under --strict.
//   D18 — the type vocabulary is closed at nine, slug-cased.
//   D19/D32 — concept frontmatter carries type + title/description/tags/
//         timestamp (+ resource for an external asset) + a nested bee: map;
//         bee.id is identity, paths are link targets.

import fs from 'node:fs';
import path from 'node:path';

import { resolveProductRoot } from './state.mjs';

export const OKF_VERSION = '0.1';

// D18: the closed nine-type vocabulary. "Pitfall" is bee.pattern with
// bee.polarity: pitfall — never a tenth type.
export const CONCEPT_TYPES = [
  'bee.area',
  'bee.feature',
  'bee.work-item',
  'bee.plan',
  'bee.delivery',
  'bee.decision',
  'bee.pattern',
  'bee.runbook',
  'bee.evidence',
];

// D19: the four lifecycle states of bee.lifecycle.
export const LIFECYCLES = ['draft', 'active', 'superseded', 'archived'];

// Canonical emission order (D19/D32). Unknown keys are legal input (OKF §8:
// consumers MUST NOT reject unknown fields) and are emitted after the known
// keys in sorted order — root-level unknowns before the bee: map, bee-level
// unknowns at the end of the map — so a canonical file with unknown keys
// still round-trips byte-identically.
const ROOT_KEY_ORDER = ['type', 'title', 'description', 'tags', 'timestamp', 'resource'];
const BEE_KEY_ORDER = [
  'id',
  'lifecycle',
  'areas',
  'required_context',
  'decisions',
  'sources',
  'lane',
  'polarity',
  'critical',
  'authoritative_for',
  'review_status',
  'supersedes',
  'superseded_by',
];

// Profile-required fields (D4 "missing profile-required field"; D10: the
// migrator never invents title/description — check warns instead). bee.id and
// bee.lifecycle join them because identity (D31 uniqueness) and lifecycle
// (D19) are what every consumer keys on.
const PROFILE_REQUIRED = [
  ['title'],
  ['description'],
  ['bee', 'id'],
  ['bee', 'lifecycle'],
];

const KEY_RE = /^[A-Za-z_][A-Za-z0-9_.-]*$/;

// OKF §3.1 reserved names — not concepts, no frontmatter obligation (D23).
const RESERVED_BASENAMES = new Set(['index.md', 'log.md']);

/**
 * The single directory this module is allowed to read (D17/D23).
 *
 * G13 (cell f3-3): `docs/knowledge/` is a PRODUCT doc tree, exactly like
 * `docs/specs/`, `docs/backlog.md` and the product README — so it resolves
 * against `resolveProductRoot(root)`, the same path inject.mjs:71,
 * backlog.mjs:19/266 and hooks/bee-session-close.mjs:110-118 already take.
 * For every ordinary single-root repo `resolveProductRoot` returns `root`
 * unchanged, so this is a zero-behaviour-change no-op there. It matters in
 * exactly one supported topology: the repo divorce (`.bee/config.json`
 * `product_root`, GitHub #14), where `.bee/` sits in a workshop root and the
 * product is an independent repo one directory down. Resolving off the
 * workshop root there graded a fully migrated host as bundle-LESS and pointed
 * its fallback at an empty workshop `docs/specs/` — new knowledge written
 * where nobody reads it, no error: the silent-rot class this work exists to
 * prevent.
 */
export function bundleDir(root) {
  return path.join(resolveProductRoot(root), 'docs', 'knowledge');
}

// ─── emitter (the subset's source of truth, D12) ────────────────────────────

function isPlainSafe(value) {
  if (typeof value !== 'string' || value.length === 0) return false;
  if (value !== value.trim()) return false;
  // Anything YAML could re-interpret (colon, hash, quotes, flow/block
  // indicators, escapes, control whitespace) is emitted JSON-quoted instead.
  if (/[:#"'\\\[\]{},\t\r\n]/.test(value)) return false;
  if (/^[-?&*!|>%@`]/.test(value)) return false;
  if (value === 'true' || value === 'false' || value === 'null') return false;
  return true;
}

function emitScalar(value, keyPath) {
  if (value === true) return 'true';
  if (value === false) return 'false';
  if (typeof value !== 'string') {
    throw new Error(
      `emitFrontmatter: "${keyPath}" must be a string or boolean (the profile emits no other scalar kinds), got ${typeof value}.`,
    );
  }
  return isPlainSafe(value) ? value : JSON.stringify(value);
}

function emitValue(value, keyPath) {
  if (Array.isArray(value)) {
    return `[${value.map((item, i) => emitScalar(item, `${keyPath}[${i}]`)).join(', ')}]`;
  }
  return emitScalar(value, keyPath);
}

function assertEmittableKey(key, keyPath) {
  if (!KEY_RE.test(key)) {
    throw new Error(`emitFrontmatter: key "${keyPath}" is not a legal frontmatter key.`);
  }
}

function emitEntries(lines, map, order, indent, prefix) {
  const known = order.filter((key) => key in map);
  const unknown = Object.keys(map)
    .filter((key) => !order.includes(key) && key !== 'bee')
    .sort();
  for (const key of [...known, ...unknown]) {
    const keyPath = prefix ? `${prefix}.${key}` : key;
    assertEmittableKey(key, keyPath);
    const value = map[key];
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      throw new Error(
        `emitFrontmatter: "${keyPath}" is a nested map — the profile's only nested map is the root-level "bee:".`,
      );
    }
    lines.push(`${indent}${key}: ${emitValue(value, keyPath)}`);
  }
}

/**
 * Emit the canonical frontmatter block (including both --- delimiter lines,
 * LF-terminated) for a concept data object. This function DEFINES the D12
 * subset: parseFrontmatter accepts exactly what this can produce, and the
 * checker's round-trip guard compares a file's real bytes against a re-emit
 * of its parsed data.
 */
export function emitFrontmatter(data) {
  if (!data || typeof data !== 'object' || Array.isArray(data)) {
    throw new Error('emitFrontmatter: data must be a plain object.');
  }
  const lines = ['---'];
  emitEntries(lines, data, ROOT_KEY_ORDER, '', '');
  if ('bee' in data) {
    const bee = data.bee;
    if (!bee || typeof bee !== 'object' || Array.isArray(bee)) {
      throw new Error('emitFrontmatter: "bee" must be a plain object.');
    }
    lines.push('bee:');
    emitEntries(lines, bee, BEE_KEY_ORDER, '  ', 'bee');
  }
  lines.push('---');
  return `${lines.join('\n')}\n`;
}

// ─── parser (accepts exactly the emitted subset; loud typed failure) ────────

function parseFailure(code, message, line) {
  return { ok: false, present: true, error: { code, message, line } };
}

function parseScalarToken(raw, lineNo) {
  if (raw === 'true') return { ok: true, value: true };
  if (raw === 'false') return { ok: true, value: false };
  if (raw.startsWith('"')) {
    let parsed;
    try {
      parsed = JSON.parse(raw);
    } catch {
      return parseFailure(
        'bad_quoted_string',
        `quoted value ${JSON.stringify(raw)} is not one complete JSON string`,
        lineNo,
      );
    }
    if (typeof parsed !== 'string') {
      return parseFailure('bad_quoted_string', 'quoted value did not decode to a string', lineNo);
    }
    return { ok: true, value: parsed };
  }
  if (raw.startsWith("'")) {
    return parseFailure(
      'single_quoted_string',
      'single-quoted scalars are outside the emitted subset — use double quotes',
      lineNo,
    );
  }
  if (/^[&*!|>%@`{}]/.test(raw)) {
    return parseFailure(
      'unsupported_scalar',
      `value starting with "${raw[0]}" (anchor/alias/block/flow-map indicator) is outside the emitted subset`,
      lineNo,
    );
  }
  // Plain scalar: the ENTIRE rest of the line, colons and hashes included.
  // The emitter would have quoted such a value, so keeping it as data here is
  // what turns "colon in an unquoted title" / "# mid-value" into a
  // round-trip (not_canonical) warning instead of a silent misparse.
  return { ok: true, value: raw };
}

function parseFlowList(raw, lineNo) {
  if (!raw.endsWith(']')) {
    return parseFailure('bad_flow_list', `flow list ${JSON.stringify(raw)} does not close with "]"`, lineNo);
  }
  const inner = raw.slice(1, -1).trim();
  if (inner === '') return { ok: true, value: [] };
  const segments = [];
  let current = '';
  let inQuote = false;
  let escaped = false;
  for (const ch of inner) {
    if (inQuote) {
      current += ch;
      if (escaped) escaped = false;
      else if (ch === '\\') escaped = true;
      else if (ch === '"') inQuote = false;
    } else if (ch === '"') {
      current += ch;
      inQuote = true;
    } else if (ch === ',') {
      segments.push(current);
      current = '';
    } else {
      current += ch;
    }
  }
  if (inQuote) {
    return parseFailure('bad_flow_list', 'unterminated quoted item inside flow list', lineNo);
  }
  segments.push(current);
  const value = [];
  for (const segment of segments) {
    const token = segment.trim();
    if (token === '') {
      return parseFailure('bad_flow_list', 'empty item inside flow list', lineNo);
    }
    const parsed = parseScalarToken(token, lineNo);
    if (!parsed.ok) return parsed;
    value.push(parsed.value);
  }
  return { ok: true, value };
}

function parseKeyValueLine(line, target, lineNo, prefix) {
  const sep = line.indexOf(': ');
  if (sep === -1) {
    return parseFailure(
      'unrecognized_line',
      `line ${JSON.stringify(line)} is not "key: value", a "bee:" map header, or a closing "---"`,
      lineNo,
    );
  }
  const key = line.slice(0, sep);
  if (!KEY_RE.test(key)) {
    return parseFailure('bad_key', `${JSON.stringify(key)} is not a legal frontmatter key`, lineNo);
  }
  if (Object.prototype.hasOwnProperty.call(target, key)) {
    return parseFailure('duplicate_key', `duplicate key "${prefix}${key}"`, lineNo);
  }
  const raw = line.slice(sep + 2);
  if (raw === '') {
    return parseFailure('empty_value', `key "${prefix}${key}" has no value after ": "`, lineNo);
  }
  const parsed = raw.startsWith('[') ? parseFlowList(raw, lineNo) : parseScalarToken(raw, lineNo);
  if (!parsed.ok) return parsed;
  target[key] = parsed.value;
  return { ok: true };
}

/**
 * Parse a file's frontmatter. Returns one of:
 *   { ok: true,  present: false }                      — no leading "---"
 *   { ok: true,  present: true, data, block, body }    — parsed; block is the
 *     EXACT frontmatter bytes from the file (delimiters included), body the
 *     rest — so callers can byte-compare block against emitFrontmatter(data)
 *   { ok: false, present: true, error: {code,message,line} } — loud typed
 *     failure, anything outside the emitted subset (D12)
 *
 * CRLF input parses (each line's trailing \r is stripped) so the DATA is
 * never mangled; the raw block keeps its \r bytes, which the round-trip
 * guard then reports as not_canonical.
 */
export function parseFrontmatter(text) {
  if (typeof text !== 'string') {
    return parseFailure('not_text', 'input is not a string', 0);
  }
  if (!text.startsWith('---\n') && !text.startsWith('---\r\n')) {
    return { ok: true, present: false };
  }
  const openLen = text.startsWith('---\r\n') ? 5 : 4;

  // Locate the closing "---" line, tracking exact byte offsets so `block`
  // reproduces the file's own bytes.
  let cursor = openLen;
  let blockEnd = -1;
  let innerEnd = -1;
  let lineNo = 1;
  while (cursor <= text.length) {
    lineNo += 1;
    const nl = text.indexOf('\n', cursor);
    const hasNewline = nl !== -1;
    const lineEnd = hasNewline ? nl : text.length;
    let line = text.slice(cursor, lineEnd);
    if (line.endsWith('\r')) line = line.slice(0, -1);
    if (line === '---') {
      innerEnd = cursor;
      blockEnd = hasNewline ? nl + 1 : text.length;
      break;
    }
    if (!hasNewline) break;
    cursor = nl + 1;
  }
  if (blockEnd === -1) {
    return parseFailure('unclosed_frontmatter', 'frontmatter opened with "---" but never closed', 1);
  }

  const block = text.slice(0, blockEnd);
  const body = text.slice(blockEnd);
  const innerRaw = text.slice(openLen, innerEnd);
  const innerLines = innerRaw === '' ? [] : innerRaw.split('\n').slice(0, -1);

  const data = {};
  let inBeeMap = false;
  let currentLineNo = 1;
  for (let rawLine of innerLines) {
    currentLineNo += 1;
    if (rawLine.endsWith('\r')) rawLine = rawLine.slice(0, -1);
    if (rawLine === '') {
      return parseFailure('blank_line', 'blank line inside frontmatter is outside the emitted subset', currentLineNo);
    }
    if (rawLine.includes('\t')) {
      return parseFailure('tab_in_frontmatter', 'tab character inside frontmatter is outside the emitted subset', currentLineNo);
    }
    if (rawLine.startsWith('  ')) {
      if (!inBeeMap) {
        return parseFailure('unexpected_indent', 'indented line outside the "bee:" map', currentLineNo);
      }
      const inner = rawLine.slice(2);
      if (inner.startsWith(' ')) {
        return parseFailure('bad_indent', 'bee: map entries are indented exactly two spaces', currentLineNo);
      }
      const result = parseKeyValueLine(inner, data.bee, currentLineNo, 'bee.');
      if (!result.ok) return result;
      continue;
    }
    if (rawLine.startsWith(' ')) {
      return parseFailure('bad_indent', 'root-level lines must not be indented', currentLineNo);
    }
    inBeeMap = false;
    const header = /^([^:\s]+):$/.exec(rawLine);
    if (header) {
      const key = header[1];
      if (!KEY_RE.test(key)) {
        return parseFailure('bad_key', `${JSON.stringify(key)} is not a legal frontmatter key`, currentLineNo);
      }
      if (key !== 'bee') {
        return parseFailure(
          'unsupported_map',
          `nested map "${key}:" is outside the emitted subset (the only nested map is "bee:")`,
          currentLineNo,
        );
      }
      if (Object.prototype.hasOwnProperty.call(data, 'bee')) {
        return parseFailure('duplicate_key', 'duplicate key "bee"', currentLineNo);
      }
      data.bee = {};
      inBeeMap = true;
      continue;
    }
    const result = parseKeyValueLine(rawLine, data, currentLineNo, '');
    if (!result.ok) return result;
  }

  return { ok: true, present: true, data, block, body };
}

// ─── bundle walk (D23: never leaves docs/knowledge/) ────────────────────────

function listBundleMarkdown(dir) {
  const out = [];
  const walk = (abs, rel) => {
    let entries;
    try {
      entries = fs.readdirSync(abs, { withFileTypes: true });
    } catch {
      return;
    }
    for (const entry of entries) {
      if (entry.isSymbolicLink()) continue; // a symlink could escape the bundle — never follow (D23)
      const childRel = rel ? `${rel}/${entry.name}` : entry.name;
      if (entry.isDirectory()) walk(path.join(abs, entry.name), childRel);
      else if (entry.isFile() && entry.name.endsWith('.md')) out.push(childRel);
    }
  };
  if (fs.existsSync(dir)) walk(dir, '');
  return out.sort();
}

// ISO 8601 date, optionally with a time part — OKF §7 for log.md headings.
const ISO_HEADING_RE = /^(\d{4})-(\d{2})-(\d{2})(?:[T ]\d{2}:\d{2}(?::\d{2}(?:\.\d+)?)?(?:Z|[+-]\d{2}:?\d{2})?)?$/;

function isIsoDateHeading(text) {
  const match = ISO_HEADING_RE.exec(text);
  if (!match) return false;
  const [, y, m, d] = match;
  const date = new Date(Date.UTC(Number(y), Number(m) - 1, Number(d)));
  return date.getUTCFullYear() === Number(y) && date.getUTCMonth() === Number(m) - 1 && date.getUTCDate() === Number(d);
}

function checkIndexFile(rel, text, errors) {
  const parsed = parseFrontmatter(text);
  const isRoot = rel === 'index.md';
  if (!isRoot) {
    // D4: frontmatter in a non-root index.md is an OKF error — presence alone
    // decides; parseability does not rescue it (§6: index files carry none).
    if (parsed.present) {
      errors.push({
        file: rel,
        code: 'index_frontmatter',
        message: 'a non-root index.md must not carry frontmatter (OKF §6; D4)',
      });
    }
    return;
  }
  if (!parsed.present) return; // generator arrives in S3; an absent block carries no illegal keys
  if (!parsed.ok) {
    errors.push({
      file: rel,
      code: 'unparseable_frontmatter',
      message: `root index.md frontmatter is unparseable — ${parsed.error.code}: ${parsed.error.message} (line ${parsed.error.line})`,
    });
    return;
  }
  const extra = Object.keys(parsed.data).filter((key) => key !== 'okf_version');
  if (extra.length > 0) {
    errors.push({
      file: rel,
      code: 'root_index_extra_keys',
      message: `root index.md may carry only okf_version (OKF §9); found extra key(s): ${extra.join(', ')}`,
    });
  }
}

function checkLogFile(rel, text, errors) {
  const lines = text.split('\n');
  for (let i = 0; i < lines.length; i += 1) {
    const match = /^##\s+(.*?)\s*$/.exec(lines[i]);
    if (!match) continue;
    if (!isIsoDateHeading(match[1])) {
      errors.push({
        file: rel,
        code: 'log_heading_not_iso',
        message: `log.md date heading ${JSON.stringify(match[1])} (line ${i + 1}) is not ISO 8601 (OKF §7 MUST)`,
      });
    }
  }
}

function readPath(data, keyPath) {
  let value = data;
  for (const key of keyPath) {
    if (!value || typeof value !== 'object') return undefined;
    value = value[key];
  }
  return value;
}

/** Resolve a bundle-relative link target inside the bundle; null when it
 *  escapes (never touch the filesystem outside docs/knowledge/ — D23). */
function resolveInsideBundle(dir, target) {
  if (typeof target !== 'string' || target === '') return null;
  const resolved = path.resolve(dir, target);
  const prefix = `${path.resolve(dir)}${path.sep}`;
  if (!resolved.startsWith(prefix)) return null;
  return resolved;
}

/**
 * Two-level check over the whole bundle (D4/D13). Read-only; walks ONLY
 * docs/knowledge/ (D23); a missing or empty bundle is OK.
 *
 * Returns { okf: {errors}, profile: {warnings}, counts, ok, strict } where
 * each finding is {file, code, message}; ok is false on any OKF error, and —
 * under strict — on any finding at all.
 */
export function checkBundle(root, { strict = false } = {}) {
  const dir = bundleDir(root);
  const errors = [];
  const warnings = [];
  // G14 LAYER 3 (cell f3-3): profile findings that FAIL the chain on their
  // own, with no --strict. The chain runs `knowledge check` non-strict by
  // design (D13), so a "backstop" living in `warnings` never blocked anything.
  const profileErrors = [];
  const files = listBundleMarkdown(dir);
  const parsedConcepts = [];
  let conceptCount = 0;

  for (const rel of files) {
    const base = rel.includes('/') ? rel.slice(rel.lastIndexOf('/') + 1) : rel;
    let text;
    try {
      text = fs.readFileSync(path.join(dir, rel), 'utf8');
    } catch (error) {
      errors.push({ file: rel, code: 'unreadable', message: `could not read file: ${error.message}` });
      continue;
    }
    if (RESERVED_BASENAMES.has(base)) {
      if (base === 'index.md') checkIndexFile(rel, text, errors);
      else checkLogFile(rel, text, errors);
      continue;
    }

    conceptCount += 1;
    const parsed = parseFrontmatter(text);
    if (!parsed.present) {
      errors.push({
        file: rel,
        code: 'missing_frontmatter',
        message: 'a non-reserved .md inside the bundle is a concept and must carry frontmatter (D23; OKF §4)',
      });
      continue;
    }
    if (!parsed.ok) {
      errors.push({
        file: rel,
        code: 'unparseable_frontmatter',
        message: `frontmatter is unparseable — ${parsed.error.code}: ${parsed.error.message} (line ${parsed.error.line})`,
      });
      continue;
    }

    const data = parsed.data;
    if (typeof data.type !== 'string' || data.type.trim() === '') {
      errors.push({
        file: rel,
        code: 'empty_type',
        message: 'type is required and must be a non-empty string (OKF §4.1 MUST)',
      });
    } else if (!CONCEPT_TYPES.includes(data.type)) {
      warnings.push({
        file: rel,
        code: 'unknown_type',
        message: `type "${data.type}" is outside the profile's nine types (D18); OKF consumers tolerate it, bee flags it`,
      });
    }

    for (const keyPath of PROFILE_REQUIRED) {
      const value = readPath(data, keyPath);
      if (typeof value !== 'string' || value.trim() === '') {
        warnings.push({
          file: rel,
          code: 'missing_profile_field',
          message: `profile-required field "${keyPath.join('.')}" is missing or empty (D10: never invented — author it)`,
        });
      }
    }

    // Advisor round-trip guard (digest s1 items 1-2): parse -> re-emit ->
    // byte-compare. A mismatch means the file was authored outside the
    // canonical emitted form; the data above is still exact, but the file
    // should be normalized before it is trusted as curated truth.
    let reEmitted = null;
    try {
      reEmitted = emitFrontmatter(data);
    } catch {
      reEmitted = null;
    }
    if (reEmitted !== parsed.block) {
      warnings.push({
        file: rel,
        code: 'not_canonical',
        message:
          'frontmatter parse→re-emit differs byte-wise from the file (hand-edited colon/#/CRLF/key-order outside the canonical emitted form) — normalize by re-emitting',
      });
    }

    parsedConcepts.push({ file: rel, data });
  }

  // ─── bundle-level profile checks (D31 uniqueness, D4 dangling targets) ────
  const byId = new Map();
  const byAuthority = new Map();
  for (const concept of parsedConcepts) {
    const bee = concept.data.bee && typeof concept.data.bee === 'object' ? concept.data.bee : {};
    if (typeof bee.id === 'string' && bee.id) {
      if (!byId.has(bee.id)) byId.set(bee.id, []);
      byId.get(bee.id).push(concept.file);
    }
    if ('authoritative_for' in bee) {
      const claim = bee.authoritative_for;
      if (typeof claim !== 'string' || claim.trim() === '') {
        // A claim bee cannot read is an owner the anti-fork gate cannot see —
        // f3-2 skipped it silently and an ARRAY-valued claim let a judge
        // author a second concept for an already-owned subject.
        profileErrors.push({
          file: concept.file,
          code: 'malformed_authoritative_for',
          message:
            `bee.authoritative_for must be one non-empty string (got ${claim === null ? 'null' : Array.isArray(claim) ? 'array' : typeof claim}) ` +
            '— a claim bee cannot read is an owner the anti-fork gate cannot see (D31)',
        });
      } else {
        // Grouped by the HARDENED subject skeleton, not the raw string: two
        // concepts whose claims differ only by punctuation, case or encoding
        // are one subject with two authorities, and exact-string grouping
        // would have called that bundle clean.
        const key = normalizeSubject(claim);
        if (!byAuthority.has(key)) byAuthority.set(key, []);
        byAuthority.get(key).push({ file: concept.file, claim });
      }
    }
  }
  for (const [id, holders] of byId) {
    if (holders.length > 1) {
      warnings.push({
        file: holders[0],
        code: 'duplicate_id',
        message: `bee.id "${id}" is claimed by ${holders.length} concepts (${holders.join(', ')}) — ids are globally unique (D31)`,
      });
    }
  }
  for (const [, holders] of byAuthority) {
    if (holders.length > 1) {
      const subjects = [...new Set(holders.map((h) => h.claim))].map((s) => `"${s}"`);
      profileErrors.push({
        file: holders[0].file,
        code: 'duplicate_authoritative_for',
        message:
          `bee.authoritative_for ${subjects.join(' / ')} ${subjects.length > 1 ? 'name one subject and are' : 'is'} ` +
          `claimed by ${holders.length} concepts (${holders.map((h) => h.file).join(', ')}) — one subject, one authority (D31). ` +
          'Two authorities on one subject both parse and both index, and no reader can tell which is true.',
      });
    }
  }
  for (const concept of parsedConcepts) {
    const bee = concept.data.bee && typeof concept.data.bee === 'object' ? concept.data.bee : {};
    if (Array.isArray(bee.required_context)) {
      for (const target of bee.required_context) {
        const resolved = typeof target === 'string' ? resolveInsideBundle(dir, target) : null;
        if (!resolved || !fs.existsSync(resolved)) {
          warnings.push({
            file: concept.file,
            code: 'dangling_required_context',
            message: `required_context target "${String(target)}" does not resolve inside the bundle (D19: bundle-relative paths)`,
          });
        }
      }
    }
    if (typeof bee.supersedes === 'string' && bee.supersedes && !byId.has(bee.supersedes)) {
      warnings.push({
        file: concept.file,
        code: 'dangling_supersedes',
        message: `supersedes target id "${bee.supersedes}" matches no concept's bee.id in the bundle`,
      });
    }
  }

  const counts = {
    files: files.length,
    concepts: conceptCount,
    errors: errors.length,
    profile_errors: profileErrors.length,
    warnings: warnings.length,
  };
  // A profile ERROR fails the chain on its own — that is what "promoted from a
  // warning" means (G14 layer 3). Warnings keep their D13 semantics exactly:
  // reported always, failing only under --strict.
  const ok = errors.length === 0 && profileErrors.length === 0 && (!strict || warnings.length === 0);
  return { okf: { errors }, profile: { errors: profileErrors, warnings }, counts, ok, strict };
}

// ─── concept inventory (shared by list and index; read-only, D23) ───────────

/**
 * Every concept in the bundle as { path, data }, path-sorted (bundle-relative,
 * '/' separators from listBundleMarkdown). Robustness over judgment: a concept
 * whose frontmatter is missing or unparseable still appears — with data {} —
 * so list/index never hide a file; grading those files is check's job (D4),
 * not this inventory's.
 */
export function collectConcepts(root) {
  const dir = bundleDir(root);
  const concepts = [];
  for (const rel of listBundleMarkdown(dir)) {
    const base = rel.includes('/') ? rel.slice(rel.lastIndexOf('/') + 1) : rel;
    if (RESERVED_BASENAMES.has(base)) continue;
    let data = {};
    try {
      const parsed = parseFrontmatter(fs.readFileSync(path.join(dir, rel), 'utf8'));
      if (parsed.ok && parsed.present) data = parsed.data;
    } catch {
      // unreadable file: keep the row with empty data (check reports it)
    }
    concepts.push({ path: rel, data });
  }
  return concepts;
}

// ─── bundle mode (G8): the ONE "does this repo have a bundle?" predicate ────

/**
 * bundleMode(root) — TRUE only when `docs/knowledge/` exists AND at least one
 * concept in it actually parses. This is the single predicate every
 * bundle-first path routes on (G8); no caller may re-invent it with a bare
 * `path.join` / `existsSync`, and no skill may re-state it in prose.
 *
 * WHY "at least one concept parses" and not "the directory exists": a host
 * repo where a stray `.gitkeep` creates `docs/knowledge/` would otherwise flip
 * into bundle mode with an EMPTY bundle — new knowledge written as concepts
 * nobody reads while `docs/specs/` quietly stops updating. It ships working
 * and rots in one release with no error (advisor-digest-f3 finding 1). A
 * directory is not a bundle.
 *
 * "Parses" means the strict OKF sense: frontmatter present, accepted by the
 * D12 parser, and carrying the required `type` (OKF §4.1). Reserved basenames
 * (`index.md`, `log.md`) are never concepts and never count. Never throws —
 * a missing root, an unreadable tree, or a FILE sitting where the bundle
 * directory should be all read as `false`, which is the compatibility-safe
 * direction.
 */
export function bundleMode(root) {
  const dir = bundleDir(root);
  let stat;
  try {
    stat = fs.statSync(dir);
  } catch {
    return false;
  }
  if (!stat.isDirectory()) return false;
  for (const rel of listBundleMarkdown(dir)) {
    const base = rel.includes('/') ? rel.slice(rel.lastIndexOf('/') + 1) : rel;
    if (RESERVED_BASENAMES.has(base)) continue;
    try {
      const parsed = parseFrontmatter(fs.readFileSync(path.join(dir, rel), 'utf8'));
      if (parsed.ok && parsed.present && typeof parsed.data?.type === 'string' && parsed.data.type) {
        return true;
      }
    } catch {
      // unreadable/unparseable: not a concept, keep looking
    }
  }
  return false;
}

// ─── scribing target (G3/G9): where a settled truth is written ──────────────

// ─── G14 LAYER 1: harden the match (cell f3-3) ─────────────────────────────
//
// f3-2 compared subjects with trim + whitespace-collapse + lowercase, and an
// independent judge walked past the gate twice on encoding alone: a trailing
// period ('billing: refunds and reversals.') and a Cyrillic-e homoglyph both
// bought a NEW concept sitting beside the owner. Encoding must never be able
// to buy a fork, so subject identity is a SKELETON, not a string:
//
//   NFKC  -> folds fullwidth, ligature, and math-alphanumeric forms
//   lowercase + combining-mark strip -> case and diacritics are not identity
//   confusable fold -> cross-script look-alikes (NFKC does NOT do this: a
//                      Cyrillic 'е' U+0435 and a Latin 'e' stay distinct
//                      codepoints forever, which is exactly the defeat)
//   punctuation -> separator, whitespace collapsed, ends trimmed
//
// What this can NEVER catch is a genuine word-order paraphrase ('refunds and
// reversals' vs 'reversals and refunds') — a different subject, not a
// different encoding of one. That residual gap is layer 3's job, not a
// pretence made here.

/** UTS #39 skeleton fold, bounded to the look-alikes that collide with ASCII. */
const CONFUSABLE_FOLD = new Map(
  Object.entries({
    // Cyrillic -> Latin
    а: 'a', в: 'b', е: 'e', ё: 'e', з: '3', к: 'k', м: 'm', н: 'h', о: 'o',
    р: 'p', с: 'c', т: 't', у: 'y', х: 'x', ѕ: 's', і: 'i', ї: 'i', ј: 'j',
    ԁ: 'd', ԛ: 'q', ԝ: 'w', ѵ: 'v', ӏ: 'l', ѡ: 'w', ғ: 'f',
    // Greek -> Latin
    α: 'a', β: 'b', γ: 'y', ε: 'e', ζ: 'z', η: 'n', ι: 'i', κ: 'k', ν: 'v',
    ο: 'o', ρ: 'p', τ: 't', υ: 'u', χ: 'x', ϲ: 'c', ϳ: 'j', ϱ: 'p',
  }),
);

/** Encoding-only fold: NFKC + lowercase + de-accent + confusables. Keeps punctuation. */
function foldEncoding(text) {
  const bare = String(text ?? '')
    .normalize('NFKC')
    .toLowerCase()
    .normalize('NFD')
    .replace(/\p{M}+/gu, '');
  let folded = '';
  for (const ch of bare) folded += CONFUSABLE_FOLD.get(ch) ?? ch;
  return folded;
}

/**
 * Subject identity for ownership. Two subjects that a human reads as the same
 * thing must normalize to the same key — case, surrounding and internal
 * whitespace, leading/trailing punctuation, and encoding are all NOT identity.
 * Returns '' for a subject that carries no letters or digits at all (null,
 * undefined, '', '   ', '...'), which is the signal LAYER 2 refuses on.
 */
function normalizeSubject(subject) {
  return foldEncoding(subject)
    .replace(/[^\p{L}\p{N}]+/gu, ' ')
    .trim();
}

/** Concept filename slug derived from a subject (kebab, bounded). */
function subjectSlug(subject) {
  const slug = foldEncoding(subject)
    .replace(/^[a-z0-9-]+\s*:\s*/, '') // drop the "<area>: " prefix — the area is the directory
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 60)
    .replace(/-+$/g, '');
  return slug || 'overview';
}

/**
 * scribingTarget(root, {area, subject, intent}) — the scribing decision tree,
 * executable instead of merely written down. It derives its mode from
 * `bundleMode` and NOTHING else (G8).
 *
 * Returns exactly these seven keys, in both modes:
 *   { bundle_mode, action, area, subject, path, owner, regenerate_index }
 *
 * FALLBACK (`bundle_mode: false`) — an un-migrated host repo. `action` is
 * `update-spec` when `docs/specs/<area>.md` already exists and `create-spec`
 * when it does not; `path` is always that one file. This is today's "one area
 * = one file, forever" rule unchanged, and the result says NOTHING else: no
 * deprecation notice, no nag, no extra field (G1). An old project must not be
 * able to tell this release happened.
 *
 * BUNDLE MODE (`bundle_mode: true`) — three paths, gated on
 * `bee.authoritative_for` so the anti-fork mechanism "one area = one file"
 * used to provide survives the split into concepts (G9):
 *   (a) the subject is already owned  -> `update-concept` at the OWNING file,
 *       bundle-wide (an owner in another area still wins over the requested
 *       area — two concepts claiming one subject both parse, both index, and
 *       no reader can tell which is true);
 *   (b) a new subject in an existing area -> `new-concept` at
 *       `areas/<area>/<subject-slug>.md`, index regenerated. If that filename
 *       already exists it is an `update-concept` on it — a "new" path is never
 *       handed back for a file that is already there, so no `-v2` can appear;
 *   (c) a brand-new area -> `new-area` at `areas/<area>/overview.md`.
 *
 * `intent: 'new-concept'` is the caller asserting "this is a new subject". If
 * the subject is in fact owned, the answer is `fork_denied` with `path: null`
 * and the owner named — a refusal, never a second file.
 *
 * THE ANTI-FORK GATE HAS THREE LAYERS (G14, cell f3-3), because exact string
 * matching on free text can never be sufficient on its own:
 *   1. the match is a SKELETON, not a string (see normalizeSubject) — encoding
 *      alone can never buy a fork;
 *   2. malformed input fails CLOSED here — a non-string/blank
 *      `bee.authoritative_for` anywhere in the bundle THROWS naming the file
 *      (never a silent skip); a blank subject with `intent: 'new-concept'` is
 *      `subject_required` with `path: null` (never routed to overview.md); two
 *      concepts claiming one subject is `duplicate_authority` with `path: null`
 *      and every claimant listed on `owner.conflicts` (never first-wins);
 *   3. `duplicate_authoritative_for` is a chain-FAILING finding in
 *      `checkBundle` — the backstop for the word-order paraphrase layer 1
 *      structurally cannot catch.
 * All three are BUNDLE-MODE ONLY. The fallback is byte-identical to f3-2 (G1):
 * an un-migrated host repo must not be able to tell this release happened.
 */
export function scribingTarget(root, { area, subject = null, intent = 'auto' } = {}) {
  const areaSlug = String(area ?? '').trim();
  if (!areaSlug) throw new Error('scribingTarget: "area" is required.');
  const base = {
    bundle_mode: false,
    action: 'update-spec',
    area: areaSlug,
    subject: subject === null || subject === undefined ? null : String(subject),
    path: null,
    owner: null,
    regenerate_index: false,
  };

  if (!bundleMode(root)) {
    // G13: docs/specs/ is a PRODUCT doc tree — the existence probe resolves
    // against the product root (= the bee root for every ordinary repo; the
    // nested product repo under the divorce topology, #14). The returned path
    // stays product-root-relative, exactly as before.
    const rel = `docs/specs/${areaSlug}.md`;
    return {
      ...base,
      action: fs.existsSync(path.join(resolveProductRoot(root), 'docs', 'specs', `${areaSlug}.md`))
        ? 'update-spec'
        : 'create-spec',
      path: rel,
    };
  }

  const concepts = collectConcepts(root);
  const wanted = normalizeSubject(subject);

  // ─── G14 LAYER 2a: a malformed authority claim is an ERROR, never a skip ──
  // f3-2 did `typeof claim !== 'string' -> continue`, so an ARRAY-valued
  // `authoritative_for` made a real owner INVISIBLE to the gate and the judge
  // authored a second concept for an already-owned subject. Every claim in the
  // bundle is validated on every call — including when the requested subject
  // is empty — because an unreadable claim is an owner nobody can see, and
  // failing closed is the only safe direction.
  const owners = [];
  for (const concept of concepts) {
    const bee = concept.data?.bee;
    if (!bee || typeof bee !== 'object' || !('authoritative_for' in bee)) continue;
    const claim = bee.authoritative_for;
    if (typeof claim !== 'string' || claim.trim() === '') {
      throw new Error(
        `scribingTarget: docs/knowledge/${concept.path} has a malformed bee.authoritative_for ` +
          `(${claim === null ? 'null' : Array.isArray(claim) ? 'array' : typeof claim}) — an authority claim ` +
          'must be one non-empty string. Fix the concept: a claim bee cannot read is an owner the anti-fork ' +
          'gate cannot see (D31).',
      );
    }
    if (wanted && normalizeSubject(claim) === wanted) {
      owners.push({
        id: typeof bee.id === 'string' ? bee.id : null,
        path: `docs/knowledge/${concept.path}`,
        authoritative_for: claim,
      });
    }
  }

  // ─── G14 LAYER 2b: "author a new concept" with NO subject is REFUSED ──────
  // f3-2 gated the ownership walk behind `if (wanted)`, so an empty, blank,
  // null or undefined subject skipped the gate ENTIRELY and the request was
  // routed to areas/<area>/overview.md — with intent 'new-concept' the caller
  // is asserting a new subject exists, and no subject is not a new subject.
  // The documented intents keep failing SAFE (an empty subject on 'auto'
  // still resolves to today's in-place overview update).
  if (!wanted && intent === 'new-concept') {
    return { ...base, bundle_mode: true, action: 'subject_required', path: null };
  }

  // ─── G14 LAYER 2c: two claimants is an AMBIGUITY, never first-wins ────────
  // f3-2 `break`-ed on the first match, so with two pre-existing owners the
  // answer depended on walk order and said nothing. If no reader can tell
  // which file is true, neither can scribing: refuse, naming every claimant.
  // (The chain-failing `duplicate_authoritative_for` finding — layer 3 — is
  // what stops the bundle reaching this state in the first place.)
  if (owners.length > 1) {
    return {
      ...base,
      bundle_mode: true,
      action: 'duplicate_authority',
      path: null,
      owner: { ...owners[0], conflicts: owners.map((o) => o.path).sort() },
    };
  }

  const owner = owners[0] ?? null;

  if (owner) {
    if (intent === 'new-concept') {
      return { ...base, bundle_mode: true, action: 'fork_denied', path: null, owner };
    }
    return { ...base, bundle_mode: true, action: 'update-concept', path: owner.path, owner };
  }

  const areaDir = path.join(bundleDir(root), 'areas', areaSlug);
  const areaKnown =
    fs.existsSync(areaDir) ||
    concepts.some((concept) => {
      const areas = concept.data?.bee?.areas;
      return Array.isArray(areas) && areas.includes(areaSlug);
    });

  if (!areaKnown) {
    return {
      ...base,
      bundle_mode: true,
      action: 'new-area',
      path: `docs/knowledge/areas/${areaSlug}/overview.md`,
      regenerate_index: true,
    };
  }

  const rel = `areas/${areaSlug}/${subjectSlug(subject)}.md`;
  const exists = fs.existsSync(path.join(bundleDir(root), rel));
  return {
    ...base,
    bundle_mode: true,
    action: exists ? 'update-concept' : 'new-concept',
    path: `docs/knowledge/${rel}`,
    regenerate_index: !exists,
  };
}

// ─── list (D15): one row per concept, filters, never content ────────────────

/**
 * listConcepts(root, {type, lifecycle, area}) — D15: rows of
 * {path, id, type, lifecycle, title}, path-sorted, NEVER file content.
 * Filters are exact matches; --area matches membership in bee.areas.
 */
export function listConcepts(root, { type = null, lifecycle = null, area = null } = {}) {
  const rows = [];
  for (const concept of collectConcepts(root)) {
    const data = concept.data;
    const bee = data.bee && typeof data.bee === 'object' ? data.bee : {};
    const row = {
      path: concept.path,
      id: typeof bee.id === 'string' && bee.id ? bee.id : null,
      type: typeof data.type === 'string' && data.type ? data.type : null,
      lifecycle: typeof bee.lifecycle === 'string' && bee.lifecycle ? bee.lifecycle : null,
      title: typeof data.title === 'string' && data.title ? data.title : null,
    };
    if (type !== null && row.type !== type) continue;
    if (lifecycle !== null && row.lifecycle !== lifecycle) continue;
    if (area !== null) {
      const areas = Array.isArray(bee.areas) ? bee.areas : [];
      if (!areas.includes(area)) continue;
    }
    rows.push(row);
  }
  return rows;
}

// ─── index (D21): per-level generated indexes, byte-identical ───────────────

// Same idiom as decisions.mjs DECISION_INDEX_HEADER: an HTML comment (never
// frontmatter — frontmatter in a non-root index.md is an OKF error, D4), and
// deliberately NO generation timestamp or any other wall-clock value: the
// must-have is "two consecutive renders over the same bundle are
// byte-identical", so every generated index is a pure function of the
// bundle's own contents.
const KNOWLEDGE_INDEX_HEADER = [
  '<!--',
  'GENERATED FILE — do not hand-edit.',
  'Rendered by `bee knowledge index` from concept frontmatter inside docs/knowledge/ (okf-foundation D21).',
  'Regenerate: `bee knowledge index`. Check freshness: `bee knowledge index --check`.',
  'Deterministic: byte-identical for the same bundle contents — path-sorted entries, LF endings,',
  'never a generation timestamp or any other wall-clock value.',
  '-->',
].join('\n');

function conceptEntryLine(concept, fromDir) {
  const target = fromDir === '' ? concept.path : concept.path.slice(fromDir.length + 1);
  const base = concept.path.slice(concept.path.lastIndexOf('/') + 1);
  const title = typeof concept.data.title === 'string' && concept.data.title ? concept.data.title : base;
  const description = typeof concept.data.description === 'string' && concept.data.description ? concept.data.description : null;
  return `- [${title}](${target})${description ? ` — ${description}` : ''}`;
}

/**
 * Compute the full generated-index set in memory: [{rel, content}] with rel
 * bundle-relative ('/' separators), path-sorted. One index per directory
 * level whose subtree contains at least one concept, plus the root index
 * always (sole carrier of okf_version — OKF §9/D4; every other index carries
 * NO frontmatter). The root additionally carries the '## Critical patterns'
 * section over every bee.critical: true concept (D21 — the generated
 * replacement for a hand-maintained critical-patterns list).
 */
export function computeIndexFiles(root) {
  const concepts = collectConcepts(root);

  // Every directory that owns an index: root always, plus each ancestor
  // directory of a concept path.
  const indexDirs = new Set(['']);
  for (const concept of concepts) {
    const segments = concept.path.split('/');
    for (let i = 1; i < segments.length; i += 1) {
      indexDirs.add(segments.slice(0, i).join('/'));
    }
  }

  const files = [];
  for (const dir of [...indexDirs].sort()) {
    const directConcepts = concepts.filter((c) => {
      const parent = c.path.includes('/') ? c.path.slice(0, c.path.lastIndexOf('/')) : '';
      return parent === dir;
    });
    const childDirs = [...indexDirs]
      .filter((d) => d !== '' && (dir === '' ? !d.includes('/') : d.startsWith(`${dir}/`) && !d.slice(dir.length + 1).includes('/')))
      .sort();

    const sections = [];
    if (directConcepts.length > 0) {
      sections.push(['## Concepts', '', ...directConcepts.map((c) => conceptEntryLine(c, dir))].join('\n'));
    }
    if (childDirs.length > 0) {
      const bullets = childDirs.map((child) => {
        const name = dir === '' ? child : child.slice(dir.length + 1);
        const count = concepts.filter((c) => c.path.startsWith(`${child}/`)).length;
        return `- [${name}/](${name}/index.md) — ${count} concept(s)`;
      });
      sections.push(['## Sections', '', ...bullets].join('\n'));
    }
    if (dir === '') {
      const critical = concepts.filter((c) => {
        const bee = c.data.bee && typeof c.data.bee === 'object' ? c.data.bee : {};
        return bee.critical === true;
      });
      sections.push(
        ['## Critical patterns', '', ...(critical.length > 0 ? critical.map((c) => conceptEntryLine(c, '')) : ['None.'])].join('\n'),
      );
    }

    const heading = dir === '' ? '# Knowledge Bundle' : `# ${dir}/`;
    const body = [heading, ...sections].join('\n\n');
    const frontmatter = dir === '' ? emitFrontmatter({ okf_version: OKF_VERSION }) : '';
    files.push({
      rel: dir === '' ? 'index.md' : `${dir}/index.md`,
      content: `${frontmatter}${KNOWLEDGE_INDEX_HEADER}\n\n${body}\n`,
    });
  }
  return files;
}

/**
 * knowledgeIndexDrift(root) — read-only --check half of the decisions.mjs
 * render --check idiom: re-render every expected index in memory and
 * byte-compare against disk (a missing file counts as drift). Returns
 * { stale: [repo-relative paths], checked }. Never writes, never throws.
 */
export function knowledgeIndexDrift(root) {
  const dir = bundleDir(root);
  const expected = computeIndexFiles(root);
  const stale = [];
  for (const file of expected) {
    let onDisk = null;
    try {
      onDisk = fs.readFileSync(path.join(dir, file.rel), 'utf8');
    } catch {
      onDisk = null;
    }
    if (onDisk !== file.content) stale.push(`docs/knowledge/${file.rel}`);
  }
  return { stale, checked: expected.length };
}

/**
 * renderKnowledgeIndexes(root) — write the full generated-index set to disk.
 * The ONLY write path in this module, and it touches ONLY generated index.md
 * files inside docs/knowledge/ (D2/D23). Returns { written, count }.
 */
export function renderKnowledgeIndexes(root) {
  const dir = bundleDir(root);
  const written = [];
  for (const file of computeIndexFiles(root)) {
    const abs = path.join(dir, file.rel);
    fs.mkdirSync(path.dirname(abs), { recursive: true });
    fs.writeFileSync(abs, file.content, 'utf8');
    written.push(`docs/knowledge/${file.rel}`);
  }
  return { written, count: written.length };
}

// ─── context (D27): the budget-aware manifest — paths, never content ───────

/**
 * The estimator's NAME, carried in every manifest (D27/D12). Bee vendors no
 * tokenizer, so the budget is spent in bytes/4 and the output declares that
 * rather than dressing an estimate as a token count.
 */
export const CONTEXT_ESTIMATOR = 'bytes/4';

/** bytes/4, rounded up — the only sizing arithmetic in this module. */
export function estimateTokens(bytes) {
  return Math.ceil(bytes / 4);
}

// ─── i54-closeout D3: lane-scaled `knowledge context` budget presets ────────
//
// The single source both consumers read from — the CLI's `--lane` shorthand
// (bee.mjs, resolved before the generic validate() layer runs) and the
// session preamble's mode-aware recommended command (inject.mjs) — so the
// four numbers exist exactly once. `standard` intentionally matches the
// long-standing bare default (20000): a bare invocation with neither --lane
// nor --budget stays byte-identical to before this cell (must-have), and a
// `--lane standard` call now resolves to the same number that was always the
// unlabelled default. Budget-cut semantics (prefix cut, critical-patterns IDF
// exception) are untouched by this table — it only picks the NUMBER fed in.
export const KNOWLEDGE_CONTEXT_LANE_BUDGETS = Object.freeze({
  tiny: 8000,
  small: 12000,
  standard: 20000,
  'high-risk': 30000,
});

/** The bare-invocation default — unchanged since before this cell. */
export const KNOWLEDGE_CONTEXT_DEFAULT_BUDGET = KNOWLEDGE_CONTEXT_LANE_BUDGETS.standard;

function beeOf(data) {
  return data && data.bee && typeof data.bee === 'object' && !Array.isArray(data.bee) ? data.bee : {};
}

function dirOf(rel) {
  return rel.includes('/') ? rel.slice(0, rel.lastIndexOf('/')) : '';
}

/** A bundle-relative required_context target (D19) normalized back to a
 *  bundle-relative concept path, or null when it would escape the bundle
 *  (D23 — a link out of docs/knowledge/ is never followed). */
function normalizeBundleTarget(dir, target) {
  const resolved = resolveInsideBundle(dir, target);
  if (!resolved) return null;
  const rel = path.relative(path.resolve(dir), resolved).split(path.sep).join('/');
  return rel === '' ? null : rel;
}

// ─── G5: relevance ranking over the critical concepts ───────────────────────
//
// WHY THIS SHAPE, IN NUMBERS. The naive design — rank the criticals by tag
// (and area) overlap with the work item — was measured against the live bundle
// before a line of it was written, on the okf-migration-f2 work item and the 49
// bee.critical concepts that exist:
//
//   signal                                     AUC    zero scores   distinct
//   tag overlap alone                          0.550  48 of 49       2
//   bee.areas overlap alone                    0.500  49 of 49       1
//   recency (the pattern's own date)           0.533   0 of 49      12
//   title+description overlap with the item    0.674  11 of 49      35
//   IDF-weighted body overlap, work item text  0.751   0 of 49      49
//   THIS: two-field IDF coverage (below)       0.805   0 of 49      49
//
// (AUC = P(a hand-labelled-relevant critical outranks a hand-labelled-
// irrelevant one); 0.5 is a coin flip. The 10 relevant / 39 irrelevant labels
// were assigned by reading each concept against the work item's outcome and
// scope, and are recorded in the f3-1 cell trace.)
//
// Tag overlap is therefore DISQUALIFIED as the ranking signal: it leaves 48 of
// 49 tied at zero, which is a path sort wearing a relevance label. It survives
// here only as a small additive bonus, for the work items that do carry
// meaningful tags. Two more measured facts shaped the rest:
//   * Widening the query with the required_context bodies HURT (0.751 -> 0.615)
//     — long shared prose dilutes the work item's own distinctive vocabulary.
//     The query is the work item's frontmatter and body, and nothing else.
//   * Normalising by the concept's own vocabulary (a COVERAGE fraction) beats
//     an unnormalised sum, which just rewards long patterns.
//
// The signal, stated plainly: what FRACTION of this concept's own distinctive
// vocabulary does the work item talk about? IDF is computed over the ranked
// population itself, so the corpus defines "distinctive" and no external word
// list ships. Meta (title/description/tags) and body are scored as separate
// fields because a bee pattern's title states its thesis while its body is
// evidence.

/** Pinned ranking constants (G5/G11). Exported so tests pin the numbers rather
 *  than re-deriving them, and so a host repo can read what it is getting. */
export const CRITICAL_RELEVANCE = Object.freeze({
  /** How many ranked criticals survive the relevance cut. Measured on the live
   *  bundle: of the 8 labelled-relevant criticals that reach the block, KEEP=20
   *  retains 6 while dropping 27 of 47 patterns; KEEP=12 would retain only 4
   *  for a further ~1.7k tokens — ranks 16-20 are the densest relevant band
   *  after the top 5, so the cut is placed after them. */
  KEEP: 20,
  /** The highest-scoring criticals that are never evicted by the budget: their
   *  cost is RESERVED out of the budget before the prefix walk spends it. Small
   *  on purpose — a floor is a guarantee against eviction, not a second cut. */
  FLOOR: 3,
  META_WEIGHT: 0.25,
  BODY_WEIGHT: 1,
  TAG_WEIGHT: 0.05,
  AREA_WEIGHT: 0.05,
  /** The zero-signal guard is about a corpus that has outgrown its signal, not
   *  about a two-concept fixture where there is nothing to rank. Below this
   *  population the count is still REPORTED, never enforced. */
  ZERO_SIGNAL_MIN_POPULATION: 10,
  /** More than half a real population scoring zero is not a ranking. */
  ZERO_SIGNAL_MAX_RATIO: 0.5,
});

/** Closed-class English words carry no topic. Kept as a literal so the ranking
 *  ships no dependency and no downloadable word list (D23). */
const RELEVANCE_STOPWORDS = new Set(
  ('a an the and or but if then else for of to in on at by is are was were be been being it its this that these those with without from as not no never always every each any all some one two three you your we our they their he she i me my do does did done can could should would may might must will shall have has had so than which who whom what when where why how more most less least very just only also into out up down over under again further once here there both few other own same too s t don now').split(' '),
);

/** Topic tokens: lowercase, alphanumeric runs, >2 chars, stopped, and crudely
 *  singularised so "rows" and "row" are the same term. Deliberately not a
 *  stemmer — a stemmer is a dependency and a source of surprise. */
function relevanceTokens(text) {
  const out = [];
  for (const raw of String(text || '').toLowerCase().split(/[^a-z0-9]+/)) {
    if (raw.length <= 2 || RELEVANCE_STOPWORDS.has(raw)) continue;
    out.push(raw.length > 4 && raw.endsWith('s') && !raw.endsWith('ss') ? raw.slice(0, -1) : raw);
  }
  return out;
}

/** The concept body, read once from the bundle (D23 — never outside it). */
function conceptBody(dir, rel) {
  try {
    const raw = fs.readFileSync(path.join(dir, rel), 'utf8');
    const parsed = parseFrontmatter(raw);
    return parsed.ok && parsed.present ? parsed.body || '' : raw;
  } catch {
    return '';
  }
}

function metaTextOf(concept) {
  const data = concept.data || {};
  const tags = Array.isArray(data.tags) ? data.tags.join(' ') : '';
  return `${data.title || ''} ${data.description || ''} ${tags}`;
}

/**
 * scoreCriticalRelevance(dir, criticals, workConcept) -> Map<path, number>.
 *
 * Every critical concept is scored — including one already selected higher up
 * the ranking — so `zero_signal_count` describes the whole population and no
 * concept can be dropped without a number attached to it (G11).
 */
function scoreCriticalRelevance(dir, criticals, workConcept) {
  const fields = new Map();
  const documentFrequency = new Map();
  for (const concept of criticals) {
    const meta = new Set(relevanceTokens(metaTextOf(concept)));
    const body = new Set(relevanceTokens(conceptBody(dir, concept.path)));
    fields.set(concept.path, { meta, body });
    for (const token of new Set([...meta, ...body])) {
      documentFrequency.set(token, (documentFrequency.get(token) || 0) + 1);
    }
  }
  const population = criticals.length;
  const idf = (token) => Math.log((population + 1) / ((documentFrequency.get(token) || 0) + 1)) + 1;

  const query = new Set(
    relevanceTokens(`${metaTextOf(workConcept)} ${conceptBody(dir, workConcept.path)}`),
  );
  const workBee = beeOf(workConcept.data);
  const workTags = new Set(
    (Array.isArray(workConcept.data.tags) ? workConcept.data.tags : [])
      .filter((tag) => typeof tag === 'string')
      .map((tag) => tag.toLowerCase()),
  );
  const workAreas = new Set(
    (Array.isArray(workBee.areas) ? workBee.areas : []).filter((area) => typeof area === 'string'),
  );

  /** The IDF-weighted fraction of a field's own vocabulary the query covers. */
  const coverage = (set) => {
    let hit = 0;
    let total = 0;
    for (const token of set) {
      const weight = idf(token);
      total += weight;
      if (query.has(token)) hit += weight;
    }
    return total === 0 ? 0 : hit / total;
  };

  const scores = new Map();
  for (const concept of criticals) {
    const bee = beeOf(concept.data);
    const tags = (Array.isArray(concept.data.tags) ? concept.data.tags : []).filter(
      (tag) => typeof tag === 'string' && workTags.has(tag.toLowerCase()),
    ).length;
    const areas = (Array.isArray(bee.areas) ? bee.areas : []).filter((area) => workAreas.has(area)).length;
    const field = fields.get(concept.path);
    const score =
      CRITICAL_RELEVANCE.TAG_WEIGHT * tags +
      CRITICAL_RELEVANCE.AREA_WEIGHT * areas +
      CRITICAL_RELEVANCE.META_WEIGHT * coverage(field.meta) +
      CRITICAL_RELEVANCE.BODY_WEIGHT * coverage(field.body);
    // Rounded at emit precision so the number an agent reads in `excluded` is
    // the number the ranking used — a displayed score that lost a tie it
    // appears to have won is its own silent defect.
    scores.set(concept.path, Number(score.toFixed(6)));
  }
  return scores;
}

/**
 * buildContextManifest(root, {work, budget}) — the D27 consumer.
 *
 * Resolves `work` to the bee.work-item concept whose bee.id matches, then
 * assembles an ORDERED manifest under the ranking law:
 *   1. the work item itself
 *   2. its bee.plan sibling in the same work/<id>/ directory, when present
 *   3. required_context, walked TRANSITIVELY in BFS depth order — an already
 *      selected path is skipped SILENTLY, so a cycle (A→B→A) is deduped and
 *      never an error, and a link that dangles or escapes the bundle is
 *      tolerated (OKF §5; `knowledge check` is what grades it, D4)
 *   4. the bee.critical: true concepts, RANKED BY RELEVANCE to the work item
 *      (G5) and cut to CRITICAL_RELEVANCE.KEEP — ties break by path, so the
 *      order is total and two runs are byte-identical
 *   5. every bee.decision concept whose bee.areas overlaps the work item's,
 *      path-sorted
 * and cuts the ranked list at `budget` estimated tokens.
 *
 * The cut is a PREFIX cut with ONE named exception: the first
 * CRITICAL_RELEVANCE.FLOOR criticals have their cost RESERVED out of what the
 * budget has left after rank 1, before the prefix walk spends it — so a
 * genuinely universal lesson is never evicted by a long required_context chain,
 * while the work item itself is never displaced by the floor. `budget` stays a
 * hard ceiling — total_est never exceeds it, and a zero budget still includes
 * nothing. Apart from the floor, the first entry that would overshoot ends the
 * manifest and it plus every lower-ranked entry is named in `truncated`:
 * skipping an overshooting entry to squeeze in a smaller lower-ranked one
 * would make the output stop meaning "the highest-ranked context that fits".
 *
 * The BFS is seeded with the work item AND its plan sibling: the plan is in
 * the manifest, so what the plan itself requires is required context too —
 * every reason still names its parent, so the provenance stays auditable.
 *
 * CONSERVATION (G11). Every bee.critical concept is accounted for exactly
 * once: in `entries` (with its score and rank in the reason), in `truncated`
 * (ranked, but the budget ran out), or in `excluded` as {path, score, reason}
 * (below the relevance cut). A pattern that would have prevented a bug is
 * never merely absent — the loud failure G5 replaces was 13k tokens of visible
 * noise, and the silent one it could have become is exactly what `excluded`
 * exists to prevent. `zero_signal_count` reports how many criticals scored
 * zero, and a population at or above ZERO_SIGNAL_MIN_POPULATION with more than
 * ZERO_SIGNAL_MAX_RATIO of it at zero FAILS the run: that is a path sort
 * wearing a relevance label, and shipping it green is the defect.
 *
 * Returns {work, decisions, budget, estimator, total_est, entries, truncated,
 * excluded, floor, critical_total, zero_signal_count} where each entry is
 * {path (repo-relative), bytes, est_tokens, reason} — NEVER file content
 * (D27). `decisions` is informational: the work item's own bee.decisions list,
 * read from its frontmatter, never from a .bee/ store (D2/D23). Read-only end
 * to end; throws a typed Error the CLI surfaces as a non-zero exit when `work`
 * resolves to nothing or when the zero-signal guard trips.
 */
export function buildContextManifest(root, { work, budget } = {}) {
  const workId = typeof work === 'string' ? work.trim() : '';
  if (!workId) {
    throw new Error('knowledge context: missing_work — --work <id> is required (D27).');
  }
  const budgetTokens = Number(budget);
  if (!Number.isFinite(budgetTokens) || budgetTokens < 0) {
    throw new Error(
      `knowledge context: bad_budget — --budget must be a non-negative token count, got ${JSON.stringify(budget)} (D27).`,
    );
  }

  const dir = bundleDir(root);
  const concepts = collectConcepts(root); // ONE inventory path, shared with list/index
  const byPath = new Map(concepts.map((concept) => [concept.path, concept]));

  const workConcept = concepts.find(
    (concept) => concept.data.type === 'bee.work-item' && beeOf(concept.data).id === workId,
  );
  if (!workConcept) {
    throw new Error(
      `knowledge context: unknown_work — no bee.work-item concept in docs/knowledge/ carries bee.id "${workId}" (D27).`,
    );
  }

  const ranked = [];
  const selected = new Set();
  const select = (rel, reason) => {
    if (selected.has(rel) || !byPath.has(rel)) return false;
    selected.add(rel);
    ranked.push({ rel, reason });
    return true;
  };

  // (1) the work item
  select(workConcept.path, 'work item');

  // (2) the plan sibling in the same work/<id>/ directory
  const workDir = dirOf(workConcept.path);
  const planConcept = concepts.find(
    (concept) => concept.data.type === 'bee.plan' && dirOf(concept.path) === workDir,
  );
  if (planConcept) select(planConcept.path, `plan sibling in ${workDir}/`);

  // (3) required_context, transitive, BFS depth order, cycles deduped silently
  const queue = ranked.map((entry) => ({ rel: entry.rel, depth: 0 }));
  while (queue.length > 0) {
    const node = queue.shift();
    const targets = beeOf(byPath.get(node.rel).data).required_context;
    if (!Array.isArray(targets)) continue;
    for (const target of targets) {
      if (typeof target !== 'string') continue;
      const rel = normalizeBundleTarget(dir, target);
      if (!rel || !byPath.has(rel) || selected.has(rel)) continue;
      select(rel, `required_context depth ${node.depth + 1} via ${node.rel}`);
      queue.push({ rel, depth: node.depth + 1 });
    }
  }

  // (4) the critical concepts, ranked by relevance and cut (G5/G11)
  const criticals = concepts.filter((concept) => beeOf(concept.data).critical === true);
  const relevance = scoreCriticalRelevance(dir, criticals, workConcept);
  const zeroSignalCount = criticals.filter((concept) => relevance.get(concept.path) === 0).length;
  if (
    criticals.length >= CRITICAL_RELEVANCE.ZERO_SIGNAL_MIN_POPULATION &&
    zeroSignalCount > criticals.length * CRITICAL_RELEVANCE.ZERO_SIGNAL_MAX_RATIO
  ) {
    throw new Error(
      `knowledge context: zero_signal — ${zeroSignalCount} of ${criticals.length} bee.critical concepts score 0 against work item "${workId}", ` +
        `above the pinned ${CRITICAL_RELEVANCE.ZERO_SIGNAL_MAX_RATIO} ratio. A ranking where most items tie at zero is a path sort wearing a ` +
        'relevance label — widen the work item\'s description/body, or fix the ranking, but do not ship this order (G11).',
    );
  }
  // Total order: score desc, then path asc. Ties never depend on readdir.
  const rankedCriticals = [...criticals].sort((a, b) => {
    const delta = relevance.get(b.path) - relevance.get(a.path);
    if (delta !== 0) return delta;
    return a.path < b.path ? -1 : a.path > b.path ? 1 : 0;
  });
  const excluded = [];
  const floorPaths = [];
  let keptCount = 0;
  for (const [index, concept] of rankedCriticals.entries()) {
    const rank = index + 1; // rank in the FULL critical population, not in the survivors
    const score = relevance.get(concept.path);
    if (selected.has(concept.path)) continue; // already in via required_context — never re-cut
    if (keptCount >= CRITICAL_RELEVANCE.KEEP) {
      excluded.push({
        path: `docs/knowledge/${concept.path}`,
        score,
        reason: `below the relevance cut — rank ${rank} of ${rankedCriticals.length}, keep ${CRITICAL_RELEVANCE.KEEP} (G5)`,
      });
      continue;
    }
    const isFloor = keptCount < CRITICAL_RELEVANCE.FLOOR;
    if (isFloor) floorPaths.push(`docs/knowledge/${concept.path}`);
    select(
      concept.path,
      `critical pattern (relevance ${score}, rank ${rank} of ${rankedCriticals.length}${isFloor ? ', floor' : ''})`,
    );
    keptCount += 1;
  }

  // (5) decisions whose areas overlap the work item's areas
  const workAreas = Array.isArray(beeOf(workConcept.data).areas)
    ? beeOf(workConcept.data).areas.filter((area) => typeof area === 'string')
    : [];
  for (const concept of concepts) {
    if (concept.data.type !== 'bee.decision') continue;
    const areas = Array.isArray(beeOf(concept.data).areas) ? beeOf(concept.data).areas : [];
    const overlap = areas.filter((area) => workAreas.includes(area));
    if (overlap.length === 0) continue;
    select(concept.path, `decision for area ${overlap.join(', ')}`);
  }

  const sized = ranked.map((item) => {
    const repoRel = `docs/knowledge/${item.rel}`;
    let bytes = 0;
    try {
      bytes = fs.statSync(path.join(dir, item.rel)).size;
    } catch {
      bytes = 0;
    }
    return { ...item, repoRel, bytes, est: estimateTokens(bytes), floor: floorPaths.includes(repoRel) };
  });

  // The floor's cost comes out of the budget FIRST — but only out of what is
  // left after rank 1. The floor exists so a long required_context chain, or a
  // pile of area decisions, cannot evict a universal lesson; it was never meant
  // to displace the work item itself, and a manifest without its work item is
  // not a manifest. Capping the reservation this way also keeps `budget` a hard
  // ceiling and keeps a zero budget including nothing.
  const floorCost = sized.filter((item) => item.floor).reduce((sum, item) => sum + item.est, 0);
  const rankOneCost = sized.length > 0 ? sized[0].est : 0;
  let reserve = Math.max(0, Math.min(floorCost, budgetTokens - rankOneCost));
  let available = budgetTokens - reserve;

  const entries = [];
  const truncated = [];
  let totalEst = 0;
  let cutting = false;
  for (const item of sized) {
    if (item.floor) {
      if (item.est > reserve) {
        truncated.push(item.repoRel);
        continue;
      }
      reserve -= item.est;
      totalEst += item.est;
      entries.push({ path: item.repoRel, bytes: item.bytes, est_tokens: item.est, reason: item.reason });
      continue;
    }
    if (cutting || item.est > available) {
      cutting = true;
      truncated.push(item.repoRel);
      continue;
    }
    available -= item.est;
    totalEst += item.est;
    entries.push({ path: item.repoRel, bytes: item.bytes, est_tokens: item.est, reason: item.reason });
  }

  const decisions = Array.isArray(beeOf(workConcept.data).decisions)
    ? beeOf(workConcept.data).decisions.filter((entry) => typeof entry === 'string')
    : [];

  // CONSERVATION (G11), asserted here rather than trusted: a critical that is
  // neither included, truncated nor excluded has been silently dropped, which
  // is the one failure this whole design exists to prevent.
  const accounted = new Set([
    ...entries.map((entry) => entry.path),
    ...truncated,
    ...excluded.map((item) => item.path),
  ]);
  const lost = criticals
    .map((concept) => `docs/knowledge/${concept.path}`)
    .filter((repoRel) => !accounted.has(repoRel));
  if (lost.length > 0) {
    throw new Error(
      `knowledge context: conservation — ${lost.length} bee.critical concept(s) were neither included, truncated nor excluded: ` +
        `${lost.join(', ')} (G11). This is a bug in the ranking, not a condition of the bundle.`,
    );
  }

  return {
    work: workId,
    decisions,
    budget: budgetTokens,
    estimator: CONTEXT_ESTIMATOR,
    total_est: totalEst,
    entries,
    truncated,
    excluded,
    floor: floorPaths,
    critical_total: criticals.length,
    zero_signal_count: zeroSignalCount,
  };
}

// ─── promote (D38): finished work PROPOSES knowledge; it never writes it ────
//
// The loop-closing verb. Everything below is READ-ONLY: the bundle (D23) plus
// the capped cell traces in .bee/cells/*.json — a read of the runtime store,
// which D2 permits explicitly while forbidding any write into it. Nothing in
// this section calls fs.writeFileSync, fs.mkdirSync, or fs.rmSync, and the
// returned proposal carries `writes: []` as the machine-readable statement of
// that contract.

/** The runtime cell store — READ-ONLY input to promote (D2). */
function cellsStoreDir(root) {
  return path.join(root, '.bee', 'cells');
}

/** Natural-order id compare so okf-10 sorts after okf-9, not after okf-1. */
function compareCellIds(a, b) {
  const split = (id) => id.split(/(\d+)/).filter((part) => part !== '');
  const left = split(a);
  const right = split(b);
  for (let i = 0; i < Math.max(left.length, right.length); i += 1) {
    const l = left[i];
    const r = right[i];
    if (l === undefined) return -1;
    if (r === undefined) return 1;
    const bothNumeric = /^\d+$/.test(l) && /^\d+$/.test(r);
    if (bothNumeric) {
      if (Number(l) !== Number(r)) return Number(l) - Number(r);
    } else if (l !== r) {
      return l < r ? -1 : 1;
    }
  }
  return 0;
}

/** Free text (a deviation, a failure signature, an outcome) flattened to one
 *  line so it can live in a frontmatter scalar or a bullet without breaking
 *  the emitted subset. Never rewritten — only whitespace-collapsed. */
function oneLine(text, limit = 0) {
  const flat = String(text).replace(/\s+/g, ' ').trim();
  return limit > 0 && flat.length > limit ? `${flat.slice(0, limit - 1)}…` : flat;
}

/** A trace deviation is either a plain string or a {type, description} record
 *  (both shapes exist in the store). Normalize to the recorded text — never
 *  paraphrased, so the proposal quotes the trace verbatim. */
function deviationText(entry) {
  if (typeof entry === 'string') return entry;
  if (entry && typeof entry === 'object') {
    if (typeof entry.description === 'string' && entry.description) {
      return typeof entry.type === 'string' && entry.type ? `${entry.type}: ${entry.description}` : entry.description;
    }
    return JSON.stringify(entry);
  }
  return String(entry);
}

/** The recorded verify evidence, reduced to one quotable line. The store keeps
 *  it as a JSON string in practice; anything unparseable is used verbatim. */
function verifySummary(trace) {
  const raw = trace && typeof trace.verification_evidence === 'string' ? trace.verification_evidence : '';
  if (!raw.trim()) return '';
  try {
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === 'object') {
      for (const key of ['verify_tail', 'verify_output', 'evidence', 'summary']) {
        if (typeof parsed[key] === 'string' && parsed[key].trim()) return oneLine(parsed[key], 200);
      }
    }
  } catch {
    // not JSON — fall through to the raw text
  }
  return oneLine(raw, 200);
}

/**
 * Read every CAPPED cell trace belonging to `feature` from .bee/cells/*.json,
 * natural-id-sorted. Read-only, tolerant: an unreadable or non-JSON file is
 * skipped rather than thrown on (a proposal is never the place a corrupt
 * runtime file surfaces), and the archive subdirectory is not descended into.
 */
export function readCappedCellTraces(root, feature) {
  const dir = cellsStoreDir(root);
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return [];
  }
  const cells = [];
  for (const entry of entries) {
    if (!entry.isFile() || !entry.name.endsWith('.json')) continue;
    let cell;
    try {
      cell = JSON.parse(fs.readFileSync(path.join(dir, entry.name), 'utf8'));
    } catch {
      continue;
    }
    if (!cell || typeof cell !== 'object') continue;
    if (cell.feature !== feature || cell.status !== 'capped') continue;
    const trace = cell.trace && typeof cell.trace === 'object' ? cell.trace : {};
    const deviations = (Array.isArray(trace.deviations) ? trace.deviations : []).map(deviationText).filter((text) => text.trim());
    const failureSignatures = [];
    for (const attempt of Array.isArray(trace.attempts) ? trace.attempts : []) {
      if (attempt && typeof attempt.failure_signature === 'string' && attempt.failure_signature.trim()) {
        failureSignatures.push(attempt.failure_signature);
      }
    }
    for (const verdict of Array.isArray(trace.semantic_judge) ? trace.semantic_judge : []) {
      if (verdict && typeof verdict.failure_signature === 'string' && verdict.failure_signature.trim()) {
        failureSignatures.push(verdict.failure_signature);
      }
    }
    cells.push({
      id: typeof cell.id === 'string' ? cell.id : entry.name.replace(/\.json$/, ''),
      title: typeof cell.title === 'string' ? cell.title : '',
      lane: typeof cell.lane === 'string' ? cell.lane : null,
      behavior_change: trace.behavior_change === true || (trace.behavior_change === undefined && cell.behavior_change === true),
      outcome: typeof trace.outcome === 'string' && trace.outcome.trim() ? trace.outcome : (typeof cell.title === 'string' ? cell.title : ''),
      files_changed: (Array.isArray(trace.files_changed) ? trace.files_changed : []).filter((file) => typeof file === 'string'),
      deviations,
      failure_signatures: failureSignatures,
      verify: typeof cell.verify === 'string' ? cell.verify : '',
      verify_summary: verifySummary(trace),
      capped_at: typeof trace.capped_at === 'string' ? trace.capped_at : null,
      trace_path: `.bee/cells/${typeof cell.id === 'string' ? cell.id : entry.name.replace(/\.json$/, '')}.json`,
    });
  }
  return cells.sort((a, b) => compareCellIds(a.id, b.id));
}

/** The date part of an ISO timestamp, or null. */
function isoDate(value) {
  return typeof value === 'string' && /^\d{4}-\d{2}-\d{2}/.test(value) ? value.slice(0, 10) : null;
}

/** Does a changed file touch a subject path? Exact match, or either path
 *  containing the other as a directory — nothing fuzzier, so every proposed
 *  bullet is traceable to the two paths that produced it. */
function touchesSubject(file, subject) {
  return file === subject || file.startsWith(`${subject}/`) || subject.startsWith(`${file}/`);
}

/**
 * buildPromotion(root, {work}) — the D38 proposal builder.
 *
 * Resolves `work` to the bee.work-item concept whose bee.id matches (the same
 * resolution `context` performs), mines the CAPPED cells of that feature from
 * .bee/cells/ and returns three proposals:
 *
 *   (a) delivery — a complete bee.delivery concept in canonical emitter form,
 *       ready to be saved as the work item's delivery.md sibling: what
 *       shipped (each cell's recorded outcome), how it was verified (each
 *       cell's recorded verify command and evidence), and every recorded
 *       deviation.
 *   (b) area_updates — for each area named in the work item's bee.areas, the
 *       capped behavior_change cells whose files_changed touch that area's
 *       subject (its concepts' own paths and their bee.sources), as candidate
 *       spec-sync bullets, each citing its cell id.
 *   (c) pattern_candidates — every capped cell whose trace carries a deviation
 *       or a failure signature, shaped as a candidate bee.pattern concept with
 *       bee.polarity pitfall and bee.lifecycle draft, quoting the trace.
 *
 * Every proposed line traces to a cell trace or to the work item; nothing is
 * invented (D10). Nothing is written (D2) — `writes` is always [].
 * Throws the typed unknown_work error when `work` resolves to nothing.
 */
export function buildPromotion(root, { work } = {}) {
  const workId = typeof work === 'string' ? work.trim() : '';
  if (!workId) {
    throw new Error('knowledge promote: missing_work — --work <id> is required (D38).');
  }

  const concepts = collectConcepts(root);
  const workConcept = concepts.find(
    (concept) => concept.data.type === 'bee.work-item' && beeOf(concept.data).id === workId,
  );
  if (!workConcept) {
    throw new Error(
      `knowledge promote: unknown_work — no bee.work-item concept in docs/knowledge/ carries bee.id "${workId}" (D38).`,
    );
  }

  const workBee = beeOf(workConcept.data);
  const workAreas = (Array.isArray(workBee.areas) ? workBee.areas : []).filter((area) => typeof area === 'string' && area);
  const workDecisions = (Array.isArray(workBee.decisions) ? workBee.decisions : []).filter((entry) => typeof entry === 'string');
  const workTags = (Array.isArray(workConcept.data.tags) ? workConcept.data.tags : []).filter((tag) => typeof tag === 'string');
  const cells = readCappedCellTraces(root, workId);

  // ── (a) the delivery draft ────────────────────────────────────────────────
  const deliveryPath = `${dirOf(workConcept.path) === '' ? '' : `${dirOf(workConcept.path)}/`}delivery.md`;
  const cappedDates = cells.map((cell) => isoDate(cell.capped_at)).filter((date) => date);
  const timestamp = cappedDates.length > 0 ? cappedDates.sort()[cappedDates.length - 1] : isoDate(workConcept.data.timestamp);
  const deviationCount = cells.reduce((sum, cell) => sum + cell.deviations.length, 0);
  const workTitle = typeof workConcept.data.title === 'string' && workConcept.data.title ? workConcept.data.title : workId;

  const deliveryData = {
    type: 'bee.delivery',
    title: `${workTitle} — delivery`,
    description: `Delivery record proposed by bee knowledge promote for work item ${workId}: ${cells.length} capped cell(s), ${deviationCount} recorded deviation(s).`,
    ...(workTags.length > 0 ? { tags: workTags } : {}),
    ...(timestamp ? { timestamp } : {}),
    bee: {
      id: `${workId}-delivery`,
      lifecycle: 'active',
      ...(workAreas.length > 0 ? { areas: workAreas } : {}),
      required_context: [workConcept.path],
      ...(workDecisions.length > 0 ? { decisions: workDecisions } : {}),
      sources: [`docs/knowledge/${workConcept.path}`, ...cells.map((cell) => cell.trace_path)],
      ...(typeof workBee.lane === 'string' && workBee.lane ? { lane: workBee.lane } : {}),
    },
  };

  const shipped = cells.length > 0
    ? cells.map((cell) => `- **${cell.id}** — ${oneLine(cell.outcome)} (${cell.files_changed.length} file(s) changed)`)
    : [`No capped cell trace for work item ${workId} exists in .bee/cells/ at proposal time.`];
  const verified = cells.length > 0
    ? cells.map((cell) => `- **${cell.id}** — \`${cell.verify}\`${cell.verify_summary ? ` — ${cell.verify_summary}` : ''}`)
    : ['Nothing to verify: no capped cell trace was found.'];
  const deviationLines = [];
  for (const cell of cells) {
    for (const deviation of cell.deviations) deviationLines.push(`- **${cell.id}** — ${oneLine(deviation)}`);
  }
  if (deviationLines.length === 0) deviationLines.push('None recorded in the capped cell traces.');

  const deliveryBody = [
    `# ${workTitle} — Delivery`,
    '',
    '## What shipped',
    '',
    ...shipped,
    '',
    '## Verify',
    '',
    'Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.',
    '',
    ...verified,
    '',
    '## Deviations',
    '',
    ...deviationLines,
    '',
    '## Provenance',
    '',
    `Proposed by \`bee knowledge promote --work ${workId}\` from ${cells.length} capped cell trace(s) in \`.bee/cells/\` and the work item \`docs/knowledge/${workConcept.path}\`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.`,
    '',
  ].join('\n');

  const delivery = {
    path: deliveryPath,
    repo_path: `docs/knowledge/${deliveryPath}`,
    content: `${emitFrontmatter(deliveryData)}\n${deliveryBody}`,
  };

  // ── (b) area updates ──────────────────────────────────────────────────────
  const areaUpdates = [];
  for (const area of workAreas) {
    const subjects = new Set();
    for (const concept of concepts) {
      const bee = beeOf(concept.data);
      const areas = Array.isArray(bee.areas) ? bee.areas : [];
      if (!areas.includes(area)) continue;
      subjects.add(`docs/knowledge/${concept.path}`);
      for (const source of Array.isArray(bee.sources) ? bee.sources : []) {
        if (typeof source === 'string' && source) subjects.add(source);
      }
    }
    const bullets = [];
    for (const cell of cells) {
      if (!cell.behavior_change) continue;
      const touched = cell.files_changed.filter((file) => [...subjects].some((subject) => touchesSubject(file, subject)));
      if (touched.length === 0) continue;
      bullets.push({ cell: cell.id, text: oneLine(cell.outcome), files: touched, trace: cell.trace_path });
    }
    areaUpdates.push({ area, subjects: [...subjects].sort(), bullets });
  }

  // ── (c) pattern candidates ────────────────────────────────────────────────
  const patternCandidates = [];
  for (const cell of cells) {
    if (cell.deviations.length === 0 && cell.failure_signatures.length === 0) continue;
    const evidence = [
      ...cell.deviations.map((text) => ({ kind: 'deviation', text })),
      ...cell.failure_signatures.map((text) => ({ kind: 'failure_signature', text })),
    ];
    const candidateData = {
      type: 'bee.pattern',
      title: `${workId} cell ${cell.id} — pitfall candidate`,
      description: `Pitfall candidate mined from cell ${cell.id}'s capped trace: ${oneLine(evidence[0].text, 160)}`,
      ...(isoDate(cell.capped_at) ? { timestamp: isoDate(cell.capped_at) } : {}),
      bee: {
        id: `${workId}-${cell.id}-pitfall`,
        lifecycle: 'draft',
        ...(workAreas.length > 0 ? { areas: workAreas } : {}),
        sources: [cell.trace_path],
        polarity: 'pitfall',
      },
    };
    const body = [
      `# ${workId} cell ${cell.id} — pitfall candidate`,
      '',
      '## What the cell did',
      '',
      oneLine(cell.outcome),
      '',
      `## Recorded evidence (verbatim from ${cell.trace_path})`,
      '',
      ...evidence.map((item) => `- **${item.kind}** — ${oneLine(item.text)}`),
      '',
      '## Status',
      '',
      'Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.',
      '',
    ].join('\n');
    patternCandidates.push({
      cell: cell.id,
      path: `patterns/${workId}-${cell.id}-pitfall.md`,
      repo_path: `docs/knowledge/patterns/${workId}-${cell.id}-pitfall.md`,
      evidence,
      content: `${emitFrontmatter(candidateData)}\n${body}`,
    });
  }

  return {
    work: workId,
    work_item: workConcept.path,
    cells,
    delivery,
    area_updates: areaUpdates,
    pattern_candidates: patternCandidates,
    // The contract, machine-readable: promote proposes and writes nothing.
    writes: [],
  };
}
