#!/usr/bin/env node
// test_decisions_propagation.mjs — decision-propagation dp-1 (CONTEXT D4a):
// structured recall for the decisions store. Covers: logDecision optional
// tags[] (lowercase-slug validated, additive/zero-migration), `decisions
// search` gaining --tag/--scope(alias --area)/--since composable with
// --text (--text becomes optional once a structured filter is present; zero
// filters still refuses), legacy metadata-less events (still text-searchable,
// excluded once a --tag filter is applied), and `decisions active` gaining
// the same filter set. Same PASS/FAIL/exit-1 contract as every other suite
// here — see scripts/lib/test-fixture.mjs.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { runModuleWorker } from '../../../scripts/lib/run-module-worker.mjs';
import {
  makeTempRepo,
  check,
  assert,
  assertThrows,
  printSummaryAndExit,
} from '../../../scripts/lib/test-fixture.mjs';
import {
  logDecision,
  activeDecisions,
  supersedeDecision,
  redactDecision,
  archiveDecisions,
  tagDecision,
  tagDecisionsBatch,
  DECISIONS_LOCK_NAME,
  renderDecisionIndex,
  decisionIndexDrift,
} from '../lib/decisions.mjs';
import { appendJsonl, readJsonl } from '../lib/fsutil.mjs';
import { pendingCaptureStubs } from '../lib/capture.mjs';
import { acquireStoreLockOnceSync } from '../lib/lock.mjs';

const beeMjsModulePath = fileURLToPath(new URL('../bee.mjs', import.meta.url));

function runBee(args, cwd, input) {
  return runModuleWorker(beeMjsModulePath, { args, cwd, input });
}

function decisionsFilePath(root) {
  return path.join(root, '.bee', 'decisions.jsonl');
}

// Test-only helper: rewrite one event's `date` in the raw JSONL store so
// --since filtering can be asserted deterministically instead of racing the
// real clock.
function setEventDate(root, id, isoDate) {
  const file = decisionsFilePath(root);
  const lines = fs.readFileSync(file, 'utf8').split(/\r?\n/).filter((l) => l.trim());
  const rewritten = lines.map((line) => {
    const obj = JSON.parse(line);
    if (obj.id === id) obj.date = isoDate;
    return JSON.stringify(obj);
  });
  fs.writeFileSync(file, `${rewritten.join('\n')}\n`, 'utf8');
}

const root = makeTempRepo();

// ─── logDecision: optional tags[], lowercase-slug validated, additive ──────

await check('logDecision: tags omitted -> event carries no tags key at all (zero-migration parity)', async () => {
  const event = logDecision(root, { decision: 'Use SQLite for the queue', rationale: 'zero ops' });
  assert(!('tags' in event), `expected no tags key on a tagless decide event, got ${JSON.stringify(event)}`);
});

await check('logDecision: tags[] round-trips as a lowercase-slug array on the event', async () => {
  const event = logDecision(root, {
    decision: 'Use structured recall for decisions',
    rationale: 'GH #32',
    tags: ['recall', 'decisions-store'],
  });
  assert(Array.isArray(event.tags), `expected event.tags array, got ${JSON.stringify(event.tags)}`);
  assert(
    event.tags.join(',') === 'recall,decisions-store',
    `expected tags to round-trip verbatim, got ${JSON.stringify(event.tags)}`,
  );
});

await check('logDecision: rejects a non-array tags value and an invalid (non-lowercase-slug) tag entry', async () => {
  assertThrows(
    () => logDecision(root, { decision: 'x', rationale: 'y', tags: 'not-an-array' }),
    'tag',
    'non-array tags rejected',
  );
  assertThrows(
    () => logDecision(root, { decision: 'x', rationale: 'y', tags: ['Not-Lowercase'] }),
    'tag',
    'uppercase tag entry rejected',
  );
  assertThrows(
    () => logDecision(root, { decision: 'x', rationale: 'y', tags: ['has space'] }),
    'tag',
    'tag with a space rejected',
  );
});

// ─── CLI: decisions log --tags round-trips ─────────────────────────────────

await check('CLI: decisions log --tags a,b round-trips tags on the logged event', async () => {
  const run = await runBee(
    [
      'decisions',
      'log',
      '--decision',
      'Adopt tag-scoped recall',
      '--rationale',
      'GH #32 recall at scale',
      '--tags',
      'alpha,beta',
      '--scope',
      'decision-propagation',
      '--json',
    ],
    root,
  );
  assert(run.status === 0, `CLI log exited ${run.status} :: ${run.stderr || run.stdout}`);
  const event = JSON.parse(run.stdout);
  assert(event.tags.join(',') === 'alpha,beta', `expected tags alpha,beta round-tripped, got ${JSON.stringify(event.tags)}`);
  assert(event.scope === 'decision-propagation', `expected scope to round-trip, got ${event.scope}`);
});

await check('CLI: decisions log --tags rejects an invalid slug with a non-zero exit and a tag-naming error', async () => {
  const run = await runBee(
    ['decisions', 'log', '--decision', 'x', '--rationale', 'y', '--tags', 'Bad_Tag', '--json'],
    root,
  );
  assert(run.status !== 0, 'invalid tag exits non-zero');
  const payload = JSON.parse(run.stdout);
  assert(/tag/i.test(payload.error || ''), `error should name the tag problem, got ${JSON.stringify(payload)}`);
});

// ─── seed a scoped/tagged corpus for search/active filter tests ───────────

const billingA = logDecision(root, {
  decision: 'Reconcile billing invoices nightly',
  rationale: 'accuracy over latency',
  tags: ['billing', 'nightly-job'],
  scope: 'billing',
});
const billingB = logDecision(root, {
  decision: 'Retry failed billing webhooks with backoff',
  rationale: 'avoid thundering herd',
  tags: ['billing', 'webhooks'],
  scope: 'billing',
});
const authA = logDecision(root, {
  decision: 'Rotate auth tokens every 24h',
  rationale: 'reduce blast radius',
  tags: ['auth'],
  scope: 'auth',
});
// Legacy-shaped event: no tags field at all (matches the pre-dp-1 shape).
const legacy = logDecision(root, {
  decision: 'Use the legacy zenoflex queue for jobs',
  rationale: 'predates tag support',
});

// ─── decisions search: --tag / --scope / --area alias / --since, composed ──

await check('CLI: decisions search --tag matches exact (case-insensitive), excludes non-matching and legacy (tagless) events', async () => {
  const run = await runBee(['decisions', 'search', '--tag', 'BILLING', '--json'], root);
  assert(run.status === 0, `search --tag exited ${run.status} :: ${run.stderr || run.stdout}`);
  const { decisions } = JSON.parse(run.stdout);
  const ids = decisions.map((d) => d.id);
  assert(ids.includes(billingA.id) && ids.includes(billingB.id), 'both billing-tagged events matched');
  assert(!ids.includes(authA.id), 'auth-tagged event excluded');
  assert(!ids.includes(legacy.id), 'legacy (tagless) event excluded when --tag is given');
});

await check('CLI: decisions search --scope matches exact (case-insensitive); --area is an alias for --scope', async () => {
  const byScope = await runBee(['decisions', 'search', '--scope', 'BILLING', '--json'], root);
  assert(byScope.status === 0, `search --scope exited ${byScope.status} :: ${byScope.stderr || byScope.stdout}`);
  const scopeIds = JSON.parse(byScope.stdout).decisions.map((d) => d.id);
  assert(scopeIds.includes(billingA.id) && scopeIds.includes(billingB.id), 'both billing-scoped events matched via --scope');
  assert(!scopeIds.includes(authA.id), 'auth-scoped event excluded via --scope');

  const byArea = await runBee(['decisions', 'search', '--area', 'billing', '--json'], root);
  assert(byArea.status === 0, `search --area exited ${byArea.status} :: ${byArea.stderr || byArea.stdout}`);
  const areaIds = JSON.parse(byArea.stdout).decisions.map((d) => d.id).sort();
  assert(areaIds.join(',') === scopeIds.sort().join(','), '--area is an exact alias for --scope');
});

await check('CLI: decisions search --since is inclusive of the boundary date and excludes events strictly before it', async () => {
  setEventDate(root, billingA.id, '2026-01-10T00:00:00.000Z');
  setEventDate(root, billingB.id, '2026-01-15T00:00:00.000Z');

  const onBoundary = await runBee(['decisions', 'search', '--since', '2026-01-10', '--tag', 'billing', '--json'], root);
  assert(onBoundary.status === 0, `search --since exited ${onBoundary.status} :: ${onBoundary.stderr || onBoundary.stdout}`);
  const boundaryIds = JSON.parse(onBoundary.stdout).decisions.map((d) => d.id);
  assert(boundaryIds.includes(billingA.id), 'event dated exactly on --since is included (inclusive)');
  assert(boundaryIds.includes(billingB.id), 'later event still included');

  const afterA = await runBee(['decisions', 'search', '--since', '2026-01-11', '--tag', 'billing', '--json'], root);
  const afterIds = JSON.parse(afterA.stdout).decisions.map((d) => d.id);
  assert(!afterIds.includes(billingA.id), 'event strictly before --since is excluded');
  assert(afterIds.includes(billingB.id), 'event on/after --since still included');
});

await check('CLI: decisions search composes --tag + --scope + --text (AND, not OR)', async () => {
  const run = await runBee(
    ['decisions', 'search', '--tag', 'billing', '--scope', 'billing', '--text', 'webhook', '--json'],
    root,
  );
  assert(run.status === 0, `composed search exited ${run.status} :: ${run.stderr || run.stdout}`);
  const ids = JSON.parse(run.stdout).decisions.map((d) => d.id);
  assert(ids.length === 1 && ids[0] === billingB.id, `expected only the webhook billing event, got ${JSON.stringify(ids)}`);
});

await check('CLI: decisions search --text stays optional once a structured filter is present', async () => {
  const run = await runBee(['decisions', 'search', '--tag', 'auth', '--json'], root);
  assert(run.status === 0, `search --tag with no --text exited ${run.status} :: ${run.stderr || run.stdout}`);
  const ids = JSON.parse(run.stdout).decisions.map((d) => d.id);
  assert(ids.length === 1 && ids[0] === authA.id, `expected the auth event without --text, got ${JSON.stringify(ids)}`);
});

await check('CLI: decisions search with zero filters at all (no --text, no structured filter) still refuses', async () => {
  const run = await runBee(['decisions', 'search', '--json'], root);
  assert(run.status !== 0, 'search with no filters at all exits non-zero');
  const payload = JSON.parse(run.stdout);
  assert(typeof payload.error === 'string' && payload.error.length > 0, 'refusal carries an error message');
});

await check('CLI: decisions search --text alone (legacy call shape) still works and finds a legacy (tagless) event', async () => {
  const run = await runBee(['decisions', 'search', '--text', 'zenoflex', '--json'], root);
  assert(run.status === 0, `legacy --text-only search exited ${run.status} :: ${run.stderr || run.stdout}`);
  const ids = JSON.parse(run.stdout).decisions.map((d) => d.id);
  assert(ids.includes(legacy.id), 'legacy metadata-less event is still found by --text substring search');
});

// ─── sqs-b1 (D 5ca69717): --cell / --feature word-boundary text filters ───
// Cell ids/feature slugs have NO structural field on a decide event (gather
// Q1) — --cell/--feature are word-boundary matches over
// decision/rationale/alternatives, not a substring (that is the si-1≈si-10
// bug in plain --text). Proven both ways: "si-1" must exclude "si-10", and
// a hyphen-suffixed slug ("billing-export-v2") must not falsely match its
// hyphen-prefix ("billing-export") — a plain \b would let the latter through
// because "-" is itself a non-word boundary character.

const cellSi1 = logDecision(root, {
  decision: 'Cell si-1 renders the CLI manifest header',
  rationale: 'keeps --help --json in sync with the registry',
});
const cellSi10 = logDecision(root, {
  decision: 'Cell si-10 renders the CLI manifest footer',
  rationale: 'unrelated cell, deliberately digit-adjacent to its shorter sibling id',
});
const featureExport = logDecision(root, {
  decision: 'Adopt streaming export for the billing-export feature',
  rationale: 'avoids buffering the whole file in memory',
});
const featureExportV2 = logDecision(root, {
  decision: 'Adopt chunked upload for the billing-export-v2 feature',
  rationale: 'billing-export-v2 supersedes the streaming approach',
});

await check('CLI: decisions search --cell matches whole-token id, excludes a substring-colliding id (si-1 vs si-10)', async () => {
  const run = await runBee(['decisions', 'search', '--cell', 'si-1', '--json'], root);
  assert(run.status === 0, `search --cell exited ${run.status} :: ${run.stderr || run.stdout}`);
  const ids = JSON.parse(run.stdout).decisions.map((d) => d.id);
  assert(ids.includes(cellSi1.id), 'search --cell si-1 must match the si-1 event');
  assert(!ids.includes(cellSi10.id), 'search --cell si-1 must NOT match si-10 (substring collision)');
});

await check('CLI: decisions active --cell matches whole-token id, excludes a substring-colliding id (si-1 vs si-10)', async () => {
  const run = await runBee(['decisions', 'active', '--cell', 'si-1', '--json'], root);
  assert(run.status === 0, `active --cell exited ${run.status} :: ${run.stderr || run.stdout}`);
  const ids = JSON.parse(run.stdout).decisions.map((d) => d.id);
  assert(ids.includes(cellSi1.id), 'active --cell si-1 must match the si-1 event');
  assert(!ids.includes(cellSi10.id), 'active --cell si-1 must NOT match si-10 (substring collision)');
});

await check('CLI: decisions search --feature matches whole-token slug, excludes a hyphen-suffixed collision (billing-export vs billing-export-v2)', async () => {
  const run = await runBee(['decisions', 'search', '--feature', 'billing-export', '--json'], root);
  assert(run.status === 0, `search --feature exited ${run.status} :: ${run.stderr || run.stdout}`);
  const ids = JSON.parse(run.stdout).decisions.map((d) => d.id);
  assert(ids.includes(featureExport.id), 'search --feature billing-export must match the billing-export event');
  assert(!ids.includes(featureExportV2.id), 'search --feature billing-export must NOT match billing-export-v2 (hyphen-suffix collision)');
});

await check('CLI: decisions active --feature matches whole-token slug, excludes a hyphen-suffixed collision (billing-export vs billing-export-v2)', async () => {
  const run = await runBee(['decisions', 'active', '--feature', 'billing-export', '--json'], root);
  assert(run.status === 0, `active --feature exited ${run.status} :: ${run.stderr || run.stdout}`);
  const ids = JSON.parse(run.stdout).decisions.map((d) => d.id);
  assert(ids.includes(featureExport.id), 'active --feature billing-export must match the billing-export event');
  assert(!ids.includes(featureExportV2.id), 'active --feature billing-export must NOT match billing-export-v2 (hyphen-suffix collision)');
});

await check('CLI: decisions search --cell alone satisfies the "at least one filter" requirement (no --text needed)', async () => {
  const run = await runBee(['decisions', 'search', '--cell', 'si-10', '--json'], root);
  assert(run.status === 0, `search --cell alone exited ${run.status} :: ${run.stderr || run.stdout}`);
  const ids = JSON.parse(run.stdout).decisions.map((d) => d.id);
  assert(ids.length === 1 && ids[0] === cellSi10.id, `expected only the si-10 event, got ${JSON.stringify(ids)}`);
});

await check('CLI: decisions search --cell/--feature are advertised in --help --json for both active and search', async () => {
  const help = await runBee(['decisions', '--help', '--json'], root);
  assert(help.status === 0, `decisions --help --json exited ${help.status} :: ${help.stderr || help.stdout}`);
  const manifest = JSON.parse(help.stdout);
  const commands = Array.isArray(manifest) ? manifest : manifest.commands;
  const activeCmd = commands.find((c) => c.name === 'decisions.active');
  const searchCmd = commands.find((c) => c.name === 'decisions.search');
  assert(activeCmd && 'cell' in activeCmd.parameters.properties && 'feature' in activeCmd.parameters.properties, 'decisions.active advertises --cell/--feature');
  assert(searchCmd && 'cell' in searchCmd.parameters.properties && 'feature' in searchCmd.parameters.properties, 'decisions.search advertises --cell/--feature');
});

// ─── decisions active: same filter set as a deliberate sibling extension ──

await check('CLI: decisions active (no flags) is unchanged — lists every active decision, newest first', async () => {
  const run = await runBee(['decisions', 'active', '--json'], root);
  assert(run.status === 0, `active exited ${run.status} :: ${run.stderr || run.stdout}`);
  const { decisions } = JSON.parse(run.stdout);
  const ids = decisions.map((d) => d.id);
  assert(ids.includes(legacy.id) && ids.includes(authA.id), 'active still lists everything without filters');
});

await check('CLI: decisions active --tag/--scope/--since filter and compose exactly like search', async () => {
  const byTag = await runBee(['decisions', 'active', '--tag', 'billing', '--json'], root);
  assert(byTag.status === 0, `active --tag exited ${byTag.status} :: ${byTag.stderr || byTag.stdout}`);
  const tagIds = JSON.parse(byTag.stdout).decisions.map((d) => d.id);
  assert(tagIds.includes(billingA.id) && tagIds.includes(billingB.id), 'active --tag matches billing events');
  assert(!tagIds.includes(authA.id) && !tagIds.includes(legacy.id), 'active --tag excludes non-matching and legacy events');

  const byArea = await runBee(['decisions', 'active', '--area', 'auth', '--json'], root);
  const areaIds = JSON.parse(byArea.stdout).decisions.map((d) => d.id);
  assert(areaIds.length === 1 && areaIds[0] === authA.id, `active --area (alias) should isolate the auth event, got ${JSON.stringify(areaIds)}`);

  const bySince = await runBee(['decisions', 'active', '--tag', 'billing', '--since', '2026-01-11', '--json'], root);
  const sinceIds = JSON.parse(bySince.stdout).decisions.map((d) => d.id);
  assert(!sinceIds.includes(billingA.id) && sinceIds.includes(billingB.id), 'active composes --tag with --since');

  const withRecent = await runBee(['decisions', 'active', '--tag', 'billing', '--recent', '1', '--json'], root);
  const recentIds = JSON.parse(withRecent.stdout).decisions.map((d) => d.id);
  assert(recentIds.length === 1, `--recent applies after filtering, expected exactly 1, got ${JSON.stringify(recentIds)}`);
});

// ─── empty store: filters degrade to an empty result, never throw ─────────

await check('empty store: search/active with a structured filter on a fresh repo with no decisions.jsonl returns [] without throwing', async () => {
  const emptyRoot = makeTempRepo();
  try {
    assert(!fs.existsSync(decisionsFilePath(emptyRoot)), 'precondition: no decisions.jsonl exists yet');
    assert(activeDecisions(emptyRoot).length === 0, 'lib-level activeDecisions on an empty store is []');

    const search = await runBee(['decisions', 'search', '--tag', 'anything', '--json'], emptyRoot);
    assert(search.status === 0, `empty-store search --tag exited ${search.status} :: ${search.stderr || search.stdout}`);
    assert(JSON.parse(search.stdout).decisions.length === 0, 'empty-store search --tag returns an empty array');

    const active = await runBee(['decisions', 'active', '--scope', 'anything', '--json'], emptyRoot);
    assert(active.status === 0, `empty-store active --scope exited ${active.status} :: ${active.stderr || active.stdout}`);
    assert(JSON.parse(active.stdout).decisions.length === 0, 'empty-store active --scope returns an empty array');

    // Zero filters at all: active still fine (matches its unchanged no-flags
    // behavior), search still refuses (matches the zero-filters refusal rule).
    const activeNoFlags = await runBee(['decisions', 'active', '--json'], emptyRoot);
    assert(activeNoFlags.status === 0, 'empty-store active with no flags does not throw');
    assert(JSON.parse(activeNoFlags.stdout).decisions.length === 0, 'empty-store active with no flags is []');

    const searchNoFlags = await runBee(['decisions', 'search', '--json'], emptyRoot);
    assert(searchNoFlags.status !== 0, 'empty-store search with zero filters still refuses, same as a non-empty store');
  } finally {
    fs.rmSync(emptyRoot, { recursive: true, force: true });
  }
});

// ─── dp-2: supersede metadata inheritance + propagation sweep + capture
// stubs (CONTEXT D2/D6). Own temp repo so the docs/** sweep never sees the
// dp-1 corpus above, and never scans the real repo's docs/ tree. ──────────

const dpRoot = makeTempRepo();

await check('dp-2: supersede without --tags/--scope inherits both from the superseded target', async () => {
  const target = logDecision(dpRoot, {
    decision: 'Use queue A',
    rationale: 'perf',
    scope: 'billing',
    tags: ['queue', 'billing'],
  });
  const event = supersedeDecision(dpRoot, {
    supersedes: target.id,
    decision: 'Use queue B instead',
    rationale: 'queue A deprecated upstream',
  });
  assert(event.scope === 'billing', `expected inherited scope "billing", got ${event.scope}`);
  assert(
    Array.isArray(event.tags) && event.tags.join(',') === 'queue,billing',
    `expected inherited tags, got ${JSON.stringify(event.tags)}`,
  );
});

await check('dp-2: supersede of a metadata-less target (no scope/tags field at all) falls back to scope "repo" and carries no tags key', async () => {
  // Simulate a pre-dp-2 legacy event (or a legacy supersede target) by
  // appending a raw event that skips logDecision's scope default entirely —
  // this is the "metadata-less target" case D6 names explicitly.
  const legacyTargetId = crypto.randomUUID();
  appendJsonl(path.join(dpRoot, '.bee', 'decisions.jsonl'), {
    id: legacyTargetId,
    type: 'decide',
    date: new Date().toISOString(),
    decision: 'Legacy target with no scope field',
    rationale: 'predates D6',
  });
  const event = supersedeDecision(dpRoot, {
    supersedes: legacyTargetId,
    decision: 'Replace legacy target',
    rationale: 'D6 fallback path',
  });
  assert(event.scope === 'repo', `expected fallback scope "repo", got ${event.scope}`);
  assert(!('tags' in event), `expected no tags key when target has none, got ${JSON.stringify(event)}`);
});

await check('dp-2: explicit --tags/--scope on supersede override inheritance from the target', async () => {
  const target = logDecision(dpRoot, {
    decision: 'Use queue X',
    rationale: 'perf',
    scope: 'billing',
    tags: ['queue'],
  });
  const event = supersedeDecision(dpRoot, {
    supersedes: target.id,
    decision: 'Use queue Y',
    rationale: 'explicit override',
    tags: ['override-tag'],
    scope: 'checkout',
  });
  assert(event.scope === 'checkout', `expected explicit scope override, got ${event.scope}`);
  assert(event.tags.join(',') === 'override-tag', `expected explicit tags override, got ${JSON.stringify(event.tags)}`);
});

const sweepTarget = logDecision(dpRoot, {
  decision: 'Old sweep target decision',
  rationale: 'to be superseded',
  scope: 'checkout',
});
const sweepShort8 = sweepTarget.id.slice(0, 8);
const specDir = path.join(dpRoot, 'docs', 'specs');
fs.mkdirSync(specDir, { recursive: true });
const checkoutSpecPath = path.join(specDir, 'checkout.md');
const specLines = [
  '# Checkout',
  '',
  `Full id citation: ${sweepTarget.id} appears here.`,
  `Short citation: ${sweepShort8} appears here.`,
  `False positive embedded: abc${sweepShort8}def should not match.`,
  '',
];
fs.writeFileSync(checkoutSpecPath, specLines.join('\n'));
// Outside docs/** — must never be scanned (D2 pins the sweep root to docs/**).
fs.writeFileSync(
  path.join(dpRoot, 'src', 'notes.md'),
  `Outside docs/**, also cites ${sweepTarget.id} but must not count.\n`,
);

await check(
  'dp-2: propagation sweep finds full-id and standalone short8 citations under docs/**, rejects an embedded (non-boundary) short8, and rides the single append',
  async () => {
    const decisionsFile = path.join(dpRoot, '.bee', 'decisions.jsonl');
    const beforeLineCount = fs.readFileSync(decisionsFile, 'utf8').split(/\r?\n/).filter((l) => l.trim()).length;

    const event = supersedeDecision(dpRoot, {
      supersedes: sweepTarget.id,
      decision: 'Replace the sweep target',
      rationale: 'dp-2 sweep coverage',
    });

    assert(event.sweep, `expected event.sweep to be present, got ${JSON.stringify(event)}`);
    assert(
      event.sweep.hit_count === 2,
      `expected 2 sweep hits (full id + standalone short8), got ${JSON.stringify(event.sweep)}`,
    );
    assert(event.sweep.files.length === 2, `expected 2 hit records, got ${JSON.stringify(event.sweep.files)}`);
    const relSpecPath = path.relative(dpRoot, checkoutSpecPath);
    const hitFiles = event.sweep.files.map((h) => h.file);
    assert(
      hitFiles.every((f) => f === relSpecPath),
      `expected every hit under ${relSpecPath}, got ${JSON.stringify(hitFiles)}`,
    );
    const hitLines = event.sweep.files.map((h) => h.line).sort((a, b) => a - b);
    assert(hitLines.join(',') === '3,4', `expected hits on lines 3 and 4 only, got ${JSON.stringify(hitLines)}`);
    const embeddedHit = event.sweep.files.find((h) => h.line === 5);
    assert(!embeddedHit, 'the embedded (non-word-boundary) short8 occurrence on line 5 must NOT be a hit');

    // Lock doctrine: sweep computed BEFORE the append — the store gains
    // exactly one new line, and that line already carries the sweep result
    // inline (never a post-append rewrite of an already-written line).
    const afterLines = fs.readFileSync(decisionsFile, 'utf8').split(/\r?\n/).filter((l) => l.trim());
    assert(
      afterLines.length === beforeLineCount + 1,
      `expected exactly one new line appended, before ${beforeLineCount} after ${afterLines.length}`,
    );
    const appendedEvent = JSON.parse(afterLines[afterLines.length - 1]);
    assert(appendedEvent.id === event.id, 'the last line in the store is the returned supersede event');
    assert(
      appendedEvent.sweep && appendedEvent.sweep.hit_count === 2,
      'the appended line already carries the sweep result inline, not a later rewrite',
    );
  },
);

await check(
  'dp-2 CLI: one capture stub per sweep hit, source "supersede-sweep", outcome naming file:line and the new decision id',
  async () => {
    const stubTarget = logDecision(dpRoot, { decision: 'Stub target', rationale: 'isolated stub check', scope: 'billing' });
    const stubSpecPath = path.join(specDir, 'stub-target.md');
    fs.writeFileSync(stubSpecPath, `Cites ${stubTarget.id} once.\n`);

    const beforePending = pendingCaptureStubs(dpRoot).length;
    const run = await runBee(
      ['decisions', 'supersede', '--id', stubTarget.id, '--decision', 'Stub replacement', '--rationale', 'stub check', '--json'],
      dpRoot,
    );
    assert(run.status === 0, `CLI supersede exited ${run.status} :: ${run.stderr || run.stdout}`);
    const event = JSON.parse(run.stdout);
    assert(event.sweep.hit_count === 1, `expected exactly 1 hit, got ${JSON.stringify(event.sweep)}`);

    const pending = pendingCaptureStubs(dpRoot);
    assert(
      pending.length === beforePending + 1,
      `expected exactly one new capture stub, before ${beforePending} after ${pending.length}`,
    );
    const stub = pending[pending.length - 1];
    assert(stub.source === 'supersede-sweep', `expected stub.source "supersede-sweep", got ${stub.source}`);
    const hit = event.sweep.files[0];
    assert(stub.outcome.includes(`${hit.file}:${hit.line}`), `expected outcome to name file:line, got ${stub.outcome}`);
    assert(stub.outcome.includes(event.id), `expected outcome to name the new decision id, got ${stub.outcome}`);
  },
);

await check('dp-2 CLI: zero-hits sweep is a clean path — hit_count 0, empty files array, no capture stub, clean human message', async () => {
  const cleanTarget = logDecision(dpRoot, { decision: 'Never cited anywhere', rationale: 'clean path check' });
  const beforePending = pendingCaptureStubs(dpRoot).length;

  const jsonRun = await runBee(
    ['decisions', 'supersede', '--id', cleanTarget.id, '--decision', 'Replace never-cited', '--rationale', 'clean path', '--json'],
    dpRoot,
  );
  assert(jsonRun.status === 0, `CLI supersede exited ${jsonRun.status} :: ${jsonRun.stderr || jsonRun.stdout}`);
  const event = JSON.parse(jsonRun.stdout);
  assert(event.sweep.hit_count === 0, `expected 0 hits, got ${JSON.stringify(event.sweep)}`);
  assert(
    Array.isArray(event.sweep.files) && event.sweep.files.length === 0,
    `expected empty files array, got ${JSON.stringify(event.sweep.files)}`,
  );

  const pending = pendingCaptureStubs(dpRoot);
  assert(
    pending.length === beforePending,
    `expected no new capture stub on a zero-hit sweep, before ${beforePending} after ${pending.length}`,
  );

  const humanRun = await runBee(
    ['decisions', 'supersede', '--id', cleanTarget.id, '--decision', 'Replace again', '--rationale', 'human text check'],
    dpRoot,
  );
  assert(humanRun.status === 0, `human supersede exited ${humanRun.status} :: ${humanRun.stderr || humanRun.stdout}`);
  assert(
    /no citations/i.test(humanRun.stdout),
    `expected a clean no-citations message, got ${JSON.stringify(humanRun.stdout)}`,
  );
});

// ─── dp-3: archive verb + --all union reads (CONTEXT D4c) ─────────────────
// Own temp repo per check group to keep the archive split assertions free of
// the dp-1/dp-2 corpora above.

function decisionsArchiveFilePath(root) {
  return path.join(root, '.bee', 'decisions-archive.jsonl');
}

function rewriteEventDate(root, id, isoDate) {
  setEventDate(root, id, isoDate);
}

await check('dp-3: archive refuses (typed) when --before is omitted — no default age window', async () => {
  const root = makeTempRepo();
  logDecision(root, { decision: 'needs a before', rationale: 'refusal check' });
  assertThrows(() => archiveDecisions(root, {}), 'before', 'archiveDecisions with no --before at all refuses');
  assertThrows(() => archiveDecisions(root, { before: '' }), 'before', 'archiveDecisions with an empty --before refuses');
  assertThrows(
    () => archiveDecisions(root, { before: 'not-a-date' }),
    'before',
    'archiveDecisions with an unparsable --before refuses',
  );

  const run = await runBee(['decisions', 'archive', '--json'], root);
  assert(run.status !== 0, 'CLI archive with no --before exits non-zero');
  const payload = JSON.parse(run.stdout);
  // The registry's `required: ['before']` schema check intercepts before the
  // handler ever runs, so the refusal carries the CLI's structured
  // {field, reason, command} shape (test_bee_cli.mjs:296) rather than a
  // free-string `error`.
  const errorField = payload.error && typeof payload.error === 'object' ? payload.error.field : null;
  assert(errorField === 'before', `error should name field "before", got ${JSON.stringify(payload)}`);
});

await check('dp-3: archive refuses (typed) when nothing qualifies — a fresh event, --before far in the past', async () => {
  const root = makeTempRepo();
  logDecision(root, { decision: 'brand new, nothing to archive', rationale: 'refusal check' });
  assertThrows(
    () => archiveDecisions(root, { before: '2000-01-01' }),
    'nothing qualifies',
    'archiveDecisions refuses typed when zero events qualify',
  );
  assert(!fs.existsSync(decisionsArchiveFilePath(root)), 'a refused archive call never creates the archive file');
});

await check(
  'dp-3: archive split — superseded/redacted events move ALWAYS (regardless of age); plain decide events move only when strictly older than --before; everything else stays',
  async () => {
    const root = makeTempRepo();

    // Old, still-active decide event — old enough to be swept by age alone.
    const oldActive = logDecision(root, { decision: 'old active decide', rationale: 'age sweep target', scope: 'archive-test' });
    rewriteEventDate(root, oldActive.id, '2020-01-01T00:00:00.000Z');

    // Recent, still-active decide event — must survive (not old enough).
    const recentActive = logDecision(root, { decision: 'recent active decide', rationale: 'must survive', scope: 'archive-test' });

    // Recent supersede target — becomes superseded, must ALWAYS archive even
    // though it is recent (newer than --before).
    const supersedeTarget = logDecision(root, { decision: 'to be superseded', rationale: 'supersede target', scope: 'archive-test' });
    const supersedeEvent = supersedeDecision(root, {
      supersedes: supersedeTarget.id,
      decision: 'replacement decision',
      rationale: 'dp-3 archive split check',
    });

    // Recent redact target — becomes redacted, must ALWAYS archive even
    // though it is recent.
    const redactTarget = logDecision(root, { decision: 'to be redacted', rationale: 'redact target', scope: 'archive-test' });
    const redactEvent = redactDecision(root, { redacts: redactTarget.id, reason: 'dp-3 archive split check' });

    // --before sits strictly between oldActive's date and everything else's
    // "now" date — only oldActive should ever be swept by the age rule.
    const result = archiveDecisions(root, { before: '2021-01-01T00:00:00.000Z' });

    const archivedIds = result.archived.slice().sort();
    const expectedArchived = [oldActive.id, supersedeTarget.id, redactTarget.id].sort();
    assert(
      archivedIds.join(',') === expectedArchived.join(','),
      `expected exactly [old, supersedeTarget, redactTarget] archived, got ${JSON.stringify(archivedIds)}`,
    );

    const activeIds = new Set(readJsonl(path.join(root, '.bee', 'decisions.jsonl')).map((e) => e.id));
    assert(!activeIds.has(oldActive.id), 'old active decide event left the active file');
    assert(!activeIds.has(supersedeTarget.id), 'superseded target left the active file');
    assert(!activeIds.has(redactTarget.id), 'redacted target left the active file');
    assert(activeIds.has(recentActive.id), 'recent active decide event stayed in the active file');
    assert(activeIds.has(supersedeEvent.id), 'the supersede ACTION record itself stays in the active file');
    assert(activeIds.has(redactEvent.id), 'the redact ACTION record itself stays in the active file');

    const archivedEvents = readJsonl(decisionsArchiveFilePath(root));
    const archivedFileIds = new Set(archivedEvents.map((e) => e.id));
    assert(archivedFileIds.has(oldActive.id) && archivedFileIds.has(supersedeTarget.id) && archivedFileIds.has(redactTarget.id),
      `archive file missing an expected id, got ${JSON.stringify([...archivedFileIds])}`);
    assert(archivedFileIds.size === 3, `expected exactly 3 archived events on disk, got ${archivedFileIds.size}`);

    // Never rewrite surviving active events (append-only integrity):
    // recentActive/supersedeEvent/redactEvent must be byte-identical to
    // their pre-archive shape.
    const survivingRecent = readJsonl(path.join(root, '.bee', 'decisions.jsonl')).find((e) => e.id === recentActive.id);
    assert(
      JSON.stringify(survivingRecent) === JSON.stringify(recentActive),
      'surviving active decide event must never be rewritten by archive',
    );
  },
);

await check('dp-3: idempotent second run — nothing new qualifies, refuses cleanly, files untouched', async () => {
  const root = makeTempRepo();
  const old = logDecision(root, { decision: 'old one', rationale: 'idempotency check' });
  rewriteEventDate(root, old.id, '2020-01-01T00:00:00.000Z');
  const first = archiveDecisions(root, { before: '2021-01-01' });
  assert(first.archived.includes(old.id), 'first run archives the old event');

  const activeBefore = fs.readFileSync(path.join(root, '.bee', 'decisions.jsonl'), 'utf8');
  const archiveBefore = fs.readFileSync(decisionsArchiveFilePath(root), 'utf8');

  assertThrows(
    () => archiveDecisions(root, { before: '2021-01-01' }),
    'nothing qualifies',
    'second run with the same cutoff finds nothing left to archive',
  );

  const activeAfter = fs.readFileSync(path.join(root, '.bee', 'decisions.jsonl'), 'utf8');
  const archiveAfter = fs.readFileSync(decisionsArchiveFilePath(root), 'utf8');
  assert(activeAfter === activeBefore, 'a refused idempotent second run never touches the active file');
  assert(archiveAfter === archiveBefore, 'a refused idempotent second run never touches the archive file');
});

await check('dp-3: search/active --all reaches archived events; default (no --all) stays limited to the active store', async () => {
  const root = makeTempRepo();
  const old = logDecision(root, { decision: 'archived tag target', rationale: 'union read check', tags: ['archived-tag'] });
  rewriteEventDate(root, old.id, '2020-01-01T00:00:00.000Z');
  const recent = logDecision(root, { decision: 'recent tag target', rationale: 'union read check', tags: ['archived-tag'] });
  archiveDecisions(root, { before: '2021-01-01' });

  const defaultActive = await runBee(['decisions', 'active', '--json'], root);
  assert(defaultActive.status === 0, `default active exited ${defaultActive.status} :: ${defaultActive.stderr || defaultActive.stdout}`);
  const defaultIds = JSON.parse(defaultActive.stdout).decisions.map((d) => d.id);
  assert(!defaultIds.includes(old.id), 'default active (no --all) never reaches the archived event');
  assert(defaultIds.includes(recent.id), 'default active still lists the recent unarchived event');

  const allActive = await runBee(['decisions', 'active', '--all', '--json'], root);
  assert(allActive.status === 0, `active --all exited ${allActive.status} :: ${allActive.stderr || allActive.stdout}`);
  const allIds = JSON.parse(allActive.stdout).decisions.map((d) => d.id);
  assert(allIds.includes(old.id), 'active --all reaches the archived event');
  assert(allIds.includes(recent.id), 'active --all still includes the recent active event');
  assert(allIds.indexOf(recent.id) < allIds.indexOf(old.id), 'active --all is newest-first: recent event ranks before the older archived one');

  const defaultSearch = await runBee(['decisions', 'search', '--tag', 'archived-tag', '--json'], root);
  const defaultSearchIds = JSON.parse(defaultSearch.stdout).decisions.map((d) => d.id);
  assert(!defaultSearchIds.includes(old.id), 'default search (no --all) never reaches the archived event');

  const allSearch = await runBee(['decisions', 'search', '--tag', 'archived-tag', '--all', '--json'], root);
  assert(allSearch.status === 0, `search --all exited ${allSearch.status} :: ${allSearch.stderr || allSearch.stdout}`);
  const allSearchIds = JSON.parse(allSearch.stdout).decisions.map((d) => d.id);
  assert(allSearchIds.includes(old.id) && allSearchIds.includes(recent.id), 'search --tag --all reaches both the active and archived matches');
});

await check('dp-3: default reads are byte-identical to today for a store with no archive file at all', async () => {
  const root = makeTempRepo();
  logDecision(root, { decision: 'one', rationale: 'byte-identical check', tags: ['parity'] });
  logDecision(root, { decision: 'two', rationale: 'byte-identical check', tags: ['parity'] });
  assert(!fs.existsSync(decisionsArchiveFilePath(root)), 'precondition: no archive file exists yet');

  const withoutAll = activeDecisions(root);
  const withAllButNoArchive = activeDecisions(root, { all: true });
  assert(
    JSON.stringify(withoutAll) === JSON.stringify(withAllButNoArchive),
    'activeDecisions({all:true}) on an unarchived store must equal the default result exactly',
  );

  const cliDefault = await runBee(['decisions', 'active', '--json'], root);
  const cliAll = await runBee(['decisions', 'active', '--all', '--json'], root);
  assert(cliDefault.stdout === cliAll.stdout, 'CLI active --all output is byte-identical to default output on an unarchived store');
});

await check('dp-3: simulated partial-crash duplicate id — union read de-dupes by id, the ACTIVE copy wins, never a duplicate entry', async () => {
  const root = makeTempRepo();
  const dupId = crypto.randomUUID();
  const now = new Date().toISOString();
  // Simulate a crash between step (1) archive-append and step (2)
  // active-prune-rewrite: the same id lands in both files. The two payloads
  // differ deliberately so the winning copy is observable.
  appendJsonl(path.join(root, '.bee', 'decisions.jsonl'), {
    id: dupId,
    type: 'decide',
    date: now,
    decision: 'ACTIVE VERSION (post-crash, still the source of truth)',
    rationale: 'dp-3 crash-dedup check',
    scope: 'repo',
  });
  appendJsonl(decisionsArchiveFilePath(root), {
    id: dupId,
    type: 'decide',
    date: now,
    decision: 'ARCHIVE VERSION (stale, pre-crash copy — must lose)',
    rationale: 'dp-3 crash-dedup check',
    scope: 'repo',
  });

  const union = activeDecisions(root, { all: true });
  const matches = union.filter((event) => event.id === dupId);
  assert(matches.length === 1, `expected exactly one entry for the duplicated id, got ${matches.length}`);
  assert(
    matches[0].decision.startsWith('ACTIVE VERSION'),
    `expected the active copy to win the dedup, got ${JSON.stringify(matches[0])}`,
  );
});

await check(
  'dp-3: append writers (logDecision) and archive share the SAME store lock — a lock held externally blocks a logDecision call rather than corrupting the file',
  async () => {
    const root = makeTempRepo();
    logDecision(root, { decision: 'seed', rationale: 'lock-sharing check' });
    const preLockContents = fs.readFileSync(path.join(root, '.bee', 'decisions.jsonl'), 'utf8');

    const lock = acquireStoreLockOnceSync(root, DECISIONS_LOCK_NAME);
    assert(lock.acquired, 'precondition: test can acquire the decisions store lock directly');
    try {
      assertThrows(
        () => logDecision(root, { decision: 'should never land while the lock is held', rationale: 'lock-sharing check' }),
        'lock',
        'logDecision refuses/times out while the decisions store lock is externally held',
      );
      assertThrows(
        () => archiveDecisions(root, { before: '2099-01-01' }),
        'lock',
        'archiveDecisions also refuses/times out on the SAME externally-held lock (proves shared lock name)',
      );
    } finally {
      lock.release();
    }

    const postLockContents = fs.readFileSync(path.join(root, '.bee', 'decisions.jsonl'), 'utf8');
    assert(postLockContents === preLockContents, 'a refused write under lock contention never partially/corruptly writes the store');

    // Lock released — a normal call now succeeds.
    const event = logDecision(root, { decision: 'after release', rationale: 'lock-sharing check' });
    assert(event.id, 'logDecision succeeds again once the externally-held lock is released');
  },
);

// ─── dp-3: concurrent log-vs-archive under the shared lock (real OS-thread
// concurrency — see race_decisions_child.mjs for why this cannot be
// simulated single-threaded: the lock's bounded retry uses a synchronous
// Atomics.wait sleep, which blocks the event loop, so no same-thread timer
// could ever release a simulated holder mid-wait). ───────────────────────

const raceDecisionsChildScript = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  'race_decisions_child.mjs',
);

await check('race: log-vs-archive — concurrent loggers and an archiver under the shared decisions lock lose no writes and never duplicate an id across both files', async () => {
  const result = await runModuleWorker(raceDecisionsChildScript, { args: ['log-vs-archive'], timeout: 60000 });
  const stdout = result.stdout || '(empty)';
  const stderr = result.stderr || '(empty)';
  assert(result.status === 0, `log-vs-archive race failed (status ${result.status}): stdout=${stdout} stderr=${stderr}`);
  assert(/^PASS +log-vs-archive/m.test(stdout), `expected a PASS summary line, got: ${stdout}`);
});

// ─── dp-5: retro-tag event + overlay merge across union reads (CONTEXT D7c)
// Own temp repo per check group (same isolation discipline as dp-2/dp-3).

function decisionsRawLines(root) {
  return fs.readFileSync(decisionsFilePath(root), 'utf8').split(/\r?\n/).filter((l) => l.trim());
}

await check('dp-5: tagDecision resolves a full id target and appends a type:"tag" event carrying target/tags/scope', async () => {
  const root = makeTempRepo();
  const target = logDecision(root, { decision: 'legacy untagged decision', rationale: 'predates tags' });
  const event = tagDecision(root, { target: target.id, tags: ['retro', 'billing'], scope: 'billing' });
  assert(event.type === 'tag', `expected type "tag", got ${event.type}`);
  assert(event.target === target.id, `expected target ${target.id}, got ${event.target}`);
  assert(event.tags.join(',') === 'retro,billing', `expected tags to round-trip, got ${JSON.stringify(event.tags)}`);
  assert(event.scope === 'billing', `expected scope to round-trip, got ${event.scope}`);
  assert(typeof event.id === 'string' && event.id, 'tag event carries its own id');
  assert(typeof event.date === 'string' && event.date, 'tag event carries a date');
});

await check('dp-5: tagDecision resolves a short8 target uniquely', async () => {
  const root = makeTempRepo();
  const target = logDecision(root, { decision: 'short8 target', rationale: 'resolution check' });
  const event = tagDecision(root, { target: target.id.slice(0, 8), tags: ['short8-ok'] });
  assert(event.target === target.id, `expected short8 to resolve to the full id ${target.id}, got ${event.target}`);
});

await check('dp-5: tagDecision refuses (typed, naming the target) on an unknown full id or unknown short8', async () => {
  const root = makeTempRepo();
  logDecision(root, { decision: 'unrelated', rationale: 'unrelated' });
  assertThrows(
    () => tagDecision(root, { target: crypto.randomUUID(), tags: ['x'] }),
    'resolve',
    'unknown full id target refuses, naming the resolution failure',
  );
  assertThrows(
    () => tagDecision(root, { target: 'deadbeef', tags: ['x'] }),
    'resolve',
    'unknown short8 target refuses, naming the resolution failure',
  );
});

await check('dp-5: tagDecision refuses (typed, "ambiguous") when a short8 prefix matches more than one candidate', async () => {
  const root = makeTempRepo();
  const decisionsFile = decisionsFilePath(root);
  appendJsonl(decisionsFile, {
    id: 'deadbeef-0000-0000-0000-000000000001',
    type: 'decide',
    date: new Date().toISOString(),
    decision: 'dupe-prefix A',
    rationale: 'ambiguity check',
    scope: 'repo',
  });
  appendJsonl(decisionsFile, {
    id: 'deadbeef-0000-0000-0000-000000000002',
    type: 'decide',
    date: new Date().toISOString(),
    decision: 'dupe-prefix B',
    rationale: 'ambiguity check',
    scope: 'repo',
  });
  assertThrows(
    () => tagDecision(root, { target: 'deadbeef', tags: ['x'] }),
    'ambiguous',
    'a short8 prefix matching 2+ candidates refuses as ambiguous, never guesses',
  );
});

await check('dp-5: tagDecision target must be a decide/supersede event — a redact or another tag event id never resolves', async () => {
  const root = makeTempRepo();
  const target = logDecision(root, { decision: 'to be redacted', rationale: 'target-type check' });
  const redactEvent = redactDecision(root, { redacts: target.id, reason: 'target-type check' });
  assertThrows(
    () => tagDecision(root, { target: redactEvent.id, tags: ['x'] }),
    'resolve',
    'a redact event id is never a valid tag target',
  );
  const tagEvent = tagDecision(root, { target: target.id, tags: ['first'] });
  assertThrows(
    () => tagDecision(root, { target: tagEvent.id, tags: ['x'] }),
    'resolve',
    'a tag event id is never itself a valid tag target',
  );
});

await check('dp-5: tagDecision requires --tags (at least one) and validates the same lowercase-slug shape as logDecision', async () => {
  const root = makeTempRepo();
  const target = logDecision(root, { decision: 'tags validation target', rationale: 'x' });
  assertThrows(() => tagDecision(root, { target: target.id, tags: [] }), 'tag', 'empty tags array refuses');
  assertThrows(() => tagDecision(root, { target: target.id, tags: undefined }), 'tag', 'missing tags refuses');
  assertThrows(
    () => tagDecision(root, { target: target.id, tags: ['Not-Lowercase'] }),
    'tag',
    'an invalid slug in tags refuses',
  );
});

await check('dp-5: tagDecision never rewrites the target event\'s original jsonl line (append-only integrity)', async () => {
  const root = makeTempRepo();
  const target = logDecision(root, { decision: 'append-only check', rationale: 'x', scope: 'repo' });
  const before = decisionsRawLines(root);
  tagDecision(root, { target: target.id, tags: ['appended'] });
  const after = decisionsRawLines(root);
  assert(after.length === before.length + 1, `expected exactly one new line, before ${before.length} after ${after.length}`);
  assert(after[0] === before[0], 'the original target line is byte-identical after tagging (never rewritten)');
  const appended = JSON.parse(after[after.length - 1]);
  assert(appended.type === 'tag' && appended.target === target.id, 'the new line is the tag event itself');
});

// ─── dp-5: batch (--stdin) all-or-nothing ──────────────────────────────────

await check('dp-5: tagDecisionsBatch validates every entry before any write — one invalid target means the WHOLE batch appends nothing', async () => {
  const root = makeTempRepo();
  const good = logDecision(root, { decision: 'batch good target', rationale: 'x' });
  const before = decisionsRawLines(root).length;
  assertThrows(
    () =>
      tagDecisionsBatch(root, [
        { target: good.id, tags: ['ok'] },
        { target: crypto.randomUUID(), tags: ['bad-target'] },
      ]),
    'resolve',
    'a batch with any unresolvable target refuses as a whole',
  );
  const after = decisionsRawLines(root).length;
  assert(after === before, `expected zero new lines on a refused batch, before ${before} after ${after}`);
});

await check('dp-5: tagDecisionsBatch appends a fully-valid batch as new lines in exactly one write', async () => {
  const root = makeTempRepo();
  const a = logDecision(root, { decision: 'batch target A', rationale: 'x' });
  const b = logDecision(root, { decision: 'batch target B', rationale: 'x' });
  const before = decisionsRawLines(root).length;
  const events = tagDecisionsBatch(root, [
    { target: a.id, tags: ['batch-a'] },
    { target: b.id, tags: ['batch-b'], scope: 'checkout' },
  ]);
  assert(events.length === 2, `expected 2 returned tag events, got ${events.length}`);
  const after = decisionsRawLines(root).length;
  assert(after === before + 2, `expected exactly 2 new lines, before ${before} after ${after}`);
});

// ─── dp-5: overlay merge at read time (activeDecisions / search / active) ──

await check('dp-5: activeDecisions overlays a tag event\'s tags/scope onto its target (default, non-all path)', async () => {
  const root = makeTempRepo();
  const legacy = logDecision(root, { decision: 'legacy untagged, retro-tagged later', rationale: 'x' });
  tagDecision(root, { target: legacy.id, tags: ['retro-tagged'], scope: 'retro-scope' });
  const active = activeDecisions(root);
  const found = active.find((e) => e.id === legacy.id);
  assert(found, 'target event still listed among active decisions');
  assert(found.tags.join(',') === 'retro-tagged', `expected overlaid tags, got ${JSON.stringify(found.tags)}`);
  assert(found.scope === 'retro-scope', `expected overlaid scope, got ${found.scope}`);
});

await check('dp-5: overlay REPLACES the whole tags array — never merges/unions with the original tags', async () => {
  const root = makeTempRepo();
  const target = logDecision(root, { decision: 'has original tags', rationale: 'x', tags: ['original-a', 'original-b'] });
  tagDecision(root, { target: target.id, tags: ['replacement-only'] });
  const found = activeDecisions(root).find((e) => e.id === target.id);
  assert(found.tags.join(',') === 'replacement-only', `expected a full replacement, got ${JSON.stringify(found.tags)}`);
  assert(!found.tags.includes('original-a') && !found.tags.includes('original-b'), 'original tags must not survive alongside the replacement');
});

await check('dp-5: scope is replaced only when the tag event carries one — a scope-less retro-tag leaves the target scope untouched', async () => {
  const root = makeTempRepo();
  const target = logDecision(root, { decision: 'has an original scope', rationale: 'x', scope: 'original-scope' });
  tagDecision(root, { target: target.id, tags: ['no-scope-here'] });
  const found = activeDecisions(root).find((e) => e.id === target.id);
  assert(found.scope === 'original-scope', `expected the original scope untouched, got ${found.scope}`);
  assert(found.tags.join(',') === 'no-scope-here', `expected tags overlaid regardless, got ${JSON.stringify(found.tags)}`);
});

await check('dp-5: latest tag event wins when several retro-tag the same decision (by date, then file order on a tie)', async () => {
  const root = makeTempRepo();
  const target = logDecision(root, { decision: 'multi-tagged over time', rationale: 'x' });
  const first = tagDecision(root, { target: target.id, tags: ['first-tag'] });
  const second = tagDecision(root, { target: target.id, tags: ['second-tag'] });
  // Force a deterministic ordering: second is dated strictly after first.
  setEventDate(root, first.id, '2020-01-01T00:00:00.000Z');
  setEventDate(root, second.id, '2020-06-01T00:00:00.000Z');
  const found = activeDecisions(root).find((e) => e.id === target.id);
  assert(found.tags.join(',') === 'second-tag', `expected the later tag event to win, got ${JSON.stringify(found.tags)}`);

  // Tie on date: file order (later-appended line) wins.
  const third = tagDecision(root, { target: target.id, tags: ['third-tag'] });
  setEventDate(root, third.id, '2020-06-01T00:00:00.000Z'); // same date as `second`
  const foundAfterTie = activeDecisions(root).find((e) => e.id === target.id);
  assert(
    foundAfterTie.tags.join(',') === 'third-tag',
    `expected the later-in-file tag event to win a date tie, got ${JSON.stringify(foundAfterTie.tags)}`,
  );
});

await check('dp-5: tag events are never listed as decisions themselves and are never counted by --recent', async () => {
  const root = makeTempRepo();
  const a = logDecision(root, { decision: 'recent-count A', rationale: 'x' });
  const b = logDecision(root, { decision: 'recent-count B', rationale: 'x' });
  tagDecision(root, { target: a.id, tags: ['recent-check'] });
  const active = activeDecisions(root);
  assert(!active.some((e) => e.type === 'tag'), 'no tag-typed event ever appears in activeDecisions output');
  const recent = activeDecisions(root, { recent: 2 });
  assert(recent.length === 2, `expected --recent 2 to still return exactly 2 decide events, got ${recent.length}`);
  assert(recent.every((e) => e.id === a.id || e.id === b.id), '--recent counts only decide/supersede events, never the tag event');
});

await check('dp-5 CLI: decisions search --tag / --scope / --area finds a legacy (pre-tag) event via its retro-tag overlay', async () => {
  const root = makeTempRepo();
  const legacy = logDecision(root, { decision: 'legacy, findable only after retro-tag', rationale: 'x' });
  const before = await runBee(['decisions', 'search', '--tag', 'newly-retro', '--json'], root);
  assert(JSON.parse(before.stdout).decisions.length === 0, 'not yet findable by the tag before retro-tagging');

  const tagRun = await runBee(
    ['decisions', 'tag', '--target', legacy.id, '--tags', 'newly-retro', '--scope', 'retro-area', '--json'],
    root,
  );
  assert(tagRun.status === 0, `CLI tag exited ${tagRun.status} :: ${tagRun.stderr || tagRun.stdout}`);

  const afterTag = await runBee(['decisions', 'search', '--tag', 'newly-retro', '--json'], root);
  assert(afterTag.status === 0, `search --tag exited ${afterTag.status} :: ${afterTag.stderr || afterTag.stdout}`);
  const ids = JSON.parse(afterTag.stdout).decisions.map((d) => d.id);
  assert(ids.includes(legacy.id), 'search --tag now finds the legacy event via the overlay');

  const byScope = await runBee(['decisions', 'search', '--scope', 'retro-area', '--json'], root);
  assert(JSON.parse(byScope.stdout).decisions.map((d) => d.id).includes(legacy.id), 'search --scope finds it via the overlaid scope');
  const byArea = await runBee(['decisions', 'active', '--area', 'retro-area', '--json'], root);
  assert(JSON.parse(byArea.stdout).decisions.map((d) => d.id).includes(legacy.id), 'active --area finds it via the overlaid scope too');
});

// ─── dp-5: overlay reaches an archived target through the --all union ─────

await check('dp-5: overlay reaches an archived-then-retro-tagged event through the --all union; tag events themselves are never archived', async () => {
  const root = makeTempRepo();
  const old = logDecision(root, { decision: 'will be archived, then retro-tagged', rationale: 'x' });
  setEventDate(root, old.id, '2020-01-01T00:00:00.000Z');
  const recent = logDecision(root, { decision: 'stays active, keeps the store non-empty', rationale: 'x' });
  archiveDecisions(root, { before: '2021-01-01' });

  // Retro-tag the now-archived decision — its own event never moves; the
  // resolution reads the active+archive union to find it.
  const tagEvent = tagDecision(root, { target: old.id, tags: ['archived-then-tagged'] });

  const defaultActive = activeDecisions(root);
  assert(!defaultActive.some((e) => e.id === old.id), 'default (no --all) read still never reaches the archived target');

  const allActive = activeDecisions(root, { all: true });
  const found = allActive.find((e) => e.id === old.id);
  assert(found, 'the --all union still reaches the archived target');
  assert(found.tags.join(',') === 'archived-then-tagged', `expected the overlay to apply even to an archived target, got ${JSON.stringify(found.tags)}`);

  // Age out the tag event's own date far in the past and archive again —
  // a tag-typed event must never be swept into the archive file (D7c
  // prohibition: tag events stay in the active file this slice).
  setEventDate(root, tagEvent.id, '2020-01-01T00:00:00.000Z');
  try {
    archiveDecisions(root, { before: '2021-01-01' });
  } catch {
    // "nothing qualifies" is an acceptable outcome here — it only proves the
    // tag event was never a candidate; recentEvent already keeps the active
    // file non-empty so this branch is not expected, but tolerate it.
  }
  const activeRaw = readJsonl(decisionsFilePath(root));
  assert(activeRaw.some((e) => e.id === tagEvent.id), 'the tag event itself is still in the ACTIVE file, never archived, even when old');
  assert(recent, 'sanity: recent stays referenced');
});

// ─── dp-5: locked append routes through the SAME shared primitive ─────────

await check(
  'dp-5: tagDecision/tagDecisionsBatch share the SAME decisions store lock as log/supersede/redact/archive — refuses while externally held',
  async () => {
    const root = makeTempRepo();
    const target = logDecision(root, { decision: 'lock-sharing target', rationale: 'x' });
    const before = decisionsRawLines(root);

    const lock = acquireStoreLockOnceSync(root, DECISIONS_LOCK_NAME);
    assert(lock.acquired, 'precondition: test can acquire the decisions store lock directly');
    try {
      assertThrows(
        () => tagDecision(root, { target: target.id, tags: ['should-never-land'] }),
        'lock',
        'tagDecision refuses/times out while the decisions store lock is externally held',
      );
      assertThrows(
        () => tagDecisionsBatch(root, [{ target: target.id, tags: ['should-never-land-batch'] }]),
        'lock',
        'tagDecisionsBatch also refuses/times out on the SAME externally-held lock',
      );
    } finally {
      lock.release();
    }

    const after = decisionsRawLines(root);
    assert(after.length === before.length, 'a refused tag write under lock contention never partially writes the store');

    const event = tagDecision(root, { target: target.id, tags: ['after-release'] });
    assert(event.id, 'tagDecision succeeds again once the externally-held lock is released');
  },
);

// ─── dp-5 CLI: --stdin batch (all-or-nothing) ──────────────────────────────

await check('CLI: decisions tag --stdin with a fully-valid JSON array appends every entry in one call', async () => {
  const root = makeTempRepo();
  const a = logDecision(root, { decision: 'stdin batch target A', rationale: 'x' });
  const b = logDecision(root, { decision: 'stdin batch target B', rationale: 'x' });
  const payload = JSON.stringify([
    { target: a.id, tags: ['stdin-a'] },
    { target: b.id, tags: ['stdin-b'], scope: 'stdin-scope' },
  ]);
  const run = await runBee(['decisions', 'tag', '--stdin', '--json'], root, payload);
  assert(run.status === 0, `CLI tag --stdin exited ${run.status} :: ${run.stderr || run.stdout}`);

  const activeA = await runBee(['decisions', 'search', '--tag', 'stdin-a', '--json'], root);
  assert(JSON.parse(activeA.stdout).decisions.map((d) => d.id).includes(a.id), 'batch entry A landed via the overlay');
  const activeB = await runBee(['decisions', 'search', '--scope', 'stdin-scope', '--json'], root);
  assert(JSON.parse(activeB.stdout).decisions.map((d) => d.id).includes(b.id), 'batch entry B landed via the overlay');
});

await check('CLI: decisions tag --stdin with one invalid target in the array appends NOTHING (all-or-nothing)', async () => {
  const root = makeTempRepo();
  const good = logDecision(root, { decision: 'stdin batch good target', rationale: 'x' });
  const before = decisionsRawLines(root).length;
  const payload = JSON.stringify([
    { target: good.id, tags: ['should-not-land'] },
    { target: crypto.randomUUID(), tags: ['bad'] },
  ]);
  const run = await runBee(['decisions', 'tag', '--stdin', '--json'], root, payload);
  assert(run.status !== 0, 'a batch with an unresolvable target exits non-zero');
  const after = decisionsRawLines(root).length;
  assert(after === before, `expected zero new lines on a refused CLI batch, before ${before} after ${after}`);
});

await check('CLI: decisions tag without --tags, or naming an unknown target, exits non-zero with a clear error', async () => {
  const root = makeTempRepo();
  const target = logDecision(root, { decision: 'cli refusal check', rationale: 'x' });

  const missingTags = await runBee(['decisions', 'tag', '--target', target.id, '--json'], root);
  assert(missingTags.status !== 0, 'missing --tags exits non-zero');

  const unknownTarget = await runBee(['decisions', 'tag', '--target', crypto.randomUUID(), '--tags', 'x', '--json'], root);
  assert(unknownTarget.status !== 0, 'unknown target exits non-zero');
  const payload = JSON.parse(unknownTarget.stdout);
  assert(typeof payload.error === 'string' ? /resolve/i.test(payload.error) : true, 'error should hint at target resolution failure when a free-string error is used');
});

// ─── dp-6 (CONTEXT D7a/b, D8b): taxonomy + write-time classification
// enforcement + ranked multi-term search ────────────────────────────────────

const REPO_ROOT = fileURLToPath(new URL('../../../', import.meta.url));

function taxonomyFilePath(root) {
  return path.join(root, 'docs', 'decisions', 'taxonomy.json');
}

function writeTaxonomyFixture(root, { tags = [], candidates = [] } = {}) {
  const file = taxonomyFilePath(root);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, `${JSON.stringify({ schema_version: 1, tags, candidates }, null, 2)}\n`, 'utf8');
}

// ─── Self-hosting steady state: dp-7 bootstrapped this repo's real
// taxonomy.json (the dp-6-era absence guard expired by design when the
// backfill landed). From here on the file must exist, parse, and carry a
// non-empty canonical vocabulary — classification is mandatory at write
// time in THIS repo now.
// ────────────────────────────────────────────────────────────────────────

await check(
  'dp-7: this repo\'s docs/decisions/taxonomy.json exists, parses, and carries a non-empty tags[] vocabulary',
  async () => {
    const p = taxonomyFilePath(REPO_ROOT);
    assert(fs.existsSync(p), 'dp-7 bootstrapped the canonical taxonomy — its absence is a regression');
    const parsed = JSON.parse(fs.readFileSync(p, 'utf8'));
    assert(Array.isArray(parsed.tags) && parsed.tags.length > 0, 'taxonomy tags[] must be a non-empty array');
    // Schema contract (dp-6, loadTaxonomy maps t.name): entries are {name}
    // objects — a plain-string seed silently classifies EVERY known tag as
    // unknown and leaks the whole vocabulary into candidates[].
    for (const entry of parsed.tags) {
      assert(
        entry && typeof entry === 'object' && typeof entry.name === 'string' && entry.name.length > 0,
        `taxonomy tags[] entry must be a {name} object, got: ${JSON.stringify(entry)}`,
      );
    }
    assert(Array.isArray(parsed.candidates), 'taxonomy candidates[] must be an array');
    const names = new Set(parsed.tags.map((t) => t.name));
    for (const c of parsed.candidates) {
      assert(!names.has(c), `candidates[] must never hold an already-known tag (found "${c}")`);
    }
  },
);

// ─── bootstrap-safe path: no taxonomy file -> warn only, never refuse ──────

await check('dp-6: without a taxonomy file, logDecision with zero tags still succeeds (bootstrap-safe warn-only, unchanged shape)', async () => {
  const r = makeTempRepo();
  assert(!fs.existsSync(taxonomyFilePath(r)), 'sanity: fresh repo has no taxonomy file');
  const event = logDecision(r, { decision: 'No taxonomy yet', rationale: 'bootstrap' });
  assert(!('tags' in event), 'untagged event still has no tags key at all, unchanged from pre-dp-6 shape');
});

await check('dp-6 CLI: decisions log --json without --tags and without a taxonomy file still succeeds (never blocks)', async () => {
  const r = makeTempRepo();
  const run = await runBee(['decisions', 'log', '--decision', 'no tags json mode', '--rationale', 'x', '--json'], r);
  assert(run.status === 0, `expected success exit, got ${run.status} :: ${run.stderr || run.stdout}`);
  const event = JSON.parse(run.stdout);
  assert(!('tags' in event), 'still no tags key, unchanged shape');
});

await check('dp-6 CLI: decisions log (non-JSON text) without --tags and without a taxonomy file warns and proceeds', async () => {
  const r = makeTempRepo();
  const run = await runBee(['decisions', 'log', '--decision', 'no tags no taxonomy', '--rationale', 'x'], r);
  assert(run.status === 0, `expected success exit, got ${run.status} :: ${run.stderr || run.stdout}`);
  assert(/warn/i.test(run.stdout), `expected a warning in the human-readable text output, got ${JSON.stringify(run.stdout)}`);
});

// ─── enforcement: taxonomy present -> zero tags refused, typed, names --tags ─

await check('dp-6: with a taxonomy file present, logDecision with zero tags is refused with a typed error naming --tags', async () => {
  const r = makeTempRepo();
  writeTaxonomyFixture(r, { tags: [{ name: 'billing', description: 'Billing' }] });
  assertThrows(
    () => logDecision(r, { decision: 'Should be refused', rationale: 'no tags supplied' }),
    '--tags',
    'zero-tag decide log refused once taxonomy exists',
  );
});

await check('dp-6 CLI: decisions log with a taxonomy file present and zero tags exits non-zero, error names --tags', async () => {
  const r = makeTempRepo();
  writeTaxonomyFixture(r, { tags: [{ name: 'billing', description: 'Billing' }] });
  const run = await runBee(['decisions', 'log', '--decision', 'x', '--rationale', 'y', '--json'], r);
  assert(run.status !== 0, 'zero-tag decide log refused via CLI once taxonomy exists');
  const payload = JSON.parse(run.stdout);
  assert(/--tags/.test(payload.error || ''), `error should name --tags, got ${JSON.stringify(payload)}`);
});

await check('dp-6: with a taxonomy file present, logDecision with a known tag succeeds and never touches candidates', async () => {
  const r = makeTempRepo();
  writeTaxonomyFixture(r, { tags: [{ name: 'billing', description: 'Billing' }] });
  const event = logDecision(r, { decision: 'Reconcile invoices', rationale: 'x', tags: ['billing'] });
  assert(event.tags.join(',') === 'billing', 'known tag round-trips onto the event');
  const taxonomy = JSON.parse(fs.readFileSync(taxonomyFilePath(r), 'utf8'));
  assert(taxonomy.candidates.length === 0, 'a known tag never lands in candidates');
});

// ─── unknown tags: never refused, accepted + appended to candidates ───────

await check('dp-6: an unknown tag is accepted on the event AND appended to taxonomy candidates in the same call', async () => {
  const r = makeTempRepo();
  writeTaxonomyFixture(r, { tags: [{ name: 'billing', description: 'Billing' }], candidates: [] });
  const event = logDecision(r, { decision: 'Adopt a new discipline', rationale: 'x', tags: ['brand-new-tag'] });
  assert(event.tags.join(',') === 'brand-new-tag', 'unknown tag accepted on the event, never refused');
  const taxonomy = JSON.parse(fs.readFileSync(taxonomyFilePath(r), 'utf8'));
  assert(
    taxonomy.candidates.includes('brand-new-tag'),
    `expected candidates to include the unknown tag, got ${JSON.stringify(taxonomy.candidates)}`,
  );
  assert(taxonomy.tags.length === 1 && taxonomy.tags[0].name === 'billing', 'hand-curated tags[] left untouched');
});

await check('dp-6: an unknown tag already sitting in candidates is not duplicated on reuse', async () => {
  const r = makeTempRepo();
  writeTaxonomyFixture(r, { tags: [], candidates: ['seen-once'] });
  logDecision(r, { decision: 'Reuse a pending candidate tag', rationale: 'x', tags: ['seen-once'] });
  const taxonomy = JSON.parse(fs.readFileSync(taxonomyFilePath(r), 'utf8'));
  assert(taxonomy.candidates.filter((c) => c === 'seen-once').length === 1, 'candidate tag not duplicated');
});

await check('dp-6: mixing a known and an unknown tag on one event never refuses — only zero tags refuses', async () => {
  const r = makeTempRepo();
  writeTaxonomyFixture(r, { tags: [{ name: 'billing', description: 'Billing' }] });
  const event = logDecision(r, { decision: 'Mixed tags', rationale: 'x', tags: ['billing', 'net-new'] });
  assert(event.tags.join(',') === 'billing,net-new', 'both tags land on the event');
  const taxonomy = JSON.parse(fs.readFileSync(taxonomyFilePath(r), 'utf8'));
  assert(taxonomy.candidates.includes('net-new'), 'the unknown tag was appended to candidates');
});

await check('dp-6: repeated unknown-tag appends leave taxonomy.json as valid, non-partial JSON with every candidate present', async () => {
  const r = makeTempRepo();
  writeTaxonomyFixture(r, { tags: [] });
  for (const tag of ['t-one', 't-two', 't-three']) {
    logDecision(r, { decision: `Decision for ${tag}`, rationale: 'x', tags: [tag] });
  }
  const raw = fs.readFileSync(taxonomyFilePath(r), 'utf8');
  const taxonomy = JSON.parse(raw); // throws if the file were ever left mid-write/partial
  assert(
    ['t-one', 't-two', 't-three'].every((t) => taxonomy.candidates.includes(t)),
    `expected all three candidates present, got ${JSON.stringify(taxonomy.candidates)}`,
  );
});

// ─── supersede inheritance consults the OVERLAY-APPLIED target ────────────

await check(
  'dp-6: supersede inherits OVERLAY-APPLIED tags from a raw-untagged, retro-tagged target — no false zero-tag refusal once taxonomy exists',
  async () => {
    const r = makeTempRepo();
    const legacyId = crypto.randomUUID();
    appendJsonl(path.join(r, '.bee', 'decisions.jsonl'), {
      id: legacyId,
      type: 'decide',
      date: new Date().toISOString(),
      decision: 'Legacy raw-untagged decision',
      rationale: 'predates tags',
    });
    tagDecision(r, { target: legacyId, tags: ['billing', 'retro'] });

    // Taxonomy now exists — inheritance reading the RAW (untagged) event
    // instead of the overlay-applied one would wrongly refuse this call.
    writeTaxonomyFixture(r, { tags: [{ name: 'billing', description: 'Billing' }], candidates: ['retro'] });

    const event = supersedeDecision(r, {
      supersedes: legacyId,
      decision: 'Replace the legacy decision',
      rationale: 'dp-6 inheritance check',
    });
    assert(
      Array.isArray(event.tags) && event.tags.join(',') === 'billing,retro',
      `expected supersede to inherit the OVERLAID tags [billing, retro], got ${JSON.stringify(event.tags)}`,
    );
  },
);

await check(
  'dp-6: supersede of a target with no tags at all (raw or overlaid) is refused once taxonomy exists, unless --tags is given explicitly',
  async () => {
    const r = makeTempRepo();
    const target = logDecision(r, { decision: 'Untagged target, no taxonomy yet', rationale: 'x' });
    writeTaxonomyFixture(r, { tags: [{ name: 'billing', description: 'Billing' }] });
    assertThrows(
      () => supersedeDecision(r, { supersedes: target.id, decision: 'Replacement', rationale: 'x' }),
      '--tags',
      'supersede with nothing to inherit and no --tags is refused once taxonomy exists',
    );
    const event = supersedeDecision(r, {
      supersedes: target.id,
      decision: 'Replacement with explicit tags',
      rationale: 'x',
      tags: ['billing'],
    });
    assert(event.tags.join(',') === 'billing', 'explicit --tags satisfies the classification requirement');
  },
);

await check('dp-6: supersede with an explicit unknown tag is accepted (never refused) and appended to taxonomy candidates', async () => {
  const r = makeTempRepo();
  const target = logDecision(r, { decision: 'Target for supersede unknown-tag test', rationale: 'x' });
  writeTaxonomyFixture(r, { tags: [{ name: 'billing', description: 'Billing' }] });
  const event = supersedeDecision(r, {
    supersedes: target.id,
    decision: 'Replacement',
    rationale: 'x',
    tags: ['billing', 'supersede-new-tag'],
  });
  assert(event.tags.join(',') === 'billing,supersede-new-tag', 'unknown tag accepted on the supersede event');
  const taxonomy = JSON.parse(fs.readFileSync(taxonomyFilePath(r), 'utf8'));
  assert(taxonomy.candidates.includes('supersede-new-tag'), 'unknown tag from supersede also lands in taxonomy candidates');
});

await check('dp-6: supersede without a taxonomy file still inherits raw target tags exactly as dp-2 (unchanged, warn-only path)', async () => {
  const r = makeTempRepo();
  const target = logDecision(r, { decision: 'Use queue A', rationale: 'perf', tags: ['queue'] });
  const event = supersedeDecision(r, { supersedes: target.id, decision: 'Use queue B', rationale: 'perf' });
  assert(event.tags.join(',') === 'queue', 'dp-2 inheritance behavior unchanged when no taxonomy file exists');
});

// ─── ranked multi-term OR search (D8b) ─────────────────────────────────────

const rankRoot = makeTempRepo();

const rOld = logDecision(rankRoot, {
  decision: 'Use exponential backoff for webhook retries',
  rationale: 'billing reliability',
  tags: ['billing', 'webhooks'],
});
setEventDate(rankRoot, rOld.id, '2026-01-01T00:00:00.000Z');

const rMid = logDecision(rankRoot, {
  decision: 'Adopt structured logging for the queue',
  rationale: 'observability',
  tags: ['observability'],
});
setEventDate(rankRoot, rMid.id, '2026-02-01T00:00:00.000Z');

const rBest = logDecision(rankRoot, {
  decision: 'Retry billing webhooks with backoff and structured logging',
  rationale: 'billing observability improvements',
  tags: ['billing', 'webhooks', 'observability'],
});
setEventDate(rankRoot, rBest.id, '2026-01-15T00:00:00.000Z');

await check('dp-6: multi-term search ranks by deterministic term-hit count descending, then date descending', async () => {
  const run = await runBee(['decisions', 'search', '--text', 'billing webhooks observability', '--json'], rankRoot);
  assert(run.status === 0, `search exited ${run.status} :: ${run.stderr || run.stdout}`);
  const ids = JSON.parse(run.stdout).decisions.map((d) => d.id);
  assert(ids[0] === rBest.id, `expected the 3-term hit first, got order ${JSON.stringify(ids)}`);
  const oldIdx = ids.indexOf(rOld.id);
  const midIdx = ids.indexOf(rMid.id);
  assert(
    oldIdx !== -1 && midIdx !== -1 && oldIdx < midIdx,
    `expected higher hit-count (rOld, 2 terms) to rank before lower hit-count (rMid, 1 term) regardless of date, got ${JSON.stringify(ids)}`,
  );
});

await check('dp-6: multi-term search matches OVERLAID tags, not just decision/rationale/alternatives text', async () => {
  const r = makeTempRepo();
  const legacyId = crypto.randomUUID();
  appendJsonl(path.join(r, '.bee', 'decisions.jsonl'), {
    id: legacyId,
    type: 'decide',
    date: new Date().toISOString(),
    decision: 'Text with no matching words at all',
    rationale: 'unrelated rationale text',
  });
  tagDecision(r, { target: legacyId, tags: ['zorbatag'] });
  const run = await runBee(['decisions', 'search', '--text', 'zorbatag', '--json'], r);
  assert(run.status === 0, `search exited ${run.status} :: ${run.stderr || run.stdout}`);
  const ids = JSON.parse(run.stdout).decisions.map((d) => d.id);
  assert(ids.includes(legacyId), `expected the tag-only term to match via the overlay, got ${JSON.stringify(ids)}`);
});

await check('dp-6: single-term search results remain a superset of the pre-dp-6 substring match', async () => {
  const run = await runBee(['decisions', 'search', '--text', 'billing', '--json'], rankRoot);
  const ids = JSON.parse(run.stdout).decisions.map((d) => d.id);
  assert(ids.includes(rOld.id) && ids.includes(rBest.id), 'both events whose text contains "billing" still match under a single term');
});

await check('dp-6: multi-term ranking spans the --all archive union with the overlay applied', async () => {
  const r = makeTempRepo();
  const oldEvent = logDecision(r, { decision: 'Archived billing webhook decision', rationale: 'x', tags: ['billing'] });
  setEventDate(r, oldEvent.id, '2020-01-01T00:00:00.000Z');
  archiveDecisions(r, { before: '2021-01-01' });

  const noAll = await runBee(['decisions', 'search', '--text', 'billing webhook', '--json'], r);
  assert(
    !JSON.parse(noAll.stdout).decisions.map((d) => d.id).includes(oldEvent.id),
    'archived event absent from the default (non-all) read',
  );

  const withAll = await runBee(['decisions', 'search', '--text', 'billing webhook', '--all', '--json'], r);
  assert(
    JSON.parse(withAll.stdout).decisions.map((d) => d.id).includes(oldEvent.id),
    'archived event reachable and ranked via --all',
  );
});

await check('dp-6: zero-filter search (no --text/--tag/--scope/--since/--untagged) still refuses exactly as before', async () => {
  const r = makeTempRepo();
  logDecision(r, { decision: 'x', rationale: 'y' });
  const run = await runBee(['decisions', 'search', '--json'], r);
  assert(run.status !== 0, 'zero-filter search still refuses');
});

// ─── --untagged listing (D7d completeness check), composable with --all ───

await check('dp-6: --untagged lists exactly the events with no tags AFTER overlay, composable with --all', async () => {
  const r = makeTempRepo();
  const tagged = logDecision(r, { decision: 'Tagged event', rationale: 'x', tags: ['billing'] });
  const untaggedRaw = logDecision(r, { decision: 'Untagged event', rationale: 'x' });
  const retroTagged = logDecision(r, { decision: 'Will be retro-tagged', rationale: 'x' });
  tagDecision(r, { target: retroTagged.id, tags: ['retro'] });

  const run = await runBee(['decisions', 'search', '--untagged', '--json'], r);
  assert(run.status === 0, `search --untagged exited ${run.status} :: ${run.stderr || run.stdout}`);
  const ids = JSON.parse(run.stdout).decisions.map((d) => d.id);
  assert(ids.includes(untaggedRaw.id), 'raw-untagged event listed');
  assert(!ids.includes(tagged.id), 'raw-tagged event excluded');
  assert(!ids.includes(retroTagged.id), 'retro-tagged event excluded once its overlay tags apply');

  const oldUntagged = logDecision(r, { decision: 'Old untagged event', rationale: 'x' });
  setEventDate(r, oldUntagged.id, '2020-01-01T00:00:00.000Z');
  archiveDecisions(r, { before: '2021-01-01' });
  const runAll = await runBee(['decisions', 'search', '--untagged', '--all', '--json'], r);
  const allIds = JSON.parse(runAll.stdout).decisions.map((d) => d.id);
  assert(allIds.includes(oldUntagged.id), 'archived untagged event reachable via --untagged --all');
});

await check('dp-6: decisions active --untagged filters the same way as search --untagged', async () => {
  const r = makeTempRepo();
  const tagged = logDecision(r, { decision: 'Tagged', rationale: 'x', tags: ['billing'] });
  const untagged = logDecision(r, { decision: 'Untagged', rationale: 'x' });
  const run = await runBee(['decisions', 'active', '--untagged', '--json'], r);
  assert(run.status === 0, `active --untagged exited ${run.status} :: ${run.stderr || run.stdout}`);
  const ids = JSON.parse(run.stdout).decisions.map((d) => d.id);
  assert(ids.includes(untagged.id) && !ids.includes(tagged.id), `expected only the untagged event, got ${JSON.stringify(ids)}`);
});

await check('dp-6: search --untagged alone satisfies the "needs at least one filter" requirement — no --text required', async () => {
  const r = makeTempRepo();
  logDecision(r, { decision: 'Untagged only', rationale: 'x' });
  const run = await runBee(['decisions', 'search', '--untagged', '--json'], r);
  assert(run.status === 0, `search --untagged alone should not require --text, got exit ${run.status} :: ${run.stderr || run.stdout}`);
});

// ─── dp-4: derived decision index — `decisions render` -> docs/decisions/
// index.md (CONTEXT D4b/D6, overlay-aware per D7/D8). Own temp repo, same
// isolation discipline as dp-2/dp-3/dp-5. ──────────────────────────────────

function decisionIndexFilePath(root) {
  return path.join(root, 'docs', 'decisions', 'index.md');
}

// Slices the rendered content down to one `## <scope>` section (up to the
// next `## ` scope heading or EOF) so assertions can check a scope's
// contents without depending on the exact position of unrelated scopes.
function extractScopeSection(content, scope) {
  const marker = `## ${scope}\n`;
  const start = content.indexOf(marker);
  if (start === -1) return null;
  const rest = content.slice(start + marker.length);
  const nextIdx = rest.search(/\n## /);
  return nextIdx === -1 ? rest : rest.slice(0, nextIdx);
}

const dp4Root = makeTempRepo();

const dp4B1 = logDecision(dp4Root, { decision: 'B1 invoice decision', rationale: 'x', scope: 'billing', tags: ['invoices'] });
const dp4B2 = logDecision(dp4Root, { decision: 'B2 invoice decision second', rationale: 'x', scope: 'billing', tags: ['invoices'] });
setEventDate(dp4Root, dp4B1.id, '2026-01-01T00:00:00.000Z');
setEventDate(dp4Root, dp4B2.id, '2026-02-01T00:00:00.000Z');

const dp4C1 = logDecision(dp4Root, { decision: 'C1 checkout decision', rationale: 'x', scope: 'checkout', tags: ['flow'] });
const dp4LegacyC = logDecision(dp4Root, { decision: 'Legacy untagged checkout decision', rationale: 'x', scope: 'checkout' });

const dp4STarget = logDecision(dp4Root, { decision: 'S target to supersede', rationale: 'x', scope: 'billing', tags: ['invoices'] });
const dp4SEvent = supersedeDecision(dp4Root, { supersedes: dp4STarget.id, decision: 'S replaced', rationale: 'dp-4 supersede placement' });

const dp4ATarget = logDecision(dp4Root, { decision: 'Archived old decision', rationale: 'x', scope: 'billing', tags: ['invoices'] });
setEventDate(dp4Root, dp4ATarget.id, '2020-01-01T00:00:00.000Z');
archiveDecisions(dp4Root, { before: '2021-01-01' });

const dp4LegacyR = logDecision(dp4Root, { decision: 'Legacy retro target', rationale: 'x' }); // scope defaults "repo", no tags
tagDecision(dp4Root, { target: dp4LegacyR.id, tags: ['retro-only'], scope: 'retro-scope' });

const dp4Multiline = logDecision(dp4Root, {
  decision: 'Multi-line decision\nSecond line detail',
  rationale: 'x',
  scope: 'checkout',
  tags: ['flow'],
});

await check('dp-4: renders a provenance header naming the generator, with no hand-edit / determinism note', async () => {
  const result = renderDecisionIndex(dp4Root, {});
  assert(typeof result.path === 'string' && result.path.length > 0, 'renderDecisionIndex must return a path');
  assert(fs.existsSync(decisionIndexFilePath(dp4Root)), 'index.md should now exist on disk');
  const content = fs.readFileSync(decisionIndexFilePath(dp4Root), 'utf8');
  assert(/bee decisions render/.test(content), 'header should name the generator command');
  assert(/hand-edit/i.test(content), 'header should warn against hand-editing');
  assert(
    !/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}/.test(content),
    'no wall-clock (full ISO timestamp) anywhere in the rendered body — only YYYY-MM-DD decision dates',
  );
});

await check('dp-4: groups by scope (alphabetical) then tag (untagged last), newest-first inside each group', async () => {
  const content = fs.readFileSync(decisionIndexFilePath(dp4Root), 'utf8');

  const billingIdx = content.indexOf('## billing');
  const checkoutIdx = content.indexOf('## checkout');
  const retroIdx = content.indexOf('## retro-scope');
  assert(billingIdx !== -1 && checkoutIdx !== -1 && retroIdx !== -1, 'expected all three scope headings present');
  assert(billingIdx < checkoutIdx && checkoutIdx < retroIdx, 'scopes render in alphabetical order (billing, checkout, retro-scope)');
  assert(!content.includes('## repo'), 'legacy event\'s original "repo" scope must not render once overlaid away to retro-scope');

  const billingSection = extractScopeSection(content, 'billing');
  const b1Line = billingSection.indexOf(dp4B1.id.slice(0, 8));
  const b2Line = billingSection.indexOf(dp4B2.id.slice(0, 8));
  assert(b1Line !== -1 && b2Line !== -1, 'both billing decisions present in the billing scope section');
  assert(b2Line < b1Line, 'newest-first within a group: B2 (later date) renders before B1');

  const checkoutSection = extractScopeSection(content, 'checkout');
  const flowIdx = checkoutSection.indexOf('### flow');
  const untaggedIdx = checkoutSection.indexOf('### untagged');
  assert(flowIdx !== -1 && untaggedIdx !== -1, 'checkout scope has both a tag subgroup and an untagged subgroup');
  assert(flowIdx < untaggedIdx, 'tag subgroups render before the untagged subgroup (untagged last)');
  assert(
    checkoutSection.indexOf(dp4LegacyC.id.slice(0, 8)) > untaggedIdx,
    'the untagged legacy checkout decision renders inside the untagged subgroup',
  );
});

await check('dp-4: superseded decisions never appear in the index; the supersede event itself renders under its inherited scope/tag', async () => {
  const content = fs.readFileSync(decisionIndexFilePath(dp4Root), 'utf8');
  assert(!content.includes(dp4STarget.id.slice(0, 8)), 'the superseded target must never appear in the rendered index');
  const billingSection = extractScopeSection(content, 'billing');
  assert(
    billingSection.includes(dp4SEvent.id.slice(0, 8)) && billingSection.includes('S replaced'),
    'the supersede event renders under scope "billing" (inherited from its superseded target), tag "invoices"',
  );
});

await check('dp-4: archived decisions are excluded by default and included only with --all', async () => {
  const withoutAll = fs.readFileSync(decisionIndexFilePath(dp4Root), 'utf8');
  assert(!withoutAll.includes(dp4ATarget.id.slice(0, 8)), 'archived decision excluded from the default render');

  const allResult = renderDecisionIndex(dp4Root, { all: true });
  const withAll = fs.readFileSync(decisionIndexFilePath(dp4Root), 'utf8');
  assert(withAll.includes(dp4ATarget.id.slice(0, 8)), 'archived decision included once rendered with --all');
  const renderedLineCount = (withAll.match(/^- /gm) || []).length;
  assert(
    allResult.count === renderedLineCount,
    `expected reported count (${allResult.count}) to match the number of rendered "- " lines (${renderedLineCount})`,
  );

  // Restore the default (non-all) rendering for the checks that follow.
  renderDecisionIndex(dp4Root, {});
});

await check('dp-4: a retro-tagged legacy event renders under its overlaid scope/tags, never under "untagged"', async () => {
  const content = fs.readFileSync(decisionIndexFilePath(dp4Root), 'utf8');
  const retroSection = extractScopeSection(content, 'retro-scope');
  assert(retroSection, 'expected a "retro-scope" scope section (the overlaid scope, not "repo")');
  const retroOnlyIdx = retroSection.indexOf('### retro-only');
  assert(retroOnlyIdx !== -1, 'expected a "retro-only" tag subgroup (the overlaid tag)');
  const legacyRLineIdx = retroSection.indexOf(dp4LegacyR.id.slice(0, 8));
  assert(legacyRLineIdx > retroOnlyIdx, 'the retro-tagged legacy event renders inside the "retro-only" tag subgroup');
  const untaggedIdx = retroSection.indexOf('### untagged');
  assert(
    untaggedIdx === -1 || legacyRLineIdx < untaggedIdx || legacyRLineIdx > retroSection.indexOf('### untagged') + 999999,
    'sanity guard (see next stronger assertion)',
  );
  // Stronger, unambiguous check: the legacy event's line never sits inside an "untagged" subgroup at all.
  if (untaggedIdx !== -1) {
    const afterUntagged = retroSection.slice(untaggedIdx);
    assert(!afterUntagged.includes(dp4LegacyR.id.slice(0, 8)), 'retro-tagged legacy event must not appear under "untagged"');
  }
});

await check('dp-4: only the first line of a multi-line decision text is rendered', async () => {
  const content = fs.readFileSync(decisionIndexFilePath(dp4Root), 'utf8');
  assert(content.includes('Multi-line decision'), 'first line of the decision text must render');
  assert(!content.includes('Second line detail'), 'second line of the decision text must never render');
});

await check('dp-4: one line per decision follows the "short8 · YYYY-MM-DD · first line" format', async () => {
  const content = fs.readFileSync(decisionIndexFilePath(dp4Root), 'utf8');
  const short8 = dp4C1.id.slice(0, 8);
  const re = new RegExp(`^- ${short8} · \\d{4}-\\d{2}-\\d{2} · C1 checkout decision$`, 'm');
  assert(re.test(content), `expected a line matching "short8 · date · text" for C1, got:\n${content}`);
});

await check('dp-4: two consecutive renders over the same (unchanged) store are byte-identical — no wall-clock in the body', async () => {
  const first = renderDecisionIndex(dp4Root, {});
  const firstBytes = fs.readFileSync(decisionIndexFilePath(dp4Root), 'utf8');
  const second = renderDecisionIndex(dp4Root, {});
  const secondBytes = fs.readFileSync(decisionIndexFilePath(dp4Root), 'utf8');
  assert(firstBytes === secondBytes, 'rendering twice over an unchanged store must produce byte-identical output');
  assert(first.content === second.content, 'in-memory computed content must also be byte-identical across two renders');
});

await check('dp-4: --check reports no drift immediately after a render, and detects hand-edited drift with a non-zero exit', async () => {
  renderDecisionIndex(dp4Root, {});
  const clean = decisionIndexDrift(dp4Root, {});
  assert(clean.drift === false, `expected no drift immediately after a render, got ${JSON.stringify(clean)}`);

  const cleanCli = await runBee(['decisions', 'render', '--check', '--json'], dp4Root);
  assert(cleanCli.status === 0, `CLI --check on a fresh render should exit 0, got ${cleanCli.status} :: ${cleanCli.stderr || cleanCli.stdout}`);

  fs.appendFileSync(decisionIndexFilePath(dp4Root), '\nHAND-EDITED DRIFT MARKER\n', 'utf8');
  const dirty = decisionIndexDrift(dp4Root, {});
  assert(dirty.drift === true, 'hand-edited file must be detected as drifted');

  const dirtyCli = await runBee(['decisions', 'render', '--check', '--json'], dp4Root);
  assert(dirtyCli.status !== 0, 'CLI --check on a hand-edited file must exit non-zero');
  const onDiskAfterCheck = fs.readFileSync(decisionIndexFilePath(dp4Root), 'utf8');
  assert(onDiskAfterCheck.includes('HAND-EDITED DRIFT MARKER'), '--check must never write — the hand-edited drift marker must survive');

  // A real (non-check) render fixes the drift.
  renderDecisionIndex(dp4Root, {});
  const fixed = decisionIndexDrift(dp4Root, {});
  assert(fixed.drift === false, 'a real render clears the previously detected drift');
});

await check('dp-4: empty store — render never throws, produces a valid file with no decisions listed', async () => {
  const emptyRoot = makeTempRepo();
  try {
    assert(!fs.existsSync(path.join(emptyRoot, '.bee', 'decisions.jsonl')), 'precondition: no decisions.jsonl exists yet');
    const result = renderDecisionIndex(emptyRoot, {});
    assert(fs.existsSync(decisionIndexFilePath(emptyRoot)), 'index.md should exist even for an empty store');
    assert(result.count === 0, `expected a count of 0 for an empty store, got ${JSON.stringify(result)}`);
    const content = fs.readFileSync(decisionIndexFilePath(emptyRoot), 'utf8');
    assert(/no active decisions/i.test(content), 'empty-store render should say so in plain text');

    const cliRun = await runBee(['decisions', 'render', '--json'], emptyRoot);
    assert(cliRun.status === 0, `CLI render on an empty store exited ${cliRun.status} :: ${cliRun.stderr || cliRun.stdout}`);
  } finally {
    fs.rmSync(emptyRoot, { recursive: true, force: true });
  }
});

await check('dp-4 CLI: decisions render --json writes docs/decisions/index.md and reports a count', async () => {
  const r = makeTempRepo();
  logDecision(r, { decision: 'CLI render fixture', rationale: 'x', scope: 'demo-scope', tags: ['demo-tag'] });
  const run = await runBee(['decisions', 'render', '--json'], r);
  assert(run.status === 0, `decisions render exited ${run.status} :: ${run.stderr || run.stdout}`);
  const payload = JSON.parse(run.stdout);
  assert(payload.count === 1, `expected count 1, got ${JSON.stringify(payload)}`);
  assert(fs.existsSync(decisionIndexFilePath(r)), 'index.md written to disk');
});

printSummaryAndExit();
