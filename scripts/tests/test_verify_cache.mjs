#!/usr/bin/env node
// test_verify_cache.mjs — proves scripts/run_verify.mjs's suite-result cache
// (tbf-1, spec #80 P7): a content-hash cache that skips a suite whose
// impact-registry closure is byte-identical to its last GREEN run.
//
// Fixture-based, not against the live repo: each test case builds a fresh
// throwaway temp-dir "mini repo" — its own byte-copies of scripts/run_verify.mjs
// and scripts/impact_registry.mjs (the cache logic under test, never
// reimplemented) plus a couple of fake, instant, controllable-exit-code
// suites under scripts/tests/. run_verify.mjs derives REPO_ROOT from its own
// __dirname, so every path the cache touches (.bee/logs/verify-cache.json,
// the scripts/tests discovery glob) resolves inside the temp dir — the live
// repo's own verify-cache.json and real test suites are never read, run, or
// written by this file. Each fake suite is a real `node <file>` subprocess
// spawn through the real run_verify.mjs CLI (--only-scoped), so this proves
// the cache end to end, not just its exported primitives.

import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..", "..");
const RUN_VERIFY_SRC = path.join(REPO_ROOT, "scripts", "run_verify.mjs");
const IMPACT_REGISTRY_SRC = path.join(REPO_ROOT, "scripts", "impact_registry.mjs");

let passed = 0;
let failed = 0;
async function check(name, fn) {
  try {
    await fn();
    passed += 1;
    console.log(`PASS ${name}`);
  } catch (error) {
    failed += 1;
    console.error(`FAIL ${name}: ${error.stack ?? error.message}`);
  }
}

// Builds one throwaway fixture repo: scripts/run_verify.mjs + impact_registry.mjs
// (byte copies of the real files) plus two fake suites under scripts/tests/:
//   - test_fake_alone.mjs: no local imports, so its closure is only itself —
//     editing helper.mjs must never invalidate it.
//   - test_fake_dep.mjs: statically imports scripts/lib/helper.mjs, so its
//     closure_sha moves whenever helper.mjs's content changes.
// Both exit codes are controllable via env var (unchanged file bytes across
// red/green toggles), so the "red is never cached" case never depends on a
// closure-sha change to prove its point.
function makeFixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "verify-cache-fixture-"));
  const scriptsDir = path.join(root, "scripts");
  const testsDir = path.join(scriptsDir, "tests");
  const libDir = path.join(scriptsDir, "lib");
  fs.mkdirSync(testsDir, { recursive: true });
  fs.mkdirSync(libDir, { recursive: true });
  fs.copyFileSync(RUN_VERIFY_SRC, path.join(scriptsDir, "run_verify.mjs"));
  fs.copyFileSync(IMPACT_REGISTRY_SRC, path.join(scriptsDir, "impact_registry.mjs"));

  fs.writeFileSync(
    path.join(testsDir, "test_fake_alone.mjs"),
    'process.exit(Number(process.env.FAKE_ALONE_EXIT ?? "0"));\n',
  );
  fs.writeFileSync(
    path.join(testsDir, "test_fake_dep.mjs"),
    'import "../lib/helper.mjs";\nprocess.exit(Number(process.env.FAKE_DEP_EXIT ?? "0"));\n',
  );
  fs.writeFileSync(path.join(libDir, "helper.mjs"), "export const v = 1;\n");

  return {
    root,
    runVerify: path.join(scriptsDir, "run_verify.mjs"),
    helperPath: path.join(libDir, "helper.mjs"),
    cachePath: path.join(root, ".bee", "logs", "verify-cache.json"),
  };
}

function run(fixture, args, env = {}) {
  return spawnSync(process.execPath, [fixture.runVerify, ...args], {
    cwd: fixture.root,
    encoding: "utf8",
    timeout: 30000,
    env: { ...process.env, ...env },
  });
}

function cleanup(fixture) {
  fs.rmSync(fixture.root, { recursive: true, force: true });
}

// ── (1)-(3): cold populate, warm skip, closure-edit invalidation ───────────
const fx = makeFixture();
const BOTH_ARGS = ["--only", "scripts/tests/test_fake_alone.mjs,scripts/tests/test_fake_dep.mjs"];
try {
  await check("cold run (no cache file yet): both fake suites run for real, PASS, and a green cache file is written", () => {
    assert.ok(!fs.existsSync(fx.cachePath), "fixture assumption: cache file must not pre-exist");
    const r = run(fx, BOTH_ARGS);
    assert.equal(r.status, 0, `expected exit 0; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/tests\/test_fake_alone\.mjs/, r.stdout);
    assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/tests\/test_fake_dep\.mjs/, r.stdout);
    assert.doesNotMatch(r.stdout, /CACHED/, "a cold run must never print CACHED");
    assert.match(r.stdout, /\(2 run, 0 cached\)/, r.stdout);
    assert.ok(fs.existsSync(fx.cachePath), "cache file must exist after a green run");
    const cache = JSON.parse(fs.readFileSync(fx.cachePath, "utf8"));
    assert.equal(cache["scripts/tests/test_fake_alone.mjs"].result, "green");
    assert.equal(cache["scripts/tests/test_fake_dep.mjs"].result, "green");
  });

  await check("identical rerun: both suites reported CACHED green, zero real executions", () => {
    const r = run(fx, BOTH_ARGS);
    assert.equal(r.status, 0, `expected exit 0; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(r.stdout, /CACHED green scripts\/tests\/test_fake_alone\.mjs \(closure unchanged\)/, r.stdout);
    assert.match(r.stdout, /CACHED green scripts\/tests\/test_fake_dep\.mjs \(closure unchanged\)/, r.stdout);
    assert.match(r.stdout, /\(0 run, 2 cached\)/, r.stdout);
  });

  await check("closure-file edit invalidates exactly the dependent suite, leaving the unrelated one cached", () => {
    fs.writeFileSync(fx.helperPath, "export const v = 2;\n");
    const r = run(fx, BOTH_ARGS);
    assert.equal(r.status, 0, `expected exit 0; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(r.stdout, /CACHED green scripts\/tests\/test_fake_alone\.mjs \(closure unchanged\)/, "test_fake_alone has no dependency on helper.mjs and must stay cached");
    assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/tests\/test_fake_dep\.mjs/, "test_fake_dep imports helper.mjs and must re-run for real");
    assert.doesNotMatch(r.stdout, /CACHED green scripts\/tests\/test_fake_dep\.mjs/, r.stdout);
    assert.match(r.stdout, /\(1 run, 1 cached\)/, r.stdout);
  });
} finally {
  cleanup(fx);
}

// ── (4): red is never cached ────────────────────────────────────────────────
await check("a red suite is never cached: it re-runs for real on every subsequent identical run, and the first green run after it also runs for real", async () => {
  const fx2 = makeFixture();
  try {
    const ONLY_ALONE = ["--only", "scripts/tests/test_fake_alone.mjs"];
    const rRed = run(fx2, ONLY_ALONE, { FAKE_ALONE_EXIT: "1" });
    assert.equal(rRed.status, 1, `a red suite must fail the run; stdout:\n${rRed.stdout}`);
    assert.match(rRed.stdout, /FAIL\s+\d+ms\s+scripts\/tests\/test_fake_alone\.mjs/, rRed.stdout);
    const cacheAfterRed = fs.existsSync(fx2.cachePath) ? JSON.parse(fs.readFileSync(fx2.cachePath, "utf8")) : {};
    assert.ok(!cacheAfterRed["scripts/tests/test_fake_alone.mjs"], "a red result must never be written to the cache");

    const rRerun = run(fx2, ONLY_ALONE, { FAKE_ALONE_EXIT: "1" });
    assert.equal(rRerun.status, 1, rRerun.stdout);
    assert.doesNotMatch(rRerun.stdout, /CACHED/, "an uncached (never-green) suite must always execute for real, never CACHED");

    const rGreen = run(fx2, ONLY_ALONE, { FAKE_ALONE_EXIT: "0" });
    assert.equal(rGreen.status, 0, rGreen.stdout);
    assert.match(rGreen.stdout, /PASS\s+\d+ms\s+scripts\/tests\/test_fake_alone\.mjs/, "the first green run after reds must execute for real, not CACHED");
    const cacheAfterGreen = JSON.parse(fs.readFileSync(fx2.cachePath, "utf8"));
    assert.equal(cacheAfterGreen["scripts/tests/test_fake_alone.mjs"].result, "green");
  } finally {
    cleanup(fx2);
  }
});

// ── (5): corrupt cache degrades to a cache miss, never a crash ─────────────
await check("a corrupt cache file degrades to a cache miss (fail-open), never a crash — the suite runs for real and the cache is repaired", () => {
  const fx3 = makeFixture();
  try {
    fs.mkdirSync(path.dirname(fx3.cachePath), { recursive: true });
    fs.writeFileSync(fx3.cachePath, "{ not valid json !!!");
    const r = run(fx3, ["--only", "scripts/tests/test_fake_alone.mjs"]);
    assert.equal(r.status, 0, `a corrupt cache must never crash the run; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/tests\/test_fake_alone\.mjs/, "corrupt cache = cache miss, so the suite must run for real");
    assert.doesNotMatch(r.stdout, /CACHED/, r.stdout);
    const cache = JSON.parse(fs.readFileSync(fx3.cachePath, "utf8"));
    assert.equal(cache["scripts/tests/test_fake_alone.mjs"].result, "green", "the cache file must be repaired (valid JSON) after this run");
  } finally {
    cleanup(fx3);
  }
});

// ── (6)-(7): CI env var and --no-cache both disable the cache outright ─────
await check("the CI env var disables the cache entirely: a suite with a valid green cache entry still runs for real, and the cache file is left untouched", () => {
  const fx4 = makeFixture();
  try {
    const ONLY_ALONE = ["--only", "scripts/tests/test_fake_alone.mjs"];
    run(fx4, ONLY_ALONE); // prime a green cache entry
    const before = fs.readFileSync(fx4.cachePath, "utf8");
    const r = run(fx4, ONLY_ALONE, { CI: "true" });
    assert.equal(r.status, 0, `expected exit 0; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/tests\/test_fake_alone\.mjs/, "CI must always run for real, never CACHED");
    assert.doesNotMatch(r.stdout, /CACHED/, r.stdout);
    const after = fs.readFileSync(fx4.cachePath, "utf8");
    assert.equal(after, before, "a CI-disabled run must never touch the cache file");
  } finally {
    cleanup(fx4);
  }
});

await check("--no-cache bypasses the cache the same way CI does: real execution, cache file left untouched", () => {
  const fx5 = makeFixture();
  try {
    const ONLY_ALONE = ["--only", "scripts/tests/test_fake_alone.mjs"];
    run(fx5, ONLY_ALONE); // prime a green cache entry
    const before = fs.readFileSync(fx5.cachePath, "utf8");
    const r = run(fx5, [...ONLY_ALONE, "--no-cache"]);
    assert.equal(r.status, 0, `expected exit 0; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/tests\/test_fake_alone\.mjs/, r.stdout);
    assert.doesNotMatch(r.stdout, /CACHED/, r.stdout);
    const after = fs.readFileSync(fx5.cachePath, "utf8");
    assert.equal(after, before, "--no-cache must never touch the cache file");
  } finally {
    cleanup(fx5);
  }
});

// ── (8): --cache-clear wipes the file ───────────────────────────────────────
await check("--cache-clear wipes the cache file and prints the cleared banner, forcing a real run even with a valid green entry", () => {
  const fx6 = makeFixture();
  try {
    const ONLY_ALONE = ["--only", "scripts/tests/test_fake_alone.mjs"];
    run(fx6, ONLY_ALONE); // prime a green cache entry
    assert.ok(fs.existsSync(fx6.cachePath), "fixture assumption: cache must exist before clearing");
    const r = run(fx6, [...ONLY_ALONE, "--cache-clear"]);
    assert.equal(r.status, 0, `expected exit 0; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(r.stdout, /VERIFY CACHE: cleared/, r.stdout);
    assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/tests\/test_fake_alone\.mjs/, "post-clear run must execute for real, not CACHED");
    assert.doesNotMatch(r.stdout, /CACHED green/, r.stdout);
  } finally {
    cleanup(fx6);
  }
});

console.log(`\ntest_verify_cache: ${passed} passed, ${failed} failed`);
if (failed) process.exit(1);
