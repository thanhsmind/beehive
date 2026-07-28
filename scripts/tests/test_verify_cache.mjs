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

// Env vars that change the CHILD run_verify's cache behavior. The parent's
// own environment must never decide them (p1-1 F1): under `CI=true` — which
// is exactly how ci.yml runs this file — every child saw the cache disabled
// and all eight cache assertions failed, while the same suite passed locally.
// A case that is ABOUT one of these passes it explicitly in `env`, and that
// explicit value wins; every other case gets a clean, cache-enabled child.
const CACHE_ENV_KEYS = ["CI", "BEE_CHECK_ONLY", "BEE_VERIFY_ONLY", "BEE_VERIFY_EXCLUDE", "BEE_VERIFY_ROOT_FILTER"];

function run(fixture, args, env = {}) {
  const childEnv = { ...process.env, ...env };
  for (const key of CACHE_ENV_KEYS) {
    if (!(key in env)) delete childEnv[key];
  }
  return spawnSync(process.execPath, [fixture.runVerify, ...args], {
    cwd: fixture.root,
    encoding: "utf8",
    timeout: 30000,
    env: childEnv,
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
    assert.match(r.stdout, /cache=disabled \(CI\)/, "the summary must name why the cache was bypassed");
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

// ── (9)-(12): the cache must never report green for checks it did not run ──
//
// p1-1 (F2): three independent reviewers found three ways the cache could
// serve a green result for work that never executed. Each is pinned here.

function writeFixtureFile(fixture, rel, content) {
  const abs = path.join(fixture.root, rel);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, content);
  return abs;
}

function cacheEntries(fixture) {
  if (!fs.existsSync(fixture.cachePath)) return {};
  return JSON.parse(fs.readFileSync(fixture.cachePath, "utf8"));
}

// (9) F2a: BEE_CHECK_ONLY scopes which checks run INSIDE a suite and is not
// part of any cache key, so a filtered run must be treated exactly like
// --no-cache — otherwise its partial green is stored under the suite's FULL
// closure sha and every later unfiltered run reports CACHED green for checks
// that never executed once.
await check("BEE_CHECK_ONLY disables the cache outright: a filtered run neither reads nor writes it, and the summary says so", () => {
  const fx7 = makeFixture();
  try {
    run(fx7, ["--only", "scripts/tests/test_fake_alone.mjs"]); // prime ONE green entry
    const before = fs.readFileSync(fx7.cachePath, "utf8");
    assert.ok(!cacheEntries(fx7)["scripts/tests/test_fake_dep.mjs"], "fixture assumption: only test_fake_alone is primed");

    const r = run(fx7, BOTH_ARGS, { BEE_CHECK_ONLY: "some check name" });
    assert.equal(r.status, 0, `expected exit 0; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.doesNotMatch(r.stdout, /CACHED/, "a filtered run must never READ the cache");
    assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/tests\/test_fake_alone\.mjs/, r.stdout);
    assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/tests\/test_fake_dep\.mjs/, r.stdout);
    assert.match(r.stdout, /cache=disabled \(BEE_CHECK_ONLY\)/, "a filtered run must be visibly uncached in the summary");
    assert.equal(fs.readFileSync(fx7.cachePath, "utf8"), before, "a filtered run must never WRITE the cache");
  } finally {
    cleanup(fx7);
  }
});

// (10) F2c: the impact registry is built by static import analysis, so files a
// suite merely readFileSync's are invisible to its closure. Exercised against
// run_verify.mjs's REAL declaration for scripts/skill_budget_fence.mjs (its
// budget JSON + every skills/**/*.md body), not a synthetic one.
await check("a declared extra input invalidates that suite's cache entry: editing a skill body, and adding a new one, both force a real re-run", () => {
  const fx8 = makeFixture();
  try {
    const FENCE = ["--only", "skill_budget_fence"];
    const CACHED_FENCE = /CACHED green scripts\/skill_budget_fence\.mjs \(closure unchanged\)/;
    const RAN_FENCE = /PASS\s+\d+ms\s+scripts\/skill_budget_fence\.mjs\s*$/m;
    writeFixtureFile(fx8, "scripts/skill_budget_fence.mjs", "process.exit(0);\n");
    writeFixtureFile(fx8, "scripts/skill-body-budget.json", '{"demo": 100}\n');
    const skillBody = writeFixtureFile(fx8, "skills/demo/SKILL.md", "# demo\n");

    const cold = run(fx8, FENCE);
    assert.equal(cold.status, 0, `expected exit 0; stdout:\n${cold.stdout}\nstderr:\n${cold.stderr}`);
    assert.match(cold.stdout, RAN_FENCE, cold.stdout);
    assert.equal(cacheEntries(fx8)["scripts/skill_budget_fence.mjs"]?.result, "green", "a declared-input suite is still cacheable");

    const warm = run(fx8, FENCE);
    assert.match(warm.stdout, CACHED_FENCE, "nothing changed, so the entry must still be served");

    fs.writeFileSync(skillBody, "# demo\n\nnow much longer\n");
    const afterEdit = run(fx8, FENCE);
    assert.equal(afterEdit.status, 0, afterEdit.stdout);
    assert.doesNotMatch(afterEdit.stdout, CACHED_FENCE, "editing a declared extra input must invalidate the entry");
    assert.match(afterEdit.stdout, RAN_FENCE, "the suite must execute for real after its declared input changed");

    writeFixtureFile(fx8, "skills/second/SKILL.md", "# second\n");
    const afterAdd = run(fx8, FENCE);
    assert.equal(afterAdd.status, 0, afterAdd.stdout);
    assert.doesNotMatch(afterAdd.stdout, CACHED_FENCE, "a NEW file matching a declared glob must invalidate the entry too");
    assert.match(afterAdd.stdout, RAN_FENCE, afterAdd.stdout);
  } finally {
    cleanup(fx8);
  }
});

// (11) F2c, the explicit escape: a broad repo scanner whose real inputs cannot
// be enumerated opts OUT of caching entirely rather than be cached wrongly.
await check("an explicitly opted-out suite is never cached: it re-runs for real forever and no entry is ever written for it", () => {
  const fx9 = makeFixture();
  try {
    const CENSUS = ["--only", "census_stale_spawn_syntax"];
    writeFixtureFile(fx9, "scripts/census_stale_spawn_syntax.mjs", "process.exit(0);\n");

    for (const pass of ["first", "second"]) {
      const r = run(fx9, CENSUS);
      assert.equal(r.status, 0, `${pass} run: expected exit 0; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
      assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/census_stale_spawn_syntax\.mjs/, `${pass} run must execute for real`);
      assert.doesNotMatch(r.stdout, /CACHED/, `${pass} run: an opted-out suite must never be served from the cache`);
      assert.ok(
        !cacheEntries(fx9)["scripts/census_stale_spawn_syntax.mjs"],
        `${pass} run: an opted-out suite must never be WRITTEN to the cache either`,
      );
    }
  } finally {
    cleanup(fx9);
  }
});

// (12) F2b, the default when in doubt: a suite that names a live repo surface
// the registry cannot walk, and declares no extra inputs, is auto-opted-out.
// Without this, editing a skill body or the operating block left the closure
// sha unchanged and the suite reported CACHED green without running — the
// exact defect class that broke three times in one session.
await check("a suite that reads an undeclared repo surface defaults to opt-out: mentioning the operating block alone makes it uncacheable", () => {
  const fx10 = makeFixture();
  try {
    const PROSE = ["--only", "test_fake_prose"];
    writeFixtureFile(
      fx10,
      "scripts/tests/test_fake_prose.mjs",
      "// reads the repo operating block at runtime: AGENTS.md\nprocess.exit(0);\n",
    );

    for (const pass of ["first", "second"]) {
      const r = run(fx10, PROSE);
      assert.equal(r.status, 0, `${pass} run: expected exit 0; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
      assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/tests\/test_fake_prose\.mjs/, `${pass} run must execute for real`);
      assert.doesNotMatch(r.stdout, /CACHED/, `${pass} run: an undeclared-input suite must never be served from the cache`);
      assert.ok(
        !cacheEntries(fx10)["scripts/tests/test_fake_prose.mjs"],
        `${pass} run: an undeclared-input suite must never be written to the cache`,
      );
    }

    // ...and the guard is targeted, not a blanket kill: the plain fake suite
    // beside it, which names no live surface, is still cached.
    run(fx10, ["--only", "scripts/tests/test_fake_alone.mjs"]);
    const warm = run(fx10, ["--only", "scripts/tests/test_fake_alone.mjs"]);
    assert.match(warm.stdout, /CACHED green scripts\/tests\/test_fake_alone\.mjs \(closure unchanged\)/, warm.stdout);
  } finally {
    cleanup(fx10);
  }
});

console.log(`\ntest_verify_cache: ${passed} passed, ${failed} failed`);
if (failed) process.exit(1);
