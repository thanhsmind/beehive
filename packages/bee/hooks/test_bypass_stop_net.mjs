#!/usr/bin/env node
// test_bypass_stop_net.mjs — fixture test for the mechanical gate-bypass net
// (GitHub #18) added to hooks/bee-session-close.mjs. Honoring gate_bypass was
// prose-only: nothing caught the model when it stopped at Gate 2/3 anyway. The
// Stop hook now returns decision:"block" (continue the turn) when the session
// tries to stop mid-planning/validating with a gate the active bypass level
// should have auto-approved. This proves the fire/no-fire matrix, the
// once-per-key loop-guard, PreCompact/SubagentStop never blocking, and
// fail-open.
//
// Spawns hooks/bee-session-close.mjs as an isolated module worker (same pattern
// as test_write_guard.mjs), feeds a Stop/PreCompact payload on stdin, and
// asserts whether stdout carries {"decision":"block"}. fakeHome isolates the
// perf-refresh side write so no test run touches the real ~/.claude tree.
// Exits 1 on any failure.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { runModuleWorker } from "../../../scripts/lib/run-module-worker.mjs";

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const HOOKS_DIR = path.dirname(SCRIPT_PATH);
const REPO_ROOT = path.dirname(path.dirname(path.dirname(HOOKS_DIR)));
const HOOK_PATH = path.join(HOOKS_DIR, "bee-session-close.mjs");
const REAL_LIB_DIR = path.join(REPO_ROOT, ".bee", "bin", "lib");

let failures = 0;
function check(condition, label, extra = "") {
  if (condition) {
    process.stdout.write(`ok    - ${label}\n`);
  } else {
    failures += 1;
    process.stdout.write(`FAIL  - ${label}${extra ? ` :: ${extra}` : ""}\n`);
  }
}

// --- fixture builders --------------------------------------------------

function copyLib(fixtureRoot) {
  const libDir = path.join(fixtureRoot, ".bee", "bin", "lib");
  fs.mkdirSync(libDir, { recursive: true });
  for (const name of fs.readdirSync(REAL_LIB_DIR)) {
    if (!name.endsWith(".mjs")) continue;
    fs.copyFileSync(path.join(REAL_LIB_DIR, name), path.join(libDir, name));
  }
}

// A working fixture at a chosen phase/mode/gate-state with a chosen bypass level.
function buildFixture({
  prefix = "bee-bypass-net-",
  phase = "planning",
  mode = "standard",
  gateBypass = "total",
  approved_gates = { context: false, shape: false, execution: false, review: false },
} = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(root, ".bee"), { recursive: true });
  fs.writeFileSync(path.join(root, ".bee", "onboarding.json"), "{}\n");
  copyLib(root);
  fs.writeFileSync(
    path.join(root, ".bee", "state.json"),
    `${JSON.stringify({ phase, mode, feature: "demo", approved_gates }, null, 2)}\n`,
  );
  fs.writeFileSync(
    path.join(root, ".bee", "config.json"),
    `${JSON.stringify({ gate_bypass: gateBypass }, null, 2)}\n`,
  );
  return root;
}

// Run the Stop hook once against a fixture. Returns { fired, status, stdout }.
async function runHook(root, { event = "Stop", sessionId = "sess-1" } = {}) {
  const payload = { hook_event_name: event, session_id: sessionId, cwd: root };
  const fakeHome = fs.mkdtempSync(path.join(os.tmpdir(), "bee-bypass-home-"));
  const res = await runModuleWorker(HOOK_PATH, {
    input: JSON.stringify(payload),
    cwd: root,
    fakeHome,
  });
  const stdout = res.stdout || "";
  return { fired: stdout.includes('"decision":"block"'), status: res.status, stdout, stderr: res.stderr };
}

// --- rows --------------------------------------------------------------

async function main() {
  // 1. Happy path: total + planning + shape pending + Stop → block once.
  {
    const root = buildFixture({ phase: "planning", gateBypass: "total" });
    const r = await runHook(root);
    check(r.fired, "total + planning + shape-pending + Stop → decision:block", r.stdout.slice(0, 120));
    check(
      r.stdout.includes("Gate 2") && r.stdout.includes("shape"),
      "block reason names Gate 2 / shape",
    );
  }

  // 2. Loop-guard: an immediate same-key re-stop degrades to advisory (no block).
  {
    const root = buildFixture({ phase: "planning", gateBypass: "total" });
    const first = await runHook(root, { sessionId: "loop-s" });
    const second = await runHook(root, { sessionId: "loop-s" });
    check(first.fired, "loop-guard: first Stop at gate → block");
    check(!second.fired, "loop-guard: immediate same-key re-stop → NO block (advisory)", second.stdout.slice(0, 120));
  }

  // 3. validating → execution / Gate 3.
  {
    const root = buildFixture({ phase: "validating", gateBypass: "total" });
    const r = await runHook(root);
    check(r.fired && r.stdout.includes("Gate 3") && r.stdout.includes("execution"),
      "total + validating + execution-pending + Stop → block (Gate 3)");
  }

  // 3b. H3: high-risk validating/execution instruction names the AO3/AO13
  // advisor-consult prerequisite before "Set the gate yourself now".
  {
    const root = buildFixture({ phase: "validating", mode: "high-risk", gateBypass: "total" });
    const r = await runHook(root);
    check(
      r.fired &&
        r.stdout.includes("state advisor-ref record") &&
        r.stdout.includes("AO3/AO13") &&
        r.stdout.indexOf("advisor consult") < r.stdout.indexOf("Set the gate yourself now"),
      "total + high-risk + validating/execution → block names advisor-consult prerequisite before setting the gate",
      r.stdout,
    );
  }

  // 3c. Non-high-risk execution instruction stays byte-unchanged — no consult
  // sentence, straight from "do NOT ask the human." to "Set the gate yourself now".
  {
    const root = buildFixture({ phase: "validating", mode: "standard", gateBypass: "total" });
    const r = await runHook(root);
    check(
      r.fired &&
        !r.stdout.includes("advisor-ref record") &&
        !r.stdout.includes("AO3/AO13") &&
        r.stdout.includes("do NOT ask the human. Set the gate yourself now:"),
      "total + standard mode + validating/execution → block instruction unchanged (no consult sentence)",
      r.stdout,
    );
  }

  // 4. exploring phase is never mechanized (Gate 1 info questions still stop).
  {
    const root = buildFixture({ phase: "exploring", gateBypass: "total" });
    const r = await runHook(root);
    check(!r.fired, "total + exploring phase → NO block (Gate 1 excluded)", r.stdout.slice(0, 120));
  }

  // 5. gate already approved → nothing to force.
  {
    const root = buildFixture({
      phase: "planning",
      gateBypass: "total",
      approved_gates: { context: true, shape: true, execution: false, review: false },
    });
    const r = await runHook(root);
    check(!r.fired, "total + planning + shape ALREADY approved → NO block");
  }

  // 6. off → never fires.
  {
    const root = buildFixture({ phase: "planning", gateBypass: false });
    const r = await runHook(root);
    check(!r.fired, "gate_bypass off + planning → NO block");
  }

  // 7. normal + standard mode → covered → block.
  {
    const root = buildFixture({ phase: "planning", mode: "standard", gateBypass: "normal" });
    const r = await runHook(root);
    check(r.fired, "normal + standard mode + planning → block");
  }

  // 8. normal + high-risk mode → floor holds → no block.
  {
    const root = buildFixture({ phase: "planning", mode: "high-risk", gateBypass: "normal" });
    const r = await runHook(root);
    check(!r.fired, "normal + high-risk mode + planning → NO block (floor holds)");
  }

  // 9. PreCompact must NEVER block, even with everything else set to fire.
  {
    const root = buildFixture({ phase: "planning", gateBypass: "total" });
    const r = await runHook(root, { event: "PreCompact" });
    check(!r.fired, "PreCompact event + total + planning → NO block", r.stdout.slice(0, 120));
  }

  // 10. Missing/empty event → fail-safe, no block.
  {
    const root = buildFixture({ phase: "planning", gateBypass: "total" });
    const r = await runHook(root, { event: "" });
    check(!r.fired, "empty event + total + planning → NO block (fail-safe)");
  }

  // 11. Fail-open: a throwing inject.mjs must not crash the hook or block.
  {
    const root = buildFixture({ phase: "planning", gateBypass: "total" });
    fs.writeFileSync(
      path.join(root, ".bee", "bin", "lib", "inject.mjs"),
      "throw new Error('boom: fixture inject.mjs throws on import');\n",
    );
    const r = await runHook(root);
    check(!r.fired, "fail-open: throwing inject.mjs → NO block (advisory fallthrough)");
    check(r.status === 0 || r.status === undefined || r.status === null,
      "fail-open: hook still exits 0", `status=${r.status}`);
  }

  process.stdout.write(`\n${failures === 0 ? "ALL PASS" : `${failures} FAILURE(S)`}\n`);
  process.exitCode = failures === 0 ? 0 : 1;
}

await main();
