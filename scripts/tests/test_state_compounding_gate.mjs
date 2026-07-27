#!/usr/bin/env node
// test_state_compounding_gate.mjs — the dedicated behavioral suite for the
// compounding gate (feature compounding-gate, cell cg-2): `state
// compounding-run` (cell cg-1, .bee/bin/lib/state.mjs + .bee/bin/bee.mjs) and
// the `compounding-complete` tail guard's freshness half
// (checkPhaseTransition, state.mjs). cg-1's own suites
// (packages/bee/tests/test_cli_state.mjs, test_bee_cli.mjs) only picked up
// fixture updates plus one registry-exercise check; this file is the
// dedicated suite proving each of the five behaviors independently.
//
// Style: spawns the REAL top-level `node .bee/bin/bee.mjs` as a child
// process against a disposable temp repo fixture — same convention as
// scripts/tests/test_worktree_cli.mjs — and reports through the shared
// check/assert/printSummaryAndExit runner every other scripts/tests suite
// uses (scripts/lib/test-fixture.mjs). Every assertion drives behavior
// through the CLI verbs (`state compounding-run`, `state set`) — never a
// hand-edited state.json standing in for a verb that exists (must-have
// prohibition on this cell).
//
// Five rows, one per must-have behavior:
//   (a) `state compounding-run` refused outside phase compounding, message
//       names the legal phase(s).
//   (b) from phase compounding, `state compounding-run --feature X
//       --learnings <path>` stamps last_compounding_run with an ISO-precise
//       `at`, and does NOT change phase.
//   (c) `state set --phase compounding-complete` refused when
//       last_compounding_run is missing, and again when its `at` predates
//       last_scribing_run.at (stale) — both refusals name `state
//       compounding-run` as the fix.
//   (d) succeeds after a fresh compounding-run.
//   (e) `--waive-compounding` permits the transition and logs an audit
//       decision naming the feature.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { check, assert, printSummaryAndExit } from '../lib/test-fixture.mjs';
import { writeJsonAtomic, readJson } from '../../.bee/bin/lib/fsutil.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..', '..');
const BEE_MJS = path.join(REPO_ROOT, '.bee', 'bin', 'bee.mjs');

function bee(cwd, args) {
  return spawnSync(process.execPath, [BEE_MJS, ...args], { cwd, encoding: 'utf8' });
}

// Disposable per-case fixture: a bare .bee/onboarding.json marker is enough
// for resolveRoots (state.mjs) to treat the temp dir as its own repo root —
// same minimal shape packages/bee/tests/test_cli_state.mjs's makeStateRepo
// uses, and deliberately never the live .bee/state.json. No real .git, no
// claims/session records either, so resolveMutationTarget's session-bound-
// lane branch always falls through to the default record (readSession finds
// nothing to bind to here) regardless of THIS calling session's own
// CLAUDE_CODE_SESSION_ID / BEE_SESSION_ID / BEE_AGENT_NAME env.
function makeStateRepo(prefix) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  return dir;
}

function readState(dir) {
  return readJson(path.join(dir, '.bee', 'state.json'), null);
}

function seedState(dir, record) {
  writeJsonAtomic(path.join(dir, '.bee', 'state.json'), record);
}

function readDecisions(dir) {
  const file = path.join(dir, '.bee', 'decisions.jsonl');
  if (!fs.existsSync(file)) return [];
  return fs
    .readFileSync(file, 'utf8')
    .split('\n')
    .filter((line) => line.trim())
    .map((line) => JSON.parse(line));
}

const ISO_MS_RE = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/;

// ── (a) refused outside phase compounding ───────────────────────────────────
await check('compounding-run refused outside phase compounding, message names the legal phase', async () => {
  const dir = makeStateRepo('bee-cg-outside-');
  try {
    // A fresh repo defaults to phase "idle" (readStateStrict) — never
    // "compounding" — so this is the outside-the-door case with zero seeding.
    const result = bee(dir, [
      'state',
      'compounding-run',
      '--feature',
      'demo-cg',
      '--learnings',
      'docs/history/demo-cg/learnings.md',
    ]);
    assert(result.status !== 0, `expected a non-zero exit from phase idle, got ${result.status}`);
    assert(
      /refused from phase "idle"/.test(result.stderr),
      `expected the refusal to name the current phase "idle", got stderr=${result.stderr}`,
    );
    assert(
      /Legal from: compounding/.test(result.stderr),
      `expected the refusal to name the legal phase list ("Legal from: compounding"), got stderr=${result.stderr}`,
    );
    assert(readState(dir) === null, 'a refused compounding-run must write nothing — state.json still absent');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ── (b) stamps last_compounding_run with an ISO at, phase unchanged ─────────
await check(
  'from phase compounding, compounding-run stamps last_compounding_run with an ISO-precise at, and does not change phase',
  async () => {
    const dir = makeStateRepo('bee-cg-stamp-');
    try {
      seedState(dir, {
        phase: 'compounding',
        feature: 'demo-cg',
        last_scribing_run: { feature: 'demo-cg', at: new Date(Date.now() - 120000).toISOString() },
      });
      const before = Date.now();
      const result = bee(dir, [
        'state',
        'compounding-run',
        '--feature',
        'demo-cg',
        '--learnings',
        'docs/history/demo-cg/learnings.md',
      ]);
      const after = Date.now();
      assert(
        result.status === 0,
        `compounding-run from phase compounding should succeed, got ${result.status}: ${result.stderr}`,
      );
      const state = readState(dir);
      assert(state.phase === 'compounding', `compounding-run must not advance phase, got "${state.phase}"`);
      const run = state.last_compounding_run;
      assert(
        run && run.feature === 'demo-cg',
        `last_compounding_run.feature should be "demo-cg", got ${JSON.stringify(run)}`,
      );
      assert(
        Boolean(run && ISO_MS_RE.test(run.at)),
        `last_compounding_run.at should be an ISO-precise (millisecond) timestamp, got ${run && run.at}`,
      );
      const atMs = Date.parse(run.at);
      assert(
        atMs >= before && atMs <= after,
        `last_compounding_run.at should fall inside the call window [${before}, ${after}], got ${run.at} (${atMs})`,
      );
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  },
);

// ── (c) compounding-complete refused: missing, then stale ───────────────────
await check(
  'compounding-complete refused from compounding when last_compounding_run is missing — FIX names state compounding-run',
  async () => {
    const dir = makeStateRepo('bee-cg-missing-');
    try {
      seedState(dir, {
        phase: 'compounding',
        feature: 'demo-cg',
        last_scribing_run: { feature: 'demo-cg', at: new Date().toISOString() },
        // no last_compounding_run field at all.
      });
      const result = bee(dir, ['state', 'set', '--owner', 'compounding', '--phase', 'compounding-complete']);
      assert(result.status !== 0, `expected a refusal with no last_compounding_run at all, got ${result.status}`);
      assert(
        /state compounding-run/.test(result.stderr),
        `expected the refusal to name \`state compounding-run\` as the fix, got stderr=${result.stderr}`,
      );
      const state = readState(dir);
      assert(state.phase === 'compounding', 'a refused close must leave phase untouched');
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  },
);

await check(
  'compounding-complete refused from compounding when last_compounding_run.at predates last_scribing_run.at (stale) — FIX names state compounding-run',
  async () => {
    const dir = makeStateRepo('bee-cg-stale-');
    try {
      const scribingAt = new Date(Date.now() - 10000).toISOString();
      const staleRunAt = new Date(Date.now() - 60000).toISOString(); // older than scribingAt — stale
      seedState(dir, {
        phase: 'compounding',
        feature: 'demo-cg',
        last_scribing_run: { feature: 'demo-cg', at: scribingAt },
        last_compounding_run: { feature: 'demo-cg', at: staleRunAt, learnings: 'x' },
      });
      const result = bee(dir, ['state', 'set', '--owner', 'compounding', '--phase', 'compounding-complete']);
      assert(result.status !== 0, `expected a refusal with a stale last_compounding_run, got ${result.status}`);
      assert(
        /state compounding-run/.test(result.stderr),
        `expected the refusal to name \`state compounding-run\` as the fix, got stderr=${result.stderr}`,
      );
      const state = readState(dir);
      assert(state.phase === 'compounding', 'a refused close must leave phase untouched');
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  },
);

// ── (d) succeeds after a fresh compounding-run ───────────────────────────────
await check('compounding-complete succeeds after a fresh compounding-run', async () => {
  const dir = makeStateRepo('bee-cg-fresh-');
  try {
    seedState(dir, {
      phase: 'compounding',
      feature: 'demo-cg',
      last_scribing_run: { feature: 'demo-cg', at: new Date(Date.now() - 30000).toISOString() },
    });
    const run = bee(dir, [
      'state',
      'compounding-run',
      '--feature',
      'demo-cg',
      '--learnings',
      'docs/history/demo-cg/learnings.md',
    ]);
    assert(run.status === 0, `compounding-run should succeed, got ${run.status}: ${run.stderr}`);
    const close = bee(dir, ['state', 'set', '--owner', 'compounding', '--phase', 'compounding-complete']);
    assert(
      close.status === 0,
      `compounding-complete should succeed after a fresh compounding-run, got ${close.status}: ${close.stderr}`,
    );
    const state = readState(dir);
    assert(state.phase === 'compounding-complete', `phase should be compounding-complete, got ${state.phase}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ── (e) --waive-compounding permits the transition and logs a decision ──────
await check(
  '--waive-compounding permits compounding-complete over a missing compounding-run and logs an audit decision naming the feature',
  async () => {
    const dir = makeStateRepo('bee-cg-waive-');
    try {
      seedState(dir, {
        phase: 'compounding',
        feature: 'demo-cg',
        last_scribing_run: { feature: 'demo-cg', at: new Date().toISOString() },
        // no last_compounding_run at all — the missing case, waived this time.
      });
      const before = readDecisions(dir).length;
      const result = bee(dir, [
        'state',
        'set',
        '--owner',
        'compounding',
        '--phase',
        'compounding-complete',
        '--waive-compounding',
      ]);
      assert(result.status === 0, `--waive-compounding should permit the close, got ${result.status}: ${result.stderr}`);
      const state = readState(dir);
      assert(state.phase === 'compounding-complete', `phase should be compounding-complete, got ${state.phase}`);

      const decisions = readDecisions(dir);
      assert(
        decisions.length === before + 1,
        `expected exactly one new decision event logged, got ${decisions.length - before}`,
      );
      const waiver = decisions[decisions.length - 1];
      assert(
        /compounding-run freshness check WAIVED/.test(waiver.decision),
        `expected the logged decision to name the compounding-run freshness waiver, got ${JSON.stringify(waiver)}`,
      );
      assert(
        waiver.decision.includes('demo-cg'),
        `expected the logged decision to name the feature "demo-cg", got ${waiver.decision}`,
      );
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  },
);

printSummaryAndExit();
