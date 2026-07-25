#!/usr/bin/env node
// test_verify_timeout.mjs — proves run_verify.mjs's per-suite timeout and
// heartbeat (i54-closeout-2, CONTEXT.md D2): a suite that never finishes must
// never hang the pool forever, and a long run must never look frozen to a
// human or agent watching it.
//
// Auto-discovered (no manual registration, PLAN-CHECK W2): this file matches
// the `scripts/tests/test_*.mjs` discovery glob same as every other suite here.
// `node scripts/run_verify.mjs --only test_verify_timeout` picks it up.
//
// Three things are proven, independently:
//   (1) BEE_VERIFY_SUITE_TIMEOUT_MS / BEE_VERIFY_HEARTBEAT_MS parsing
//       (resolveSuiteTimeoutMs / resolveHeartbeatMs) — unit-level, no spawn.
//   (2) runOne() integration: a suite that outlives a tiny configured
//       timeout is killed — process GROUP, not just the direct child, so a
//       grandchild the suite spawned cannot outlive it either — and reported
//       as a distinct TIMEOUT (never conflated with an ordinary FAIL); a
//       fast suite under the identical tiny timeout still passes normally,
//       proving default behavior for fast suites is unchanged.
//   (3) The heartbeat mechanism (createInFlightTracker/startHeartbeat/
//       stopHeartbeat): emits to stderr (never stdout) naming whichever
//       suite is tracked in flight, on an injectable interval — no
//       unconditional 30s sleep anywhere in this file — and stays silent
//       when nothing is in flight.

import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..", "..");

const {
  resolveSuiteTimeoutMs,
  resolveHeartbeatMs,
  runOne,
  createInFlightTracker,
  startHeartbeat,
  stopHeartbeat,
} = await import(pathToFileURL(path.join(REPO_ROOT, "scripts", "run_verify.mjs")).href);

let passed = 0;
let failed = 0;
async function check(name, fn) {
  try {
    await fn();
    passed += 1;
    console.log(`PASS ${name}`);
  } catch (error) {
    failed += 1;
    console.error(`FAIL ${name}: ${error.stack ?? error.message}`);
  }
}

function isAlive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

async function waitFor(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// ── BEE_VERIFY_SUITE_TIMEOUT_MS parsing ─────────────────────────────────
const ENV_TIMEOUT = "BEE_VERIFY_SUITE_TIMEOUT_MS";
const savedTimeoutEnv = process.env[ENV_TIMEOUT];

await check("resolveSuiteTimeoutMs defaults to 300000ms when the env var is unset", () => {
  delete process.env[ENV_TIMEOUT];
  assert.equal(resolveSuiteTimeoutMs(), 300000);
});

await check('resolveSuiteTimeoutMs treats "0" as disabled (0)', () => {
  process.env[ENV_TIMEOUT] = "0";
  assert.equal(resolveSuiteTimeoutMs(), 0);
});

await check('resolveSuiteTimeoutMs treats "none" (any case) as disabled (0)', () => {
  process.env[ENV_TIMEOUT] = "none";
  assert.equal(resolveSuiteTimeoutMs(), 0);
  process.env[ENV_TIMEOUT] = "NONE";
  assert.equal(resolveSuiteTimeoutMs(), 0);
});

await check("resolveSuiteTimeoutMs respects an explicit override", () => {
  process.env[ENV_TIMEOUT] = "1234";
  assert.equal(resolveSuiteTimeoutMs(), 1234);
});

await check("resolveSuiteTimeoutMs falls back to the default on an unparseable value", () => {
  process.env[ENV_TIMEOUT] = "not-a-number";
  assert.equal(resolveSuiteTimeoutMs(), 300000);
});

if (savedTimeoutEnv === undefined) delete process.env[ENV_TIMEOUT];
else process.env[ENV_TIMEOUT] = savedTimeoutEnv;

// ── BEE_VERIFY_HEARTBEAT_MS parsing ──────────────────────────────────────
const ENV_HEARTBEAT = "BEE_VERIFY_HEARTBEAT_MS";
const savedHeartbeatEnv = process.env[ENV_HEARTBEAT];

await check("resolveHeartbeatMs defaults to 30000ms when the env var is unset", () => {
  delete process.env[ENV_HEARTBEAT];
  assert.equal(resolveHeartbeatMs(), 30000);
});

await check("resolveHeartbeatMs respects an explicit override", () => {
  process.env[ENV_HEARTBEAT] = "5000";
  assert.equal(resolveHeartbeatMs(), 5000);
});

await check("resolveHeartbeatMs falls back to the default on zero/negative/unparseable", () => {
  process.env[ENV_HEARTBEAT] = "0";
  assert.equal(resolveHeartbeatMs(), 30000);
  process.env[ENV_HEARTBEAT] = "-5";
  assert.equal(resolveHeartbeatMs(), 30000);
  process.env[ENV_HEARTBEAT] = "nope";
  assert.equal(resolveHeartbeatMs(), 30000);
});

if (savedHeartbeatEnv === undefined) delete process.env[ENV_HEARTBEAT];
else process.env[ENV_HEARTBEAT] = savedHeartbeatEnv;

// ── runOne() timeout integration ─────────────────────────────────────────
// Fixtures are built at runtime under os.tmpdir() — this repo has no
// scripts/tests/fixtures/ convention (confirmed: every existing spawn-driven test
// writes its synthetic child scripts to a mkdtemp sandbox), so a hung
// fixture here is never at risk of being auto-discovered as its own suite
// (it isn't under any DISCOVERY_ROOTS at all) and needs no special naming.
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "verify-timeout-"));
try {
  const fastScript = path.join(tmp, "fast_suite.mjs");
  fs.writeFileSync(fastScript, "console.log('fast suite ran');\nprocess.exit(0);\n");

  // Sleeps far longer than any timeout used below — proves the kill is what
  // ends it, never a natural exit racing the timer.
  const hangScript = path.join(tmp, "hang_suite.mjs");
  fs.writeFileSync(hangScript, "setTimeout(() => process.exit(0), 10000);\n");

  await check("a suite that outlives its timeout is killed and reported as a distinct TIMEOUT, not an ordinary FAIL", async () => {
    const result = await runOne([hangScript], { timeoutMs: 150 });
    assert.equal(result.timedOut, true, "expected timedOut: true");
    assert.notEqual(result.code, 0, "a timed-out suite must never report exit code 0");
    assert.ok(result.ms < 5000, `expected the kill to end the run well under the 10s hang, took ${result.ms}ms`);
    assert.match(result.stderr, /TIMEOUT after 150ms/, `expected a distinct TIMEOUT note in stderr, got: ${result.stderr}`);
  });

  await check("a fast suite under the identical tiny timeout still passes normally (default behavior for fast suites is unchanged)", async () => {
    const result = await runOne([fastScript], { timeoutMs: 150 });
    assert.equal(result.timedOut, false);
    assert.equal(result.code, 0);
    assert.match(result.stdout, /fast suite ran/);
  });

  await check('a timeoutMs of 0 (the "disabled" sentinel) never arms a timer — a fast suite still passes', async () => {
    const result = await runOne([fastScript], { timeoutMs: 0 });
    assert.equal(result.timedOut, false);
    assert.equal(result.code, 0);
  });

  // Process-GROUP kill, not just the direct child: the hung suite spawns its
  // own grandchild (undetached, so it inherits the suite's own process
  // group) and records the grandchild's pid before it sleeps. If the timeout
  // only killed the direct child, this grandchild would still be alive after
  // runOne() resolves — exactly the orphan-process failure mode D2 exists to
  // close.
  const grandchildPidFile = path.join(tmp, "grandchild.pid");
  const hangWithGrandchildScript = path.join(tmp, "hang_with_grandchild.mjs");
  fs.writeFileSync(
    hangWithGrandchildScript,
    [
      "import { spawn } from 'node:child_process';",
      "import fs from 'node:fs';",
      `const grandchild = spawn(process.execPath, ['-e', 'setTimeout(() => {}, 15000)'], { stdio: 'ignore' });`,
      `fs.writeFileSync(${JSON.stringify(grandchildPidFile)}, String(grandchild.pid));`,
      "setTimeout(() => process.exit(0), 15000);",
      "",
    ].join("\n"),
  );

  await check("timeout kills the WHOLE process group — a grandchild the suite spawned is killed too, never left orphaned", async () => {
    const result = await runOne([hangWithGrandchildScript], { timeoutMs: 400 });
    assert.equal(result.timedOut, true);
    const grandchildPid = Number(fs.readFileSync(grandchildPidFile, "utf8").trim());
    assert.ok(Number.isInteger(grandchildPid) && grandchildPid > 0, "expected the fixture to have recorded a grandchild pid before it was killed");
    // SIGKILL is immediate but the OS needs a brief moment to reap; give it
    // one short grace tick rather than sleeping 30s+ to prove a negative.
    await waitFor(200);
    assert.equal(isAlive(grandchildPid), false, `expected grandchild pid ${grandchildPid} to be dead after the process-group kill, but it is still alive`);
  });
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}

// ── heartbeat mechanism ───────────────────────────────────────────────────
await check("startHeartbeat stays silent when nothing is tracked as in flight", async () => {
  const tracker = createInFlightTracker();
  const lines = [];
  const timer = startHeartbeat(tracker, { intervalMs: 20, wallStart: Date.now(), log: (line) => lines.push(line) });
  await waitFor(90);
  stopHeartbeat(timer);
  assert.equal(lines.length, 0, `expected no HEARTBEAT lines with nothing in flight, got: ${JSON.stringify(lines)}`);
});

await check("startHeartbeat emits a line naming an in-flight suite, on an injectable interval (no 30s sleep needed)", async () => {
  const tracker = createInFlightTracker();
  const lines = [];
  tracker.start("demo/suite.mjs");
  const timer = startHeartbeat(tracker, { intervalMs: 20, wallStart: Date.now(), log: (line) => lines.push(line) });
  await waitFor(90);
  tracker.end("demo/suite.mjs");
  stopHeartbeat(timer);
  assert.ok(lines.length > 0, "expected at least one HEARTBEAT line while a suite was tracked in flight");
  assert.ok(
    lines.every((l) => l.startsWith("HEARTBEAT:") && l.includes("demo/suite.mjs")),
    `expected every line to name the in-flight suite, got: ${JSON.stringify(lines)}`,
  );
});

await check("startHeartbeat's default log target is stderr (console.error), never stdout (console.log) — machine-readable output stays clean", async () => {
  const originalError = console.error;
  const originalLog = console.log;
  const errorLines = [];
  const logLines = [];
  console.error = (line) => errorLines.push(line);
  console.log = (line) => logLines.push(line);
  try {
    const tracker = createInFlightTracker();
    tracker.start("stderr-check/suite.mjs");
    const timer = startHeartbeat(tracker, { intervalMs: 20, wallStart: Date.now() });
    await waitFor(90);
    tracker.end("stderr-check/suite.mjs");
    stopHeartbeat(timer);
  } finally {
    console.error = originalError;
    console.log = originalLog;
  }
  assert.ok(errorLines.some((l) => l.startsWith("HEARTBEAT:")), "expected the default heartbeat log target to print via console.error");
  assert.ok(logLines.length === 0, `expected zero console.log calls from the heartbeat, got: ${JSON.stringify(logLines)}`);
});

await check("startHeartbeat with intervalMs <= 0 never arms a timer", () => {
  const tracker = createInFlightTracker();
  const timer = startHeartbeat(tracker, { intervalMs: 0 });
  assert.equal(timer, null);
});

console.log(`\ntest_verify_timeout: ${passed} passed, ${failed} failed`);
if (failed) process.exit(1);
