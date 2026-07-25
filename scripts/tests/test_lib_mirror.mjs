#!/usr/bin/env node
// Guard: packages/bee/lib/ and .bee/bin/lib/ must stay byte-
// identical mirrors of each other. PROJ-08 (SPEC), crit-pattern 20260714
// (derive file lists, never hand-enumerate): the template copy is what ships
// to a fresh onboard, the .bee/bin copy is what actually runs in this repo —
// if they drift, a fix lands in one and silently never reaches the other.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");

const TEMPLATES_LIB = path.join(REPO_ROOT, "packages", "bee", "lib");
const BIN_LIB = path.join(REPO_ROOT, ".bee", "bin", "lib");
const SOURCE_HOOKS = path.join(REPO_ROOT, "hooks");
const BIN_HOOKS = path.join(REPO_ROOT, ".bee", "bin", "hooks");
const HOOK_PROJECTIONS = [path.join(SOURCE_HOOKS, "hooks.json"), path.join(SOURCE_HOOKS, "claude-hooks.json")];

/**
 * Lists the plain files (not directories) directly inside dir, derived via
 * fs.readdirSync — never a hand-maintained list.
 */
function listFiles(dir) {
  return fs
    .readdirSync(dir, { withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => entry.name)
    .sort();
}

/**
 * Compares two directories of flat files: every file present in one side
 * must be present and byte-identical in the other. Returns
 * { ok, missingInB, extraInB, diffing } where:
 *   - missingInB: files present in dirA but absent from dirB
 *   - extraInB:   files present in dirB but absent from dirA
 *   - diffing:    files present in both but with different byte content
 */
function compareDirs(dirA, dirB) {
  const filesA = new Set(listFiles(dirA));
  const filesB = new Set(listFiles(dirB));

  const missingInB = [...filesA].filter((f) => !filesB.has(f)).sort();
  const extraInB = [...filesB].filter((f) => !filesA.has(f)).sort();

  const common = [...filesA].filter((f) => filesB.has(f)).sort();
  const diffing = common.filter((file) => {
    const bufA = fs.readFileSync(path.join(dirA, file));
    const bufB = fs.readFileSync(path.join(dirB, file));
    return !bufA.equals(bufB);
  });

  const ok = missingInB.length === 0 && extraInB.length === 0 && diffing.length === 0;
  return { ok, missingInB, extraInB, diffing };
}

function launcherModules(projectionFiles) {
  const out = new Set();
  const visit = (value) => {
    if (Array.isArray(value)) return value.forEach(visit);
    if (!value || typeof value !== "object") return;
    if (typeof value.command === "string") {
      for (const match of value.command.matchAll(/(?:^|[\\/])(bee-[A-Za-z0-9-]+\.mjs)(?=["'\s]|$)/g)) {
        out.add(match[1]);
      }
    }
    Object.values(value).forEach(visit);
  };
  for (const file of projectionFiles) visit(JSON.parse(fs.readFileSync(file, "utf8")));
  return out;
}

function runtimeHookInventory(sourceDir, projectionFiles) {
  const expected = launcherModules(projectionFiles);
  const queue = [...expected];
  const importRe = /(?:import|export)\s+(?:[^"']*?\s+from\s+)?["']\.\/([^"']+\.mjs)["']/g;
  while (queue.length) {
    const name = queue.shift();
    const file = path.join(sourceDir, name);
    if (!fs.existsSync(file)) throw new Error(`runtime hook inventory: launcher/import target missing: ${name}`);
    const text = fs.readFileSync(file, "utf8");
    for (const match of text.matchAll(importRe)) {
      const dep = path.basename(match[1]);
      if (dep !== match[1] || !dep.endsWith(".mjs")) {
        throw new Error(`runtime hook inventory: unsafe relative import from ${name}: ${match[1]}`);
      }
      if (!expected.has(dep)) {
        expected.add(dep);
        queue.push(dep);
      }
    }
  }
  return [...expected].sort();
}

function compareInventory(expected, sourceDir, runtimeDir) {
  const expectedSet = new Set(expected);
  const actual = new Set(listFiles(runtimeDir));
  const missing = expected.filter((name) => !actual.has(name));
  const extra = [...actual].filter((name) => !expectedSet.has(name)).sort();
  const diffing = expected
    .filter((name) => actual.has(name))
    .filter((name) => !fs.readFileSync(path.join(sourceDir, name)).equals(fs.readFileSync(path.join(runtimeDir, name))));
  return { ok: missing.length === 0 && extra.length === 0 && diffing.length === 0, missing, extra, diffing };
}

// ─── internal self-test: prove the checker actually bites ─────────────────
// Build two TEMP directories (never the real tree), seed them identically,
// then inject a single byte-diff into one file on the copy side and assert
// compareDirs() flags exactly that file. Everything is cleaned up after.
{
  const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), "test-lib-mirror-selftest-"));
  const dirA = path.join(tmpRoot, "a");
  const dirB = path.join(tmpRoot, "b");
  fs.mkdirSync(dirA);
  fs.mkdirSync(dirB);

  try {
    fs.writeFileSync(path.join(dirA, "same.mjs"), "export const x = 1;\n");
    fs.writeFileSync(path.join(dirB, "same.mjs"), "export const x = 1;\n");
    fs.writeFileSync(path.join(dirA, "diverges.mjs"), "export const y = 2;\n");
    fs.writeFileSync(path.join(dirB, "diverges.mjs"), "export const y = 3;\n"); // injected byte-diff
    fs.writeFileSync(path.join(dirA, "only-in-a.mjs"), "export const z = 4;\n");
    fs.writeFileSync(path.join(dirB, "only-in-b.mjs"), "export const w = 5;\n");

    const selfTestResult = compareDirs(dirA, dirB);

    const caughtDiff = selfTestResult.diffing.includes("diverges.mjs");
    const caughtMissing = selfTestResult.missingInB.includes("only-in-a.mjs");
    const caughtExtra = selfTestResult.extraInB.includes("only-in-b.mjs");

    if (selfTestResult.ok || !caughtDiff || !caughtMissing || !caughtExtra) {
      console.error("FAIL test_lib_mirror: internal self-test did not catch the injected byte-diff / missing / extra files");
      console.error(`      checker result: ${JSON.stringify(selfTestResult)}`);
      process.exit(1);
    }

    console.log("PASS test_lib_mirror: internal self-test — checker correctly flags injected byte-diff, missing file, and extra file");
  } finally {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  }
}

// Derivation self-test: launcher -> transitive same-directory import, plus
// missing/extra/byte-drift detection. Source-only catalog/tests/config never
// enter expected unless a production launcher imports them.
{
  const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), "test-hook-mirror-selftest-"));
  const source = path.join(tmpRoot, "source");
  const runtime = path.join(tmpRoot, "runtime");
  fs.mkdirSync(source);
  fs.mkdirSync(runtime);
  try {
    fs.writeFileSync(path.join(tmpRoot, "hooks.json"), JSON.stringify({ hooks: { Stop: [{ hooks: [{ command: 'node "${CLAUDE_PLUGIN_ROOT}/hooks/bee-a.mjs"' }] }] } }));
    fs.writeFileSync(path.join(source, "bee-a.mjs"), 'import "./adapter.mjs";\n');
    fs.writeFileSync(path.join(source, "adapter.mjs"), "export const x = 1;\n");
    fs.writeFileSync(path.join(source, "catalog.mjs"), "source only\n");
    fs.copyFileSync(path.join(source, "bee-a.mjs"), path.join(runtime, "bee-a.mjs"));
    fs.writeFileSync(path.join(runtime, "adapter.mjs"), "export const x = 2;\n");
    fs.writeFileSync(path.join(runtime, "extra.mjs"), "extra\n");
    const expected = runtimeHookInventory(source, [path.join(tmpRoot, "hooks.json")]);
    const injected = compareInventory(expected, source, runtime);
    if (
      expected.join(",") !== "adapter.mjs,bee-a.mjs" ||
      !injected.diffing.includes("adapter.mjs") ||
      !injected.extra.includes("extra.mjs") ||
      injected.missing.length !== 0
    ) {
      throw new Error(`hook inventory self-test did not catch derivation/diff/extra: ${JSON.stringify({ expected, injected })}`);
    }
    fs.rmSync(path.join(runtime, "bee-a.mjs"));
    const missing = compareInventory(expected, source, runtime);
    if (!missing.missing.includes("bee-a.mjs")) {
      throw new Error(`hook inventory self-test did not catch missing launcher: ${JSON.stringify(missing)}`);
    }
    console.log("PASS test_lib_mirror: runtime hook inventory derives launchers/imports and catches missing, extra, and byte drift");
  } finally {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  }
}

// ─── real check: THIS repo's real packages/bee/lib <-> .bee/bin/lib mirror ─

if (!fs.existsSync(TEMPLATES_LIB)) {
  console.error(`FAIL test_lib_mirror: ${TEMPLATES_LIB} does not exist`);
  process.exit(1);
}
if (!fs.existsSync(BIN_LIB)) {
  console.error(`FAIL test_lib_mirror: ${BIN_LIB} does not exist`);
  process.exit(1);
}

const result = compareDirs(TEMPLATES_LIB, BIN_LIB);
if (!result.ok) {
  console.error("FAIL test_lib_mirror: packages/bee/lib/ and .bee/bin/lib/ have drifted:");
  if (result.missingInB.length) {
    console.error(`      present in packages/bee/lib but missing from .bee/bin/lib: ${result.missingInB.join(", ")}`);
  }
  if (result.extraInB.length) {
    console.error(`      present in .bee/bin/lib but missing from packages/bee/lib: ${result.extraInB.join(", ")}`);
  }
  if (result.diffing.length) {
    console.error(`      byte-diffing in both: ${result.diffing.join(", ")}`);
  }
  process.exit(1);
}

console.log(`PASS test_lib_mirror: packages/bee/lib and .bee/bin/lib are byte-identical (${listFiles(TEMPLATES_LIB).length} files)`);

if (!fs.existsSync(BIN_HOOKS)) {
  console.error(`FAIL test_lib_mirror: ${BIN_HOOKS} does not exist`);
  process.exit(1);
}
const hookExpected = runtimeHookInventory(SOURCE_HOOKS, HOOK_PROJECTIONS);
const hookResult = compareInventory(hookExpected, SOURCE_HOOKS, BIN_HOOKS);
if (!hookResult.ok) {
  console.error(`FAIL test_lib_mirror: runtime-derived hook mirror drifted: ${JSON.stringify(hookResult)}`);
  process.exit(1);
}
console.log(`PASS test_lib_mirror: runtime-derived hook inventory is byte-identical (${hookExpected.length} files)`);
