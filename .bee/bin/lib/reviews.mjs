// reviews.mjs — review-session store + candidates ledger (SPEC:
// docs/history/review-on-demand/SPEC.md §8, decisions 565e68d0/bb4bb18e).
//
// Full independent review is user-invoked (565e68d0): a review session is an
// immutable-scope inspection over completed changes, separate from mandatory
// per-cell verification. This module owns two stores:
//   - .bee/reviews/<id>.json          one session per file, mirrors cells.mjs
//   - .bee/review-candidates.jsonl    append-only, one entry per feature close
//
// CLI-owned mutation contract (bb4bb18e): every mutation is a CLI verb (see
// bee.mjs reviews); write paths here use STRICT reads (readReviewStrict — a
// corrupt session file fails loud rather than being silently rebuilt), while
// list/show stay fail-open (skip + warn on a corrupt file) so one bad session
// never breaks the whole ledger view. Session scope (baseline, head,
// included, excluded) freezes at `create` and `recordOnReview` refuses any
// attempt to touch those four fields afterward (R5).

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { readJson, writeJsonAtomic, appendJsonl, readJsonl } from './fsutil.mjs';
import { readCell } from './cells.mjs';

const ID_PATTERN = /^[A-Za-z0-9][A-Za-z0-9._-]*$/;

// Entry `type` values for included/excluded scope entries (SPEC §8: "feature/
// cell/commit"). Only `cell` entries are checked by the A6/A10 mechanics in
// this slice — feature/commit scope resolution is review-od-2's territory.
export const SCOPE_ENTRY_TYPES = ['cell', 'feature', 'commit'];

// The closing feature's lane, recorded verbatim on every candidate ledger
// entry (review-od-3's R7 high-risk warning reads this field). Adds `docs`
// to cells.mjs's LANES — a docs-only feature close still gets a candidate.
export const REVIEW_MODES = ['docs', 'tiny', 'small', 'spike', 'standard', 'high-risk'];

// Fields frozen at `create` (R5). `record` refuses a payload that carries
// any of these keys at all — none of the five sub-record kinds (manifest,
// preflight, finding, uat, decision) ever legitimately needs them.
const IMMUTABLE_FIELDS = ['baseline', 'head', 'included', 'excluded'];

const RECORD_KINDS = ['manifest', 'preflight', 'finding', 'uat', 'decision'];
const DECISION_STATUSES = ['pending', 'blocked', 'approved'];

function utcNow() {
  return new Date().toISOString();
}

export function reviewsDir(root) {
  return path.join(root, '.bee', 'reviews');
}

function reviewFile(root, id) {
  return path.join(reviewsDir(root), `${id}.json`);
}

export function candidatesPath(root) {
  return path.join(root, '.bee', 'review-candidates.jsonl');
}

function assertValidId(id) {
  if (typeof id !== 'string' || !ID_PATTERN.test(id)) {
    throw new Error(
      `invalid review id "${id}" — use letters, digits, dot, dash, underscore (e.g. "review-2026-07-12").`,
    );
  }
}

// ─── strict read (write-verb sibling of readReview) ────────────────────────
// Mirrors lib/state.mjs's readStateStrict: absent file -> a clear "not found"
// error (nothing to mutate); present-but-corrupt -> throws loud rather than
// silently returning a default that a write would then clobber the file
// with. Only create/record (write verbs) use this; list/show fail open.
export function readReviewStrict(root, id) {
  assertValidId(id);
  const file = reviewFile(root, id);
  let text;
  try {
    text = fs.readFileSync(file, 'utf8');
  } catch (err) {
    if (err && err.code === 'ENOENT') {
      throw new Error(`readReviewStrict: review session "${id}" not found at ${file}.`);
    }
    throw new Error(
      `readReviewStrict: could not read "${file}" (${err && err.code ? err.code : err}).`,
    );
  }
  let parsed;
  try {
    parsed = JSON.parse(text);
  } catch {
    throw new Error(
      `readReviewStrict: "${file}" exists but is not valid JSON. The bee CLI refuses to mutate ` +
        'a present-but-corrupt review session — that could silently clobber real review state ' +
        `(findings, decision, scope). FIX: inspect/restore the file (e.g. "git checkout -- ${file}"), then retry.`,
    );
  }
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error(
      `readReviewStrict: "${file}" exists but is not a JSON object (found ${Array.isArray(parsed) ? 'an array' : typeof parsed}).`,
    );
  }
  return parsed;
}

/** Fail-open read for a single session — read paths only, never a write precondition. */
export function readReview(root, id) {
  if (typeof id !== 'string' || !ID_PATTERN.test(id)) return null;
  return readJson(reviewFile(root, id), null);
}

/**
 * List every session, fail-open per file: a corrupt session.json is skipped
 * with a warning rather than breaking the whole listing (mirrors
 * listCells/cellsDir in lib/cells.mjs).
 */
export function listReviews(root) {
  const dir = reviewsDir(root);
  let entries;
  try {
    entries = fs.readdirSync(dir);
  } catch {
    return [];
  }
  const sessions = [];
  for (const entry of entries) {
    if (!entry.endsWith('.json')) continue;
    const session = readJson(path.join(dir, entry), null);
    if (!session || typeof session !== 'object' || Array.isArray(session)) {
      console.warn(`reviews: skipping corrupt session file ${entry} (list stays fail-open)`);
      continue;
    }
    sessions.push(session);
  }
  sessions.sort((a, b) => String(a.id).localeCompare(String(b.id), 'en', { numeric: true }));
  return sessions;
}

function writeReview(root, session) {
  writeJsonAtomic(reviewFile(root, session.id), session);
  return session;
}

function normalizeScopeEntry(raw) {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) {
    throw new Error(`create: scope entry must be an object, got ${JSON.stringify(raw)}.`);
  }
  const type = typeof raw.type === 'string' ? raw.type.trim() : '';
  if (!SCOPE_ENTRY_TYPES.includes(type)) {
    throw new Error(
      `create: scope entry has invalid type "${raw.type}" — must be one of ${SCOPE_ENTRY_TYPES.join(', ')}.`,
    );
  }
  if (typeof raw.id !== 'string' || !raw.id.trim()) {
    throw new Error('create: scope entry is missing a non-empty "id".');
  }
  const entry = { type, id: raw.id.trim() };
  if (typeof raw.reason === 'string' && raw.reason.trim()) entry.reason = raw.reason.trim();
  return entry;
}

/**
 * A10 + A6: walk the `included` cell entries, auto-excluding any cell whose
 * status is open/claimed (A6 — "in progress", never silently reviewed-in)
 * and fail-closing the whole create() when a remaining behavior_change cell
 * lacks recorded verification_evidence (A10). Pure — throws or returns the
 * split { included, excluded, checked } without writing anything, so create()
 * can run this BEFORE the first byte is written to disk.
 */
function runPreflight(root, included) {
  const stillIncluded = [];
  const autoExcluded = [];
  const checked = [];
  const missingEvidence = [];

  for (const entry of included) {
    if (entry.type !== 'cell') {
      stillIncluded.push(entry);
      continue;
    }
    const cell = readCell(root, entry.id);
    if (!cell) {
      throw new Error(
        `create: preflight cannot resolve included cell "${entry.id}" — no such cell. FIX: fix the scope input or drop the entry.`,
      );
    }
    if (cell.status === 'open' || cell.status === 'claimed') {
      // A6 — in-progress work is never silently included in a review scope.
      autoExcluded.push({ ...entry, reason: 'in progress' });
      continue;
    }
    stillIncluded.push(entry);
    const trace = cell.trace || {};
    if (trace.behavior_change === true) {
      checked.push(entry.id);
      const evidence = trace.verification_evidence;
      const hasEvidence =
        evidence != null && (typeof evidence !== 'string' || evidence.trim().length > 0);
      if (!hasEvidence) missingEvidence.push(entry.id);
    }
  }

  if (missingEvidence.length > 0) {
    // A10 — fail closed BEFORE any session file is written.
    throw new Error(
      `create: preflight failed — behavior_change cell(s) in scope have no recorded verification_evidence: ${missingEvidence.join(', ')}. ` +
        'Review cannot substitute for missing verification; fix evidence at the cell (bee.mjs cells cap --evidence-stdin) or drop the entry from scope, then retry.',
    );
  }

  return { included: stillIncluded, excluded: autoExcluded, checked };
}

/**
 * create — freezes a review scope (R5) into `.bee/reviews/<id>.json`.
 * Input `scope` carries: id, requested_by, scope_description, included,
 * excluded (optional pre-exclusions), baseline, head — echoed verbatim into
 * the stored session (SPEC §8 is the schema). Runs the A10 preflight and A6
 * auto-exclusion BEFORE any write; refuses an already-existing id (id
 * non-reuse, §8). On any refusal, zero files are written.
 */
export function createReview(root, scope) {
  if (!scope || typeof scope !== 'object' || Array.isArray(scope)) {
    throw new Error('create: scope input must be a JSON object.');
  }
  for (const field of ['id', 'requested_by', 'scope_description', 'baseline', 'head']) {
    if (typeof scope[field] !== 'string' || !scope[field].trim()) {
      throw new Error(`create: scope is missing required field "${field}" (non-empty string).`);
    }
  }
  assertValidId(scope.id);
  if (!Array.isArray(scope.included) || scope.included.length === 0) {
    throw new Error('create: scope requires a non-empty "included" array.');
  }
  if (scope.excluded !== undefined && !Array.isArray(scope.excluded)) {
    throw new Error('create: scope "excluded" must be an array when present.');
  }

  // Id non-reuse (§8: "Định danh ổn định, không tái sử dụng") — checked
  // before any normalization work so a duplicate id refuses cleanly.
  if (fs.existsSync(reviewFile(root, scope.id))) {
    throw new Error(
      `create: review session "${scope.id}" already exists — review ids are never reused. FIX: pick a new id.`,
    );
  }

  const includedEntries = scope.included.map(normalizeScopeEntry);
  const preExcluded = (scope.excluded || []).map(normalizeScopeEntry).map((entry) => ({
    ...entry,
    reason: entry.reason || 'excluded at request',
  }));

  const { included, excluded: autoExcluded, checked } = runPreflight(root, includedEntries);

  const now = utcNow();
  const session = {
    id: scope.id.trim(),
    requested_by: scope.requested_by.trim(),
    requested_at: now,
    scope_description: scope.scope_description.trim(),
    included,
    excluded: [...preExcluded, ...autoExcluded],
    baseline: scope.baseline.trim(),
    head: scope.head.trim(),
    reviewer_manifest: [],
    verification_preflight: { checked_at: now, cells_checked: checked, passed: true },
    findings: [],
    uat: [],
    decision: { status: 'pending', gate4: null },
    created_at: now,
    updated_at: now,
  };
  return writeReview(root, session);
}

/**
 * record — appends/sets a sub-record on an existing session. `kind` selects
 * the slot: manifest/preflight/decision SET (replace the whole field),
 * finding/uat APPEND one entry per call. Refuses (non-zero, file untouched)
 * when the payload carries any of baseline/head/included/excluded — those
 * are frozen at create (R5) and no sub-record kind legitimately needs them.
 */
export function recordOnReview(root, id, { kind, payload }) {
  if (!RECORD_KINDS.includes(kind)) {
    throw new Error(`record: invalid kind "${kind}" — must be one of ${RECORD_KINDS.join(', ')}.`);
  }
  if (payload === undefined || payload === null) {
    throw new Error('record: payload is required.');
  }
  if (typeof payload !== 'object' || Array.isArray(payload)) {
    throw new Error(`record: payload for kind "${kind}" must be a JSON object.`);
  }
  const forbidden = IMMUTABLE_FIELDS.filter((field) => field in payload);
  if (forbidden.length > 0) {
    throw new Error(
      `record: refused — payload attempts to touch immutable scope field(s): ${forbidden.join(', ')}. ` +
        'baseline/head/included/excluded are frozen at create (R5) and cannot change afterward.',
    );
  }

  // Strict read (write-verb sibling): a corrupt session file fails loud
  // BEFORE any write, rather than being silently treated as absent.
  const session = readReviewStrict(root, id);

  if (kind === 'decision') {
    const status = payload.status;
    if (!DECISION_STATUSES.includes(status)) {
      throw new Error(
        `record: decision.status must be one of ${DECISION_STATUSES.join(', ')}, got "${status}".`,
      );
    }
    session.decision = { ...payload };
  } else if (kind === 'manifest') {
    session.reviewer_manifest = payload;
  } else if (kind === 'preflight') {
    session.verification_preflight = payload;
  } else if (kind === 'finding') {
    session.findings = Array.isArray(session.findings) ? session.findings : [];
    session.findings.push(payload);
  } else if (kind === 'uat') {
    session.uat = Array.isArray(session.uat) ? session.uat : [];
    session.uat.push(payload);
  }

  session.updated_at = utcNow();
  return writeReview(root, session);
}

// ─── candidates ledger ──────────────────────────────────────────────────────
// Append-only, one entry per feature close (SPEC §7.1 step 6 / §11.5). Mirrors
// decisions.jsonl / backlog.jsonl (fsutil's appendJsonl/readJsonl). Status is
// intentionally NOT stored here — deriving unreviewed/in review/reviewed/
// review stale from git + open sessions is review-od-2's scope.

export function addCandidate(root, { feature, head, mode, baseline = null, cells = [] }) {
  if (typeof feature !== 'string' || !feature.trim()) {
    throw new Error('candidate add: feature is required.');
  }
  if (typeof head !== 'string' || !head.trim()) {
    throw new Error('candidate add: head (commit sha) is required.');
  }
  if (typeof mode !== 'string' || !mode.trim() || !REVIEW_MODES.includes(mode.trim())) {
    throw new Error(
      `candidate add: --mode is required and must be one of ${REVIEW_MODES.join(', ')} (the closing feature's lane).`,
    );
  }
  const entry = {
    id: crypto.randomUUID(),
    type: 'candidate',
    date: utcNow(),
    feature: feature.trim(),
    head: head.trim(),
    mode: mode.trim(),
    baseline: baseline == null ? null : String(baseline).trim() || null,
    cells: Array.isArray(cells) ? cells.filter((c) => typeof c === 'string' && c.trim()) : [],
  };
  appendJsonl(candidatesPath(root), entry);
  return entry;
}

/** Fail-open: readJsonl already skips corrupt lines rather than throwing. */
export function listCandidates(root) {
  return readJsonl(candidatesPath(root));
}

// ─── derived coverage / staleness (review-od-2, SPEC §5/§8/R6/R10, A7/A8) ──
// Candidate status is NEVER stored — always derived at read time from
// session records + git (R6/R10: "status độc lập với implementation status").
// Coverage attaches only to immutable baseline/head content identity (§8),
// never to feature name or date. Git failures NEVER throw out of this read
// path (fail toward honesty, plan.md open question 1): an unresolvable range
// (missing git binary, unknown sha after rebase/amend, shallow clone) with a
// covering session degrades to 'review stale' with a 'range unresolvable'
// note; with no covering session it degrades to plain 'unreviewed'. Exported
// for `bee.mjs status` (review-od-3) to summarize candidate counts.

export const CANDIDATE_STATUSES = ['unreviewed', 'in review', 'reviewed', 'review stale'];

function sessionCoversCandidate(session, candidate) {
  if (!session || !Array.isArray(session.included)) return false;
  const featureMatch = session.included.some(
    (e) => e && e.type === 'feature' && e.id === candidate.feature,
  );
  if (featureMatch) return true;
  const cells = Array.isArray(candidate.cells) ? candidate.cells.filter(Boolean) : [];
  if (cells.length === 0) return false;
  const includedCellIds = new Set(
    session.included.filter((e) => e && e.type === 'cell').map((e) => e.id),
  );
  return cells.every((id) => includedCellIds.has(id));
}

/** SPEC §5: "In review" also covers a `blocked` session (P1 fix pending, R8) — anything short of `approved`. */
function isSessionOpen(session) {
  return !session.decision || session.decision.status !== 'approved';
}

// Every git call in this module takes an explicit `root` as cwd — never
// process.cwd() — consistent with every other lib function taking root.
function defaultRunGit(root, args) {
  return spawnSync('git', args, { cwd: root, encoding: 'utf8' });
}

// D2 (docs/history/cli-performance/CONTEXT.md): a derivation PASS over many
// candidates (buildReviewBlock / buildReviewsStatusSummary in bee.mjs) sees
// the same (head, ref) / (ref) git questions over and over across candidates
// that share a covering session. `memo`, when passed, is a single pass-local
// Map (born in the caller's candidate loop, never persisted, never TTL'd —
// see CONTEXT "Pass-local cache") that these two functions read-through and
// populate. Key strings are namespace-prefixed so the two call shapes can
// never collide inside one Map. `runGit` is an injection seam (defaults to
// the real spawnSync-backed runner) — production callers never pass it;
// tests use it to count real git invocations without faking the answers.
function coveredByKey(head, ref) {
  return `covered ${head} ${ref}`;
}
function commitsSinceKey(ref) {
  return `since ${ref}`;
}

/**
 * Is `head` an ancestor of (or equal to) `ref` in `root`'s git history?
 * Returns { covered: true|false, unresolved: false } on a clean answer, or
 * { covered: null, unresolved: true } when git cannot answer (missing
 * binary, unknown sha after rebase/amend, shallow clone) — never throws.
 */
function headCoveredBy(root, head, ref, memo, runGit = defaultRunGit) {
  if (head === ref) return { covered: true, unresolved: false };
  const key = memo ? coveredByKey(head, ref) : null;
  if (memo && memo.has(key)) return memo.get(key);
  const result = runGit(root, ['merge-base', '--is-ancestor', head, ref]);
  let value;
  if (result.status === 0) value = { covered: true, unresolved: false };
  else if (result.status === 1) value = { covered: false, unresolved: false };
  else value = { covered: null, unresolved: true }; // null or e.g. exit 128 — unknown/invalid revision
  if (memo) memo.set(key, value);
  return value;
}

/**
 * Commits reachable from HEAD but not from `ref` in `root`. Returns
 * { count, unresolved } — unresolved (never throws) covers a missing git
 * binary, an unresolvable ref, or non-numeric output.
 */
function commitsSince(root, ref, memo, runGit = defaultRunGit) {
  const key = memo ? commitsSinceKey(ref) : null;
  if (memo && memo.has(key)) return memo.get(key);
  const result = runGit(root, ['rev-list', `${ref}..HEAD`, '--count']);
  let value;
  if (result.status !== 0) {
    value = { count: null, unresolved: true };
  } else {
    const count = parseInt(String(result.stdout).trim(), 10);
    value = Number.isFinite(count) ? { count, unresolved: false } : { count: null, unresolved: true };
  }
  if (memo) memo.set(key, value);
  return value;
}

/**
 * Derive a candidate's review status at read time — NEVER stored (R6/R10).
 * `candidate` carries at least { feature, head, cells }. `opts.sessions`
 * lets a caller iterating many candidates (e.g. a status summary) pass a
 * pre-fetched listReviews(root) once instead of re-reading per candidate.
 * `opts.gitMemo` (D2, cli-performance CONTEXT): an optional pass-local Map,
 * born once in the caller's candidate loop and passed to every
 * deriveCandidateStatus call in that pass, so repeated (head,ref)/(ref) git
 * questions across candidates sharing a covering session are answered once.
 * Never persisted beyond the pass, never TTL'd. `opts.runGit` is a test-only
 * injection seam for the underlying git runner — production callers never
 * set it.
 *
 * Priority: any covering session still open (pending/blocked) -> 'in review'
 * (an active session always outranks a stale older approval). Otherwise, the
 * first covering approved session whose head is an ancestor-or-equal of the
 * candidate's head decides reviewed vs. stale (git rev-list count since that
 * session's head). A covering session whose ancestry can't be resolved
 * degrades the candidate to 'review stale' with a 'range unresolvable' note
 * rather than throwing or silently reporting 'reviewed'. No covering session
 * at all (legacy feature or genuinely new work) -> 'unreviewed' — no fake
 * session records are ever fabricated (SPEC §11.3).
 */
export function deriveCandidateStatus(root, candidate, opts = {}) {
  const sessions = Array.isArray(opts.sessions) ? opts.sessions : listReviews(root);
  const gitMemo = opts.gitMemo;
  const runGit = opts.runGit;
  const covering = sessions.filter((s) => sessionCoversCandidate(s, candidate));

  const open = covering.filter(isSessionOpen);
  if (open.length > 0) {
    const session = open[open.length - 1];
    return { status: 'in review', session: session.id };
  }

  const approved = covering.filter((s) => !isSessionOpen(s));
  let unresolvedSession = null;
  for (const session of approved) {
    const coverage = headCoveredBy(root, candidate.head, session.head, gitMemo, runGit);
    if (coverage.unresolved) {
      unresolvedSession = unresolvedSession || session;
      continue;
    }
    if (!coverage.covered) continue; // candidate's work postdates this session's frozen head — not this session's coverage
    const since = commitsSince(root, session.head, gitMemo, runGit);
    if (since.unresolved) {
      return { status: 'review stale', session: session.id, note: 'range unresolvable' };
    }
    if (since.count > 0) {
      return { status: 'review stale', session: session.id };
    }
    return { status: 'reviewed', session: session.id };
  }
  if (unresolvedSession) {
    return { status: 'review stale', session: unresolvedSession.id, note: 'range unresolvable' };
  }
  return { status: 'unreviewed' };
}
