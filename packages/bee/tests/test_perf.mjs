// test_perf.mjs — self-contained, SYNCHRONOUS tests for lib/perf.mjs.
// Assertions run inline (no async bodies — a non-awaiting runner would pass those
// vacuously, critical-patterns 20260714). Fixtures are written to an os.tmpdir()
// temp tree; no literal home path anywhere; the real ~/.config/beehive is never
// touched (BEEHIVE_PERF_DIR is redirected).

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {
  encodeProjectDir,
  resolveTranscript,
  sliceEvents,
  aggregateUsage,
  runningTimeMs,
  detectParallel,
  walkSubagents,
  computeMetrics,
  globalPerfDir,
  globalPerfLogPath,
  humanizeMs,
  buildSection,
  appendSection,
  readSections,
  rollupTranscript,
  scanProjects,
  renderMatrixHtml,
  writeReport,
  projectName,
  syncSessionsToLog,
  readSessionRecords,
  upsertSessionRecords,
  sessionRecord,
  buildMatrixFromLog,
} from '../lib/perf.mjs';

let pass = 0;
let fail = 0;
function check(name, fn) {
  try {
    fn();
    pass += 1;
    console.log(`PASS ${name}`);
  } catch (err) {
    fail += 1;
    console.log(`FAIL ${name}: ${err && err.message ? err.message : err}`);
  }
}
function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}
function eq(a, b, msg) {
  if (a !== b) throw new Error(`${msg || 'not equal'}: ${JSON.stringify(a)} !== ${JSON.stringify(b)}`);
}

const iso = (ms) => new Date(ms).toISOString();
const BASE = Date.parse('2026-07-16T00:00:00.000Z');

// --- encodeProjectDir ---------------------------------------------------
check('encodeProjectDir mirrors CC / and . -> -', () => {
  eq(encodeProjectDir('/home/u/projects/goglbe/beegog'), '-home-u-projects-goglbe-beegog', 'slash encoding');
  eq(encodeProjectDir('/a.b/c'), '-a-b-c', 'dot encoding');
});

// --- aggregateUsage: dedup + per-model new/cached/total + <synthetic> ----
const usageEvents = [
  // r1 chunk 1 (smaller output) then r1 chunk 2 (final, larger output) — dedup keeps chunk 2
  { type: 'assistant', timestamp: iso(BASE + 1000), requestId: 'r1', message: { model: 'claude-opus-4-8', usage: { input_tokens: 2, output_tokens: 100, cache_creation_input_tokens: 10, cache_read_input_tokens: 1000 } } },
  { type: 'assistant', timestamp: iso(BASE + 1500), requestId: 'r1', message: { model: 'claude-opus-4-8', usage: { input_tokens: 2, output_tokens: 531, cache_creation_input_tokens: 10, cache_read_input_tokens: 1000 } } },
  { type: 'assistant', timestamp: iso(BASE + 2000), requestId: 'r2', message: { model: 'claude-haiku-4-5-20251001', usage: { input_tokens: 5, output_tokens: 50, cache_creation_input_tokens: 0, cache_read_input_tokens: 200 } } },
  // synthetic — excluded entirely
  { type: 'assistant', timestamp: iso(BASE + 2500), requestId: 'r3', message: { model: '<synthetic>', usage: { input_tokens: 9999, output_tokens: 9999 } } },
  { type: 'user', timestamp: iso(BASE + 3000), message: { role: 'user', content: [{ type: 'text', text: 'hi' }] } },
];

check('aggregateUsage dedups by requestId (keep max output)', () => {
  const agg = aggregateUsage(usageEvents);
  const opus = agg.models['claude-opus-4-8'];
  eq(opus.output, 531, 'opus output = final chunk only, not summed chunks');
  eq(opus.cache_read, 1000, 'opus cache_read counted once');
});

check('aggregateUsage per-model new/cached/total math', () => {
  const agg = aggregateUsage(usageEvents);
  const opus = agg.models['claude-opus-4-8'];
  eq(opus.new, 2 + 531 + 10, 'opus new = input+output+cache_write');
  eq(opus.cached, 1000, 'opus cached = cache_read');
  eq(opus.total, 543 + 1000, 'opus total = new+cached');
  const haiku = agg.models['claude-haiku-4-5-20251001'];
  eq(haiku.new, 5 + 50 + 0, 'haiku new');
  eq(haiku.cached, 200, 'haiku cached');
  eq(agg.totals.new, 543 + 55, 'totals.new across models');
  eq(agg.totals.total, 1543 + 255, 'totals.total across models');
});

check('aggregateUsage excludes <synthetic>', () => {
  const agg = aggregateUsage(usageEvents);
  assert(!('<synthetic>' in agg.models), '<synthetic> must not appear');
  assert(agg.totals.new < 9999, 'synthetic tokens excluded from totals');
});

// Falsifiability: prove the dedup assertion is not vacuous — summing WITHOUT
// dedup (both r1 chunks) would give a strictly larger output than the deduped result.
check('falsifiability: dedup removes the duplicate (naive sum differs)', () => {
  const agg = aggregateUsage(usageEvents);
  const naiveOpusOutput = usageEvents
    .filter((e) => e.type === 'assistant' && e.message.model === 'claude-opus-4-8')
    .reduce((s, e) => s + e.message.usage.output_tokens, 0); // 100 + 531 = 631
  assert(naiveOpusOutput !== agg.models['claude-opus-4-8'].output, 'dedup must change the number vs naive sum');
  eq(naiveOpusOutput, 631, 'naive (buggy) sum is 631; deduped is 531');
});

// --- sliceEvents inclusive boundaries -----------------------------------
check('sliceEvents keeps boundary events inclusive', () => {
  const evs = [
    { timestamp: iso(BASE + 0) },
    { timestamp: iso(BASE + 500) },
    { timestamp: iso(BASE + 1000) },
    { timestamp: iso(BASE + 1500) },
  ];
  const out = sliceEvents(evs, BASE + 0, BASE + 1000);
  eq(out.length, 3, 'start and end boundaries included');
});

// --- runningTimeMs: turn_duration primary + gap fallback -----------------
check('runningTimeMs sums turn_duration durationMs (idle excluded by harness)', () => {
  const evs = [
    { type: 'system', subtype: 'turn_duration', durationMs: 1000, timestamp: iso(BASE + 1000) },
    { type: 'assistant', timestamp: iso(BASE + 2000), requestId: 'x', message: { model: 'claude-opus-4-8', usage: {} } },
    { type: 'system', subtype: 'turn_duration', durationMs: 2000, timestamp: iso(BASE + 999999) },
  ];
  eq(runningTimeMs(evs), 3000, 'sum of durationMs = 1000+2000');
});

check('runningTimeMs fallback sums gaps below idle threshold, excludes long idle', () => {
  const evs = [
    { type: 'assistant', timestamp: iso(BASE + 0), message: {} },
    { type: 'assistant', timestamp: iso(BASE + 1000), message: {} }, // gap 1000
    { type: 'assistant', timestamp: iso(BASE + 2000), message: {} }, // gap 1000
    { type: 'assistant', timestamp: iso(BASE + 600000), message: {} }, // gap 598000 > 300000 idle -> excluded
    { type: 'assistant', timestamp: iso(BASE + 600500), message: {} }, // gap 500
  ];
  eq(runningTimeMs(evs), 2500, 'active = 1000+1000+500, idle gap dropped');
});

// --- global dir resolution (cross-platform, no literal home) -------------
check('globalPerfDir honors BEEHIVE_PERF_DIR > XDG > home fallback', () => {
  eq(globalPerfDir({ BEEHIVE_PERF_DIR: '/tmp/pd' }, '/fake/home'), '/tmp/pd', 'env override wins');
  eq(globalPerfDir({ XDG_CONFIG_HOME: '/x/cfg' }, '/fake/home'), path.join('/x/cfg', 'beehive'), 'xdg');
  eq(globalPerfDir({}, '/fake/home'), path.join('/fake/home', '.config', 'beehive'), 'home fallback (injected homedir)');
  eq(globalPerfLogPath({ BEEHIVE_PERF_DIR: '/tmp/pd' }, '/fake/home'), path.join('/tmp/pd', 'performance.jsonl'), 'log path');
});

check('perf.mjs source hard-codes no literal home path', () => {
  const src = fs.readFileSync(new URL('../lib/perf.mjs', import.meta.url), 'utf8');
  assert(!/\/home\/[a-z]/i.test(src), 'no /home/<user> literal in source');
  assert(!/[Cc]:\\\\Users/.test(src), 'no C:\\Users literal in source');
});

// --- humanizeMs ---------------------------------------------------------
check('humanizeMs renders compact durations', () => {
  eq(humanizeMs(0), '0s', 'zero');
  eq(humanizeMs(2500), '3s', 'rounds to seconds');
  eq(humanizeMs(3661000), '1h1m1s', 'h/m/s');
});

// --- filesystem fixtures: transcript + sidecar --------------------------
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'perf-test-'));
const projectsRoot = path.join(tmp, 'projects');
const projectPath = '/work/demo';
const encDir = path.join(projectsRoot, encodeProjectDir(projectPath));
fs.mkdirSync(encDir, { recursive: true });
const sessionId = 'sess-1111';
const transcript = path.join(encDir, `${sessionId}.jsonl`);
fs.writeFileSync(transcript, usageEvents.concat([
  { type: 'system', subtype: 'turn_duration', durationMs: 4000, timestamp: iso(BASE + 2600) },
]).map((e) => JSON.stringify(e)).join('\n') + '\n', 'utf8');

// sidecar: two OVERLAPPING agents (parallel) + a meta file each
const sideDir = path.join(encDir, sessionId, 'subagents');
fs.mkdirSync(sideDir, { recursive: true });
function writeAgent(hex, model, t0, t1, out) {
  const lines = [
    { type: 'assistant', isSidechain: true, timestamp: iso(t0), requestId: `${hex}-a`, message: { model, usage: { input_tokens: 1, output_tokens: out, cache_creation_input_tokens: 0, cache_read_input_tokens: 10 } } },
    { type: 'assistant', isSidechain: true, timestamp: iso(t1), requestId: `${hex}-b`, message: { model, usage: { input_tokens: 1, output_tokens: out, cache_creation_input_tokens: 0, cache_read_input_tokens: 10 } } },
  ];
  fs.writeFileSync(path.join(sideDir, `agent-${hex}.jsonl`), lines.map((l) => JSON.stringify(l)).join('\n') + '\n', 'utf8');
  fs.writeFileSync(path.join(sideDir, `agent-${hex}.meta.json`), JSON.stringify({ agentType: 'Explore', toolUseId: `tu-${hex}`, spawnDepth: 1 }), 'utf8');
}
// agent aa: [BASE+1000, BASE+5000]; agent bb: [BASE+3000, BASE+7000] -> overlap at 3000..5000
writeAgent('aa', 'claude-sonnet-5', BASE + 1000, BASE + 5000, 40);
writeAgent('bb', 'claude-sonnet-5', BASE + 3000, BASE + 7000, 60);

check('resolveTranscript finds explicit session file', () => {
  eq(resolveTranscript(projectsRoot, projectPath, { sessionId }), transcript, 'explicit id');
  eq(resolveTranscript(projectsRoot, projectPath, {}), transcript, 'newest-mtime (only one)');
  eq(resolveTranscript(projectsRoot, projectPath, { sessionId: 'nope' }), null, 'missing id -> null');
});

check('walkSubagents attributes worker tokens from sidecar', () => {
  const sessionDir = transcript.replace(/\.jsonl$/, '');
  const sub = walkSubagents(sessionDir, BASE, BASE + 999999);
  eq(sub.agents.length, 2, 'two agents in window');
  const sonnet = sub.models['claude-sonnet-5'];
  eq(sonnet.output, 40 + 40 + 60 + 60, 'worker output summed across both agents (2 msgs each)');
  eq(sonnet.cached, 10 * 4, 'worker cache_read summed');
});

check('detectParallel true for overlapping sidecar spans', () => {
  const sessionDir = transcript.replace(/\.jsonl$/, '');
  const sub = walkSubagents(sessionDir, BASE, BASE + 999999);
  assert(detectParallel(sub.agents, []) === true, 'overlapping agents -> parallel');
});

check('detectParallel false for sequential spans', () => {
  const seq = [{ startMs: 0, endMs: 100 }, { startMs: 200, endMs: 300 }];
  eq(detectParallel(seq, []), false, 'no overlap -> not parallel');
});

check('detectParallel true for >=2 Agent tool_use in one turn', () => {
  const parent = [{ type: 'assistant', message: { content: [
    { type: 'tool_use', name: 'Agent', id: 't1' },
    { type: 'tool_use', name: 'Agent', id: 't2' },
  ] } }];
  eq(detectParallel([], parent), true, 'two Agent calls one turn -> parallel');
});

check('computeMetrics end-to-end (main + subagent + parallel + running time)', () => {
  const m = computeMetrics(transcript, BASE, BASE + 999999);
  eq(m.models['claude-opus-4-8'].output, 531, 'main opus deduped');
  eq(m.running_time_ms, 4000, 'running time from the one turn_duration in window');
  eq(m.parallel, true, 'overlapping subagents detected');
  eq(m.subagent_count, 2, 'two subagents');
  assert(m.subagent_models['claude-sonnet-5'].total > 0, 'worker tokens present');
});

// --- graceful degradation -----------------------------------------------
check('computeMetrics tolerates null / missing transcript (no throw, zeroed)', () => {
  const m1 = computeMetrics(null, BASE, BASE + 1000);
  eq(m1.event_count, 0, 'null transcript -> 0 events');
  eq(m1.running_time_ms, 0, 'null transcript -> 0 running time');
  eq(m1.parallel, false, 'null transcript -> not parallel');
  const m2 = computeMetrics(path.join(encDir, 'does-not-exist.jsonl'), BASE, BASE + 1000);
  eq(m2.event_count, 0, 'missing file -> 0 events, no throw');
});

// --- section record + append/read roundtrip (BEEHIVE_PERF_DIR temp) ------
check('buildSection + appendSection + readSections roundtrip', () => {
  const perfDir = path.join(tmp, 'perfhome');
  const env = { BEEHIVE_PERF_DIR: perfDir };
  const metrics = computeMetrics(transcript, BASE, BASE + 999999);
  const rec = buildSection({
    label: 'demo work',
    note: 'testing',
    projectPath,
    branch: 'main',
    sessionId,
    startTs: iso(BASE),
    endTs: iso(BASE + 999999),
    metrics,
  });
  eq(rec.schema, 'bee-perf/v1', 'schema tag');
  eq(rec.parallel, true, 'section carries parallel');
  assert(typeof rec.running_time_human === 'string', 'human running time present');
  eq(rec.project, projectPath, 'project tagged');
  eq(rec.branch, 'main', 'branch tagged');
  const file = appendSection(rec, env);
  assert(fs.existsSync(file), 'log file created under BEEHIVE_PERF_DIR');
  appendSection(buildSection({ label: 'second', projectPath, startTs: iso(BASE), endTs: iso(BASE + 1), metrics: {} }), env);
  const all = readSections({}, env);
  eq(all.length, 2, 'two sections appended');
  const last1 = readSections({ limit: 1 }, env);
  eq(last1.length, 1, 'limit honored');
  eq(last1[0].label, 'second', 'limit returns most recent');
});

// --- cross-project scan + HTML matrix -----------------------------------
// Build a fake projects root with two projects, each with session transcripts
// carrying a `cwd` label, then assert per-project rollups, cache reuse, and HTML.
const scanRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'perf-scan-'));
function writeSession(projCwd, sessId, model, out, cacheRead, withTurn) {
  const enc = encodeProjectDir(projCwd);
  const dir = path.join(scanRoot, enc);
  fs.mkdirSync(dir, { recursive: true });
  const evs = [
    { type: 'user', timestamp: iso(BASE + 0), cwd: projCwd, message: { role: 'user', content: [{ type: 'text', text: 'go' }] } },
    { type: 'assistant', timestamp: iso(BASE + 1000), requestId: `${sessId}-r`, cwd: projCwd, message: { model, usage: { input_tokens: 10, output_tokens: out, cache_creation_input_tokens: 5, cache_read_input_tokens: cacheRead } } },
  ];
  if (withTurn) evs.push({ type: 'system', subtype: 'turn_duration', durationMs: 5000, timestamp: iso(BASE + 2000) });
  fs.writeFileSync(path.join(dir, `${sessId}.jsonl`), evs.map((e) => JSON.stringify(e)).join('\n') + '\n', 'utf8');
}
writeSession('/work/alpha', 'a1', 'claude-opus-4-8', 100, 1000, true);
writeSession('/work/alpha', 'a2', 'claude-sonnet-5', 50, 500, true);
writeSession('/work/beta', 'b1', 'claude-opus-4-8', 200, 2000, false);

check('rollupTranscript reads the cwd label and full-file totals', () => {
  const f = path.join(scanRoot, encodeProjectDir('/work/beta'), 'b1.jsonl');
  const r = rollupTranscript(f);
  eq(r.cwd, '/work/beta', 'cwd label from event');
  eq(r.models['claude-opus-4-8'].output, 200, 'full-file output');
});

check('scanProjects rolls sessions up per project', () => {
  const scan = scanProjects(scanRoot, { cachePath: path.join(scanRoot, 'cache.json') });
  eq(scan.projects.length, 2, 'two projects');
  const alpha = scan.projects.find((p) => p.project === '/work/alpha');
  eq(alpha.sessions, 2, 'alpha has 2 sessions');
  eq(alpha.running_time_ms, 10000, 'alpha active time = 5000+5000 from turn_duration');
  assert(alpha.models['claude-opus-4-8'] && alpha.models['claude-sonnet-5'], 'both models present in alpha');
  eq(scan.totals.projects, 2, 'totals project count');
  assert(scan.totals.total_tokens > 0, 'totals token count');
});

check('scanProjects uses the mtime+size cache on a repeat scan (no re-parse)', () => {
  const cp = path.join(scanRoot, 'cache2.json');
  const first = scanProjects(scanRoot, { cachePath: cp });
  assert(first.cache_stats.misses >= 3, 'first scan parses every transcript');
  const second = scanProjects(scanRoot, { cachePath: cp });
  eq(second.cache_stats.misses, 0, 'repeat scan re-parses nothing');
  assert(second.cache_stats.hits >= 3, 'repeat scan is all cache hits');
});

check('scanProjects --since filters to recently-active sessions', () => {
  const future = new Date(BASE + 10 * 86400000).toISOString();
  const scan = scanProjects(scanRoot, { since: future });
  eq(scan.projects.length, 0, 'nothing active after the window');
});

check('renderMatrixHtml is self-contained and lists each project', () => {
  const scan = scanProjects(scanRoot, {});
  const html = renderMatrixHtml(scan);
  assert(html.startsWith('<!doctype html>'), 'is a full HTML document');
  assert(html.includes('/work/alpha') && html.includes('/work/beta'), 'lists both projects');
  assert(!/https?:\/\//.test(html), 'no external URLs (self-contained)');
  assert(/prefers-color-scheme/.test(html), 'theme-aware');
});

check('writeReport writes the HTML file', () => {
  const scan = scanProjects(scanRoot, {});
  const out = path.join(scanRoot, 'report.html');
  const file = writeReport(scan, { out });
  eq(file, out, 'returns the path');
  assert(fs.readFileSync(out, 'utf8').includes('bee performance'), 'file has the report');
});

check('scanProjects tolerates a missing root (no throw)', () => {
  const scan = scanProjects(path.join(scanRoot, 'does-not-exist'), {});
  eq(scan.projects.length, 0, 'missing root -> empty, no throw');
});

// --- persistent store (performance.jsonl) + basename grouping ------------
check('projectName returns the last folder segment', () => {
  eq(projectName('/home/u/projects/goglbe/beegog'), 'beegog', 'basename');
  eq(projectName('/work/alpha/'), 'alpha', 'trailing slash ignored');
});

// a second project sharing beta's basename under a different path
writeSession('/other/beta', 'b2', 'claude-opus-4-8', 30, 300, true);
const storeEnv = { BEEHIVE_PERF_DIR: path.join(scanRoot, 'store') };

check('syncSessionsToLog writes session rows; report reads them grouped by basename', () => {
  const res = syncSessionsToLog(scanRoot, { cachePath: path.join(scanRoot, 'sync-cache.json'), env: storeEnv });
  assert(res.sessions >= 4, 'all sessions written to the log');
  const recs = readSessionRecords(storeEnv);
  assert(recs.every((r) => r.kind === 'session' && r.project_name), 'records are session rows with a basename');
  const matrix = buildMatrixFromLog(storeEnv);
  // /work/beta and /other/beta collapse into one "beta" row (basename grouping)
  const beta = matrix.projects.find((p) => p.project === 'beta');
  assert(beta, 'a basename-grouped "beta" project exists');
  eq(beta.sessions, 2, 'both beta paths merged under one basename');
  assert(beta.paths.includes('/work/beta') && beta.paths.includes('/other/beta'), 'full paths retained for the tooltip');
  const alpha = matrix.projects.find((p) => p.project === 'alpha');
  eq(alpha.sessions, 2, 'alpha keeps its 2 sessions');
});

check('upsertSessionRecords dedups by session_id (rewrite, not append)', () => {
  const env = { BEEHIVE_PERF_DIR: path.join(scanRoot, 'store2') };
  const rec = sessionRecord(rollupTranscript(path.join(scanRoot, encodeProjectDir('/work/beta'), 'b1.jsonl')));
  upsertSessionRecords([rec], env);
  upsertSessionRecords([rec], env); // second write must REPLACE, not duplicate
  eq(readSessionRecords(env).length, 1, 'session recorded exactly once after two upserts');
});

check('buildMatrixFromLog is a pure read of the log (empty when nothing synced)', () => {
  const env = { BEEHIVE_PERF_DIR: path.join(scanRoot, 'store3') };
  eq(buildMatrixFromLog(env).projects.length, 0, 'no log -> empty matrix, no scan');
});

console.log(`\ntest_perf: ${pass} passed, ${fail} failed`);
process.exit(fail === 0 ? 0 : 1);
