#!/usr/bin/env node
// bump_version.mjs — set the bee release version in ONE command.
//
// The release version lives in four physical places (the tuple registry in
// scripts/lib/release-tuple.mjs). Before this script, a release hand-edited
// each one plus a pair of hardcoded test anchors — error-prone, and the tuple
// checker only caught the manifest members, not the anchors. Now:
//
//   node scripts/bump_version.mjs 1.3.6
//
// writes every tuple member from the single registry, regenerates the release
// hash manifest, and prints the remaining (already-automated) release steps.
// The split-brain regression anchors are no longer hand-edited — they derive
// the current version at runtime. Decision cba8b832.
//
// Usage:
//   node scripts/bump_version.mjs <version>          bump + regenerate manifest
//   node scripts/bump_version.mjs <version> --no-manifest   bump only
//   node scripts/bump_version.mjs --check            print current tuple, no writes
//
// Exit 0 on success; exit 1 on a bad argument, a desynced starting tuple, or a
// write/manifest failure.

import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  COMPONENTS,
  REPO_ROOT,
  readComponents,
  checkTupleEquality,
  writeComponentVersion,
} from "./lib/release-tuple.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// A release version is a 3- or 4-component dotted number (bee ships a 3-part
// semver today; the 4th slot leaves room for the historical build component).
const VERSION_RE = /^\d+\.\d+\.\d+(\.\d+)?$/;

function fail(message) {
  console.error(`bump_version: ${message}`);
  process.exit(1);
}

function printCurrent() {
  let entries;
  try {
    entries = readComponents();
  } catch (error) {
    fail(error.message);
  }
  const { ok } = checkTupleEquality(entries);
  console.log(`Current release tuple${ok ? "" : " (DESYNCED)"}:`);
  for (const entry of entries) {
    console.log(`  ${entry.version}  ${entry.name}`);
  }
  return { entries, ok };
}

// ─── argument parsing ──────────────────────────────────────────────────────
const args = process.argv.slice(2);

if (args.includes("--check")) {
  const { ok } = printCurrent();
  process.exit(ok ? 0 : 1);
}

const positional = args.filter((a) => !a.startsWith("--"));
const flags = args.filter((a) => a.startsWith("--"));
const unknownFlag = flags.find((f) => f !== "--no-manifest");
if (unknownFlag) {
  fail(`unknown flag "${unknownFlag}". Usage: bump_version.mjs <version> [--no-manifest] | --check`);
}
if (positional.length !== 1) {
  fail("exactly one version argument is required. Usage: bump_version.mjs <version> [--no-manifest] | --check");
}

const newVersion = positional[0];
if (!VERSION_RE.test(newVersion)) {
  fail(`"${newVersion}" is not a valid version (expected N.N.N or N.N.N.N).`);
}

// ─── read the current tuple; refuse to bump a desynced starting point ──────
const { entries: before, ok: startOk } = printCurrent();
if (!startOk) {
  fail("the release tuple is already desynced — resolve that before bumping (every component must start equal).");
}
const currentVersion = before[0].version;

if (currentVersion === newVersion) {
  console.log(`\nAlready at ${newVersion} — nothing to write.`);
} else {
  console.log(`\nBumping ${currentVersion} -> ${newVersion} across ${COMPONENTS.length} components:`);
  for (const component of COMPONENTS) {
    try {
      const previous = writeComponentVersion(component, newVersion);
      console.log(`  ok  ${previous} -> ${newVersion}  ${component.name}`);
    } catch (error) {
      fail(`writing ${component.name} failed: ${error.message} (some components may already be written — re-run once fixed).`);
    }
  }

  // Verify the write landed and is internally consistent.
  const after = readComponents();
  const recheck = checkTupleEquality(after);
  if (!recheck.ok || after[0].version !== newVersion) {
    fail("post-write verification failed — the tuple did not settle on the new version.");
  }
  console.log(`\nAll ${after.length} components now agree on ${newVersion}.`);
}

// ─── regenerate the hash manifest (covers the state.mjs + plugin.json set) ──
if (!flags.includes("--no-manifest")) {
  console.log("\nRegenerating release manifest (scripts/release_manifest.mjs --write) ...");
  try {
    execFileSync("node", [path.join(__dirname, "release_manifest.mjs"), "--write"], {
      cwd: REPO_ROOT,
      stdio: "inherit",
    });
  } catch (error) {
    fail(`release manifest regeneration failed: ${error.message}`);
  }
}

// ─── remaining release steps (already automated, listed for the operator) ──
console.log(`
Version set to ${newVersion}. Remaining release steps:
  1. Propagate projections + onboarding ledger:
       node packages/bee/scripts/onboard_bee.mjs --repo-root . --apply
     (refreshes .claude/ and .agents/ skill copies and .bee/onboarding.json;
      the split-brain test anchors self-update — no hand edit.)
  2. Run the impacted verify run (must be green):
       node scripts/run_verify.mjs --impacted-from-git
  3. Commit, then tag: git tag v${newVersion} && git push --tags
  4. Dispatch the CI full run: gh workflow run CI --ref main
     (the full suite is CI-owned; a red run files its own verify-red issue.)
`);
