#!/usr/bin/env node
// test_dispatch_prepare.mjs — contract tests for `bee dispatch prepare`
// (g22-1, GH #22 P0-3): the payload/economics builder in
// skills/bee-hive/templates/lib/dispatch-prepare.mjs, run both through the
// real dispatcher (bee.mjs) and, for the headline round-trip, directly
// against the REAL dispatch-guard.mjs evaluateDispatch() the PreToolUse hook
// enforces — proving prepare and the guard share one vocabulary rather than
// two copies of the same judgment call (advisor A2).
//
// Self-contained, no framework, mirrors hooks/test_model_guard.mjs's
// check()/assert() style. Exits 1 on any failure.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { runModuleWorker } from "../lib/run-module-worker.mjs";

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const REPO_ROOT = path.dirname(path.dirname(path.dirname(SCRIPT_PATH)));
const TEMPLATES_LIB = path.join(REPO_ROOT, "skills", "bee-hive", "templates", "lib");
const BEE_MJS = path.join(REPO_ROOT, "skills", "bee-hive", "templates", "bee.mjs");

// Direct lib imports (codex-native-transport cnt-3): the native-override
// branch needs a `classification` value prepareDispatch never derives itself
// (see its docstring) — calling it directly, bypassing the real
// readNativeTransportClassification probe/CLI round-trip, is the only way to
// exercise a CONFIRMED native_model_override golden row deterministically,
// since no real host in this environment ships multi_agent_v2 (E3: "under
// development" everywhere as of codex-cli 0.144.4) for the real reader to
// ever confirm.
const {
  evaluateDispatch,
  NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE,
  NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY,
  NATIVE_TRANSPORT_EXTERNAL_CLI_ONLY,
} = await import(pathToFileURL(path.join(TEMPLATES_LIB, "dispatch-guard.mjs")).href);
const { prepareDispatch } = await import(pathToFileURL(path.join(TEMPLATES_LIB, "dispatch-prepare.mjs")).href);

let passed = 0;
let failed = 0;

async function check(name, fn) {
  try {
    await fn();
    passed += 1;
    console.log(`PASS  ${name}`);
  } catch (error) {
    failed += 1;
    console.log(`FAIL  ${name}`);
    console.log(`      ${error instanceof Error ? error.stack || error.message : error}`);
  }
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

// ─── fixture builders ───────────────────────────────────────────────────────

function mkFixture(prefix) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(root, ".bee"), { recursive: true });
  fs.writeFileSync(path.join(root, ".bee", "onboarding.json"), '{"schema_version":"1.0","bee_version":"0.1.0"}\n');
  return root;
}

function writeConfig(root, models) {
  fs.writeFileSync(
    path.join(root, ".bee", "config.json"),
    `${JSON.stringify({ models }, null, 2)}\n`,
  );
}

function writeCellFixture(root, cell) {
  const cellsDir = path.join(root, ".bee", "cells");
  fs.mkdirSync(cellsDir, { recursive: true });
  fs.writeFileSync(
    path.join(cellsDir, `${cell.id}.json`),
    `${JSON.stringify(
      {
        status: "open",
        lane: "small",
        trace: {},
        ...cell,
      },
      null,
      2,
    )}\n`,
  );
}

function readLastJsonl(file) {
  if (!fs.existsSync(file)) return null;
  const lines = fs
    .readFileSync(file, "utf8")
    .split(/\r?\n/)
    .filter((l) => l.trim());
  if (lines.length === 0) return null;
  return JSON.parse(lines[lines.length - 1]);
}

async function runBee(args, cwd) {
  return runModuleWorker(BEE_MJS, { args, cwd });
}

async function prepareOk(args, cwd) {
  const result = await runBee(["dispatch", "prepare", ...args, "--json"], cwd);
  assert(result.status === 0, `dispatch prepare ${args.join(" ")} exited ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  return JSON.parse(result.stdout);
}

// ─── THE HEADLINE (advisor A2): prepare's codex output through the REAL
// evaluateDispatch — proving prepare and the guard share one vocabulary. ────

await check("codex gather payload round-trips ALLOW through the real evaluateDispatch; a stripped marker DENIES", async () => {
  const root = mkFixture("dispatch-prepare-codex-headline-");
  writeConfig(root, { codex: { extraction: "gpt-5.5", generation: "gpt-5.5", review: null } });

  const out = await prepareOk(["--runtime", "codex", "--kind", "gather"], root);
  assert(out.tool === "spawn_agent", `expected tool spawn_agent, got ${JSON.stringify(out)}`);
  // i54-closeout D1 (validation-canary, live codex 0.145.0): the observed
  // schema requires task_name + message; agent_type does not exist in it
  // (R18: never emit a field the probe did not observe).
  assert(out.payload.task_name === "bee-gather", `expected task_name "bee-gather" (0.145.0: task_name is required), got ${JSON.stringify(out.payload)}`);
  assert(!("agent_type" in out.payload), `agent_type does not exist in the live-probed 0.145.0 schema (R18), got ${JSON.stringify(out.payload)}`);
  assert(out.payload.fork_turns === "none", `expected fork_turns "none" (ORCH-02 isolation), got ${JSON.stringify(out.payload)}`);
  assert(
    /^\[bee-tier: generation\]/.test(out.payload.message),
    `expected message to open with the generation marker, got ${JSON.stringify(out.payload.message)}`,
  );

  const dispatchGuard = await import(pathToFileURL(path.join(TEMPLATES_LIB, "dispatch-guard.mjs")).href);

  const allowed = dispatchGuard.evaluateDispatch("spawn_agent", out.payload, root);
  assert(allowed.decision === "allow", `expected ALLOW for prepare's own codex payload, got ${JSON.stringify(allowed)}`);
  assert(
    allowed.transport === "codex-spawn-marker" && allowed.tier === "generation",
    `expected a real codex-spawn-marker verdict (never noOpinion) for the helper-emitted shape, got ${JSON.stringify(allowed)}`,
  );

  const stripped = { ...out.payload, message: out.payload.message.replace(/^\[bee-tier: generation\]\n/, "") };
  const denied = dispatchGuard.evaluateDispatch("spawn_agent", stripped, root);
  assert(denied.decision === "deny", `expected DENY once the marker is stripped, got ${JSON.stringify(denied)}`);
  assert(denied.transport === "codex-spawn-unmarked", `expected codex-spawn-unmarked, got ${JSON.stringify(denied)}`);
});

// ─── i54-closeout D1: the doc-canonical shape (swarming-reference.md "Spawn"
// row) and the legacy 0.144.4 shape BOTH get real guard verdicts — marked =
// allow, unmarked = deny, never a silent noOpinion. This is exactly the
// untested direction issue #54 flagged. ─────────────────────────────────────

await check("doc-canonical {task_name, message, fork_turns} AND legacy {agent_type, message} payloads round-trip marked=ALLOW / unmarked=DENY through the guard", async () => {
  const root = mkFixture("dispatch-prepare-doc-canonical-");
  writeConfig(root, { codex: { extraction: "gpt-5.5", generation: "gpt-5.5", review: null } });

  const docCanonical = { task_name: "wt-a1", message: "[bee-tier: generation]\ndo the gather", fork_turns: "none" };
  const allowed = evaluateDispatch("spawn_agent", docCanonical, root);
  assert(
    allowed.decision === "allow" && allowed.transport === "codex-spawn-marker" && allowed.tier === "generation",
    `expected a real ALLOW verdict for the doc-canonical shape, got ${JSON.stringify(allowed)}`,
  );

  const unmarked = { ...docCanonical, message: "do the gather, no marker" };
  const denied = evaluateDispatch("spawn_agent", unmarked, root);
  assert(
    denied.decision === "deny" && denied.transport === "codex-spawn-unmarked",
    `expected DENY for the unmarked doc-canonical shape (never noOpinion), got ${JSON.stringify(denied)}`,
  );

  const legacyMarked = { agent_type: "worker", message: "[bee-tier: extraction]\nlegacy 0.144.4 shape" };
  const legacyAllowed = evaluateDispatch("spawn_agent", legacyMarked, root);
  assert(
    legacyAllowed.decision === "allow" && legacyAllowed.transport === "codex-spawn-marker",
    `expected the legacy 0.144.4 shape to stay ALLOWed when marked, got ${JSON.stringify(legacyAllowed)}`,
  );

  const legacyUnmarked = { agent_type: "worker", message: "legacy shape, no marker" };
  const legacyDenied = evaluateDispatch("spawn_agent", legacyUnmarked, root);
  assert(
    legacyDenied.decision === "deny" && legacyDenied.transport === "codex-spawn-unmarked",
    `expected the legacy unmarked shape to stay DENIED (deny never weakened), got ${JSON.stringify(legacyDenied)}`,
  );
});

// ─── Claude payload through evaluateDispatch's claude branch ───────────────

await check("claude gather payload round-trips ALLOW through evaluateDispatch; general-purpose + the same marker DENIES", async () => {
  const root = mkFixture("dispatch-prepare-claude-headline-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus" } });

  const out = await prepareOk(["--runtime", "claude", "--kind", "gather"], root);
  assert(out.tool === "Agent", `expected tool Agent, got ${JSON.stringify(out)}`);
  assert(out.payload.subagent_type === "bee-gather", `expected pinned type bee-gather, got ${JSON.stringify(out.payload)}`);
  assert(out.payload.model === "sonnet", `expected model sonnet, got ${JSON.stringify(out.payload)}`);
  assert(
    /^\[bee-tier: generation\]/.test(out.payload.prompt),
    `expected prompt to open with the generation marker, got ${JSON.stringify(out.payload.prompt)}`,
  );
  assert(out.payload.description === "gather (sonnet)", `expected description "gather (sonnet)", got ${JSON.stringify(out.payload.description)}`);

  const dispatchGuard = await import(pathToFileURL(path.join(TEMPLATES_LIB, "dispatch-guard.mjs")).href);

  const allowed = dispatchGuard.evaluateDispatch("Agent", out.payload, root);
  assert(allowed.decision === "allow", `expected ALLOW for prepare's own claude payload, got ${JSON.stringify(allowed)}`);
  assert(allowed.transport === "model-param", `expected model-param transport, got ${JSON.stringify(allowed)}`);

  const generalPurpose = { ...out.payload, subagent_type: "general-purpose" };
  const denied = dispatchGuard.evaluateDispatch("Agent", generalPurpose, root);
  assert(denied.decision === "deny", `expected DENY once subagent_type flips to general-purpose, got ${JSON.stringify(denied)}`);
  assert(denied.transport === "generic-type-denied", `expected generic-type-denied, got ${JSON.stringify(denied)}`);
});

// ─── claude channel economics (g22-2, GH #22 P1-6 D3): a real model param on
// the payload is 'pinned' and effective_model equals that param — never null
// — because we watched the caller build a structural pin, not merely a
// prompt-stated budget. ──────────────────────────────────────────────────

await check("claude channel economics: model-param enforcement, pinned status, effective_model equals the payload's model", async () => {
  const root = mkFixture("dispatch-prepare-claude-economics-pinned-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus" } });

  const out = await prepareOk(["--runtime", "claude", "--kind", "gather"], root);
  assert(out.payload.model === "sonnet", `expected payload.model sonnet, got ${JSON.stringify(out.payload)}`);
  assert(out.economics.channel === "claude-agent", `expected channel claude-agent, got ${JSON.stringify(out.economics)}`);
  assert(out.economics.enforcement === "model-param", `expected enforcement model-param, got ${JSON.stringify(out.economics)}`);
  assert(out.economics.effective_model_status === "pinned", `expected pinned status, got ${JSON.stringify(out.economics)}`);
  assert(out.economics.effective_model === "sonnet", `expected effective_model sonnet (a real param was passed), got ${JSON.stringify(out.economics)}`);
  assert(out.economics.requested_model === "sonnet", `expected requested_model sonnet, got ${JSON.stringify(out.economics)}`);
});

// ─── claude channel economics on a null (budget-shaped) generation slot: no
// structural model param exists to pin, so the honest status is 'unverified'
// — NEVER 'inherited-or-unknown' (that status is codex-native-only; a claude
// dispatch relying on the prompt-stated tier budget is a different, weaker
// claim than "the runtime has no per-agent model selection at all"). ──────

await check("claude channel economics on a budget-shaped (null) generation slot: prompt-budget enforcement, unverified status, no effective_model", async () => {
  const root = mkFixture("dispatch-prepare-claude-economics-budget-");
  writeConfig(root, { claude: { extraction: "haiku", generation: null, review: "opus" } });

  const out = await prepareOk(["--runtime", "claude", "--kind", "gather"], root);
  assert(!("model" in out.payload), `a budget-shaped slot must carry no structural model param, got ${JSON.stringify(out.payload)}`);
  assert(out.economics.channel === "claude-agent", `expected channel claude-agent, got ${JSON.stringify(out.economics)}`);
  assert(out.economics.enforcement === "prompt-budget", `expected enforcement prompt-budget, got ${JSON.stringify(out.economics)}`);
  assert(
    out.economics.effective_model_status === "unverified",
    `expected unverified status (never inherited-or-unknown — that's codex-only), got ${JSON.stringify(out.economics)}`,
  );
  assert(out.economics.effective_model === null, `expected effective_model null, got ${JSON.stringify(out.economics)}`);
  assert(out.economics.requested_model === null, `a budget-shaped (null) slot names no model, got ${JSON.stringify(out.economics)}`);
});

// ─── cli-cell refusal: prepare NEVER routes around a refusal ───────────────

await check("kind cell against a cli-shaped generation slot returns the typed cli_tier_gather_only refusal, never a payload", async () => {
  const root = mkFixture("dispatch-prepare-cli-cell-");
  writeConfig(root, {
    claude: {
      extraction: "haiku",
      generation: { kind: "cli", command: "codex exec -m gpt-5.5 -s read-only -" },
      review: "opus",
    },
  });
  writeCellFixture(root, {
    id: "demo-1",
    feature: "demo",
    title: "Demo cell",
    action: "Do the demo thing.",
    verify: 'node -e "process.exit(0)"',
    status: "claimed",
    trace: { worker: "exec-demo-1" },
  });

  const result = await runBee(["dispatch", "prepare", "--runtime", "claude", "--kind", "cell", "--cell", "demo-1", "--worker", "exec-demo-1", "--json"], root);
  assert(result.status === 0, `expected exit 0 (a typed refusal is not a crash), got ${result.status}: ${result.stderr}`);
  const out = JSON.parse(result.stdout);
  assert(out.ok === false, `expected ok:false, got ${JSON.stringify(out)}`);
  assert(out.type === "refused" && out.reason === "cli_tier_gather_only", `expected the typed cli_tier_gather_only refusal, got ${JSON.stringify(out)}`);
  assert(out.slot === "generation", `expected slot generation, got ${JSON.stringify(out)}`);
  assert(typeof out.fix === "string" && out.fix, `expected a fix string, got ${JSON.stringify(out)}`);
  assert(!("tool" in out) && !("payload" in out), `a refusal must never carry a payload, got ${JSON.stringify(out)}`);
});

// ─── kind cell on a normal (non-cli) generation slot builds a real payload
// from the loaded cell's own id/feature ──────────────────────────────────

await check("kind cell on a model-shaped generation slot loads the cell and embeds its id/feature in the prompt", async () => {
  const root = mkFixture("dispatch-prepare-cell-ok-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus" } });
  writeCellFixture(root, {
    id: "demo-7",
    feature: "widget-feature",
    title: "Implement the widget",
    action: "Implement the widget end to end.",
    verify: 'node -e "process.exit(0)"',
    status: "claimed",
    trace: { worker: "exec-demo-7" },
  });

  const out = await prepareOk(["--runtime", "claude", "--kind", "cell", "--cell", "demo-7", "--worker", "exec-demo-7"], root);
  assert(out.tool === "Agent", `expected tool Agent, got ${JSON.stringify(out)}`);
  assert(out.payload.prompt.includes("demo-7"), `expected the cell id in the prompt, got ${out.payload.prompt}`);
  assert(out.payload.prompt.includes("widget-feature"), `expected the feature in the prompt, got ${out.payload.prompt}`);
  assert(out.economics.logical_tier === "generation", `expected logical_tier generation, got ${JSON.stringify(out.economics)}`);
});

await check("kind cell without --cell throws (non-zero exit, no payload)", async () => {
  const root = mkFixture("dispatch-prepare-cell-missing-flag-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus" } });
  const result = await runBee(["dispatch", "prepare", "--runtime", "claude", "--kind", "cell", "--json"], root);
  assert(result.status !== 0, `expected non-zero exit, got ${result.status}`);
});

await check("kind cell against an unknown cell id throws (non-zero exit)", async () => {
  const root = mkFixture("dispatch-prepare-cell-unknown-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus" } });
  const result = await runBee(["dispatch", "prepare", "--runtime", "claude", "--kind", "cell", "--cell", "ghost-1", "--json"], root);
  assert(result.status !== 0, `expected non-zero exit, got ${result.status}`);
});

await check("kind cell without --worker throws (non-zero exit, no payload)", async () => {
  const root = mkFixture("dispatch-prepare-cell-missing-worker-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus" } });
  writeCellFixture(root, {
    id: "demo-8",
    feature: "demo",
    title: "Demo cell",
    action: "Do the demo thing.",
    verify: 'node -e "process.exit(0)"',
    status: "claimed",
    trace: { worker: "exec-demo-8" },
  });
  const result = await runBee(["dispatch", "prepare", "--runtime", "claude", "--kind", "cell", "--cell", "demo-8", "--json"], root);
  assert(result.status !== 0, `expected non-zero exit, got ${result.status}`);
});

// ─── hardening-7: claim-ownership guard (dispatch-prepare.mjs
// checkCellClaimOwnership) — mirrors msh-4's audited-door pattern, but
// checks the CELL RECORD's own status/trace.worker (never the claims.mjs
// session store, which dispatch prepare never touches). ───────────────────

await check("hardening-7: kind cell against an unclaimed (open) cell is refused, naming its actual status", async () => {
  const root = mkFixture("dispatch-prepare-not-claimed-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus" } });
  writeCellFixture(root, {
    id: "open-1",
    feature: "demo",
    title: "Open cell",
    action: "Do the demo thing.",
    verify: 'node -e "process.exit(0)"',
    // status defaults to "open" (writeCellFixture default) — never claimed.
  });

  const result = await runBee(["dispatch", "prepare", "--runtime", "claude", "--kind", "cell", "--cell", "open-1", "--worker", "exec-open-1", "--json"], root);
  assert(result.status === 0, `expected exit 0 (a typed refusal is not a crash), got ${result.status}: ${result.stderr}`);
  const out = JSON.parse(result.stdout);
  assert(out.ok === false && out.type === "refused" && out.reason === "claim_ownership", `expected the typed claim_ownership refusal, got ${JSON.stringify(out)}`);
  assert(out.code === "not_claimed", `expected code not_claimed, got ${JSON.stringify(out)}`);
  assert(out.status === "open", `expected the actual status "open" named, got ${JSON.stringify(out)}`);
  assert(out.owner === null, `expected owner null (never claimed), got ${JSON.stringify(out)}`);
  assert(!("tool" in out) && !("payload" in out), `a refusal must never carry a payload, got ${JSON.stringify(out)}`);
});

await check("hardening-7: kind cell claimed by a DIFFERENT worker is refused, naming the actual owner", async () => {
  const root = mkFixture("dispatch-prepare-not-owner-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus" } });
  writeCellFixture(root, {
    id: "owned-1",
    feature: "demo",
    title: "Owned cell",
    action: "Do the demo thing.",
    verify: 'node -e "process.exit(0)"',
    status: "claimed",
    trace: { worker: "exec-owner-a" },
  });

  const result = await runBee(["dispatch", "prepare", "--runtime", "claude", "--kind", "cell", "--cell", "owned-1", "--worker", "exec-owner-b", "--json"], root);
  assert(result.status === 0, `expected exit 0 (a typed refusal is not a crash), got ${result.status}: ${result.stderr}`);
  const out = JSON.parse(result.stdout);
  assert(out.ok === false && out.type === "refused" && out.reason === "claim_ownership", `expected the typed claim_ownership refusal, got ${JSON.stringify(out)}`);
  assert(out.code === "not_owner", `expected code not_owner, got ${JSON.stringify(out)}`);
  assert(out.owner === "exec-owner-a", `expected the actual owner "exec-owner-a" named, got ${JSON.stringify(out)}`);
  assert(!("tool" in out) && !("payload" in out), `a refusal must never carry a payload, got ${JSON.stringify(out)}`);
});

await check("hardening-7: kind cell claimed by the MATCHING worker proceeds unaffected — a real payload, no refusal", async () => {
  const root = mkFixture("dispatch-prepare-matching-owner-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus" } });
  writeCellFixture(root, {
    id: "owned-2",
    feature: "demo",
    title: "Owned cell",
    action: "Do the demo thing.",
    verify: 'node -e "process.exit(0)"',
    status: "claimed",
    trace: { worker: "exec-owner-a" },
  });

  const out = await prepareOk(["--runtime", "claude", "--kind", "cell", "--cell", "owned-2", "--worker", "exec-owner-a"], root);
  assert(out.tool === "Agent", `expected a real Agent payload for the matching owner, got ${JSON.stringify(out)}`);
  const line = readLastJsonl(path.join(root, ".bee", "logs", "dispatch.jsonl"));
  assert(!("ownership_override" in line), `a matching-owner dispatch must never carry an ownership_override, got ${JSON.stringify(line)}`);
});

await check("hardening-7: --force-ownership overrides a not_owner refusal and appends an audited ownership_override record", async () => {
  const root = mkFixture("dispatch-prepare-force-ownership-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus" } });
  writeCellFixture(root, {
    id: "owned-3",
    feature: "demo",
    title: "Owned cell",
    action: "Do the demo thing.",
    verify: 'node -e "process.exit(0)"',
    status: "claimed",
    trace: { worker: "exec-owner-a" },
  });

  const out = await prepareOk(["--runtime", "claude", "--kind", "cell", "--cell", "owned-3", "--worker", "exec-rescuer", "--force-ownership"], root);
  assert(out.tool === "Agent", `expected --force-ownership to still build a real payload, got ${JSON.stringify(out)}`);
  const line = readLastJsonl(path.join(root, ".bee", "logs", "dispatch.jsonl"));
  assert(line && line.ownership_override, `expected an audited ownership_override record, got ${JSON.stringify(line)}`);
  assert(line.ownership_override.forced_by === "exec-rescuer", `expected forced_by exec-rescuer, got ${JSON.stringify(line.ownership_override)}`);
  assert(line.ownership_override.bypassed === true, `expected bypassed true (there WAS a real conflict), got ${JSON.stringify(line.ownership_override)}`);
  assert(line.ownership_override.code === "not_owner", `expected code not_owner, got ${JSON.stringify(line.ownership_override)}`);
  assert(line.ownership_override.owner_bypassed === "exec-owner-a", `expected owner_bypassed exec-owner-a, got ${JSON.stringify(line.ownership_override)}`);
});

await check("hardening-7: --force-ownership on an unclaimed cell also proceeds, audited with bypassed:true, code not_claimed", async () => {
  const root = mkFixture("dispatch-prepare-force-ownership-unclaimed-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus" } });
  writeCellFixture(root, {
    id: "open-2",
    feature: "demo",
    title: "Open cell",
    action: "Do the demo thing.",
    verify: 'node -e "process.exit(0)"',
    // never claimed
  });

  const out = await prepareOk(["--runtime", "claude", "--kind", "cell", "--cell", "open-2", "--worker", "exec-rescuer", "--force-ownership"], root);
  assert(out.tool === "Agent", `expected --force-ownership to still build a real payload for an unclaimed cell, got ${JSON.stringify(out)}`);
  const line = readLastJsonl(path.join(root, ".bee", "logs", "dispatch.jsonl"));
  assert(line.ownership_override.code === "not_claimed", `expected code not_claimed, got ${JSON.stringify(line.ownership_override)}`);
  assert(line.ownership_override.owner_bypassed === null, `expected owner_bypassed null (never claimed, nothing to name), got ${JSON.stringify(line.ownership_override)}`);
});

// ─── advisor kind resolves the ADVISOR model, never generation's ──────────

await check("kind advisor resolves the configured advisor model, not the generation model", async () => {
  const root = mkFixture("dispatch-prepare-advisor-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus", advisor: "fable" } });

  const out = await prepareOk(["--runtime", "claude", "--kind", "advisor"], root);
  assert(out.economics.requested_model === "fable", `expected advisor model fable, got ${JSON.stringify(out.economics)}`);
  assert(out.economics.requested_model !== "sonnet", `advisor must never resolve the generation model, got ${JSON.stringify(out.economics)}`);
  assert(out.payload.model === "fable", `expected payload.model fable, got ${JSON.stringify(out.payload)}`);
  assert(out.economics.logical_tier === "advisor", `expected logical_tier advisor, got ${JSON.stringify(out.economics)}`);
});

await check("kind advisor with no advisor slot configured returns a typed refusal, never a payload", async () => {
  const root = mkFixture("dispatch-prepare-advisor-unconfigured-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus" } });

  const result = await runBee(["dispatch", "prepare", "--runtime", "claude", "--kind", "advisor", "--json"], root);
  assert(result.status === 0, `expected exit 0 (a typed refusal is not a crash), got ${result.status}: ${result.stderr}`);
  const out = JSON.parse(result.stdout);
  assert(out.ok === false && out.reason === "advisor_not_configured", `expected the advisor_not_configured refusal, got ${JSON.stringify(out)}`);
  assert(!("tool" in out) && !("payload" in out), `a refusal must never carry a payload, got ${JSON.stringify(out)}`);
});

// ─── reviewer kind resolves the review slot, and a cli-shaped review slot
// builds an external-executor payload instead of an Agent/spawn_agent one ──

await check("kind reviewer resolves the review slot (not generation)", async () => {
  const root = mkFixture("dispatch-prepare-reviewer-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus" } });

  const out = await prepareOk(["--runtime", "claude", "--kind", "reviewer"], root);
  assert(out.payload.model === "opus", `expected review model opus, got ${JSON.stringify(out.payload)}`);
  assert(out.payload.subagent_type === "bee-review", `expected pinned type bee-review, got ${JSON.stringify(out.payload)}`);
  assert(out.economics.logical_tier === "review", `expected logical_tier review, got ${JSON.stringify(out.economics)}`);
});

await check("a cli-shaped review slot builds a Bash-shaped external-executor payload for kind reviewer, not Agent/spawn_agent", async () => {
  const root = mkFixture("dispatch-prepare-cli-reviewer-");
  writeConfig(root, {
    claude: {
      extraction: "haiku",
      generation: "sonnet",
      review: { kind: "cli", command: "codex exec -m gpt-5.5 -s read-only -" },
    },
  });

  const out = await prepareOk(["--runtime", "claude", "--kind", "reviewer"], root);
  assert(out.tool === "Bash", `expected tool Bash for a cli-shaped review slot, got ${JSON.stringify(out)}`);
  assert(out.payload.command === "codex exec -m gpt-5.5 -s read-only -", `expected the configured command verbatim, got ${JSON.stringify(out.payload)}`);
  assert(typeof out.payload.stdin === "string" && out.payload.stdin, `expected a prompt on stdin, got ${JSON.stringify(out.payload)}`);
  assert(out.economics.channel === "cli-exec" && out.economics.enforcement === "cli-command", `expected cli-exec/cli-command economics, got ${JSON.stringify(out.economics)}`);
  assert(out.economics.effective_model_status === "unverified", `expected unverified effective_model_status, got ${JSON.stringify(out.economics)}`);
  assert(out.economics.requested_model === null, `a cli command names its own model — requested_model must be null, got ${JSON.stringify(out.economics)}`);
});

// ─── codex channel/enforcement economics: prompt-budget + inherited-or-unknown,
// never a structural model field on the spawn_agent payload ────────────────

await check("codex channel economics: prompt-budget enforcement, inherited-or-unknown status, no model field on the payload", async () => {
  const root = mkFixture("dispatch-prepare-codex-economics-");
  writeConfig(root, { codex: { extraction: "gpt-5.5", generation: "gpt-5.5", review: null } });

  const out = await prepareOk(["--runtime", "codex", "--kind", "gather"], root);
  assert(!("model" in out.payload), `codex spawn_agent has no per-agent model field, got ${JSON.stringify(out.payload)}`);
  assert(out.economics.channel === "codex-native", `expected channel codex-native, got ${JSON.stringify(out.economics)}`);
  assert(out.economics.enforcement === "prompt-budget", `expected enforcement prompt-budget, got ${JSON.stringify(out.economics)}`);
  assert(out.economics.effective_model_status === "inherited-or-unknown", `expected inherited-or-unknown, got ${JSON.stringify(out.economics)}`);
  assert(out.economics.requested_model === "gpt-5.5", `expected requested_model gpt-5.5 (informational even though unenforced), got ${JSON.stringify(out.economics)}`);
  assert(out.economics.effective_model === null, `effective_model must always be null at prepare time, got ${JSON.stringify(out.economics)}`);
});

// ─── prepare-time dispatch record: written with economics + dispatch_id ───

await check("prepare-time dispatch record is appended to .bee/logs/dispatch.jsonl with economics + dispatch_id", async () => {
  const root = mkFixture("dispatch-prepare-record-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus" } });

  const out = await prepareOk(["--runtime", "claude", "--kind", "gather"], root);
  const line = readLastJsonl(path.join(root, ".bee", "logs", "dispatch.jsonl"));
  assert(line, "expected a line in .bee/logs/dispatch.jsonl");
  assert(line.source === "prepare", `expected source:prepare, got ${JSON.stringify(line)}`);
  assert(line.dispatch_id === out.dispatch_id, `expected the record's dispatch_id to match the returned one, got ${JSON.stringify(line)}`);
  assert(line.kind === "gather" && line.runtime === "claude", `expected kind/runtime recorded, got ${JSON.stringify(line)}`);
  assert(line.logical_tier === out.economics.logical_tier, `expected logical_tier to match economics, got ${JSON.stringify(line)}`);
  assert(line.requested_model === out.economics.requested_model, `expected requested_model to match economics, got ${JSON.stringify(line)}`);
  assert(line.channel === out.economics.channel && line.enforcement === out.economics.enforcement, `expected channel/enforcement to match economics, got ${JSON.stringify(line)}`);
  assert(line.cell === null, `a gather dispatch carries no cell id, got ${JSON.stringify(line)}`);
});

await check("prepare-time dispatch record for kind cell carries the cell id", async () => {
  const root = mkFixture("dispatch-prepare-record-cell-");
  writeConfig(root, { claude: { extraction: "haiku", generation: "sonnet", review: "opus" } });
  writeCellFixture(root, {
    id: "demo-9",
    feature: "demo",
    title: "Demo cell",
    action: "Do the demo thing.",
    verify: 'node -e "process.exit(0)"',
    status: "claimed",
    trace: { worker: "exec-demo-9" },
  });

  await prepareOk(["--runtime", "claude", "--kind", "cell", "--cell", "demo-9", "--worker", "exec-demo-9"], root);
  const line = readLastJsonl(path.join(root, ".bee", "logs", "dispatch.jsonl"));
  assert(line && line.cell === "demo-9", `expected cell demo-9 recorded, got ${JSON.stringify(line)}`);
});

// ─── native-transport (codex-native-transport D1/D3/D7, R1/R3/R5) ─────────
// prepareDispatch's `classification` param is caller-supplied (see its
// docstring) — these tests call it directly rather than through the real
// CLI's readNativeTransportClassification probe, so a CONFIRMED
// native_model_override golden row is deterministic on every host,
// regardless of whether the locally installed codex-cli actually ships
// multi_agent_v2 (it does not, anywhere, per E3 as of 0.144.4).

function nativeConfig(advisorSlot) {
  return { codex: { extraction: null, generation: null, review: null, advisor: advisorSlot } };
}

await check("R1 golden row: a confirmed native_model_override advisor payload round-trips ALLOW through the real evaluateDispatch", async () => {
  const root = mkFixture("dispatch-prepare-native-golden-");
  writeConfig(root, nativeConfig({ kind: "native", model: "gpt-5.5-high", effort: "high" }));

  const out = prepareDispatch(root, { runtime: "codex", kind: "advisor", classification: NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE });
  assert(out.tool === "spawn_agent", `expected tool spawn_agent, got ${JSON.stringify(out)}`);
  assert(out.payload.agent_type === "worker", `expected agent_type worker, got ${JSON.stringify(out.payload)}`);
  assert(out.payload.model === "gpt-5.5-high", `expected model gpt-5.5-high on the payload, got ${JSON.stringify(out.payload)}`);
  assert(out.payload.reasoning_effort === "high", `expected reasoning_effort high, got ${JSON.stringify(out.payload)}`);
  assert(out.payload.fork_turns === "none", `expected fork_turns "none" (E2 validity precondition), got ${JSON.stringify(out.payload)}`);
  assert(
    /^\[bee-tier: advisor\]/.test(out.payload.message),
    `expected message to open with the advisor marker, got ${JSON.stringify(out.payload.message)}`,
  );
  assert(out.transport === "native-override", `expected transport native-override, got ${JSON.stringify(out)}`);

  const allowed = evaluateDispatch("spawn_agent", out.payload, root);
  assert(allowed.decision === "allow", `expected ALLOW for the confirmed native advisor payload, got ${JSON.stringify(allowed)}`);
  assert(allowed.transport === "codex-spawn-marker" && allowed.tier === "advisor", `expected codex-spawn-marker/advisor, got ${JSON.stringify(allowed)}`);

  const stripped = { ...out.payload, message: out.payload.message.replace(/^\[bee-tier: advisor\]\n/, "") };
  const denied = evaluateDispatch("spawn_agent", stripped, root);
  assert(denied.decision === "deny" && denied.transport === "codex-spawn-unmarked", `expected DENY once the marker is stripped, got ${JSON.stringify(denied)}`);
});

await check("native economics: confirmed override reports native-requested status, effective_model null, requested_model the native model", async () => {
  const root = mkFixture("dispatch-prepare-native-economics-");
  writeConfig(root, nativeConfig({ kind: "native", model: "gpt-5.5-high" }));

  const out = prepareDispatch(root, { runtime: "codex", kind: "advisor", classification: NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE });
  assert(out.economics.channel === "codex-native", `expected channel codex-native, got ${JSON.stringify(out.economics)}`);
  assert(out.economics.enforcement === "native-model-param", `expected enforcement native-model-param, got ${JSON.stringify(out.economics)}`);
  assert(out.economics.effective_model_status === "native-requested", `expected native-requested status, got ${JSON.stringify(out.economics)}`);
  assert(out.economics.effective_model === null, `effective_model must stay null (D7: catalog-accepted is not runtime-confirmed), got ${JSON.stringify(out.economics)}`);
  assert(out.economics.requested_model === "gpt-5.5-high", `expected requested_model gpt-5.5-high, got ${JSON.stringify(out.economics)}`);
});

await check("D1: native route not confirmed, with an explicit-only cli fallback configured -> Bash payload + recorded reason, never silent", async () => {
  const root = mkFixture("dispatch-prepare-native-fallback-");
  writeConfig(
    root,
    nativeConfig({
      primary: { kind: "native", model: "gpt-5.5-high" },
      fallback: { kind: "cli", command: "codex exec -m gpt-5.5 -s read-only -" },
      fallback_policy: "explicit-only",
    }),
  );

  const out = prepareDispatch(root, { runtime: "codex", kind: "advisor", classification: NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY });
  assert(out.tool === "Bash", `expected tool Bash (the explicit fallback), got ${JSON.stringify(out)}`);
  assert(out.payload.command === "codex exec -m gpt-5.5 -s read-only -", `expected the configured fallback command verbatim, got ${JSON.stringify(out.payload)}`);
  assert(out.fallback_reason === "native_unavailable", `expected a recorded fallback_reason, got ${JSON.stringify(out)}`);
  assert(out.economics.channel === "cli-exec", `expected channel cli-exec, got ${JSON.stringify(out.economics)}`);

  const line = readLastJsonl(path.join(root, ".bee", "logs", "dispatch.jsonl"));
  assert(line.native_fallback_reason === "native_unavailable", `expected the fallback reason recorded in the dispatch log, got ${JSON.stringify(line)}`);
  assert(line.native_classification === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY, `expected the blocking classification recorded, got ${JSON.stringify(line)}`);
});

await check("D1: native route not confirmed, no fallback configured -> typed native_unavailable refusal naming the classification, never a payload", async () => {
  const root = mkFixture("dispatch-prepare-native-refused-");
  writeConfig(root, nativeConfig({ kind: "native", model: "gpt-5.5-high" }));

  const out = prepareDispatch(root, { runtime: "codex", kind: "advisor", classification: NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY });
  assert(out.ok === false && out.type === "refused" && out.reason === "native_unavailable", `expected the typed native_unavailable refusal, got ${JSON.stringify(out)}`);
  assert(out.detail === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY, `expected detail to name the blocking classification, got ${JSON.stringify(out)}`);
  assert(!("tool" in out) && !("payload" in out), `a refusal must never carry a payload, got ${JSON.stringify(out)}`);
});

await check("D3a coupling: external_cli_only classification on a bare native slot (no fallback) also refuses — never an invented cli command, never a silent budget fallback", async () => {
  const root = mkFixture("dispatch-prepare-native-external-cli-only-");
  writeConfig(root, nativeConfig({ kind: "native", model: "gpt-5.5-high" }));

  const out = prepareDispatch(root, { runtime: "codex", kind: "advisor", classification: NATIVE_TRANSPORT_EXTERNAL_CLI_ONLY });
  assert(out.ok === false && out.reason === "native_unavailable" && out.detail === NATIVE_TRANSPORT_EXTERNAL_CLI_ONLY, `expected a typed refusal naming external_cli_only, got ${JSON.stringify(out)}`);
});

await check("a composite native slot whose fallback_policy is NOT explicit-only never silently falls back to cli — refuses instead", async () => {
  const root = mkFixture("dispatch-prepare-native-no-explicit-policy-");
  writeConfig(
    root,
    nativeConfig({
      primary: { kind: "native", model: "gpt-5.5-high" },
      fallback: { kind: "cli", command: "codex exec -m gpt-5.5 -s read-only -" },
      // fallback_policy intentionally omitted — normalizeTierValue (state.mjs)
      // strips the fallback leg entirely unless it is exactly "explicit-only".
    }),
  );

  const out = prepareDispatch(root, { runtime: "codex", kind: "advisor", classification: NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY });
  assert(out.ok === false && out.reason === "native_unavailable", `expected a typed refusal (no implicit fallback), got ${JSON.stringify(out)}`);
});

await check("native routing is inert when classification is omitted — every non-native caller (incl. every earlier test in this file) stays byte-identical", async () => {
  const root = mkFixture("dispatch-prepare-native-omitted-classification-");
  writeConfig(root, { codex: { extraction: "gpt-5.5", generation: "gpt-5.5", review: null } });

  const out = prepareDispatch(root, { runtime: "codex", kind: "gather" });
  assert(out.tool === "spawn_agent" && !("model" in out.payload), `expected the ordinary budget-only spawn_agent payload, got ${JSON.stringify(out)}`);
  assert(out.economics.effective_model_status === "inherited-or-unknown", `expected inherited-or-unknown (no native slot configured at all), got ${JSON.stringify(out.economics)}`);
  assert(!("transport" in out) && !("fallback_reason" in out), `a non-native payload must never carry the native-only envelope extras, got ${JSON.stringify(out)}`);
});

await check("end-to-end via the real CLI: a native-configured advisor slot with no probe file on disk refuses through the real readNativeTransportClassification wiring", async () => {
  const root = mkFixture("dispatch-prepare-native-e2e-");
  writeConfig(root, nativeConfig({ kind: "native", model: "gpt-5.5-high" }));
  // No .bee/native-transport-probe.json written — readNativeTransportClassification
  // must fall back to native_budget_only (reason: no_probe_record, D3), and
  // handleDispatchPrepare (bee.mjs) must pass that through to prepareDispatch
  // exactly as this test's direct-call siblings above assert.

  const result = await runBee(["dispatch", "prepare", "--runtime", "codex", "--kind", "advisor", "--json"], root);
  assert(result.status === 0, `expected exit 0 (a typed refusal is not a crash), got ${result.status}: ${result.stderr}`);
  const out = JSON.parse(result.stdout);
  assert(out.ok === false && out.reason === "native_unavailable", `expected the typed native_unavailable refusal from the real CLI wiring, got ${JSON.stringify(out)}`);
  assert(out.detail === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY, `expected detail native_budget_only (unprobed host, D3), got ${JSON.stringify(out)}`);
});

// ─── bad --runtime / --kind refuse loudly ──────────────────────────────────

await check("an unknown --runtime is refused (non-zero exit)", async () => {
  const root = mkFixture("dispatch-prepare-bad-runtime-");
  const result = await runBee(["dispatch", "prepare", "--runtime", "banana", "--kind", "gather", "--json"], root);
  assert(result.status !== 0, `expected non-zero exit, got ${result.status}`);
});

await check("an unknown --kind is refused (non-zero exit)", async () => {
  const root = mkFixture("dispatch-prepare-bad-kind-");
  const result = await runBee(["dispatch", "prepare", "--runtime", "claude", "--kind", "banana", "--json"], root);
  assert(result.status !== 0, `expected non-zero exit, got ${result.status}`);
});

process.stdout.write(`\n${failed === 0 ? "ALL PASS" : `${failed} FAILURE(S)`} (${passed} passed, ${failed} failed)\n`);
process.exitCode = failed === 0 ? 0 : 1;
