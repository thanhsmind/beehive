#!/usr/bin/env node
// test_conformance.mjs — cnr2-14 (CONTEXT.md D12): black-box conformance
// suite over the AUTOMATABLE subset of docs/REFs/be-codex.md's P2 12-scenario
// list. Every scenario below subprocess-drives a real PUBLIC ENTRYPOINT
// (.bee/bin/bee.mjs, .bee/bin/hooks/*.mjs — the exact binaries wired into
// .codex/hooks.json / .claude/settings.json) against an isolated fixture
// store built fresh per scenario. NEVER touches this repo's own .bee/ state,
// NEVER fabricates a result, and asserts the NEGATIVE state too — that the
// refused action changed nothing — not merely that a deny code came back.
//
// Interactive / agent-behavior scenarios (1 tiny-no-ceremony, 2 standard
// Gates 1-3, 4 worker-single-cell-selection, 5-part-b the worker's own
// [BLOCKED] response, 7 package-install checkpoint, 8 subagent timeout /no
// duplicate dispatch, 9 compaction handoff, 10 feature-finish no auto-review,
// 11 review fan-out) are judgment calls a human or a live agent session
// makes, not something a fixture harness can force deterministically — they
// live in the manual checklist (docs/history/codex-native-runtime-v2/
// conformance-checklist.md) with the metric each feeds, never faked here.
//
// The matcher/spawn-guard checks below are labeled ADAPTER REGRESSIONS on
// purpose: they pin D4 (state-sync matcher superset) and D1/D4 (codex
// spawn_agent guard) capability decisions already implemented by earlier
// cnr2 cells — they are not one of the 12 numbered P2 scenarios.
//
// Helper policy (advisor finding 4, cell action text): existing test files
// (hooks/test_write_guard.mjs, hooks/test_model_guard.mjs,
// skills/bee-hive/templates/tests/test_bee_cli.mjs) export nothing, so
// fixture builders below are narrow, deliberate duplication of theirs (mkFixture,
// copyLib, buildDoctorFixture) — never an import of those top-level runners,
// which would execute them as a side effect.
//
// Exits 1 on any FAIL.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";

import { runModuleWorker } from "../lib/run-module-worker.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");
const BEE_MJS = path.join(REPO_ROOT, ".bee", "bin", "bee.mjs");
const WRITE_GUARD = path.join(REPO_ROOT, ".bee", "bin", "hooks", "bee-write-guard.mjs");
const MODEL_GUARD = path.join(REPO_ROOT, ".bee", "bin", "hooks", "bee-model-guard.mjs");
const REAL_LIB_DIR = path.join(REPO_ROOT, ".bee", "bin", "lib");

let failures = 0;

function record(id, description, passed, detail = "") {
  const label = passed ? "PASS" : "FAIL";
  if (!passed) failures += 1;
  console.log(`${label}  [${id}] ${description}${passed ? "" : ` :: ${detail}`}`);
}

// ─── shared fixture builders ────────────────────────────────────────────────
// Narrow duplication of hooks/test_write_guard.mjs / hooks/test_model_guard.mjs's
// own mkFixture/copyLib (both files independently carry the identical
// helper already — this is a third, equally narrow copy, not a new pattern).

function mkFixture(prefix) {
  return fs.mkdtempSync(path.join(os.tmpdir(), prefix));
}

function copyLib(fixtureRoot) {
  const libDir = path.join(fixtureRoot, ".bee", "bin", "lib");
  fs.mkdirSync(libDir, { recursive: true });
  for (const name of fs.readdirSync(REAL_LIB_DIR)) {
    if (!name.endsWith(".mjs")) continue;
    fs.copyFileSync(path.join(REAL_LIB_DIR, name), path.join(libDir, name));
  }
}

function writeStateFile(root, state) {
  fs.writeFileSync(path.join(root, ".bee", "state.json"), `${JSON.stringify(state, null, 2)}\n`);
}

// A store fixture usable by both bee.mjs (subprocess dispatcher) and the hook
// binaries (which dynamically import THIS root's own .bee/bin/lib copy).
function buildStoreFixture(prefix, { phase = "swarming", executionApproved = true, feature = "conform-demo" } = {}) {
  const root = mkFixture(prefix);
  fs.mkdirSync(path.join(root, ".bee"), { recursive: true });
  fs.writeFileSync(path.join(root, ".bee", "onboarding.json"), "{}\n");
  copyLib(root);
  writeStateFile(root, {
    phase,
    mode: "standard",
    feature,
    approved_gates: { context: true, shape: true, execution: executionApproved, review: false },
  });
  return root;
}

async function runBee(cwd, args, input) {
  return runModuleWorker(BEE_MJS, { args, cwd, input });
}

async function runHook(hookPath, payload, cwd) {
  return runModuleWorker(hookPath, { input: JSON.stringify(payload), cwd });
}

function rm(root) {
  fs.rmSync(root, { recursive: true, force: true });
}

// ═════════════════════════════════════════════════════════════════════════
// Scenario 3 — source write before Gate 3 -> write-guard binary denies
// (fixture payload; no live-repo mutation; negative-state: nothing written)
// ═════════════════════════════════════════════════════════════════════════

async function scenario3() {
  const root = buildStoreFixture("bee-conformance-s3-", { phase: "validating", executionApproved: false });
  const target = "src/new-feature.js";
  const res = await runHook(WRITE_GUARD, { tool_name: "Edit", tool_input: { file_path: target } }, root);
  const denied = res.status === 2 && /bee gate/.test(res.stderr);
  const nothingWritten = !fs.existsSync(path.join(root, target));
  // Control row: the same gated phase still allows a docs/ target — proves
  // the deny is gate policy, not a broad write block (negative control).
  const control = await runHook(WRITE_GUARD, { tool_name: "Edit", tool_input: { file_path: "docs/notes.md" } }, root);
  record(
    "scenario-3",
    "a source write before Gate 3 is denied by the write-guard binary (public entrypoint) and nothing was written; a docs/ write in the same gated phase still passes (control)",
    denied && nothingWritten && control.status === 0,
    `deny: status=${res.status} stderr=${res.stderr}; nothingWritten=${nothingWritten}; control status=${control.status} stderr=${control.stderr}`,
  );
  rm(root);
}

// ═════════════════════════════════════════════════════════════════════════
// Scenario 6 — verify-red never caps -> cells cap refusal on a failed verify
// record, in a fixture store (public entrypoint: bee.mjs cells verify / cap)
// ═════════════════════════════════════════════════════════════════════════

async function scenario6() {
  const root = buildStoreFixture("bee-conformance-s6-");
  fs.writeFileSync(
    path.join(root, "cell.json"),
    JSON.stringify({
      id: "s6-1",
      feature: "conform-demo",
      title: "fixture cell for scenario 6 (verify-red never caps)",
      lane: "small",
      action: "Fixture only — never actually run outside this suite.",
      verify: 'node -e "process.exit(1)"',
    }),
  );
  const added = await runBee(root, ["cells", "add", "--file", "cell.json", "--json"]);
  const claimed = await runBee(root, ["cells", "claim", "--id", "s6-1", "--worker", "kevinb", "--json"]);
  const verified = await runBee(root, [
    "cells", "verify", "--id", "s6-1",
    "--command", 'node -e "process.exit(1)"',
    "--output", "exit 1 (red)", "--passed", "false", "--json",
  ]);
  const capAttempt = await runBee(root, [
    "cells", "cap", "--id", "s6-1", "--outcome", "must never cap", "--files", "a.js", "--json",
  ]);
  const refused = capAttempt.status === 1 && /no passing verify result/.test(capAttempt.stdout);
  const shown = await runBee(root, ["cells", "show", "--id", "s6-1", "--json"]);
  const shownParsed = JSON.parse(shown.stdout);
  const nothingCapped = shownParsed.trace.capped_at === null && shownParsed.status !== "capped";
  record(
    "scenario-6",
    "cells cap refuses on a recorded failed (red) verify via the public entrypoint, and the cell stays uncapped",
    added.status === 0 && claimed.status === 0 && verified.status === 0 && refused && nothingCapped,
    `add=${added.status} claim=${claimed.status} verify=${verified.status} cap: status=${capAttempt.status} stdout=${capAttempt.stdout} trace=${JSON.stringify(shownParsed.trace)}`,
  );
  rm(root);
}

// ═════════════════════════════════════════════════════════════════════════
// Scenario 5-part-a — reservation conflict: reserve refusal names the holder
// (mechanical half; the worker-side [BLOCKED] response is agent behavior ->
// manual checklist item, advisor finding 3)
// ═════════════════════════════════════════════════════════════════════════

async function scenario5a() {
  const root = buildStoreFixture("bee-conformance-s5a-");
  const first = await runBee(root, ["reservations", "reserve", "--agent", "otto", "--cell", "demo-1", "--path", "src/shared.js", "--json"]);
  const second = await runBee(root, ["reservations", "reserve", "--agent", "mel", "--cell", "demo-2", "--path", "src/shared.js", "--json"]);
  const secondParsed = second.status === 0 ? null : JSON.parse(second.stdout);
  const namesHolder =
    second.status === 1 &&
    secondParsed &&
    secondParsed.ok === false &&
    Array.isArray(secondParsed.conflicts) &&
    secondParsed.conflicts.some((c) => c.agent === "otto" && c.path === "src/shared.js");
  const listResult = await runBee(root, ["reservations", "list", "--active-only", "--json"]);
  const reservations = JSON.parse(listResult.stdout).reservations || [];
  const otto = reservations.filter((r) => r.agent === "otto" && r.path === "src/shared.js");
  const mel = reservations.filter((r) => r.agent === "mel" && r.path === "src/shared.js");
  const nothingGrantedToLoser = otto.length === 1 && mel.length === 0;
  record(
    "scenario-5-part-a",
    "a reservation conflict on the public entrypoint returns ok:false naming the holder, and no second reservation is granted for the loser",
    first.status === 0 && namesHolder && nothingGrantedToLoser,
    `first=${first.status} second: status=${second.status} stdout=${second.stdout} active=${JSON.stringify(reservations)}`,
  );
  rm(root);
}

// ═════════════════════════════════════════════════════════════════════════
// Scenario 12 — doctor fail-closed: a drifted fixture never reaches "ready"
// (cnr2-13 doctor fixtures; D11 — file presence is never capability). g22-3
// D4 three-state update: the CLEAN baseline (mechanical rows all ok, trust
// rows structurally unknown, no attestation recorded) now reads 'degraded',
// not the old bare 'not_ready' — it is recoverable via `doctor attest`,
// never a mechanical failure. The DRIFTED case (capability-baseline byte
// mismatch, a MECHANICAL fact per D4) now reads 'blocked' — strictly worse
// than 'degraded', since nothing is provable once the baseline itself is
// wrong. Neither ever reaches 'ready' from file presence alone (D11 intact).
// ═════════════════════════════════════════════════════════════════════════

const DOCTOR_HOOKS_JSON = {
  hooks: {
    PreToolUse: [
      {
        matcher: "spawn_agent",
        hooks: [{ type: "command", command: 'exec node "$r"/.bee/bin/hooks/bee-model-guard.mjs --source=repo' }],
      },
    ],
    Stop: [{ hooks: [{ type: "command", command: 'exec node "$r"/.bee/bin/hooks/bee-state-sync.mjs --source=repo' }] }],
  },
};

async function scenario12() {
  const dir = mkFixture("bee-conformance-s12-");
  try {
    fs.mkdirSync(path.join(dir, ".bee"), { recursive: true });
    fs.mkdirSync(path.join(dir, ".codex"), { recursive: true });
    fs.mkdirSync(path.join(dir, "hooks"), { recursive: true });
    const hooksJsonPath = path.join(dir, ".codex", "hooks.json");
    fs.writeFileSync(hooksJsonPath, `${JSON.stringify(DOCTOR_HOOKS_JSON, null, 2)}\n`, "utf8");
    fs.writeFileSync(path.join(dir, "hooks", "bee-model-guard.mjs"), "// stub\n", "utf8");
    fs.writeFileSync(path.join(dir, "hooks", "bee-state-sync.mjs"), "// stub\n", "utf8");
    fs.writeFileSync(path.join(dir, ".codex", "config.toml"), 'approval_policy = "never"\n', "utf8");
    // skills_installed is one of D4's mechanical blocking rows, but this
    // scenario is testing the trust-row degrade, not a skills gap — give it
    // a (deliberately legacy v1) sidecar so the row reads 'warn'/non-blocking
    // (g22-4/D7) rather than pulling in the full deep-audit machinery that
    // scenario 14/15 below exercise directly.
    fs.mkdirSync(path.join(dir, ".agents", "skills"), { recursive: true });
    fs.writeFileSync(
      path.join(dir, ".agents", "skills", ".bee-render.json"),
      `${JSON.stringify({ schema: "bee-render/1", target_runtime: "codex" }, null, 2)}\n`,
      "utf8",
    );

    const crypto = await import("node:crypto");
    const baselineHash = crypto.createHash("sha256").update(fs.readFileSync(hooksJsonPath)).digest("hex");
    fs.writeFileSync(
      path.join(dir, ".bee", "onboarding.json"),
      `${JSON.stringify(
        {
          schema_version: "1.0",
          bee_version: "0.1.0",
          managed: { repo_hooks: { ".codex/hooks.json": baselineHash } },
          agents_sync: { files: [] },
        },
        null,
        2,
      )}\n`,
      "utf8",
    );

    // Baseline (clean, unmutated) fixture: mechanical rows are all ok, but
    // codex's structurally-unknown trust/discovery rows still hold it at
    // 'degraded' with no attestation recorded (D4/D11 — never "ready" from
    // file presence alone; degraded is recoverable via `doctor attest`).
    const clean = await runBee(dir, ["doctor", "--runtime", "codex", "--json"]);
    const cleanParsed = clean.status === 0 ? JSON.parse(clean.stdout) : null;
    const cleanDegraded = clean.status === 0 && cleanParsed.overall_status === "degraded";

    // Now drift the file AFTER the baseline hash was recorded — a real
    // post-onboarding drift, not a missing-baseline case. capability_
    // baseline_match is a MECHANICAL blocking row (D4), so this now reads
    // 'blocked' — strictly worse than the clean case's 'degraded'.
    fs.writeFileSync(hooksJsonPath, `${JSON.stringify({ hooks: { Stop: [] } }, null, 2)}\n`, "utf8");
    const drifted = await runBee(dir, ["doctor", "--runtime", "codex", "--json"]);
    const driftedParsed = drifted.status === 0 ? JSON.parse(drifted.stdout) : null;
    const driftedBlocked = drifted.status === 0 && driftedParsed && driftedParsed.overall_status === "blocked";
    const driftRow = driftedParsed && driftedParsed.rows.find((r) => r.row === "capability_baseline_match");
    const driftWarned = driftRow && driftRow.status === "warn" && typeof driftRow.fix === "string" && driftRow.fix.length > 0;

    record(
      "scenario-12",
      "doctor --runtime codex fail-closes on a drifted fixture: overall_status never reaches ready on the clean baseline (degraded) or the drifted fixture (blocked), and the drifted row carries a FIX line",
      cleanDegraded && driftedBlocked && driftWarned,
      `clean: status=${clean.status} overall=${cleanParsed && cleanParsed.overall_status}; drifted: status=${drifted.status} overall=${driftedParsed && driftedParsed.overall_status} row=${JSON.stringify(driftRow)}`,
    );
  } finally {
    rm(dir);
  }
}

// ═════════════════════════════════════════════════════════════════════════
// Scenario 13 — doctor codex host topology (GH #22 P1-1): a NORMAL host
// install renders .codex/hooks.json command strings with the host prefix
// ("$r"/.bee/bin/hooks/<f>) and keeps handler files under .bee/bin/hooks/
// ONLY — no root hooks/ dir at all (unlike scenario 12 above, which mimics
// bee's own source-checkout topology and is left untouched). Before the
// fix, hook_handlers_resolvable checked a single hard-coded "hooks/" dir and
// reported every healthy hybrid host install as broken. Also proves the
// negative: deleting one handler file from .bee/bin/hooks/ still warns.
// ═════════════════════════════════════════════════════════════════════════

const DOCTOR_HOOKS_JSON_HOST_TOPOLOGY = {
  hooks: {
    PreToolUse: [
      {
        matcher: "spawn_agent",
        hooks: [{ type: "command", command: 'exec node "$r"/.bee/bin/hooks/bee-model-guard.mjs --source=repo' }],
      },
    ],
    Stop: [{ hooks: [{ type: "command", command: 'exec node "$r"/.bee/bin/hooks/bee-state-sync.mjs --source=repo' }] }],
  },
};

async function scenario13() {
  const dir = mkFixture("bee-conformance-s13-");
  try {
    fs.mkdirSync(path.join(dir, ".bee", "bin", "hooks"), { recursive: true });
    fs.mkdirSync(path.join(dir, ".codex"), { recursive: true });
    const hooksJsonPath = path.join(dir, ".codex", "hooks.json");
    fs.writeFileSync(hooksJsonPath, `${JSON.stringify(DOCTOR_HOOKS_JSON_HOST_TOPOLOGY, null, 2)}\n`, "utf8");
    fs.writeFileSync(path.join(dir, ".bee", "bin", "hooks", "bee-model-guard.mjs"), "// stub\n", "utf8");
    fs.writeFileSync(path.join(dir, ".bee", "bin", "hooks", "bee-state-sync.mjs"), "// stub\n", "utf8");
    // bee.mjs refuses to run without a discoverable repo root
    // (.bee/onboarding.json or .git up the tree) — this fixture lives under
    // os.tmpdir(), which has neither on its own.
    fs.writeFileSync(
      path.join(dir, ".bee", "onboarding.json"),
      `${JSON.stringify({ schema_version: "1.0", bee_version: "0.1.0", managed: {}, agents_sync: { files: [] } }, null, 2)}\n`,
      "utf8",
    );

    // No root hooks/ dir anywhere — this is the defining trait of a normal
    // host install (repoOwnsHookCatalog must read false).
    const rootHooksAbsent = !fs.existsSync(path.join(dir, "hooks"));

    const ok = await runBee(dir, ["doctor", "--runtime", "codex", "--json"]);
    const okParsed = ok.status === 0 ? JSON.parse(ok.stdout) : null;
    const okRow = okParsed && okParsed.rows.find((r) => r.row === "hook_handlers_resolvable");
    const okResolved =
      okRow &&
      okRow.status === "ok" &&
      typeof okRow.evidence === "string" &&
      okRow.evidence.includes(".bee/bin/hooks/") &&
      !okRow.evidence.includes("source-checkout");

    // Negative: delete one handler file from .bee/bin/hooks/ (its only
    // location in this topology) — must warn and name the missing file.
    fs.rmSync(path.join(dir, ".bee", "bin", "hooks", "bee-state-sync.mjs"));
    const missing = await runBee(dir, ["doctor", "--runtime", "codex", "--json"]);
    const missingParsed = missing.status === 0 ? JSON.parse(missing.stdout) : null;
    const missingRow = missingParsed && missingParsed.rows.find((r) => r.row === "hook_handlers_resolvable");
    const missingWarned =
      missingRow && missingRow.status === "warn" && missingRow.evidence.includes("bee-state-sync.mjs");

    record(
      "scenario-13",
      "doctor --runtime codex resolves hook handlers at host topology (.bee/bin/hooks, no root hooks/ dir), naming the resolved location, and still warns with the missing filename when a handler is deleted",
      rootHooksAbsent && okResolved && missingWarned,
      `rootHooksAbsent=${rootHooksAbsent}; ok: status=${ok.status} row=${JSON.stringify(okRow)}; missing: status=${missing.status} row=${JSON.stringify(missingRow)}`,
    );
  } finally {
    rm(dir);
  }
}

// ═════════════════════════════════════════════════════════════════════════
// ADAPTER REGRESSION — update_plan present in every rendered state-sync
// matcher (D4: superset, never a swap). Checked against THIS repo's own
// rendered targets, not a fixture — these are read-only inspections of
// tracked files, never a mutation.
// ═════════════════════════════════════════════════════════════════════════

function findStateSyncMatcher(hooksBlock) {
  const postToolUse = (hooksBlock && hooksBlock.PostToolUse) || [];
  for (const entry of postToolUse) {
    const names = (entry.hooks || []).map((h) => String(h.command || ""));
    if (names.some((c) => c.includes("bee-state-sync.mjs")) && typeof entry.matcher === "string") {
      return entry.matcher;
    }
  }
  return null;
}

function adapterRegressionMatcherSuperset() {
  const targets = [
    path.join(REPO_ROOT, "hooks", "hooks.json"),
    path.join(REPO_ROOT, "hooks", "claude-hooks.json"),
    path.join(REPO_ROOT, ".codex", "hooks.json"),
  ];
  const detail = [];
  let allOk = true;
  for (const file of targets) {
    let matcher = null;
    try {
      const parsed = JSON.parse(fs.readFileSync(file, "utf8"));
      matcher = findStateSyncMatcher(parsed.hooks || parsed);
    } catch (error) {
      matcher = null;
      detail.push(`${file}: unreadable (${error.message})`);
      allOk = false;
      continue;
    }
    const ok = typeof matcher === "string" && matcher.includes("update_plan");
    if (!ok) allOk = false;
    detail.push(`${path.relative(REPO_ROOT, file)}: matcher=${matcher}`);
  }
  // .claude/settings.json embeds the same hooks block under a top-level
  // "hooks" key rather than the plugin-manifest shape — read separately.
  const claudeSettings = path.join(REPO_ROOT, ".claude", "settings.json");
  try {
    const parsed = JSON.parse(fs.readFileSync(claudeSettings, "utf8"));
    const matcher = findStateSyncMatcher(parsed.hooks || {});
    const ok = typeof matcher === "string" && matcher.includes("update_plan");
    if (!ok) allOk = false;
    detail.push(`${path.relative(REPO_ROOT, claudeSettings)}: matcher=${matcher}`);
  } catch (error) {
    allOk = false;
    detail.push(`${claudeSettings}: unreadable (${error.message})`);
  }
  record(
    "adapter-regression-matcher-superset",
    "update_plan is present in the state-sync PostToolUse matcher of every rendered hook target (hooks/hooks.json, hooks/claude-hooks.json, .codex/hooks.json, .claude/settings.json)",
    allOk,
    detail.join(" | "),
  );
}

// ═════════════════════════════════════════════════════════════════════════
// ADAPTER REGRESSION — codex spawn guard: bare-denied / anchored-allowed /
// unobserved-fail-open (D1/D4 capability-matrix row D1; codex-cli 0.144.4
// spike). Subprocess-drives the real hooks/bee-model-guard.mjs binary.
// ═════════════════════════════════════════════════════════════════════════

async function adapterRegressionSpawnGuard() {
  const root = mkFixture("bee-conformance-spawnguard-");
  fs.mkdirSync(path.join(root, ".bee"), { recursive: true });
  fs.writeFileSync(path.join(root, ".bee", "onboarding.json"), "{}\n");
  copyLib(root);

  const bareDenied = await runHook(
    MODEL_GUARD,
    { tool_name: "spawn_agent", tool_input: { agent_type: "worker", message: "no marker here at all" } },
    root,
  );
  const bareDeniedOk = bareDenied.status === 2 && /spawn_agent/.test(bareDenied.stderr);

  const anchoredAllowed = await runHook(
    MODEL_GUARD,
    { tool_name: "spawn_agent", tool_input: { agent_type: "worker", message: "[bee-tier: generation] gather the callers" } },
    root,
  );
  const anchoredAllowedOk = anchoredAllowed.status === 0;

  // i54-closeout-1 (D1) widened evaluateCodexSpawn to judge every spawn_agent
  // payload that carries a `message` string, regardless of the agent_type/
  // task_name field — coverage only widens, per the locked decision, so an
  // `agent_type`-shaped payload WITH a message is now an OBSERVED shape and
  // correctly denies when unmarked (this is not the fail-open case anymore).
  // The genuinely unobserved shape under the current guard is a spawn_agent
  // call with no usable `message` at all (evaluateCodexSpawn's own noOpinion
  // branch, dispatch-guard.mjs:212-214) — that is what this fixture must
  // exercise to test the real fail-open path.
  const unobservedFailOpen = await runHook(
    MODEL_GUARD,
    { tool_name: "spawn_agent", tool_input: { agent_type: "default" } },
    root,
  );
  const unobservedFailOpenOk = unobservedFailOpen.status === 0;

  record(
    "adapter-regression-spawn-guard",
    "codex spawn_agent guard: unmarked worker spawn denies (bare-denied), an anchored [bee-tier:] marker allows (anchored-allowed), and an unobserved agent_type shape fails open (unobserved-fail-open)",
    bareDeniedOk && anchoredAllowedOk && unobservedFailOpenOk,
    `bare: status=${bareDenied.status} stderr=${bareDenied.stderr}; anchored: status=${anchoredAllowed.status} stderr=${anchoredAllowed.stderr}; unobserved: status=${unobservedFailOpen.status} stderr=${unobservedFailOpen.stderr}`,
  );
  rm(root);
}

// ═════════════════════════════════════════════════════════════════════════
// Scenario 14 — doctor deep skill-inventory audit (g22-4, D7 / advisor R5): a
// bee-render/2 sidecar drives a DEEP audit (present + recomputed content-hash
// match) of every expected skill dir, not the old shallow "dir count +
// provenance present" check. One fixture sequence pins all three mechanical
// mismatch facts named by the evidence string: deleting an expected skill
// dir -> "missing: <name>"; an unexpected plain bee-* dir -> "stray: <name>";
// flipping a byte inside a rendered file, sidecar untouched -> "drifted:
// <name>". A clean install passes ('ok'); each mutation is restored before
// the next so failures never compound across cases.
// ═════════════════════════════════════════════════════════════════════════

function skillDirDigest(files) {
  // files: { relPath: utf8 content } for ONE skill dir — mirrors bee.mjs's
  // doctorWalkSkillDir / onboard_bee.mjs's skillDigest fold exactly: sha256
  // of each file's raw bytes, sorted by relPath, folded into one sha256 over
  // the sorted [relPath, hash] JSON array.
  const hashed = Object.entries(files)
    .map(([rel, content]) => [rel, createHash("sha256").update(Buffer.from(content, "utf8")).digest("hex")])
    .sort((a, b) => (a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0));
  return createHash("sha256").update(JSON.stringify(hashed)).digest("hex");
}

function writeSkillDir(skillsRoot, name, files) {
  for (const [rel, content] of Object.entries(files)) {
    const abs = path.join(skillsRoot, name, ...rel.split("/"));
    fs.mkdirSync(path.dirname(abs), { recursive: true });
    fs.writeFileSync(abs, content, "utf8");
  }
}

async function scenario14() {
  const dir = mkFixture("bee-conformance-s14-");
  try {
    fs.mkdirSync(path.join(dir, ".bee"), { recursive: true });
    fs.writeFileSync(
      path.join(dir, ".bee", "onboarding.json"),
      `${JSON.stringify({ schema_version: "1.0", bee_version: "0.1.0", managed: {}, agents_sync: { files: [] } }, null, 2)}\n`,
      "utf8",
    );
    const skillsRoot = path.join(dir, ".agents", "skills");
    const alpha = { "SKILL.md": "# alpha\n" };
    const beta = { "SKILL.md": "# beta\n", "references/notes.md": "beta notes\n" };
    writeSkillDir(skillsRoot, "bee-alpha", alpha);
    writeSkillDir(skillsRoot, "bee-beta", beta);
    const sidecar = {
      schema: "bee-render/2",
      target_runtime: "codex",
      skills: [
        { name: "bee-alpha", sha256: skillDirDigest(alpha) },
        { name: "bee-beta", sha256: skillDirDigest(beta) },
      ],
    };
    fs.writeFileSync(path.join(skillsRoot, ".bee-render.json"), `${JSON.stringify(sidecar, null, 2)}\n`, "utf8");

    async function skillsRow() {
      const res = await runBee(dir, ["doctor", "--runtime", "codex", "--json"]);
      const parsed = res.status === 0 ? JSON.parse(res.stdout) : null;
      return parsed && parsed.rows.find((r) => r.row === "skills_installed");
    }

    const passRow = await skillsRow();
    const passOk = passRow && passRow.status === "ok" && passRow.blocking === true;

    fs.rmSync(path.join(skillsRoot, "bee-beta"), { recursive: true, force: true });
    const missingRow = await skillsRow();
    const missingNamed = missingRow && missingRow.status === "warn" && /missing: bee-beta/.test(missingRow.evidence);
    writeSkillDir(skillsRoot, "bee-beta", beta); // restore before the next case

    writeSkillDir(skillsRoot, "bee-fake", { "SKILL.md": "# fake\n" });
    const strayRow = await skillsRow();
    const strayNamed = strayRow && strayRow.status === "warn" && /stray: bee-fake/.test(strayRow.evidence);
    fs.rmSync(path.join(skillsRoot, "bee-fake"), { recursive: true, force: true }); // restore

    fs.writeFileSync(path.join(skillsRoot, "bee-alpha", "SKILL.md"), "# ALPHA (one byte flipped)\n", "utf8");
    const driftRow = await skillsRow();
    const driftNamed = driftRow && driftRow.status === "warn" && /drifted: bee-alpha/.test(driftRow.evidence);
    fs.writeFileSync(path.join(skillsRoot, "bee-alpha", "SKILL.md"), "# alpha\n", "utf8"); // restore

    const restoredRow = await skillsRow();
    const restored = restoredRow && restoredRow.status === "ok";

    record(
      "scenario-14",
      "doctor's skills_installed deep-audits a bee-render/2 sidecar via the public entrypoint: a matching install passes, a deleted skill is named missing, an unexpected bee-* dir is named stray, and a one-byte content drift is named drifted",
      passOk && missingNamed && strayNamed && driftNamed && restored,
      `pass=${JSON.stringify(passRow)}; missing=${JSON.stringify(missingRow)}; stray=${JSON.stringify(strayRow)}; drift=${JSON.stringify(driftRow)}; restored=${JSON.stringify(restoredRow)}`,
    );
  } finally {
    rm(dir);
  }
}

// ═════════════════════════════════════════════════════════════════════════
// Scenario 15 — legacy bee-render/1 sidecar warns, never blocks (g22-4, D7):
// a pre-D7 install carrying only the shallow provenance stamp must stay
// usable — deep inventory is simply unavailable, which is a warn, not a
// mechanical failure. Distinct from scenario 14's v2 mismatches, which DO
// block (D4 three-state).
// ═════════════════════════════════════════════════════════════════════════

async function scenario15() {
  const dir = mkFixture("bee-conformance-s15-");
  try {
    fs.mkdirSync(path.join(dir, ".bee"), { recursive: true });
    fs.writeFileSync(
      path.join(dir, ".bee", "onboarding.json"),
      `${JSON.stringify({ schema_version: "1.0", bee_version: "0.1.0", managed: {}, agents_sync: { files: [] } }, null, 2)}\n`,
      "utf8",
    );
    const skillsRoot = path.join(dir, ".agents", "skills");
    writeSkillDir(skillsRoot, "bee-alpha", { "SKILL.md": "# alpha\n" });
    fs.writeFileSync(
      path.join(skillsRoot, ".bee-render.json"),
      `${JSON.stringify({ schema: "bee-render/1", target_runtime: "codex" }, null, 2)}\n`,
      "utf8",
    );
    const res = await runBee(dir, ["doctor", "--runtime", "codex", "--json"]);
    const parsed = res.status === 0 ? JSON.parse(res.stdout) : null;
    const row = parsed && parsed.rows.find((r) => r.row === "skills_installed");
    const warnsNotBlocks =
      row && row.status === "warn" && row.blocking === false && /bee-render\/1/.test(row.evidence);
    record(
      "scenario-15",
      "doctor's skills_installed on a legacy bee-render/1 sidecar warns (inventory unavailable, re-run onboarding/render to upgrade) but never blocks readiness",
      warnsNotBlocks,
      `status=${res.status} row=${JSON.stringify(row)}`,
    );
  } finally {
    rm(dir);
  }
}

// ═════════════════════════════════════════════════════════════════════════

async function main() {
  await scenario3();
  await scenario6();
  await scenario5a();
  await scenario12();
  await scenario13();
  await scenario14();
  await scenario15();
  adapterRegressionMatcherSuperset();
  await adapterRegressionSpawnGuard();

  const checklist = path.join(
    REPO_ROOT,
    "docs",
    "history",
    "codex-native-runtime-v2",
    "conformance-checklist.md",
  );
  const checklistExists = fs.existsSync(checklist) && fs.statSync(checklist).size > 0;
  record(
    "manual-checklist-present",
    "the manual checklist for the interactive/judgment scenarios exists and is non-empty",
    checklistExists,
    checklist,
  );

  console.log(`\n${failures === 0 ? "ALL PASS" : `${failures} FAILURE(S)`}`);
  process.exitCode = failures === 0 ? 0 : 1;
}

await main();
