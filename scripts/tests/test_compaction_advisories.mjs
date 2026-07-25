#!/usr/bin/env node
// test_compaction_advisories.mjs — the suite for the TWO ADVISORY SURFACES the
// compaction feature adds to running hooks (feature compaction-hardening, cell
// cz-7; decisions D4/D5/D9/D10/D11/D23).
//
// cz-3 shipped the module (appendCompactionRecord / survivalWarning /
// anchorMissing) and scripts/tests/test_compaction_module.mjs proves it in isolation.
// NOTHING there can prove that a hook actually calls it, and a module nobody
// calls advises nobody. So every row below drives the REAL hooks —
// hooks/bee-session-close.mjs on PreCompact and hooks/bee-prompt-context.mjs on
// UserPromptSubmit — against a vendored temp repo, and reads their stdout.
//
// Five obligations, each unprovable from any other suite in this feature:
//
//   1. PRECOMPACT NEVER CARRIES A TURN-CONTROL VERDICT (B2/R14, D9). PreCompact
//      output is emitted through emitHookOutput as a systemMessage and never
//      encodeBlock. This is asserted with a POSITIVE CONTROL: the same fixture
//      that makes the gate-bypass net fire on Stop is fed to PreCompact, so the
//      row proves the event discriminates rather than proving the fixture was
//      simply incapable of blocking.
//   2. THE SURVIVAL WARNING FIRES ON A UNIT'S SECOND COMPACTION AND NOT ITS
//      FIRST (D9). The threshold is evaluated against the record for the
//      compaction NOW beginning, so a hook that logged before counting — or
//      counted `resume` records too — is off by exactly one and shows up here.
//   3. THE PRECOMPACT NUDGE IS FORCED PAST A HOT DEDUP CACHE (D11), and does
//      NOT mark it. `anchorMissing` performs no dedup of its own — it never
//      reads shouldInject and never calls markInjected — so BOTH halves are the
//      caller's, and both are asserted: the nudge survives a freshly-marked
//      cache, and a PreCompact leaves the cache cold so the next
//      UserPromptSubmit still nudges. (A forced surface that marked the cache
//      would silently swallow the deduped one — the mirror of
//      maybeCaptureQueueNudge, where `force` skips both.)
//   4. THE USERPROMPTSUBMIT NUDGE DOES DEDUP (D11) — the same predicate on the
//      other surface is throttled, or it prints on every single turn.
//   5. THE PREDICATE IS HONORED, NOT APPROXIMATED (D10): no nudge when an
//      anchor exists, and none when the phase is terminal — asserted on BOTH
//      surfaces, because each hook owns its own call site.
//
// EVERY ROW IS WRITTEN TO FAIL AGAINST THE UN-WIRED HOOKS. That is not a style
// note: cz-6 measured that a row asserting only that a line is PRESENT can pass
// against an un-wired hook when another code path emits the same line, so each
// row here either asserts a string only the new wiring can produce, or pins the
// exact bytes the module returns.
//
// D17: the suite imports the SOURCE modules (skills/bee-hive/templates/lib/)
// and vendors .bee/bin/lib into each fixture, exactly as
// scripts/tests/test_compact_capsule.mjs does; scripts/tests/test_lib_mirror.mjs is what
// proves the two trees match.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { check, assert, printSummaryAndExit } from '../lib/test-fixture.mjs';
import { runModuleWorker } from '../lib/run-module-worker.mjs';
import {
  anchorMissing,
  survivalWarning,
  readCompactionRecords,
  ANCHOR_NUDGE_COMMAND,
} from '../../skills/bee-hive/templates/lib/compaction.mjs';
import { shouldInject, markInjected } from '../../skills/bee-hive/templates/lib/inject.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..', '..');
const REAL_LIB_DIR = path.join(REPO_ROOT, '.bee', 'bin', 'lib');
const SESSION_CLOSE_HOOK = path.join(REPO_ROOT, 'hooks', 'bee-session-close.mjs');
const PROMPT_CONTEXT_HOOK = path.join(REPO_ROOT, 'hooks', 'bee-prompt-context.mjs');

// The exact lead of the nudge string compaction.mjs builds. Used only to prove
// ABSENCE (a message that must not carry a nudge); every presence assertion
// pins the module's full bytes instead.
const NUDGE_LEAD = 'bee: work is active and NO INTENT ANCHOR is stored';

const pending = [];
const acheck = (name, fn) => {
  pending.push(check(name, fn));
};

// ─── fixtures ───────────────────────────────────────────────────────────────

const tempRoots = [];

function writeJson(file, obj) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, `${JSON.stringify(obj, null, 2)}\n`, 'utf8');
}

/** Vendor the real .bee/bin/lib into a fixture — the hooks load lib from root. */
function vendorLib(root) {
  const libDir = path.join(root, '.bee', 'bin', 'lib');
  fs.mkdirSync(libDir, { recursive: true });
  for (const name of fs.readdirSync(REAL_LIB_DIR)) {
    if (name.endsWith('.mjs')) fs.copyFileSync(path.join(REAL_LIB_DIR, name), path.join(libDir, name));
  }
  return root;
}

function makeRepo({
  phase = 'swarming',
  mode = 'standard',
  feature = 'demo',
  execution = true,
  shape = true,
  context = true,
  nextAction = '',
  config = null,
} = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-advisory-'));
  tempRoots.push(root);
  fs.mkdirSync(path.join(root, '.bee', 'logs'), { recursive: true });
  writeJson(path.join(root, '.bee', 'onboarding.json'), { schema_version: '1.0' });
  writeJson(path.join(root, '.bee', 'state.json'), {
    schema_version: '1.0',
    phase,
    mode,
    feature,
    approved_gates: { context, shape, execution, review: false },
    workers: [],
    summary: '',
    next_action: nextAction,
  });
  if (config) writeJson(path.join(root, '.bee', 'config.json'), config);
  return vendorLib(root);
}

function addCell(root, { id, feature = 'demo', status = 'claimed', session = null, deps = [] }) {
  writeJson(path.join(root, '.bee', 'cells', `${id}.json`), {
    id,
    feature,
    title: `Cell ${id}`,
    lane: 'small',
    status,
    deps,
    action: 'Do the thing.',
    verify: 'node -e "process.exit(0)"',
    trace: { worker: 'tester', claim_session: session, claimed_at: '2026-01-01T00:00:00.000Z' },
  });
}

function addAnchor(root, key = 'demo') {
  writeJson(path.join(root, '.bee', 'intent', `${key}.json`), {
    schema_version: '1.0',
    key,
    written_at: '2026-01-01T00:00:00.000Z',
    request: 'do the thing the user actually asked for',
    acceptance: 'the thing is done',
    next_action: null,
    feature: null,
    lane: null,
    cell: null,
    do_not_reverse: [],
    stop_conditions: [],
  });
}

// ─── running the two hooks ──────────────────────────────────────────────────

/**
 * A fake HOME per run isolates bee-session-close's perf-refresh side write, so
 * no test run touches the real ~/.claude tree (same discipline as
 * hooks/test_bypass_stop_net.mjs).
 */
function fakeHome() {
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-advisory-home-'));
  tempRoots.push(home);
  return home;
}

/** PreCompact (or Stop) through the real session-close hook. */
async function runSessionClose(root, { sessionId, event = 'PreCompact' }) {
  const result = await runModuleWorker(SESSION_CLOSE_HOOK, {
    input: JSON.stringify({ hook_event_name: event, session_id: sessionId, cwd: root }),
    cwd: root,
    fakeHome: fakeHome(),
    timeout: 30_000,
  });
  assert(
    result.status === 0,
    `${event} is fail-open and must exit 0 — got status=${result.status}\nstderr: ${result.stderr}`,
  );
  const stdout = result.stdout || '';
  let json = null;
  if (stdout.trim()) {
    try {
      json = JSON.parse(stdout.trim());
    } catch {
      json = null;
    }
  }
  const message = json && typeof json.systemMessage === 'string' ? json.systemMessage : '';
  return { stdout, json, message };
}

/** UserPromptSubmit through the real prompt-context hook (plain stdout). */
async function runPromptContext(root, { sessionId }) {
  const result = await runModuleWorker(PROMPT_CONTEXT_HOOK, {
    input: JSON.stringify({
      hook_event_name: 'UserPromptSubmit',
      prompt: 'carry on',
      session_id: sessionId,
      cwd: root,
    }),
    cwd: root,
    fakeHome: fakeHome(),
    timeout: 30_000,
  });
  assert(
    result.status === 0,
    `UserPromptSubmit is fail-open and must exit 0 — got status=${result.status}\nstderr: ${result.stderr}`,
  );
  return result.stdout || '';
}

/** The nudge the module would produce for this fixture — asserted non-null. */
function expectedNudge(root, sessionId) {
  const nudge = anchorMissing(root, { sessionId });
  assert(
    nudge,
    'the fixture must satisfy D10 (non-terminal phase + active work + no anchor) — otherwise the row proves nothing',
  );
  return nudge;
}

// ─── 1. PreCompact never carries a turn-control verdict (B2/R14, D9) ────────

acheck('PreCompact carries NO turn-control verdict under any input (B2/R14) — with the Stop positive control', async () => {
  const sid = 'sess-verdict';

  // (a) anchor present, (b) anchor absent → the nudge fires, (c) a unit on its
  // second compaction → the warning fires, (d) the fixture that makes the
  // gate-bypass net BLOCK on Stop.
  const withAnchor = makeRepo();
  addAnchor(withAnchor, 'demo');
  const noAnchor = makeRepo();
  const second = makeRepo();
  addCell(second, { id: 'k-1', session: sid });
  const bypass = makeRepo({
    phase: 'planning',
    shape: false,
    execution: false,
    config: { schema_version: '1.0', gate_bypass: 'total' },
  });

  for (const [label, root] of [
    ['anchor present', withAnchor],
    ['anchor absent (nudge fires)', noAnchor],
    ['second compaction (warning fires)', second],
    ['gate-bypass total mid-planning', bypass],
  ]) {
    // `second` needs a first compaction to be on its second one.
    if (root === second) await runSessionClose(root, { sessionId: sid });
    const r = await runSessionClose(root, { sessionId: sid });
    assert(
      !r.stdout.includes('"decision"'),
      `${label}: PreCompact stdout must never carry a verdict field at all — got:\n${r.stdout}`,
    );
    assert(
      r.stdout.trim() === '' || (r.json && typeof r.json.systemMessage === 'string'),
      `${label}: any non-empty PreCompact stdout is a parseable JSON systemMessage — got:\n${r.stdout}`,
    );
    assert(
      !r.json || r.json.decision !== 'block',
      `${label}: PreCompact must never emit decision:"block" (B2/R14) — got:\n${r.stdout}`,
    );
  }

  // POSITIVE CONTROL: the last fixture is genuinely capable of blocking — on
  // Stop it does. Without this, the row above could pass on a fixture that
  // could never have blocked in the first place.
  const stop = await runSessionClose(bypass, { sessionId: sid, event: 'Stop' });
  assert(
    stop.stdout.includes('"decision":"block"'),
    `the bypass fixture must block on Stop — otherwise the PreCompact row proves nothing. Got:\n${stop.stdout}`,
  );
});

// ─── 2. the survival warning: second compaction, not the first (D9) ─────────

acheck('the survival warning fires on a unit\'s SECOND compaction and not its first (D9)', async () => {
  const root = makeRepo();
  const sid = 'sess-survival';
  addCell(root, { id: 'k-1', session: sid });

  const first = await runSessionClose(root, { sessionId: sid });
  assert(
    !first.message.includes('has now survived'),
    `a unit's FIRST compaction carries no survival warning (D9) — got:\n${first.message}`,
  );
  const afterFirst = readCompactionRecords(root);
  assert(
    afterFirst.length === 1 && afterFirst[0].event === 'precompact',
    `PreCompact appends exactly one \`precompact\` record (D4/D5) — got ${JSON.stringify(afterFirst)}`,
  );
  assert(
    afterFirst[0].cell === 'k-1' && afterFirst[0].cell_compact_count === 1,
    `the record names the claimed unit and counts itself (D5) — got ${JSON.stringify(afterFirst[0])}`,
  );

  const secondRun = await runSessionClose(root, { sessionId: sid });
  assert(
    secondRun.message.includes(survivalWarning(2)),
    `the SECOND compaction of the same unit renders the module's warning VERBATIM (D9) — expected ` +
      `"${survivalWarning(2)}" in:\n${secondRun.message}`,
  );
  assert(
    secondRun.message.includes('k-1'),
    `the warning names the unit it is about — got:\n${secondRun.message}`,
  );
  const afterSecond = readCompactionRecords(root);
  assert(
    afterSecond.length === 2 && afterSecond[1].cell_compact_count === 2,
    `the second record counts inclusively (D5) — got ${JSON.stringify(afterSecond)}`,
  );
});

// ─── 3. the PreCompact nudge is FORCED, and leaves the cache cold (D11) ─────

acheck('the PreCompact nudge survives a freshly-marked dedup cache (D11 force)', async () => {
  const root = makeRepo();
  const sid = 'sess-forced';
  const nudge = expectedNudge(root, sid);

  // Make the cache as hot as it can be: this exact key+hash, marked right now.
  markInjected(root, nudge.key, nudge.hash);
  assert(
    shouldInject(root, nudge.key, nudge.hash) === false,
    'the fixture must have a genuinely hot cache — otherwise "forced" is indistinguishable from "deduped"',
  );

  const r = await runSessionClose(root, { sessionId: sid });
  assert(
    r.message.includes(nudge.message),
    `PreCompact is the last moment the objective can be captured verbatim: the nudge is forced past the ` +
      `30-minute throttle (D11) — expected the module's nudge in:\n${r.message}`,
  );
  assert(
    r.message.includes(ANCHOR_NUDGE_COMMAND),
    `the nudge names the exact command to run (D10) — got:\n${r.message}`,
  );
});

acheck('PreCompact does NOT mark the dedup cache — the next UserPromptSubmit still nudges (D11)', async () => {
  const root = makeRepo();
  const sid = 'sess-cold';
  const nudge = expectedNudge(root, sid);
  assert(shouldInject(root, nudge.key, nudge.hash) === true, 'the fixture starts with a cold cache');

  const pre = await runSessionClose(root, { sessionId: sid });
  assert(pre.message.includes(nudge.message), `PreCompact nudges on a cold cache too — got:\n${pre.message}`);
  assert(
    shouldInject(root, nudge.key, nudge.hash) === true,
    'the FORCED surface must skip markInjected as well as shouldInject (maybeCaptureQueueNudge\'s shape) — ' +
      'marking it here would let a compaction swallow the deduped surface\'s next nudge',
  );

  const out = await runPromptContext(root, { sessionId: sid });
  assert(
    out.includes(nudge.message),
    `the UserPromptSubmit nudge must still be available after a PreCompact — got:\n${out}`,
  );
});

// ─── 4. the UserPromptSubmit nudge DOES dedup (D11) ─────────────────────────

acheck('the UserPromptSubmit nudge is deduped within the interval (D11)', async () => {
  const root = makeRepo();
  const sid = 'sess-dedup';
  const nudge = expectedNudge(root, sid);

  const first = await runPromptContext(root, { sessionId: sid });
  assert(first.includes(nudge.message), `the first prompt of an anchorless session nudges — got:\n${first}`);
  assert(
    shouldInject(root, nudge.key, nudge.hash) === false,
    'the deduped surface MUST mark the cache — otherwise it nudges on every single turn',
  );

  const secondOut = await runPromptContext(root, { sessionId: sid });
  assert(
    !secondOut.includes(NUDGE_LEAD),
    `a second prompt inside the 30-minute interval carries no nudge (D11) — got:\n${secondOut}`,
  );
});

// ─── 5. the predicate is honored on BOTH surfaces (D10) ─────────────────────

acheck('no nudge on either surface when an anchor exists (D10)', async () => {
  const root = makeRepo();
  const sid = 'sess-anchored';
  addAnchor(root, 'demo');
  assert(anchorMissing(root, { sessionId: sid }) === null, 'the fixture holds an anchor, so D10 must not fire');

  const pre = await runSessionClose(root, { sessionId: sid });
  assert(
    pre.message.includes('=== BEE INTENT ANCHOR'),
    `the anchored fixture still re-asserts the anchor on PreCompact (ia-1 D3, untouched) — got:\n${pre.message}`,
  );
  assert(!pre.message.includes(NUDGE_LEAD), `an anchored session is never nudged — got:\n${pre.message}`);

  const out = await runPromptContext(root, { sessionId: sid });
  assert(!out.includes(NUDGE_LEAD), `an anchored session is never nudged on prompt either — got:\n${out}`);
});

acheck('no nudge on either surface when the phase is terminal (D10)', async () => {
  for (const phase of ['idle', 'compounding-complete']) {
    // Gates stay approved after a feature closes — that is exactly why D10's
    // predicate is the PHASE and not the gate (AGENTS.md's intake-gate rule).
    const root = makeRepo({ phase, execution: true });
    const sid = `sess-terminal-${phase}`;
    assert(anchorMissing(root, { sessionId: sid }) === null, `phase "${phase}" is terminal — D10 must not fire`);

    const pre = await runSessionClose(root, { sessionId: sid });
    assert(!pre.message.includes(NUDGE_LEAD), `phase "${phase}": no PreCompact nudge — got:\n${pre.message}`);

    const out = await runPromptContext(root, { sessionId: sid });
    assert(!out.includes(NUDGE_LEAD), `phase "${phase}": no prompt nudge — got:\n${out}`);
  }
});

// ─── cleanup ────────────────────────────────────────────────────────────────

process.on('exit', () => {
  for (const root of tempRoots) {
    try {
      fs.rmSync(root, { recursive: true, force: true });
    } catch {
      /* best-effort: a leftover temp dir is harmless */
    }
  }
});

await Promise.all(pending);

printSummaryAndExit();
