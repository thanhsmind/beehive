#!/usr/bin/env node
// test_heartbeat_touch.mjs — msh-5 (CONTEXT.md D5, Δ3/Δ4/Δ6-amended; D6):
// proves the hook-driven throttled heartbeat + lease renewal touch
// (claims.mjs heartbeatTouch/renewClaimTTL, reservations.mjs
// renewHoldsBySession) and the D6 state-verb locking, against BOTH direct
// lib calls (deterministic, synthetic clocks) and the ACTUAL wrapper hooks
// (hooks/bee-prompt-context.mjs, hooks/bee-state-sync.mjs) run through the
// shared isolated worker runner (scripts/lib/run-module-worker.mjs — the
// same harness hooks/test_hook_contracts.mjs uses), so the hook-site
// composition of the two leaf modules is exercised for real, not assumed.
//
// Header note — Δ6 documented non-goal (CONTEXT.md D5): a session idling in
// unrelated chat still blanket-renews its own claims/holds on every hook
// event, regardless of whether it is doing anything bee-relevant right now.
// This is an ACCEPTED residual, not a bug under test here — D4's audited
// force-ownership door and release-on-terminal-transition are the rescue,
// not a narrower renewal rule. Nothing below asserts idle-session renewal
// is refused.
//
// Five cases (msh-5 action):
//   1. throttle no-op — heartbeat younger than
//      HEARTBEAT_TOUCH_THROTTLE_SECONDS -> byte-identical session + claim
//      files (direct heartbeatTouch call, synthetic clock).
//   2. over-throttle refresh — heartbeat older than the throttle -> session
//      heartbeat refreshed, the session's owned claim TTL renewed, and its
//      reservation hold renewed, driven through the REAL
//      bee-prompt-context.mjs hook (proves the hook-site composition of
//      claims.mjs + reservations.mjs, not just the lib functions in
//      isolation).
//   3. touch-throw stays green — a sabotaged claims.mjs heartbeatTouch that
//      throws synchronously still leaves both hooks' exit code 0 and their
//      primary job intact (fail-open), with the throw visible in
//      .bee/logs/hooks.jsonl.
//   4. LOCK_BUSY -> silent skip — a pre-held sessions.lock (heartbeatTouch's
//      own session-heartbeat write) and a pre-held state.lock
//      (bee-state-sync's own read-modify-write) both skip their write
//      without throwing, blocking, or logging a crash.
//   5. renewal-vs-.adopting-gate — a claim gated by an in-flight
//      adopt/sweep is SKIPPED by renewClaimTTL and never rewritten, while a
//      sibling ungated claim for the same session still renews, and a
//      different session's claim is never touched at all.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { runModuleWorker } from "./lib/run-module-worker.mjs";

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(SCRIPT_DIR, "..");
const LIB_DIR = path.join(REPO_ROOT, ".bee", "bin", "lib");
const HOOKS_DIR = path.join(REPO_ROOT, "hooks");
const PROMPT_CONTEXT_HOOK = path.join(HOOKS_DIR, "bee-prompt-context.mjs");
const STATE_SYNC_HOOK = path.join(HOOKS_DIR, "bee-state-sync.mjs");
const SPAWN_TIMEOUT_MS = 20000;

function libUrl(name) {
  return pathToFileURL(path.join(LIB_DIR, name)).href;
}

const claimsLib = await import(libUrl("claims.mjs"));
const reservationsLib = await import(libUrl("reservations.mjs"));
const leaseStoreLib = await import(libUrl("lease-store.mjs"));
const lockLib = await import(libUrl("lock.mjs"));
const fsutilLib = await import(libUrl("fsutil.mjs"));

const {
  createSession,
  readSession,
  sessionPath,
  claimCellFile,
  readClaim,
  claimPath,
  claimGatePath,
  renewClaimTTL,
  heartbeatTouch,
  HEARTBEAT_TOUCH_THROTTLE_SECONDS,
} = claimsLib;
const { reserve, listReservations } = reservationsLib;
const { withStoreLock, LockBusyError, lockFilePath, locksDir } = lockLib;
const { ensureDir } = fsutilLib;

// --- sandbox helpers --------------------------------------------------------

function mkSandbox(prefix) {
  return fs.mkdtempSync(path.join(os.tmpdir(), prefix));
}

// A full hook-runnable fixture: onboarding marker (no .git, so it resolves
// "ordinary" per adapter.mjs resolveRoots), a copied .bee/bin/lib (so the
// hook's own dynamic `import(libModuleUrl(...))` calls resolve inside the
// fixture, matching hooks/test_hook_contracts.mjs's buildFixture/copyLib
// pattern), an empty cells dir, and a minimal state.json.
function mkHookFixture(prefix) {
  const root = mkSandbox(prefix);
  fs.mkdirSync(path.join(root, ".bee"), { recursive: true });
  fs.writeFileSync(path.join(root, ".bee", "onboarding.json"), "{}\n");
  const libDir = path.join(root, ".bee", "bin", "lib");
  fs.mkdirSync(libDir, { recursive: true });
  for (const name of fs.readdirSync(LIB_DIR)) {
    if (!name.endsWith(".mjs")) continue;
    fs.copyFileSync(path.join(LIB_DIR, name), path.join(libDir, name));
  }
  fs.mkdirSync(path.join(root, ".bee", "cells"), { recursive: true });
  fs.writeFileSync(
    path.join(root, ".bee", "state.json"),
    `${JSON.stringify(
      {
        phase: "swarming",
        mode: "standard",
        feature: "msh-5-fixture",
        approved_gates: { context: true, shape: true, execution: true, review: false },
      },
      null,
      2,
    )}\n`,
  );
  return root;
}

// Rewrites the fixture's OWN copy of claims.mjs so heartbeatTouch throws
// synchronously as its very first statement — every other export (including
// state.mjs's own top-level imports of claims.mjs) stays intact, so only the
// touch path is sabotaged.
function sabotageHeartbeatTouch(fixtureRoot) {
  const target = path.join(fixtureRoot, ".bee", "bin", "lib", "claims.mjs");
  const original = fs.readFileSync(target, "utf8");
  const marker = "export async function heartbeatTouch(root, sessionId, { now = Date.now() } = {}) {\n";
  if (!original.includes(marker)) {
    throw new Error("sabotageHeartbeatTouch: heartbeatTouch signature not found — source shape changed, fix the marker");
  }
  const sabotaged = original.replace(marker, `${marker}  throw new Error('sabotage-heartbeat-touch');\n`);
  if (sabotaged === original) {
    throw new Error("sabotageHeartbeatTouch: replace produced no change");
  }
  fs.writeFileSync(target, sabotaged, "utf8");
}

function readRaw(file) {
  return fs.readFileSync(file, "utf8");
}

function readHooksLog(root) {
  const file = path.join(root, ".bee", "logs", "hooks.jsonl");
  if (!fs.existsSync(file)) return [];
  return fs
    .readFileSync(file, "utf8")
    .split(/\r?\n/)
    .filter((line) => line.trim())
    .map((line) => JSON.parse(line));
}

async function runHook(hookPath, root, payload) {
  return runModuleWorker(hookPath, {
    input: JSON.stringify(payload),
    cwd: root,
    timeout: SPAWN_TIMEOUT_MS,
  });
}

// --- test rows ---------------------------------------------------------------

const results = [];
function record(name, pass, note) {
  results.push({ name, pass, note });
}

// 1. throttle no-op: heartbeat younger than the throttle -> byte-identical
// session + claim files, touched:false/reason:"throttled".
{
  const root = mkSandbox("bee-heartbeat-throttle-");
  const NOW = Date.parse("2026-01-01T00:00:00.000Z");
  const sessionId = "sess-throttle";
  createSession(root, { id: sessionId, now: NOW });
  claimCellFile(root, sessionId, "cell-a", 3600, { now: NOW });

  const sessionBefore = readRaw(sessionPath(root, sessionId));
  const claimBefore = readRaw(claimPath(root, "cell-a"));

  const touch = await heartbeatTouch(root, sessionId, { now: NOW + 30_000 }); // 30s < 60s throttle

  const sessionAfter = readRaw(sessionPath(root, sessionId));
  const claimAfter = readRaw(claimPath(root, "cell-a"));

  record(
    "throttle-noop:touched-false",
    touch.touched === false && touch.reason === "throttled",
    `touch=${JSON.stringify(touch)}`,
  );
  record(
    "throttle-noop:session-byte-identical",
    sessionAfter === sessionBefore,
    sessionAfter === sessionBefore ? "unchanged" : "session file was rewritten despite throttle",
  );
  record(
    "throttle-noop:claim-byte-identical",
    claimAfter === claimBefore,
    claimAfter === claimBefore ? "unchanged" : "claim file was rewritten despite throttle",
  );

  fs.rmSync(root, { recursive: true, force: true });
}

// 2. over-throttle refresh, driven through the REAL bee-prompt-context.mjs
// hook: session heartbeat refreshes, owned claim TTL renews, and the
// session's reservation hold renews — proving the hook composes
// claims.mjs's heartbeatTouch with reservations.mjs's renewHoldsBySession,
// not just that the lib functions work standalone.
{
  const root = mkHookFixture("bee-heartbeat-refresh-");
  const sessionId = "sess-refresh";
  const backdated = Date.now() - 70_000; // real 70s ago: over the 60s touch throttle
  createSession(root, { id: sessionId, now: backdated });
  claimCellFile(root, sessionId, "cell-x", 3600, { now: backdated });
  const reserved = await reserve(root, {
    agent: "worker-1",
    cell: "cell-x",
    path: "src/refresh-target.mjs",
    session: sessionId,
  });
  record("refresh:reserve-precondition-ok", Boolean(reserved && reserved.ok === true), `reserve() must succeed, got ${JSON.stringify(reserved)}`);

  const sessionBeforeMs = Date.parse(readSession(root, sessionId).last_heartbeat);
  const claimBeforeMs = Date.parse(readClaim(root, "cell-x").claimed_at);
  // multisession-native-16: renewal now extends the underlying lease's
  // expires_at directly (lease-store.mjs's renewLease) without moving
  // acquired_at — DIFFERENT from the pre-msn-16 renewHoldsBySession, which
  // advanced reserved_at (millisecond precision) and left ttl_seconds fixed.
  // The TRANSLATED reservation shape's ttl_seconds is only integer-second
  // precision (leaseToReservation, reservations.mjs), so a renewal that
  // lands within the same second as the original acquire (routine in a fast
  // test/heartbeat cycle) can round-trip to an IDENTICAL derived ttl_seconds
  // even though the real underlying expires_at moved — isLeaseRecordExpired
  // (the actually-correctness-critical function) is unaffected since it
  // reads expires_at directly, but this proof needs that same raw,
  // millisecond-precision field to avoid a false negative from the rounding.
  const rawLeaseExpiryMs = (r, resourcePath) => {
    const record = leaseStoreLib.listLeases(r).leases.find((l) => l.resource === `path:${resourcePath}`);
    return record ? Date.parse(record.expires_at) : null;
  };
  const holdBeforeExpiryMs = rawLeaseExpiryMs(root, "src/refresh-target.mjs");

  const result = await runHook(PROMPT_CONTEXT_HOOK, root, {
    hook_event_name: "UserPromptSubmit",
    session_id: sessionId,
    cwd: root,
  });

  record("refresh:hook-exit-0", result.status === 0, `status=${result.status} stderr=${result.stderr}`);

  const sessionAfter = readSession(root, sessionId);
  const claimAfter = readClaim(root, "cell-x");
  const holdAfter = listReservations(root, { activeOnly: true }).find((r) => r.cell === "cell-x" && r.agent === "worker-1");
  const holdAfterExpiryMs = rawLeaseExpiryMs(root, "src/refresh-target.mjs");

  record(
    "refresh:session-heartbeat-advanced",
    Boolean(sessionAfter) && Date.parse(sessionAfter.last_heartbeat) > sessionBeforeMs,
    `before=${sessionBeforeMs} after=${sessionAfter && Date.parse(sessionAfter.last_heartbeat)}`,
  );
  record(
    "refresh:claim-ttl-renewed",
    Boolean(claimAfter) && Date.parse(claimAfter.claimed_at) > claimBeforeMs,
    `before=${claimBeforeMs} after=${claimAfter && Date.parse(claimAfter.claimed_at)}`,
  );
  record(
    "refresh:hold-renewed",
    Boolean(holdAfter) && holdAfter.released_at == null && holdAfterExpiryMs > holdBeforeExpiryMs,
    `before_expiry=${holdBeforeExpiryMs} after_expiry=${holdAfterExpiryMs} released_at=${holdAfter && holdAfter.released_at}`,
  );

  fs.rmSync(root, { recursive: true, force: true });
}

// 3. touch-throw stays green: a sabotaged heartbeatTouch throws
// synchronously; both hooks must still exit 0 (fail-open) with the throw
// logged, never crashing the hook's primary job.
{
  for (const [label, hookPath, event] of [
    ["prompt-context", PROMPT_CONTEXT_HOOK, "UserPromptSubmit"],
    ["state-sync", STATE_SYNC_HOOK, "PostToolUse"],
  ]) {
    const root = mkHookFixture(`bee-heartbeat-throw-${label}-`);
    sabotageHeartbeatTouch(root);
    const sessionId = "sess-throw";

    const result = await runHook(hookPath, root, {
      hook_event_name: event,
      session_id: sessionId,
      cwd: root,
    });

    record(`throw:${label}:exit-0`, result.status === 0, `status=${result.status} stderr=${result.stderr}`);

    const log = readHooksLog(root);
    const crashLine = log.find(
      (entry) => entry.hook === label && String(entry.error || "").includes("sabotage-heartbeat-touch"),
    );
    record(
      `throw:${label}:logged`,
      Boolean(crashLine),
      crashLine ? "crash line found" : `no matching crash line in hooks.jsonl (log=${JSON.stringify(log)})`,
    );

    if (label === "state-sync") {
      // The throw is caught by touch's OWN try/catch, separate from the
      // state RMW below it — the hook's primary job (cells/last_activity
      // refresh) must still have run.
      const state = JSON.parse(readRaw(path.join(root, ".bee", "state.json")));
      record(
        "throw:state-sync:primary-job-ran",
        typeof state.last_activity === "string" && state.last_activity.length > 0,
        `state.last_activity=${JSON.stringify(state.last_activity)}`,
      );
    }

    fs.rmSync(root, { recursive: true, force: true });
  }
}

// 4a. LOCK_BUSY -> silent skip at the lib level: a pre-held sessions.lock
// makes heartbeatTouch's own session-heartbeat write skip (never throw,
// never wait — maxAttempts:1), while claim renewal (gated separately, not
// by this lock) still proceeds.
{
  const root = mkSandbox("bee-heartbeat-lockbusy-");
  const NOW = Date.parse("2026-01-01T00:00:00.000Z");
  const sessionId = "sess-lockbusy";
  createSession(root, { id: sessionId, now: NOW });
  claimCellFile(root, sessionId, "cell-b", 3600, { now: NOW });

  ensureDir(locksDir(root));
  fs.writeFileSync(
    lockFilePath(root, "sessions"),
    `${JSON.stringify({ pid: 999999, session: "someone-else", ts: new Date().toISOString(), token: "holder" })}\n`,
    { encoding: "utf8", flag: "wx" },
  );

  const sessionBefore = readRaw(sessionPath(root, sessionId));

  let touch;
  let threw = null;
  try {
    touch = await heartbeatTouch(root, sessionId, { now: NOW + 70_000 }); // over throttle -> attempts a real write
  } catch (error) {
    threw = error;
  }

  record("lockbusy:no-throw", threw === null, threw ? `heartbeatTouch threw: ${threw}` : "resolved normally");
  record(
    "lockbusy:heartbeat-code",
    Boolean(touch) && touch.touched === true && touch.heartbeat && touch.heartbeat.code === "LOCK_BUSY",
    `touch=${JSON.stringify(touch)}`,
  );
  record(
    "lockbusy:session-untouched",
    readRaw(sessionPath(root, sessionId)) === sessionBefore,
    "session heartbeat file must stay byte-identical when the lock is busy",
  );
  record(
    "lockbusy:claim-still-renewed",
    Boolean(touch) && Array.isArray(touch.claims && touch.claims.renewed) && touch.claims.renewed.includes("cell-b"),
    `touch.claims=${JSON.stringify(touch && touch.claims)}`,
  );

  fs.rmSync(root, { recursive: true, force: true });
}

// 4b. LOCK_BUSY -> silent skip for bee-state-sync's OWN state
// read-modify-write (D3-amended): a pre-held state.lock must leave
// state.json byte-identical and must never surface as a crash log line.
{
  const root = mkHookFixture("bee-statesync-lockbusy-");
  ensureDir(locksDir(root));
  fs.writeFileSync(
    lockFilePath(root, "state"),
    `${JSON.stringify({ pid: 999999, session: "someone-else", ts: new Date().toISOString(), token: "holder" })}\n`,
    { encoding: "utf8", flag: "wx" },
  );

  const stateBefore = readRaw(path.join(root, ".bee", "state.json"));

  const result = await runHook(STATE_SYNC_HOOK, root, {
    hook_event_name: "PostToolUse",
    cwd: root,
  });

  record("statesync-lockbusy:exit-0", result.status === 0, `status=${result.status} stderr=${result.stderr}`);
  record(
    "statesync-lockbusy:state-byte-identical",
    readRaw(path.join(root, ".bee", "state.json")) === stateBefore,
    "state.json must stay untouched when its own RMW loses the lock race",
  );
  const log = readHooksLog(root);
  const crashLine = log.find((entry) => entry.hook === "state-sync" && !entry.event);
  record(
    "statesync-lockbusy:no-crash-log",
    !crashLine,
    crashLine ? `unexpected crash line: ${JSON.stringify(crashLine)}` : "no crash line, as required",
  );

  fs.rmSync(root, { recursive: true, force: true });
}

// 5. renewal-vs-.adopting-gate: a claim gated by an in-flight adopt/sweep is
// SKIPPED by renewClaimTTL and never rewritten; a sibling ungated claim for
// the same session still renews; a different session's claim is never
// touched at all.
{
  const root = mkSandbox("bee-heartbeat-gate-");
  const NOW = Date.parse("2026-01-01T00:00:00.000Z");
  const sessA = "sess-a";
  const sessB = "sess-b";
  claimCellFile(root, sessA, "cell-gated", 3600, { now: NOW });
  claimCellFile(root, sessA, "cell-free", 3600, { now: NOW });
  claimCellFile(root, sessB, "cell-other", 3600, { now: NOW });

  // Simulate another in-flight adopt/sweep holding cell-gated's exclusive
  // gate — the same file acquireGate() itself would create.
  fs.writeFileSync(
    claimGatePath(root, "cell-gated"),
    `${JSON.stringify({ pid: 424242, at: new Date(NOW).toISOString() })}\n`,
    { encoding: "utf8", flag: "wx" },
  );

  const gatedBefore = readRaw(claimPath(root, "cell-gated"));
  const otherBefore = readRaw(claimPath(root, "cell-other"));

  const renewal = renewClaimTTL(root, sessA, { now: NOW + 70_000 });

  record(
    "gate:cell-gated-skipped",
    renewal.skipped.includes("cell-gated") && !renewal.renewed.includes("cell-gated"),
    `renewal=${JSON.stringify(renewal)}`,
  );
  record(
    "gate:cell-gated-never-rewritten",
    readRaw(claimPath(root, "cell-gated")) === gatedBefore,
    "a gate-held claim must stay byte-identical, never rewritten",
  );
  record(
    "gate:cell-free-renewed",
    renewal.renewed.includes("cell-free") &&
      Date.parse(readClaim(root, "cell-free").claimed_at) === NOW + 70_000,
    `renewal=${JSON.stringify(renewal)} claim=${JSON.stringify(readClaim(root, "cell-free"))}`,
  );
  record(
    "gate:different-session-claim-never-touched",
    readRaw(claimPath(root, "cell-other")) === otherBefore,
    "renewClaimTTL(sessA) must never touch a claim owned by sessB",
  );

  fs.rmSync(root, { recursive: true, force: true });
}

// 6. GH #27.1 (D-GHF-B): heartbeat-invariant claim counting — a heartbeat
// renewal (renewClaimTTL) between two failed attempts under the SAME claim
// epoch must not inflate checkCellBudgets' claims_used past the real
// acquisition count. acquired_at is stamped once at claim creation and must
// survive renewClaimTTL's spread untouched, unlike claimed_at (the expiry
// clock) which the heartbeat legitimately advances; checkCellBudgets pairs
// on (claim_session, acquired_at ?? claimed_at) instead of the
// heartbeat-mutated claimed_at alone. max_claims is pinned to 2 so the
// pre-fix bug (heartbeat splits one epoch into two counted pairs -> claims_used
// 3 > 2 -> refused) and the fix (one pair -> claims_used 2 <= 2 -> ok) land on
// opposite sides of the same boundary.
{
  const cellsLib = await import(libUrl("cells.mjs"));
  const { claimCellCrossSession, recordVerify, readCell, checkCellBudgets } = cellsLib;

  const root = mkSandbox("bee-heartbeat-budget-");
  fs.mkdirSync(path.join(root, ".bee", "cells"), { recursive: true });
  fs.writeFileSync(
    path.join(root, ".bee", "state.json"),
    `${JSON.stringify(
      {
        schema_version: "1.0",
        phase: "swarming",
        feature: "hb-budget-feat",
        mode: "standard",
        approved_gates: { context: true, shape: true, execution: true, review: false },
        workers: [],
      },
      null,
      2,
    )}\n`,
  );
  const cellId = "hb-budget-cell";
  fs.writeFileSync(
    path.join(root, ".bee", "cells", `${cellId}.json`),
    `${JSON.stringify(
      {
        id: cellId,
        feature: "hb-budget-feat",
        title: "heartbeat budget test cell",
        lane: "tiny",
        status: "open",
        deps: [],
        action: "heartbeat budget target",
        verify: 'node -e "process.exit(0)"',
        budgets: { max_claims: 2 },
        trace: {},
      },
      null,
      2,
    )}\n`,
  );

  const sessionId = "sess-hb-budget";

  const claimed = await claimCellCrossSession(root, { sessionId, worker: "worker-hb", cellId });
  record("hb-budget:claim-ok", Boolean(claimed.ok), `claimed=${JSON.stringify(claimed)}`);

  await recordVerify(root, cellId, {
    command: "node fail-1.mjs",
    output: "fail one",
    passed: false,
    sessionId,
    signature: "hb-budget-sig-1",
  });

  const heartbeat = renewClaimTTL(root, sessionId, { now: Date.now() + 90_000 });
  record("hb-budget:heartbeat-renewed", heartbeat.renewed.includes(cellId), `heartbeat=${JSON.stringify(heartbeat)}`);

  await recordVerify(root, cellId, {
    command: "node fail-2.mjs",
    output: "fail two",
    passed: false,
    sessionId,
    signature: "hb-budget-sig-2",
  });

  const cellAfter = readCell(root, cellId);
  const attempts = (cellAfter.trace && cellAfter.trace.attempts) || [];
  record("hb-budget:two-attempts-recorded", attempts.length === 2, `attempts=${JSON.stringify(attempts)}`);
  record(
    "hb-budget:claimed_at-advanced-by-heartbeat",
    Boolean(attempts[0]) && Boolean(attempts[1]) && attempts[0].claimed_at !== attempts[1].claimed_at,
    `attempt0.claimed_at=${attempts[0] && attempts[0].claimed_at} attempt1.claimed_at=${attempts[1] && attempts[1].claimed_at}`,
  );
  record(
    "hb-budget:acquired_at-stable-across-heartbeat",
    Boolean(attempts[0]) &&
      Boolean(attempts[0].acquired_at) &&
      attempts[0].acquired_at === (attempts[1] && attempts[1].acquired_at),
    `attempt0.acquired_at=${attempts[0] && attempts[0].acquired_at} attempt1.acquired_at=${attempts[1] && attempts[1].acquired_at}`,
  );

  const budgetCheck = checkCellBudgets(cellAfter);
  record(
    "hb-budget:checkCellBudgets-counts-one-acquisition",
    budgetCheck.ok === true,
    `budgetCheck=${JSON.stringify(budgetCheck)} (claims_used must be 2 = 1 heartbeat-collapsed pair + 1 current, ` +
      "under max_claims=2; a heartbeat wrongly splitting the epoch would push it to 3 and refuse)",
  );

  fs.rmSync(root, { recursive: true, force: true });
}

// --- report ------------------------------------------------------------------

for (const r of results) {
  process.stdout.write(`${r.pass ? "ok  " : "FAIL"} - ${r.name} :: ${r.note}\n`);
}
const failures = results.filter((r) => !r.pass);
process.stdout.write(`\n${results.length} rows, ${failures.length} failing\n`);
process.stdout.write(`\n${failures.length === 0 ? "ALL PASS" : `${failures.length} FAILURE(S)`}\n`);
process.exitCode = failures.length === 0 ? 0 : 1;
