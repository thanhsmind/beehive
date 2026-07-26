// perf.mjs — global cross-project performance log for bee.
//
// Each "section" summarizes one piece of work: which models ran, the per-model
// token breakdown (new / cached / total), whether the work went parallel, and
// the section's RUNNING time (active execution, not idle/wall-clock). All metrics
// are recovered post-hoc from the Claude Code session transcript on disk — the
// only trustworthy source (the agent cannot self-report mid-flight token counts).
//
// The pure functions (sliceEvents, aggregateUsage, detectParallel, runningTimeMs,
// buildSection, globalPerfDir, humanizeMs) take already-parsed data so they are
// unit-testable with no filesystem. The I/O helpers (resolveTranscript,
// walkSubagents, computeMetrics, appendSection, readSections) read real paths.
//
// Node 18+, Windows-safe. Only bee.mjs imports this module (never
// command-registry.mjs — that keeps perf.mjs out of the write-guard fixture's
// hand-listed VENDORED_LIB_MODULES, critical-patterns 20260712/20260714).

import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { ensureDir, appendJsonl, readJsonl, readJson, writeJsonAtomic } from './fsutil.mjs';

const SYNTHETIC_MODEL = '<synthetic>';
const DEFAULT_IDLE_THRESHOLD_MS = 300000; // 5 min — a longer gap is "alive but idle"

// --- transcript location -------------------------------------------------

// encodeProjectDir — mirror Claude Code's project-dir encoding: the absolute
// project path with '/', '\' and '.' each replaced by '-' (a leading separator
// becomes the leading '-'). e.g. /a/b/c -> -a-b-c.
export function encodeProjectDir(projectPath) {
  return String(projectPath).replace(/[\\/.]/g, '-');
}

// claudeProjectsRoot — where Claude Code stores per-project transcripts.
// Honors CLAUDE_CONFIG_DIR; defaults to <home>/.claude/projects.
export function claudeProjectsRoot(env = process.env, homedir = os.homedir()) {
  const base = env.CLAUDE_CONFIG_DIR || path.join(homedir, '.claude');
  return path.join(base, 'projects');
}

// resolveTranscript — the session transcript file for a project. With sessionId,
// return <root>/<enc>/<sessionId>.jsonl (or null if absent). Otherwise the
// newest-mtime top-level *.jsonl in that dir (the live session), or null.
//
// hardening-1-7-10 D5 (Codex session bridge): `transcriptPath` — when it is a
// non-empty string AND exists on disk — is returned directly, BEFORE any
// layout math runs at all (so it works even when `projectsRoot`/`projectPath`
// don't apply to this runtime, e.g. a real Codex rollout file living nowhere
// near Claude's encoded-project-dir layout). This is what makes a session's
// own stored transcript_path (claims.mjs createSession/recovery.mjs) real
// transcript resolution instead of a relabeled Claude-layout guess. Absent or
// non-existent transcriptPath falls straight through to the pre-existing
// layout math below — byte-identical to before this cell.
export function resolveTranscript(projectsRoot, projectPath, { sessionId, transcriptPath } = {}) {
  if (typeof transcriptPath === 'string' && transcriptPath.trim() && fs.existsSync(transcriptPath.trim())) {
    return transcriptPath.trim();
  }
  if (!projectsRoot || !projectPath) return null;
  const dir = path.join(projectsRoot, encodeProjectDir(projectPath));
  if (sessionId) {
    const file = path.join(dir, `${sessionId}.jsonl`);
    return fs.existsSync(file) ? file : null;
  }
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return null;
  }
  let best = null;
  let bestMtime = -Infinity;
  for (const e of entries) {
    if (!e.isFile() || !e.name.endsWith('.jsonl')) continue;
    const full = path.join(dir, e.name);
    let m;
    try {
      m = fs.statSync(full).mtimeMs;
    } catch {
      continue;
    }
    if (m > bestMtime) {
      bestMtime = m;
      best = full;
    }
  }
  return best;
}

// --- pure event helpers --------------------------------------------------

function toMs(v) {
  if (v == null) return NaN;
  return typeof v === 'string' ? Date.parse(v) : Number(v);
}

function eventMs(event) {
  return event && typeof event.timestamp === 'string' ? Date.parse(event.timestamp) : NaN;
}

function num(v) {
  return Number.isFinite(v) ? v : 0;
}

function zeroModel() {
  return { input: 0, output: 0, cache_write: 0, cache_read: 0, new: 0, cached: 0, total: 0 };
}

// new = fresh input + generated output + cache writes (all billed at full/premium);
// cached = cache reads (billed at ~1/10). total = new + cached.
function finalizeModel(m) {
  m.new = m.input + m.output + m.cache_write;
  m.cached = m.cache_read;
  m.total = m.new + m.cached;
  return m;
}

// sliceEvents — keep events whose top-level ISO timestamp is within [start,end]
// inclusive. start/end may be ISO strings or epoch-ms numbers.
export function sliceEvents(events, start, end) {
  const s = toMs(start);
  const e = toMs(end);
  return (events || []).filter((ev) => {
    const t = eventMs(ev);
    if (Number.isNaN(t)) return false;
    return t >= s && t <= e;
  });
}

// aggregateUsage — per-model token totals over assistant events, deduped by
// top-level requestId (streamed chunks repeat one id; keep the record with the
// largest output_tokens). Model '<synthetic>' is excluded (local/interrupt msgs).
export function aggregateUsage(events) {
  const byReq = new Map();
  const noReqId = [];
  for (const ev of events || []) {
    if (!ev || ev.type !== 'assistant') continue;
    const msg = ev.message || {};
    const model = msg.model;
    if (!model || model === SYNTHETIC_MODEL) continue;
    const usage = msg.usage || {};
    const rec = {
      model,
      input: num(usage.input_tokens),
      output: num(usage.output_tokens),
      cache_write: num(usage.cache_creation_input_tokens),
      cache_read: num(usage.cache_read_input_tokens),
    };
    const rid = ev.requestId;
    if (rid) {
      const prev = byReq.get(rid);
      if (!prev || rec.output > prev.output) byReq.set(rid, rec);
    } else {
      noReqId.push(rec);
    }
  }
  const models = {};
  const totals = zeroModel();
  for (const r of [...byReq.values(), ...noReqId]) {
    const m = models[r.model] || (models[r.model] = zeroModel());
    m.input += r.input;
    m.output += r.output;
    m.cache_write += r.cache_write;
    m.cache_read += r.cache_read;
    totals.input += r.input;
    totals.output += r.output;
    totals.cache_write += r.cache_write;
    totals.cache_read += r.cache_read;
  }
  for (const m of Object.values(models)) finalizeModel(m);
  finalizeModel(totals);
  return { models, modelList: Object.keys(models), totals };
}

// runningTimeMs — active execution time in the window. Primary: sum the
// harness-emitted system/turn_duration durationMs (already excludes idle waits).
// Fallback (no turn_duration events): sum consecutive-event gaps below the idle
// threshold, so a long user-away pause is never counted.
export function runningTimeMs(events, { idleThresholdMs = DEFAULT_IDLE_THRESHOLD_MS } = {}) {
  const turns = (events || []).filter(
    (e) => e && e.type === 'system' && e.subtype === 'turn_duration' && Number.isFinite(e.durationMs),
  );
  if (turns.length > 0) {
    return turns.reduce((sum, e) => sum + e.durationMs, 0);
  }
  const stamps = (events || [])
    .map(eventMs)
    .filter((t) => !Number.isNaN(t))
    .sort((a, b) => a - b);
  let sum = 0;
  for (let i = 1; i < stamps.length; i++) {
    const gap = stamps[i] - stamps[i - 1];
    if (gap > 0 && gap < idleThresholdMs) sum += gap;
  }
  return sum;
}

// detectParallel — true when >=2 subagent time-spans overlap, OR any single
// assistant turn dispatched >=2 Agent tool_use blocks.
export function detectParallel(agents = [], parentEvents = []) {
  const spans = (agents || [])
    .filter((a) => Number.isFinite(a.startMs) && Number.isFinite(a.endMs))
    .map((a) => [a.startMs, a.endMs])
    .sort((x, y) => x[0] - y[0]);
  for (let i = 1; i < spans.length; i++) {
    if (spans[i][0] <= spans[i - 1][1]) return true;
  }
  for (const ev of parentEvents || []) {
    if (!ev || ev.type !== 'assistant') continue;
    const content = ev.message && Array.isArray(ev.message.content) ? ev.message.content : [];
    const agentCalls = content.filter((b) => b && b.type === 'tool_use' && b.name === 'Agent');
    if (agentCalls.length >= 2) return true;
  }
  return false;
}

// --- subagent sidecar walk ----------------------------------------------

// walkSubagents — attribute worker cost from <sessionDir>/subagents/agent-*.jsonl
// (+ .meta.json). An agent counts if its event span overlaps [start,end].
// Returns { models, totals, agents:[{file, agentType, model, startMs, endMs}] }.
export function walkSubagents(sessionDir, start, end) {
  const startMs = toMs(start);
  const endMs = toMs(end);
  const empty = { models: {}, totals: zeroModel(), agents: [] };
  if (!sessionDir) return empty;
  const subDir = path.join(sessionDir, 'subagents');
  let names;
  try {
    names = fs.readdirSync(subDir);
  } catch {
    return empty;
  }
  const models = {};
  const totals = zeroModel();
  const agents = [];
  for (const name of names) {
    if (!name.endsWith('.jsonl')) continue;
    const events = readJsonl(path.join(subDir, name));
    const stamps = events.map(eventMs).filter((t) => !Number.isNaN(t));
    if (stamps.length === 0) continue;
    const aStart = Math.min(...stamps);
    const aEnd = Math.max(...stamps);
    if (aEnd < startMs || aStart > endMs) continue; // no overlap with the window
    const agg = aggregateUsage(events);
    for (const [model, m] of Object.entries(agg.models)) {
      const acc = models[model] || (models[model] = zeroModel());
      acc.input += m.input;
      acc.output += m.output;
      acc.cache_write += m.cache_write;
      acc.cache_read += m.cache_read;
      totals.input += m.input;
      totals.output += m.output;
      totals.cache_write += m.cache_write;
      totals.cache_read += m.cache_read;
    }
    const meta = readJson(path.join(subDir, name.replace(/\.jsonl$/, '.meta.json')), {}) || {};
    agents.push({
      file: name,
      agentType: meta.agentType || null,
      models: agg.models,
      startMs: aStart,
      endMs: aEnd,
    });
  }
  for (const m of Object.values(models)) finalizeModel(m);
  finalizeModel(totals);
  return { models, totals, agents };
}

// computeMetrics — end-to-end for a transcript file + window. The sidecar dir is
// the transcript path minus its .jsonl suffix (<session-uuid>/ sits beside
// <session-uuid>.jsonl). Tolerates a null/missing transcript with zeroed output.
export function computeMetrics(transcriptFile, start, end, opts = {}) {
  const startMs = toMs(start);
  const endMs = toMs(end);
  const all = transcriptFile ? readJsonl(transcriptFile) : [];
  const windowed = sliceEvents(all, startMs, endMs);
  const usage = aggregateUsage(windowed);
  const sessionDir = transcriptFile ? transcriptFile.replace(/\.jsonl$/, '') : null;
  const sub = walkSubagents(sessionDir, startMs, endMs);
  return {
    models: usage.models,
    modelList: usage.modelList,
    totals: usage.totals,
    subagent_models: sub.models,
    subagent_totals: sub.totals,
    subagent_count: sub.agents.length,
    parallel: detectParallel(sub.agents, windowed),
    running_time_ms: runningTimeMs(windowed, opts),
    event_count: windowed.length,
  };
}

// --- global log location + section record --------------------------------

// globalPerfDir — the cross-project log directory. BEEHIVE_PERF_DIR wins (tests
// use it); else XDG_CONFIG_HOME/beehive; else <home>/.config/beehive. No literal
// home path is ever hard-coded — homedir is always injected/derived.
export function globalPerfDir(env = process.env, homedir = os.homedir()) {
  if (env.BEEHIVE_PERF_DIR) return env.BEEHIVE_PERF_DIR;
  if (env.XDG_CONFIG_HOME) return path.join(env.XDG_CONFIG_HOME, 'beehive');
  return path.join(homedir, '.config', 'beehive');
}

export function globalPerfLogPath(env = process.env, homedir = os.homedir()) {
  return path.join(globalPerfDir(env, homedir), 'performance.jsonl');
}

// humanizeMs — compact "1h2m3s" rendering of a running-time.
export function humanizeMs(ms) {
  if (!Number.isFinite(ms) || ms <= 0) return '0s';
  const s = Math.round(ms / 1000);
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  const parts = [];
  if (h) parts.push(`${h}h`);
  if (m) parts.push(`${m}m`);
  if (sec || parts.length === 0) parts.push(`${sec}s`);
  return parts.join('');
}

// buildSection — the JSON record appended to the global log (schema bee-perf/v1).
export function buildSection({
  label,
  note = null,
  projectPath,
  branch = null,
  sessionId = null,
  startTs,
  endTs,
  metrics = {},
}) {
  const startMs = toMs(startTs);
  const endMs = toMs(endTs);
  const runMs = num(metrics.running_time_ms);
  return {
    schema: 'bee-perf/v1',
    label: label || null,
    note,
    project: projectPath || null,
    branch,
    session_id: sessionId,
    started_at: typeof startTs === 'string' ? startTs : Number.isNaN(startMs) ? null : new Date(startMs).toISOString(),
    ended_at: typeof endTs === 'string' ? endTs : Number.isNaN(endMs) ? null : new Date(endMs).toISOString(),
    running_time_ms: runMs,
    running_time_human: humanizeMs(runMs),
    parallel: Boolean(metrics.parallel),
    subagent_count: num(metrics.subagent_count),
    models: metrics.models || {},
    subagent_models: metrics.subagent_models || {},
    event_count: num(metrics.event_count),
    logged_at: new Date().toISOString(),
  };
}

export function appendSection(record, env = process.env, homedir = os.homedir()) {
  const file = globalPerfLogPath(env, homedir);
  ensureDir(path.dirname(file));
  appendJsonl(file, record);
  return file;
}

export function readSections({ limit } = {}, env = process.env, homedir = os.homedir()) {
  const all = readJsonl(globalPerfLogPath(env, homedir));
  if (limit && limit > 0) return all.slice(-limit);
  return all;
}

// ─── cross-project scan + HTML matrix report ─────────────────────────────
// The matrix is derived directly from every project's session transcripts, so
// the operator never tracks anything: whatever real work happened is reflected.
// A per-transcript mtime+size cache keeps repeat scans (and the session-close
// hook's refresh) cheap — only changed transcripts are re-parsed.

export function scanCachePath(env = process.env, homedir = os.homedir()) {
  return path.join(globalPerfDir(env, homedir), 'cache', 'scan-cache.json');
}

export function reportHtmlPath(env = process.env, homedir = os.homedir()) {
  return path.join(globalPerfDir(env, homedir), 'performance.html');
}

// listProjectDirs — every top-level project directory under the Claude
// projects root (each is one encoded project path). Missing root → [].
export function listProjectDirs(projectsRoot) {
  let entries;
  try {
    entries = fs.readdirSync(projectsRoot, { withFileTypes: true });
  } catch {
    return [];
  }
  return entries
    .filter((e) => e.isDirectory())
    .map((e) => ({ dir: path.join(projectsRoot, e.name), encoded: e.name }));
}

// rollupTranscript — parse ONE session transcript once and return its full-file
// rollup (per-model tokens, running time, parallel, subagents, span, and the
// real project path from the first event carrying a `cwd`). Empty file → null.
export function rollupTranscript(transcriptFile) {
  const events = readJsonl(transcriptFile);
  if (!events.length) return null;
  const usage = aggregateUsage(events);
  const sessionDir = transcriptFile.replace(/\.jsonl$/, '');
  const sub = walkSubagents(sessionDir, 0, Number.MAX_SAFE_INTEGER);
  const stamps = events.map(eventMs).filter((t) => !Number.isNaN(t));
  let cwd = null;
  for (const e of events) {
    if (e && typeof e.cwd === 'string' && e.cwd) {
      cwd = e.cwd;
      break;
    }
  }
  return {
    sessionId: path.basename(transcriptFile, '.jsonl'),
    cwd,
    models: usage.models,
    subagent_models: sub.models,
    subagent_count: sub.agents.length,
    parallel: detectParallel(sub.agents, events),
    running_time_ms: runningTimeMs(events),
    event_count: events.length,
    started_ms: stamps.length ? Math.min(...stamps) : null,
    ended_ms: stamps.length ? Math.max(...stamps) : null,
  };
}

function addRawModels(dst, src) {
  for (const [m, v] of Object.entries(src || {})) {
    const a = dst[m] || (dst[m] = zeroModel());
    a.input += num(v.input);
    a.output += num(v.output);
    a.cache_write += num(v.cache_write);
    a.cache_read += num(v.cache_read);
  }
}

function newProjectAgg(label, encoded) {
  return {
    project: label,
    encoded,
    sessions: 0,
    parallel_sessions: 0,
    subagent_count: 0,
    event_count: 0,
    running_time_ms: 0,
    models: {},
    subagent_models: {},
    first_ms: null,
    last_ms: null,
    total_tokens: 0,
    new_tokens: 0,
    cached_tokens: 0,
  };
}

function mergeSessionIntoProject(p, r) {
  p.sessions += 1;
  addRawModels(p.models, r.models);
  addRawModels(p.subagent_models, r.subagent_models);
  p.running_time_ms += num(r.running_time_ms);
  if (r.parallel) p.parallel_sessions += 1;
  p.subagent_count += num(r.subagent_count);
  p.event_count += num(r.event_count);
  if (r.started_ms != null) p.first_ms = p.first_ms == null ? r.started_ms : Math.min(p.first_ms, r.started_ms);
  if (r.ended_ms != null) p.last_ms = p.last_ms == null ? r.ended_ms : Math.max(p.last_ms, r.ended_ms);
}

function finalizeProjectAgg(p) {
  for (const m of Object.values(p.models)) finalizeModel(m);
  for (const m of Object.values(p.subagent_models)) finalizeModel(m);
  let total = 0;
  let fresh = 0;
  let cached = 0;
  for (const m of Object.values(p.models)) {
    total += m.total;
    fresh += m.new;
    cached += m.cached;
  }
  p.total_tokens = total;
  p.new_tokens = fresh;
  p.cached_tokens = cached;
}

function totalsFor(projects) {
  const totals = { projects: projects.length, sessions: 0, running_time_ms: 0, total_tokens: 0, new_tokens: 0, cached_tokens: 0, models: {} };
  for (const p of projects) {
    totals.sessions += p.sessions;
    totals.running_time_ms += p.running_time_ms;
    totals.total_tokens += p.total_tokens;
    totals.new_tokens += p.new_tokens;
    totals.cached_tokens += p.cached_tokens;
    addRawModels(totals.models, p.models);
  }
  for (const m of Object.values(totals.models)) finalizeModel(m);
  return totals;
}

// collectSessionRollups — every session's full-file rollup across all project
// dirs, using the mtime+size cache (a hit whose mtime AND size match is reused).
export function collectSessionRollups(projectsRoot, { cachePath } = {}) {
  const cache = cachePath ? readJson(cachePath, {}) || {} : {};
  const nextCache = {};
  let hits = 0;
  let misses = 0;
  const rollups = [];
  for (const { dir, encoded } of listProjectDirs(projectsRoot)) {
    let files;
    try {
      files = fs.readdirSync(dir).filter((f) => f.endsWith('.jsonl'));
    } catch {
      continue;
    }
    for (const name of files) {
      const file = path.join(dir, name);
      let st;
      try {
        st = fs.statSync(file);
      } catch {
        continue;
      }
      const cached = cache[file];
      let rollup;
      if (cached && cached.mtime === st.mtimeMs && cached.size === st.size) {
        rollup = cached.rollup;
        hits += 1;
      } else {
        rollup = rollupTranscript(file);
        misses += 1;
      }
      nextCache[file] = { mtime: st.mtimeMs, size: st.size, rollup };
      if (rollup) rollups.push({ ...rollup, encoded });
    }
  }
  if (cachePath) {
    try {
      writeJsonAtomic(cachePath, nextCache);
    } catch {
      // cache is an optimization; a write failure never fails the scan.
    }
  }
  return { rollups, cache_stats: { hits, misses } };
}

// scanProjects — transcript-derived per-project matrix grouped by FULL path.
// The persistent store + report use buildMatrixFromLog (grouped by last folder);
// scanProjects stays as the raw derive-and-group view and the sync engine's core.
export function scanProjects(projectsRoot, { cachePath, since } = {}) {
  const { rollups, cache_stats } = collectSessionRollups(projectsRoot, { cachePath });
  const sinceMs = since != null ? toMs(since) : null;
  const byProject = new Map();
  for (const r of rollups) {
    if (sinceMs != null && (r.ended_ms == null || r.ended_ms < sinceMs)) continue;
    const label = r.cwd || r.encoded;
    let agg = byProject.get(label);
    if (!agg) {
      agg = newProjectAgg(label, r.encoded);
      agg.paths = [label];
      byProject.set(label, agg);
    }
    mergeSessionIntoProject(agg, r);
  }
  const projects = [...byProject.values()];
  for (const p of projects) finalizeProjectAgg(p);
  projects.sort((a, b) => b.total_tokens - a.total_tokens);
  return { generated_at: new Date().toISOString(), projects, totals: totalsFor(projects), cache_stats };
}

// ─── persistent store (performance.jsonl) + read/upsert/sync/matrix ──────
// The store is the source of truth for the report: session rollups are written
// here (by the session-close hook and by `perf sync`), and the HTML matrix is
// READ from here — never scanned live at view time. Records carry `kind:
// "session"` and are deduped by session_id.

// projectName — the last path segment (the folder the project lives in), used
// as the human-facing project label and grouping key.
export function projectName(p) {
  if (!p) return '(unknown)';
  const parts = String(p).replace(/[\\/]+$/, '').split(/[\\/]/);
  return parts[parts.length - 1] || String(p);
}

// sessionRecord — a performance.jsonl row (kind:"session") for one rollup.
export function sessionRecord(rollup, { branch = null } = {}) {
  const project = rollup.cwd || rollup.encoded || null;
  return {
    schema: 'bee-perf/v1',
    kind: 'session',
    session_id: rollup.sessionId,
    project,
    project_name: projectName(project),
    branch,
    started_at: rollup.started_ms != null ? new Date(rollup.started_ms).toISOString() : null,
    ended_at: rollup.ended_ms != null ? new Date(rollup.ended_ms).toISOString() : null,
    running_time_ms: num(rollup.running_time_ms),
    parallel: Boolean(rollup.parallel),
    subagent_count: num(rollup.subagent_count),
    models: rollup.models || {},
    subagent_models: rollup.subagent_models || {},
    event_count: num(rollup.event_count),
    started_ms: rollup.started_ms,
    ended_ms: rollup.ended_ms,
    logged_at: new Date().toISOString(),
  };
}

// readSessionRecords — session rows from performance.jsonl, deduped by
// session_id (keep the latest by logged_at). Other record kinds are ignored.
export function readSessionRecords(env = process.env, homedir = os.homedir()) {
  const bySession = new Map();
  for (const r of readJsonl(globalPerfLogPath(env, homedir))) {
    if (!r || r.kind !== 'session' || !r.session_id) continue;
    const prev = bySession.get(r.session_id);
    if (!prev || String(r.logged_at || '') >= String(prev.logged_at || '')) bySession.set(r.session_id, r);
  }
  return [...bySession.values()];
}

// upsertSessionRecords — write session rows into performance.jsonl, replacing
// any existing session row with the same session_id. Non-session records
// (manual spans) are preserved untouched.
export function upsertSessionRecords(records, env = process.env, homedir = os.homedir()) {
  const file = globalPerfLogPath(env, homedir);
  const ids = new Set(records.map((r) => r.session_id));
  const kept = readJsonl(file).filter((r) => !(r && r.kind === 'session' && ids.has(r.session_id)));
  const merged = kept.concat(records);
  ensureDir(path.dirname(file));
  fs.writeFileSync(file, merged.length ? merged.map((r) => JSON.stringify(r)).join('\n') + '\n' : '', 'utf8');
  return file;
}

// syncSessionsToLog — scan every transcript and upsert one session row each into
// performance.jsonl (backfill + refresh). Cache-backed.
export function syncSessionsToLog(projectsRoot, { cachePath, env = process.env, homedir = os.homedir() } = {}) {
  const { rollups, cache_stats } = collectSessionRollups(projectsRoot, { cachePath });
  const records = rollups.map((r) => sessionRecord(r));
  upsertSessionRecords(records, env, homedir);
  return { sessions: records.length, projects: new Set(records.map((r) => r.project_name)).size, cache_stats };
}

// buildMatrixFromLog — the per-project matrix READ from performance.jsonl,
// grouped by project_name (the last folder). No transcript scan happens here.
export function buildMatrixFromLog(env = process.env, homedir = os.homedir(), { since } = {}) {
  const sinceMs = since != null ? toMs(since) : null;
  const byName = new Map();
  for (const r of readSessionRecords(env, homedir)) {
    if (sinceMs != null && (r.ended_ms == null || r.ended_ms < sinceMs)) continue;
    const name = r.project_name || projectName(r.project);
    let agg = byName.get(name);
    if (!agg) {
      agg = newProjectAgg(name, null);
      agg.paths = [];
      byName.set(name, agg);
    }
    if (r.project && !agg.paths.includes(r.project)) agg.paths.push(r.project);
    mergeSessionIntoProject(agg, r);
  }
  const projects = [...byName.values()];
  for (const p of projects) finalizeProjectAgg(p);
  projects.sort((a, b) => b.total_tokens - a.total_tokens);
  return { generated_at: new Date().toISOString(), projects, totals: totalsFor(projects) };
}

// --- HTML matrix rendering ----------------------------------------------

function shortModel(model) {
  return String(model).replace(/^claude-/, '').replace(/-\d{6,}$/, '');
}

function esc(s) {
  return String(s == null ? '' : s).replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
}

function fmtTokens(n) {
  const v = num(n);
  if (v >= 1e9) return `${(v / 1e9).toFixed(2)}B`;
  if (v >= 1e6) return `${(v / 1e6).toFixed(2)}M`;
  if (v >= 1e3) return `${(v / 1e3).toFixed(1)}k`;
  return String(v);
}

function cachePct(total, cached) {
  const t = num(total);
  return t > 0 ? `${Math.round((num(cached) / t) * 100)}%` : '—';
}

function fmtDate(ms) {
  return ms == null ? '—' : new Date(ms).toISOString().slice(0, 16).replace('T', ' ');
}

// renderMatrixHtml — a self-contained (inline CSS/JS, no external requests),
// theme-aware HTML page: a totals strip and a per-project matrix with an
// expandable per-model breakdown per project.
export function renderMatrixHtml(scan) {
  const s = scan || { projects: [], totals: {}, generated_at: new Date().toISOString() };
  const t = s.totals || {};
  const rows = (s.projects || [])
    .map((p, i) => {
      const models = Object.entries(p.models || {})
        .sort((a, b) => b[1].total - a[1].total)
        .map(([m, v]) => `<tr><td class="mdl">${esc(shortModel(m))}</td><td class="num">${fmtTokens(v.total)}</td><td class="num">${fmtTokens(v.new)}</td><td class="num">${fmtTokens(v.cached)}</td></tr>`)
        .join('');
      const modelNames = Object.keys(p.models || {}).map(shortModel).join(', ') || '—';
      return `<tbody class="proj">
  <tr class="row" data-i="${i}">
    <td class="name" title="${esc((p.paths && p.paths.length ? p.paths.join(', ') : p.project))}">${esc(p.project)}</td>
    <td class="num">${p.sessions}</td>
    <td class="num">${esc(humanizeMs(p.running_time_ms))}</td>
    <td class="num strong">${fmtTokens(p.total_tokens)}</td>
    <td class="num">${fmtTokens(p.new_tokens)}</td>
    <td class="num">${fmtTokens(p.cached_tokens)}</td>
    <td class="num">${cachePct(p.total_tokens, p.cached_tokens)}</td>
    <td class="num">${p.parallel_sessions}/${p.sessions}</td>
    <td class="models">${esc(modelNames)}</td>
    <td class="num">${esc(fmtDate(p.last_ms))}</td>
  </tr>
  <tr class="detail"><td colspan="10"><table class="mtx"><thead><tr><th>model</th><th>total</th><th>new</th><th>cached</th></tr></thead><tbody>${models}</tbody></table></td></tr>
</tbody>`;
    })
    .join('\n');
  const summary = [
    ['projects', t.projects || 0],
    ['sessions', t.sessions || 0],
    ['active time', humanizeMs(t.running_time_ms)],
    ['total tokens', fmtTokens(t.total_tokens)],
    ['new', fmtTokens(t.new_tokens)],
    ['cached', fmtTokens(t.cached_tokens)],
    ['cache %', cachePct(t.total_tokens, t.cached_tokens)],
  ]
    .map(([k, v]) => `<div class="card"><div class="k">${esc(k)}</div><div class="v">${esc(v)}</div></div>`)
    .join('');
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>bee performance</title>
<style>
:root{--bg:#f7f8fa;--fg:#1a1d23;--muted:#6b7280;--card:#fff;--line:#e5e7eb;--accent:#b45309;--rowhover:#f0f1f4;}
@media (prefers-color-scheme: dark){:root{--bg:#0f1115;--fg:#e6e8eb;--muted:#9aa1ab;--card:#171a21;--line:#262b34;--accent:#f59e0b;--rowhover:#1c2029;}}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--fg);font:14px/1.5 system-ui,-apple-system,Segoe UI,Roboto,sans-serif;padding:24px;}
h1{font-size:20px;margin:0 0 4px}
.sub{color:var(--muted);font-size:12px;margin-bottom:20px}
.cards{display:flex;flex-wrap:wrap;gap:12px;margin-bottom:24px}
.card{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:12px 16px;min-width:110px}
.card .k{color:var(--muted);font-size:11px;text-transform:uppercase;letter-spacing:.04em}
.card .v{font-size:20px;font-weight:600;margin-top:2px}
.wrap{overflow-x:auto;border:1px solid var(--line);border-radius:10px;background:var(--card)}
table.matrix{border-collapse:collapse;width:100%;min-width:820px}
table.matrix thead th{position:sticky;top:0;background:var(--card);text-align:right;padding:10px 12px;font-size:11px;text-transform:uppercase;letter-spacing:.04em;color:var(--muted);border-bottom:1px solid var(--line);cursor:pointer;white-space:nowrap}
table.matrix thead th:first-child{text-align:left}
.row td{padding:10px 12px;border-bottom:1px solid var(--line);text-align:right;white-space:nowrap}
.row td.name{text-align:left;font-weight:600;max-width:340px;overflow:hidden;text-overflow:ellipsis}
.row td.models{text-align:left;color:var(--muted);font-size:12px;max-width:220px;overflow:hidden;text-overflow:ellipsis}
.row td.strong{font-weight:700;color:var(--accent)}
.num{font-variant-numeric:tabular-nums}
.row:hover{background:var(--rowhover)}
.row{cursor:pointer}
.detail{display:none}
.detail.open{display:table-row}
.detail td{padding:0 12px 12px 24px;border-bottom:1px solid var(--line)}
table.mtx{border-collapse:collapse;margin:6px 0}
table.mtx th,table.mtx td{padding:3px 14px 3px 0;text-align:right;font-size:12px;color:var(--muted)}
table.mtx th:first-child,table.mtx td.mdl{text-align:left;color:var(--fg)}
.empty{padding:40px;text-align:center;color:var(--muted)}
</style>
</head>
<body>
<h1>bee performance</h1>
<div class="sub">${(s.projects || []).length} project(s) · generated ${esc((s.generated_at || '').slice(0, 19).replace('T', ' '))} UTC · active time excludes idle</div>
<div class="cards">${summary}</div>
<div class="wrap">
<table class="matrix">
<thead><tr>
<th data-sort="name">Project</th><th data-sort="num">Sessions</th><th data-sort="num">Active</th>
<th data-sort="num">Total</th><th data-sort="num">New</th><th data-sort="num">Cached</th><th data-sort="num">Cache%</th>
<th data-sort="num">Parallel</th><th data-sort="name">Models</th><th data-sort="num">Last active</th>
</tr></thead>
${rows || '<tbody><tr><td class="empty" colspan="10">No sessions found yet. Do some work, then reopen this page.</td></tr></tbody>'}
</table>
</div>
<script>
// expand a project row to show its per-model breakdown
document.querySelectorAll('tr.row').forEach(function(r){
  r.addEventListener('click',function(){
    var d=r.parentNode.querySelector('tr.detail');
    if(d) d.classList.toggle('open');
  });
});
</script>
</body>
</html>
`;
}

export function writeReport(scan, { env = process.env, homedir = os.homedir(), out } = {}) {
  const file = out || reportHtmlPath(env, homedir);
  ensureDir(path.dirname(file));
  fs.writeFileSync(file, renderMatrixHtml(scan), 'utf8');
  return file;
}
