#!/usr/bin/env node
// Ledger parity checker (lpsp-1) — the THIRD copy relationship this repo
// carries, and until now the only unguarded one:
//
//   packages/bee/** <-> .bee/bin/**                        guarded by scripts/tests/test_lib_mirror.mjs
//   the plugin trees                                       guarded by scripts/release_manifest.mjs --check
//   .bee/bin/** <-> .bee/onboarding.json managed hashes    guarded by NOTHING  <- this file
//
// USER-REPORTED, and it shipped to main: a clean checkout of main reported
// `drift: true` for .bee/bin/lib/lock.mjs at version 1.9.0 while the files
// were byte-identical to their source — only the TRACKED .bee/onboarding.json
// still held the pre-fix hash, because a release ran self-onboard and THEN
// the tag was re-pointed to a later fix commit that changed .bee/bin/**
// without re-onboarding (hand-fixed in 6412017). A stale ledger shipped
// silently and then lied to every new session about its own install.
//
// This recomputes, for every file the ledger's managed.lib / managed.helpers
// maps claim, the live sha256 and compares it to the recorded one — same
// idiom as release_manifest.mjs --check. It ALSO flags an unrecorded *.mjs
// sitting in the fully-managed .bee/bin/lib dir (file-set drift), mirroring
// bee.mjs's own computeRuntimeDrift extra-file check.
//
// This check REPORTS ONLY — it never rewrites onboarding.json. Self-healing
// here would hide the exact ordering bug it exists to catch (release, THEN
// re-tag without re-onboarding). The fix is always: re-run self-onboard.
//
// Hashing: reuses hashFile from .bee/bin/lib/fsutil.mjs — the SAME function
// buildManagedVersions (skills/bee-hive/scripts/onboard_bee.mjs) records
// with, and the SAME function computeRuntimeDrift (.bee/bin/bee.mjs) reads
// with. Never a second hashing implementation — recorder and checker can
// never disagree about what a vendored file's fingerprint is.
//
// Usage:
//   node scripts/ledger_parity.mjs --check [--root <path>]
//
// --root lets tests point this at a disposable fixture repo (must itself
// contain .bee/onboarding.json and .bee/bin/**); it defaults to this file's
// own repo root. The hashing algorithm is pure content hashing, not
// root-scoped, so borrowing THIS repo's hashFile to hash a fixture's files is
// exactly as valid as a fixture importing its own copy would be — and it
// keeps this checker from depending on a fixture carrying a full .bee/bin/lib
// tree of its own.
//
// Exit 0 when every recorded hash matches the live file and no unrecorded
// .mjs sits in the managed lib dir; exit 1 otherwise, naming every offending
// file and the one-line fix.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DEFAULT_REPO_ROOT = path.join(__dirname, "..");
const FIX_HINT = "node skills/bee-hive/scripts/onboard_bee.mjs --repo-root . --apply";

let cachedHashFile = null;
// Dynamic import: this file's own .bee/bin/lib/fsutil.mjs, the same module
// buildManagedVersions and computeRuntimeDrift both import hashFile from.
async function loadHashFile() {
  if (!cachedHashFile) {
    const mod = await import(path.join(DEFAULT_REPO_ROOT, ".bee", "bin", "lib", "fsutil.mjs"));
    cachedHashFile = mod.hashFile;
  }
  return cachedHashFile;
}

function relPosix(root, absPath) {
  return path.relative(root, absPath).split(path.sep).join("/");
}

/**
 * Recompute ledger parity for `root` (a repo containing .bee/onboarding.json
 * and .bee/bin/**). Read-only — never mutates anything on disk.
 *
 * Returns { ok, stale, missing, extra, error }:
 *   - stale:   recorded paths whose live sha256 no longer matches
 *   - missing: recorded paths with no file on disk
 *   - extra:   unrecorded *.mjs files present in the fully-managed lib dir
 *   - error:   a top-level reason (no ledger, malformed ledger, no managed
 *              map) when the check could not even run a comparison; ok is
 *              always false alongside a non-null error
 */
export async function computeLedgerParity(root) {
  const hashFile = await loadHashFile();
  const onboardingPath = path.join(root, ".bee", "onboarding.json");

  if (!fs.existsSync(onboardingPath)) {
    return {
      ok: false,
      stale: [],
      missing: [],
      extra: [],
      error: `no ledger found at ${relPosix(root, onboardingPath)} — run self-onboard first: ${FIX_HINT}`,
    };
  }

  let onboarding;
  try {
    onboarding = JSON.parse(fs.readFileSync(onboardingPath, "utf8"));
  } catch (err) {
    return {
      ok: false,
      stale: [],
      missing: [],
      extra: [],
      error: `ledger at ${relPosix(root, onboardingPath)} is unreadable/malformed: ${err.message}`,
    };
  }

  const managed = onboarding && typeof onboarding === "object" ? onboarding.managed : null;
  if (!managed || typeof managed !== "object") {
    return {
      ok: false,
      stale: [],
      missing: [],
      extra: [],
      error: `ledger at ${relPosix(root, onboardingPath)} carries no managed hash map — run self-onboard: ${FIX_HINT}`,
    };
  }

  const stale = [];
  const missing = [];

  const checkGroup = (recorded, relDir) => {
    if (!recorded || typeof recorded !== "object") return;
    for (const [name, recordedHash] of Object.entries(recorded)) {
      const abs = path.join(root, ".bee", "bin", relDir, name);
      const relPath = [".bee", "bin", relDir, name].filter(Boolean).join("/");
      let live;
      try {
        live = hashFile(abs);
      } catch {
        missing.push(relPath);
        continue;
      }
      if (live !== recordedHash) stale.push(relPath);
    }
  };
  checkGroup(managed.lib, "lib");
  checkGroup(managed.helpers, "");

  // File-set drift for the fully-managed lib dir only (helpers live beside
  // non-managed files in .bee/bin, so extra-detection there would false-positive
  // on ordinary unmanaged scripts) — same scoping as bee.mjs's computeRuntimeDrift.
  const extra = [];
  if (managed.lib && typeof managed.lib === "object") {
    const libDir = path.join(root, ".bee", "bin", "lib");
    try {
      for (const f of fs.readdirSync(libDir)) {
        if (f.endsWith(".mjs") && !Object.prototype.hasOwnProperty.call(managed.lib, f)) {
          extra.push(`.bee/bin/lib/${f}`);
        }
      }
    } catch {
      missing.push(".bee/bin/lib (unreadable)");
    }
  }

  stale.sort();
  missing.sort();
  extra.sort();

  const ok = stale.length === 0 && missing.length === 0 && extra.length === 0;
  return { ok, stale, missing, extra, error: null };
}

function printReport(result) {
  if (result.error) {
    console.error(`FAIL ledger_parity: ${result.error}`);
    return;
  }
  for (const p of result.stale) {
    console.error(`MISMATCH stale (ledger hash no longer matches live file): ${p}`);
  }
  for (const p of result.missing) {
    console.error(`MISMATCH missing (ledger claims this file, not found on disk): ${p}`);
  }
  for (const p of result.extra) {
    console.error(`MISMATCH extra (unrecorded .mjs in the fully-managed lib dir): ${p}`);
  }
  if (!result.ok) {
    console.error(
      `ledger_parity --check: FAIL (${result.stale.length} stale, ${result.missing.length} missing, ${result.extra.length} extra) — the .bee/onboarding.json managed-hash ledger is out of sync with .bee/bin/**. FIX: re-run self-onboard: ${FIX_HINT}`,
    );
  }
}

async function runCheck(root) {
  const result = await computeLedgerParity(root);
  if (result.ok) {
    console.log("ledger_parity --check: .bee/bin/** matches the .bee/onboarding.json managed-hash ledger");
    return 0;
  }
  printReport(result);
  return 1;
}

function parseRoot(args) {
  const i = args.indexOf("--root");
  if (i === -1) return DEFAULT_REPO_ROOT;
  const value = args[i + 1];
  if (!value) {
    console.error("usage: ledger_parity.mjs --check [--root <path>]");
    process.exit(1);
  }
  return path.resolve(value);
}

async function main() {
  const args = process.argv.slice(2);
  if (!args.includes("--check")) {
    console.error("usage: ledger_parity.mjs --check [--root <path>]");
    process.exit(1);
  }
  const root = parseRoot(args);
  let exitCode;
  try {
    exitCode = await runCheck(root);
  } catch (error) {
    console.error(`FAIL ledger_parity: ${error.message}`);
    process.exit(1);
  }
  process.exit(exitCode);
}

const isMain = process.argv[1] && path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url));
if (isMain) {
  main();
}
