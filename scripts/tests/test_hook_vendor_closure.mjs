#!/usr/bin/env node
// test_hook_vendor_closure.mjs — regression guard for cell i54-closeout-9.
//
// Canary P5 (docs/history/i54-closeout/reports/validation-canary.md §1) caught
// a live bug: hooks/bee-write-guard.mjs imports ./tokenize-command.mjs, but
// HOOK_FILENAMES in skills/bee-hive/scripts/onboard_bee.mjs never listed it,
// so a fresh --repo-hooks install never vendored the file and the write guard
// crashed ERR_MODULE_NOT_FOUND — a hard import-time crash, invisible to the
// guard's own try/catch fail-open path — on every fresh install, silently
// disabling the pre-Gate-3 write block instead of enforcing it.
//
// Two checks, cheapest first:
//   1. STATIC CLOSURE — parse HOOK_FILENAMES straight out of onboard_bee.mjs's
//      source (never hand-copy the list into this file — a stale copy would
//      silently drift exactly like the bug this guards against) and statically
//      parse every relative (`./x.mjs`) import out of each hooks/ file it
//      names. Every imported basename must itself be listed. This is the
//      fast, always-on guard against the CLASS of drift: any hook gaining a
//      new sibling import the vendoring step misses. Only same-directory
//      (`./x.mjs`) imports are in scope — no shipped hook currently imports
//      `../lib/...`; if one ever does, this check needs a matching branch
//      before it can claim coverage of that shape.
//      A self-test proves the parser actually bites before it is trusted
//      against the real tree.
//   2. FRESH-INSTALL PROOF — actually run onboard_bee.mjs --apply --repo-hooks
//      against a brand-new temp repo (same isolation pattern as
//      skills/bee-hive/scripts/test_onboard_bee.mjs: fake HOME, throwaway
//      target dir), then spawn the VENDORED copy of bee-write-guard.mjs — not
//      the canonical source, whose sibling import always resolves and so
//      never reproduces the bug — and assert it never crashes with
//      ERR_MODULE_NOT_FOUND. `node --check` was considered and rejected here:
//      it only validates syntax, not import resolution (verified empirically:
//      `node --check` exits 0 on a file importing a nonexistent sibling), so
//      it cannot prove this closure. A real subprocess run is the only proof
//      that actually reproduces canary P5's failure mode.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { runModuleWorker } from "../lib/run-module-worker.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");
const ONBOARD = path.join(REPO_ROOT, "skills", "bee-hive", "scripts", "onboard_bee.mjs");
const HOOKS_DIR = path.join(REPO_ROOT, "packages", "bee", "hooks");

let failures = 0;
function check(condition, label, extra = "") {
  if (condition) {
    console.log(`PASS ${label}`);
  } else {
    failures += 1;
    console.error(`FAIL ${label}${extra ? ` :: ${extra}` : ""}`);
  }
}

// --- shared parser: extract HOOK_FILENAMES + closure violations -----------

const RELATIVE_IMPORT_RE = /(?:import|export)\s+(?:[^"';]*?\s+from\s+)?["']\.\/([^"']+\.mjs)["']/g;

function extractHookFilenames(onboardSource) {
  const match = onboardSource.match(/const HOOK_FILENAMES\s*=\s*\[([\s\S]*?)\];/);
  if (!match) {
    throw new Error("extractHookFilenames: could not locate a HOOK_FILENAMES array in onboard_bee.mjs");
  }
  const names = [...match[1].matchAll(/"([^"]+\.mjs)"/g)].map((m) => m[1]);
  if (names.length === 0) {
    throw new Error("extractHookFilenames: HOOK_FILENAMES parsed as empty — regex likely stale");
  }
  return names;
}

/**
 * For every file named in `hookFilenames`, read it from `hooksDir` and
 * statically parse its same-directory (`./x.mjs`) imports. Returns a list of
 * violations: { importer, missing } for every imported basename not itself
 * present in hookFilenames. A named file absent from hooksDir is skipped —
 * not this check's concern; the vendoring step itself fails loudly on that.
 */
function findClosureViolations(hooksDir, hookFilenames) {
  const known = new Set(hookFilenames);
  const violations = [];
  for (const name of hookFilenames) {
    const filePath = path.join(hooksDir, name);
    if (!fs.existsSync(filePath)) continue;
    const source = fs.readFileSync(filePath, "utf8");
    for (const m of source.matchAll(RELATIVE_IMPORT_RE)) {
      const importedName = m[1];
      if (!known.has(importedName)) {
        violations.push({ importer: name, missing: importedName });
      }
    }
  }
  return violations;
}

// --- 1. self-test: prove the parser actually bites before trusting it -----
{
  const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), "test-hook-vendor-closure-selftest-"));
  try {
    const hooksFixture = path.join(tmpRoot, "hooks");
    fs.mkdirSync(hooksFixture);
    fs.writeFileSync(
      path.join(hooksFixture, "listed-a.mjs"),
      'import { x } from "./listed-b.mjs";\nimport { y } from "./unlisted.mjs";\n',
    );
    fs.writeFileSync(path.join(hooksFixture, "listed-b.mjs"), "export const x = 1;\n");
    fs.writeFileSync(path.join(hooksFixture, "unlisted.mjs"), "export const y = 2;\n");

    const violations = findClosureViolations(hooksFixture, ["listed-a.mjs", "listed-b.mjs"]);
    const bites =
      violations.length === 1 && violations[0].importer === "listed-a.mjs" && violations[0].missing === "unlisted.mjs";
    check(bites, "self-test: closure parser flags an unlisted sibling import", JSON.stringify(violations));

    const clean = findClosureViolations(hooksFixture, ["listed-a.mjs", "listed-b.mjs", "unlisted.mjs"]);
    check(clean.length === 0, "self-test: closure parser is silent once the import is listed", JSON.stringify(clean));
  } finally {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  }
}

// --- 2. real check: onboard_bee.mjs's HOOK_FILENAMES is import-closed ------

const onboardSource = fs.readFileSync(ONBOARD, "utf8");
const hookFilenames = extractHookFilenames(onboardSource);
check(
  hookFilenames.includes("tokenize-command.mjs"),
  "HOOK_FILENAMES lists tokenize-command.mjs (i54-closeout-9 fix)",
);

const realViolations = findClosureViolations(HOOKS_DIR, hookFilenames);
check(
  realViolations.length === 0,
  "HOOK_FILENAMES is import-closed: every relative import of a listed hook is itself listed",
  realViolations.map((v) => `${v.importer} imports unlisted ./${v.missing}`).join("; "),
);

// --- 3. fresh-install proof: a real --repo-hooks apply actually vendors the
//        file, and the vendored guard never crashes at import (canary P5) --

async function freshInstallProof() {
  const fakeHome = fs.mkdtempSync(path.join(os.tmpdir(), "bee-hook-closure-home-"));
  const target = fs.mkdtempSync(path.join(os.tmpdir(), "bee-hook-closure-target-"));
  try {
    const env = { ...process.env, HOME: fakeHome, USERPROFILE: fakeHome };
    const applyResult = await runModuleWorker(ONBOARD, {
      args: ["--repo-root", target, "--apply", "--repo-hooks", "--json"],
      env,
      fakeHome,
    });
    let payload = null;
    try {
      payload = JSON.parse(applyResult.stdout || "null");
    } catch {
      payload = null;
    }
    check(
      payload?.status === "applied",
      "fresh --repo-hooks apply succeeds",
      applyResult.stdout || applyResult.stderr,
    );

    const vendoredTokenizer = path.join(target, ".bee", "bin", "hooks", "tokenize-command.mjs");
    check(fs.existsSync(vendoredTokenizer), "fresh install vendors .bee/bin/hooks/tokenize-command.mjs");

    const vendoredGuard = path.join(target, ".bee", "bin", "hooks", "bee-write-guard.mjs");
    check(fs.existsSync(vendoredGuard), "fresh install vendors .bee/bin/hooks/bee-write-guard.mjs");

    if (fs.existsSync(vendoredGuard)) {
      const guardResult = await runModuleWorker(vendoredGuard, {
        input: JSON.stringify({ cwd: target, tool_name: "Read", tool_input: { file_path: "README.md" } }),
        cwd: target,
        env,
      });
      const crashedAtImport = /ERR_MODULE_NOT_FOUND/.test(guardResult.stderr || "");
      check(
        !crashedAtImport,
        "vendored bee-write-guard.mjs never crashes ERR_MODULE_NOT_FOUND at import (canary P5 regression)",
        guardResult.stderr,
      );
    }
  } finally {
    fs.rmSync(fakeHome, { recursive: true, force: true });
    fs.rmSync(target, { recursive: true, force: true });
  }
}

await freshInstallProof();

if (failures > 0) {
  console.error(`FAIL test_hook_vendor_closure: ${failures} check(s) failed`);
  process.exit(1);
}
console.log("PASS test_hook_vendor_closure: all checks passed");
