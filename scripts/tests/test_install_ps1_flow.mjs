#!/usr/bin/env node
// Static conformance test for cell hardening-6: install.ps1 must stage its
// plugin work in the same probe -> confirm -> mutate -> rollback order as
// install.sh's proven D8 machinery (scripts/install.sh :270-292 read-only
// probe, :430 confirm, :433 transition, :339-374 rollback). There is no
// Windows runner in this repo/CI, so this test never executes install.ps1 —
// it PARSES the script text and asserts structural facts about it:
//
//   1. every literal plugin-mutation invocation (marketplace add / plugin
//      add / install / remove / uninstall) that lives OUTSIDE a function
//      DEFINITION (defining a function does not execute it) appears strictly
//      AFTER the "Apply this onboarding plan" confirmation line — i.e.
//      nothing mutates a runtime plugin before the user has confirmed;
//   2. a rollback-shaped function exists and is actually CALLED from a
//      failure-handling path (not merely defined and orphaned);
//   3. the old "no plugin transition/rollback machinery" admission comment
//      is gone — it described exactly the bug this cell fixes.
//
// Optional live leg: if `pwsh` is on PATH, run install.ps1 -DryRun against a
// throwaway directory and assert it exits 0 having written nothing (canary
// style — skips VISIBLY, not silently, when pwsh is absent, mirroring
// scripts/canary_codex.mjs's `codexOnPath()` skip pattern).
//
// Usage: node scripts/tests/test_install_ps1_flow.mjs

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..", "..");
const INSTALL_PS1 = path.join(REPO_ROOT, "scripts", "install.ps1");

let failures = 0;
function check(label, cond, detail) {
  if (cond) {
    console.log(`PASS test_install_ps1_flow: ${label}`);
  } else {
    failures += 1;
    console.error(`FAIL test_install_ps1_flow: ${label}${detail ? ` :: ${detail}` : ""}`);
  }
}

const source = fs.readFileSync(INSTALL_PS1, "utf8");
const lines = source.split("\n");

// ─── locate function-DEFINITION spans (excluded from execution-order checks) ───
// PowerShell function bodies in this file never contain a brace inside a
// string or comment (verified: no line matches /"[^"]*[{}][^"]*"/ and every
// "#" in the file starts a full comment line), so a naive brace counter over
// non-comment lines is a safe, exact matcher for this specific file.
function isCommentLine(line) {
  return /^\s*#/.test(line);
}

const functionDefLineNumbers = new Set(); // 1-indexed lines inside a function DEFINITION body
const functionStarts = []; // { name, startLine }

for (let i = 0; i < lines.length; i += 1) {
  const m = lines[i].match(/^\s*function\s+([A-Za-z0-9_-]+)/);
  if (!m) continue;
  const name = m[1];
  const startLine = i + 1; // 1-indexed
  // Find the opening brace at/after the function keyword line, then match to
  // its closing brace via depth counting over non-comment lines.
  let depth = 0;
  let opened = false;
  let endLine = null;
  for (let j = i; j < lines.length; j += 1) {
    const lineNo = j + 1;
    if (isCommentLine(lines[j])) continue;
    for (const ch of lines[j]) {
      if (ch === "{") { depth += 1; opened = true; }
      else if (ch === "}") { depth -= 1; }
    }
    if (opened && depth === 0) { endLine = lineNo; break; }
  }
  if (endLine === null) endLine = lines.length; // defensive fallback, should not trigger
  for (let l = startLine; l <= endLine; l += 1) functionDefLineNumbers.add(l);
  functionStarts.push({ name, startLine, endLine });
}

check(
  "at least one function definition was found (sanity check on the brace matcher)",
  functionStarts.length >= 5,
  `found ${functionStarts.length}`,
);

// ─── (1) confirmation strictly before any top-level plugin mutation ───

const CONFIRM_PATTERN = /Confirm-Step\s*\(?["']Apply this onboarding plan/;
let confirmLine = null;
for (let i = 0; i < lines.length; i += 1) {
  if (CONFIRM_PATTERN.test(lines[i]) && !functionDefLineNumbers.has(i + 1)) {
    confirmLine = i + 1;
    break;
  }
}
check("the 'Apply this onboarding plan' confirmation call exists at top level (outside any function definition)", confirmLine !== null);

// Literal mutating plugin verbs. Deliberately narrow (not just "plugin "):
// `plugin list` is the read-only probe and must NOT trip this.
const MUTATION_PATTERN = /\bplugin\s+(marketplace\s+add|add\s+['"]bee@bee['"]|install\s+['"]bee@bee['"]|remove\s+['"]bee@bee['"]|uninstall\s+['"]bee@bee['"])/;

const topLevelMutationLines = [];
for (let i = 0; i < lines.length; i += 1) {
  if (functionDefLineNumbers.has(i + 1)) continue; // inside a function definition: not top-level execution
  if (MUTATION_PATTERN.test(lines[i])) topLevelMutationLines.push(i + 1);
}

if (confirmLine !== null) {
  const violations = topLevelMutationLines.filter((ln) => ln < confirmLine);
  check(
    "no top-level plugin-mutation invocation appears before the confirmation gate",
    violations.length === 0,
    violations.length ? `mutation line(s) before confirm (line ${confirmLine}): ${violations.join(", ")}` : "",
  );
} else {
  check("no top-level plugin-mutation invocation appears before the confirmation gate", false, "confirmation call not found");
}

// The call SITE that triggers the (function-wrapped) mutation must itself be
// top-level and must run strictly after confirmation — proves the staging is
// wired, not just that the mutating verbs happen to live inside a function.
const TRANSITION_CALL_PATTERN = /\bInvoke-PluginTransition\b/;
let transitionCallLine = null;
for (let i = 0; i < lines.length; i += 1) {
  if (functionDefLineNumbers.has(i + 1)) continue;
  if (TRANSITION_CALL_PATTERN.test(lines[i]) && !/^\s*function\s/.test(lines[i])) {
    transitionCallLine = i + 1;
    break;
  }
}
check("a top-level call site invokes the plugin transition after confirmation", confirmLine !== null && transitionCallLine !== null && transitionCallLine > confirmLine, `confirmLine=${confirmLine}, transitionCallLine=${transitionCallLine}`);

// ─── (2) a rollback function exists and is referenced from a failure path ───

const rollbackFn = functionStarts.find((f) => /rollback/i.test(f.name));
check("a rollback-shaped function is defined", Boolean(rollbackFn), `functions found: ${functionStarts.map((f) => f.name).join(", ")}`);

let rollbackReferencedFromFailurePath = false;
if (rollbackFn) {
  // A "failure path" function: one whose name suggests failure handling AND
  // whose body both calls the rollback function and terminates the script
  // (an `exit` statement) — i.e. it is the thing invoked when something
  // upstream goes wrong, not an unrelated caller.
  const failureFn = functionStarts.find((f) => /failure/i.test(f.name));
  if (failureFn) {
    const bodyLines = lines.slice(failureFn.startLine - 1, failureFn.endLine).join("\n");
    const callsRollback = new RegExp(`\\b${rollbackFn.name}\\b`).test(bodyLines) && !new RegExp(`function\\s+${rollbackFn.name}\\b`).test(bodyLines);
    const exits = /\bexit\s+\d/.test(bodyLines);
    rollbackReferencedFromFailurePath = callsRollback && exits;
  }
}
check("the rollback function is called from a failure-handling function that exits nonzero", rollbackReferencedFromFailurePath);

// The failure-path function itself must actually be wired to the real
// failure sites (transition / preflight / apply / cleanup refusals) — not
// merely defined. Assert it is called at least twice at top level (there are
// at least 3 known post-confirm failure sites: transition, preflight/apply).
if (rollbackFn) {
  const failureFn = functionStarts.find((f) => /failure/i.test(f.name));
  if (failureFn) {
    let callSites = 0;
    for (let i = 0; i < lines.length; i += 1) {
      if (functionDefLineNumbers.has(i + 1)) continue;
      if (new RegExp(`\\b${failureFn.name}\\b`).test(lines[i])) callSites += 1;
    }
    check(`the failure-handling function (${failureFn.name}) is invoked from multiple top-level failure sites`, callSites >= 2, `found ${callSites} call site(s)`);
  }
}

// ─── (3) the old "no rollback machinery" admission comment is gone ───

check(
  "the stale 'no plugin transition/rollback machinery' comment has been removed",
  !/no plugin transition\/rollback machinery/i.test(source) && !/this installer has no plugin transition/i.test(source),
);

// ─── optional live leg: -DryRun smoke test, only when pwsh is on PATH ───
// Canary-style: skip VISIBLY (not silently) when pwsh is absent, exit 0
// either way so a missing Windows runner never fails verify.

function pwshOnPath() {
  const probe = spawnSync("pwsh", ["-NoLogo", "-NonInteractive", "-Command", "$PSVersionTable.PSVersion.ToString()"], {
    encoding: "utf8",
  });
  return probe.status === 0;
}

if (pwshOnPath()) {
  const target = fs.mkdtempSync(path.join(os.tmpdir(), "install-ps1-dryrun-"));
  try {
    const result = spawnSync(
      "pwsh",
      ["-NoLogo", "-NonInteractive", "-File", INSTALL_PS1, "-Directory", target, "-Source", REPO_ROOT, "-DryRun", "-Yes"],
      { encoding: "utf8" },
    );
    check("pwsh -DryRun smoke leg exits 0", result.status === 0, `stdout: ${(result.stdout || "").slice(-500)} stderr: ${(result.stderr || "").slice(-500)}`);
    const wroteSomething = fs.readdirSync(target).length > 0;
    check("pwsh -DryRun smoke leg writes nothing into the target directory", !wroteSomething, `target now contains: ${fs.readdirSync(target).join(", ")}`);
  } finally {
    fs.rmSync(target, { recursive: true, force: true });
  }
} else {
  console.log("test_install_ps1_flow: pwsh smoke leg skipped (no pwsh on PATH)");
}

if (failures > 0) {
  console.error(`FAIL test_install_ps1_flow: ${failures} check(s) failed`);
  process.exit(1);
}
console.log("PASS test_install_ps1_flow: all checks passed");
