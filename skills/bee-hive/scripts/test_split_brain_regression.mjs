#!/usr/bin/env node
// test_split_brain_regression.mjs - FROZEN acceptance-regression fixture for
// the source/distribution split-brain (SPEC.md §2 E-02/E-03, §6.4 VER-01..06,
// decision D-04; docs/history/codex-harness-hardening/SPEC.md).
//
// Reproduces the exact real-world defect: a Codex task loads its project
// skill projection from .agents/skills/bee-hive; that launcher is STALE
// (0.1.43) while the repo's vendored .bee/bin runtime is CURRENT (0.1.44)
// and the onboarding ledger reports drift:false (E-02). Running
//   node .agents/skills/bee-hive/scripts/onboard_bee.mjs --repo-root <repo> --json
// (E-03) is a *self-onboard* invocation (the launcher lives inside the repo
// it targets), so computeSkillSync's version preflight self-skips the
// .claude/skills and .agents/skills targets entirely - that preflight is the
// ONLY place onboard_bee.mjs currently compares source version against host
// version. Step 3 of computePlan ("vendored helpers + lib") is a raw
// byte-diff against the running launcher's OWN vendored packages/bee with NO version
// gate whatsoever, so a stale launcher plans (and, on --apply, actually
// performs) a downgrade of .bee/bin/lib/state.mjs from 0.1.44 back to 0.1.43.
//
// TARGET (post-fix) behavior (VER-06, frozen acceptance fixture): onboard
// run from the stale launcher reports status blocked_downgrade, and a
// subsequent --apply performs ZERO mutation anywhere in the repo
// (hashTree(repo) identical before/after).
//
// STATUS: GREEN since cell codex-harness-hardening-1b-1 (2026-07-15). The fix
// hoists a target-independent runtime-lib downgrade guard into computeSkillSync
// (hostLibDowngradeBlock -> result.blocked) so it fires even when every skill
// target self_skips; the existing whole-apply abort then refuses with zero
// mutation. This fixture now GUARDS AGAINST REGRESSION and is part of
// commands.verify. If it ever exits 3 again, the split-brain downgrade hole
// has reopened.
//
// Pre-fix defect (for the record): plan reported changes_needed (via the
// ungated copy_lib item for .bee/bin/lib/state.mjs) and --apply actually
// downgraded that file's BEE_VERSION back to the stale launcher's version. On
// that confirmed-defect path the fixture prints exactly:
//   FREEZE-RED: split-brain defect present (OBSERVED=... EXPECTED=blocked_downgrade+zero-mutation)
// and exits with SENTINEL code 3 - distinct from node's uncaught-throw exit
// 1, so a fixture crash (spawn failure, unparseable JSON, unexpected
// exception) is never mistaken for the confirmed defect: those paths exit 2
// with a "fixture bug:" message instead. Exits 0 ONLY when the target is met.
//
// Self-contained, single file. Does NOT import test_onboard_bee.mjs (it
// exports nothing) - hashTree() and the recursive tree copy are
// re-implemented inline here, and the source tree carried into the fixture
// is discovered via readdirSync recursion, never a hand-kept file list.
//
// PROHIBITED in this file: any fix to onboard_bee.mjs / mirror / detector
// logic (the fix lives in onboard_bee.mjs, never here). Joining
// commands.verify was gated on this fixture being GREEN - now satisfied.

import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { runModuleWorker } from "../../../scripts/lib/run-module-worker.mjs";

const SENTINEL_DEFECT = 3;
const FIXTURE_BUG_CODE = 2;

const SCRIPT_PATH = fileURLToPath(import.meta.url);
const SCRIPTS_DIR = path.dirname(SCRIPT_PATH);
const REAL_HIVE_DIR = path.dirname(SCRIPTS_DIR); // .../skills/bee-hive (this repo's real, current tree)
const REPO_ROOT = path.join(SCRIPTS_DIR, "..", "..", ".."); // matches test_onboard_bee.mjs's REPO_ROOT calc
const REAL_BEE_BIN_DIR = path.join(REPO_ROOT, ".bee", "bin");
// D3 (packages-restructure): onboard_bee.mjs resolves its vendored payload
// PLUGIN_ROOT-relative, not self-relative to its own scripts dir anymore. A
// launcher copied to <root>/.agents/skills/bee-hive/scripts/onboard_bee.mjs
// computes PLUGIN_ROOT as 3-up from its own dir, i.e. <root>/.agents (NOT
// fixtureRepo itself) - so each projection's packages/bee copy must live at
// that SAME nesting depth for the copied launcher to resolve its own imports
// and its own source version marker.
const REAL_PACKAGES_BEE_DIR = path.join(REPO_ROOT, "packages", "bee");

class FixtureBugError extends Error {}

function fixtureBug(message) {
  throw new FixtureBugError(message);
}

// ---- inline hashTree - re-implemented per must_haves (test_onboard_bee.mjs
// exports nothing). Stable full-tree digest, lstat semantics: symlinks
// recorded by target, never followed. ----
function hashTree(dir) {
  if (!fs.existsSync(dir)) {
    return "ABSENT";
  }
  const lines = [];
  const walk = (d, prefix) => {
    const entries = fs
      .readdirSync(d, { withFileTypes: true })
      .sort((a, b) => a.name.localeCompare(b.name));
    for (const e of entries) {
      const rel = prefix ? `${prefix}/${e.name}` : e.name;
      const abs = path.join(d, e.name);
      if (e.isSymbolicLink()) {
        lines.push(`link ${rel} -> ${fs.readlinkSync(abs)}`);
      } else if (e.isDirectory()) {
        lines.push(`dir ${rel}`);
        walk(abs, rel);
      } else {
        lines.push(
          `file ${rel} ${crypto.createHash("sha256").update(fs.readFileSync(abs)).digest("hex")}`,
        );
      }
    }
  };
  walk(dir, "");
  return lines.join("\n");
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

const VERSION_LINE_RE = /^export const BEE_VERSION = ['"][^'"]*['"];?[ \t]*\r?$/m;

function patchVersion(stateFile, version) {
  const text = fs.readFileSync(stateFile, "utf8");
  if (!VERSION_LINE_RE.test(text)) {
    fixtureBug(`${stateFile} has no single-line BEE_VERSION declaration to patch`);
  }
  fs.writeFileSync(
    stateFile,
    text.replace(VERSION_LINE_RE, `export const BEE_VERSION = '${version}';`),
    "utf8",
  );
}

function readVersionLoose(stateFile) {
  const text = fs.readFileSync(stateFile, "utf8");
  const m = text.match(/export const BEE_VERSION = ['"]([^'"]+)['"]/);
  return m ? m[1] : null;
}

function parseJsonOrNull(text) {
  try {
    return JSON.parse(text || "");
  } catch {
    return null;
  }
}

// Spawn the real onboard from the given launcher and FIRST confirm it
// actually ran: a spawn error or unparseable/statusless stdout is a fixture
// bug, never the confirmed defect.
async function runOnboard(launcher, fixtureRepo, fakeHome, extraArgs) {
  const env = { ...process.env, HOME: fakeHome, USERPROFILE: fakeHome };
  const result = await runModuleWorker(launcher, {
    args: ["--repo-root", fixtureRepo, "--json", ...extraArgs],
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
let outputLine = null;
let fixtureRepo = null;
let fakeHome = null;

try {
  // ---------------------------------------------------------------------
  // 1. Build the fixture repo.
  // ---------------------------------------------------------------------
  fixtureRepo = fs.mkdtempSync(path.join(os.tmpdir(), "bee-split-brain-"));
  fakeHome = fs.mkdtempSync(path.join(os.tmpdir(), "bee-split-brain-home-"));

  // Codex projection AND launcher source (same tree - exactly the real E-03
  // scenario, where the launcher IS the .agents/skills/bee-hive projection):
  // a full recursive copy of the real, current skills/bee-hive, patched to
  // the STALE 0.1.43.
  const agentsHive = path.join(fixtureRepo, ".agents", "skills", "bee-hive");
  copyTree(REAL_HIVE_DIR, agentsHive);
  const agentsPackagesBee = path.join(fixtureRepo, ".agents", "packages", "bee");
  copyTree(REAL_PACKAGES_BEE_DIR, agentsPackagesBee);
  patchVersion(path.join(agentsPackagesBee, "lib", "state.mjs"), "0.1.43");

  // Claude projection: same stale content (independent copy, same patch).
  const claudeHive = path.join(fixtureRepo, ".claude", "skills", "bee-hive");
  copyTree(REAL_HIVE_DIR, claudeHive);
  const claudePackagesBee = path.join(fixtureRepo, ".claude", "packages", "bee");
  copyTree(REAL_PACKAGES_BEE_DIR, claudePackagesBee);
  patchVersion(path.join(claudePackagesBee, "lib", "state.mjs"), "0.1.43");

  // The current release version is derived from the real source of truth, not
  // hardcoded here — a release bump no longer has to hand-edit this fixture
  // (decision cba8b832). The value must be equal across the whole tuple; the
  // canonical source is packages/bee/lib/state.mjs (test_release_tuple guards
  // the rest).
  const expectedCurrentVersion = readVersionLoose(
    path.join(REAL_PACKAGES_BEE_DIR, "lib", "state.mjs"),
  );
  if (!expectedCurrentVersion) {
    fixtureBug(
      "could not read the current BEE_VERSION from packages/bee/lib/state.mjs - " +
        "cannot establish the fixture's expected runtime version",
    );
  }

  // Vendored runtime: real CURRENT .bee/bin (expectedCurrentVersion), unpatched.
  const repoBeeBinDir = path.join(fixtureRepo, ".bee", "bin");
  copyTree(REAL_BEE_BIN_DIR, repoBeeBinDir);
  const repoLibStateFile = path.join(repoBeeBinDir, "lib", "state.mjs");
  const preVersion = readVersionLoose(repoLibStateFile);
  if (preVersion !== expectedCurrentVersion) {
    fixtureBug(
      `this checkout's real .bee/bin/lib/state.mjs (${preVersion}) does not match the ` +
        `canonical packages/bee/lib/state.mjs (${expectedCurrentVersion}) - the release tuple is ` +
        "desynced, this is not the defect under test (run test_release_tuple / bump_version)",
    );
  }

  // Onboarding ledger: believes the runtime is current, drift:false (E-02).
  // No managed.repo_hooks / managed.statusline entries - keeps this run out
  // of the --repo-hooks / statusline code paths, which are irrelevant to the
  // split-brain defect under test.
  const onboardingPath = path.join(fixtureRepo, ".bee", "onboarding.json");
  fs.mkdirSync(path.dirname(onboardingPath), { recursive: true });
  fs.writeFileSync(
    onboardingPath,
    `${JSON.stringify(
      {
        schema_version: "1.0",
        bee_version: expectedCurrentVersion,
        managed: { agents_block: "fixture", gitignore_block: "fixture", helpers: {}, lib: {} },
        created_at: "2026-01-01T00:00:00.000Z",
        updated_at: "2026-01-01T00:00:00.000Z",
      },
      null,
      2,
    )}\n`,
    "utf8",
  );

  const launcher = path.join(agentsHive, "scripts", "onboard_bee.mjs");

  // ---------------------------------------------------------------------
  // 2. Spawn the real onboard from the stale launcher, plan mode. Confirms
  //    it actually ran (inside runOnboard) BEFORE any defect assertion.
  // ---------------------------------------------------------------------
  const planPayload = await runOnboard(launcher, fixtureRepo, fakeHome, []);
  const planStatus = planPayload.status;

  // ---------------------------------------------------------------------
  // 3. Zero-mutation check: hash the whole repo before and after --apply.
  // ---------------------------------------------------------------------
  const hashBefore = hashTree(fixtureRepo);
  const applyPayload = await runOnboard(launcher, fixtureRepo, fakeHome, ["--apply"]);
  const hashAfter = hashTree(fixtureRepo);
  const zeroMutation = hashBefore === hashAfter;
  const postVersion = readVersionLoose(repoLibStateFile);

  const targetMet =
    planStatus === "blocked_downgrade" && applyPayload.status === "blocked_downgrade" && zeroMutation;

  if (targetMet) {
    outputLine =
      "split-brain regression fixture: TARGET met - " +
      `plan=${planStatus} apply=${applyPayload.status} zero_mutation=${zeroMutation}`;
    exitCode = 0;
  } else {
    const observed =
      `plan_status=${planStatus} apply_status=${applyPayload.status} ` +
      `zero_mutation=${zeroMutation} runtime_lib_state_version_before=${preVersion} after=${postVersion}`;
    outputLine = `FREEZE-RED: split-brain defect present (OBSERVED=${observed} EXPECTED=blocked_downgrade+zero-mutation)`;
    exitCode = SENTINEL_DEFECT;
  }
} catch (err) {
  if (err instanceof FixtureBugError) {
    outputLine = `fixture bug: ${err.message}`;
  } else {
    outputLine = `fixture bug: unexpected exception: ${(err && err.stack) || err}`;
  }
  exitCode = FIXTURE_BUG_CODE;
} finally {
  for (const dir of [fixtureRepo, fakeHome]) {
    if (dir) {
      try {
        fs.rmSync(dir, { recursive: true, force: true });
      } catch {
        // best-effort cleanup
      }
    }
  }
}

process.stdout.write(`${outputLine}\n`);
process.exit(exitCode);
