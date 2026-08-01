#!/usr/bin/env node
// test_hook_contracts.mjs - RED-checkpoint harness (cell codex-parity-1,
// docs/history/codex-runtime-parity/plan.md test matrix row 2 "input
// extremes"; decision D2 in CONTEXT.md: "Codex receives full hook parity on
// every compatible event and tool path ... unsupported paths fail open with
// visible limits and runtime-specific tests").
//
// Executes EACH of the seven established production wrapper hooks
// (bee-session-init, bee-prompt-context, bee-state-sync, bee-chain-nudge,
// bee-session-close, bee-model-guard, bee-write-guard) through the shared
// isolated Worker runner - the same real direct-entry path as
// hooks/test_write_guard.mjs and hooks/test_model_guard.mjs - and feeds it a
// table of adversarial stdin
// rows: empty input, junk bytes, top-level null, JSON array, object cwd,
// missing cwd, a ~2MB payload, plus Codex event-output parse rows
// (PreCompact/SubagentStop/Stop advisory shape, PreToolUse apply_patch deny
// shape). Every row's `expect` function encodes the TARGET/desired contract
// (fail-open on malformed input; parseable JSON systemMessage advisories,
// never `decision:"block"`; apply_patch targets run the same gate/direct-
// edit/reservation decisions as Edit/Write/Bash).
//
// This cell fixes NOTHING (per D2 and learning 20260711-model-tier-guard:
// adversarial rows from the start, checkpointed before repair) - it only
// observes the CURRENT unmodified wrappers against that target contract.
//
// Three modes:
//   --baseline     : run every wrapper row against the current wrappers and
//                    record every failure verbatim into
//                    docs/history/codex-runtime-parity/reports/red-baseline.md.
//                    Exits 0 regardless of failures - the failures ARE the
//                    expected RED evidence being captured, not a harness bug.
//   --catalog-only : (cell codex-parity-2) run ONLY the catalog drift-check,
//                    allowed-differences, and isolated-CODEX_HOME
//                    codex-acceptance rows; exit non-zero if any fails. Never
//                    touches the seven-wrapper table.
//   (default)      : run every row (wrapper table + catalog + acceptance) and
//                    exit non-zero if any row violates its target contract.
//                    The wrapper table's green state is the contract cell
//                    codex-parity-3 must reach; until then default mode is
//                    expected to fail.
//
// Never edits hooks/*.mjs. Builds isolated tmp fixtures so no run ever
// touches this project's real .bee/state.json or .bee/logs/*.jsonl.

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { runModuleWorker } from "../../../scripts/lib/run-module-worker.mjs";
import { resolveRoots as resolveHookRoots, controlRootFor as hookControlRootFor } from "./adapter.mjs";
import {
  RUNTIMES,
  TARGETS,
  renderProjection,
  renderProjectionText,
  ALLOWED_DIFFERENCES,
  REPO_TRANSPORT_UNAVAILABLE_DIAGNOSTIC,
} from "./catalog.mjs";

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const HOOKS_DIR = path.dirname(SCRIPT_PATH);
const REPO_ROOT = path.dirname(path.dirname(path.dirname(HOOKS_DIR)));
const REAL_LIB_DIR = path.join(REPO_ROOT, ".bee", "bin", "lib");
const REPORT_PATH = path.join(
  REPO_ROOT,
  "docs",
  "history",
  "codex-runtime-parity",
  "reports",
  "red-baseline.md",
);

// cell codex-parity-2 (docs/history/codex-runtime-parity/plan.md, approach.md
// section 1): hooks/hooks.json is the CODEX default projection,
// hooks/claude-hooks.json is the explicit CLAUDE projection. Both are
// rendered from the single logical catalog in hooks/catalog.mjs.
const CODEX_DEFAULT_HOOKS_PATH = path.join(HOOKS_DIR, "hooks.json");
const CLAUDE_HOOKS_PATH = path.join(HOOKS_DIR, "claude-hooks.json");
const CODEX_PLUGIN_MANIFEST_PATH = path.join(REPO_ROOT, ".codex-plugin", "plugin.json");

// cell codex-parity-6a: the ACTIVE source-repository Codex fallback. Unlike
// the two plugin projections above it is rendered with target "repo".
const CODEX_REPO_HOOKS_PATH = path.join(REPO_ROOT, ".codex", "hooks.json");

// REQUIRED-ROW MANIFEST (cell codex-parity-6a).
//
// `failures = results.filter(r => !r.pass)` counts only rows that RAN and
// failed, while a skip row is constructed with `pass: true` — so a row that is
// ABSENT (never registered) or SKIPPED ("codex CLI unavailable") prints
// ALL PASS and exits 0. That is a hole big enough to cap a cell through
// having done nothing. These row ids MUST be present AND MUST NOT be skipped
// in the group they belong to, or the run exits non-zero.
const REQUIRED_CATALOG_ROW_IDS = Object.freeze([
  "codex-repo-target-generated-topology",
  "codex-generated-repo-audit-execution",
  "codex-repo-target-byte-identical",
  "codex-commandWindows-nested-cwd-execution",
]);

// Turn "required row is absent or skipped" into a real, visible FAILING row,
// so it flows through the ordinary failures filter and the ordinary output.
// `group` only labels the synthesized row; the enforcement is identical for
// every caller (cell codex-parity-6b reuses it for the installed-route rows,
// whose required ids are DERIVED FROM .codex/hooks.json itself — so dropping a
// configured command from the harness cannot go unnoticed either).
function enforceRequiredRows(results, requiredIds, group = "catalog-drift") {
  const extra = [];
  for (const id of requiredIds) {
    const row = results.find((r) => r.id === id);
    if (!row) {
      extra.push(
        genericRow(
          group,
          `required-row-present:${id}`,
          false,
          `REQUIRED ROW ABSENT: "${id}" was never registered in this run — a required row ` +
            "cannot be silently dropped (cell codex-parity-6a required-row manifest)",
        ),
      );
    } else if (row.skip) {
      extra.push(
        genericRow(
          group,
          `required-row-present:${id}`,
          false,
          `REQUIRED ROW SKIPPED: "${id}" reported skip — a required row may never be skipped ` +
            "(cell codex-parity-6a required-row manifest)",
        ),
      );
    }
  }
  return extra;
}

function genericRow(group, id, pass, note, extra = {}) {
  return {
    wrapper: group,
    id,
    status: extra.status ?? 0,
    signal: extra.signal ?? null,
    stdout: extra.stdout ?? "",
    stderr: extra.stderr ?? "",
    pass,
    note,
  };
}

function runWorktreeAdapterRows() {
  const mainRoot = mkFixture("hook-adapter-main-");
  const workRoot = mkFixture("hook-adapter-work-");
  const invalidRoot = mkFixture("hook-adapter-invalid-");
  const separateRoot = mkFixture("hook-adapter-separate-");
  try {
    const gitdir = path.join(mainRoot, ".git", "worktrees", "fixture");
    fs.mkdirSync(gitdir, { recursive: true });
    fs.writeFileSync(path.join(workRoot, ".git"), `gitdir: ${gitdir}\n`);
    fs.writeFileSync(path.join(gitdir, "gitdir"), `${path.join(workRoot, ".git")}\n`);
    const linked = resolveHookRoots(workRoot);

    const badGitdir = path.join(mainRoot, ".git", "worktrees", "bad");
    fs.mkdirSync(badGitdir, { recursive: true });
    fs.writeFileSync(path.join(invalidRoot, ".git"), `gitdir: ${badGitdir}\n`);
    const invalid = resolveHookRoots(invalidRoot);

    const detached = path.join(separateRoot, "git-data");
    fs.mkdirSync(detached);
    fs.writeFileSync(path.join(separateRoot, ".git"), `gitdir: ${detached}\n`);
    const separate = resolveHookRoots(separateRoot);

    return [
      genericRow(
        "worktree-adapter",
        "typed-linked-valid-root-store-split",
        linked.worktreeResolution === "linked-valid" && linked.workRoot === fs.realpathSync.native(workRoot) && linked.storeRoot === fs.realpathSync.native(mainRoot),
        `linked=${JSON.stringify(linked)}`,
      ),
      genericRow(
        "worktree-adapter",
        "typed-linked-invalid-nonthrowing",
        invalid.worktreeResolution === "linked-invalid" && invalid.workRoot === fs.realpathSync.native(invalidRoot) && invalid.storeRoot === null,
        `invalid=${JSON.stringify(invalid)}`,
      ),
      genericRow(
        "worktree-adapter",
        "separate-git-dir-remains-ordinary",
        separate.worktreeResolution === "ordinary" && separate.workRoot === fs.realpathSync.native(separateRoot) && separate.storeRoot === fs.realpathSync.native(separateRoot),
        `separate=${JSON.stringify(separate)}`,
      ),
      // msn-18c: controlRootFor(root) — session/workflow/claims/handoff-mailbox
      // stores are control-plane, grant-INDEPENDENT (unlike storeRoot above,
      // which follows the worktree-grants.json opt-in). A linked worktree
      // resolves to MAIN regardless of whether it also holds its own granted
      // local store; every other topology (ordinary, linked-invalid) falls
      // back to `root` itself — never null, never a throw (fail-open, same
      // posture as every other resolution in this file).
      genericRow(
        "worktree-adapter",
        "control-root-linked-valid-resolves-to-main",
        hookControlRootFor(workRoot) === fs.realpathSync.native(mainRoot),
        `controlRootFor(workRoot)=${hookControlRootFor(workRoot)} main=${fs.realpathSync.native(mainRoot)}`,
      ),
      genericRow(
        "worktree-adapter",
        "control-root-linked-invalid-fails-open-to-root-itself",
        hookControlRootFor(invalidRoot) === fs.realpathSync.native(invalidRoot),
        `controlRootFor(invalidRoot)=${hookControlRootFor(invalidRoot)} invalidRoot=${fs.realpathSync.native(invalidRoot)}`,
      ),
      genericRow(
        "worktree-adapter",
        "control-root-ordinary-checkout-resolves-to-itself",
        hookControlRootFor(separateRoot) === fs.realpathSync.native(separateRoot),
        `controlRootFor(separateRoot)=${hookControlRootFor(separateRoot)} separate=${fs.realpathSync.native(separateRoot)}`,
      ),
    ];
  } finally {
    fs.rmSync(mainRoot, { recursive: true, force: true });
    fs.rmSync(workRoot, { recursive: true, force: true });
    fs.rmSync(invalidRoot, { recursive: true, force: true });
    fs.rmSync(separateRoot, { recursive: true, force: true });
  }
}

const WRAPPERS = [
  "bee-session-init.mjs",
  "bee-prompt-context.mjs",
  "bee-state-sync.mjs",
  "bee-chain-nudge.mjs",
  "bee-session-close.mjs",
  "bee-model-guard.mjs",
  "bee-write-guard.mjs",
  "bee-tools-logger.mjs",
];

const HUGE_BLOB_LEN = 2 * 1024 * 1024; // ~2MB
const SPAWN_TIMEOUT_MS = 20000;
const SPAWN_MAX_BUFFER = 20 * 1024 * 1024;

// --- fixture builder ------------------------------------------------------

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

// One shared fixture shape for every wrapper: phase "swarming" with every
// gate approved (the most permissive gated phase, matching
// hooks/test_write_guard.mjs's buildFixture default) and NO .bee/HANDOFF.json.
// This is deliberate, not incidental: it is the one state that lets every
// row exercise real wrapper behavior instead of an early no-op --
// bee-session-close's "hive door open" warning and bee-chain-nudge's
// SubagentStop nudge both only fire outside phase "idle" with no HANDOFF, and
// bee-write-guard's unconditional `.bee/state.json` direct-edit deny needs a
// phase where writes are otherwise allowed to prove the deny is truly
// unconditional (not merely a side effect of a gate that would deny anyway).
function buildFixture(prefix) {
  const root = mkFixture(prefix);
  fs.mkdirSync(path.join(root, ".bee"), { recursive: true });
  fs.writeFileSync(path.join(root, ".bee", "onboarding.json"), "{}\n");
  copyLib(root);
  fs.writeFileSync(
    path.join(root, ".bee", "state.json"),
    `${JSON.stringify(
      {
        phase: "swarming",
        mode: "standard",
        feature: "demo",
        approved_gates: { context: true, shape: true, execution: true, review: false },
      },
      null,
      2,
    )}\n`,
  );
  return root;
}

// --- wrapper invocation ----------------------------------------------------

async function runWrapper(wrapperBase, input, cwd, extraArgs = []) {
  const hookPath = path.join(HOOKS_DIR, wrapperBase);
  return await runModuleWorker(hookPath, {
    args: extraArgs,
    input,
    cwd,
    timeout: SPAWN_TIMEOUT_MS,
  });
}

// --- expectation helpers (the TARGET/desired contract) ---------------------

function truncate(text, len) {
  const s = text == null ? "" : String(text);
  if (s.length <= len) return s;
  return `${s.slice(0, len)}...[TRUNCATED, ${s.length} total chars]`;
}

// Malformed/adversarial input must never crash the wrapper (D2: fail open
// with visible limits). None of these rows are a legitimate deny case, so
// the only acceptable exit is 0. A Node uncaught exception exits non-zero
// (usually 1) with a stack trace on stderr - that is exactly the failure
// mode this row set is built to catch.
function expectNoCrash(result) {
  if (result.status === 0) {
    return { pass: true, note: "exit 0 (fail-open) as required" };
  }
  if (result.status == null && result.error) {
    return { pass: false, note: `launch error: ${result.error.message}` };
  }
  return {
    pass: false,
    note:
      `expected exit 0 (fail-open) on malformed/adversarial input; got status=${result.status}` +
      `${result.signal ? ` signal=${result.signal}` : ""}`,
  };
}

function parseAdvisoryStdout(stdout) {
  const trimmed = (stdout || "").trim();
  if (!trimmed) return { empty: true, json: null };
  try {
    return { empty: false, json: JSON.parse(trimmed) };
  } catch {
    return { empty: false, json: null };
  }
}

// approach.md section 2: "Encode Codex PreCompact, SubagentStop, and Stop
// advisories as parseable JSON systemMessage. Do not use decision: "block",
// so close/nudge behavior remains advisory and cannot accidentally continue
// a child or loop a turn." Silence is fine (many hooks are conditionally
// silent); any non-empty stdout must be a JSON object with a string
// `systemMessage` and never `decision:"block"`.
function expectAdvisoryJsonOrSilent(result) {
  if (result.status !== 0) {
    if (result.status == null && result.error) {
      return { pass: false, note: `launch error: ${result.error.message}` };
    }
    return { pass: false, note: `expected exit 0 (advisory-only, never blocking); got status=${result.status}` };
  }
  const parsed = parseAdvisoryStdout(result.stdout);
  if (parsed.empty) {
    return { pass: true, note: "silent (no advisory text emitted) - acceptable" };
  }
  if (!parsed.json || typeof parsed.json !== "object" || Array.isArray(parsed.json)) {
    return {
      pass: false,
      note:
        "stdout is non-empty but is not a parseable JSON object - Codex advisory output must be a JSON " +
        `systemMessage, not plain prose. Actual stdout: ${truncate(result.stdout, 500)}`,
    };
  }
  if (typeof parsed.json.systemMessage !== "string") {
    return {
      pass: false,
      note: `parsed JSON lacks a string "systemMessage" field. Actual stdout: ${truncate(result.stdout, 500)}`,
    };
  }
  if (parsed.json.decision === "block") {
    return {
      pass: false,
      note: `advisory used decision:"block" (must stay advisory-only, never blocking). Actual stdout: ${truncate(result.stdout, 500)}`,
    };
  }
  return { pass: true, note: "parseable JSON systemMessage, non-blocking" };
}

// approach.md section 2: "Adapt Codex write guarding to canonical apply_patch
// input ... run the existing gate/direct-edit/reservation decisions on every
// target." `.bee/state.json` is direct-edit-denied in EVERY phase regardless
// of tool shape (guards.mjs DIRECT_EDIT_DENY, proven unconditional by
// hooks/test_write_guard.mjs row1/row6) - so an apply_patch write targeting
// it must be denied (exit 2) exactly like an Edit/Write/Bash write is today.
function expectApplyPatchDenied(result) {
  if (result.status === 2 && typeof result.stderr === "string" && result.stderr.trim()) {
    return { pass: true, note: "apply_patch target denied (exit 2) with a reason" };
  }
  if (result.status == null && result.error) {
    return { pass: false, note: `launch error: ${result.error.message}` };
  }
  return {
    pass: false,
    note:
      "expected exit 2 with a deny reason for an apply_patch write targeting .bee/state.json " +
      "(D2: apply_patch must run the same gate/direct-edit/reservation decisions as Edit/Write/Bash); " +
      `got status=${result.status} stderr=${truncate(result.stderr, 500)}`,
  };
}

// A Codex PreToolUse/apply_patch event is simply not a dispatch tool
// (Agent|Task) for bee-model-guard - approach.md section 2 explicitly keeps
// model-tier guard Claude-only ("Codex does not expose collaboration spawn
// through PreToolUse"). The only requirement here is that the unfamiliar
// shape doesn't crash the wrapper.
const expectApplyPatchIgnoredByModelGuard = expectNoCrash;

// --- row table ---------------------------------------------------------

function universalRows(fixtureRoot) {
  return [
    {
      id: "empty-stdin",
      input: "",
      expect: expectNoCrash,
    },
    {
      id: "junk-bytes",
      input: "not json at all {{{",
      expect: expectNoCrash,
    },
    {
      id: "top-level-null",
      input: "null",
      expect: expectNoCrash,
    },
    {
      id: "json-array",
      input: "[]",
      expect: expectNoCrash,
    },
    {
      id: "object-cwd",
      input: JSON.stringify({ cwd: { not: "a string" } }),
      expect: expectNoCrash,
    },
    {
      id: "missing-cwd",
      input: JSON.stringify({ tool_name: "Read" }),
      expect: expectNoCrash,
    },
    {
      id: "huge-payload-2mb",
      input: JSON.stringify({ cwd: fixtureRoot, blob: "a".repeat(HUGE_BLOB_LEN) }),
      expect: expectNoCrash,
    },
  ];
}

const APPLY_PATCH_PAYLOAD = (fixtureRoot) =>
  JSON.stringify({
    hook_event_name: "PreToolUse",
    tool_name: "apply_patch",
    tool_input: {
      input:
        "*** Begin Patch\n*** Update File: .bee/state.json\n@@\n-old\n+new\n*** End Patch",
    },
    cwd: fixtureRoot,
  });

// Event-output parse rows, wired to match the ACTUAL event -> wrapper
// matchers in hooks/hooks.json (not every event is meaningful for every
// wrapper): PreCompact only fires bee-session-close; SubagentStop fires
// bee-state-sync + bee-chain-nudge; Stop fires bee-state-sync +
// bee-session-close; PreToolUse (apply_patch) fires bee-write-guard +
// bee-model-guard.
function eventRows(wrapperBase, fixtureRoot) {
  const rows = [];
  if (wrapperBase === "bee-session-close.mjs") {
    rows.push({
      id: "codex-precompact-advisory",
      input: JSON.stringify({ hook_event_name: "PreCompact", cwd: fixtureRoot }),
      expect: expectAdvisoryJsonOrSilent,
    });
  }
  if (wrapperBase === "bee-state-sync.mjs" || wrapperBase === "bee-chain-nudge.mjs") {
    rows.push({
      id: "codex-subagentstop-advisory",
      input: JSON.stringify({
        hook_event_name: "SubagentStop",
        agent_name: "kevin",
        cwd: fixtureRoot,
      }),
      expect: expectAdvisoryJsonOrSilent,
    });
  }
  if (wrapperBase === "bee-state-sync.mjs" || wrapperBase === "bee-session-close.mjs") {
    rows.push({
      id: "codex-stop-advisory",
      input: JSON.stringify({ hook_event_name: "Stop", cwd: fixtureRoot }),
      expect: expectAdvisoryJsonOrSilent,
    });
  }
  if (wrapperBase === "bee-write-guard.mjs") {
    rows.push({
      id: "codex-pretooluse-applypatch-deny",
      input: APPLY_PATCH_PAYLOAD(fixtureRoot),
      expect: expectApplyPatchDenied,
    });
  }
  if (wrapperBase === "bee-model-guard.mjs") {
    rows.push({
      id: "codex-pretooluse-applypatch-ignored",
      input: APPLY_PATCH_PAYLOAD(fixtureRoot),
      expect: expectApplyPatchIgnoredByModelGuard,
    });
  }
  if (wrapperBase === "bee-tools-logger.mjs") {
    rows.push(...toolsLoggerRows(fixtureRoot));
  }
  if (wrapperBase === "bee-state-sync.mjs") {
    rows.push(stateSyncUpdatePlanRow(fixtureRoot));
  }
  return rows;
}

// D4 (codex-native-runtime-v2, advisor findings 1/2): bee-state-sync.mjs
// filters on NEITHER tool_name NOR tool_input - the PostToolUse matcher lives
// in hooks.json/catalog.mjs, not inside this wrapper. This row proves the
// wrapper still does its one job (refresh cell counts + last_activity, leave
// every unrelated state.json field untouched) when fed a REAL update_plan
// payload shape. A cell is planted on disk right before the row is built (not
// inside `expect`, which runs after the wrapper has already executed) so the
// refreshed count is a fact the wrapper had to compute, not a fixture that
// merely echoes back its own initial state.
function stateSyncUpdatePlanRow(fixtureRoot) {
  const cellsDir = path.join(fixtureRoot, ".bee", "cells");
  fs.mkdirSync(cellsDir, { recursive: true });
  fs.writeFileSync(
    path.join(cellsDir, "update-plan-probe-1.json"),
    `${JSON.stringify({ id: "update-plan-probe-1", feature: "demo", status: "open" }, null, 2)}\n`,
  );
  const stateBefore = JSON.parse(
    fs.readFileSync(path.join(fixtureRoot, ".bee", "state.json"), "utf8"),
  );
  return {
    id: "update-plan-refreshes-counts-preserves-unrelated-state",
    input: JSON.stringify({
      hook_event_name: "PostToolUse",
      tool_name: "update_plan",
      tool_input: { plan: [{ step: "ship D4", status: "in_progress" }] },
      cwd: fixtureRoot,
    }),
    expect: (result) => {
      if (result.status !== 0) {
        return { pass: false, note: `expected exit 0 (silent state-sync); got status=${result.status}` };
      }
      const stateAfter = JSON.parse(
        fs.readFileSync(path.join(fixtureRoot, ".bee", "state.json"), "utf8"),
      );
      const countsOk =
        stateAfter.cells &&
        stateAfter.cells.open === 1 &&
        stateAfter.cells.claimed === 0 &&
        stateAfter.cells.capped === 0 &&
        stateAfter.cells.blocked === 0;
      const activityOk =
        typeof stateAfter.last_activity === "string" && !Number.isNaN(Date.parse(stateAfter.last_activity));
      const unrelatedPreserved =
        stateAfter.phase === stateBefore.phase &&
        stateAfter.mode === stateBefore.mode &&
        stateAfter.feature === stateBefore.feature &&
        JSON.stringify(stateAfter.approved_gates) === JSON.stringify(stateBefore.approved_gates);
      if (!countsOk) {
        return {
          pass: false,
          note: `cell counts not refreshed from the planted fixture cell: ${JSON.stringify(stateAfter.cells)}`,
        };
      }
      if (!activityOk) {
        return { pass: false, note: `last_activity not a valid ISO timestamp: ${stateAfter.last_activity}` };
      }
      if (!unrelatedPreserved) {
        return {
          pass: false,
          note:
            `unrelated state.json fields changed: before=${JSON.stringify(stateBefore)} ` +
            `after=${JSON.stringify(stateAfter)}`,
        };
      }
      return {
        pass: true,
        note: "update_plan payload refreshed cell counts + last_activity; unrelated state preserved",
      };
    },
  };
}

// W4 (advisor-and-orchestration, AO15/decision f1ca79b9): bee-tools-logger.mjs
// is a passive PostToolUse measurement hook — zero enforcement, fail-open.
// Distinguishing synthetic tool_name values (never produced by any other row
// in this file, including the universal malformed-input rows) so each
// assertion below targets its OWN emitted line, not an incidental one from an
// earlier row sharing this wrapper's single fixtureRoot.
//
// The standing trap this row set exists to close: fail-open turns a crashed
// logger into universal green. The happy-path rows are the "fails when
// broken" half — they FAIL if the logger silently stops writing lines.
function toolsLoggerRows(fixtureRoot) {
  const rows = [];
  rows.push({
    id: "tools-logger-happy-path-orchestrator",
    input: JSON.stringify({
      hook_event_name: "PostToolUse",
      tool_name: "ToolsLoggerHappyOrchestratorProbe",
      tool_input: { secret: "must-never-be-logged" },
      cwd: fixtureRoot,
    }),
    expect: (result) => {
      if (result.status !== 0) {
        return { pass: false, note: `expected exit 0 (passive logger); got status=${result.status}` };
      }
      const line = readToolsJsonl(fixtureRoot).find(
        (l) => l.parsed.tool_name === "ToolsLoggerHappyOrchestratorProbe",
      );
      if (!line) {
        return {
          pass: false,
          note: "no tools.jsonl line was written for an orchestrator-shaped PostToolUse payload " +
            "(fails-when-broken: the logger must write on every enabled call)",
        };
      }
      const { ts, tool_name, agent_id, agent_type, ...rest } = line.parsed;
      const shapeOk =
        typeof ts === "string" &&
        tool_name === "ToolsLoggerHappyOrchestratorProbe" &&
        agent_id === null &&
        agent_type === null &&
        Object.keys(rest).length === 0;
      const noBody = !line.raw.includes("must-never-be-logged");
      if (!shapeOk) {
        return { pass: false, note: `unexpected line shape: ${line.raw}` };
      }
      if (!noBody) {
        return { pass: false, note: `tool_input body leaked into the log line: ${line.raw}` };
      }
      return { pass: true, note: "orchestrator call logged with tool_name + null attribution, no body" };
    },
  });
  rows.push({
    id: "tools-logger-happy-path-subagent",
    input: JSON.stringify({
      hook_event_name: "PostToolUse",
      tool_name: "ToolsLoggerHappySubagentProbe",
      agent_id: "agent-jerry-1",
      agent_type: "generation",
      tool_response: { secret: "must-never-be-logged-either" },
      cwd: fixtureRoot,
    }),
    expect: (result) => {
      if (result.status !== 0) {
        return { pass: false, note: `expected exit 0 (passive logger); got status=${result.status}` };
      }
      const line = readToolsJsonl(fixtureRoot).find(
        (l) => l.parsed.tool_name === "ToolsLoggerHappySubagentProbe",
      );
      if (!line) {
        return { pass: false, note: "no tools.jsonl line was written for a subagent-shaped PostToolUse payload" };
      }
      const attributionOk = line.parsed.agent_id === "agent-jerry-1" && line.parsed.agent_type === "generation";
      const noBody = !line.raw.includes("must-never-be-logged-either");
      if (!attributionOk) {
        return { pass: false, note: `agent_id/agent_type not logged verbatim: ${line.raw}` };
      }
      if (!noBody) {
        return { pass: false, note: `tool_response body leaked into the log line: ${line.raw}` };
      }
      return { pass: true, note: "subagent call logged with agent_id/agent_type verbatim, no body" };
    },
  });
  return rows;
}

// Crash-injection half of the fails-when-broken pair, run standalone (own
// dedicated fixture, own direct runWrapper call — never the shared
// per-wrapper fixture the WRAPPERS loop above reuses across every row,
// because this row deliberately corrupts state.mjs so it throws on import
// and that corruption must not leak into any other row). Mirrors
// test_model_guard.mjs's buildThrowingStateFixture pattern. fail-open must
// stay VISIBLE (a crash line lands in .bee/logs/hooks.jsonl), never
// fail-silent — and the broken logger must NOT have written a tools.jsonl
// line for the call that crashed it.
async function runToolsLoggerCrashRows() {
  const root = buildFixture("hook-contracts-tools-logger-crash-");
  fs.writeFileSync(
    path.join(root, ".bee", "bin", "lib", "state.mjs"),
    "throw new Error('boom: fixture state.mjs deliberately throws on import');\n",
  );
  const result = await runWrapper(
    "bee-tools-logger.mjs",
    JSON.stringify({
      hook_event_name: "PostToolUse",
      tool_name: "ToolsLoggerCrashProbe",
      cwd: root,
    }),
    root,
  );
  const crashLine = readHooksJsonl(root).find((l) => l && l.hook === "tools-logger" && l.error);
  const noToolsLine = !readToolsJsonl(root).some(
    (l) => l.parsed.tool_name === "ToolsLoggerCrashProbe",
  );
  const pass = result.status === 0 && Boolean(crashLine) && noToolsLine;
  return [
    genericRow(
      "bee-tools-logger.mjs",
      "tools-logger-crash-injection-fails-open-visibly",
      pass,
      pass
        ? "broken state.mjs -> exit 0 (fail-open) AND a visible crash line in hooks.jsonl, no silent tools.jsonl write"
        : `expected exit 0 + visible crash line + no tools.jsonl write; got status=${result.status} ` +
            `crashLine=${JSON.stringify(crashLine)} noToolsLine=${noToolsLine}`,
      { status: result.status, stdout: result.stdout, stderr: result.stderr },
    ),
  ];
}

function buildRowsForWrapper(wrapperBase, fixtureRoot) {
  return [...universalRows(fixtureRoot), ...eventRows(wrapperBase, fixtureRoot)];
}

// --- report writer -----------------------------------------------------

function previewInput(input) {
  return truncate(input, 300);
}

function writeBaselineReport(results, failures) {
  const total = results.length;
  const byWrapper = new Map();
  for (const r of results) {
    if (!byWrapper.has(r.wrapper)) byWrapper.set(r.wrapper, []);
    byWrapper.get(r.wrapper).push(r);
  }

  const lines = [];
  lines.push("# RED Baseline — Codex Runtime Parity (cell codex-parity-1)");
  lines.push("");
  lines.push(
    "Generated by `node hooks/test_hook_contracts.mjs --baseline` against the " +
      "CURRENT unmodified wrappers in `hooks/`. This cell fixes nothing — decision " +
      "D2 (CONTEXT.md) and learning `20260711-model-tier-guard` require adversarial " +
      "rows checkpointed BEFORE any repair. Every failure below is RED evidence for " +
      "the repair cells in the Safety foundation slice (plan.md epic E1).",
  );
  lines.push("");
  lines.push(`Generated: ${new Date().toISOString()}`);
  lines.push("");
  lines.push("## Summary");
  lines.push("");
  lines.push(`- ${WRAPPERS.length} wrappers, ${total} total probe rows.`);
  lines.push(
    `- **${failures.length} of ${total}** probes fail the target/desired contract encoded in ` +
      "`hooks/test_hook_contracts.mjs` (run the harness without `--baseline` to see this as a " +
      "non-zero exit code).",
  );
  lines.push("");
  lines.push("| Wrapper | Rows | Failing |");
  lines.push("|---|---|---|");
  for (const wrapperBase of WRAPPERS) {
    const rows = byWrapper.get(wrapperBase) || [];
    const fail = rows.filter((r) => !r.pass).length;
    lines.push(`| ${wrapperBase} | ${rows.length} | ${fail} |`);
  }
  lines.push("");
  lines.push("## Findings by wrapper");

  for (const wrapperBase of WRAPPERS) {
    const rows = byWrapper.get(wrapperBase) || [];
    lines.push("");
    lines.push(`### ${wrapperBase}`);
    lines.push("");
    lines.push("| Row | Verdict | Note |");
    lines.push("|---|---|---|");
    for (const r of rows) {
      const verdict = r.pass ? "ok" : "**RED**";
      lines.push(`| ${r.id} | ${verdict} | ${r.note.replace(/\|/g, "\\|")} |`);
    }

    const rowFailures = rows.filter((r) => !r.pass);
    if (rowFailures.length > 0) {
      lines.push("");
      lines.push("#### RED failures (verbatim)");
      for (const r of rowFailures) {
        lines.push("");
        lines.push(`**${r.id}**`);
        lines.push("");
        lines.push(`- input sent: \`${previewInput(r.inputRaw)}\``);
        lines.push(`- exit status: ${r.status === null ? "null" : r.status}`);
        lines.push(`- signal: ${r.signal || "null"}`);
        lines.push("- stderr:");
        lines.push("```");
        lines.push(r.stderr && r.stderr.length ? r.stderr : "<empty>");
        lines.push("```");
        lines.push("- stdout:");
        lines.push("```");
        lines.push(r.stdout && r.stdout.length ? truncate(r.stdout, 2000) : "<empty>");
        lines.push("```");
      }
    }
  }

  lines.push("");
  lines.push("## Decision D2");
  lines.push("");
  lines.push(
    "> D2 | Codex receives full hook parity on every compatible event and tool path: " +
      "session bootstrap, prompt reminder, write/privacy/reservation guard, state sync, " +
      "subagent-chain nudge, and session-close hygiene. Shared helpers remain the final " +
      "enforcement belt; unsupported paths fail open with visible limits and runtime-specific " +
      "tests. | The goal is Claude-like understanding and enforcement without pretending hooks " +
      "are a complete security boundary. Durable decision: `b7af1bf9`.",
  );
  lines.push("");
  lines.push(
    "This baseline is the \"before\" state proving the current wrappers do not yet meet that " +
      "bar. In this run:",
  );
  lines.push("");
  const crashers = WRAPPERS.filter((w) => {
    const rows = byWrapper.get(w) || [];
    return rows.some((r) => !r.pass && (r.id === "top-level-null" || r.id === "object-cwd"));
  });
  lines.push(
    `- ${crashers.length} of ${WRAPPERS.length} wrapper(s) crash (uncaught exception, non-zero exit) ` +
      "on a top-level JSON `null` payload and/or a non-string (object) `cwd`, instead of failing open: " +
      `${crashers.length ? crashers.join(", ") : "none"}.`,
  );
  const applyPatchDenyRow = (byWrapper.get("bee-write-guard.mjs") || []).find(
    (r) => r.id === "codex-pretooluse-applypatch-deny",
  );
  lines.push(
    `- bee-write-guard.mjs ${applyPatchDenyRow && !applyPatchDenyRow.pass ? "silently ALLOWS" : "correctly denies"} ` +
      "an `apply_patch` write whose target (`.bee/state.json`) is unconditionally direct-edit-denied for " +
      "every other recognized tool shape.",
  );
  const advisoryRowIds = [
    "codex-precompact-advisory",
    "codex-subagentstop-advisory",
    "codex-stop-advisory",
  ];
  const advisoryFailers = WRAPPERS.filter((w) => {
    const rows = byWrapper.get(w) || [];
    return rows.some((r) => !r.pass && advisoryRowIds.includes(r.id));
  });
  lines.push(
    `- ${advisoryFailers.length ? advisoryFailers.join(", ") : "none"} emit plain-text prose for a Codex ` +
      "PreCompact/SubagentStop/Stop advisory instead of the parseable JSON `systemMessage` D2 and " +
      "approach.md section 2 call for.",
  );
  lines.push("");

  fs.mkdirSync(path.dirname(REPORT_PATH), { recursive: true });
  fs.writeFileSync(REPORT_PATH, `${lines.join("\n")}\n`);
}

// --- catalog drift-check + allowed-differences (cell codex-parity-2) ------
//
// Proves hooks/catalog.mjs is the single source of truth for both checked-in
// projections: rendering "claude" must reproduce hooks/claude-hooks.json
// byte-for-byte, rendering "codex" must reproduce hooks/hooks.json (the
// Codex default projection) byte-for-byte, and every directional structural
// difference between them is named in ALLOWED_DIFFERENCES. Any other drift —
// added, removed, or reordered rules that are not declared — fails this row.

function catalogDriftRow(id, pass, note) {
  return { wrapper: "catalog-drift", id, status: 0, signal: null, stdout: "", stderr: "", pass, note };
}

function groupMatchesDifference(group, difference) {
  const matcherMatches = difference.matcher === null
    ? group.matcher === undefined
    : group.matcher === difference.matcher;
  return matcherMatches && group.hooks.some(
    (hook) => typeof hook.command === "string" && hook.command.includes(`/hooks/${difference.script}`),
  );
}

function projectionHasDifference(projection, difference) {
  return (projection.hooks[difference.event] || []).some(
    (group) => groupMatchesDifference(group, difference),
  );
}

function removeDifference(projection, difference) {
  const groups = projection.hooks[difference.event] || [];
  const remainingGroups = [];
  for (const group of groups) {
    const matcherMatches = difference.matcher === null
      ? group.matcher === undefined
      : group.matcher === difference.matcher;
    if (!matcherMatches) {
      remainingGroups.push(group);
      continue;
    }
    const hooks = group.hooks.filter(
      (hook) => !(typeof hook.command === "string" && hook.command.includes(`/hooks/${difference.script}`)),
    );
    if (hooks.length > 0) remainingGroups.push({ ...group, hooks });
  }
  if (remainingGroups.length > 0) projection.hooks[difference.event] = remainingGroups;
  else delete projection.hooks[difference.event];
}

function commandEntries(projection, event, script) {
  return (projection.hooks[event] || [])
    .flatMap((group) => group.hooks || [])
    .filter(
      (hook) => typeof hook.command === "string" && hook.command.includes(`/hooks/${script}`),
    );
}

function runCatalogDriftChecks() {
  const rows = [];

  const claudeText = renderProjectionText(RUNTIMES.CLAUDE);
  const codexText = renderProjectionText(RUNTIMES.CODEX);
  const onDiskClaude = fs.readFileSync(CLAUDE_HOOKS_PATH, "utf8");
  const onDiskCodex = fs.readFileSync(CODEX_DEFAULT_HOOKS_PATH, "utf8");

  rows.push(
    catalogDriftRow(
      "claude-projection-byte-identical",
      claudeText === onDiskClaude,
      claudeText === onDiskClaude
        ? "rendering the logical catalog for \"claude\" reproduces hooks/claude-hooks.json byte-for-byte"
        : "DRIFT: rendering the logical catalog for \"claude\" does NOT reproduce hooks/claude-hooks.json byte-for-byte",
    ),
  );
  rows.push(
    catalogDriftRow(
      "codex-projection-byte-identical",
      codexText === onDiskCodex,
      codexText === onDiskCodex
        ? "rendering the logical catalog for \"codex\" reproduces hooks/hooks.json (Codex default projection) byte-for-byte"
        : "DRIFT: rendering the logical catalog for \"codex\" does NOT reproduce hooks/hooks.json byte-for-byte",
    ),
  );

  const claudeProjection = renderProjection(RUNTIMES.CLAUDE);
  const codexProjection = renderProjection(RUNTIMES.CODEX);

  // Every directional difference must be present only on its named runtime.
  const declaredAccurate = ALLOWED_DIFFERENCES.every((d) => {
    const owning = d.runtime === RUNTIMES.CLAUDE ? claudeProjection : codexProjection;
    const other = d.runtime === RUNTIMES.CLAUDE ? codexProjection : claudeProjection;
    return projectionHasDifference(owning, d) && !projectionHasDifference(other, d);
  });

  // Remove exactly the declared runtime-owned hooks from both sides; the
  // remainder must be byte-structurally equal.
  const claudeWithAllowedRemoved = JSON.parse(JSON.stringify(claudeProjection));
  const codexWithAllowedRemoved = JSON.parse(JSON.stringify(codexProjection));
  for (const d of ALLOWED_DIFFERENCES) {
    removeDifference(
      d.runtime === RUNTIMES.CLAUDE ? claudeWithAllowedRemoved : codexWithAllowedRemoved,
      d,
    );
  }
  const noUndeclaredDrift =
    JSON.stringify(claudeWithAllowedRemoved) === JSON.stringify(codexWithAllowedRemoved);

  const allowedDifferencesOk = declaredAccurate && noUndeclaredDrift;
  rows.push(
    catalogDriftRow(
      "allowed-differences-only",
      allowedDifferencesOk,
      allowedDifferencesOk
        ? `only the declared difference(s) separate the two projections: ${ALLOWED_DIFFERENCES.map((d) => d.id).join(", ")}`
        : "DRIFT: claude/codex projections differ beyond the declared ALLOWED_DIFFERENCES " +
            `(declaredAccurate=${declaredAccurate}, noUndeclaredDrift=${noUndeclaredDrift})`,
    ),
  );

  const codexStartAudit = commandEntries(
    codexProjection,
    "SubagentStart",
    "bee-codex-subagent-audit.mjs",
  );
  const codexStopAudit = commandEntries(
    codexProjection,
    "SubagentStop",
    "bee-codex-subagent-audit.mjs",
  );
  const claudeAuditCount = ["SubagentStart", "SubagentStop"].reduce(
    (count, event) => count + commandEntries(
      claudeProjection,
      event,
      "bee-codex-subagent-audit.mjs",
    ).length,
    0,
  );
  // rust-port R6: the plugin projection prefers the native binary and keeps
  // the node wrapper as the fallback arm of the SAME one-line command (see
  // catalog.mjs pluginCommand). Pinned whole, because the binary candidate
  // ORDER is the contract: the host repo's own .bee/bin first (the
  // deployment-true location — the binary is machine-local, never shipped in
  // a plugin root), then the plugin root for a built-in-place source
  // checkout, then node.
  const expectedPluginAuditCommand =
    'for b in "$CLAUDE_PROJECT_DIR/.bee/bin/bee" "$CLAUDE_PROJECT_DIR/.bee/bin/bee.exe" ' +
    '"${CLAUDE_PLUGIN_ROOT}/.bee/bin/bee" "${CLAUDE_PLUGIN_ROOT}/.bee/bin/bee.exe"; ' +
    'do [ -x "$b" ] && exec "$b" hook codex-subagent-audit; done; ' +
    'exec node "${CLAUDE_PLUGIN_ROOT}/packages/bee/hooks/bee-codex-subagent-audit.mjs"';
  const pluginTopologyOk =
    codexStartAudit.length === 1 &&
    codexStopAudit.length === 1 &&
    codexStartAudit[0].command === expectedPluginAuditCommand &&
    codexStopAudit[0].command === expectedPluginAuditCommand &&
    claudeAuditCount === 0;
  rows.push(
    catalogDriftRow(
      "codex-plugin-subagent-audit-topology",
      pluginTopologyOk,
      pluginTopologyOk
        ? "Codex plugin maps SubagentStart and SubagentStop exactly once to one plugin-root audit handler; Claude maps neither"
        : `unexpected plugin topology: start=${codexStartAudit.length} stop=${codexStopAudit.length} claude=${claudeAuditCount}`,
    ),
  );

  // --- generated repo-target rows -----------------------------------------
  //
  // Root .codex/hooks.json is regenerated FROM the catalog repo-target render
  // (cell cnr2-7, codex-native-runtime-v2 D4) and pinned byte-identical below
  // - no allowed-difference masks it, so this is a real drift check, not a
  // development/fallback snapshot left unproven. A separate fixture executes
  // its new SubagentStart/Stop audit commands further down.

  // D2 / the incident itself: the repo transport must carry NO Claude-only
  // root variable (Codex sets neither; $CLAUDE_PROJECT_DIR unset is exactly
  // what collapsed the old commands to `node /.bee/bin/hooks/...` and killed
  // every hook with MODULE_NOT_FOUND), must launch the CURRENT source
  // wrappers under .bee/bin/hooks/, must declare source identity repo, and
  // must carry the pinned VISIBLE fail-open diagnostic on stderr.
  const repoProjection = renderProjection(RUNTIMES.CODEX, { target: TARGETS.REPO });
  const repoCommands = Object.values(repoProjection.hooks)
    .flat()
    .flatMap((g) => g.hooks)
    .map((h) => h.command);
  const noClaudeVars = repoCommands.every(
    (c) => !/CLAUDE_PROJECT_DIR|CLAUDE_PLUGIN_ROOT/.test(c),
  );
  const launchesSourceWrappers = repoCommands.every((c) =>
    /exec node "\$r"\/\.bee\/bin\/hooks\/bee-[a-z-]+\.mjs --source=repo$/m.test(c),
  );
  // rust-port R6: the repo transport tries the vendored native binary FIRST
  // and only falls through to the wrapper above when the host carries none.
  // Both arms are asserted: dropping the binary arm silently re-Nodes every
  // Codex hook, dropping the node arm breaks every host that has not built
  // the binary yet.
  const prefersVendoredBinary = repoCommands.every((c) =>
    /^for b in "\$r"\/\.bee\/bin\/bee "\$r"\/\.bee\/bin\/bee\.exe; do \[ -x "\$b" \] && exec "\$b" hook [a-z-]+ --source=repo; done$/m.test(
      c,
    ),
  );
  const visibleFailOpen = repoCommands.every(
    (c) =>
      c.includes(`echo "${REPO_TRANSPORT_UNAVAILABLE_DIAGNOSTIC}" >&2`) &&
      c.includes("exit 0"),
  );
  const repoStartAudit = commandEntries(
    repoProjection,
    "SubagentStart",
    "bee-codex-subagent-audit.mjs",
  );
  const repoStopAudit = commandEntries(
    repoProjection,
    "SubagentStop",
    "bee-codex-subagent-audit.mjs",
  );
  const transportOk =
    repoCommands.length === 13 &&
    noClaudeVars &&
    prefersVendoredBinary &&
    launchesSourceWrappers &&
    visibleFailOpen &&
    repoStartAudit.length === 1 &&
    repoStopAudit.length === 1;
  rows.push(
    catalogDriftRow(
      "codex-repo-target-generated-topology",
      transportOk,
      transportOk
        ? `all ${repoCommands.length} generated repo commands carry no Claude root variable, include paired audit events, prefer .bee/bin/bee[.exe] hook <name> --source=repo with the .bee/bin/hooks/bee-*.mjs wrapper as fallback, and fail open visibly`
        : `repo transport contract violated: commands=${repoCommands.length} (expected 13) ` +
            `noClaudeVars=${noClaudeVars} prefersVendoredBinary=${prefersVendoredBinary} ` +
            `launchesSourceWrappers=${launchesSourceWrappers} ` +
            `visibleFailOpen=${visibleFailOpen} startAudit=${repoStartAudit.length} stopAudit=${repoStopAudit.length}`,
    ),
  );

  // cell cnr2-7 (codex-native-runtime-v2 D4): the checked-in .codex/hooks.json
  // must byte-equal the SAME repo-target render proved above - rendered from
  // the catalog AT TEST TIME, never a stored snapshot, so any hand-edit or
  // stale regeneration of the file shows up here rather than only in the
  // generic route rows below.
  const repoText = renderProjectionText(RUNTIMES.CODEX, { target: TARGETS.REPO });
  const onDiskRepo = fs.readFileSync(CODEX_REPO_HOOKS_PATH, "utf8");
  const repoByteIdentical = repoText === onDiskRepo;
  rows.push(
    catalogDriftRow(
      "codex-repo-target-byte-identical",
      repoByteIdentical,
      repoByteIdentical
        ? "rendering the logical catalog for \"codex\" at target \"repo\" reproduces .codex/hooks.json byte-for-byte"
        : "DRIFT: rendering the logical catalog for \"codex\" at target \"repo\" does NOT reproduce .codex/hooks.json byte-for-byte",
    ),
  );

  // D4 (codex-native-runtime-v2): the state-sync PostToolUse matcher is a
  // SUPERSET, never a swap - update_plan joins TaskCreate/TaskUpdate/TodoWrite
  // in BOTH catalog projections. A text assertion, not merely "the row set
  // still runs", because a matcher regressed to a swap (or to only one
  // runtime) would otherwise pass every other check here silently.
  const EXPECTED_STATE_SYNC_MATCHER = "update_plan|TaskCreate|TaskUpdate|TodoWrite";
  function stateSyncMatcher(projection) {
    const group = (projection.hooks.PostToolUse || []).find((g) =>
      (g.hooks || []).some((h) => typeof h.command === "string" && h.command.includes("bee-state-sync.mjs")),
    );
    return group ? group.matcher : undefined;
  }
  const claudeStateSyncMatcher = stateSyncMatcher(claudeProjection);
  const codexStateSyncMatcher = stateSyncMatcher(codexProjection);
  const stateSyncSupersetOk =
    claudeStateSyncMatcher === EXPECTED_STATE_SYNC_MATCHER &&
    codexStateSyncMatcher === EXPECTED_STATE_SYNC_MATCHER;
  rows.push(
    catalogDriftRow(
      "state-sync-matcher-is-superset",
      stateSyncSupersetOk,
      stateSyncSupersetOk
        ? `both catalog projections carry the state-sync PostToolUse superset matcher "${EXPECTED_STATE_SYNC_MATCHER}"`
        : `DRIFT: expected "${EXPECTED_STATE_SYNC_MATCHER}" on both projections, ` +
            `got claude="${claudeStateSyncMatcher}" codex="${codexStateSyncMatcher}"`,
    ),
  );

  // cell cnr2-8 (codex-native-runtime-v2 D4): the Codex spawn guard now wires
  // bee-model-guard.mjs on BOTH runtimes, so its source (hooks/) and runtime
  // mirror (.bee/bin/hooks/) must stay byte-identical. The release manifest
  // inventories hooks/ but NOT .bee/bin/hooks/, so it would not catch a mirror
  // that drifted from source — this row pins the pair directly. (test_lib_mirror
  // derives the same pair from the projections, but this suite runs in the cell
  // verify and must fail on drift on its own.)
  const guardSource = path.join(HOOKS_DIR, "bee-model-guard.mjs");
  const guardMirror = path.join(REPO_ROOT, ".bee", "bin", "hooks", "bee-model-guard.mjs");
  const mirrorExists = fs.existsSync(guardMirror);
  const guardParity =
    mirrorExists &&
    fs.readFileSync(guardSource).equals(fs.readFileSync(guardMirror));
  rows.push(
    catalogDriftRow(
      "model-guard-source-mirror-byte-identical",
      guardParity,
      guardParity
        ? "hooks/bee-model-guard.mjs and .bee/bin/hooks/bee-model-guard.mjs are byte-identical"
        : mirrorExists
          ? "DRIFT: hooks/bee-model-guard.mjs and .bee/bin/hooks/bee-model-guard.mjs differ byte-for-byte (the Codex spawn branch must land in both)"
          : "MISSING: .bee/bin/hooks/bee-model-guard.mjs does not exist (the runtime mirror of the source guard)",
    ),
  );

  // cell codex-command-windows-1, tightened by cell 1710-6
  // (hardening-1-7-10 D6): the Codex hook schema's optional `commandWindows`
  // override (run with the session cwd as working dir, no `$SHELL -lc`)
  // must be present on every codex-repo-target entry. The pre-1710-6 bare
  // `node .bee/bin/hooks/<f> --source=repo` form assumed the session cwd IS
  // the repo root — from a NESTED cwd it threw MODULE_NOT_FOUND (see the
  // execution row below). The replacement is
  // a `node -e SCRIPT <script>` bootstrap that resolves the real git root
  // first (mirrors the POSIX command's own `git rev-parse
  // --show-toplevel`), so it is cwd-independent like the POSIX form.
  // NODE_WINDOWS_BOOTSTRAP pins the exact structural shape (require the node
  // builtins with SINGLE quotes, resolve the git root, prefer
  // .bee/bin/bee[.exe] `hook <name>` and fall back to the .bee/bin/hooks
  // wrapper, spawnSync with inherited stdio, exit with the child's status,
  // trailing hook NAME argv — rust-port R6 replaced the wrapper filename argv
  // with the name both arms derive from). This is the ONE hook surface where
  // Node cannot be feature-detected away: the same ban on `$`, `%` and
  // backtick that makes the string shell-agnostic also bans command
  // substitution, so the command itself cannot ask git for the repo root, and
  // the binary can only resolve its own root once something has launched it
  // from a root-dependent path. FORBIDDEN_WINDOWS_CHARS
  // independently asserts none of `$`, `%`, or a backtick appear ANYWHERE —
  // cmd.exe treats `%` as a variable sigil and PowerShell treats `$`/backtick
  // specially even inside double quotes, so any of the three breaks one of
  // the two target shells. commandWindows is codex-repo-only: the Codex
  // plugin manifest omits codex hooks (only .codex/hooks.json, the repo
  // target, ever loads), so neither the codex PLUGIN projection nor the
  // Claude projection may carry it.
  const repoCommandWindowsEntries = Object.values(repoProjection.hooks)
    .flat()
    .flatMap((g) => g.hooks);
  const NODE_WINDOWS_BOOTSTRAP =
    /^node -e "var cp=require\('child_process'\);var path=require\('path'\);var fs=require\('fs'\);var hook=process\.argv\[1\];var root='';try\{root=cp\.execSync\('git rev-parse --show-toplevel',\{stdio:\['ignore','pipe','ignore'\]\}\)\.toString\(\)\.trim\(\);\}catch\(e\)\{root='';\}if\(!root\)\{process\.exit\(0\);\}var bin=path\.join\(root,'\.bee','bin','bee\.exe'\);if\(!fs\.existsSync\(bin\)\)\{bin=path\.join\(root,'\.bee','bin','bee'\);\}var r=fs\.existsSync\(bin\)\?cp\.spawnSync\(bin,\['hook',hook,'--source=repo'\],\{stdio:'inherit'\}\):cp\.spawnSync\(process\.execPath,\[path\.join\(root,'\.bee','bin','hooks','bee-'\+hook\+'\.mjs'\),'--source=repo'\],\{stdio:'inherit'\}\);if\(r\.error\)\{process\.exit\(1\);\}process\.exit\(r\.status===null\?1:r\.status\);" [a-z-]+$/;
  const FORBIDDEN_WINDOWS_CHARS = /[$%`]/;
  const repoCommandWindowsOk =
    repoCommandWindowsEntries.length > 0 &&
    repoCommandWindowsEntries.every(
      (h) =>
        typeof h.commandWindows === "string" &&
        NODE_WINDOWS_BOOTSTRAP.test(h.commandWindows) &&
        !FORBIDDEN_WINDOWS_CHARS.test(h.commandWindows),
    );
  const claudeCommandWindowsAbsent = Object.values(claudeProjection.hooks)
    .flat()
    .flatMap((g) => g.hooks)
    .every((h) => h.commandWindows === undefined);
  const codexPluginCommandWindowsAbsent = Object.values(codexProjection.hooks)
    .flat()
    .flatMap((g) => g.hooks)
    .every((h) => h.commandWindows === undefined);
  const commandWindowsContractOk =
    repoCommandWindowsOk && claudeCommandWindowsAbsent && codexPluginCommandWindowsAbsent;
  rows.push(
    catalogDriftRow(
      "codex-repo-commandWindows-contract",
      commandWindowsContractOk,
      commandWindowsContractOk
        ? `all ${repoCommandWindowsEntries.length} codex-repo-target entries carry the git-root-resolving, binary-first "node -e SCRIPT <name>" commandWindows bootstrap (no $, %, or backtick anywhere); claude and codex-plugin entries carry none`
        : `DRIFT: commandWindows contract violated: repoOk=${repoCommandWindowsOk} ` +
            `claudeAbsent=${claudeCommandWindowsAbsent} codexPluginAbsent=${codexPluginCommandWindowsAbsent}`,
    ),
  );

  // cell 1710-6: EXECUTION test, not just a pattern match — actually run the
  // emitted commandWindows string with cwd set to a NESTED subdirectory of a
  // fixture repo. `node -e` is cross-platform, so this bootstrap (unlike the
  // POSIX `command` string, which needs `$SHELL -lc`) runs directly on Linux
  // CI too. Two arms: (1) inside the fixture repo but in a NESTED cwd, the
  // hook must actually execute (root resolved, hook prints its marker); (2)
  // outside any git repo entirely, it must exit 0 silently (no git root =
  // transport unavailable, POSIX parity). Uses a real script from
  // repoCommandWindowsEntries so this exercises the actual rendered catalog
  // output, not a hand-rebuilt copy of it.
  const commandWindowsExecutionRow = (() => {
    const sampleEntry = repoCommandWindowsEntries.find(
      (h) => typeof h.commandWindows === "string",
    );
    if (!sampleEntry) {
      return catalogDriftRow(
        "codex-commandWindows-nested-cwd-execution",
        false,
        "no codex-repo-target entry carries a commandWindows string to execute",
      );
    }
    const fixtureRoot = fs.realpathSync(
      fs.mkdtempSync(path.join(os.tmpdir(), "bee-commandWindows-nested-")),
    );
    const nonGitRoot = fs.realpathSync(
      fs.mkdtempSync(path.join(os.tmpdir(), "bee-commandWindows-nongit-")),
    );
    try {
      const nestedDir = path.join(fixtureRoot, "src", "deep", "nest");
      const hooksDir = path.join(fixtureRoot, ".bee", "bin", "hooks");
      fs.mkdirSync(nestedDir, { recursive: true });
      fs.mkdirSync(hooksDir, { recursive: true });
      const MARKER = "COMMAND_WINDOWS_EXECUTION_MARKER";
      const scriptName = sampleEntry.command.match(/bee-[a-z-]+\.mjs/)[0];
      fs.writeFileSync(
        path.join(hooksDir, scriptName),
        `process.stdout.write(${JSON.stringify(MARKER)} + "\\n");\nprocess.exit(0);\n`,
      );
      const gitInit = spawnSync("git", ["init", "-q", fixtureRoot], {
        encoding: "utf8",
        timeout: SPAWN_TIMEOUT_MS,
      });
      if (gitInit.status !== 0) {
        return catalogDriftRow(
          "codex-commandWindows-nested-cwd-execution",
          false,
          `fixture git init failed: status=${gitInit.status} ${gitInit.stderr || gitInit.error?.message || ""}`,
        );
      }
      const nestedResult = spawnSync(ROUTE_SHELL, ["-lc", sampleEntry.commandWindows], {
        cwd: nestedDir,
        encoding: "utf8",
        input: "",
        timeout: SPAWN_TIMEOUT_MS,
        maxBuffer: SPAWN_MAX_BUFFER,
      });
      const nonGitResult = spawnSync(ROUTE_SHELL, ["-lc", sampleEntry.commandWindows], {
        cwd: nonGitRoot,
        encoding: "utf8",
        input: "",
        timeout: SPAWN_TIMEOUT_MS,
        maxBuffer: SPAWN_MAX_BUFFER,
      });
      const nestedOk =
        nestedResult.status === 0 && (nestedResult.stdout || "").includes(MARKER);
      const nonGitOk =
        nonGitResult.status === 0 &&
        !(nonGitResult.stdout || "").trim() &&
        !(nonGitResult.stdout || "").includes(MARKER);
      // rust-port R6 third arm: with a vendored binary present the bootstrap
      // must take the BINARY, never the wrapper. Proven by planting a stand-in
      // binary that prints its own marker — the wrapper's MARKER must not
      // appear. Skipped (not failed) where a stand-in cannot be made
      // executable, so this row never turns red on a platform quirk.
      const BIN_MARKER = "COMMAND_WINDOWS_BINARY_ARM_MARKER";
      let binaryArm = "skipped";
      let binaryResult = null;
      // On POSIX a stand-in shell script IS an executable, so the arm can be
      // proven with a marker of its own. On win32 only a real PE image can be
      // spawned, so the arm borrows this repo's own built .bee/bin/bee.exe
      // when it exists (machine-local by decision 1f4262ca — absent on a
      // fresh clone, hence a skip rather than a failure) and asserts the
      // wrapper MARKER never appears.
      const vendoredBinary = path.join(REPO_ROOT, ".bee", "bin", "bee.exe");
      const planted =
        process.platform === "win32"
          ? fs.existsSync(vendoredBinary)
            ? (() => {
                const p = path.join(fixtureRoot, ".bee", "bin", "bee.exe");
                fs.copyFileSync(vendoredBinary, p);
                return { path: p, expectMarker: null };
              })()
            : null
          : (() => {
              const p = path.join(fixtureRoot, ".bee", "bin", "bee");
              fs.writeFileSync(p, `#!/bin/sh\nprintf '%s\\n' "${BIN_MARKER}"\nexit 0\n`);
              fs.chmodSync(p, 0o755);
              return { path: p, expectMarker: BIN_MARKER };
            })();
      if (planted) {
        binaryResult = spawnSync(ROUTE_SHELL, ["-lc", sampleEntry.commandWindows], {
          cwd: nestedDir,
          encoding: "utf8",
          input: "",
          timeout: SPAWN_TIMEOUT_MS,
          maxBuffer: SPAWN_MAX_BUFFER,
        });
        const stdout = binaryResult.stdout || "";
        binaryArm =
          binaryResult.status === 0 &&
          !stdout.includes(MARKER) &&
          (planted.expectMarker === null || stdout.includes(planted.expectMarker))
            ? "ok"
            : "failed";
        fs.rmSync(planted.path, { force: true });
      }
      const pass = nestedOk && nonGitOk && binaryArm !== "failed";
      return catalogDriftRow(
        "codex-commandWindows-nested-cwd-execution",
        pass,
        pass
          ? `commandWindows bootstrap executed from a NESTED fixture cwd resolves the git root and runs the real hook; outside any git repo it exits 0 silently; binary-preference arm: ${binaryArm}`
          : `commandWindows execution failed: nested(status=${nestedResult.status}, stdout=${truncate(nestedResult.stdout, 300)}, stderr=${truncate(nestedResult.stderr, 300)}) ` +
              `nonGit(status=${nonGitResult.status}, stdout=${truncate(nonGitResult.stdout, 300)}, stderr=${truncate(nonGitResult.stderr, 300)}) ` +
              `binaryArm=${binaryArm}(status=${binaryResult?.status}, stdout=${truncate(binaryResult?.stdout, 300)})`,
      );
    } finally {
      fs.rmSync(fixtureRoot, { recursive: true, force: true });
      fs.rmSync(nonGitRoot, { recursive: true, force: true });
    }
  })();
  rows.push(commandWindowsExecutionRow);

  return rows;
}

// --- isolated-CODEX_HOME codex-acceptance row (cell codex-parity-2) -------
//
// plan.md Safety-foundation exit / validation-safety-foundation.md B4: in an
// isolated CODEX_HOME fixture (no auth needed for marketplace/list verbs),
// `codex plugin marketplace add <repo-root>` and
// `codex plugin list --available --json` must accept this repo's manifest
// and list plugin bee@bee cleanly, with the default hooks/hooks.json route
// present (.codex-plugin/plugin.json carries no "hooks" override, or one
// that already points at hooks/hooks.json — approach.md section 1: "Make
// hooks/hooks.json the Codex default projection so the Codex manifest can
// omit a redundant hooks field."). If the codex CLI is absent, this row
// SKIPS loudly by name — it never silently disappears from the output.

function detectCodexCli() {
  const result = spawnSync("codex", ["--version"], { encoding: "utf8", timeout: 10000 });
  if (result.status !== 0) {
    return {
      present: false,
      reason:
        `exit status=${result.status} stderr=${truncate(result.stderr, 300)}` +
        (result.status == null && result.error ? ` launch error=${result.error.message}` : ""),
    };
  }
  const version = typeof result.stdout === "string" ? result.stdout.trim() : "";
  if (!version) return { present: false, reason: "exit 0 but version stdout was empty" };
  return { present: true, version };
}

function codexAcceptanceRow(id, pass, note, extra = {}) {
  return {
    wrapper: "codex-acceptance",
    id,
    status: extra.status ?? null,
    signal: extra.signal ?? null,
    stdout: extra.stdout ?? "",
    stderr: extra.stderr ?? "",
    pass,
    skip: Boolean(extra.skip),
    note,
  };
}

function runCodexAcceptanceRows() {
  const detect = detectCodexCli();
  if (!detect.present) {
    return [
      codexAcceptanceRow(
        "codex-plugin-marketplace-accept",
        true,
        `SKIP: codex CLI not found on PATH (${detect.reason}) — isolated CODEX_HOME ` +
          "marketplace-add / plugin-list acceptance row cannot run in this environment",
        { skip: true },
      ),
    ];
  }

  const codexHome = fs.mkdtempSync(path.join(os.tmpdir(), "codex-home-acceptance-"));
  const env = { ...process.env, CODEX_HOME: codexHome };
  const rows = [];
  try {
    const addResult = spawnSync(
      "codex",
      ["plugin", "marketplace", "add", REPO_ROOT, "--json"],
      { encoding: "utf8", env, timeout: 30000 },
    );
    let addJson = null;
    try {
      addJson = JSON.parse((addResult.stdout || "").trim());
    } catch {
      addJson = null;
    }
    const addAccepted = Boolean(
      addResult.status === 0 && addJson && addJson.installedRoot === REPO_ROOT,
    );
    rows.push(
      codexAcceptanceRow(
        "codex-plugin-marketplace-add",
        addAccepted,
        addAccepted
          ? `manifest accepted (marketplace "${addJson.marketplaceName}", root ${addJson.installedRoot})`
          : "codex plugin marketplace add did not accept this repo's manifest: " +
              `status=${addResult.status} stdout=${truncate(addResult.stdout, 500)} stderr=${truncate(addResult.stderr, 500)}`,
        { status: addResult.status, signal: addResult.signal, stdout: addResult.stdout, stderr: addResult.stderr },
      ),
    );
    if (!addAccepted) return rows;

    const listResult = spawnSync(
      "codex",
      ["plugin", "list", "--available", "--json"],
      { encoding: "utf8", env, timeout: 30000 },
    );
    let listJson = null;
    try {
      listJson = JSON.parse((listResult.stdout || "").trim());
    } catch {
      listJson = null;
    }
    const available = listJson && Array.isArray(listJson.available) ? listJson.available : [];
    const beePlugin = available.find((p) => p && (p.pluginId === "bee@bee" || p.name === "bee"));
    const listedCleanly = Boolean(listResult.status === 0 && beePlugin);
    rows.push(
      codexAcceptanceRow(
        "codex-plugin-list-available",
        listedCleanly,
        listedCleanly
          ? `plugin "${beePlugin.pluginId || beePlugin.name}" listed cleanly from marketplace "${beePlugin.marketplaceName}"`
          : "bee plugin not found in \"codex plugin list --available --json\" output: " +
              `status=${listResult.status} stdout=${truncate(listResult.stdout, 500)} stderr=${truncate(listResult.stderr, 500)}`,
        { status: listResult.status, signal: listResult.signal, stdout: listResult.stdout, stderr: listResult.stderr },
      ),
    );

    const codexManifest = JSON.parse(fs.readFileSync(CODEX_PLUGIN_MANIFEST_PATH, "utf8"));
    const hooksOverride = codexManifest.hooks;
    // packages-restructure cell 2: hooks/hooks.json moved to
    // packages/bee/hooks/hooks.json, off Codex's undocumented plugin-root
    // default lookup path (catalog.mjs header comment). An explicit "hooks"
    // field naming the new canonical path is now the ONLY proven route — an
    // absent field (the old implicit-default state) or the stale pre-move
    // literal no longer names a file that exists, so neither proves anything.
    const CANONICAL_HOOKS_OVERRIDE = "./packages/bee/hooks/hooks.json";
    const usesDefaultRoute = hooksOverride === CANONICAL_HOOKS_OVERRIDE;
    const defaultHooksExists = fs.existsSync(CODEX_DEFAULT_HOOKS_PATH);
    const routeProven = usesDefaultRoute && defaultHooksExists;
    rows.push(
      codexAcceptanceRow(
        "codex-default-hooks-route",
        routeProven,
        routeProven
          ? `.codex-plugin/plugin.json carries the explicit canonical hooks override (${CANONICAL_HOOKS_OVERRIDE}) and that file exists at plugin root`
          : `canonical hooks.json route not proven: hooksOverride=${JSON.stringify(hooksOverride)} defaultHooksExists=${defaultHooksExists}`,
      ),
    );

    return rows;
  } finally {
    fs.rmSync(codexHome, { recursive: true, force: true });
  }
}

// --- adapter contract rows (cell codex-parity-3) ---------------------------
//
// ADDED row groups only — the cell-1 seven-wrapper fixture table above is
// untouched and these groups run in DEFAULT mode only, so --baseline keeps
// characterizing exactly the original table (cell codex-parity-1's contract)
// and --catalog-only keeps cell codex-parity-2's contract.
//
// Group "chain-nudge-nickname": a worker registered by the state CLI
// (bee.mjs state worker add --nickname N — discovery.md Proved Gaps: the CLI
// stores `nickname` while chain-nudge previously read name|agent|worker) must
// be matched by bee-chain-nudge, the generic fallback keys must keep working,
// and an unregistered agent must stay silent.
//
// Group "coverage-gap": D2's "unsupported paths fail open with visible
// limits" — each adapter coverage-gap class lands a visible line in
// .bee/logs/hooks.jsonl (never silently), and a log-write failure NEVER flips
// the computed allow/deny result in either direction.

function adapterRow(group, id, pass, note, extra = {}) {
  return {
    wrapper: group,
    id,
    status: extra.status ?? null,
    signal: extra.signal ?? null,
    stdout: extra.stdout ?? "",
    stderr: extra.stderr ?? "",
    pass,
    note,
  };
}

function readHooksJsonl(fixtureRoot) {
  const file = path.join(fixtureRoot, ".bee", "logs", "hooks.jsonl");
  if (!fs.existsSync(file)) return [];
  return fs
    .readFileSync(file, "utf8")
    .split(/\r?\n/)
    .filter((l) => l.trim())
    .map((l) => {
      try {
        return JSON.parse(l);
      } catch {
        return null;
      }
    })
    .filter(Boolean);
}

// bee-tools-logger.mjs's own log (W4, AO15). Same shape/discipline as
// readHooksJsonl above — malformed lines are dropped rather than thrown.
function readToolsJsonl(fixtureRoot) {
  const file = path.join(fixtureRoot, ".bee", "logs", "tools.jsonl");
  if (!fs.existsSync(file)) return [];
  return fs
    .readFileSync(file, "utf8")
    .split(/\r?\n/)
    .filter((l) => l.trim())
    .map((l) => {
      try {
        return { raw: l, parsed: JSON.parse(l) };
      } catch {
        return null;
      }
    })
    .filter(Boolean);
}

function findGapLine(fixtureRoot, hook, gap) {
  return readHooksJsonl(fixtureRoot).find(
    (line) => line && line.hook === hook && line.event === "coverage-gap" && line.gap === gap,
  );
}

async function runCodexSubagentAuditRows() {
  const rows = [];
  const root = buildFixture("hook-contracts-codex-subagent-audit-");
  const forbiddenSentinel = "FORBIDDEN_PROMPT_TRANSCRIPT_ENV_SECRET_7f3a";
  const longAgentName = "agent-" + "x".repeat(180);

  const start = await runWrapper(
    "bee-codex-subagent-audit.mjs",
    JSON.stringify({
      hook_event_name: "SubagentStart",
      cwd: root,
      session_id: "session-audit",
      agent_id: "agent-audit",
      agent_name: longAgentName,
      agent_type: "worker",
      prompt: forbiddenSentinel,
      transcript: forbiddenSentinel,
      environment: { SECRET: forbiddenSentinel },
      credentials: forbiddenSentinel,
    }),
    root,
    ["--source=plugin"],
  );
  const stop = await runWrapper(
    "bee-codex-subagent-audit.mjs",
    JSON.stringify({
      hook_event_name: "SubagentStop",
      cwd: root,
      session_id: "session-audit",
      agent_id: "agent-audit",
      agent_name: longAgentName,
      agent_type: "worker",
      prompt: forbiddenSentinel,
      transcript_path: forbiddenSentinel,
      env: forbiddenSentinel,
      secret: forbiddenSentinel,
    }),
    root,
    ["--source=plugin"],
  );
  const auditLines = readHooksJsonl(root).filter(
    (line) => line.hook === "codex-subagent-audit" && line.event === "subagent-audit",
  );
  const startLine = auditLines.find((line) => line.lifecycle === "start");
  const stopLine = auditLines.find((line) => line.lifecycle === "stop");
  const logBytes = fs.readFileSync(path.join(root, ".bee", "logs", "hooks.jsonl"), "utf8");
  const boundedAndAllowlisted = Boolean(
    startLine &&
      stopLine &&
      startLine.authority === "audit-only" &&
      startLine.timing === "post-start" &&
      startLine.source === "plugin" &&
      stopLine.authority === "audit-only" &&
      stopLine.source === "plugin" &&
      typeof startLine.agent_name === "string" &&
      startLine.agent_name.length <= 123 &&
      !logBytes.includes(forbiddenSentinel) &&
      !("prompt" in startLine) &&
      !("transcript" in startLine) &&
      !("environment" in startLine),
  );
  rows.push(
    adapterRow(
      "codex-subagent-audit",
      "paired-bounded-audit",
      start.status === 0 &&
        stop.status === 0 &&
        start.stdout === "" &&
        stop.stdout === "" &&
        start.stderr === "" &&
        stop.stderr === "" &&
        auditLines.length === 2 &&
        boundedAndAllowlisted,
      boundedAndAllowlisted
        ? "start/stop write two silent audit-only records with bounded allowlisted identifiers; prompt/transcript/env/secret content is absent"
        : `unexpected audit contract: start=${start.status} stop=${stop.status} lines=${JSON.stringify(auditLines)}`,
    ),
  );

  const malformedRoot = buildFixture("hook-contracts-codex-subagent-malformed-");
  const malformed = await runWrapper(
    "bee-codex-subagent-audit.mjs",
    "null",
    malformedRoot,
    ["--source=plugin"],
  );
  const malformedGap = findGapLine(
    malformedRoot,
    "codex-subagent-audit",
    "malformed-payload",
  );
  const unsupportedGap = findGapLine(
    malformedRoot,
    "codex-subagent-audit",
    "unsupported-subagent-event",
  );
  const malformedAuditLines = readHooksJsonl(malformedRoot).filter(
    (line) => line.event === "subagent-audit",
  );
  rows.push(
    adapterRow(
      "codex-subagent-audit",
      "malformed-input-fails-open",
      malformed.status === 0 &&
        malformed.stdout === "" &&
        malformed.stderr === "" &&
        Boolean(malformedGap) &&
        Boolean(unsupportedGap) &&
        malformedAuditLines.length === 0,
      malformed.status === 0 && malformedGap && unsupportedGap
        ? "top-level null fails open, records bounded coverage gaps, and fabricates no lifecycle audit"
        : `malformed contract failed: status=${malformed.status} gaps=${JSON.stringify(readHooksJsonl(malformedRoot))}`,
    ),
  );

  const logFailRoot = buildFixture("hook-contracts-codex-subagent-logfail-");
  fs.writeFileSync(path.join(logFailRoot, ".bee", "logs"), "not a directory\n");
  const logFail = await runWrapper(
    "bee-codex-subagent-audit.mjs",
    JSON.stringify({
      hook_event_name: "SubagentStart",
      cwd: logFailRoot,
      agent_id: "agent-logfail",
    }),
    logFailRoot,
  );
  rows.push(
    adapterRow(
      "codex-subagent-audit",
      "audit-write-failure-never-gates",
      logFail.status === 0 && logFail.stdout === "" && logFail.stderr === "",
      logFail.status === 0 && logFail.stdout === "" && logFail.stderr === ""
        ? "an unwritable audit log stays silent and never denies, blocks, or changes exit success"
        : `audit failure changed behavior: status=${logFail.status} stdout=${truncate(logFail.stdout, 200)} stderr=${truncate(logFail.stderr, 200)}`,
    ),
  );

  return rows;
}

async function runNicknameRows() {
  const rows = [];
  // Dedicated fixture: a NON-swarming, non-reviewing phase (planning), so
  // the nudge can only fire through registered-worker matching — never
  // through the phase==="swarming" shortcut the shared fixture uses.
  // (validation-diet D3/D4: was "validating" pre-cut, now retired outright —
  // any non-swarming/non-reviewing phase proves the same "not the shortcut"
  // shape, and this hook does not gate on GATED_PHASES at all.)
  const root = buildFixture("hook-contracts-nickname-");
  fs.writeFileSync(
    path.join(root, ".bee", "state.json"),
    `${JSON.stringify(
      {
        phase: "planning",
        mode: "standard",
        feature: "demo",
        approved_gates: { context: true, shape: true, execution: false, review: false },
        workers: [
          // exactly what `bee.mjs state worker add --nickname kevin --cell demo-1` stores
          { nickname: "kevin", cell: "demo-1", tier: "generation", status: "working" },
          // a foreign/legacy entry shape: the generic fallback must still match
          { name: "legacy-worker" },
        ],
      },
      null,
      2,
    )}\n`,
  );

  const subagentStop = (agent) =>
    JSON.stringify({ hook_event_name: "SubagentStop", agent_name: agent, cwd: root });

  const r1 = await runWrapper("bee-chain-nudge.mjs", subagentStop("kevin"), root);
  const p1 = parseAdvisoryStdout(r1.stdout);
  const r1pass = Boolean(
    r1.status === 0 &&
      p1.json &&
      typeof p1.json.systemMessage === "string" &&
      p1.json.systemMessage.includes('Worker "kevin"') &&
      p1.json.decision !== "block",
  );
  rows.push(
    adapterRow(
      "chain-nudge-nickname",
      "registered-nickname-matched",
      r1pass,
      r1pass
        ? 'worker registered under `nickname` ("kevin") is matched and nudged via JSON systemMessage'
        : `expected a JSON systemMessage naming Worker "kevin"; got status=${r1.status} stdout=${truncate(r1.stdout, 300)}`,
      r1,
    ),
  );

  const r2 = await runWrapper("bee-chain-nudge.mjs", subagentStop("legacy-worker"), root);
  const p2 = parseAdvisoryStdout(r2.stdout);
  const r2pass = Boolean(
    r2.status === 0 &&
      p2.json &&
      typeof p2.json.systemMessage === "string" &&
      p2.json.systemMessage.includes('Worker "legacy-worker"'),
  );
  rows.push(
    adapterRow(
      "chain-nudge-nickname",
      "generic-fallback-still-works",
      r2pass,
      r2pass
        ? "legacy entry registered under `name` still matches through the generic fallback"
        : `expected a JSON systemMessage naming Worker "legacy-worker"; got status=${r2.status} stdout=${truncate(r2.stdout, 300)}`,
      r2,
    ),
  );

  const r3 = await runWrapper("bee-chain-nudge.mjs", subagentStop("stranger"), root);
  const r3pass = r3.status === 0 && !(r3.stdout || "").trim();
  rows.push(
    adapterRow(
      "chain-nudge-nickname",
      "unregistered-agent-stays-silent",
      r3pass,
      r3pass
        ? "an agent not registered under any key stays silent (control: the match is real)"
        : `expected silence for an unregistered agent; got status=${r3.status} stdout=${truncate(r3.stdout, 300)}`,
      r3,
    ),
  );

  return rows;
}

// fresh-session-handoff fsh-6 (D4): bee-chain-nudge and bee-session-close
// thread payload.session_id into resolvePipeline (via the vendored .bee/bin
// lib copied by copyLib above) so the phase they consult comes from the
// ACTING SESSION'S lane when bound, the default record otherwise — never
// touching bee-session-init.mjs (S4's SessionStart threading stays out of
// scope here). Writes lane/session records directly as JSON, mirroring this
// file's existing buildFixture style rather than importing lib/state.mjs or
// lib/claims.mjs (this harness executes wrappers as isolated Workers and
// never imports the lib it is testing).
function writeLaneFile(root, feature, extra = {}) {
  const lanesDir = path.join(root, ".bee", "lanes");
  fs.mkdirSync(lanesDir, { recursive: true });
  fs.writeFileSync(
    path.join(lanesDir, `${feature}.json`),
    `${JSON.stringify(
      {
        schema_version: "1.0",
        feature,
        mode: null,
        phase: "idle",
        approved_gates: { context: false, shape: false, execution: false, review: false },
        summary: "",
        next_action: "",
        created_at: new Date().toISOString(),
        ...extra,
      },
      null,
      2,
    )}\n`,
  );
}

function writeSessionFile(root, sessionId, { lane } = {}) {
  const sessionsDir = path.join(root, ".bee", "sessions");
  fs.mkdirSync(sessionsDir, { recursive: true });
  const record = {
    id: sessionId,
    started_at: new Date().toISOString(),
    last_heartbeat: new Date().toISOString(),
    ...(lane ? { lane } : {}),
  };
  fs.writeFileSync(path.join(sessionsDir, `${sessionId}.json`), `${JSON.stringify(record, null, 2)}\n`);
}

async function runLaneSessionRows() {
  const rows = [];

  // ── bee-chain-nudge: a bound session's SWARMING lane fires the nudge even
  // though the DEFAULT state.json sits at "idle" (no registered worker,
  // phase !== swarming/reviewing at default) — proving phase resolution
  // came from the session's lane, not the default record.
  const chainRoot = buildFixture("hook-contracts-lane-chain-");
  fs.writeFileSync(
    path.join(chainRoot, ".bee", "state.json"),
    `${JSON.stringify(
      { phase: "idle", mode: null, feature: null, approved_gates: { context: false, shape: false, execution: false, review: false }, workers: [] },
      null,
      2,
    )}\n`,
  );
  writeLaneFile(chainRoot, "lane-swarm", {
    phase: "swarming",
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });
  writeSessionFile(chainRoot, "sess-bound", { lane: "lane-swarm" });
  writeSessionFile(chainRoot, "sess-unbound");

  const subagentStopWithSession = (sessionId) =>
    JSON.stringify({ hook_event_name: "SubagentStop", session_id: sessionId, cwd: chainRoot });

  const cControl = await runWrapper("bee-chain-nudge.mjs", JSON.stringify({ hook_event_name: "SubagentStop", cwd: chainRoot }), chainRoot);
  const cControlPass = cControl.status === 0 && !(cControl.stdout || "").trim();
  rows.push(
    adapterRow(
      "chain-nudge-lane",
      "no-session-id-stays-on-default-idle",
      cControlPass,
      cControlPass
        ? "no session_id: default record (idle, no registered worker) stays silent, as a control"
        : `expected silence with the default idle record; got status=${cControl.status} stdout=${truncate(cControl.stdout, 300)}`,
      cControl,
    ),
  );

  const cUnbound = await runWrapper("bee-chain-nudge.mjs", subagentStopWithSession("sess-unbound"), chainRoot);
  const cUnboundPass = cUnbound.status === 0 && !(cUnbound.stdout || "").trim();
  rows.push(
    adapterRow(
      "chain-nudge-lane",
      "unbound-session-keeps-default",
      cUnboundPass,
      cUnboundPass
        ? "a session with no lane binding resolves to the default record (idle) — stays silent"
        : `expected silence for an unbound session; got status=${cUnbound.status} stdout=${truncate(cUnbound.stdout, 300)}`,
      cUnbound,
    ),
  );

  const cBound = await runWrapper("bee-chain-nudge.mjs", subagentStopWithSession("sess-bound"), chainRoot);
  const cBoundParsed = parseAdvisoryStdout(cBound.stdout);
  const cBoundPass = Boolean(
    cBound.status === 0 &&
      cBoundParsed.json &&
      typeof cBoundParsed.json.systemMessage === "string" &&
      cBoundParsed.json.systemMessage.includes("[STATUS]") &&
      cBoundParsed.json.decision !== "block",
  );
  rows.push(
    adapterRow(
      "chain-nudge-lane",
      "bound-session-lane-swarming-fires-nudge",
      cBoundPass,
      cBoundPass
        ? "a session bound to a SWARMING lane fires the chain-nudge even though the default record sits at idle"
        : `expected a JSON systemMessage nudge from the bound lane's swarming phase; got status=${cBound.status} stdout=${truncate(cBound.stdout, 300)}`,
      cBound,
    ),
  );

  // ── bee-session-close: a bound session's IDLE lane suppresses the "hive
  // door open" warning even though the DEFAULT state.json sits at
  // "swarming" with no HANDOFF (which fires the warning today) — proving
  // phase resolution came from the session's lane, not the default record.
  const closeRoot = buildFixture("hook-contracts-lane-close-"); // default: swarming, no HANDOFF
  writeLaneFile(closeRoot, "lane-idle", { phase: "idle" });
  writeSessionFile(closeRoot, "sess-close-bound", { lane: "lane-idle" });
  writeSessionFile(closeRoot, "sess-close-unbound");

  const stopWithSession = (sessionId) =>
    JSON.stringify({ hook_event_name: "Stop", session_id: sessionId, cwd: closeRoot });

  const sControl = await runWrapper("bee-session-close.mjs", JSON.stringify({ hook_event_name: "Stop", cwd: closeRoot }), closeRoot);
  const sControlParsed = parseAdvisoryStdout(sControl.stdout);
  const sControlPass = Boolean(
    sControl.status === 0 &&
      sControlParsed.json &&
      typeof sControlParsed.json.systemMessage === "string" &&
      sControlParsed.json.systemMessage.includes("hive door open"),
  );
  rows.push(
    adapterRow(
      "session-close-lane",
      "no-session-id-keeps-default-swarming-warning",
      sControlPass,
      sControlPass
        ? "no session_id: the default record (swarming, no HANDOFF) fires the hive-door-open warning, as a control"
        : `expected the hive-door-open warning from the default swarming record; got status=${sControl.status} stdout=${truncate(sControl.stdout, 300)}`,
      sControl,
    ),
  );

  const sUnbound = await runWrapper("bee-session-close.mjs", stopWithSession("sess-close-unbound"), closeRoot);
  const sUnboundParsed = parseAdvisoryStdout(sUnbound.stdout);
  const sUnboundPass = Boolean(
    sUnbound.status === 0 &&
      sUnboundParsed.json &&
      typeof sUnboundParsed.json.systemMessage === "string" &&
      sUnboundParsed.json.systemMessage.includes("hive door open"),
  );
  rows.push(
    adapterRow(
      "session-close-lane",
      "unbound-session-keeps-default",
      sUnboundPass,
      sUnboundPass
        ? "a session with no lane binding resolves to the default record (swarming) — warning still fires"
        : `expected the hive-door-open warning for an unbound session; got status=${sUnbound.status} stdout=${truncate(sUnbound.stdout, 300)}`,
      sUnbound,
    ),
  );

  const sBound = await runWrapper("bee-session-close.mjs", stopWithSession("sess-close-bound"), closeRoot);
  const sBoundPass = sBound.status === 0 && !(sBound.stdout || "").trim();
  rows.push(
    adapterRow(
      "session-close-lane",
      "bound-session-lane-idle-suppresses-default-warning",
      sBoundPass,
      sBoundPass
        ? "a session bound to an IDLE lane suppresses the hive-door-open warning even though the default record sits at swarming"
        : `expected silence from the bound lane's idle phase; got status=${sBound.status} stdout=${truncate(sBound.stdout, 300)}`,
      sBound,
    ),
  );

  // ── bee-prompt-context (P0, codex-loop-p0): the coverage gap that let the bug
  // survive. bee-chain-nudge and bee-session-close were session-threaded and
  // tested; bee-prompt-context was NOT — it called buildPromptReminder(root)
  // without the sessionId, so a bound session read the DEFAULT record's phase.
  // This row proves the reminder now reflects the ACTING SESSION'S lane: a bound
  // session on a swarming lane must see phase=swarming, while the default state
  // sits at idle. A regression to the sessionId-less call fails this row.
  const promptRoot = buildFixture("hook-contracts-lane-prompt-");
  fs.writeFileSync(
    path.join(promptRoot, ".bee", "state.json"),
    `${JSON.stringify(
      { phase: "idle", mode: null, feature: null, approved_gates: { context: false, shape: false, execution: false, review: false }, workers: [] },
      null,
      2,
    )}\n`,
  );
  writeLaneFile(promptRoot, "lane-prompt-swarm", {
    phase: "swarming",
    mode: "standard",
    approved_gates: { context: true, shape: true, execution: true, review: false },
    next_action: "bound-lane next action",
  });
  writeSessionFile(promptRoot, "sess-prompt-bound", { lane: "lane-prompt-swarm" });

  const promptWithSession = (sessionId) =>
    JSON.stringify({ hook_event_name: "UserPromptSubmit", prompt: "hi", session_id: sessionId, cwd: promptRoot });

  const pBound = await runWrapper("bee-prompt-context.mjs", promptWithSession("sess-prompt-bound"), promptRoot);
  const pBoundOut = `${pBound.stdout || ""}`;
  const pBoundPass = pBound.status === 0 && /phase=swarming/.test(pBoundOut) && !/phase=idle/.test(pBoundOut);
  rows.push(
    adapterRow(
      "prompt-context-lane",
      "bound-session-reminder-reflects-its-own-lane-not-the-default",
      pBoundPass,
      pBoundPass
        ? "the reminder for a bound session reports its lane's phase=swarming, not the default record's idle — sessionId is threaded (P0 fix)"
        : `expected the bound lane's phase=swarming in the reminder, got status=${pBound.status} stdout=${truncate(pBoundOut, 300)}`,
      pBound,
    ),
  );

  // and the on-demand review gate never surfaces as pending here: the bound lane
  // has gates 1-3 approved and is not `reviewing`, so a correct reminder is
  // silent on gates (the false "gate pending: review" every turn is gone).
  rows.push(
    adapterRow(
      "prompt-context-lane",
      "no-false-review-gate-outside-a-review-session",
      pBound.status === 0 && !/gate pending: review/.test(pBoundOut),
      !/gate pending: review/.test(pBoundOut)
        ? "the reminder does not print the on-demand review gate as pending outside a review session (P0 fix)"
        : `the reminder still prints a false review gate: ${truncate(pBoundOut, 200)}`,
      pBound,
    ),
  );

  return rows;
}

// fresh-session-handoff fsh-8 (D3/D4): bee-write-guard threads
// payload.session_id into guards.checkWrite's optional sessionId argument
// (fsh-5's contract, fsh-7's hold-deny + corrupt-store implementation).
//
// multisession-native-16: `.bee/reservations.json` is now a rebuildable
// PROJECTION over lease-store.mjs (see reservations.mjs's own module
// header) — the hook's session-hold conflict check (findSessionConflicts)
// reads lease-store's sharded per-resource files directly, so a fixture
// reservation must be seeded as a real lease file. seedReservationLease
// below is a raw fs write of exactly the record shape/path lease-store.mjs's
// acquireLeases would produce for a `type:'path'` request (resource key
// `path:<canonicalPath>`, sha256-hashed filename under
// `.bee/runtime/leases/paths/`) — a deliberate hand-rolled duplicate rather
// than importing lease-store.mjs, mirroring this file's existing
// writeSessionFile/writeLaneFile direct-JSON style (this harness executes
// wrappers as isolated Workers and never imports the lib under test).
function seedReservationLease(root, { path: reservedPath, agent, cell, session = null, ttlSeconds = 3600, reservedAt = new Date().toISOString() }) {
  const canonical = String(reservedPath || "")
    .replace(/\\/g, "/")
    .replace(/^\.\/+/, "")
    .replace(/\/+$/, "");
  const resourceKey = `path:${canonical}`;
  const hash = createHash("sha256").update(resourceKey).digest("hex");
  const dir = path.join(root, ".bee", "runtime", "leases", "paths");
  fs.mkdirSync(dir, { recursive: true });
  const acquiredMs = Date.parse(reservedAt);
  const record = {
    resource: resourceKey,
    mode: "write",
    workflow_id: cell,
    // A control-char-wrapped sentinel (never a real session id, which always
    // comes from claims.mjs) — mirrors reservations.mjs's own
    // SESSIONLESS_SESSION_ID so an omitted `session` here reproduces a
    // genuinely session-less/"legacy" row through the shim, exactly as
    // before this cell.
    session_id: session || "\u0000bee-reservation-sessionless\u0000",
    workspace_id: `agent:${agent}`,
    epoch: 0,
    acquired_at: reservedAt,
    expires_at: Number.isFinite(ttlSeconds) && ttlSeconds > 0 ? new Date(acquiredMs + ttlSeconds * 1000).toISOString() : null,
    kind: "lease",
  };
  fs.writeFileSync(path.join(dir, `${hash}.json`), `${JSON.stringify(record, null, 2)}\n`);
}

// A PRESENT but unparseable store (panel B1 / C7): must fail closed through
// the real hook (exit 2), never silently read as empty.
function writeCorruptReservationsFile(root) {
  const dir = path.join(root, ".bee");
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(path.join(dir, "reservations.json"), "{ this is not valid json");
}

function editPayload({ sessionId, cwd, filePath }) {
  return JSON.stringify({
    hook_event_name: "PreToolUse",
    tool_name: "Edit",
    tool_input: { file_path: filePath },
    ...(sessionId ? { session_id: sessionId } : {}),
    cwd,
  });
}

// okf-integration-close-f4 f4-6: bee-session-close's decision-0003 CAPTURE
// NUDGE names the RESOLVED state layer, not a hardcoded `docs/specs/`.
//
// This is the session-CLOSE twin of the session-INIT fix in f4-3. The nudge's
// LOGIC has been bundle-aware since okf-foundation D34 (it maxes the retired
// tree's newest .md against the bundle's), so in a migrated repo it fires
// precisely BECAUSE the bundle is stale — and the wording then sent the agent
// to merge the settlement into "the touched area's spec", under a tree that
// `scripts/okf_specs_fence.mjs` fails the chain for accepting new content. An
// agent obeying the nudge would have been stopped by another guard.
//
// Both branches are asserted end-to-end through the real wrapper, on two
// fixtures identical except for the presence of ONE parsing concept, because
// that one file is the whole predicate (`bundleMode`). The no-bundle branch is
// pinned to its EXACT pre-change wording: a never-migrated host must not be
// able to tell this shipped.
const CAPTURE_NUDGE_NO_BUNDLE =
  "bee capture nudge (decision 0003): the newest decision is more recent than every " +
  "area spec under docs/specs/ — a settled outcome may exist only in the decision log " +
  "and the chat. Before finishing, invoke bee-capturing capture to merge it into the " +
  "touched area's spec (or confirm no spec is affected).";

function buildCaptureNudgeFixture(label, { withBundle }) {
  const root = buildFixture(`hook-contracts-capture-nudge-${label}-`);
  // A settled decision, dated far enough ahead that it is unambiguously newer
  // than every doc mtime the fixture just wrote (the nudge compares a decision
  // DATE against file mtimes, so "now" would be a coin flip).
  fs.writeFileSync(
    path.join(root, ".bee", "decisions.jsonl"),
    `${JSON.stringify({
      id: "f4-6-capture-nudge-fixture",
      type: "decide",
      date: "2099-01-01T00:00:00.000Z",
      decision: "A settled outcome that never reached the state layer.",
      rationale: "fixture",
      scope: "repo",
      source: "user",
    })}\n`,
  );
  // The retired tree exists in BOTH fixtures — the branch must come from the
  // bundle predicate, never from "docs/specs/ is missing".
  fs.mkdirSync(path.join(root, "docs", "specs"), { recursive: true });
  fs.writeFileSync(path.join(root, "docs", "specs", "billing.md"), "# Billing\n");
  if (withBundle) {
    const conceptDir = path.join(root, "docs", "knowledge", "areas", "billing");
    fs.mkdirSync(conceptDir, { recursive: true });
    fs.writeFileSync(
      path.join(conceptDir, "overview.md"),
      ["---", "type: bee.area", "title: Billing — purpose", "description: A fixture concept.", "timestamp: 2026-07-22", "---", "", "# Billing", "", "Body.", ""].join("\n"),
    );
  }
  return root;
}

async function runCaptureNudgeRows() {
  const rows = [];

  const noBundleRoot = buildCaptureNudgeFixture("nobundle", { withBundle: false });
  const noBundle = await runWrapper(
    "bee-session-close.mjs",
    JSON.stringify({ hook_event_name: "Stop", cwd: noBundleRoot }),
    noBundleRoot,
  );
  const noBundleParsed = parseAdvisoryStdout(noBundle.stdout);
  const noBundleMsg =
    noBundleParsed.json && typeof noBundleParsed.json.systemMessage === "string" ? noBundleParsed.json.systemMessage : "";
  const noBundlePass = noBundle.status === 0 && noBundleMsg.includes(CAPTURE_NUDGE_NO_BUNDLE);
  rows.push(
    adapterRow(
      "session-close-capture-nudge",
      "no-bundle-wording-is-byte-identical-to-before",
      noBundlePass,
      noBundlePass
        ? "a repo with no bundle gets the pre-change nudge verbatim — a never-migrated host cannot tell this shipped"
        : `expected the exact pre-change no-bundle nudge; got status=${noBundle.status} stdout=${truncate(noBundle.stdout, 400)}`,
      noBundle,
    ),
  );

  const bundleRoot = buildCaptureNudgeFixture("bundle", { withBundle: true });
  const withBundle = await runWrapper(
    "bee-session-close.mjs",
    JSON.stringify({ hook_event_name: "Stop", cwd: bundleRoot }),
    bundleRoot,
  );
  const bundleParsed = parseAdvisoryStdout(withBundle.stdout);
  const bundleMsg =
    bundleParsed.json && typeof bundleParsed.json.systemMessage === "string" ? bundleParsed.json.systemMessage : "";
  const bundlePass =
    withBundle.status === 0 &&
    bundleMsg.includes("bee capture nudge (decision 0003)") &&
    bundleMsg.includes("knowledge bundle (docs/knowledge/)") &&
    !bundleMsg.includes("docs/specs/") &&
    !bundleMsg.includes("touched area's spec");
  rows.push(
    adapterRow(
      "session-close-capture-nudge",
      "bundle-mode-names-the-bundle-and-never-the-retired-tree",
      bundlePass,
      bundlePass
        ? "one parsing concept flips the nudge to the bundle: it names docs/knowledge/ and never sends the agent to a tree okf_specs_fence would fail it for writing"
        : `expected a bundle-branch nudge naming docs/knowledge/ and no docs/specs/; got status=${withBundle.status} stdout=${truncate(withBundle.stdout, 400)}`,
      withBundle,
    ),
  );

  return rows;
}

async function runHoldSessionRows() {
  const rows = [];

  // ── group 1: cross-session hold deny, phase-independence via a bound
  // SWARMING + execution-approved lane (C8) — the default state.json is
  // deliberately left at "idle" with no gates so a pass here can only come
  // from the bound lane's record, never from the default falling through
  // permissively.
  const holdRoot = buildFixture("hook-contracts-hold-");
  fs.writeFileSync(
    path.join(holdRoot, ".bee", "state.json"),
    `${JSON.stringify(
      { phase: "idle", mode: null, feature: null, approved_gates: { context: false, shape: false, execution: false, review: false } },
      null,
      2,
    )}\n`,
  );
  writeLaneFile(holdRoot, "lane-hold-swarm", {
    phase: "swarming",
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });
  writeSessionFile(holdRoot, "sess-acting", { lane: "lane-hold-swarm" });

  const nowIso = new Date().toISOString();
  const expiredIso = new Date(Date.now() - 3600_000).toISOString();
  seedReservationLease(holdRoot, { agent: "otto", cell: "fsh-x", path: "src/held.js", ttlSeconds: 3600, reservedAt: nowIso, session: "sess-holder" });
  seedReservationLease(holdRoot, { agent: "phil", cell: "fsh-8", path: "src/own.js", ttlSeconds: 3600, reservedAt: nowIso, session: "sess-acting" });
  seedReservationLease(holdRoot, { agent: "otto", cell: "fsh-x", path: "src/expired.js", ttlSeconds: 1, reservedAt: expiredIso, session: "sess-holder-expired" });
  seedReservationLease(holdRoot, { agent: "otto", cell: "fsh-x", path: "src/legacy.js", ttlSeconds: 3600, reservedAt: nowIso });

  const rHeld = await runWrapper(
    "bee-write-guard.mjs",
    editPayload({ sessionId: "sess-acting", cwd: holdRoot, filePath: "src/held.js" }),
    holdRoot,
  );
  const rHeldPass = Boolean(
    rHeld.status === 2 &&
      typeof rHeld.stderr === "string" &&
      rHeld.stderr.includes('session "sess-holder"') &&
      rHeld.stderr.includes("agent otto") &&
      rHeld.stderr.includes("expires"),
  );
  rows.push(
    adapterRow(
      "write-guard-hold",
      "cross-session-hold-denied-in-swarming-lane",
      rHeldPass,
      rHeldPass
        ? "a write into another session's active hold is denied (exit 2) through the real hook, in a swarming-phase execution-approved lane, naming the holder session/agent/expiry (C8)"
        : `expected exit 2 with holder+expiry in stderr; got status=${rHeld.status} stderr=${truncate(rHeld.stderr, 400)}`,
      rHeld,
    ),
  );

  const rOwn = await runWrapper(
    "bee-write-guard.mjs",
    editPayload({ sessionId: "sess-acting", cwd: holdRoot, filePath: "src/own.js" }),
    holdRoot,
  );
  const rOwnPass = rOwn.status === 0;
  rows.push(
    adapterRow(
      "write-guard-hold",
      "own-session-hold-never-blocks",
      rOwnPass,
      rOwnPass
        ? "the acting session's own hold on the target path never blocks its own write"
        : `expected exit 0 (own hold never blocks); got status=${rOwn.status} stderr=${truncate(rOwn.stderr, 400)}`,
      rOwn,
    ),
  );

  const rExpired = await runWrapper(
    "bee-write-guard.mjs",
    editPayload({ sessionId: "sess-acting", cwd: holdRoot, filePath: "src/expired.js" }),
    holdRoot,
  );
  const rExpiredPass = rExpired.status === 0;
  rows.push(
    adapterRow(
      "write-guard-hold",
      "expired-hold-never-blocks",
      rExpiredPass,
      rExpiredPass
        ? "an expired hold on the target path never blocks another session's write"
        : `expected exit 0 (expired hold never blocks); got status=${rExpired.status} stderr=${truncate(rExpired.stderr, 400)}`,
      rExpired,
    ),
  );

  const rLegacy = await runWrapper(
    "bee-write-guard.mjs",
    editPayload({ sessionId: "sess-acting", cwd: holdRoot, filePath: "src/legacy.js" }),
    holdRoot,
  );
  const rLegacyPass = rLegacy.status === 0;
  rows.push(
    adapterRow(
      "write-guard-hold",
      "legacy-session-less-row-never-blocks",
      rLegacyPass,
      rLegacyPass
        ? "a legacy reservation row with no session field never blocks a session-aware write"
        : `expected exit 0 (legacy row never blocks); got status=${rLegacy.status} stderr=${truncate(rLegacy.stderr, 400)}`,
      rLegacy,
    ),
  );

  // ── group 2: a lane-bound session_id is gated by THAT lane's phase/gates,
  // not the default record — the default record here is deliberately
  // permissive (swarming, every gate approved) so a deny can only come from
  // the bound lane's own (planning, execution unapproved) state.
  const gateRoot = buildFixture("hook-contracts-holdgate-");
  writeLaneFile(gateRoot, "lane-gated", {
    phase: "planning",
    approved_gates: { context: true, shape: true, execution: false, review: false },
  });
  writeSessionFile(gateRoot, "sess-gated", { lane: "lane-gated" });

  const gControl = await runWrapper(
    "bee-write-guard.mjs",
    editPayload({ cwd: gateRoot, filePath: "src/new.js" }),
    gateRoot,
  );
  const gControlPass = gControl.status === 0;
  rows.push(
    adapterRow(
      "write-guard-hold",
      "no-session-id-uses-permissive-default-as-control",
      gControlPass,
      gControlPass
        ? "no session_id: the permissive default record (swarming, execution approved) allows the write, as a control"
        : `expected exit 0 from the permissive default record; got status=${gControl.status} stderr=${truncate(gControl.stderr, 400)}`,
      gControl,
    ),
  );

  const gBound = await runWrapper(
    "bee-write-guard.mjs",
    editPayload({ sessionId: "sess-gated", cwd: gateRoot, filePath: "src/new.js" }),
    gateRoot,
  );
  const gBoundPass = Boolean(
    gBound.status === 2 &&
      typeof gBound.stderr === "string" &&
      gBound.stderr.includes('phase is "planning"') &&
      gBound.stderr.includes('gate "execution" is not approved'),
  );
  rows.push(
    adapterRow(
      "write-guard-hold",
      "lane-bound-session-gated-by-its-own-lane",
      gBoundPass,
      gBoundPass
        ? "a payload carrying a session_id bound to a lane is gated by THAT lane's phase/gates, even though the default record is permissive"
        : `expected exit 2 from the bound lane's own gate (planning/execution unapproved); got status=${gBound.status} stderr=${truncate(gBound.stderr, 400)}`,
      gBound,
    ),
  );

  // ── group 3: a present-but-corrupt reservation store fails closed (panel
  // B1, C7) — driven through the REAL hook child, not just the lib.
  const corruptRoot = buildFixture("hook-contracts-holdcorrupt-");
  writeCorruptReservationsFile(corruptRoot);

  const rCorrupt = await runWrapper(
    "bee-write-guard.mjs",
    editPayload({ sessionId: "sess-corrupt", cwd: corruptRoot, filePath: "src/anything.js" }),
    corruptRoot,
  );
  const rCorruptPass = Boolean(
    rCorrupt.status === 2 &&
      typeof rCorrupt.stderr === "string" &&
      rCorrupt.stderr.includes("unreadable/corrupt"),
  );
  rows.push(
    adapterRow(
      "write-guard-hold",
      "corrupt-reservation-store-fails-closed",
      rCorruptPass,
      rCorruptPass
        ? "a present-but-corrupt .bee/reservations.json yields a deny (exit 2) through the real hook for a session-carrying payload — the fail-open outer catch never swallows it into an allow"
        : `expected exit 2 naming the store unreadable/corrupt; got status=${rCorrupt.status} stderr=${truncate(rCorrupt.stderr, 400)}`,
      rCorrupt,
    ),
  );

  // ── group 4: zero-difference control — a payload with NO session_id at
  // all behaves byte-identically to today even when an active session-owned
  // hold exists on the exact target path.
  const zeroDiffRoot = buildFixture("hook-contracts-holdzerodiff-");
  seedReservationLease(zeroDiffRoot, { agent: "otto", cell: "fsh-x", path: "src/heldzero.js", ttlSeconds: 3600, reservedAt: nowIso, session: "sess-other" });
  const rZero = await runWrapper(
    "bee-write-guard.mjs",
    editPayload({ cwd: zeroDiffRoot, filePath: "src/heldzero.js" }),
    zeroDiffRoot,
  );
  const rZeroPass = rZero.status === 0;
  rows.push(
    adapterRow(
      "write-guard-hold",
      "no-session-id-is-zero-difference-even-with-a-hold",
      rZeroPass,
      rZeroPass
        ? "a payload without session_id is never gated on any hold — byte-identical to today, even with an active session-owned hold on the exact path"
        : `expected exit 0 (no session_id never consults holds); got status=${rZero.status} stderr=${truncate(rZero.stderr, 400)}`,
      rZero,
    ),
  );

  return rows;
}

// multisession-native-21b (P2 goal-check residual): bee-state-sync.mjs and
// bee-prompt-context.mjs each call claims.heartbeatTouch(root, sessionId)
// with the BARE checkout root. Sessions/claims are control-plane (msn-18c,
// hooks/adapter.mjs's own ctx.controlRoot comment) and MUST resolve against
// ctx.controlRoot, exactly like bee-session-init.mjs and bee-write-guard.mjs
// already do. From a GRANTED linked worktree (its own local `.bee` store,
// same shape a real worktree-feature-parallelism checkout has) a passive
// hook tick must still renew the acting session's heartbeat in MAIN's
// session store, never the worktree's own — a live write-owner renewing
// against the wrong store is silently reclaimable as dead (isOwnerLive /
// guard class c read the stale MAIN-store heartbeat).
function writeStaleSessionFile(root, sessionId, { ageSeconds = 120 } = {}) {
  const sessionsDir = path.join(root, ".bee", "sessions");
  fs.mkdirSync(sessionsDir, { recursive: true });
  const stamp = new Date(Date.now() - ageSeconds * 1000).toISOString();
  const record = { id: sessionId, started_at: stamp, last_heartbeat: stamp };
  fs.writeFileSync(path.join(sessionsDir, `${sessionId}.json`), `${JSON.stringify(record, null, 2)}\n`);
  return record;
}

function readSessionHeartbeat(root, sessionId) {
  try {
    const raw = fs.readFileSync(path.join(root, ".bee", "sessions", `${sessionId}.json`), "utf8");
    return JSON.parse(raw).last_heartbeat;
  } catch {
    return null;
  }
}

// A GRANTED linked worktree: mainRoot/workRoot are independent buildFixture()
// checkouts, linked via a real .git-file gitdir round trip (same shape
// runWorktreeAdapterRows uses), with workRoot's id registered TRUE in
// mainRoot's runtime/worktree-grants.json — the opt-in that gives a linked
// worktree its own local store (storeRoot=workRoot) while controlRoot stays
// grant-independent (always mainRoot).
function buildGrantedWorktreeFixture(prefix) {
  const mainRoot = buildFixture(`${prefix}main-`);
  const workRoot = buildFixture(`${prefix}work-`);
  const gitdir = path.join(mainRoot, ".git", "worktrees", "fixture");
  fs.mkdirSync(gitdir, { recursive: true });
  fs.writeFileSync(path.join(workRoot, ".git"), `gitdir: ${gitdir}\n`);
  fs.writeFileSync(path.join(gitdir, "gitdir"), `${path.join(workRoot, ".git")}\n`);
  fs.mkdirSync(path.join(mainRoot, ".bee", "runtime"), { recursive: true });
  fs.writeFileSync(
    path.join(mainRoot, ".bee", "runtime", "worktree-grants.json"),
    `${JSON.stringify({ fixture: true }, null, 2)}\n`,
  );
  return { mainRoot, workRoot };
}

async function runGrantedWorktreeHeartbeatRows() {
  const rows = [];
  const { mainRoot, workRoot } = buildGrantedWorktreeFixture("hook-msn21b-");

  // Sanity: the fixture really is a GRANTED linked worktree — control-root
  // resolves to main, storeRoot resolves to the worktree's own local store
  // (grant=true). If this row ever fails the rows below are testing nothing.
  const linked = resolveHookRoots(workRoot);
  const sanityPass = Boolean(
    linked.worktreeResolution === "linked-valid" &&
      linked.storeRoot === fs.realpathSync.native(workRoot) &&
      hookControlRootFor(workRoot) === fs.realpathSync.native(mainRoot),
  );
  rows.push(
    genericRow(
      "granted-worktree-heartbeat",
      "fixture-is-a-granted-linked-worktree",
      sanityPass,
      sanityPass
        ? "fixture resolves linked-valid, storeRoot=workRoot (granted), controlRoot=mainRoot — the rows below exercise a real granted-worktree topology"
        : `fixture topology is wrong: ${JSON.stringify(linked)} controlRoot=${hookControlRootFor(workRoot)}`,
    ),
  );

  // ── bee-state-sync: a passive tick from the granted worktree must renew
  // the session's heartbeat in MAIN's store, never the worktree's own.
  const stateSyncSession = "sess-msn21b-state-sync";
  const staleBefore = writeStaleSessionFile(mainRoot, stateSyncSession);
  const sTick = await runWrapper(
    "bee-state-sync.mjs",
    JSON.stringify({ hook_event_name: "Stop", session_id: stateSyncSession, cwd: workRoot }),
    workRoot,
  );
  const sSilent = sTick.status === 0 && !(sTick.stdout || "").trim();
  const sAfter = readSessionHeartbeat(mainRoot, stateSyncSession);
  const sRenewed = Boolean(sAfter && sAfter !== staleBefore.last_heartbeat && Date.now() - Date.parse(sAfter) < 30_000);
  const sNeverInWorktree = !fs.existsSync(path.join(workRoot, ".bee", "sessions", `${stateSyncSession}.json`));
  const sPass = sSilent && sRenewed && sNeverInWorktree;
  rows.push(
    adapterRow(
      "granted-worktree-heartbeat",
      "state-sync-passive-tick-renews-main-store-heartbeat",
      sPass,
      sPass
        ? "a passive bee-state-sync tick from a granted worktree renews the session heartbeat in MAIN's store (ctx.controlRoot), never the worktree's own — a live write-owner stays live"
        : `expected a silent renewal landing in MAIN's session store; status=${sTick.status} stdout=${truncate(sTick.stdout, 200)} before=${staleBefore.last_heartbeat} after=${sAfter} strayInWorktree=${!sNeverInWorktree}`,
      sTick,
    ),
  );

  // ── bee-prompt-context: same contract, independent session id so the two
  // rows can never mask each other via the 60s heartbeat-touch throttle.
  const promptSession = "sess-msn21b-prompt-context";
  const staleBefore2 = writeStaleSessionFile(mainRoot, promptSession);
  const pTick = await runWrapper(
    "bee-prompt-context.mjs",
    JSON.stringify({ hook_event_name: "UserPromptSubmit", prompt: "hi", session_id: promptSession, cwd: workRoot }),
    workRoot,
  );
  const pAfter = readSessionHeartbeat(mainRoot, promptSession);
  const pRenewed = Boolean(pAfter && pAfter !== staleBefore2.last_heartbeat && Date.now() - Date.parse(pAfter) < 30_000);
  const pNeverInWorktree = !fs.existsSync(path.join(workRoot, ".bee", "sessions", `${promptSession}.json`));
  const pPass = pTick.status === 0 && pRenewed && pNeverInWorktree;
  rows.push(
    adapterRow(
      "granted-worktree-heartbeat",
      "prompt-context-passive-tick-renews-main-store-heartbeat",
      pPass,
      pPass
        ? "a passive bee-prompt-context tick from a granted worktree renews the session heartbeat in MAIN's store (ctx.controlRoot), never the worktree's own"
        : `expected a renewal landing in MAIN's session store; status=${pTick.status} before=${staleBefore2.last_heartbeat} after=${pAfter} strayInWorktree=${!pNeverInWorktree}`,
      pTick,
    ),
  );

  // ── control: an ORDINARY (non-worktree) repo renews byte-identically —
  // controlRoot === root there, so this must pass unchanged before AND
  // after the fix (must_have: "solo repo byte-identical").
  const soloRoot = buildFixture("hook-msn21b-solo-");
  const soloSession = "sess-msn21b-solo";
  const soloBefore = writeStaleSessionFile(soloRoot, soloSession);
  const soloTick = await runWrapper(
    "bee-state-sync.mjs",
    JSON.stringify({ hook_event_name: "Stop", session_id: soloSession, cwd: soloRoot }),
    soloRoot,
  );
  const soloAfter = readSessionHeartbeat(soloRoot, soloSession);
  const soloRenewed = Boolean(soloAfter && soloAfter !== soloBefore.last_heartbeat);
  const soloPass = soloTick.status === 0 && soloRenewed;
  rows.push(
    adapterRow(
      "granted-worktree-heartbeat",
      "solo-repo-heartbeat-renewal-byte-identical",
      soloPass,
      soloPass
        ? "an ordinary (non-worktree) repo still renews its own session heartbeat in its own store — controlRoot===root there, unaffected by the fix (solo repo byte-identical)"
        : `expected the solo repo's own heartbeat to renew; status=${soloTick.status} before=${soloBefore.last_heartbeat} after=${soloAfter}`,
      soloTick,
    ),
  );

  return rows;
}

// fresh-session-handoff fsh-10 (D1, D4, validation-s4 C11/C12): bee-session-init
// is the ONLY place a planned-next handoff is ever adopted — the hook threads
// payload.session_id + payload.source, performs the source-gated adoption via
// fsh-9's adoptHandoff, and passes the typed outcome into the pure builder
// (buildSessionPreamble). Direct-JSON fixture writers, mirroring this file's
// existing writeLaneFile/writeSessionFile/writeReservationsFile style (this
// harness executes wrappers as isolated Workers and never imports the lib
// under test).
function writeCellFileFixture(root, id, extra = {}) {
  const dir = path.join(root, ".bee", "cells");
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(
    path.join(dir, `${id}.json`),
    `${JSON.stringify({ id, feature: "fresh-session-handoff", title: "fixture", lane: "high-risk", ...extra }, null, 2)}\n`,
  );
}

function writeClaimFileFixture(root, cellId, session, extra = {}) {
  const dir = path.join(root, ".bee", "claims");
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(
    path.join(dir, `${cellId}.json`),
    `${JSON.stringify({ cell: cellId, session, ttl_seconds: 3600, claimed_at: new Date().toISOString(), ...extra }, null, 2)}\n`,
  );
}

function writeHandoffFileFixture(root, record) {
  fs.writeFileSync(path.join(root, ".bee", "HANDOFF.json"), `${JSON.stringify(record, null, 2)}\n`);
}

function readJsonFileOrNull(file) {
  try {
    return JSON.parse(fs.readFileSync(file, "utf8"));
  } catch {
    return null;
  }
}

function sessionStartPayload({ sessionId, source, cwd }) {
  return JSON.stringify({
    hook_event_name: "SessionStart",
    ...(source ? { source } : {}),
    ...(sessionId ? { session_id: sessionId } : {}),
    cwd,
  });
}

// One shared planned-next fixture: previous cell capped+green, next cell
// claimed by the writer session, a planned-next HANDOFF.json carrying it.
function buildPlannedNextFixture(prefix, { writerSession = "sess-writer" } = {}) {
  const root = buildFixture(prefix);
  writeCellFileFixture(root, "prev-fsh10", { status: "capped", trace: { verify_passed: true } });
  writeCellFileFixture(root, "next-fsh10", {
    status: "open",
    lane: "high-risk",
    verify: "node verify-fsh10.mjs",
  });
  writeClaimFileFixture(root, "next-fsh10", writerSession);
  writeHandoffFileFixture(root, {
    kind: "planned-next",
    writer_session: writerSession,
    previous_cell: "prev-fsh10",
    next_cell: "next-fsh10",
    next_action: "start next-fsh10",
  });
  return root;
}

async function runHandoffSessionRows() {
  const rows = [];

  // ── row (a): source "clear" through the REAL hook child — start-now block
  // naming the adopted cell, AND the claim's ownership actually transfers.
  const clearRoot = buildPlannedNextFixture("hook-contracts-handoff-clear-");
  const rClear = await runWrapper(
    "bee-session-init.mjs",
    sessionStartPayload({ sessionId: "sess-fresh", source: "clear", cwd: clearRoot }),
    clearRoot,
  );
  const clearClaim = readJsonFileOrNull(path.join(clearRoot, ".bee", "claims", "next-fsh10.json"));
  const rClearPass = Boolean(
    rClear.status === 0 &&
      /PLANNED-NEXT ADOPTED/.test(rClear.stdout || "") &&
      /next-fsh10/.test(rClear.stdout || "") &&
      clearClaim &&
      clearClaim.session === "sess-fresh" &&
      !fs.existsSync(path.join(clearRoot, ".bee", "HANDOFF.json")),
  );
  rows.push(
    adapterRow(
      "session-init-handoff",
      "clear-source-adopts-and-transfers-claim",
      rClearPass,
      rClearPass
        ? 'source "clear": a planned-next handoff yields a start-now preamble naming the adopted cell, the claim transfers to the new session, and the handoff is cleared'
        : `expected a start-now preamble + transferred claim; got status=${rClear.status} claim=${JSON.stringify(clearClaim)} stdout=${truncate(rClear.stdout, 400)}`,
      rClear,
    ),
  );

  // ── row (a'): source "startup" (the OTHER qualifying source) behaves the
  // same as "clear" when the acting session is genuinely a different session
  // than the one that wrote the handoff.
  const startupRoot = buildPlannedNextFixture("hook-contracts-handoff-startup-");
  const rStartup = await runWrapper(
    "bee-session-init.mjs",
    sessionStartPayload({ sessionId: "sess-fresh-2", source: "startup", cwd: startupRoot }),
    startupRoot,
  );
  const startupClaim = readJsonFileOrNull(path.join(startupRoot, ".bee", "claims", "next-fsh10.json"));
  const rStartupPass = Boolean(
    rStartup.status === 0 &&
      /PLANNED-NEXT ADOPTED/.test(rStartup.stdout || "") &&
      startupClaim &&
      startupClaim.session === "sess-fresh-2" &&
      !fs.existsSync(path.join(startupRoot, ".bee", "HANDOFF.json")),
  );
  rows.push(
    adapterRow(
      "session-init-handoff",
      "startup-source-from-a-different-session-adopts",
      rStartupPass,
      rStartupPass
        ? 'source "startup" from a genuinely different acting session also adopts and transfers the claim'
        : `expected adoption on startup from a different session; got status=${rStartup.status} claim=${JSON.stringify(startupClaim)} stdout=${truncate(rStartup.stdout, 400)}`,
      rStartup,
    ),
  );

  // ── rows (b): source "resume"/"compact" NEVER adopt — mandatory negative
  // rows (validation-s4 C11). The handoff stays on disk, the claim owner is
  // unchanged, and the preamble shows a pending-wait block (never start-now).
  for (const source of ["resume", "compact"]) {
    const root = buildPlannedNextFixture(`hook-contracts-handoff-${source}-`);
    const result = await runWrapper(
      "bee-session-init.mjs",
      sessionStartPayload({ sessionId: "sess-fresh-neg", source, cwd: root }),
      root,
    );
    const claim = readJsonFileOrNull(path.join(root, ".bee", "claims", "next-fsh10.json"));
    const handoffIntact = fs.existsSync(path.join(root, ".bee", "HANDOFF.json"));
    const pass = Boolean(
      result.status === 0 &&
        /HANDOFF present — present it and WAIT/.test(result.stdout || "") &&
        !/PLANNED-NEXT ADOPTED/.test(result.stdout || "") &&
        handoffIntact &&
        claim &&
        claim.session === "sess-writer",
    );
    rows.push(
      adapterRow(
        "session-init-handoff",
        `${source}-source-never-adopts`,
        pass,
        pass
          ? `source "${source}": a planned-next handoff NEVER adopts — pending-wait block, handoff intact, claim owner unchanged (C11)`
          : `expected a pending-wait block with the handoff/claim untouched; got status=${result.status} handoffIntact=${handoffIntact} claim=${JSON.stringify(claim)} stdout=${truncate(result.stdout, 400)}`,
        result,
      ),
    );
  }

  // ── row: a "startup" source whose writer_session equals the acting session
  // is ALSO refused — not a genuine fresh-session boundary (panel W1 pin).
  const sameSessionRoot = buildPlannedNextFixture("hook-contracts-handoff-samesession-", {
    writerSession: "sess-same",
  });
  const rSameSession = await runWrapper(
    "bee-session-init.mjs",
    sessionStartPayload({ sessionId: "sess-same", source: "startup", cwd: sameSessionRoot }),
    sameSessionRoot,
  );
  const sameSessionClaim = readJsonFileOrNull(path.join(sameSessionRoot, ".bee", "claims", "next-fsh10.json"));
  const rSameSessionPass = Boolean(
    rSameSession.status === 0 &&
      /HANDOFF present — present it and WAIT/.test(rSameSession.stdout || "") &&
      !/PLANNED-NEXT ADOPTED/.test(rSameSession.stdout || "") &&
      fs.existsSync(path.join(sameSessionRoot, ".bee", "HANDOFF.json")) &&
      sameSessionClaim &&
      sameSessionClaim.session === "sess-same",
  );
  rows.push(
    adapterRow(
      "session-init-handoff",
      "startup-same-writer-session-never-self-adopts",
      rSameSessionPass,
      rSameSessionPass
        ? 'source "startup" whose writer_session equals the acting session is refused — not a fresh-session boundary'
        : `expected a refused pending-wait block; got status=${rSameSession.status} claim=${JSON.stringify(sameSessionClaim)} stdout=${truncate(rSameSession.stdout, 400)}`,
      rSameSession,
    ),
  );

  // ── row (c): a pause handoff renders today's wait block byte-identically
  // whether or not a session_id is present (only planned-next branches).
  const pauseRootA = buildFixture("hook-contracts-handoff-pause-a-");
  writeHandoffFileFixture(pauseRootA, { kind: "pause", cell: "wip-x", next_action: "resume wip-x" });
  const pauseRootB = buildFixture("hook-contracts-handoff-pause-b-");
  writeHandoffFileFixture(pauseRootB, { kind: "pause", cell: "wip-x", next_action: "resume wip-x" });
  const rPauseNoSession = await runWrapper(
    "bee-session-init.mjs",
    sessionStartPayload({ cwd: pauseRootA }),
    pauseRootA,
  );
  const rPauseWithSession = await runWrapper(
    "bee-session-init.mjs",
    sessionStartPayload({ sessionId: "sess-pause", source: "clear", cwd: pauseRootB }),
    pauseRootB,
  );
  const pausePass = Boolean(
    rPauseNoSession.status === 0 &&
      rPauseWithSession.status === 0 &&
      rPauseNoSession.stdout === rPauseWithSession.stdout &&
      /HANDOFF present — present it and WAIT/.test(rPauseNoSession.stdout || ""),
  );
  rows.push(
    adapterRow(
      "session-init-handoff",
      "pause-handoff-byte-identical-regardless-of-session",
      pausePass,
      pausePass
        ? "a pause handoff renders today's wait block byte-identically whether or not session_id/source clear is present — never adopted"
        : `expected byte-identical pause rendering; got noSession.status=${rPauseNoSession.status} withSession.status=${rPauseWithSession.status} equal=${rPauseNoSession.stdout === rPauseWithSession.stdout}`,
      rPauseWithSession,
    ),
  );

  // ── row (c'): a kindless (legacy, no `kind` field) handoff normalizes to
  // pause and renders the identical wait block too.
  const kindlessRootA = buildFixture("hook-contracts-handoff-kindless-a-");
  writeHandoffFileFixture(kindlessRootA, { cell: "wip-legacy", done: [], remaining: [] });
  const kindlessRootB = buildFixture("hook-contracts-handoff-kindless-b-");
  writeHandoffFileFixture(kindlessRootB, { cell: "wip-legacy", done: [], remaining: [] });
  const rKindlessNoSession = await runWrapper(
    "bee-session-init.mjs",
    sessionStartPayload({ cwd: kindlessRootA }),
    kindlessRootA,
  );
  const rKindlessWithSession = await runWrapper(
    "bee-session-init.mjs",
    sessionStartPayload({ sessionId: "sess-kindless", source: "clear", cwd: kindlessRootB }),
    kindlessRootB,
  );
  const kindlessPass = Boolean(
    rKindlessNoSession.status === 0 &&
      rKindlessWithSession.status === 0 &&
      rKindlessNoSession.stdout === rKindlessWithSession.stdout &&
      /HANDOFF present — present it and WAIT/.test(rKindlessNoSession.stdout || ""),
  );
  rows.push(
    adapterRow(
      "session-init-handoff",
      "kindless-legacy-handoff-byte-identical-regardless-of-session",
      kindlessPass,
      kindlessPass
        ? "a legacy handoff with no kind field normalizes to pause and renders the identical wait block regardless of session_id/source"
        : `expected byte-identical kindless rendering; got noSession.status=${rKindlessNoSession.status} withSession.status=${rKindlessWithSession.status} equal=${rKindlessNoSession.stdout === rKindlessWithSession.stdout}`,
      rKindlessWithSession,
    ),
  );

  // ── row (d): payloads without session_id render byte-identically to today
  // EVEN with a planned-next handoff present — no adoption is ever attempted.
  const noIdRootA = buildPlannedNextFixture("hook-contracts-handoff-noid-a-");
  const noIdRootB = buildPlannedNextFixture("hook-contracts-handoff-noid-b-");
  const rNoIdBare = await runWrapper("bee-session-init.mjs", sessionStartPayload({ cwd: noIdRootA }), noIdRootA);
  const rNoIdWithSource = await runWrapper(
    "bee-session-init.mjs",
    sessionStartPayload({ source: "clear", cwd: noIdRootB }),
    noIdRootB,
  );
  const noIdBareClaim = readJsonFileOrNull(path.join(noIdRootA, ".bee", "claims", "next-fsh10.json"));
  const noIdPass = Boolean(
    rNoIdBare.status === 0 &&
      rNoIdWithSource.status === 0 &&
      rNoIdBare.stdout === rNoIdWithSource.stdout &&
      /HANDOFF present — present it and WAIT/.test(rNoIdBare.stdout || "") &&
      !/PLANNED-NEXT ADOPTED/.test(rNoIdBare.stdout || "") &&
      !/Adoption not applied/.test(rNoIdBare.stdout || "") &&
      noIdBareClaim &&
      noIdBareClaim.session === "sess-writer" &&
      fs.existsSync(path.join(noIdRootA, ".bee", "HANDOFF.json")),
  );
  rows.push(
    adapterRow(
      "session-init-handoff",
      "no-session-id-byte-identical-even-with-planned-next",
      noIdPass,
      noIdPass
        ? "no session_id: byte-identical to today even with a planned-next handoff on disk and source=clear — no adoption is ever attempted, claim/handoff untouched"
        : `expected byte-identical no-adoption rendering; got bare.status=${rNoIdBare.status} withSource.status=${rNoIdWithSource.status} equal=${rNoIdBare.stdout === rNoIdWithSource.stdout} claim=${JSON.stringify(noIdBareClaim)}`,
      rNoIdWithSource,
    ),
  );

  return rows;
}

// ─── intent anchor (intent-anchor ia-1, CONTEXT.md D3/D4/D5) ───────────────
//
// Three properties, each with its own row, all through the REAL hook children:
//
//   D3  PreCompact re-asserts the anchor and STAYS ADVISORY — a labelled
//       systemMessage, never `decision:"block"`. The B2/R14 contract
//       (docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md)
//       forbids a turn-control verdict on compaction and this must not become
//       the exception that erodes it.
//   D4  A compact/resume SessionStart LEADS with the anchor; the phase detail
//       follows. The ordering is the fix, so the row asserts position, not
//       mere presence.
//   D5  With no anchor, both hooks are byte-identical to today. Proven in the
//       durable form: the anchor path is strictly ADDITIVE — the with-anchor
//       output is exactly (anchor block + separator + the no-anchor output),
//       so every pre-existing byte is untouched, and a repo with no anchor
//       (or a corrupt one) renders exactly what it always did.
//
// Anchors are written as plain JSON, matching this file's buildFixture style
// (this harness executes wrappers as isolated Workers and never imports the
// lib it is testing).

const INTENT_VERBATIM_REQUEST =
  "Make the /orders endpoint idempotent under retries — the same Idempotency-Key must never create a second order.";

function writeIntentFileFixture(root, key, extra = {}) {
  const dir = path.join(root, ".bee", "intent");
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(
    path.join(dir, `${key}.json`),
    `${JSON.stringify(
      {
        schema_version: "1.0",
        key,
        written_at: new Date().toISOString(),
        request: INTENT_VERBATIM_REQUEST,
        acceptance: "A replayed POST with the same key returns the first order and creates no second row.",
        next_action: "write the dedupe index migration",
        feature: "demo",
        lane: null,
        cell: null,
        do_not_reverse: [],
        stop_conditions: [],
        ...extra,
      },
      null,
      2,
    )}\n`,
  );
}

async function runIntentAnchorRows() {
  const rows = [];

  const precompactPayload = (root, sessionId) =>
    JSON.stringify({
      hook_event_name: "PreCompact",
      ...(sessionId ? { session_id: sessionId } : {}),
      cwd: root,
    });

  // ── D3: PreCompact emits the labelled anchor as an ADVISORY systemMessage.
  const preRoot = buildFixture("hook-contracts-intent-precompact-");
  const preControl = buildFixture("hook-contracts-intent-precompact-control-");
  // The fixture's phase is "demo"/swarming, so the anchor keys on the feature.
  writeIntentFileFixture(preRoot, "demo");
  const rPre = await runWrapper("bee-session-close.mjs", precompactPayload(preRoot, "sess-pre"), preRoot);
  const rPreControl = await runWrapper(
    "bee-session-close.mjs",
    precompactPayload(preControl, "sess-pre"),
    preControl,
  );
  const prePayload = parseAdvisoryStdout(rPre.stdout);
  const preControlPayload = parseAdvisoryStdout(rPreControl.stdout);
  const preMessage = prePayload.json && typeof prePayload.json.systemMessage === "string" ? prePayload.json.systemMessage : "";
  const preControlMessage =
    preControlPayload.json && typeof preControlPayload.json.systemMessage === "string"
      ? preControlPayload.json.systemMessage
      : "";
  const rPrePass = Boolean(
    rPre.status === 0 &&
      prePayload.json &&
      prePayload.json.decision !== "block" &&
      preMessage.startsWith("=== BEE INTENT ANCHOR") &&
      preMessage.includes("DO NOT SUMMARIZE") &&
      preMessage.includes(INTENT_VERBATIM_REQUEST),
  );
  rows.push(
    adapterRow(
      "session-close-intent",
      "precompact-emits-labelled-anchor-and-stays-advisory",
      rPrePass,
      rPrePass
        ? "PreCompact re-asserts the anchor VERBATIM in a labelled block, leading a parseable systemMessage advisory — never a decision:\"block\" turn-control verdict (D3, B2/R14 untouched)"
        : `expected a labelled advisory carrying the verbatim request; got status=${rPre.status} decision=${prePayload.json && prePayload.json.decision} stdout=${truncate(rPre.stdout, 400)}`,
      rPre,
    ),
  );

  // ── D5 (session-close) + D23 (compaction-hardening cz-7): additivity, SPLIT.
  //
  // Until cz-7 this was ONE assertion: the with-anchor message equals the
  // anchor block plus the no-anchor control's exact bytes. D10's anchor nudge
  // fires precisely when readIntent returns null — true in `preControl` and
  // false in `preRoot` — and the shared fixture (buildFixture, :240-259) writes
  // phase "swarming" with approved_gates.execution true, satisfying D10's
  // predicate. So the control now leads with a nudge line the with-anchor
  // message does not have, and the single assertion goes red BY DESIGN. That is
  // recorded as locked decision D23, which also fixes the split below.
  //
  // SPLIT, NEVER WEAKENED. Two byte-exact assertions replace one; neither is a
  // substring or regex sniff of the whole message:
  //
  //   (a) ADDITIVITY OVER THE SHARED PORTION. Each message is `lead + "\n" +
  //       tail` (main() joins `parts` with "\n"). The leads differ by
  //       construction — the anchor block in one, the nudge in the other — and
  //       the TAILS MUST BE IDENTICAL BYTES: every byte this hook emitted
  //       before the feature is still emitted, unchanged, on both paths. The
  //       tail is additionally asserted NON-EMPTY and carrying the "hive door
  //       open" warning, because with an empty tail (a) would be vacuously true
  //       and would still pass against a hook that emitted nothing else.
  //
  //   (b) THE NUDGE, BOTH DIRECTIONS, EXACTLY. The control's lead line must
  //       equal the bytes `anchorMissing` itself produces for that fixture —
  //       resolved by importing the very compaction.mjs the hook loaded, out of
  //       the fixture's own vendored lib — and the with-anchor message must not
  //       contain those bytes anywhere at all.
  const preAnchorBlock = preMessage.split("\n=== END BEE INTENT ANCHOR ===\n")[0];
  const preAnchorLead = `${preAnchorBlock}\n=== END BEE INTENT ANCHOR ===`;
  // `lead + "\n" + tail` → tail, or null when the message does not lead as claimed.
  const tailBelow = (message, lead) => {
    if (message === lead) return "";
    return message.startsWith(`${lead}\n`) ? message.slice(lead.length + 1) : null;
  };
  const preControlLead = preControlMessage.split("\n")[0];
  const preTail = tailBelow(preMessage, preAnchorLead);
  const preControlTail = tailBelow(preControlMessage, preControlLead);
  const preAdditivePass = Boolean(
    rPreControl.status === 0 &&
      !preControlMessage.includes("BEE INTENT ANCHOR") &&
      preTail !== null &&
      preControlTail !== null &&
      preTail === preControlTail &&
      preTail.includes("bee session-close warning: session is ending mid-phase"),
  );
  rows.push(
    adapterRow(
      "session-close-intent",
      "precompact-anchor-and-nudge-leads-share-an-identical-advisory-tail",
      preAdditivePass,
      preAdditivePass
        ? "below its lead line each PreCompact advisory is BYTE-IDENTICAL to the other's — the anchor (or, with no anchor, the D10 nudge) is a lead, and every byte the hook emitted before this feature is still emitted unchanged (D5/D23)"
        : `expected identical advisory tails below the two leads; controlTail=${truncate(String(preControlTail), 300)} withTail=${truncate(String(preTail), 300)}`,
      rPreControl,
    ),
  );

  // (b) — the nudge is present in the control and absent with an anchor. The
  // expected bytes come from the module the hook actually ran, so this row
  // cannot drift into asserting a paraphrase of the nudge.
  const preNudgeExpected = await (async () => {
    try {
      const mod = await import(
        pathToFileURL(path.join(preControl, ".bee", "bin", "lib", "compaction.mjs")).href
      );
      const nudge = mod.anchorMissing(preControl, { sessionId: "sess-pre" });
      return nudge ? nudge.message : null;
    } catch {
      return null;
    }
  })();
  const preNudgePass = Boolean(
    preNudgeExpected &&
      preControlLead === preNudgeExpected &&
      !preMessage.includes(preNudgeExpected),
  );
  rows.push(
    adapterRow(
      "session-close-intent",
      "precompact-nudges-when-no-anchor-exists-and-never-when-one-does",
      preNudgePass,
      preNudgePass
        ? "with no anchor the PreCompact advisory LEADS with D10's nudge, byte-for-byte as compaction.mjs builds it; with an anchor stored the nudge is absent entirely — the two leads are mutually exclusive by predicate (D10/D11/D23)"
        : `expected the control to lead with the module's nudge and the with-anchor message to carry none; expected=${truncate(String(preNudgeExpected), 300)} controlLead=${truncate(preControlLead, 300)}`,
      rPreControl,
    ),
  );

  // ── D3 negative: Stop is NOT the compaction event — no anchor there.
  const rStop = await runWrapper(
    "bee-session-close.mjs",
    JSON.stringify({ hook_event_name: "Stop", session_id: "sess-pre", cwd: preRoot }),
    preRoot,
  );
  const stopMessage = (() => {
    const parsed = parseAdvisoryStdout(rStop.stdout);
    return parsed.json && typeof parsed.json.systemMessage === "string" ? parsed.json.systemMessage : "";
  })();
  const rStopPass = Boolean(rStop.status === 0 && !stopMessage.includes("BEE INTENT ANCHOR"));
  rows.push(
    adapterRow(
      "session-close-intent",
      "stop-event-never-emits-the-anchor",
      rStopPass,
      rStopPass
        ? "the anchor rides the COMPACTION event only — a Stop close is unchanged (its job is to survive a summary, not to close one)"
        : `expected no anchor on Stop; got status=${rStop.status} stdout=${truncate(rStop.stdout, 400)}`,
      rStop,
    ),
  );

  // ── D4: a compact/resume SessionStart LEADS with the anchor, and the
  // anchor path is additive over the exact preamble today's hook prints.
  for (const source of ["compact", "resume"]) {
    const root = buildFixture(`hook-contracts-intent-${source}-`);
    const control = buildFixture(`hook-contracts-intent-${source}-control-`);
    writeIntentFileFixture(root, "demo");
    const withAnchor = await runWrapper(
      "bee-session-init.mjs",
      sessionStartPayload({ sessionId: "sess-anchor", source, cwd: root }),
      root,
    );
    const noAnchor = await runWrapper(
      "bee-session-init.mjs",
      sessionStartPayload({ sessionId: "sess-anchor", source, cwd: control }),
      control,
    );
    const out = withAnchor.stdout || "";
    const controlOut = noAnchor.stdout || "";
    const leadPass = Boolean(
      withAnchor.status === 0 &&
        noAnchor.status === 0 &&
        out.startsWith("## INTENT ANCHOR — read this FIRST") &&
        out.includes(INTENT_VERBATIM_REQUEST) &&
        // ordering IS the fix: objective first, phase detail after
        out.indexOf(INTENT_VERBATIM_REQUEST) < out.indexOf("- Phase:") &&
        // additive-only: the preamble bytes are exactly the control's
        out.endsWith(`\n\n${controlOut}`) &&
        !controlOut.includes("INTENT ANCHOR"),
    );
    rows.push(
      adapterRow(
        "session-init-intent",
        `${source}-source-leads-with-the-anchor-and-is-additive`,
        leadPass,
        leadPass
          ? `source "${source}": the emitted block LEADS with the verbatim objective and the phase detail follows, and the preamble below it is byte-identical to the no-anchor rendering (D4/D5)`
          : `expected an anchor-led, strictly additive preamble; got status=${withAnchor.status} startsWithAnchor=${out.startsWith("## INTENT ANCHOR")} additive=${out.endsWith(`\n\n${controlOut}`)} stdout=${truncate(out, 500)}`,
        withAnchor,
      ),
    );
    fs.rmSync(control, { recursive: true, force: true });
    fs.rmSync(root, { recursive: true, force: true });
  }

  // ── D4/D5 negative: clear/startup are NOT compact-resume sources — an
  // anchor on disk changes nothing there.
  for (const source of ["clear", "startup"]) {
    const root = buildFixture(`hook-contracts-intent-nonresume-${source}-`);
    const control = buildFixture(`hook-contracts-intent-nonresume-${source}-control-`);
    writeIntentFileFixture(root, "demo");
    const withAnchor = await runWrapper(
      "bee-session-init.mjs",
      sessionStartPayload({ sessionId: "sess-anchor", source, cwd: root }),
      root,
    );
    const noAnchor = await runWrapper(
      "bee-session-init.mjs",
      sessionStartPayload({ sessionId: "sess-anchor", source, cwd: control }),
      control,
    );
    const pass = Boolean(
      withAnchor.status === 0 &&
        noAnchor.status === 0 &&
        withAnchor.stdout === noAnchor.stdout &&
        !(withAnchor.stdout || "").includes("INTENT ANCHOR"),
    );
    rows.push(
      adapterRow(
        "session-init-intent",
        `${source}-source-is-byte-identical-with-or-without-an-anchor`,
        pass,
        pass
          ? `source "${source}" is a fresh-session boundary, not a compact-resume — an anchor on disk leaves its rendering byte-identical (D4 scope, D5)`
          : `expected byte-identical rendering; got equal=${withAnchor.stdout === noAnchor.stdout} stdout=${truncate(withAnchor.stdout, 400)}`,
        withAnchor,
      ),
    );
    fs.rmSync(control, { recursive: true, force: true });
    fs.rmSync(root, { recursive: true, force: true });
  }

  // ── D5: a CORRUPT anchor is indistinguishable from no anchor at all.
  const corruptRoot = buildFixture("hook-contracts-intent-corrupt-");
  const corruptControl = buildFixture("hook-contracts-intent-corrupt-control-");
  fs.mkdirSync(path.join(corruptRoot, ".bee", "intent"), { recursive: true });
  fs.writeFileSync(path.join(corruptRoot, ".bee", "intent", "demo.json"), "{not json at all");
  const rCorrupt = await runWrapper(
    "bee-session-init.mjs",
    sessionStartPayload({ sessionId: "sess-anchor", source: "compact", cwd: corruptRoot }),
    corruptRoot,
  );
  const rCorruptControl = await runWrapper(
    "bee-session-init.mjs",
    sessionStartPayload({ sessionId: "sess-anchor", source: "compact", cwd: corruptControl }),
    corruptControl,
  );
  const corruptPass = Boolean(
    rCorrupt.status === 0 &&
      rCorruptControl.status === 0 &&
      rCorrupt.stdout === rCorruptControl.stdout &&
      !(rCorrupt.stdout || "").includes("INTENT ANCHOR"),
  );
  rows.push(
    adapterRow(
      "session-init-intent",
      "corrupt-anchor-renders-exactly-like-no-anchor",
      corruptPass,
      corruptPass
        ? "an unparseable anchor file fails open to silence — byte-identical to a repo that has none (D5); a half-anchor is never handed to a summarizer"
        : `expected byte-identical fail-open silence; got status=${rCorrupt.status} equal=${rCorrupt.stdout === rCorruptControl.stdout} stdout=${truncate(rCorrupt.stdout, 400)}`,
      rCorrupt,
    ),
  );
  fs.rmSync(corruptRoot, { recursive: true, force: true });
  fs.rmSync(corruptControl, { recursive: true, force: true });

  // ── the pin that matters most: handoff adoption is UNCHANGED. A compacted
  // session with an anchor AND a planned-next handoff still never adopts.
  const adoptRoot = buildPlannedNextFixture("hook-contracts-intent-handoff-");
  writeIntentFileFixture(adoptRoot, "demo");
  const rAdopt = await runWrapper(
    "bee-session-init.mjs",
    sessionStartPayload({ sessionId: "sess-anchor", source: "compact", cwd: adoptRoot }),
    adoptRoot,
  );
  const adoptClaim = readJsonFileOrNull(path.join(adoptRoot, ".bee", "claims", "next-fsh10.json"));
  const adoptPass = Boolean(
    rAdopt.status === 0 &&
      (rAdopt.stdout || "").startsWith("## INTENT ANCHOR — read this FIRST") &&
      /HANDOFF present — present it and WAIT/.test(rAdopt.stdout || "") &&
      !/PLANNED-NEXT ADOPTED/.test(rAdopt.stdout || "") &&
      fs.existsSync(path.join(adoptRoot, ".bee", "HANDOFF.json")) &&
      adoptClaim &&
      adoptClaim.session === "sess-writer",
  );
  rows.push(
    adapterRow(
      "session-init-intent",
      "anchor-never-changes-handoff-adoption-on-compact",
      adoptPass,
      adoptPass
        ? "an anchor LEADS a compacted session's block and still changes nothing about adoption — the planned-next handoff stays on disk, the claim owner is unchanged, the wait block renders (D4: the anchor is a prefix, never a router)"
        : `expected an anchor-led wait block with the handoff/claim untouched; got status=${rAdopt.status} claim=${JSON.stringify(adoptClaim)} stdout=${truncate(rAdopt.stdout, 500)}`,
      rAdopt,
    ),
  );
  fs.rmSync(adoptRoot, { recursive: true, force: true });
  fs.rmSync(preRoot, { recursive: true, force: true });
  fs.rmSync(preControl, { recursive: true, force: true });

  return rows;
}

async function runCoverageGapRows() {
  const rows = [];

  // -- gap class: malformed-payload (top-level null) + source threading -----
  const f1 = buildFixture("hook-contracts-gap-malformed-");
  const g1 = await runWrapper("bee-session-init.mjs", "null", f1, ["--source=plugin"]);
  const line1 = findGapLine(f1, "session-init", "malformed-payload");
  const g1pass = Boolean(g1.status === 0 && line1 && line1.source === "plugin");
  rows.push(
    adapterRow(
      "coverage-gap",
      "malformed-payload-logged",
      g1pass,
      g1pass
        ? 'top-level null fails open AND lands a visible coverage-gap line (gap "malformed-payload") carrying the explicit --source identity'
        : `expected exit 0 plus a hooks.jsonl coverage-gap line with gap="malformed-payload" and source="plugin"; got status=${g1.status} line=${JSON.stringify(line1)}`,
      g1,
    ),
  );

  // -- gap class: invalid-cwd ------------------------------------------------
  const f2 = buildFixture("hook-contracts-gap-cwd-");
  const g2 = await runWrapper("bee-session-init.mjs", JSON.stringify({ cwd: { not: "a string" } }), f2);
  const line2 = findGapLine(f2, "session-init", "invalid-cwd");
  const g2pass = Boolean(g2.status === 0 && line2);
  rows.push(
    adapterRow(
      "coverage-gap",
      "invalid-cwd-logged",
      g2pass,
      g2pass
        ? 'non-string cwd fails open AND lands a visible coverage-gap line (gap "invalid-cwd")'
        : `expected exit 0 plus a hooks.jsonl coverage-gap line with gap="invalid-cwd"; got status=${g2.status} line=${JSON.stringify(line2)}`,
      g2,
    ),
  );

  // -- gap class: invalid-source ----------------------------------------------
  const f3 = buildFixture("hook-contracts-gap-source-");
  const g3 = await runWrapper("bee-session-init.mjs", JSON.stringify({ cwd: f3 }), f3, [
    "--source=weird",
  ]);
  const line3 = findGapLine(f3, "session-init", "invalid-source");
  const g3pass = Boolean(g3.status === 0 && line3);
  rows.push(
    adapterRow(
      "coverage-gap",
      "invalid-source-logged",
      g3pass,
      g3pass
        ? 'an unknown --source identity is recorded as a visible coverage-gap line (gap "invalid-source"), never a behavior change'
        : `expected exit 0 plus a hooks.jsonl coverage-gap line with gap="invalid-source"; got status=${g3.status} line=${JSON.stringify(line3)}`,
      g3,
    ),
  );

  // -- gap class: applypatch-unparsed -> DENY (P1 repair, cell codex-parity-4,
  // plan-review third bullet) --------------------------------------------------
  // codex-parity-3/-4 boundary, RETARGETED (the ONE sanctioned expectation
  // change in this suite, per codex-parity-4's cell): an intercepted
  // apply_patch (a canonical "*** Begin Patch" envelope was found) whose
  // targets cannot be proved — here, zero Add/Update/Delete/Move/"Move to"
  // lines parsed at all — now DENIES (exit 2) instead of failing open. The
  // visible coverage-gap line is still logged either way (D2: "visible
  // limits"); only the allow/deny outcome strengthens. This is honestly a
  // behavior change, not a weakening: it was fail-open under cell
  // codex-parity-3, and codex-parity-4's plan-reviewed P1 repair explicitly
  // retargets it to deny.
  const f4 = buildFixture("hook-contracts-gap-applypatch-");
  const g4 = await runWrapper(
    "bee-write-guard.mjs",
    JSON.stringify({
      hook_event_name: "PreToolUse",
      tool_name: "apply_patch",
      tool_input: { input: "*** Begin Patch\njunk hunk with no file verbs\n*** End Patch" },
      cwd: f4,
    }),
    f4,
  );
  const line4 = findGapLine(f4, "write-guard", "applypatch-unparsed");
  const g4pass = Boolean(
    g4.status === 2 && typeof g4.stderr === "string" && g4.stderr.trim() && line4,
  );
  rows.push(
    adapterRow(
      "coverage-gap",
      "applypatch-unparsed-logged",
      g4pass,
      g4pass
        ? 'an intercepted apply_patch with no provable target is DENIED (exit 2, corrective stderr) and still logs a visible coverage-gap line (gap "applypatch-unparsed") — P1 repair, cell codex-parity-4'
        : `expected exit 2 with a deny reason plus a hooks.jsonl coverage-gap line with gap="applypatch-unparsed"; got status=${g4.status} stderr=${truncate(g4.stderr, 300)} line=${JSON.stringify(line4)}`,
      g4,
    ),
  );

  // -- invariant: a log-write failure never flips a DENY into an allow --------
  // .bee/logs is created as a regular FILE so every mkdir/append inside the
  // logging path fails. The patch has one unprovable target (gap log attempt
  // fails) AND one provable direct-edit-denied target — the deny must survive.
  const f5 = buildFixture("hook-contracts-logfail-deny-");
  fs.writeFileSync(path.join(f5, ".bee", "logs"), "not a directory\n");
  const g5 = await runWrapper(
    "bee-write-guard.mjs",
    JSON.stringify({
      hook_event_name: "PreToolUse",
      tool_name: "apply_patch",
      tool_input: {
        input:
          "*** Begin Patch\n*** Update File: /etc/outside-the-repo\n@@\n-a\n+b\n*** Update File: .bee/state.json\n@@\n-old\n+new\n*** End Patch",
      },
      cwd: f5,
    }),
    f5,
  );
  const g5pass = Boolean(
    g5.status === 2 && typeof g5.stderr === "string" && g5.stderr.includes("bee.mjs state"),
  );
  rows.push(
    adapterRow(
      "coverage-gap",
      "log-write-failure-never-flips-deny",
      g5pass,
      g5pass
        ? "with .bee/logs unwritable, the computed deny (exit 2, direct-edit reason) still stands"
        : `expected exit 2 naming bee.mjs state despite the unwritable log path; got status=${g5.status} stderr=${truncate(g5.stderr, 300)}`,
      g5,
    ),
  );

  // -- invariant: a log-write failure never flips an ALLOW into a deny --------
  const f6 = buildFixture("hook-contracts-logfail-allow-");
  fs.writeFileSync(path.join(f6, ".bee", "logs"), "not a directory\n");
  const g6 = await runWrapper("bee-session-init.mjs", "null", f6);
  const g6pass = g6.status === 0;
  rows.push(
    adapterRow(
      "coverage-gap",
      "log-write-failure-never-flips-allow",
      g6pass,
      g6pass
        ? "with .bee/logs unwritable, malformed input still fails open (exit 0) — the gap-log failure is swallowed"
        : `expected exit 0 despite the unwritable log path; got status=${g6.status} stderr=${truncate(g6.stderr, 300)}`,
      g6,
    ),
  );

  return rows;
}

// --- installed-route rows (cell codex-parity-6b) ---------------------------
//
// Every row above spawns a wrapper DIRECTLY (`node hooks/bee-*.mjs`). That is
// not how Codex runs a hook, and a direct-wrapper test is exactly what let the
// original incident ship: the wrappers were fine, the INSTALLED COMMAND STRING
// was broken (`node "$CLAUDE_PROJECT_DIR"/.bee/bin/hooks/...` with
// $CLAUDE_PROJECT_DIR unset on Codex collapsed to `node /.bee/bin/hooks/...`
// -> MODULE_NOT_FOUND, every hook dead).
//
// So these rows take the OTHER route: they read the ACTIVE .codex/hooks.json,
// take each configured command string VERBATIM, and execute it the way Codex
// does — `bash -lc <command>` with the session cwd and the event payload on
// stdin (validation-codex-repo-fallback-split.md A1: the shipped Codex binary
// spawns `$SHELL -lc`; A2: commands run with the session cwd). `bash` is
// pinned rather than the developer's real $SHELL: a fish/nu $SHELL cannot run
// this POSIX transport, and that gap is declared, not silently "passed".
//
// Isolation (all of it load-bearing):
//   - a throwaway git fixture whose ROOT PATH CONTAINS SPACES AND UNICODE, so
//     an unquoted `$r` in any command string is caught here rather than in a
//     user's checkout;
//   - a NESTED cwd inside it, so git-root resolution is genuinely exercised;
//   - CLAUDE_PROJECT_DIR / CLAUDE_PLUGIN_ROOT UNSET — the incident's condition;
//   - HOME and CODEX_HOME pointed at the fixture, so a login shell cannot read
//     an rc file that `cd`s the process back into the live repo and lets
//     git-root resolution escape the fixture;
//   - the live repo's own .bee/state.json, .bee/.inject-cache.json and
//     .bee/logs/hooks.jsonl are hashed before and after: these rows drive REAL
//     state-mutating hooks, and they must not touch the running project.
//
// RED sensitivity is mechanical, not asserted: `--config-ref=<git-ref>` feeds
// the SAME rows the .codex/hooks.json of another commit. Against the pre-6a
// config (ba31819…) the transport is the broken one and these rows go RED.

const ROUTE_SHELL = fs.existsSync("/bin/bash") ? "/bin/bash" : "bash";
const CODEX_REPO_HOOKS_REPO_RELPATH = ".codex/hooks.json";

// Executables the login shell + the transport legitimately need. `git` is
// deliberately NOT here: the shim directory built from this list is what makes
// the git-absent row real.
const SHIM_BINARIES = Object.freeze([
  "node",
  "bash",
  "sh",
  "env",
  "cat",
  "id",
  "uname",
  "dirname",
  "basename",
  "sed",
  "grep",
  "tr",
  "expr",
  "mktemp",
  "tput",
  "locale",
  "which",
]);

function routeRow(id, pass, note, extra = {}) {
  return genericRow("repo-route", id, pass, note, extra);
}

// Read the configured commands from the active worktree file, or (bounded
// sensitivity option) from `<ref>:.codex/hooks.json`.
function readRepoHooksConfig(configRef) {
  if (!configRef) {
    return { text: fs.readFileSync(CODEX_REPO_HOOKS_PATH, "utf8"), origin: "worktree" };
  }
  const shown = spawnSync("git", ["show", `${configRef}:${CODEX_REPO_HOOKS_REPO_RELPATH}`], {
    cwd: REPO_ROOT,
    encoding: "utf8",
    timeout: SPAWN_TIMEOUT_MS,
    maxBuffer: SPAWN_MAX_BUFFER,
  });
  if (shown.status !== 0 || typeof shown.stdout !== "string") {
    throw new Error(
      `cannot read ${CODEX_REPO_HOOKS_REPO_RELPATH} at ref "${configRef}": ` +
        `status=${shown.status} ${truncate(shown.stderr, 300)}`,
    );
  }
  return { text: shown.stdout, origin: `ref ${configRef}` };
}

// Flatten the config into the exact list of command strings Codex would run.
function parseConfiguredCommands(configText) {
  const config = JSON.parse(configText);
  const commands = [];
  for (const [event, groups] of Object.entries(config.hooks || {})) {
    (groups || []).forEach((group, gi) => {
      (group.hooks || []).forEach((entry, hi) => {
        const command = entry && typeof entry.command === "string" ? entry.command : "";
        const scriptMatch = command.match(/bee-([a-z-]+)\.mjs/);
        const script = scriptMatch ? scriptMatch[1] : `unknown-${gi}-${hi}`;
        commands.push({
          event,
          matcher: group.matcher ?? null,
          script,
          command,
          id: `${event}:${script}`,
        });
      });
    });
  }
  return commands;
}

// An apply_patch envelope whose target is written RELATIVE TO THE SESSION CWD
// — which is how Codex's apply_patch addresses files. From the fixture root
// that is `.bee/state.json`; from the nested cwd it is `../../.bee/state.json`.
// Getting this right matters twice over: it is what the real tool sends, and
// it makes the row prove that the guard RESOLVES the target (a guard that
// merely string-matched ".bee/state.json" would deny the nested case for the
// wrong reason, and would also deny an innocent nested `.bee/state.json` that
// is not the bee state file at all).
function routeApplyPatchPayload(cwd, fixtureRoot) {
  const target = path
    .relative(cwd, path.join(fixtureRoot, ".bee", "state.json"))
    .split(path.sep)
    .join("/");
  return JSON.stringify({
    hook_event_name: "PreToolUse",
    tool_name: "apply_patch",
    tool_input: {
      input: `*** Begin Patch\n*** Update File: ${target}\n@@\n-old\n+new\n*** End Patch`,
    },
    cwd,
  });
}

// The event payload Codex would put on the hook's stdin.
function routePayload(event, cwd, fixtureRoot) {
  switch (event) {
    case "PreToolUse":
      return routeApplyPatchPayload(cwd, fixtureRoot);
    case "PostToolUse":
      return JSON.stringify({ hook_event_name: "PostToolUse", tool_name: "TodoWrite", cwd });
    case "SubagentStart":
      return JSON.stringify({
        hook_event_name: "SubagentStart",
        session_id: "session-route",
        agent_id: "agent-route",
        agent_type: "worker",
        cwd,
      });
    case "SubagentStop":
      return JSON.stringify({
        hook_event_name: "SubagentStop",
        session_id: "session-route",
        agent_id: "agent-route",
        agent_name: "kevin",
        agent_type: "worker",
        cwd,
      });
    case "SessionStart":
      return JSON.stringify({ hook_event_name: "SessionStart", source: "startup", cwd });
    case "UserPromptSubmit":
      return JSON.stringify({ hook_event_name: "UserPromptSubmit", prompt: "status?", cwd });
    default:
      return JSON.stringify({ hook_event_name: event, cwd });
  }
}

// PreToolUse now carries TWO configured commands (cnr2-8): bee-write-guard
// DENIES a gated apply_patch, while bee-model-guard (Codex spawn guard, matcher
// spawn_agent) is not a dispatch tool for an apply_patch payload and must FAIL
// OPEN. So the PreToolUse contract is keyed on the script, not the event alone;
// every other event is fail-open/advisory.
function routeExpectation(event, script) {
  if (event === "PreToolUse") {
    return script === "write-guard" ? expectApplyPatchDenied : expectNoCrash;
  }
  if (ADVISORY_ROUTE_EVENTS.includes(event)) return expectAdvisoryJsonOrSilent;
  return expectNoCrash;
}

const ADVISORY_ROUTE_EVENTS = Object.freeze(["PreCompact", "SubagentStop", "Stop"]);

function buildRouteFixture() {
  // Spaces AND Unicode in the root path — an unquoted "$r" dies here.
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "bee route ✦ fixture ")));
  const home = path.join(root, ".fixture-home");
  const codexHome = path.join(root, ".fixture-codex-home");
  const nested = path.join(root, "src", "deep nest ✦");
  const shim = path.join(root, ".shim-no-git");
  const routeHooksDir = path.join(root, ".bee", "bin", "hooks");
  for (const dir of [home, codexHome, nested, shim, routeHooksDir]) {
    fs.mkdirSync(dir, { recursive: true });
  }

  // The wrappers the configured commands actually exec (the committed,
  // git-tracked .bee/bin/hooks/ vendored location the repo target resolves
  // against), plus the shared libs they import at runtime.
  for (const name of fs.readdirSync(HOOKS_DIR)) {
    if (name.endsWith(".mjs")) {
      fs.copyFileSync(path.join(HOOKS_DIR, name), path.join(routeHooksDir, name));
    }
  }
  copyLib(root);
  fs.writeFileSync(path.join(root, ".bee", "onboarding.json"), "{}\n");
  fs.writeFileSync(
    path.join(root, ".bee", "state.json"),
    `${JSON.stringify(
      {
        phase: "swarming",
        mode: "standard",
        feature: "demo",
        approved_gates: { context: true, shape: true, execution: true, review: false },
      },
      null,
      2,
    )}\n`,
  );

  const gitInit = spawnSync("git", ["init", "-q", root], {
    encoding: "utf8",
    env: { ...process.env, HOME: home },
    timeout: SPAWN_TIMEOUT_MS,
  });
  if (gitInit.status !== 0) {
    throw new Error(
      `route fixture: git init failed: status=${gitInit.status} ${gitInit.stderr || gitInit.error?.message || ""}`,
    );
  }

  // The git-absent shim: node + bash + the login shell's usual coreutils, and
  // NO git. `command -v git` under this PATH must come back empty — a row
  // asserts exactly that before any git-absent conclusion is drawn.
  //
  // Two platform shapes for ONE contract (a PATH value carrying bash + node
  // + coreutils but no git):
  //   - POSIX: a directory of symlinks to the resolved binaries (unchanged).
  //   - win32: symlink creation needs elevation AND extension-less symlink
  //     names are not spawnable via CreateProcess, so the shim is instead a
  //     PATH *string* of two REAL directories: Git-for-Windows' usr\bin
  //     (bash.exe + coreutils, NO git.exe lives there) plus node's own dir.
  //     The login profile would re-prepend /mingw64/bin (where git.exe DOES
  //     live) when MSYSTEM is set, so routeEnv() strips the MSYSTEM family
  //     for the git-absent arm — verified empirically: with that strip,
  //     `bash -lc 'command -v git'` under this PATH is empty while node,
  //     bash and the coreutils all resolve.
  let shimPath = shim;
  if (process.platform === "win32") {
    shimPath = null;
    const whereGit = spawnSync("where", ["git"], { encoding: "utf8", timeout: 5000 });
    const gitExe = (whereGit.stdout || "").split(/\r?\n/).map((l) => l.trim()).find(Boolean);
    // Walk up from git.exe to the Git-for-Windows root (git.exe may live in
    // cmd\, bin\ or mingw64\bin\) — the root is the ancestor carrying
    // usr\bin\bash.exe.
    for (let dir = gitExe ? path.dirname(gitExe) : null; dir; ) {
      const usrBin = path.join(dir, "usr", "bin");
      if (fs.existsSync(path.join(usrBin, "bash.exe")) && !fs.existsSync(path.join(usrBin, "git.exe"))) {
        shimPath = `${usrBin}${path.delimiter}${path.dirname(process.execPath)}`;
        break;
      }
      const parent = path.dirname(dir);
      dir = parent === dir ? null : parent;
    }
    // If no git-free bash directory exists on this box, leave shimPath at the
    // empty shim dir: the honest route-gitabsent-shim-precondition row below
    // reports exactly why the git-absent rows cannot be trusted here.
    if (shimPath === null) shimPath = shim;
  } else {
    for (const bin of SHIM_BINARIES) {
      const found = spawnSync("sh", ["-c", `command -v ${bin}`], { encoding: "utf8", timeout: 5000 });
      const resolved = (found.stdout || "").trim();
      if (found.status === 0 && resolved && path.basename(resolved) !== "git") {
        try {
          fs.symlinkSync(resolved, path.join(shim, bin));
        } catch {
          // a missing optional utility is not fatal — the shim only has to carry
          // enough for `bash -lc` and `node`
        }
      }
    }
  }

  // A directory that is inside NO git repository (the "non-git cwd" arm).
  const nonGit = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "bee route non-git ")));

  return { root, home, codexHome, nested, shim: shimPath, nonGit };
}

// The environment Codex would hand the hook — minus the two Claude-only root
// variables, with HOME/CODEX_HOME caged inside the fixture.
function routeEnv(fixture, { noGit = false } = {}) {
  const env = { ...process.env };
  delete env.CLAUDE_PROJECT_DIR;
  delete env.CLAUDE_PLUGIN_ROOT;
  env.HOME = fixture.home;
  env.CODEX_HOME = fixture.codexHome;
  if (noGit) {
    env.PATH = fixture.shim;
    if (process.platform === "win32") {
      // A Git-Bash login shell re-prepends /mingw64/bin (which carries
      // git.exe) whenever the MSYSTEM family is set — stripping it keeps the
      // git-absent PATH honest (see the shim-construction comment in
      // buildRouteFixture).
      delete env.MSYSTEM;
      delete env.MSYS2_PATH_TYPE;
      delete env.ORIGINAL_PATH;
      delete env.MINGW_PREFIX;
    }
  }
  return env;
}

function runRouteCommand(command, cwd, input, env) {
  // The managed Codex sandbox can attach EPERM metadata even when the real
  // Bash command ran and returned a concrete status. When Bash's final
  // `exec node ...` replaces the shell, pipe capture can additionally lose
  // that hook's stdout/stderr. File descriptors keep the single real
  // Bash -> installed Node command boundary intact while preserving the
  // streams every existing route assertion grades.
  const captureDir = fs.mkdtempSync(path.join(os.tmpdir(), "bee-route-output-"));
  const stdinPath = path.join(captureDir, "stdin");
  const stdoutPath = path.join(captureDir, "stdout");
  const stderrPath = path.join(captureDir, "stderr");
  fs.writeFileSync(stdinPath, input === undefined || input === null ? "" : String(input));
  const stdinFd = fs.openSync(stdinPath, "r");
  const stdoutFd = fs.openSync(stdoutPath, "w");
  const stderrFd = fs.openSync(stderrPath, "w");
  try {
    const result = spawnSync(ROUTE_SHELL, ["-lc", command], {
      cwd,
      env,
      encoding: "utf8",
      stdio: [stdinFd, stdoutFd, stderrFd],
      timeout: SPAWN_TIMEOUT_MS,
      maxBuffer: SPAWN_MAX_BUFFER,
    });
    fs.closeSync(stdinFd);
    fs.closeSync(stdoutFd);
    fs.closeSync(stderrFd);
    return {
      ...result,
      stdout: fs.readFileSync(stdoutPath, "utf8"),
      stderr: fs.readFileSync(stderrPath, "utf8"),
    };
  } finally {
    try {
      fs.closeSync(stdinFd);
    } catch {}
    try {
      fs.closeSync(stdoutFd);
    } catch {}
    try {
      fs.closeSync(stderrFd);
    } catch {}
    fs.rmSync(captureDir, { recursive: true, force: true });
  }
}

function runGeneratedRepoAuditRows() {
  const projection = renderProjection(RUNTIMES.CODEX, { target: TARGETS.REPO });
  const commands = parseConfiguredCommands(`${JSON.stringify(projection, null, 2)}\n`).filter(
    (entry) => entry.script === "codex-subagent-audit",
  );
  const fixture = buildRouteFixture();
  try {
    const expectedEvents = ["SubagentStart", "SubagentStop"];
    const results = [];
    for (const event of expectedEvents) {
      const configured = commands.filter((entry) => entry.event === event);
      if (configured.length !== 1) {
        return [
          catalogDriftRow(
            "codex-generated-repo-audit-execution",
            false,
            `generated repo projection expected one ${event} audit command, found ${configured.length}`,
          ),
        ];
      }
      const cwd = event === "SubagentStart" ? fixture.root : fixture.nested;
      results.push({
        event,
        result: runRouteCommand(
          configured[0].command,
          cwd,
          routePayload(event, cwd, fixture.root),
          routeEnv(fixture),
        ),
      });
    }
    const auditLines = readHooksJsonl(fixture.root).filter(
      (line) => line.hook === "codex-subagent-audit" && line.event === "subagent-audit",
    );
    const startLine = auditLines.find((line) => line.lifecycle === "start");
    const stopLine = auditLines.find((line) => line.lifecycle === "stop");
    const pass =
      results.every(
        ({ result }) => result.status === 0 && result.stdout === "" && result.stderr === "",
      ) &&
      auditLines.length === 2 &&
      startLine &&
      stopLine &&
      startLine.source === "repo" &&
      stopLine.source === "repo" &&
      startLine.agent_id === "agent-route" &&
      stopLine.agent_id === "agent-route";
    return [
      catalogDriftRow(
        "codex-generated-repo-audit-execution",
        Boolean(pass),
        pass
          ? "generated repo-target SubagentStart/Stop commands execute fixture-locally from root/nested cwd and write paired source=repo audit records"
          : `generated repo audit execution failed: results=${JSON.stringify(results.map(({ event, result }) => ({ event, status: result.status, stdout: result.stdout, stderr: result.stderr })))} lines=${JSON.stringify(auditLines)}`,
      ),
    ];
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
    fs.rmSync(fixture.nonGit, { recursive: true, force: true });
  }
}

// The pinned fail-open contract (codex-parity-6a): exit 0, stdout EMPTY,
// stderr carrying the exact diagnostic literal. Login-shell noise (an
// /etc/profile.d script complaining about a utility the shim does not carry)
// may share stderr — the literal must be PRESENT, not alone.
function expectTransportFailOpen(result) {
  const stdout = result.stdout || "";
  const stderr = result.stderr || "";
  const problems = [];
  if (result.status !== 0) problems.push(`expected exit 0 (fail-open), got status=${result.status}`);
  if (result.status == null && result.error) problems.push(`launch error: ${result.error.message}`);
  if (stdout.trim()) problems.push(`expected EMPTY stdout, got: ${truncate(stdout, 300)}`);
  if (!stderr.includes(REPO_TRANSPORT_UNAVAILABLE_DIAGNOSTIC)) {
    problems.push(
      `stderr does not carry the pinned literal "${REPO_TRANSPORT_UNAVAILABLE_DIAGNOSTIC}"; got: ${truncate(stderr, 300)}`,
    );
  }
  if (problems.length > 0) return { pass: false, note: problems.join(" | ") };
  return {
    pass: true,
    note: `transport fails open VISIBLY: exit 0, empty stdout, stderr carries "${REPO_TRANSPORT_UNAVAILABLE_DIAGNOSTIC}"`,
  };
}

function liveStateFingerprint() {
  const beeDir = path.join(REPO_ROOT, ".bee");
  const fingerprint = {};
  for (const rel of ["state.json", path.join("cache", "inject-cache.json"), path.join("logs", "hooks.jsonl")]) {
    const file = path.join(beeDir, rel);
    if (!fs.existsSync(file)) {
      fingerprint[rel] = "ABSENT";
      continue;
    }
    const bytes = fs.readFileSync(file);
    fingerprint[rel] = `${bytes.length}:${createHash("sha256").update(bytes).digest("hex")}`;
  }
  return fingerprint;
}

// "Rerun a parseable plugin census and block if a bee plugin appeared": these
// rows install nothing, and the default suite's marketplace rows run under an
// isolated CODEX_HOME. If a bee plugin has nevertheless landed in the REAL
// Codex home, the isolation claim is false and the run must say so.
function runPluginCensusRow() {
  const detect = detectCodexCli();
  if (!detect.present) {
    return routeRow(
      "route-plugin-census",
      true,
      `SKIP: codex CLI not found on PATH (${detect.reason}) — plugin census cannot run here`,
      { skip: true },
    );
  }
  const listed = spawnSync("codex", ["plugin", "list", "--json"], {
    encoding: "utf8",
    timeout: 30000,
    maxBuffer: SPAWN_MAX_BUFFER,
  });
  let json = null;
  try {
    json = JSON.parse((listed.stdout || "").trim());
  } catch {
    json = null;
  }
  if (listed.status !== 0 || !json || typeof json !== "object") {
    return routeRow(
      "route-plugin-census",
      false,
      "plugin census is not parseable: " +
        `status=${listed.status} stdout=${truncate(listed.stdout, 300)} stderr=${truncate(listed.stderr, 300)}`,
      listed,
    );
  }
  const entries = Object.values(json)
    .filter(Array.isArray)
    .flat()
    .filter((p) => p && typeof p === "object");
  const bee = entries.find(
    (p) => p.name === "bee" || (typeof p.pluginId === "string" && p.pluginId.startsWith("bee@")),
  );
  return routeRow(
    "route-plugin-census",
    !bee,
    bee
      ? `A BEE PLUGIN APPEARED in the real Codex home (${bee.pluginId || bee.name}) — these rows install nothing; isolation is broken`
      : `plugin census parses (${entries.length} entr(y|ies)) and carries no bee plugin — nothing was installed into the real Codex home`,
    listed,
  );
}

// The REQUIRED-ROW MANIFEST for the route group, computed from the CONFIG —
// deliberately NOT from the rows that were actually produced. If the manifest
// were assembled as a side effect of building rows, then deleting a whole arm
// (say, the git-absent one) would delete its own requirement along with it and
// the suite would go green on less work. Derived this way, every command in
// .codex/hooks.json owes four arm rows and every Stop command owes a handler
// row: drop one and the run reports REQUIRED ROW ABSENT and exits non-zero.
function routeRequiredRowIds(commands, { configRef = null } = {}) {
  const ids = [
    "route-config-readable",
    "route-config-thirteen-commands",
    "route-gitabsent-shim-precondition",
    "route-nongit-cwd-precondition",
    "route-pretooluse-command-present",
    "route-pretooluse-applypatch-deny",
    "route-live-state-untouched",
  ];
  if (configRef) ids.push("route-config-ref-differs-from-worktree");
  for (const cmd of commands) {
    ids.push(
      `route-root:${cmd.id}`,
      `route-nested:${cmd.id}`,
      `route-nongit:${cmd.id}`,
      `route-gitabsent:${cmd.id}`,
    );
  }
  for (const cmd of commands.filter((c) => c.event === "Stop")) {
    ids.push(`route-stop-handler:${cmd.script}`);
  }
  return ids;
}

// Returns { rows, requiredIds }.
function runRepoRouteRows({ configRef = null } = {}) {
  const rows = [];

  let config;
  try {
    config = readRepoHooksConfig(configRef);
  } catch (error) {
    rows.push(routeRow("route-config-readable", false, String(error.message)));
    return { rows, requiredIds: routeRequiredRowIds([], { configRef }) };
  }
  rows.push(
    routeRow(
      "route-config-readable",
      true,
      `configured Codex commands read from ${config.origin} (${CODEX_REPO_HOOKS_REPO_RELPATH})`,
    ),
  );

  // RED-sensitivity guard (validation blocker 6b-C2): a --config-ref run only
  // MEANS anything if that ref's config actually differs from the active one.
  // If they are byte-identical, the "RED" would be fabricated — fail loudly
  // instead of reporting a red/green split that does not exist.
  if (configRef) {
    const worktree = fs.readFileSync(CODEX_REPO_HOOKS_PATH, "utf8");
    const differs = worktree !== config.text;
    rows.push(
      routeRow(
        "route-config-ref-differs-from-worktree",
        differs,
        differs
          ? `sensitivity ref "${configRef}" carries a DIFFERENT .codex/hooks.json than the worktree — a RED here is a real RED`
          : `sensitivity ref "${configRef}" is BYTE-IDENTICAL to the worktree .codex/hooks.json — no red/green split exists at this ref; any "RED" reported from it would be fabricated`,
      ),
    );
  }

  const commands = parseConfiguredCommands(config.text);
  const requiredIds = routeRequiredRowIds(commands, { configRef });
  rows.push(
    routeRow(
      "route-config-thirteen-commands",
      commands.length === 13,
      commands.length === 13
        ? `all 13 configured commands loaded from ${config.origin} and exercised through ${ROUTE_SHELL} -lc`
        : `expected 13 configured commands, found ${commands.length} (${commands.map((c) => c.id).join(", ")})`,
    ),
  );

  const before = liveStateFingerprint();
  const fixture = buildRouteFixture();

  try {
    // -- precondition: the shim really removes git, and keeps node ------------
    const shimEnv = routeEnv(fixture, { noGit: true });
    const gitProbe = runRouteCommand("command -v git", fixture.nested, "", shimEnv);
    const nodeProbe = runRouteCommand("command -v node", fixture.nested, "", shimEnv);
    const shimOk = !(gitProbe.stdout || "").trim() && Boolean((nodeProbe.stdout || "").trim());
    rows.push(
      routeRow(
        "route-gitabsent-shim-precondition",
        shimOk,
        shimOk
          ? `PATH shim ${fixture.shim} resolves node but NOT git — the git-absent rows below are real`
          : `git-absent shim is not honest: command -v git => "${truncate(gitProbe.stdout, 120)}", ` +
              `command -v node => "${truncate(nodeProbe.stdout, 120)}" (a naive PATH shim that still exposes git would make every git-absent row a false PASS)`,
        gitProbe,
      ),
    );

    // -- precondition: the non-git cwd is really outside any git repo ---------
    const nonGitProbe = spawnSync("git", ["rev-parse", "--show-toplevel"], {
      cwd: fixture.nonGit,
      encoding: "utf8",
      timeout: SPAWN_TIMEOUT_MS,
    });
    const nonGitOk = nonGitProbe.status !== 0 && !(nonGitProbe.stdout || "").trim();
    rows.push(
      routeRow(
        "route-nongit-cwd-precondition",
        nonGitOk,
        nonGitOk
          ? `${fixture.nonGit} is inside no git repository — the non-git rows below are real`
          : `the "non-git" cwd resolves a git root (${truncate(nonGitProbe.stdout, 200)}) — the non-git rows would be false PASSes`,
        nonGitProbe,
      ),
    );

    const env = routeEnv(fixture);

    for (const cmd of commands) {
      // -- arm 1: repo ROOT cwd ------------------------------------------------
      const atRoot = runRouteCommand(
        cmd.command,
        fixture.root,
        routePayload(cmd.event, fixture.root, fixture.root),
        env,
      );
      const rootVerdict = routeExpectation(cmd.event, cmd.script)(atRoot);
      rows.push(
        routeRow(
          `route-root:${cmd.id}`,
          rootVerdict.pass,
          `[cwd=fixture root] ${rootVerdict.note}`,
          atRoot,
        ),
      );

      // -- arm 2: NESTED cwd (git-root resolution genuinely exercised) ---------
      const atNested = runRouteCommand(
        cmd.command,
        fixture.nested,
        routePayload(cmd.event, fixture.nested, fixture.root),
        env,
      );
      const nestedVerdict = routeExpectation(cmd.event, cmd.script)(atNested);
      rows.push(
        routeRow(
          `route-nested:${cmd.id}`,
          nestedVerdict.pass,
          `[cwd=nested, spaces+Unicode path] ${nestedVerdict.note}`,
          atNested,
        ),
      );

      // -- arm 3: NON-GIT cwd -> visible fail-open ------------------------------
      const atNonGit = runRouteCommand(
        cmd.command,
        fixture.nonGit,
        routePayload(cmd.event, fixture.nonGit, fixture.root),
        env,
      );
      const nonGitVerdict = expectTransportFailOpen(atNonGit);
      rows.push(
        routeRow(
          `route-nongit:${cmd.id}`,
          nonGitVerdict.pass,
          `[cwd=non-git dir] ${nonGitVerdict.note}`,
          atNonGit,
        ),
      );

      // -- arm 4: git ABSENT FROM PATH -> visible fail-open ---------------------
      //
      // THE row this cell exists for (validation blocker F2(b)): with `git`
      // off the PATH, `$(git rev-parse --show-toplevel)` collapses to the empty
      // string. That is precisely the condition that produced the original
      // MODULE_NOT_FOUND. Under the repaired transport it must fail open
      // VISIBLY; under the pre-fix transport it crashes.
      const atGitAbsent = runRouteCommand(
        cmd.command,
        fixture.nested,
        routePayload(cmd.event, fixture.nested, fixture.root),
        shimEnv,
      );
      const gitAbsentVerdict = expectTransportFailOpen(atGitAbsent);
      rows.push(
        routeRow(
          `route-gitabsent:${cmd.id}`,
          gitAbsentVerdict.pass,
          `[git shimmed OFF the PATH — the exact MODULE_NOT_FOUND branch] ${gitAbsentVerdict.note}`,
          atGitAbsent,
        ),
      );
    }

    // -- the two configured Stop handlers, asserted individually ---------------
    //
    // Both must exit 0. bee-session-close is the handler with an output path:
    // the fixture (mid-phase, no HANDOFF) FORCES its "hive door open" advisory,
    // so its stdout must be NON-EMPTY and must parse as a JSON systemMessage
    // with no decision:"block". bee-state-sync has no stdout path at all (it is
    // silent by contract, cell codex-parity-3) — silence there is proved to be
    // REAL WORK rather than a dead command by requiring that the route run
    // actually mutated the FIXTURE's .bee/state.json.
    const stopCommands = commands.filter((c) => c.event === "Stop");
    for (const cmd of stopCommands) {
      const fixtureState = path.join(fixture.root, ".bee", "state.json");
      const stateBefore = fs.readFileSync(fixtureState, "utf8");
      const result = runRouteCommand(
        cmd.command,
        fixture.nested,
        routePayload("Stop", fixture.nested, fixture.root),
        env,
      );
      const parsed = parseAdvisoryStdout(result.stdout);
      const exitOk = result.status === 0;
      const notBlocking = !parsed.json || parsed.json.decision !== "block";

      if (cmd.script === "state-sync") {
        const stateAfter = fs.readFileSync(fixtureState, "utf8");
        let mutated = false;
        try {
          mutated = Boolean(JSON.parse(stateAfter).last_activity) && stateAfter !== stateBefore;
        } catch {
          mutated = false;
        }
        const pass = exitOk && parsed.empty && mutated;
        rows.push(
          routeRow(
            `route-stop-handler:${cmd.script}`,
            pass,
            pass
              ? "Stop handler bee-state-sync ran through the installed command route: exit 0, silent by contract, " +
                  "and it actually refreshed the FIXTURE .bee/state.json (last_activity) — the silence is real work, not a dead command"
              : `Stop handler bee-state-sync: expected exit 0 + empty stdout + a mutated fixture .bee/state.json; ` +
                  `got status=${result.status} stdout=${truncate(result.stdout, 200)} stateMutated=${mutated} stderr=${truncate(result.stderr, 300)}`,
            result,
          ),
        );
        continue;
      }

      const hasSystemMessage =
        Boolean(parsed.json) &&
        typeof parsed.json === "object" &&
        !Array.isArray(parsed.json) &&
        typeof parsed.json.systemMessage === "string" &&
        parsed.json.systemMessage.trim().length > 0;
      const pass = exitOk && !parsed.empty && hasSystemMessage && notBlocking;
      rows.push(
        routeRow(
          `route-stop-handler:${cmd.script}`,
          pass,
          pass
            ? `Stop handler bee-${cmd.script} ran through the installed command route: exit 0, NON-EMPTY stdout parsing as ` +
                'a JSON systemMessage, no decision:"block"'
            : `Stop handler bee-${cmd.script}: expected exit 0 + non-empty stdout parsing as a JSON systemMessage with no ` +
                `decision:"block"; got status=${result.status} stdout=${truncate(result.stdout, 300)} stderr=${truncate(result.stderr, 300)}`,
          result,
        ),
      );
    }

    // -- the CONFIGURED PreToolUse command denies the gated apply_patch --------
    //
    // Not a direct wrapper call: the command string out of .codex/hooks.json,
    // through the login shell, must itself come back exit 2 with a reason.
    const preToolUse = commands.filter((c) => c.event === "PreToolUse");
    const writeGuardPre = preToolUse.filter((c) => c.script === "write-guard");
    const modelGuardPre = preToolUse.filter((c) => c.script === "model-guard");
    const preShapeOk = writeGuardPre.length === 1 && modelGuardPre.length === 1;
    rows.push(
      routeRow(
        "route-pretooluse-command-present",
        preShapeOk,
        preShapeOk
          ? "the two configured PreToolUse commands are exactly one write-guard (apply_patch deny) and one " +
              "model-guard (Codex spawn guard, matcher spawn_agent)"
          : `expected exactly 1 write-guard + 1 model-guard PreToolUse command, found ` +
              `write-guard=${writeGuardPre.length} model-guard=${modelGuardPre.length} (total ${preToolUse.length})`,
      ),
    );
    // Only the write-guard command carries the apply_patch DENY contract; the
    // spawn guard is exercised as a fail-open through the per-command arm loop
    // above (cnr2-8).
    for (const cmd of writeGuardPre) {
      const denied = runRouteCommand(
        cmd.command,
        fixture.nested,
        routeApplyPatchPayload(fixture.nested, fixture.root),
        env,
      );
      const verdict = expectApplyPatchDenied(denied);
      rows.push(
        routeRow(
          "route-pretooluse-applypatch-deny",
          verdict.pass,
          `[configured PreToolUse command, not a direct wrapper call] ${verdict.note}`,
          denied,
        ),
      );
    }

    // -- the live repository was not touched -----------------------------------
    const after = liveStateFingerprint();
    const drifted = Object.keys(before).filter((k) => before[k] !== after[k]);
    rows.push(
      routeRow(
        "route-live-state-untouched",
        drifted.length === 0,
        drifted.length === 0
          ? "live .bee/state.json hash and the presence/bytes of .bee/cache/inject-cache.json and .bee/logs/hooks.jsonl are " +
              "IDENTICAL before and after — these rows drove real state-mutating hooks entirely inside the fixture"
          : `LIVE STATE MUTATED by the route rows: ${drifted
              .map((k) => `${k} (${before[k]} -> ${after[k]})`)
              .join(", ")}`,
      ),
    );

    rows.push(runPluginCensusRow());

    return { rows, requiredIds };
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
    fs.rmSync(fixture.nonGit, { recursive: true, force: true });
  }
}

// --- main ----------------------------------------------------------------

// STRICT argv (cell codex-parity-6a). Previously ANY unrecognized flag was
// silently ignored and the runner fell through to the default suite, printed
// ALL PASS, and exited 0 — so a verify command naming a mode that was never
// implemented passed GREEN against a broken tree. An unknown flag is now a
// hard, non-zero error that names the modes it does know.
const KNOWN_FLAGS = Object.freeze(["--baseline", "--catalog-only", "--repo-route-only"]);
const CONFIG_REF_PREFIX = "--config-ref=";

function parseArgv(argv) {
  const unknown = argv.filter(
    (a) => !KNOWN_FLAGS.includes(a) && !(a.startsWith(CONFIG_REF_PREFIX) && a.length > CONFIG_REF_PREFIX.length),
  );
  if (unknown.length > 0) {
    return { ok: false, unknown };
  }
  const refArg = argv.find((a) => a.startsWith(CONFIG_REF_PREFIX));
  return {
    ok: true,
    baselineMode: argv.includes("--baseline"),
    catalogOnlyMode: argv.includes("--catalog-only"),
    repoRouteOnlyMode: argv.includes("--repo-route-only"),
    configRef: refArg ? refArg.slice(CONFIG_REF_PREFIX.length) : null,
  };
}

function reportRows(results, headline) {
  const failures = results.filter((r) => !r.pass);
  const skipped = results.filter((r) => r.skip);
  for (const r of results) {
    const mark = r.skip ? "SKIP" : r.pass ? "ok  " : "FAIL";
    process.stdout.write(`${mark} - ${r.wrapper} :: ${r.id} :: ${r.note}\n`);
  }
  process.stdout.write(
    `\n${headline}: ${results.length} rows (${skipped.length} skipped), ${failures.length} failing target contract\n`,
  );
  process.stdout.write(`\n${failures.length === 0 ? "ALL PASS" : `${failures.length} FAILURE(S)`}\n`);
  return failures.length;
}

async function main() {
  const parsed = parseArgv(process.argv.slice(2));
  if (!parsed.ok) {
    process.stderr.write(
      `test_hook_contracts: unknown flag(s): ${parsed.unknown.join(", ")}\n` +
        `known modes: ${KNOWN_FLAGS.join(", ")} (or no flag for the full default suite)\n`,
    );
    process.exitCode = 2;
    return;
  }
  const { baselineMode, catalogOnlyMode, repoRouteOnlyMode, configRef } = parsed;

  // Modes are mutually exclusive, and --config-ref is BOUNDED: it is a
  // sensitivity probe for the installed-route rows only. It can never be used
  // to make the default suite (or --catalog-only) grade itself against some
  // other commit's config.
  if ([baselineMode, catalogOnlyMode, repoRouteOnlyMode].filter(Boolean).length > 1) {
    process.stderr.write(
      "test_hook_contracts: --baseline, --catalog-only and --repo-route-only are mutually exclusive\n",
    );
    process.exitCode = 2;
    return;
  }
  if (configRef && !repoRouteOnlyMode) {
    process.stderr.write(
      `test_hook_contracts: ${CONFIG_REF_PREFIX}<ref> is only valid with --repo-route-only ` +
        "(it is a RED-sensitivity probe for the installed-route rows, not a way to grade any other suite " +
        "against a different commit's config)\n",
    );
    process.exitCode = 2;
    return;
  }

  // --repo-route-only (cell codex-parity-6b): run ONLY the installed-route
  // rows — every command in the ACTIVE .codex/hooks.json, executed the way
  // Codex executes it. Every row here is REQUIRED (ids derived from the config
  // itself): an absent or skipped one exits non-zero, so this mode cannot pass
  // by not doing the work.
  if (repoRouteOnlyMode) {
    const { rows, requiredIds } = runRepoRouteRows({ configRef });
    const results = [...rows];
    results.push(...enforceRequiredRows(results, requiredIds, "repo-route"));
    const failures = reportRows(
      results,
      `--repo-route-only${configRef ? ` (config-ref ${configRef})` : ""}`,
    );
    process.exitCode = failures === 0 ? 0 : 1;
    return;
  }

  // --catalog-only (cell codex-parity-2 verify mode; extended by cell
  // codex-parity-6a with the repo-target rows): run ONLY the catalog-drift +
  // codex-acceptance row groups. The bare default mode below keeps gating the
  // FULL seven-wrapper table (that table's green state is cell
  // codex-parity-3's exit target), and --baseline keeps cell codex-parity-1's
  // characterization contract untouched.
  if (catalogOnlyMode) {
    const results = [
      ...runCatalogDriftChecks(),
      ...runGeneratedRepoAuditRows(),
      ...runCodexAcceptanceRows(),
    ];
    results.push(...enforceRequiredRows(results, REQUIRED_CATALOG_ROW_IDS));
    const failures = results.filter((r) => !r.pass);
    const skipped = results.filter((r) => r.skip);
    for (const r of results) {
      const mark = r.skip ? "SKIP" : r.pass ? "ok  " : "FAIL";
      process.stdout.write(`${mark} - ${r.wrapper} :: ${r.id} :: ${r.note}\n`);
    }
    process.stdout.write(
      `\n--catalog-only: ${results.length} rows (${skipped.length} skipped), ${failures.length} failing target contract\n`,
    );
    process.stdout.write(
      `\n${failures.length === 0 ? "ALL PASS" : `${failures.length} FAILURE(S)`}\n`,
    );
    process.exitCode = failures.length === 0 ? 0 : 1;
    return;
  }

  const results = [];

  for (const wrapperBase of WRAPPERS) {
    const fixtureRoot = buildFixture(`hook-contracts-${wrapperBase.replace(/\.mjs$/, "")}-`);
    const rows = buildRowsForWrapper(wrapperBase, fixtureRoot);
    for (const row of rows) {
      const spawnResult = await runWrapper(wrapperBase, row.input, fixtureRoot);
      const verdict = row.expect(spawnResult);
      results.push({
        wrapper: wrapperBase,
        id: row.id,
        inputRaw: row.input,
        status: spawnResult.status,
        signal: spawnResult.signal,
        stdout: spawnResult.stdout,
        stderr: spawnResult.stderr,
        pass: verdict.pass,
        note: verdict.note,
      });
    }
  }

  if (baselineMode) {
    // cell codex-parity-1's contract, byte-for-byte unchanged: baseline mode
    // only ever characterizes the seven-wrapper fixture table.
    const failures = results.filter((r) => !r.pass);

    for (const r of results) {
      const mark = r.pass ? "ok  " : "FAIL";
      process.stdout.write(`${mark} - ${r.wrapper} :: ${r.id} :: ${r.note}\n`);
    }
    process.stdout.write(
      `\n${results.length} rows across ${WRAPPERS.length} wrappers, ${failures.length} failing target contract\n`,
    );

    writeBaselineReport(results, failures);
    process.stdout.write(
      `\nred-baseline.md written (${REPORT_PATH}) with ${failures.length} RED finding(s).\n`,
    );
    process.stdout.write("BASELINE CAPTURE COMPLETE\n");
    process.exitCode = 0;
    return;
  }

  // Default (non-baseline) mode is the cell's verify contract: the
  // seven-wrapper table must be GREEN, plus cell codex-parity-2's catalog
  // drift-check, allowed-differences, and isolated-CODEX_HOME
  // codex-acceptance rows, plus cell codex-parity-3's adapter contract rows
  // (registered-worker nickname matching and visible coverage-gap logging).
  results.push(...runCatalogDriftChecks());
  results.push(...runWorktreeAdapterRows());
  results.push(...runGeneratedRepoAuditRows());
  results.push(...runCodexAcceptanceRows());
  results.push(...await runCodexSubagentAuditRows());
  results.push(...await runNicknameRows());
  results.push(...await runLaneSessionRows());
  results.push(...await runGrantedWorktreeHeartbeatRows());
  results.push(...await runCaptureNudgeRows());
  results.push(...await runHoldSessionRows());
  results.push(...await runHandoffSessionRows());
  results.push(...await runIntentAnchorRows());
  results.push(...await runCoverageGapRows());
  results.push(...await runToolsLoggerCrashRows());
  // ...plus cell codex-parity-6b's installed-route rows: the default suite
  // must also prove the ACTIVE .codex/hooks.json commands work through the
  // real Codex route, so deleting the route group cannot hide behind
  // "--repo-route-only was green last week".
  const route = runRepoRouteRows({ configRef: null });
  results.push(...route.rows);
  // The required-row manifests bind in the default suite too, not only in the
  // dedicated modes: an absent or skipped required row fails EVERY path.
  results.push(...enforceRequiredRows(results, REQUIRED_CATALOG_ROW_IDS));
  results.push(...enforceRequiredRows(results, route.requiredIds, "repo-route"));

  const failures = results.filter((r) => !r.pass);
  const skipped = results.filter((r) => r.skip);

  for (const r of results) {
    const mark = r.skip ? "SKIP" : r.pass ? "ok  " : "FAIL";
    process.stdout.write(`${mark} - ${r.wrapper} :: ${r.id} :: ${r.note}\n`);
  }
  const groupCount = new Set(results.map((r) => r.wrapper)).size;
  process.stdout.write(
    `\n${results.length} rows across ${groupCount} check group(s) (${skipped.length} skipped), ${failures.length} failing target contract\n`,
  );

  process.stdout.write(`\n${failures.length === 0 ? "ALL PASS" : `${failures.length} FAILURE(S)`}\n`);
  process.exitCode = failures.length === 0 ? 0 : 1;
}

await main();
