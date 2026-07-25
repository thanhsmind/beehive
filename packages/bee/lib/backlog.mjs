// backlog.mjs — parser + mechanical passes for docs/backlog.md (the product-backlog
// layer, D6), PLUS (backlog-unification D1-D5) the event-sourced PBI layer that
// now backs it: PBIs are `kind:'pbi'` records appended to the SAME
// .bee/backlog.jsonl stream the friction/proposal rows already live in
// (`bee backlog add`) — current PBI state is derived by folding those events
// (foldPbis), never hand-edited. The legacy docs/backlog.md TABLE parser below
// (readBacklogCounts/rankBacklog/findDuplicateBacklogIds/featureBacklogRank)
// stays intact for repos that have not migrated yet (no kind:'pbi' events on
// file): every one of those four re-derives from the fold FIRST and only falls
// back to the legacy table parse when foldPbis(root).hasEvents is false — see
// each function's own comment. Migration into the event stream is a separate
// feature step (backlog-unification bu-2); this file supports both shapes at
// once so a host repo's existing table keeps working unmigrated.
//
// Status TRANSITIONS on the LEGACY table stay prose-ruled (per D7) and are
// never written here; the two mechanical passes below (P2 rank = reorder rows
// by status group, P3 badges = render counts into README markers) change no
// row's content. `bee backlog rank --write` is retired at the CLI layer
// (bee.mjs) in favor of the new generated view (renderBacklogPbiView /
// `bee backlog render`) — the rankBacklog() function itself stays, since it is
// still the one parser findDuplicateBacklogIds and the legacy-table branch of
// featureBacklogRank walk.
// One parser, shared by bee_status and the session preamble.

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { resolveProductRoot } from './state.mjs';

// D6: the fixed status enum, priority-ordered. Exported so bee_status, the
// preamble, and the drift guard all read one source of truth. This is the
// LEGACY docs/backlog.md table's 3-value enum — kept unchanged (migration is
// bu-2's job). The event-sourced PBI layer's 5-value enum is PBI_STATUSES,
// below.
export const BACKLOG_STATUSES = ['proposed', 'in-flight', 'done'];

// ─── event-sourced PBIs (backlog-unification D1-D5) ─────────────────────────
// PBIs live as {ts, kind:'pbi', event:'add'|'status'|'amend', id, title?,
// cos?, status?, feature?} records appended to .bee/backlog.jsonl — the SAME
// stream `bee backlog add` (friction/proposal rows) already writes, never a
// second store. Current state is always a FOLD over that stream
// (last-event-wins per field); nothing here ever hand-edits a row in place.

// D4: the real status enum (measured from the legacy table's actual live
// values, including the two statuses the old 3-value BACKLOG_STATUSES never
// recognized: 'parked' and 'declined').
export const PBI_STATUSES = ['proposed', 'in-flight', 'parked', 'done', 'declined'];

// Priority weight shared by featureBacklogRank's fold branch and the render's
// row ordering: in-flight first (active work), then proposed, then parked,
// with done/declined lowest (history sinks) — same spirit as the legacy
// table's RANK_WEIGHT, extended to all five statuses so nothing ties.
const PBI_RANK_WEIGHT = { 'in-flight': 0, proposed: 1, parked: 2, done: 3, declined: 4 };
const PBI_RANK_UNKNOWN_WEIGHT = 5;

function backlogJsonlPath(root) {
  return path.join(root, '.bee', 'backlog.jsonl');
}

// Bare fs.appendFileSync, no lock — the SAME no-coordination pattern
// bee.mjs's existing `backlog add` (friction/proposal) verb already uses to
// append to this exact file. D2: id generation is collision-free by
// construction (crypto randomness), so no lock is needed for correctness;
// the fold (below) is the defensive backstop against a genuine collision.
function appendPbiEvent(root, event) {
  const file = backlogJsonlPath(root);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.appendFileSync(file, `${JSON.stringify(event)}\n`, 'utf8');
}

function generatePbiId() {
  return `p-${crypto.randomBytes(4).toString('hex')}`;
}

/**
 * Fold every kind:'pbi' event in .bee/backlog.jsonl into current PBI state —
 * last-event-wins per field (D1). A malformed line (bad JSON, non-object, no
 * usable id) is skipped, never thrown. A second 'add' event for an id already
 * present is REFUSED defensively: the first add wins and the later one is
 * ignored (D2) — the append-only log can't be un-appended, so this is where a
 * stray duplicate (e.g. two racing writers who both generated the same id, or
 * a bad --id migration override) is neutralized; addPbi below is the primary
 * guard and throws before ever reaching the log, so this path is a backstop,
 * not the normal case.
 * @param {string} root
 * @returns {{items: Map<string, {id:string, title:string, cos:string,
 *   status:string, feature:string|null}>, hasEvents: boolean}} hasEvents is
 *   true the moment ANY kind:'pbi' row is seen (even a malformed one) — it is
 *   the fold-first/legacy-fallback gate every other exported reader below
 *   checks.
 */
export function foldPbis(root) {
  const items = new Map();
  let hasEvents = false;
  let text;
  try {
    text = fs.readFileSync(backlogJsonlPath(root), 'utf8');
  } catch {
    return { items, hasEvents };
  }
  for (const line of text.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    let row;
    try {
      row = JSON.parse(trimmed);
    } catch {
      continue;
    }
    if (!row || typeof row !== 'object' || row.kind !== 'pbi') continue;
    hasEvents = true;
    const id = typeof row.id === 'string' && row.id ? row.id : null;
    if (!id) continue;
    if (row.event === 'add') {
      if (items.has(id)) continue; // duplicate add refused — first add wins
      items.set(id, {
        id,
        title: typeof row.title === 'string' ? row.title : '',
        cos: typeof row.cos === 'string' ? row.cos : '',
        status: PBI_STATUSES.includes(row.status) ? row.status : 'proposed',
        feature: typeof row.feature === 'string' && row.feature ? row.feature : null,
      });
    } else if (row.event === 'status') {
      const item = items.get(id);
      if (!item) continue; // a status event for an unknown id is a no-op fold
      if (PBI_STATUSES.includes(row.status)) item.status = row.status;
      if (typeof row.feature === 'string' && row.feature) item.feature = row.feature;
    } else if (row.event === 'amend') {
      const item = items.get(id);
      if (!item) continue;
      if (typeof row.title === 'string' && row.title) item.title = row.title;
      if (typeof row.cos === 'string' && row.cos) item.cos = row.cos;
    }
  }
  return { items, hasEvents };
}

/**
 * Append a new PBI 'add' event. Generates a collision-free `p-<8hex>` id via
 * crypto randomness (no lock, no read-then-increment — D2) unless an explicit
 * id is given (`--id`, migration-only: preserves a legacy P<n> id verbatim).
 * Refuses up front (throws) when the id — generated or given — already names
 * an item in the current fold, so an operator sees an error immediately
 * rather than a silently-ignored duplicate landing in the log.
 * @returns {{id:string, title:string, cos:string, status:string,
 *   feature:string|null}} the created item
 */
export function addPbi(root, { id, title, cos = '', status = 'proposed', feature } = {}) {
  if (typeof title !== 'string' || !title.trim()) {
    throw new Error('pbi add: --title is required and must be non-empty.');
  }
  if (!PBI_STATUSES.includes(status)) {
    throw new Error(`pbi add: invalid --status "${status}". FIX: use one of ${PBI_STATUSES.join(', ')}.`);
  }
  const { items } = foldPbis(root);
  let finalId = typeof id === 'string' && id.trim() ? id.trim() : null;
  if (finalId) {
    if (items.has(finalId)) {
      throw new Error(
        `pbi add: id "${finalId}" already exists — duplicate add refused. FIX: use "bee backlog pbi amend --id ${finalId}" or "bee backlog pbi status --id ${finalId} --to <status>" instead.`,
      );
    }
  } else {
    do {
      finalId = generatePbiId();
    } while (items.has(finalId));
  }
  const trimmedCos = typeof cos === 'string' ? cos.trim() : '';
  const trimmedFeature = typeof feature === 'string' ? feature.trim() : '';
  const event = {
    ts: new Date().toISOString(),
    kind: 'pbi',
    event: 'add',
    id: finalId,
    title: title.trim(),
    status,
  };
  if (trimmedCos) event.cos = trimmedCos;
  if (trimmedFeature) event.feature = trimmedFeature;
  appendPbiEvent(root, event);
  return { id: finalId, title: event.title, cos: trimmedCos, status, feature: trimmedFeature || null };
}

/**
 * Append a 'status' event, flipping a PBI's status (and optionally stamping
 * its feature slug in the same move — the exploring-D11a flip needs both at
 * once). Refuses an unknown id or an out-of-enum --to, throwing before
 * anything is appended.
 */
export function setPbiStatus(root, { id, to, feature } = {}) {
  if (typeof id !== 'string' || !id.trim()) {
    throw new Error('pbi status: --id is required.');
  }
  if (!PBI_STATUSES.includes(to)) {
    throw new Error(`pbi status: invalid --to "${to}". FIX: use one of ${PBI_STATUSES.join(', ')}.`);
  }
  const trimmedId = id.trim();
  const { items } = foldPbis(root);
  const current = items.get(trimmedId);
  if (!current) {
    throw new Error(`pbi status: unknown id "${trimmedId}". FIX: check "bee backlog pbi list --json" for valid ids.`);
  }
  const trimmedFeature = typeof feature === 'string' ? feature.trim() : '';
  const event = { ts: new Date().toISOString(), kind: 'pbi', event: 'status', id: trimmedId, status: to };
  if (trimmedFeature) event.feature = trimmedFeature;
  appendPbiEvent(root, event);
  return { ...current, status: to, feature: trimmedFeature || current.feature };
}

/**
 * Append an 'amend' event, updating title and/or cos. At least one of the two
 * is required; status/feature never move through amend (that is status's
 * job). Refuses an unknown id.
 */
export function amendPbi(root, { id, title, cos } = {}) {
  if (typeof id !== 'string' || !id.trim()) {
    throw new Error('pbi amend: --id is required.');
  }
  const trimmedId = id.trim();
  const hasTitle = typeof title === 'string' && title.trim().length > 0;
  const hasCos = typeof cos === 'string' && cos.trim().length > 0;
  if (!hasTitle && !hasCos) {
    throw new Error('pbi amend: at least one of --title or --cos is required.');
  }
  const { items } = foldPbis(root);
  const current = items.get(trimmedId);
  if (!current) {
    throw new Error(`pbi amend: unknown id "${trimmedId}". FIX: check "bee backlog pbi list --json" for valid ids.`);
  }
  const event = { ts: new Date().toISOString(), kind: 'pbi', event: 'amend', id: trimmedId };
  if (hasTitle) event.title = title.trim();
  if (hasCos) event.cos = cos.trim();
  appendPbiEvent(root, event);
  return { ...current, ...(hasTitle ? { title: event.title } : {}), ...(hasCos ? { cos: event.cos } : {}) };
}

/**
 * The fold as a query — the token-cheap read path (D3). Optionally filtered
 * to one status (throws on an out-of-enum filter, same as the writers).
 * Id-sorted for determinism.
 */
export function listPbis(root, { status } = {}) {
  const { items } = foldPbis(root);
  let list = [...items.values()];
  if (typeof status === 'string' && status) {
    if (!PBI_STATUSES.includes(status)) {
      throw new Error(`pbi list: invalid --status "${status}". FIX: use one of ${PBI_STATUSES.join(', ')}.`);
    }
    list = list.filter((item) => item.status === status);
  }
  list.sort((a, b) => a.id.localeCompare(b.id));
  return list;
}

// ─── render: the generated docs/backlog.md view (D3/D5) ─────────────────────
// Same generated-header idiom as knowledge.mjs's KNOWLEDGE_INDEX_HEADER
// (knowledge.mjs:1100) — an HTML comment, command strings swapped to `bee
// backlog render`, deliberately no generation timestamp or other wall-clock
// value so two consecutive renders over the same backlog.jsonl are
// byte-identical.

const BACKLOG_RENDER_HEADER = [
  '<!--',
  'GENERATED FILE — do not hand-edit.',
  "Rendered by `bee backlog render` from event-sourced PBI records in .bee/backlog.jsonl (backlog-unification D1/D3).",
  'Regenerate: `bee backlog render --write`. Check freshness: `bee backlog render --check`.',
  'Deterministic: byte-identical for the same backlog.jsonl contents — status-grouped, id-sorted entries, LF endings,',
  'never a generation timestamp or any other wall-clock value.',
  '-->',
].join('\n');

// proposed/in-flight/parked render as full table rows; done/declined collapse
// to one-line links so the view stays short forever (D5).
const RENDER_COLLAPSED_STATUSES = new Set(['done', 'declined']);

function escapeCell(value) {
  return String(value || '')
    .replace(/\r?\n/g, ' ')
    .replace(/\|/g, '\\|')
    .trim();
}

/**
 * Pure function: compute the generated docs/backlog.md content from the
 * current fold. Deterministic — status-weight then id-sorted, LF endings, no
 * timestamp; a repo with zero PBI events still renders a stable empty-table
 * shell (never throws, never omits the header).
 */
export function computeBacklogRenderContent(root) {
  const { items } = foldPbis(root);
  const all = [...items.values()].sort(
    (a, b) => (PBI_RANK_WEIGHT[a.status] ?? PBI_RANK_UNKNOWN_WEIGHT) - (PBI_RANK_WEIGHT[b.status] ?? PBI_RANK_UNKNOWN_WEIGHT) || a.id.localeCompare(b.id),
  );
  const fullRows = all.filter((item) => !RENDER_COLLAPSED_STATUSES.has(item.status));
  const collapsed = all.filter((item) => RENDER_COLLAPSED_STATUSES.has(item.status));

  const lines = ['# Product Backlog', '', BACKLOG_RENDER_HEADER, ''];
  lines.push('| ID | Story | CoS | Status | Feature |');
  lines.push('|----|-------|-----|--------|---------|');
  for (const item of fullRows) {
    lines.push(`| ${escapeCell(item.id)} | ${escapeCell(item.title)} | ${escapeCell(item.cos)} | ${item.status} | ${escapeCell(item.feature || '—')} |`);
  }

  if (collapsed.length > 0) {
    lines.push('', '## Done / Declined', '');
    for (const item of collapsed) {
      lines.push(`- [${escapeCell(item.id)}] ${escapeCell(item.title)} — ${item.status}`);
    }
  }

  return `${lines.join('\n')}\n`;
}

/**
 * Compute (and with `write: true` apply) the generated docs/backlog.md view.
 * @returns {{changed:boolean, content:string}} changed = the generated
 *   content differs from what is currently on disk (an absent file counts as
 *   changed too, same as every other generated-doc contract in this repo).
 */
export function renderBacklogPbiView(root, { write = false } = {}) {
  const content = computeBacklogRenderContent(root);
  const file = backlogPath(root);
  let existing = null;
  try {
    existing = fs.readFileSync(file, 'utf8');
  } catch {
    existing = null;
  }
  const changed = existing !== content;
  if (write) {
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, content, 'utf8');
  }
  return { changed, content };
}

// docs/backlog.md is a PRODUCT doc — it resolves against the product root, which
// equals the bee root for every ordinary repo but points at the nested product
// repo under the repo-divorce topology (GitHub #14).
function backlogPath(root) {
  return path.join(resolveProductRoot(root), 'docs', 'backlog.md');
}

// 'in-flight' -> 'inFlight'; the count-object key is derived from the token so
// the enum stays the single source of truth (no rival literal list).
function tokenKey(token) {
  return token.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
}

// Split a markdown table line into trimmed cells, dropping the empty edges that
// bordering pipes produce. Tolerant of rows written without outer pipes.
function splitRow(line) {
  const cells = line.split('|').map((cell) => cell.trim());
  if (cells.length && cells[0] === '') cells.shift();
  if (cells.length && cells[cells.length - 1] === '') cells.pop();
  return cells;
}

// Strip bold/italic/code markup and lowercase, preserving the hyphen in
// 'in-flight'. A separator row cell ('---') or a header cell ('Status') simply
// fails the enum match and is skipped — no special-casing needed.
function normalizeStatus(cell) {
  return cell.replace(/[*`_]/g, '').trim().toLowerCase();
}

/**
 * Fold-derived counts — one entry per PBI_STATUSES value, keyed via
 * tokenKey, plus total. Only called once foldPbis(root).hasEvents is true.
 */
function foldedBacklogCounts(items) {
  const counts = {};
  for (const status of PBI_STATUSES) counts[tokenKey(status)] = 0;
  for (const item of items.values()) {
    if (PBI_STATUSES.includes(item.status)) counts[tokenKey(item.status)] += 1;
  }
  const total = PBI_STATUSES.reduce((sum, status) => sum + counts[tokenKey(status)], 0);
  return { ...counts, total };
}

/**
 * Legacy docs/backlog.md TABLE parser: count rows by their Status column.
 * Only reached when foldPbis(root).hasEvents is false (pre-migration repos —
 * backlog-unification D3's fold-first/legacy-fallback rule).
 * @returns {{proposed:number, inFlight:number, done:number, total:number}|null}
 *   null only when the file is absent/unreadable; a present-but-tableless file
 *   returns zeroed counts (the file's existence is what gates the preamble line).
 */
function legacyBacklogCounts(root) {
  let text;
  try {
    text = fs.readFileSync(backlogPath(root), 'utf8');
  } catch {
    return null;
  }

  const counts = {};
  for (const status of BACKLOG_STATUSES) counts[tokenKey(status)] = 0;

  const lines = text.split(/\r?\n/);
  let statusIndex = -1;
  for (let i = 0; i < lines.length; i += 1) {
    if (!lines[i].includes('|')) continue;
    const cells = splitRow(lines[i]);
    if (statusIndex === -1) {
      // The header row is the first table row carrying a 'Status' column.
      const idx = cells.findIndex((cell) => normalizeStatus(cell) === 'status');
      if (idx !== -1) statusIndex = idx;
      continue;
    }
    // A row missing the Status column (malformed / too few cells) is skipped.
    if (cells.length <= statusIndex) continue;
    const token = normalizeStatus(cells[statusIndex]);
    if (BACKLOG_STATUSES.includes(token)) counts[tokenKey(token)] += 1;
  }

  const total = BACKLOG_STATUSES.reduce((sum, status) => sum + counts[tokenKey(status)], 0);
  return { ...counts, total };
}

/**
 * PBI counts (done/in-flight/proposed/parked/declined/total). Fold-first
 * (backlog-unification D3): once ANY kind:'pbi' event exists in
 * .bee/backlog.jsonl, counts derive from the fold exclusively — the legacy
 * docs/backlog.md table is only consulted for repos that have not migrated
 * at all yet.
 * @returns {object|null} null only when there is neither a fold nor a
 *   parseable legacy table (mirrors the pre-existing null contract).
 */
export function readBacklogCounts(root) {
  const folded = foldPbis(root);
  if (folded.hasEvents) return foldedBacklogCounts(folded.items);
  return legacyBacklogCounts(root);
}

// ─── P2: mechanical rank pass ───────────────────────────────────────────────
// Reorders the table's data rows by status group — in-flight first (active work
// on top), then proposed, then done (history sinks) — stable within each group
// so hand-ordering inside a group is preserved. Rows whose status is not in the
// enum keep a neutral weight between proposed and done. No cell is edited.

const RANK_WEIGHT = { 'in-flight': 0, proposed: 1, done: 3 };
const RANK_UNKNOWN_WEIGHT = 2;

// Shared row walk (i-1, docs/history/issues-46-53/CONTEXT.md D1): the one
// parse of docs/backlog.md's data rows that carries each row's ID cell, its
// exact line number, and its rank weight. rankBacklog below and
// findDuplicateBacklogIds further down both read from THIS walk — neither
// re-parses the file. Returns null when the file is absent or has no
// parseable table (no Status column found, no separator row, or zero data
// rows) — the same "nothing to check" cases the two callers already treat
// as null/empty.
function walkBacklogIdRows(root) {
  const file = backlogPath(root);
  let text;
  try {
    text = fs.readFileSync(file, 'utf8');
  } catch {
    return null;
  }

  const lines = text.split(/\r?\n/);
  let statusIndex = -1;
  let separatorLine = -1;
  const rows = [];
  for (let i = 0; i < lines.length; i += 1) {
    if (!lines[i].includes('|')) {
      // A non-table line after the table body ends the block.
      if (separatorLine !== -1 && rows.length > 0) break;
      continue;
    }
    const cells = splitRow(lines[i]);
    if (statusIndex === -1) {
      const idx = cells.findIndex((cell) => normalizeStatus(cell) === 'status');
      if (idx !== -1) {
        statusIndex = idx;
      }
      continue;
    }
    if (separatorLine === -1) {
      separatorLine = i; // the |---| row right after the header
      continue;
    }
    const token = cells.length > statusIndex ? normalizeStatus(cells[statusIndex]) : '';
    rows.push({
      line: lines[i],
      lineIndex: i, // 0-based index into `lines`; +1 for a human-facing line number
      id: cells[0] ? cells[0].replace(/[*`_]/g, '').trim() : '',
      weight: RANK_WEIGHT[token] !== undefined ? RANK_WEIGHT[token] : RANK_UNKNOWN_WEIGHT,
      position: rows.length,
    });
  }
  if (statusIndex === -1 || separatorLine === -1 || rows.length === 0) return null;

  return { file, lines, rows };
}

/**
 * Compute (and with `write: true` apply) the rank pass.
 * @returns {{changed:boolean, order:string[]}|null} null when the file is
 *   absent or has no parseable table; `order` lists the first cell (ID) of each
 *   data row in ranked order.
 */
export function rankBacklog(root, { write = false } = {}) {
  const walked = walkBacklogIdRows(root);
  if (!walked) return null;
  const { file, lines, rows } = walked;

  const ranked = [...rows].sort(
    (a, b) => a.weight - b.weight || a.position - b.position,
  );
  const changed = ranked.some((row, i) => row !== rows[i]);
  const order = ranked.map((row) => row.id);

  if (write && changed) {
    // Write ranked lines back into the rows' original line slots, so any
    // surrounding non-table content is untouched even if rows are not contiguous.
    const slots = rows.map((row) => row.lineIndex);
    for (let i = 0; i < ranked.length; i += 1) {
      lines[slots[i]] = ranked[i].line;
    }
    fs.writeFileSync(file, lines.join('\n'), 'utf8');
  }
  return { changed, order };
}

// ─── uniqueness check (i-1, docs/history/issues-46-53/CONTEXT.md D1) ───────
// #49 reported "ids allocated max+1, concurrent readers collide" — at the
// time, there was no allocator anywhere in this repo; the id rule lived only
// in prose (an agent computing the sequential integer that came after every
// id already on the table, then hand-editing the row in) and the committed
// duplicate (two `P50` rows, authored a day apart on different branches)
// proved it was never a race: each author computed "next" from a snapshot
// that predated the other's commit. A lock would have prevented nothing —
// what was missing was a CHECK. This is that check. backlog-unification D2
// later closed the underlying gap for real: new PBI ids are now generated
// (`p-<8hex>`, crypto randomness, no read-then-increment, no lock needed),
// so this check now also guards the legacy `P<n>` ids the migration
// preserved verbatim.
/**
 * Find IDs that appear on more than one data row of docs/backlog.md.
 * @returns {{id:string, lines:number[]}[]} one entry per duplicated,
 *   non-empty id, each carrying the 1-based line numbers (in file order) of
 *   every row that carries it. Empty when the file is absent, has no
 *   parseable table, or every id is unique.
 */
export function findDuplicateBacklogIds(root) {
  const walked = walkBacklogIdRows(root);
  if (!walked) return [];

  const byId = new Map();
  for (const row of walked.rows) {
    if (!row.id) continue; // a blank ID cell never collides with anything
    if (!byId.has(row.id)) byId.set(row.id, []);
    byId.get(row.id).push(row.lineIndex + 1); // 1-based, human-facing
  }

  const duplicates = [];
  for (const [id, lineNumbers] of byId) {
    if (lineNumbers.length > 1) duplicates.push({ id, lines: lineNumbers });
  }
  return duplicates;
}

// ─── featureBacklogRank: Feature-column rank (fresh-session-handoff fsh-11,
// D2 cross-lane ordering) ────────────────────────────────────────────────────
// rankBacklog above reorders rows by Status and returns the ID-column order —
// it never reads the Feature column at all. claim-next's cross-lane pull
// needs the OPPOSITE lookup: "where does lane/feature X rank in the backlog",
// keyed by feature, not by row id. This walks the same table shape (status-
// grouped weight, stable within a group — the exact RANK_WEIGHT ordering
// rankBacklog itself uses) but captures the Feature column per row instead.
//
// A row whose Feature cell is missing, blank, or the placeholder "—"/"-"
// contributes no mapping — it never claims a feature slug. When two rows
// name the SAME feature, the row closest to rank 0 (its best-ranked
// occurrence) wins, so a feature with both an in-flight and a done row ranks
// at the in-flight row's position.
//
// @returns {Map<string, number>} feature slug -> rank position (0 = highest
//   priority). Empty when the file is absent or has no parseable table with a
//   Feature column — callers treat a missing entry as "unranked" (sorts last
//   alongside every other unranked feature, callers' tie-break decides ties).
//
// Fold-first (backlog-unification D3): once ANY kind:'pbi' event exists in
// .bee/backlog.jsonl, the mapping derives from the fold exclusively (weighted
// by PBI_RANK_WEIGHT over all five statuses) — the legacy docs/backlog.md
// table below is only consulted pre-migration.
export function featureBacklogRank(root) {
  const folded = foldPbis(root);
  if (folded.hasEvents) {
    const rows = [...folded.items.values()]
      .map((item) => ({
        feature: item.feature || null,
        weight: PBI_RANK_WEIGHT[item.status] !== undefined ? PBI_RANK_WEIGHT[item.status] : PBI_RANK_UNKNOWN_WEIGHT,
        id: item.id,
      }))
      .sort((a, b) => a.weight - b.weight || a.id.localeCompare(b.id));
    const map = new Map();
    rows.forEach((row, rank) => {
      if (row.feature && !map.has(row.feature)) map.set(row.feature, rank);
    });
    return map;
  }

  const file = backlogPath(root);
  let text;
  try {
    text = fs.readFileSync(file, 'utf8');
  } catch {
    return new Map();
  }

  const lines = text.split(/\r?\n/);
  let statusIndex = -1;
  let featureIndex = -1;
  let separatorLine = -1;
  const rows = [];
  for (let i = 0; i < lines.length; i += 1) {
    if (!lines[i].includes('|')) {
      if (separatorLine !== -1 && rows.length > 0) break;
      continue;
    }
    const cells = splitRow(lines[i]);
    if (statusIndex === -1) {
      const idx = cells.findIndex((cell) => normalizeStatus(cell) === 'status');
      if (idx !== -1) {
        statusIndex = idx;
        featureIndex = cells.findIndex((cell) => normalizeStatus(cell) === 'feature');
      }
      continue;
    }
    if (separatorLine === -1) {
      separatorLine = i; // the |---| row right after the header
      continue;
    }
    const token = cells.length > statusIndex ? normalizeStatus(cells[statusIndex]) : '';
    const rawFeature = featureIndex !== -1 && cells.length > featureIndex ? cells[featureIndex] : '';
    const feature = rawFeature.replace(/[*`_]/g, '').trim();
    rows.push({
      feature: feature && feature !== '—' && feature !== '-' ? feature : null,
      weight: RANK_WEIGHT[token] !== undefined ? RANK_WEIGHT[token] : RANK_UNKNOWN_WEIGHT,
      position: rows.length,
    });
  }
  if (statusIndex === -1 || featureIndex === -1 || separatorLine === -1 || rows.length === 0) {
    return new Map();
  }

  const ranked = [...rows].sort((a, b) => a.weight - b.weight || a.position - b.position);
  const map = new Map();
  ranked.forEach((row, rank) => {
    if (row.feature && !map.has(row.feature)) map.set(row.feature, rank);
  });
  return map;
}

// ─── P3: README badges ──────────────────────────────────────────────────────
// Renders the counts as shields.io static badges between BEE markers in
// README.md. Idempotent; creates the marker block after the first heading when
// absent. Counts-only — no row content leaves the backlog file.

export const BADGE_MARKER_START = '<!-- BEE:BACKLOG-BADGES:START -->';
export const BADGE_MARKER_END = '<!-- BEE:BACKLOG-BADGES:END -->';

// Superset of both enums (D4: badges/counts/preamble include every value) —
// harmless for the legacy 3-status branch, which only ever looks up its own
// three keys.
const BADGE_COLORS = { done: 'brightgreen', 'in-flight': 'blue', proposed: 'lightgrey', parked: 'yellow', declined: 'red' };

function shieldsEscape(text) {
  // shields.io static badges: '-' doubles, '_' doubles, space becomes '%20'.
  return String(text).replace(/-/g, '--').replace(/_/g, '__').replace(/ /g, '%20');
}

/**
 * Fold-first (backlog-unification D3/D4): once ANY kind:'pbi' event exists,
 * badges cover all five PBI_STATUSES; a pre-migration repo keeps the legacy
 * three-status badge set. readBacklogCounts already made the same fold-first
 * decision internally, so counts and the status set here can never disagree.
 */
export function renderBacklogBadges(root) {
  const counts = readBacklogCounts(root);
  if (!counts) return null;
  const statuses = foldPbis(root).hasEvents ? PBI_STATUSES : BACKLOG_STATUSES;
  const badges = statuses
    .slice()
    .reverse() // done first — the headline number
    .map((status) => {
      const label = shieldsEscape(`backlog ${status}`);
      const value = counts[tokenKey(status)] || 0;
      return `![backlog ${status}](https://img.shields.io/badge/${label}-${value}-${BADGE_COLORS[status]})`;
    });
  return badges.join(' ');
}

/**
 * Insert or refresh the badge block in README.md.
 * @returns {{changed:boolean, badges:string}|null} null when README.md or the
 *   backlog is absent.
 */
export function updateReadmeBadges(root, { write = false } = {}) {
  const badges = renderBacklogBadges(root);
  if (badges == null) return null;
  // The backlog badges belong in the PRODUCT README (same root as docs/backlog.md).
  const file = path.join(resolveProductRoot(root), 'README.md');
  let text;
  try {
    text = fs.readFileSync(file, 'utf8');
  } catch {
    return null;
  }

  const block = `${BADGE_MARKER_START}\n${badges}\n${BADGE_MARKER_END}`;
  let next;
  if (text.includes(BADGE_MARKER_START) && text.includes(BADGE_MARKER_END)) {
    const pattern = new RegExp(
      `${BADGE_MARKER_START.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}[\\s\\S]*?${BADGE_MARKER_END.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`,
    );
    next = text.replace(pattern, block);
  } else {
    // No markers yet: place the block right under the first heading line.
    const lines = text.split(/\r?\n/);
    const headingIdx = lines.findIndex((line) => line.startsWith('#'));
    const at = headingIdx === -1 ? 0 : headingIdx + 1;
    lines.splice(at, 0, '', block);
    next = lines.join('\n');
  }

  const changed = next !== text;
  if (write && changed) fs.writeFileSync(file, next, 'utf8');
  return { changed, badges };
}
