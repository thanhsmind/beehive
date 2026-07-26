// decisions.mjs — event-sourced decisions in .bee/decisions.jsonl.
// Write-time secret & injection rejection; datamarked reads.

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { appendJsonl, readJsonl, ensureDir, readJson, writeJsonAtomic } from './fsutil.mjs';
import { acquireStoreLockOnceSync } from './lock.mjs';

/** Content patterns that must never enter the decision log. */
export const SECRET_CONTENT_PATTERNS = [
  /-----BEGIN [A-Z ]*PRIVATE KEY-----/,
  /\bAKIA[0-9A-Z]{16}\b/,
  /\bghp_[A-Za-z0-9]{20,}\b/,
  /\bsk-[A-Za-z0-9_-]{20,}\b/,
  /\beyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{10,}/,
  /\b(?:api[_-]?key|secret|token|password|passwd)\s*[:=]\s*['"]?[^\s'"]{6,}/i,
];

/** Instruction-injection heuristics rejected at write time. */
export const INJECTION_PATTERNS = [
  /ignore\s+(?:all\s+)?(?:previous|prior|above|earlier)\s+(?:instructions|messages|context|prompts?)/i,
  /disregard\s+(?:all\s+)?(?:previous|prior|above|earlier)/i,
  /<\/?\s*(?:system|assistant|user|developer|tool)\b[^>]*>/i,
  /\[\s*(?:system|assistant|user|developer)\s*\]/i,
];

function decisionsPath(root) {
  return path.join(root, '.bee', 'decisions.jsonl');
}

// decision-propagation dp-3 (CONTEXT D4c): the archive sidecar. Same
// directory as the active store, never touched by any reader that doesn't
// explicitly opt into --all.
function decisionsArchivePath(root) {
  return path.join(root, '.bee', 'decisions-archive.jsonl');
}

// dp-3 CONCURRENCY (plan-checker BLOCKER — same class as cells.mjs's
// writeCell retrofit at cells.mjs:414-426): archiveDecisions prunes and
// rewrites the active store, so every writer that APPENDS to it
// (logDecision/supersedeDecision/redactDecision) must serialize against that
// rewrite under the SAME cross-process lock — a bare appendJsonl is only
// safe while nobody else is rewriting the file underneath it, and archive
// breaks that assumption. One unscoped lock name (not per-id, like
// cells.mjs's `cells:<id>` locks) because archive operates on the WHOLE
// store, not a single record.
export const DECISIONS_LOCK_NAME = 'decisions';

// Bounded synchronous retry on top of lock.mjs's single-attempt
// acquireStoreLockOnceSync — mirrors claims.mjs's acquireGateWithRetry
// (GATE_RETRY_ATTEMPTS/GATE_RETRY_DELAY_MS, ~300ms worst case) rather than
// cells.mjs's writeCell (which refuses instantly on contention): every
// caller here (logDecision/supersedeDecision/redactDecision/archiveDecisions)
// must stay fully synchronous (many call sites — cells.mjs's
// resetCellBudget/recordJudgeVerdict — invoke logDecision synchronously
// from inside their OWN already-locked `withStoreLock(cells:<id>, ...)`
// callback, which cannot become async), so this cannot use lock.mjs's async
// withStoreLock. The decisions store's critical sections are small file
// reads/writes (never a child spawn), so a short bounded wait is the right
// shape — an instant refusal would make ordinary concurrent logging flaky
// under the sub-ms-to-tens-of-ms contention this repo's lock doctrine
// expects, not a genuine failure.
const DECISIONS_LOCK_RETRY_ATTEMPTS = 15;
const DECISIONS_LOCK_RETRY_DELAY_MS = 20; // ~300ms worst-case wait, matching acquireGateWithRetry's budget

function sleepSyncMs(ms) {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
}

/** Typed refusal thrown by withDecisionsLockSync on timeout — never a silent unlocked write. */
export class DecisionsLockBusyError extends Error {
  constructor(holder) {
    const who =
      holder && typeof holder === 'object'
        ? `pid=${holder.pid ?? 'unknown'} session=${holder.session ?? 'unknown'} since ${holder.ts ?? 'unknown'}`
        : 'unknown holder';
    super(`decisions store lock "${DECISIONS_LOCK_NAME}" busy: held by ${who}`);
    this.name = 'DecisionsLockBusyError';
    this.code = 'DECISIONS_LOCK_BUSY';
    this.holder = holder ?? null;
  }
}

// withDecisionsLockSync(root, fn) — run fn() with the decisions store lock
// held, via a bounded synchronous retry loop (see comment above). Always
// releases in `finally`. Throws typed DecisionsLockBusyError after the
// budget is exhausted — never a fall-through unlocked write.
function withDecisionsLockSync(root, fn) {
  let lock = acquireStoreLockOnceSync(root, DECISIONS_LOCK_NAME);
  let attempt = 0;
  while (!lock.acquired && attempt < DECISIONS_LOCK_RETRY_ATTEMPTS) {
    sleepSyncMs(DECISIONS_LOCK_RETRY_DELAY_MS);
    lock = acquireStoreLockOnceSync(root, DECISIONS_LOCK_NAME);
    attempt += 1;
  }
  if (!lock.acquired) {
    throw new DecisionsLockBusyError(lock.holder);
  }
  try {
    return fn();
  } finally {
    lock.release();
  }
}

// decision-propagation dp-6 (CONTEXT D7a/b): the hand-curated tag taxonomy.
// Read by the CLI whenever docs/decisions/taxonomy.json exists; the CLI
// itself only ever appends unknown-tag names to `candidates[]` — it never
// touches the hand-curated `tags[]` vocabulary. Absence of the file is the
// bootstrap-safe state (dp-7 owns creating it for this repo; see the CAUTION
// note in the dp-6 cell — this module must never create it either).
function taxonomyPath(root) {
  return path.join(root, 'docs', 'decisions', 'taxonomy.json');
}

/** True when a taxonomy file exists at docs/decisions/taxonomy.json — the bootstrap/enforced boundary (D7b). */
export function taxonomyFileExists(root) {
  return fs.existsSync(taxonomyPath(root));
}

// readJson already fails open (returns the fallback) on a missing OR
// malformed file, warning to stderr on malformed JSON — reused here rather
// than a second bespoke parse-or-null implementation (fsutil.mjs is already
// imported by this module for appendJsonl/readJsonl/ensureDir).
function loadTaxonomy(root) {
  const raw = readJson(taxonomyPath(root), null);
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return null;
  const tags = Array.isArray(raw.tags) ? raw.tags : [];
  const candidates = Array.isArray(raw.candidates) ? raw.candidates.filter((c) => typeof c === 'string') : [];
  return { schema_version: raw.schema_version ?? 1, tags, candidates };
}

/** Typed refusal from classifyDecisionTags when a taxonomy exists but zero tags were supplied — decision-propagation D7b. */
export class DecisionsUntaggedRefusedError extends Error {
  constructor() {
    super(
      'decisions: docs/decisions/taxonomy.json exists — this decision event needs at least one tag. Pass --tags (e.g. "billing,recall").',
    );
    this.name = 'DecisionsUntaggedRefusedError';
    this.code = 'DECISIONS_UNTAGGED_REFUSED';
  }
}

// appendTaxonomyCandidatesSync — the CLI's only write path onto
// taxonomy.json: append previously-unseen tag names to `candidates[]`,
// de-duplicated against BOTH the hand-curated `tags[]` and the existing
// candidates. Runs under the SAME decisions store lock every other writer in
// this module uses (critical rule: "every store write keeps routing through
// the shared lock primitive") and re-reads the file fresh under that lock —
// a classic locked read-modify-write, so two concurrent unknown-tag appends
// can never lose one of them. The write itself is fsutil.mjs's own
// writeJsonAtomic (temp-write+rename) — reused rather than duplicated, so
// there is exactly one atomic-JSON-write implementation in this codebase,
// and a crash mid-write can never leave a partially-written taxonomy.json.
function appendTaxonomyCandidatesSync(root, unknownTags) {
  withDecisionsLockSync(root, () => {
    const fresh = loadTaxonomy(root);
    if (!fresh) return; // taxonomy vanished between the read and the write — nothing to append to
    const known = new Set([...fresh.tags.map((t) => t && t.name), ...fresh.candidates]);
    const nextCandidates = fresh.candidates.slice();
    for (const tag of unknownTags) {
      if (!known.has(tag) && !nextCandidates.includes(tag)) nextCandidates.push(tag);
    }
    if (nextCandidates.length !== fresh.candidates.length) {
      writeJsonAtomic(taxonomyPath(root), {
        schema_version: fresh.schema_version,
        tags: fresh.tags,
        candidates: nextCandidates,
      });
    }
  });
}

// classifyDecisionTags — decision-propagation dp-6 (CONTEXT D7b): the single
// write-time classification gate shared by logDecision and supersedeDecision
// (never source-exempted — "audit-source events follow the same rule,
// callers pass tags"). `tags` here is always the FINAL resolved array for
// the event about to be written (for supersede: after inheritance from the
// OVERLAY-APPLIED target — see supersedeDecision below).
//
//   - no taxonomy.json at all -> bootstrap-safe: never refuses, never writes
//     candidates. This is the unconditional current state of THIS repo (no
//     docs/decisions/taxonomy.json exists here — see the dp-6 cell's
//     self-hosting CAUTION), so every existing caller's behavior is
//     byte-unchanged until some later feature (dp-7) creates that file.
//   - taxonomy.json exists + zero tags -> typed refusal (DecisionsUntaggedRefusedError).
//   - taxonomy.json exists + at least one tag -> never refuses. Any tag not
//     already in tags[] or candidates[] is accepted onto the event AND
//     appended to candidates[] in this same call — never a second call, never
//     a refusal loop.
function classifyDecisionTags(root, tags) {
  const taxonomy = loadTaxonomy(root);
  if (!taxonomy) {
    return { taxonomyPresent: false };
  }
  if (!Array.isArray(tags) || tags.length === 0) {
    throw new DecisionsUntaggedRefusedError();
  }
  const known = new Set([...taxonomy.tags.map((t) => t && t.name), ...taxonomy.candidates]);
  const unknown = tags.filter((tag) => !known.has(tag));
  if (unknown.length) {
    appendTaxonomyCandidatesSync(root, unknown);
  }
  return { taxonomyPresent: true, unknownTags: unknown };
}

// writeJsonlAtomic — temp-write+rename the WHOLE active store, for archive's
// prune step. Local to this module (never added to the shared fsutil.mjs,
// which has no jsonl-atomic-rewrite primitive today and is out of this
// cell's file scope) — same atomic-rename shape as fsutil.mjs's own
// writeJsonAtomic, specialized for a jsonl body.
// tree-hygiene D3: same failed-rename discipline as fsutil.mjs's own
// writeJsonAtomic — unlink the tmp file best-effort, then rethrow the
// ORIGINAL error unchanged. Never masked, never leaked.
let writeJsonlAtomicCounter = 0;
export function writeJsonlAtomic(file, events) {
  ensureDir(path.dirname(file));
  const body = events.map((event) => JSON.stringify(event)).join('\n');
  const unique = `${process.pid}-${(writeJsonlAtomicCounter++).toString(36)}-${crypto.randomBytes(4).toString('hex')}`;
  const tmp = `${file}.${unique}.tmp`;
  try {
    fs.writeFileSync(tmp, body.length ? `${body}\n` : '', 'utf8');
    fs.renameSync(tmp, file);
  } catch (error) {
    try {
      fs.rmSync(tmp, { force: true });
    } catch {
      // best-effort cleanup — never let a cleanup failure mask the real error
    }
    throw error;
  }
}

// decision-propagation dp-5 (CONTEXT D7c, plan-check BLOCKER B1): the batch
// counterpart of fsutil.mjs's single-event appendJsonl, for tagDecisionsBatch
// below — every validated entry lands as ONE fs.appendFileSync call (one new
// jsonl "line group") rather than N sequential appendJsonl calls, matching
// the cell's "N sequential appends would leave a partial batch on crash"
// requirement. Local to this module for the same reason writeJsonlAtomic is
// (fsutil.mjs has no jsonl-batch-append primitive today and is out of this
// cell's file scope).
function appendJsonlBatch(file, events) {
  ensureDir(path.dirname(file));
  const body = events.map((event) => JSON.stringify(event)).join('\n');
  fs.appendFileSync(file, `${body}\n`, 'utf8');
}

// decision-propagation dp-1 (CONTEXT D4a): optional tags[] on a decide
// event, for structured recall alongside the existing free-string `scope`
// (which stays the spec-area dimension — no separate `area` field, fresh-
// eyes P2). Lowercase-slug shape mirrors the repo's existing feature-slug
// convention (worktree-store.mjs's FEATURE_SLUG_RE): one leading alnum,
// then alnum/hyphen.
export const TAG_PATTERN = /^[a-z0-9][a-z0-9-]*$/;

// undefined/null tags -> null (event gains NO tags key at all — additive,
// zero-migration parity with the 400+ pre-dp-1 events that never had one).
// Anything else must be a non-empty array of TAG_PATTERN-valid strings.
function normalizeTags(tags) {
  if (tags === undefined || tags === null) return null;
  if (!Array.isArray(tags)) {
    throw new Error('logDecision: tags must be an array of lowercase slugs (e.g. ["billing", "nightly-job"]).');
  }
  const cleaned = tags.map((tag) => String(tag).trim());
  for (const tag of cleaned) {
    if (!TAG_PATTERN.test(tag)) {
      throw new Error(
        `logDecision: tag ${JSON.stringify(tag)} is not a valid lowercase slug (must match ${TAG_PATTERN}).`,
      );
    }
  }
  return cleaned.length ? cleaned : null;
}

function assertSafeContent(field, value) {
  if (typeof value !== 'string' || !value) return;
  for (const pattern of SECRET_CONTENT_PATTERNS) {
    if (pattern.test(value)) {
      throw new Error(
        `Decision rejected: field "${field}" matches a secret pattern (${pattern}). Never log credentials — describe the decision without the secret.`,
      );
    }
  }
  for (const pattern of INJECTION_PATTERNS) {
    if (pattern.test(value)) {
      throw new Error(
        `Decision rejected: field "${field}" contains instruction-like content (${pattern}). Decision text must be data, not instructions.`,
      );
    }
  }
}

function assertSafe(fields) {
  for (const [field, value] of Object.entries(fields)) {
    assertSafeContent(field, value);
  }
}

export function logDecision(
  root,
  { decision, rationale, alternatives = null, scope = 'repo', source = 'user', confidence = null, tags = undefined },
) {
  if (typeof decision !== 'string' || !decision.trim()) {
    throw new Error('logDecision: decision text is required.');
  }
  if (typeof rationale !== 'string' || !rationale.trim()) {
    throw new Error('logDecision: rationale is required.');
  }
  assertSafe({ decision, rationale, alternatives, scope, source });
  const normalizedTags = normalizeTags(tags);

  // decision-propagation dp-6 (CONTEXT D7b): write-time classification gate.
  // Bootstrap-safe (no taxonomy.json -> never refuses); once a taxonomy
  // exists, a zero-tag decide event is refused (typed) and an unknown tag is
  // accepted onto the event while landing in taxonomy.json's candidates[].
  classifyDecisionTags(root, normalizedTags || []);

  const event = {
    id: crypto.randomUUID(),
    type: 'decide',
    date: new Date().toISOString(),
    decision: decision.trim(),
    rationale: rationale.trim(),
    alternatives,
    scope,
    source,
    confidence,
  };
  if (normalizedTags) event.tags = normalizedTags;
  // dp-3: the append itself runs under the SAME store lock archiveDecisions
  // holds for its whole read-prune-rewrite transaction — see
  // withDecisionsLockSync's comment above. Either this append lands fully
  // before archive ever reads the file, or fully after archive's rename —
  // never mid-transaction, so no write is ever lost or silently clobbered.
  withDecisionsLockSync(root, () => appendJsonl(decisionsPath(root), event));
  return event;
}

// decision-propagation dp-2 (CONTEXT D2/D3): the propagation sweep. Scans
// docs/** ONLY (D2 pinned root — .bee/spikes/ sits outside docs/ and is
// excluded by construction, no special-case needed) for text files (md,
// json, yaml/yml, txt) citing the superseded decision by its full id or its
// short8 form (the id's first 8 hex chars, e.g. "1178cfce" from a uuid like
// "1178cfce-...") — a \b...\b word-boundary match so a short8 embedded
// inside a longer alnum run (e.g. "abc1178cfcedef") never false-positives.
const SWEEP_TEXT_EXTENSIONS = new Set(['.md', '.json', '.yaml', '.yml', '.txt']);
const SWEEP_EXCERPT_MAX = 160;

// Exported (sqs-b1, D 5ca69717) so bee.mjs's --cell/--feature word-boundary
// text filters reuse the SAME escaping discipline as sweepDecisionCitations
// below, rather than a second copy drifting out of sync.
export function escapeRegExp(value) {
  return String(value).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function collectSweepFiles(dir, out) {
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return;
  }
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      collectSweepFiles(full, out);
    } else if (entry.isFile() && SWEEP_TEXT_EXTENSIONS.has(path.extname(entry.name).toLowerCase())) {
      out.push(full);
    }
  }
}

/**
 * Scan docs/** for citations of a superseded decision id (full id and
 * short8, word-boundary matched). Returns {scanned_at, hit_count, files[]}
 * with one entry per citing LINE: {file (repo-relative), line (1-based),
 * excerpt (trimmed, <=160 chars)}. Never edits the citing files — read-only.
 */
export function sweepDecisionCitations(root, { id, short8 }) {
  const docsRoot = path.join(root, 'docs');
  const candidateFiles = [];
  collectSweepFiles(docsRoot, candidateFiles);

  const idPattern = new RegExp(`\\b${escapeRegExp(id)}\\b`, 'i');
  const shortPattern = new RegExp(`\\b${escapeRegExp(short8)}\\b`, 'i');
  const files = [];
  for (const file of candidateFiles) {
    let text;
    try {
      text = fs.readFileSync(file, 'utf8');
    } catch {
      continue;
    }
    const lines = text.split(/\r?\n/);
    lines.forEach((line, index) => {
      if (idPattern.test(line) || shortPattern.test(line)) {
        const trimmed = line.trim();
        const excerpt = trimmed.length > SWEEP_EXCERPT_MAX ? `${trimmed.slice(0, SWEEP_EXCERPT_MAX - 3)}...` : trimmed;
        files.push({ file: path.relative(root, file), line: index + 1, excerpt });
      }
    });
  }
  return { scanned_at: new Date().toISOString(), hit_count: files.length, files };
}

export function supersedeDecision(root, { supersedes, decision, rationale, tags = undefined, scope = undefined }) {
  if (typeof supersedes !== 'string' || !supersedes.trim()) {
    throw new Error('supersedeDecision: supersedes (decision id) is required.');
  }
  if (typeof decision !== 'string' || !decision.trim()) {
    throw new Error('supersedeDecision: replacement decision text is required.');
  }
  if (typeof rationale !== 'string' || !rationale.trim()) {
    throw new Error('supersedeDecision: rationale is required.');
  }
  const targetId = supersedes.trim();
  assertSafe({ decision, rationale });

  // decision-propagation dp-2 (CONTEXT D6): resolve scope/tags — an explicit
  // flag wins; otherwise inherit from the superseded target; a metadata-less
  // target (or a target id not found in the store at all) falls back to
  // scope "repo" and no tags key at all, mirroring logDecision's zero-
  // migration additive shape.
  //
  // decision-propagation dp-6 (plan-check W3): inheritance consults the
  // OVERLAY-APPLIED target, not the raw event — a legacy target classified
  // only via a dp-5 retro-tag event (raw event.tags undefined) must still be
  // seen as tagged here, or a taxonomy-present supersede would wrongly
  // refuse it as zero-tag. applyTagOverlay returns the SAME object unchanged
  // when there is no overlay for that id, so this is a no-op for every
  // target dp-2's existing tests already cover.
  const events = readJsonl(decisionsPath(root));
  const rawTarget = events.find((event) => event && event.id === targetId);
  const target = rawTarget ? applyTagOverlay(rawTarget, buildTagOverlay(root)) : null;

  let resolvedScope;
  if (scope !== undefined && scope !== null && String(scope).trim()) {
    resolvedScope = String(scope).trim();
  } else if (target && typeof target.scope === 'string' && target.scope.trim()) {
    resolvedScope = target.scope.trim();
  } else {
    resolvedScope = 'repo';
  }
  assertSafeContent('scope', resolvedScope);

  let resolvedTags;
  if (tags !== undefined) {
    resolvedTags = normalizeTags(tags);
  } else if (target && Array.isArray(target.tags) && target.tags.length) {
    resolvedTags = normalizeTags(target.tags);
  } else {
    resolvedTags = null;
  }

  // decision-propagation dp-6 (CONTEXT D7b): same write-time classification
  // gate as logDecision, applied to the FINAL resolved (explicit-or-
  // inherited) tags — audit-source callers get no exemption.
  classifyDecisionTags(root, resolvedTags || []);

  // decision-propagation dp-2 (CONTEXT D2, lock doctrine): compute the
  // propagation sweep BEFORE the append below — the event is written to the
  // store exactly once, already carrying the sweep result inline. Never a
  // post-append rewrite of an already-written jsonl line.
  const short8 = targetId.slice(0, 8);
  const sweep = sweepDecisionCitations(root, { id: targetId, short8 });

  const event = {
    id: crypto.randomUUID(),
    type: 'supersede',
    date: new Date().toISOString(),
    supersedes: targetId,
    decision: decision.trim(),
    rationale: rationale.trim(),
    scope: resolvedScope,
    sweep,
  };
  if (resolvedTags) event.tags = resolvedTags;
  // dp-3: the append itself runs under the SAME store lock archiveDecisions
  // holds for its whole read-prune-rewrite transaction — see
  // withDecisionsLockSync's comment above. Either this append lands fully
  // before archive ever reads the file, or fully after archive's rename —
  // never mid-transaction, so no write is ever lost or silently clobbered.
  withDecisionsLockSync(root, () => appendJsonl(decisionsPath(root), event));
  return event;
}

export function redactDecision(root, { redacts, reason }) {
  if (typeof redacts !== 'string' || !redacts.trim()) {
    throw new Error('redactDecision: redacts (decision id) is required.');
  }
  if (typeof reason !== 'string' || !reason.trim()) {
    throw new Error('redactDecision: reason is required.');
  }
  const event = {
    id: crypto.randomUUID(),
    type: 'redact',
    date: new Date().toISOString(),
    redacts: redacts.trim(),
    reason: reason.trim(),
  };
  // dp-3: the append itself runs under the SAME store lock archiveDecisions
  // holds for its whole read-prune-rewrite transaction — see
  // withDecisionsLockSync's comment above. Either this append lands fully
  // before archive ever reads the file, or fully after archive's rename —
  // never mid-transaction, so no write is ever lost or silently clobbered.
  withDecisionsLockSync(root, () => appendJsonl(decisionsPath(root), event));
  return event;
}

// decision-propagation dp-5 (CONTEXT D7c): the retro-tag event. Append-only
// {type:'tag', id, date, target, tags, scope?} — never rewrites the target's
// own jsonl line; activeDecisions overlays tags/scope onto the target at
// read time (see buildTagOverlay/applyTagOverlay below). Tag events are
// never archived this slice (D7c prohibition) — archiveDecisions' existing
// rules (age sweep only for type 'decide'; supersede/redact-id sweep only
// for ids named by a LATER supersede/redact event) already never touch a
// 'tag'-typed event, so no change was needed there.

/** Typed refusal from tagDecision/tagDecisionsBatch when a target does not resolve to any decide/supersede event. */
export class DecisionsTagTargetUnresolvedError extends Error {
  constructor(target) {
    super(
      `decisions tag: target ${JSON.stringify(target)} does not resolve to any decide/supersede event in the active+archive union.`,
    );
    this.name = 'DecisionsTagTargetUnresolvedError';
    this.code = 'DECISIONS_TAG_TARGET_UNRESOLVED';
    this.target = target;
  }
}

/** Typed refusal when a short8 prefix matches more than one candidate — never guesses. */
export class DecisionsTagTargetAmbiguousError extends Error {
  constructor(target, matches) {
    super(
      `decisions tag: short id ${JSON.stringify(target)} is ambiguous — matches ${matches.length} events (${matches.join(', ')}); use the full id.`,
    );
    this.name = 'DecisionsTagTargetAmbiguousError';
    this.code = 'DECISIONS_TAG_TARGET_AMBIGUOUS';
    this.target = target;
    this.matches = matches;
  }
}

const SHORT8_PATTERN = /^[0-9a-f]{8}$/i;

// decision-propagation dp-5 (TARGET RESOLUTION): candidates are decide/
// supersede events from the SAME active+archive union dp-3's activeDecisions
// already reads (de-dup by id, active copy wins) — a redact or tag event id
// is never a valid retro-tag target.
function decisionTargetCandidates(root) {
  const activeEvents = readJsonl(decisionsPath(root));
  const archivedEvents = readJsonl(decisionsArchivePath(root));
  const byId = new Map();
  for (const event of activeEvents) {
    if (event && typeof event.id === 'string') byId.set(event.id, event);
  }
  for (const event of archivedEvents) {
    if (event && typeof event.id === 'string' && !byId.has(event.id)) byId.set(event.id, event);
  }
  return [...byId.values()].filter((event) => event && (event.type === 'decide' || event.type === 'supersede'));
}

function resolveTagTarget(candidates, target) {
  const raw = typeof target === 'string' ? target.trim() : '';
  if (!raw) {
    throw new Error('decisions tag: target id (full id or short8) is required.');
  }
  const exact = candidates.find((event) => event.id === raw);
  if (exact) return exact.id;
  if (SHORT8_PATTERN.test(raw)) {
    const matches = candidates.filter((event) => event.id.toLowerCase().startsWith(raw.toLowerCase()));
    if (matches.length === 1) return matches[0].id;
    if (matches.length > 1) throw new DecisionsTagTargetAmbiguousError(raw, matches.map((event) => event.id));
  }
  throw new DecisionsTagTargetUnresolvedError(raw);
}

// tags is required (unlike logDecision's optional tags[]) — a tag event with
// zero tags has no purpose; overlay REPLACES the whole array, so this is
// always a full, validated set.
function normalizeTagEventTags(tags) {
  if (!Array.isArray(tags) || tags.length === 0) {
    throw new Error('decisions tag: --tags is required (at least one lowercase slug, e.g. "billing,nightly-job").');
  }
  const cleaned = tags.map((tag) => String(tag).trim());
  for (const tag of cleaned) {
    if (!TAG_PATTERN.test(tag)) {
      throw new Error(
        `decisions tag: tag ${JSON.stringify(tag)} is not a valid lowercase slug (must match ${TAG_PATTERN}).`,
      );
    }
  }
  return cleaned;
}

/**
 * tagDecisionsBatch(root, entries) — decision-propagation dp-5 (CONTEXT D7c).
 * `entries` is an array of {target, tags, scope?}. EVERY entry is resolved
 * and validated BEFORE any write (all-or-nothing — a single unresolvable
 * target or invalid tags entry anywhere in the array means the WHOLE batch
 * refuses and nothing is appended). Once validated, every event lands in
 * exactly ONE locked append (appendJsonlBatch), the SAME
 * withDecisionsLockSync primitive every other decisions-store writer uses —
 * never a bare unlocked appendJsonl (plan-check BLOCKER B1).
 */
export function tagDecisionsBatch(root, entries) {
  if (!Array.isArray(entries) || entries.length === 0) {
    throw new Error('decisions tag: at least one entry ({target, tags, scope?}) is required.');
  }
  const candidates = decisionTargetCandidates(root);
  const now = new Date().toISOString();
  const events = entries.map((entry) => {
    if (!entry || typeof entry !== 'object' || Array.isArray(entry)) {
      throw new Error(`decisions tag: batch entry must be an object {target, tags, scope?}, got ${JSON.stringify(entry)}.`);
    }
    const targetId = resolveTagTarget(candidates, entry.target);
    const tags = normalizeTagEventTags(entry.tags);
    let scope;
    if (entry.scope !== undefined && entry.scope !== null && String(entry.scope).trim()) {
      scope = String(entry.scope).trim();
    }
    assertSafeContent('scope', scope);
    const event = {
      id: crypto.randomUUID(),
      type: 'tag',
      date: now,
      target: targetId,
      tags,
    };
    if (scope) event.scope = scope;
    return event;
  });

  // dp-3 lock doctrine (see withDecisionsLockSync's comment above): every
  // event in the batch is already fully built before the lock is taken —
  // the critical section is exactly the one appendJsonlBatch write.
  withDecisionsLockSync(root, () => appendJsonlBatch(decisionsPath(root), events));
  return events;
}

/** Single-entry convenience wrapper over tagDecisionsBatch — same all-or-nothing validate-then-append shape, one entry. */
export function tagDecision(root, { target, tags, scope } = {}) {
  return tagDecisionsBatch(root, [{ target, tags, scope }])[0];
}

// decision-propagation dp-5 (OVERLAY MERGE, plan-check W2): the overlay map
// is built from tag events in the ACTIVE file ONLY (tag events are never
// archived this slice — see the prohibition above), keyed by target id.
// "Latest tag event wins ... by event date, then file order": entries are
// visited oldest-to-newest (ties broken by ascending original file
// position), and each Map.set overwrites the previous value for that
// target — so the LAST entry visited (latest date, or the later-in-file
// entry on an exact-timestamp tie) is what survives in the map.
function buildTagOverlay(root) {
  const tagEvents = readJsonl(decisionsPath(root))
    .map((event, idx) => ({ event, idx }))
    .filter(({ event }) => event && event.type === 'tag' && typeof event.target === 'string');
  tagEvents.sort((a, b) => {
    const aMs = Date.parse(a.event.date);
    const bMs = Date.parse(b.event.date);
    if (Number.isFinite(aMs) && Number.isFinite(bMs) && aMs !== bMs) return aMs - bMs;
    return a.idx - b.idx;
  });
  const overlay = new Map();
  for (const { event } of tagEvents) {
    overlay.set(event.target, {
      tags: Array.isArray(event.tags) ? event.tags.slice() : undefined,
      scope: typeof event.scope === 'string' && event.scope ? event.scope : undefined,
    });
  }
  return overlay;
}

// Overlay REPLACES the whole tags array (never merges/unions); scope is
// replaced only when the winning tag event actually carries one. Returns
// the SAME event object unchanged when there is no overlay for its id —
// never a defensive clone on the common (untagged) path.
function applyTagOverlay(event, overlay) {
  const patch = overlay.get(event.id);
  if (!patch) return event;
  const next = { ...event };
  if (patch.tags !== undefined) next.tags = patch.tags;
  if (patch.scope !== undefined) next.scope = patch.scope;
  return next;
}

/** Typed refusal from archiveDecisions when zero events qualify (never a silent no-op). */
export class DecisionsArchiveNothingQualifiesError extends Error {
  constructor(before) {
    super(
      `archiveDecisions: nothing qualifies for archiving — no superseded/redacted events and no decide events strictly older than ${before} (decision-propagation D4c: --before is explicit or the verb refuses; there is never a default age-based purge).`,
    );
    this.name = 'DecisionsArchiveNothingQualifiesError';
    this.code = 'DECISIONS_ARCHIVE_NOTHING_QUALIFIES';
  }
}

// decision-propagation dp-3 (CONTEXT D4c): moves (1) every superseded/
// redacted event ALWAYS, regardless of age — an event is "superseded" or
// "redacted" the moment some OTHER event's `supersedes`/`redacts` field
// names its id, and such an event is already permanently excluded from
// activeDecisions' result, so relocating it changes nothing observable about
// the active set — and (2) every plain `decide` event strictly older than
// the explicit `before` cutoff, from .bee/decisions.jsonl to
// .bee/decisions-archive.jsonl. `supersede`/`redact` ACTION records
// themselves (the events THAT perform a supersession/redaction) are never
// swept by the age rule — only by rule (1), if some LATER event also
// supersedes/redacts them — so the active file's own audit trail of
// "what superseded what" never ages out silently.
//
// CRASH SAFETY (plan-checker BLOCKER): under the SAME store lock every
// append writer takes (DECISIONS_LOCK_NAME), qualifying events are appended
// to the archive file FIRST, then the pruned active file is written via
// temp-write+rename. A crash between those two steps leaves the same id in
// BOTH files — union reads (activeDecisions({all:true})) de-duplicate by id
// with the ACTIVE copy winning, so recovery is automatic; there is
// deliberately no rename-journal (cells.mjs's journal maps to whole-file
// renames, not jsonl line partitioning — it does not fit this shape).
//
// Refuses (typed DecisionsArchiveNothingQualifiesError) when `before` is
// missing/invalid, or when zero events qualify under either rule — archiving
// is opt-in and explicit, never a default purge, and a no-op call is never
// silently accepted as success (this is also what makes a second run over
// the same cutoff idempotent: nothing new qualifies, so it refuses cleanly
// and leaves both files byte-untouched).
export function archiveDecisions(root, { before } = {}) {
  if (before === undefined || before === null || !String(before).trim()) {
    throw new Error(
      'archiveDecisions: --before <ISO date> is required — decisions archive never runs a default age-based purge (decision-propagation D4c).',
    );
  }
  const beforeStr = String(before).trim();
  const beforeMs = Date.parse(beforeStr);
  if (!Number.isFinite(beforeMs)) {
    throw new Error(`archiveDecisions: --before must be a valid ISO date, got ${JSON.stringify(beforeStr)}.`);
  }

  return withDecisionsLockSync(root, () => {
    const activePath = decisionsPath(root);
    const archivePath = decisionsArchivePath(root);
    const events = readJsonl(activePath);

    const supersededIds = new Set();
    const redactedIds = new Set();
    for (const event of events) {
      if (event && event.type === 'supersede' && event.supersedes) supersededIds.add(event.supersedes);
      if (event && event.type === 'redact' && event.redacts) redactedIds.add(event.redacts);
    }

    const toArchive = [];
    const toKeep = [];
    for (const event of events) {
      if (!event || typeof event !== 'object' || typeof event.id !== 'string') {
        toKeep.push(event); // never drop a malformed-but-parsed line
        continue;
      }
      if (supersededIds.has(event.id) || redactedIds.has(event.id)) {
        toArchive.push(event);
        continue;
      }
      if (event.type === 'decide') {
        const eventMs = Date.parse(event.date);
        if (Number.isFinite(eventMs) && eventMs < beforeMs) {
          toArchive.push(event);
          continue;
        }
      }
      toKeep.push(event);
    }

    if (toArchive.length === 0) {
      throw new DecisionsArchiveNothingQualifiesError(beforeStr);
    }

    // Crash ordering (CONCURRENCY note above): archive-append FIRST.
    ensureDir(path.dirname(archivePath));
    const archiveBody = toArchive.map((event) => JSON.stringify(event)).join('\n');
    fs.appendFileSync(archivePath, `${archiveBody}\n`, 'utf8');

    // Then the pruned active file, as a single atomic temp-write+rename —
    // surviving events are written back VERBATIM (never rewritten/touched).
    writeJsonlAtomic(activePath, toKeep);

    return {
      archived: toArchive.map((event) => event.id),
      kept: toKeep.length,
      before: beforeStr,
    };
  });
}

/**
 * Decide/supersede events not themselves superseded or redacted, newest
 * first. Default (no `all`) is byte-identical to pre-dp-3 behavior — reads
 * ONLY the active store.
 *
 * `all: true` (decision-propagation D4c) additionally unions in
 * .bee/decisions-archive.jsonl (missing/empty archive file is silently
 * treated as "nothing extra"): active events first, then any archived event
 * whose id is not already present in the active file (de-dup by id — the
 * active copy always wins, matching archiveDecisions' crash-ordering note).
 * Superseded/redacted resolution runs over the FULL union, since an archived
 * decide event's supersede/redact record always lives in the active file
 * (dp-3 never archives an action record purely by age). Ordering is sorted
 * explicitly by event date descending (never a positional .reverse() —
 * merging two independently-chronological files cannot rely on file order
 * alone once activity happened before AND after `before` on either side).
 */
export function activeDecisions(root, { recent = null, all = false } = {}) {
  // decision-propagation dp-5 (D7c OVERLAY MERGE, plan-check W2): built once
  // per call, from the active file's tag events only, and applied to BOTH
  // branches below identically — this is what keeps the `all` branch byte-
  // identical to the default branch whenever there is nothing in the
  // archive to actually merge (dp-3's existing byte-identity requirement),
  // and what makes an archived-then-retro-tagged target still read overlaid
  // (the overlay is keyed by target id, independent of which file that
  // target's own event currently lives in).
  const overlay = buildTagOverlay(root);
  if (!all) {
    const events = readJsonl(decisionsPath(root));
    const superseded = new Set();
    const redacted = new Set();
    for (const event of events) {
      if (event.type === 'supersede' && event.supersedes) superseded.add(event.supersedes);
      if (event.type === 'redact' && event.redacts) redacted.add(event.redacts);
    }
    const active = events
      .filter(
        (event) =>
          (event.type === 'decide' || event.type === 'supersede') &&
          !superseded.has(event.id) &&
          !redacted.has(event.id),
      )
      .reverse()
      .map((event) => applyTagOverlay(event, overlay));
    return recent != null ? active.slice(0, recent) : active;
  }

  const activeEvents = readJsonl(decisionsPath(root));
  const archivedEvents = readJsonl(decisionsArchivePath(root));
  const byId = new Map();
  for (const event of activeEvents) {
    if (event && typeof event.id === 'string') byId.set(event.id, event);
  }
  for (const event of archivedEvents) {
    if (event && typeof event.id === 'string' && !byId.has(event.id)) byId.set(event.id, event);
  }
  // Indexed BEFORE filtering so same-timestamp ties can break by original
  // position — two events sharing a millisecond-precision date are common
  // (back-to-back logDecision calls). On an unarchived store `events` is
  // exactly `activeEvents` in file (chronological, non-decreasing) order, so
  // "newest date first, ties broken by higher original index first" is
  // mathematically identical to `.reverse()` — this is what makes the `all`
  // path byte-identical to the default path whenever there is nothing in
  // the archive to actually merge in (D4c's byte-identical-for-unarchived
  // requirement), not merely usually-identical.
  const events = [...byId.values()];
  const indexed = events.map((event, idx) => ({ event, idx }));
  const superseded = new Set();
  const redacted = new Set();
  for (const { event } of indexed) {
    if (event.type === 'supersede' && event.supersedes) superseded.add(event.supersedes);
    if (event.type === 'redact' && event.redacts) redacted.add(event.redacts);
  }
  const active = indexed
    .filter(
      ({ event }) =>
        (event.type === 'decide' || event.type === 'supersede') &&
        !superseded.has(event.id) &&
        !redacted.has(event.id),
    )
    .sort((a, b) => {
      const bMs = Date.parse(b.event.date);
      const aMs = Date.parse(a.event.date);
      if (Number.isFinite(aMs) && Number.isFinite(bMs) && aMs !== bMs) return bMs - aMs;
      return b.idx - a.idx; // tie -> later-inserted (higher original index) first, matching .reverse()
    })
    // Overlay applied AFTER union de-dup (plan-check W2) — an archived
    // target that lost the dedup to nothing (i.e. it's the surviving
    // union entry) still gets its overlay applied here, exactly once.
    .map(({ event }) => applyTagOverlay(event, overlay));
  return recent != null ? active.slice(0, recent) : active;
}

// decision-propagation dp-4 (CONTEXT D4b/D6, overlay-aware per D7/D8): the
// derived, CLI-rendered decision index at docs/decisions/index.md — never
// hand-edited, regenerated only. Consumes activeDecisions(root, {all}), the
// SAME overlay-applied read path dp-5/dp-6 already established, so a
// retro-tagged legacy event (or a supersede event that inherited scope/tags
// from its target at WRITE time — dp-2/D6) renders under its final,
// overlay-applied scope/tags exactly as `search`/`active` already see it —
// no separate overlay logic needed here.
function decisionIndexPath(root) {
  return path.join(root, 'docs', 'decisions', 'index.md');
}

// Deliberately carries NO generation timestamp or any other wall-clock
// value — the must-have is "two consecutive renders over the same store are
// byte-identical", so this header (and the body below) is a pure function
// of the store's own event dates, never of when the render ran.
const DECISION_INDEX_HEADER = [
  '<!--',
  'GENERATED FILE — do not hand-edit.',
  'Rendered by `bee decisions render` from the decisions store (decision-propagation D4b/D8a).',
  'Regenerate: `bee decisions render`. Check freshness: `bee decisions render --check`.',
  'Deterministic: byte-identical for the same store contents — this file never includes a',
  'generation timestamp or any other wall-clock value, only the dates already recorded on',
  'each decision event.',
  '-->',
  '',
  '# Decision Index',
].join('\n');

// One line per decision: `short8 · YYYY-MM-DD · first line of decision
// text` — event.date is already a full ISO string, sliced to its date-only
// prefix (never the time-of-day component, which would reintroduce a
// wall-clock-shaped value read straight from data at least, but the spec's
// literal format is date-only). Only the FIRST line of a multi-line
// decision renders, so an embedded newline in `decision` can never break
// the one-line-per-decision contract.
function formatIndexLine(event) {
  const short8 = String(event.id).slice(0, 8);
  const date = typeof event.date === 'string' ? event.date.slice(0, 10) : '0000-00-00';
  const firstLine = String(event.decision ?? '').split(/\r?\n/)[0];
  return `- ${short8} · ${date} · ${firstLine}`;
}

// Grouped by scope (alphabetical), then by tag (alphabetical, untagged
// last). A decision's group tag is its FIRST tag (declared/overlay order) —
// one home per decision, never cross-listed under every tag it carries, so
// the rendered line count always equals the number of decisions rendered
// (no ambiguity for --check's byte-diff or any caller counting entries).
// `all` reaches the archive (D4c), matching search/active's own flag.
// Ordering within a group is newest-first, inherited for free from
// activeDecisions' own newest-first order (stable partition, never re-sorted).
function buildDecisionIndexBody(root, { all = false } = {}) {
  const decisions = activeDecisions(root, { all });
  const byScope = new Map();
  for (const event of decisions) {
    const scope = typeof event.scope === 'string' && event.scope.trim() ? event.scope.trim() : 'repo';
    if (!byScope.has(scope)) byScope.set(scope, []);
    byScope.get(scope).push(event);
  }
  const scopeNames = [...byScope.keys()].sort((a, b) => a.localeCompare(b));

  const blocks = [];
  let count = 0;
  for (const scope of scopeNames) {
    const scopeLines = [`## ${scope}`];
    const events = byScope.get(scope);
    const byTag = new Map();
    const untagged = [];
    for (const event of events) {
      const tag = Array.isArray(event.tags) && event.tags.length ? String(event.tags[0]) : null;
      if (tag) {
        if (!byTag.has(tag)) byTag.set(tag, []);
        byTag.get(tag).push(event);
      } else {
        untagged.push(event);
      }
    }
    const tagNames = [...byTag.keys()].sort((a, b) => a.localeCompare(b));
    for (const tag of tagNames) {
      scopeLines.push('', `### ${tag}`, '');
      for (const event of byTag.get(tag)) {
        scopeLines.push(formatIndexLine(event));
        count += 1;
      }
    }
    if (untagged.length) {
      scopeLines.push('', '### untagged', '');
      for (const event of untagged) {
        scopeLines.push(formatIndexLine(event));
        count += 1;
      }
    }
    blocks.push(scopeLines.join('\n'));
  }

  const body = blocks.length ? blocks.join('\n\n') : 'No active decisions.';
  return { body, count };
}

function decisionIndexContent(root, { all = false } = {}) {
  const { body, count } = buildDecisionIndexBody(root, { all });
  return { content: `${DECISION_INDEX_HEADER}\n\n${body}\n`, count };
}

// writeTextAtomic — same temp-write+rename shape as this module's own
// writeJsonlAtomic (dp-3), specialized for a plain-text body. Local for the
// same reason writeJsonlAtomic is: fsutil.mjs has no text-atomic-write
// primitive today and is out of this cell's file scope.
// tree-hygiene D3: same failed-rename discipline as writeJsonlAtomic above.
let writeTextAtomicCounter = 0;
export function writeTextAtomic(file, text) {
  ensureDir(path.dirname(file));
  const unique = `${process.pid}-${(writeTextAtomicCounter++).toString(36)}-${crypto.randomBytes(4).toString('hex')}`;
  const tmp = `${file}.${unique}.tmp`;
  try {
    fs.writeFileSync(tmp, text, 'utf8');
    fs.renameSync(tmp, file);
  } catch (error) {
    try {
      fs.rmSync(tmp, { force: true });
    } catch {
      // best-effort cleanup — never let a cleanup failure mask the real error
    }
    throw error;
  }
}

/**
 * renderDecisionIndex(root, {all}) — decision-propagation dp-4 (CONTEXT
 * D4b/D6). Computes the index from the active (+archive when `all`) store
 * and writes it atomically to docs/decisions/index.md. Returns
 * {path (repo-relative), content, count}. Never throws on an empty store —
 * an empty store still renders a valid file saying so.
 */
export function renderDecisionIndex(root, { all = false } = {}) {
  const { content, count } = decisionIndexContent(root, { all });
  const file = decisionIndexPath(root);
  writeTextAtomic(file, content);
  return { path: path.relative(root, file), content, count };
}

/**
 * decisionIndexDrift(root, {all}) — read-only: computes what the index
 * SHOULD be right now and compares it byte-for-byte against whatever is on
 * disk (a missing file counts as drift). Never writes. The CLI's `--check`
 * mode turns a `drift: true` result into a non-zero exit; this function
 * itself never throws.
 */
export function decisionIndexDrift(root, { all = false } = {}) {
  const { content } = decisionIndexContent(root, { all });
  const file = decisionIndexPath(root);
  let onDisk = null;
  try {
    onDisk = fs.readFileSync(file, 'utf8');
  } catch {
    onDisk = null;
  }
  return { drift: onDisk !== content, path: path.relative(root, file) };
}

/** Neutralize resurfaced text so it can never act as instructions. */
export function datamark(text) {
  const cleaned = String(text ?? '')
    .replace(/```+/g, '')
    .replace(/<\/?\s*(?:system|assistant|user|developer|tool)\b[^>]*>/gi, '')
    .replace(/[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F]/g, '')
    .trim();
  return `«${cleaned}»`;
}
