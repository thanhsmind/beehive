#!/usr/bin/env node
// test_split_brain_regression.mjs - FROZEN acceptance-regression fixture for
// the source/distribution split-brain (SPEC.md §2 E-02/E-03, §6.4 VER-01..06,
// decision D-04; docs/history/codex-harness-hardening/SPEC.md).
//
// ORIGINAL scenario (pre packages-engine-move, historical record): a Codex
// task loaded its project skill projection from .agents/skills/bee-hive; that
// launcher was STALE while the repo's vendored .bee/bin runtime was CURRENT,
// and onboarding's ledger reported drift:false (E-02). Running
//   node .agents/skills/bee-hive/scripts/onboard_bee.mjs --repo-root <repo> --json
// (E-03) was a *self-onboard* invocation (the launcher lived INSIDE the
// projection it targeted) that could downgrade the vendored runtime.
//
// DISSOLVED by packages-engine-move D3 (docs/history/packages-engine-move/
// plan.md, validation-slice1.md C4): the onboarding/distribution engine
// (onboard_bee.mjs, plugin_distribution.mjs) no longer lives inside the
// skills tree it renders. It moved to packages/bee/scripts/ - a host
// projection (.claude/skills/bee-hive, .agents/skills/bee-hive) is now a
// compliance mirror that syncs SKILL.md + instruction content only, and
// structurally can never carry a scripts/ subdirectory of its own, because
// skill-sync copies the skills/bee-hive tree and the engine no longer lives
// under it. A launcher "living inside a projection" - the precondition the
// original E-03 defect depended on - can no longer occur via the real sync
// path. This file is DISSOLVED-AND-REPLACED, not deleted: it now guards the
// TWO invariants that make the original scenario structurally impossible,
// so a future change that re-introduces either one re-opens the class of bug
// this fixture used to catch by a different route.
//
// TARGET (post-move) behavior, frozen acceptance fixture:
//   (A) a projection tree synced from the real, current skills/bee-hive
//       carries NO scripts/ subdirectory - i.e., no launcher - full stop.
//   (B) if an engine copy is ever invoked from a projection-shaped root
//       anyway (a stray/legacy relic, a manual mis-copy - not something the
//       real sync path produces, but not something the identity anchor
//       should trust either), it fails closed: onboard_bee.mjs's own source
//       identity resolution (classifySource -> project_projection, then
//       readSourceReleaseIdentity's payload-version check) reports
//       status "blocked_no_source", never a usable plan.
//
// STATUS: GREEN since cell packages-engine-move-1 (2026-07-26), continuing
// the GREEN lineage established by cell codex-harness-hardening-1b-1
// (2026-07-15) for the original scenario. This fixture now GUARDS AGAINST
// REGRESSION of (A)/(B) and is part of commands.verify. If either invariant
// ever breaks, the split-brain-shaped hole has reopened by a new route.
//
// Self-contained, single file. Does NOT import test_onboard_bee.mjs (it
// exports nothing) - copyTree is re-implemented inline here, same as before.
//
// PROHIBITED in this file: any fix to onboard_bee.mjs / source-identity.mjs
// logic (a fix, if ever needed, lives there, never here). source-identity.mjs
// itself is a prohibited edit for this feature (packages-engine-move C3) -
// this fixture only relocates a verbatim copy of it alongside a relocated
// engine copy, exactly like a real deploy would.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { runModuleWorker } from "../../../scripts/lib/run-module-worker.mjs";

const FIXTURE_BUG_CODE = 2;
const INVARIANT_BROKEN_CODE = 1;

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const SCRIPTS_DIR = path.dirname(SCRIPT_PATH); // packages/bee/scripts (this file's own dir, post-move)
const REPO_ROOT = path.join(SCRIPTS_DIR, "..", "..", ".."); // matches test_onboard_bee.mjs's REPO_ROOT calc
// packages-engine-move D1: skills/bee-hive is instruction-only post-move -
// carries no scripts/ of its own anymore. This IS the fact invariant (A)
// checks; REAL_HIVE_DIR is deliberately read from the live tree, never
// hand-asserted, so a future accidental re-add of scripts/ under it is
// caught here regardless of how it happened.
const REAL_HIVE_DIR = path.join(REPO_ROOT, "skills", "bee-hive");
const REAL_ENGINE_LAUNCHER = path.join(SCRIPTS_DIR, "onboard_bee.mjs"); // this checkout's own real, current launcher
const REAL_PACKAGES_BEE_LIB_DIR = path.join(REPO_ROOT, "packages", "bee", "lib");

class FixtureBugError extends Error {}

function fixtureBug(message) {
  throw new FixtureBugError(message);
}

// ---- inline recursive tree copy - readdirSync recursion, NO hand-kept file
// list (F4/TEST-03: source authority is the actual tree on disk). ----
function copyTree(srcDir, dstDir) {
  fs.mkdirSync(dstDir, { recursive: true });
  for (const entry of fs.readdirSync(srcDir, { withFileTypes: true })) {
    const s = path.join(srcDir, entry.name);
    const d = path.join(dstDir, entry.name);
    if (entry.isSymbolicLink()) {
      fs.symlinkSync(fs.readlinkSync(s), d);
    } else if (entry.isDirectory()) {
      copyTree(s, d);
    } else {
      fs.copyFileSync(s, d);
    }
  }
}

function parseJsonOrNull(text) {
  try {
    return JSON.parse(text || "");
  } catch {
    return null;
  }
}

// Spawn onboard_bee.mjs from the given launcher and FIRST confirm it
// actually ran: a spawn error or unparseable/statusless stdout is a fixture
// bug, never an invariant result either way.
async function runOnboard(launcher, repoRoot, fakeHome, extraArgs = []) {
  const env = { ...process.env, HOME: fakeHome, USERPROFILE: fakeHome };
  const result = await runModuleWorker(launcher, {
    args: ["--repo-root", repoRoot, "--json", ...extraArgs],
    env,
    fakeHome,
  });
  if (result.error) {
    fixtureBug(
      `onboard_bee.mjs (${extraArgs.join(" ") || "plan"}) failed to spawn: ${result.error.message}`,
    );
  }
  const payload = parseJsonOrNull(result.stdout);
  if (!payload || typeof payload.status !== "string") {
    fixtureBug(
      `onboard_bee.mjs (${extraArgs.join(" ") || "plan"}) did not run to a parseable status ` +
        `(exit=${result.status}, stdout=${JSON.stringify(result.stdout)}, stderr=${JSON.stringify(result.stderr)})`,
    );
  }
  return payload;
}

let exitCode = 0;
const outputLines = [];
const tempDirs = [];

function mkTemp(prefix) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  tempDirs.push(dir);
  return dir;
}

try {
  // ---------------------------------------------------------------------
  // (A) A synced projection carries no launcher.
  // ---------------------------------------------------------------------
  const projA = mkTemp("bee-split-brain-projA-");
  const agentsHive = path.join(projA, ".agents", "skills", "bee-hive");
  copyTree(REAL_HIVE_DIR, agentsHive);
  const carriesNoScriptsDir = !fs.existsSync(path.join(agentsHive, "scripts"));
  const carriesNoLauncher = !fs.existsSync(path.join(agentsHive, "scripts", "onboard_bee.mjs"));
  const invariantA = carriesNoScriptsDir && carriesNoLauncher;
  outputLines.push(
    `${invariantA ? "ok" : "FAIL"}  - (A) projection tree carries no launcher ` +
      `(scripts_dir_absent=${carriesNoScriptsDir} launcher_absent=${carriesNoLauncher})`,
  );

  // ---------------------------------------------------------------------
  // (B) Invoking the engine from a projection-shaped root yields
  //     blocked_no_source. Not something the real sync path can produce
  //     anymore (per (A)) - this simulates a stray/legacy engine copy to
  //     prove the IDENTITY ANCHOR, not the sync path, is what refuses it.
  // ---------------------------------------------------------------------
  const projB = mkTemp("bee-split-brain-projB-");
  // legacy layout fixture: the engine copy below is deliberately placed
  // inside a projection-shaped root (a shape the real sync path can no
  // longer produce, per invariant (A) above) to prove the identity anchor
  // refuses it on its own.
  // Shape: <projB>/.agents is what this copied engine's own PLUGIN_ROOT
  // arithmetic resolves to (ENGINE_DIR's grandparent basename == ".agents"),
  // which source-identity.mjs's classifySource (untouched - packages-
  // engine-move C3) reads as project_projection purely from the path
  // string, independent of what exists on disk.
  const engineDir = path.join(projB, ".agents", "packages", "bee");
  const engineScripts = path.join(engineDir, "scripts");
  const engineLib = path.join(engineDir, "lib");
  fs.mkdirSync(engineScripts, { recursive: true });
  fs.mkdirSync(engineLib, { recursive: true });
  // The engine's immediate static-import deps are copied verbatim so the
  // module actually loads - these are pure utility modules, relocated
  // exactly like a real deploy would, never modified.
  for (const libName of ["commands_detect.mjs", "fsutil.mjs", "source-identity.mjs"]) {
    fs.copyFileSync(path.join(REAL_PACKAGES_BEE_LIB_DIR, libName), path.join(engineLib, libName));
  }
  // Deliberately NO resolvable BEE_VERSION under engineLib/state.mjs: a
  // genuine projection never carries a packages/bee payload at all (only the
  // skills tree syncs), so the version marker this engine's own release
  // identity reads is absent - this IS the "projection-only root" (skills
  // shape present via the classifySource path string, no real payload). A
  // bare COMMAND_KEYS-only stub still satisfies commands_detect.mjs's static
  // `import { COMMAND_KEYS } from './state.mjs'` (msn-18d) so the module
  // graph loads far enough to actually report blocked_no_source, rather than
  // crashing on a missing file before onboard_bee.mjs's own logic runs.
  fs.writeFileSync(
    path.join(engineLib, "state.mjs"),
    "export const COMMAND_KEYS = ['setup', 'start', 'test', 'verify'];\n",
    "utf8",
  );
  fs.writeFileSync(path.join(engineScripts, "onboard_bee.mjs"), fs.readFileSync(REAL_ENGINE_LAUNCHER, "utf8"), "utf8");
  // Mirror invariant (A) inside this same fixture too: even the stray
  // skills/bee-hive sibling a naive relic-copy might carry has no scripts/
  // of its own (copied from the real, current tree).
  copyTree(REAL_HIVE_DIR, path.join(projB, ".agents", "skills", "bee-hive"));

  const targetRepo = mkTemp("bee-split-brain-target-");
  const fakeHome = mkTemp("bee-split-brain-home-");
  const launcher = path.join(engineScripts, "onboard_bee.mjs");
  const planPayload = await runOnboard(launcher, targetRepo, fakeHome, []);
  const invariantB = planPayload.status === "blocked_no_source";
  outputLines.push(
    `${invariantB ? "ok" : "FAIL"}  - (B) engine invoked from a projection-shaped root reports ` +
      `blocked_no_source (observed status=${planPayload.status})`,
  );

  if (invariantA && invariantB) {
    outputLines.push("split-brain regression fixture: TARGET met - both invariants hold.");
    exitCode = 0;
  } else {
    outputLines.push(
      "FREEZE-RED: split-brain-shaped defect present - a projection tree can carry a launcher, " +
        "or an engine copy invoked from a projection-shaped root resolves as an authoritative source.",
    );
    exitCode = INVARIANT_BROKEN_CODE;
  }
} catch (err) {
  if (err instanceof FixtureBugError) {
    outputLines.push(`fixture bug: ${err.message}`);
  } else {
    outputLines.push(`fixture bug: unexpected exception: ${(err && err.stack) || err}`);
  }
  exitCode = FIXTURE_BUG_CODE;
} finally {
  for (const dir of tempDirs) {
    try {
      fs.rmSync(dir, { recursive: true, force: true });
    } catch {
      // best-effort cleanup
    }
  }
}

for (const line of outputLines) {
  process.stdout.write(`${line}\n`);
}
process.exit(exitCode);
