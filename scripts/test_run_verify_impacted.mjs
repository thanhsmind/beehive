#!/usr/bin/env node
// test_run_verify_impacted.mjs — proves run_verify.mjs's impacted-file
// selection (cov-2, ci-owned-verify CONTEXT.md D4): `--impacted <files>`
// (repeatable/comma) and `--impacted-from-git` map changed files through
// the impact registry (scripts/impact_registry.mjs) to an EXACT suite
// selection, reusing the shared execution tail the `--only` scoped run
// already uses. Loud `IMPACTED RUN` banner, unmapped-file listing, a loud
// zero-impacted pass naming CI, and a typed `--impacted` vs `--only`
// mutual-exclusion refusal are all exercised. CRITICAL (P1-4):
// --impacted-from-git must count a staged-but-uncommitted change (the
// exact state `bee worktree merge` gates in) — proven against a disposable
// real git fixture repo, never the live project tree.
//
// Also folds in (test-economy gtp-3, groom-test-prune): the standalone
// `--only`/`BEE_VERIFY_ONLY` scoped-include-filter suite and the loud-skip
// (CANARY_SKIP marker) suite — both exercise the same `run_verify.mjs`
// surface and the same spawn+banner oracle this file already uses, so they
// live here as their own sections instead of separate files.
//
// Every spawned real run against THIS repo is scoped to exactly one fast
// suite (scripts/test_release_tuple.mjs) — the whole file must finish well
// under a minute. NEVER spawn an unfiltered full run here.

import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");
const RUN_VERIFY = path.join(REPO_ROOT, "scripts", "run_verify.mjs");
const IMPACT_REGISTRY_SRC = path.join(REPO_ROOT, "scripts", "impact_registry.mjs");
const CANARY_SCRIPT = path.join(REPO_ROOT, "scripts", "canary_codex.mjs");

const { filterSuitesByLabels, filterSuitesByOnly, SKIP_MARKER_RE, skipNote, runOne, SUITES } = await import(
  pathToFileURL(RUN_VERIFY).href
);

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

function runVerify(args, env = {}, opts = {}) {
  return spawnSync(process.execPath, [RUN_VERIFY, ...args], {
    cwd: opts.cwd ?? REPO_ROOT,
    encoding: "utf8",
    env: { ...process.env, ...env },
    timeout: 30000,
  });
}

// ── unit-level: filterSuitesByLabels is exact, not substring ──────────────
await check("filterSuitesByLabels selects only an exact suiteLabel match", () => {
  const target = SUITES.find((entry) => entry[0] === "scripts/test_release_tuple.mjs");
  assert.ok(target, "fixture assumption: scripts/test_release_tuple.mjs must be a discovered suite");
  const result = filterSuitesByLabels(SUITES, new Set(["scripts/test_release_tuple.mjs"]));
  assert.equal(result.length, 1, `expected exactly 1 exact match, got ${result.length}`);
  assert.equal(result[0][0], "scripts/test_release_tuple.mjs");
});

await check("filterSuitesByLabels returns empty for an empty label set (no accidental full-pool fallback)", () => {
  const result = filterSuitesByLabels(SUITES, new Set());
  assert.equal(result.length, 0);
});

// ── (a) mapped-file / self-selecting-suite: --impacted <own file> selects exactly itself ──
await check("--impacted scripts/test_release_tuple.mjs self-selects exactly that suite (registry closure includes its own entry), with a correct IMPACTED banner", () => {
  const r = runVerify(["--impacted", "scripts/test_release_tuple.mjs"]);
  assert.equal(r.status, 0, `expected exit 0, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
  assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/test_release_tuple\.mjs/, `expected the single suite's own PASS line:\n${r.stdout}`);
  assert.match(r.stdout, /run_verify: 1 suite\(s\)/, `expected exactly 1 suite to have run, not the full pool:\n${r.stdout}`);
  const expectedBanner = "IMPACTED RUN: 1 suite(s) from 1 changed file(s)";
  const occurrences = r.stdout.split(expectedBanner).length - 1;
  assert.equal(occurrences, 2, `expected the IMPACTED banner exactly twice (once before results, once in the summary):\n${r.stdout}`);
});

// ── comma-separated / repeatable --impacted, plus unmapped listing ─────────
await check("comma-separated --impacted mixes a mapped and an unmapped file: mapped one runs, unmapped one is listed loudly on stderr, never silently dropped", () => {
  const r = runVerify(["--impacted", "scripts/test_release_tuple.mjs,docs/does-not-exist-nor-map-to-anything.md"]);
  assert.equal(r.status, 0, `expected exit 0, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
  assert.match(r.stdout, /IMPACTED RUN: 1 suite\(s\) from 2 changed file\(s\)/, `expected 1 suite from 2 changed files:\n${r.stdout}`);
  assert.match(
    r.stderr,
    /UNMAPPED: docs\/does-not-exist-nor-map-to-anything\.md \(no known suite relates to this file — full verify still covers it\)/,
    `expected a loud UNMAPPED note on stderr:\n${r.stderr}`,
  );
});

await check("repeatable --impacted (two separate flags) unions into one changed-file set", () => {
  const r = runVerify(["--impacted", "scripts/test_release_tuple.mjs", "--impacted", "docs/does-not-exist-nor-map-to-anything.md"]);
  assert.equal(r.status, 0, `expected exit 0, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
  assert.match(r.stdout, /IMPACTED RUN: 1 suite\(s\) from 2 changed file\(s\)/, `expected repeatable --impacted flags to union:\n${r.stdout}`);
});

// ── (d) zero-impacted: loud pass naming CI, exit 0, no suites run ──────────
await check("--impacted with only an unmapped file is a loud zero-impacted pass naming CI, exit 0, zero suites run", () => {
  const r = runVerify(["--impacted", "docs/does-not-exist-nor-map-to-anything.md"]);
  assert.equal(r.status, 0, `expected exit 0 (zero impacted is a PASS, not a refusal), got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
  assert.match(
    r.stdout,
    /IMPACTED RUN: 0 suite\(s\) from 1 changed file\(s\) — full verify delegated to CI/,
    `expected the loud zero-impacted CI message:\n${r.stdout}`,
  );
  assert.doesNotMatch(r.stdout, /PASS\s+\d+ms/, `zero impacted must run no suites at all:\n${r.stdout}`);
  assert.match(r.stderr, /UNMAPPED: docs\/does-not-exist-nor-map-to-anything\.md/, `expected the unmapped file still listed loudly:\n${r.stderr}`);
});

// ── (e) mutual exclusion: --impacted + --only is a typed refusal ───────────
await check("--impacted combined with --only is a typed refusal, exit 1, before any suite runs", () => {
  const r = runVerify(["--impacted", "scripts/test_release_tuple.mjs", "--only", "test_release_tuple"]);
  assert.equal(r.status, 1, `expected exit 1, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
  assert.match(
    r.stderr,
    /--impacted\/--impacted-from-git and --only\/BEE_VERIFY_ONLY are mutually exclusive selection modes/,
    `expected the typed mutual-exclusion refusal on stderr:\n${r.stderr}`,
  );
  assert.equal(r.stdout, "", "a mutual-exclusion refusal must exit before any suite runs or banner prints");
});

await check("--impacted-from-git combined with BEE_VERIFY_ONLY env is also a typed refusal", () => {
  const r = runVerify(["--impacted-from-git"], { BEE_VERIFY_ONLY: "test_release_tuple" });
  assert.equal(r.status, 1, `expected exit 1, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
  assert.match(r.stderr, /mutually exclusive selection modes/, `expected the refusal even via the env var form:\n${r.stderr}`);
});

// ── (g) impacted-level1 D1: --level 1 selection ─────────────────────────────

await check("--level 1 without --impacted/--impacted-from-git is a typed refusal, exit 1, before any suite runs", () => {
  const r = runVerify(["--level", "1"]);
  assert.equal(r.status, 1, `expected exit 1, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
  assert.match(
    r.stderr,
    /--level is only valid alongside --impacted\/--impacted-from-git/,
    `expected a typed refusal naming the reason on stderr:\n${r.stderr}`,
  );
  assert.equal(r.stdout, "", "a --level refusal must exit before any suite runs or banner prints");
});

await check("--impacted with --level 2 (unrecognized value) is a typed refusal, exit 1", () => {
  const r = runVerify(["--impacted", "scripts/test_release_tuple.mjs", "--level", "2"]);
  assert.equal(r.status, 1, `expected exit 1, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
  assert.match(r.stderr, /not a recognized level/, `expected a typed refusal on stderr:\n${r.stderr}`);
});

// A disposable, non-git fixture repo (same "copy today's implementation
// live, run a real spawned process" idiom as the P1-4 fixture below, minus
// the git plumbing --level 1 doesn't need): three suite files establish a
// controlled direct-vs-transitive graph so these assertions never depend on
// the live repo's own suite mix (which changes over time) and, critically,
// never spawn more than two trivial, near-instant suites — the file-header
// rule ("NEVER spawn an unfiltered full run here") stays honored even though
// --level 1's whole point is a hub-file comparison.
//
//   test_direct_suite.mjs      -> inner_target.mjs                 (DIRECT)
//   test_transitive_suite.mjs  -> spawnee.mjs                      (DIRECT,
//                                  via a spawnSync(...) call that is present
//                                  in source for the regex scanner but never
//                                  actually executed — the call sits behind
//                                  an always-false guard)
//   spawnee.mjs                -> inner_target.mjs, deep_target.mjs (its own
//                                  direct imports; NOT itself a discovered
//                                  suite, so these become TRANSITIVE for
//                                  test_transitive_suite.mjs, mirroring how
//                                  bee.mjs's own imports go transitive for
//                                  its spawners)
//
// So: inner_target.mjs is direct for 1 suite, all for 2 (level 1 strictly
// narrower). deep_target.mjs is direct for 0 suites, all for 1 (the
// direct=0/transitive>0 loud-note case).
function buildLevelFixture() {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "impacted-level1-fixture-"));
  const scriptsDir = path.join(fixture, "scripts");
  fs.mkdirSync(scriptsDir, { recursive: true });
  fs.copyFileSync(RUN_VERIFY, path.join(scriptsDir, "run_verify.mjs"));
  fs.copyFileSync(IMPACT_REGISTRY_SRC, path.join(scriptsDir, "impact_registry.mjs"));
  fs.writeFileSync(path.join(scriptsDir, "inner_target.mjs"), "export const X = 1;\n");
  fs.writeFileSync(path.join(scriptsDir, "deep_target.mjs"), "export const Y = 2;\n");
  fs.writeFileSync(
    path.join(scriptsDir, "spawnee.mjs"),
    "import \"./inner_target.mjs\";\nimport \"./deep_target.mjs\";\nprocess.exit(0);\n",
  );
  fs.writeFileSync(
    path.join(scriptsDir, "test_direct_suite.mjs"),
    "import \"./inner_target.mjs\";\nprocess.exit(0);\n",
  );
  fs.writeFileSync(
    path.join(scriptsDir, "test_transitive_suite.mjs"),
    [
      "import path from \"node:path\";",
      "import { fileURLToPath } from \"node:url\";",
      "import { spawnSync } from \"node:child_process\";",
      "const __dirname = path.dirname(fileURLToPath(import.meta.url));",
      "// Never actually executed — present only so the registry's regex-level",
      "// spawn-argv scanner picks up the spawnee.mjs edge as DIRECT for this",
      "// suite, exactly as a real conditional spawn would source-scan.",
      "if (process.env.IMPACTED_LEVEL1_FIXTURE_NEVER_SET) {",
      "  spawnSync(process.execPath, [path.join(__dirname, \"spawnee.mjs\")]);",
      "}",
      "process.exit(0);",
      "",
    ].join("\n"),
  );
  return { fixture, scriptsDir, runVerifyPath: path.join(scriptsDir, "run_verify.mjs") };
}

function runFixtureVerify(runVerifyPath, cwd, args, envOverride = {}) {
  return spawnSync(process.execPath, [runVerifyPath, ...args], {
    cwd,
    encoding: "utf8",
    timeout: 30000,
    env: { ...process.env, ...envOverride },
  });
}

await check("--impacted <hub file> --level 1 selects strictly fewer suites than the default (transitive) impacted run, with the level-1 banner", () => {
  const { fixture, runVerifyPath } = buildLevelFixture();
  try {
    const full = runFixtureVerify(runVerifyPath, fixture, ["--impacted", "scripts/inner_target.mjs"]);
    const level1 = runFixtureVerify(runVerifyPath, fixture, ["--impacted", "scripts/inner_target.mjs", "--level", "1"]);
    assert.equal(full.status, 0, `expected exit 0 for the default run, got ${full.status}; stdout:\n${full.stdout}\nstderr:\n${full.stderr}`);
    assert.equal(level1.status, 0, `expected exit 0 for the level-1 run, got ${level1.status}; stdout:\n${level1.stdout}\nstderr:\n${level1.stderr}`);
    assert.match(full.stdout, /IMPACTED RUN: 2 suite\(s\) from 1 changed file\(s\)/, `expected both fixture suites (direct + transitive) in the default run:\n${full.stdout}`);
    assert.match(level1.stdout, /IMPACTED RUN \(level 1\): 1 direct suite\(s\) from 1 changed file\(s\)/, `expected exactly the direct fixture suite at level 1:\n${level1.stdout}`);
    assert.match(level1.stdout, /PASS\s+\d+ms\s+scripts\/test_direct_suite\.mjs/, `expected only the direct suite to actually run:\n${level1.stdout}`);
    assert.doesNotMatch(level1.stdout, /test_transitive_suite/, `the transitive-only suite must NOT run at level 1:\n${level1.stdout}`);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

await check("--impacted <transitive-only file> --level 1: zero direct suites but nonzero transitive is a loud deferred-count note, exit 0, no suites run", () => {
  const { fixture, runVerifyPath } = buildLevelFixture();
  try {
    const r = runFixtureVerify(runVerifyPath, fixture, ["--impacted", "scripts/deep_target.mjs", "--level", "1"]);
    assert.equal(r.status, 0, `expected exit 0 (a loud deferred note is a pass, not a refusal), got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(
      r.stdout,
      /IMPACTED RUN \(level 1\): 0 direct suite\(s\) from 1 changed file\(s\) — 1 suite\(s\) reachable only transitively, deferred to full verify\/CI/,
      `expected the loud direct=0\/transitive>0 deferred note:\n${r.stdout}`,
    );
    assert.doesNotMatch(r.stdout, /PASS\s+\d+ms/, `zero direct selection must run no suites at all:\n${r.stdout}`);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

// ── (f) CRITICAL P1-4: --impacted-from-git counts a STAGED, uncommitted change ──
// Built against a disposable real git fixture repo (never the live project
// tree): one commit on `main`, then a mapped suite file is edited and
// `git add`ed (staged, NOT committed) — the exact state `bee worktree
// merge` gates its semantic-conflict check in. run_verify.mjs +
// impact_registry.mjs are copied in live (not the committed project
// version) so this proves today's implementation, not a stale checkout.
await check("--impacted-from-git counts a staged-but-uncommitted change in a fixture repo (P1-4)", () => {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "cov2-git-fixture-"));
  try {
    const scriptsDir = path.join(fixture, "scripts");
    fs.mkdirSync(scriptsDir, { recursive: true });
    fs.copyFileSync(RUN_VERIFY, path.join(scriptsDir, "run_verify.mjs"));
    fs.copyFileSync(IMPACT_REGISTRY_SRC, path.join(scriptsDir, "impact_registry.mjs"));
    const suitePath = path.join(scriptsDir, "test_fixture_suite.mjs");
    fs.writeFileSync(suitePath, "process.exit(0);\n");

    const git = (args) =>
      spawnSync("git", args, {
        cwd: fixture,
        encoding: "utf8",
        env: { ...process.env, GIT_CONFIG_NOSYSTEM: "1" },
      });

    assert.equal(git(["init", "-q"]).status, 0, "git init failed");
    assert.equal(git(["config", "user.email", "cov2-fixture@example.com"]).status, 0);
    assert.equal(git(["config", "user.name", "cov2 fixture"]).status, 0);
    assert.equal(git(["config", "commit.gpgsign", "false"]).status, 0);
    assert.equal(git(["add", "-A"]).status, 0);
    const commit = git(["commit", "-q", "-m", "init"]);
    assert.equal(commit.status, 0, `initial commit failed: ${commit.stderr}`);
    assert.equal(git(["branch", "-m", "main"]).status, 0, "renaming default branch to main failed");

    // Edit the mapped suite file and stage it — NO commit. This is the
    // staged-but-uncommitted state the union's status-porcelain half must
    // catch even when the committed-diff-vs-merge-base half sees nothing
    // (merge-base HEAD main === HEAD here, so that half contributes zero).
    fs.writeFileSync(suitePath, "// staged edit\nprocess.exit(0);\n");
    assert.equal(git(["add", "scripts/test_fixture_suite.mjs"]).status, 0);

    const statusCheck = git(["status", "--porcelain"]);
    assert.match(statusCheck.stdout, /^M {2}scripts\/test_fixture_suite\.mjs$/m, `fixture setup sanity: expected a staged-only M line:\n${statusCheck.stdout}`);

    const r = spawnSync(process.execPath, [path.join(scriptsDir, "run_verify.mjs"), "--impacted-from-git"], {
      cwd: fixture,
      encoding: "utf8",
      timeout: 30000,
    });
    assert.equal(r.status, 0, `expected exit 0, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(
      r.stdout,
      /IMPACTED RUN: 1 suite\(s\) from 1 changed file\(s\)/,
      `expected the staged-only change to be picked up as exactly 1 impacted file/suite:\n${r.stdout}`,
    );
    assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/test_fixture_suite\.mjs/, `expected the fixture suite's own PASS line:\n${r.stdout}`);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

// ── test-economy D6: impacted cap — transitive tail delegated to CI ───────
// A disposable fixture repo (same live-copy idiom as buildLevelFixture
// above): `lib.mjs` is reached TRANSITIVELY (via `mid.mjs`) by HUB_COUNT
// suites (test_hub1..test_hubN) and DIRECTLY by exactly 1 suite
// (test_direct1.mjs). Two more suites (test_narrow1/2.mjs) import
// `narrow.mjs` DIRECTLY, so it maps to exactly those 2 suites regardless of
// HUB_COUNT.
//
// The copied run_verify.mjs also carries its own hardcoded EXTRA_SUITES
// entries (release_manifest.mjs, okf_migrate.mjs, etc.) unconditionally,
// even though none of those files exist in this fixture — they inflate the
// discovered SUITES total without ever being reachable from lib.mjs/
// narrow.mjs (so they're never selected to actually run, just counted).
// Rather than hardcode that incidental denominator, each test below
// discovers the fixture's OWN total suite count dynamically (fixtureTotal)
// and picks HUB_COUNT generously (20) so lib.mjs's ratio clears the 0.30
// default cap with margin even if that unrelated extra-suite count grows.
const HUB_COUNT = 20;

function buildCapFixture({ failDirect = false } = {}) {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "impacted-cap-fixture-"));
  const scriptsDir = path.join(fixture, "scripts");
  fs.mkdirSync(scriptsDir, { recursive: true });
  fs.copyFileSync(RUN_VERIFY, path.join(scriptsDir, "run_verify.mjs"));
  fs.copyFileSync(IMPACT_REGISTRY_SRC, path.join(scriptsDir, "impact_registry.mjs"));
  fs.writeFileSync(path.join(scriptsDir, "lib.mjs"), "export const X = 1;\n");
  fs.writeFileSync(path.join(scriptsDir, "mid.mjs"), "import \"./lib.mjs\";\nexport const Y = 1;\n");
  fs.writeFileSync(path.join(scriptsDir, "narrow.mjs"), "export const Z = 1;\n");
  for (let i = 1; i <= HUB_COUNT; i += 1) {
    fs.writeFileSync(
      path.join(scriptsDir, `test_hub${i}.mjs`),
      "import \"./mid.mjs\";\nprocess.exit(0);\n",
    );
  }
  fs.writeFileSync(
    path.join(scriptsDir, "test_direct1.mjs"),
    `import "./lib.mjs";\nprocess.exit(${failDirect ? 1 : 0});\n`,
  );
  fs.writeFileSync(path.join(scriptsDir, "test_narrow1.mjs"), "import \"./narrow.mjs\";\nprocess.exit(0);\n");
  fs.writeFileSync(path.join(scriptsDir, "test_narrow2.mjs"), "import \"./narrow.mjs\";\nprocess.exit(0);\n");
  return { fixture, scriptsDir, runVerifyPath: path.join(scriptsDir, "run_verify.mjs") };
}

function writeBeeConfig(fixture, config) {
  const beeDir = path.join(fixture, ".bee");
  fs.mkdirSync(beeDir, { recursive: true });
  fs.writeFileSync(path.join(beeDir, "config.json"), JSON.stringify(config));
}

// The fixture's own copy of run_verify.mjs exports SUITES; import it (a
// plain, side-effect-free ESM import — main() only runs when the file is
// executed directly, per its own isMain guard) to read the real total this
// fixture discovers, EXTRA_SUITES padding included.
async function fixtureTotalSuites(runVerifyPath) {
  const mod = await import(pathToFileURL(runVerifyPath).href);
  return mod.SUITES.length;
}

await check("test-economy D6: below cap (narrow.mjs, 2 suites) runs the full transitive selection unchanged, no cap banner", () => {
  const { fixture, runVerifyPath } = buildCapFixture();
  try {
    const r = runFixtureVerify(runVerifyPath, fixture, ["--impacted", "scripts/narrow.mjs"]);
    assert.equal(r.status, 0, `expected exit 0, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(r.stdout, /IMPACTED RUN: 2 suite\(s\) from 1 changed file\(s\)/, `expected both narrow suites selected:\n${r.stdout}`);
    assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/test_narrow1\.mjs/, r.stdout);
    assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/test_narrow2\.mjs/, r.stdout);
    assert.doesNotMatch(r.stdout, /exceeds cap/, `below-cap run must never print the cap banner:\n${r.stdout}`);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

await check("test-economy D6: config missing -> default cap 0.30 -> over-cap (lib.mjs) falls back to level-1, banner printed, direct suite still runs", async () => {
  const { fixture, runVerifyPath } = buildCapFixture();
  try {
    const total = await fixtureTotalSuites(runVerifyPath);
    const mapped = HUB_COUNT + 1;
    assert.ok(mapped / total > 0.3, `fixture assumption: HUB_COUNT=${HUB_COUNT} must clear the 0.30 cap against this fixture's total=${total} (mapped=${mapped})`);
    const r = runFixtureVerify(runVerifyPath, fixture, ["--impacted", "scripts/lib.mjs"], { PATH: "/nonexistent-bin" });
    assert.equal(r.status, 0, `expected exit 0 (the one direct suite passes), got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(
      r.stdout,
      new RegExp(`impacted ${mapped}/${total} exceeds cap \\(0\\.3\\) — transitive tail delegated to CI`),
      `expected the D6 cap banner naming the pre-cap total and cap:\n${r.stdout}`,
    );
    assert.match(r.stdout, /IMPACTED RUN \(level 1\): 1 direct suite\(s\) from 1 changed file\(s\)/, `expected the level-1 fallback banner:\n${r.stdout}`);
    assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/test_direct1\.mjs/, `expected the direct suite to actually run:\n${r.stdout}`);
    assert.doesNotMatch(r.stdout, /test_hub/, `the pure-transitive hub suites must NOT run once capped:\n${r.stdout}`);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

await check("test-economy D6: a red suite inside the capped (level-1) selection still exits red — cap never silences a real failure", () => {
  const { fixture, runVerifyPath } = buildCapFixture({ failDirect: true });
  try {
    const r = runFixtureVerify(runVerifyPath, fixture, ["--impacted", "scripts/lib.mjs"], { PATH: "/nonexistent-bin" });
    assert.equal(r.status, 1, `expected exit 1 (the capped selection's own suite failed), got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(r.stdout, /exceeds cap/, `expected the cap banner even on a red capped run:\n${r.stdout}`);
    assert.match(r.stdout, /FAIL\s+\d+ms\s+scripts\/test_direct1\.mjs/, r.stdout);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

await check("test-economy D6: verify_impacted_cap = 1 disables the cutoff entirely — full transitive selection runs even over the default cap", () => {
  const { fixture, runVerifyPath } = buildCapFixture();
  try {
    writeBeeConfig(fixture, { verify_impacted_cap: 1 });
    const r = runFixtureVerify(runVerifyPath, fixture, ["--impacted", "scripts/lib.mjs"]);
    assert.equal(r.status, 0, `expected exit 0, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(r.stdout, new RegExp(`IMPACTED RUN: ${HUB_COUNT + 1} suite\\(s\\) from 1 changed file\\(s\\)`), `expected the uncapped full transitive selection:\n${r.stdout}`);
    assert.doesNotMatch(r.stdout, /exceeds cap/, `cap=1 must fully disable the cutoff banner:\n${r.stdout}`);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

await check("test-economy D6: config present but verify_impacted_cap key missing -> falls back to default 0.30 (same as no config)", async () => {
  const { fixture, runVerifyPath } = buildCapFixture();
  try {
    const total = await fixtureTotalSuites(runVerifyPath);
    const mapped = HUB_COUNT + 1;
    writeBeeConfig(fixture, { some_other_key: true });
    const r = runFixtureVerify(runVerifyPath, fixture, ["--impacted", "scripts/lib.mjs"], { PATH: "/nonexistent-bin" });
    assert.equal(r.status, 0, `expected exit 0, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(r.stdout, new RegExp(`impacted ${mapped}/${total} exceeds cap \\(0\\.3\\)`), `expected the default 0.30 cap to apply when the key is absent:\n${r.stdout}`);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

await check("test-economy D6: malformed .bee/config.json -> parse failure falls back to default 0.30 (fail-open, never a crash)", async () => {
  const { fixture, runVerifyPath } = buildCapFixture();
  try {
    const total = await fixtureTotalSuites(runVerifyPath);
    const mapped = HUB_COUNT + 1;
    const beeDir = path.join(fixture, ".bee");
    fs.mkdirSync(beeDir, { recursive: true });
    fs.writeFileSync(path.join(beeDir, "config.json"), "{ not valid json");
    const r = runFixtureVerify(runVerifyPath, fixture, ["--impacted", "scripts/lib.mjs"], { PATH: "/nonexistent-bin" });
    assert.equal(r.status, 0, `expected exit 0, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(r.stdout, new RegExp(`impacted ${mapped}/${total} exceeds cap \\(0\\.3\\)`), `expected the default 0.30 cap on parse failure:\n${r.stdout}`);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

await check("test-economy D6: --level 1 is a no-op for the cap — no banner, no cut, even with an over-cap changed file and a low cap configured", () => {
  const { fixture, runVerifyPath } = buildCapFixture();
  try {
    writeBeeConfig(fixture, { verify_impacted_cap: 0.01 });
    const r = runFixtureVerify(runVerifyPath, fixture, ["--impacted", "scripts/lib.mjs", "--level", "1"]);
    assert.equal(r.status, 0, `expected exit 0, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(r.stdout, /IMPACTED RUN \(level 1\): 1 direct suite\(s\) from 1 changed file\(s\)/, `expected the plain level-1 selection, untouched by the cap:\n${r.stdout}`);
    assert.doesNotMatch(r.stdout, /exceeds cap/, `--level 1 must never print the cap banner (must-have: no-op, no banner):\n${r.stdout}`);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

await check("test-economy D6: gh missing/erroring during a capped run prints a one-line note but never changes the exit code", () => {
  const { fixture, runVerifyPath } = buildCapFixture();
  try {
    const r = runFixtureVerify(runVerifyPath, fixture, ["--impacted", "scripts/lib.mjs"], { PATH: "/nonexistent-bin" });
    assert.equal(r.status, 0, `expected exit 0 unaffected by gh being unavailable, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
    assert.match(
      r.stdout,
      /gh workflow run CI.*did not succeed/,
      `expected a one-line gh-unavailable note:\n${r.stdout}`,
    );
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

// ── folded from test_run_verify_only.mjs: scoped `--only`/BEE_VERIFY_ONLY
// include filter (vs-1). A repeatable/comma-separated `--only <token>` CLI
// flag plus a BEE_VERIFY_ONLY env var (CLI wins when both are set), matching
// a runnable by case-insensitive substring against its repo-relative path OR
// its display label, applied BEFORE BEE_VERIFY_EXCLUDE so exclude can still
// narrow (never widen) the included set. Zero matches is a typed refusal,
// exit 1. Every scoped run prints the loud "SCOPED RUN (--only): N of M
// runnables selected" banner before results and again in the summary; an
// unscoped run prints neither. ──────────────────────────────────────────────

// ── (d) unit-level: no-flag no-op is a true no-op ───────────────────────────
await check("filterSuitesByOnly(suites, []) returns the SAME array reference (byte-identical no-op)", () => {
  const result = filterSuitesByOnly(SUITES, []);
  assert.equal(result, SUITES, "must be reference-equal, not just element-equal — a copy would not prove the no-op contract");
});

await check("SUITES has more than one runnable (sanity: the M in N of M banners must be > 1)", () => {
  assert.ok(SUITES.length > 1, `expected multiple suites, got ${SUITES.length}`);
});

// ── (a) --only test_release_tuple runs exactly that subset, banner has correct N/M ──
await check("--only test_release_tuple selects exactly that suite and passes, with a correct banner", () => {
  const r = runVerify(["--only", "test_release_tuple"]);
  assert.equal(r.status, 0, `expected exit 0, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
  assert.match(r.stdout, /PASS\s+\d+ms\s+scripts\/test_release_tuple\.mjs/, `expected the single suite's own PASS line:\n${r.stdout}`);
  assert.match(
    r.stdout,
    /run_verify: 1 suite\(s\)/,
    `expected exactly 1 suite to have run, not the full pool:\n${r.stdout}`,
  );
  const expectedBanner = `SCOPED RUN (--only): 1 of ${SUITES.length} runnables selected — full verify still required before close`;
  const occurrences = r.stdout.split(expectedBanner).length - 1;
  assert.equal(
    occurrences,
    2,
    `expected the banner to appear exactly twice (once before results, once in the summary) with N=1, M=${SUITES.length}:\n${r.stdout}`,
  );
});

// ── (b) zero-match token → typed refusal, exit 1 ────────────────────────────
await check("--only with a token matching zero suites is a typed refusal, exit 1", () => {
  const r = runVerify(["--only", "zzz-no-such-suite-token"]);
  assert.equal(r.status, 1, `expected exit 1, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
  assert.match(
    r.stderr,
    /matched zero suites — refusing a silent trivial-green run/,
    `expected the typed refusal message on stderr:\n${r.stderr}`,
  );
  assert.equal(r.stdout, "", "a zero-match refusal must exit before any suite runs — no PASS/FAIL lines, no banner");
});

// ── (c) include-before-exclude: BEE_VERIFY_EXCLUDE can narrow the included set ──
await check("BEE_VERIFY_EXCLUDE narrows an --only selection down to zero, and still refuses", () => {
  const r = runVerify(["--only", "test_release_tuple"], {
    BEE_VERIFY_EXCLUDE: "scripts/test_release_tuple.mjs",
  });
  assert.equal(r.status, 1, `expected exit 1 once exclude drops the only included suite, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
  assert.match(
    r.stderr,
    /BEE_VERIFY_EXCLUDE="scripts\/test_release_tuple\.mjs" excluded every remaining suite/,
    `expected the exclude-specific refusal, not the --only refusal:\n${r.stderr}`,
  );
});

// ── (e) BEE_VERIFY_ONLY env alone works; CLI --only overrides env when both set ──
await check("BEE_VERIFY_ONLY env var alone selects the scoped subset", () => {
  const r = runVerify([], { BEE_VERIFY_ONLY: "test_release_tuple" });
  assert.equal(r.status, 0, `expected exit 0, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
  assert.match(r.stdout, /run_verify: 1 suite\(s\)/, `expected exactly 1 suite via env var alone:\n${r.stdout}`);
  assert.match(r.stdout, /SCOPED RUN \(--only\)/, `expected the scoped banner even when only the env var is set:\n${r.stdout}`);
});

await check("CLI --only overrides BEE_VERIFY_ONLY when both are set (env token would refuse, CLI token passes)", () => {
  const r = runVerify(["--only", "test_release_tuple"], { BEE_VERIFY_ONLY: "zzz-no-such-suite-token" });
  assert.equal(
    r.status,
    0,
    `expected exit 0 — CLI must win over env, so the env's zero-match token must be ignored entirely; got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`,
  );
  assert.match(r.stdout, /run_verify: 1 suite\(s\)/, `expected exactly 1 suite (the CLI token's match):\n${r.stdout}`);
});

// ── no flag / no env: byte-identical current behavior, no banner ───────────
await check("with no --only and no BEE_VERIFY_ONLY, filterSuitesByOnly is never given a chance to fire — no banner text appears even on a scoped-looking exclude run", () => {
  // Full-pool spawn is out of scope for this fast-only suite; instead prove
  // the absent-banner contract at the unit level (matches (d) above) plus a
  // real spawn that is ALSO scoped (via BEE_VERIFY_ROOT_FILTER, not --only)
  // so this stays well under a minute while still exercising a real process
  // that took the "isScopedRun === false" branch.
  const r = runVerify([], { BEE_VERIFY_ROOT_FILTER: "scripts", BEE_VERIFY_EXCLUDE: SUITES.filter((s) => s[0] !== "scripts/test_release_tuple.mjs").map((s) => s[0]).join(",") });
  assert.equal(r.status, 0, `expected exit 0, got ${r.status}; stdout:\n${r.stdout}\nstderr:\n${r.stderr}`);
  assert.doesNotMatch(r.stdout, /SCOPED RUN \(--only\)/, `no --only/BEE_VERIFY_ONLY was set — the scoped banner must never appear:\n${r.stdout}`);
});

// ── folded from test_run_verify_skip_marker.mjs: loud-skip surfacing
// (hardening-8). A suite that self-skips (exits 0 having done nothing, e.g.
// scripts/canary_codex.mjs's no-codex-binary path) prints a CANARY_SKIP
// marker line, and run_verify's own summary loop turns that into a visible
// "[SKIPPED: ...]" note beside PASS instead of silently folding it into an
// ordinary green result. Exit codes are never affected by this — only the
// printed line changes. Two things are proven, independently: (1) the
// general mechanism (SKIP_MARKER_RE / skipNote) against a synthetic suite
// spawned through run_verify's OWN runOne(); (2) the CONCRETE instance:
// scripts/canary_codex.mjs's default (P1-P5) and --probe no-codex-binary
// paths actually print the marker and still exit 0, run with a PATH that
// excludes codex so this is deterministic regardless of whether this
// machine happens to have a real codex binary installed. ───────────────────

// A PATH with no codex binary reachable — node's own dir plus the standard
// bins, deliberately omitting the real ~/.local/bin (mirrors
// test_installers_e2e.mjs's BASE_PATH discipline: this must be
// deterministic regardless of what happens to be installed on the machine
// actually running this suite).
const NO_CODEX_PATH = [path.dirname(process.execPath), "/usr/bin", "/bin", "/usr/local/bin"]
  .filter((dir, i, all) => all.indexOf(dir) === i)
  .join(path.delimiter);

function runCanary(args) {
  return spawnSync(process.execPath, [CANARY_SCRIPT, ...args], {
    encoding: "utf8",
    env: { ...process.env, PATH: NO_CODEX_PATH },
    timeout: 15000,
  });
}

// ── skipNote() unit behavior ─────────────────────────────────────────────
await check("skipNote returns null for ordinary stdout with no marker", () => {
  assert.equal(skipNote("all good\nPASS foo\n"), null);
});

await check("skipNote extracts the reason text from a CANARY_SKIP marker line", () => {
  assert.equal(skipNote("some preamble\nCANARY_SKIP reason=no-codex-binary mode=default\ntrailer\n"), "reason=no-codex-binary mode=default");
});

await check("SKIP_MARKER_RE only matches a line that STARTS WITH CANARY_SKIP (anchored, not a bare substring)", () => {
  assert.equal(SKIP_MARKER_RE.test("this text mentions CANARY_SKIP mid-sentence, not at line start"), false);
  assert.equal(SKIP_MARKER_RE.test("CANARY_SKIP reason=x"), true);
});

// ── integration: a synthetic self-skipping suite run through runOne() ────
await check("a synthetic suite that prints CANARY_SKIP and exits 0 is captured by runOne() with an intact exit code", async () => {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "verify-skip-marker-"));
  try {
    const script = path.join(tmp, "fake_skip_suite.mjs");
    fs.writeFileSync(
      script,
      "console.log('CANARY_SKIP reason=synthetic-test-fixture');\nconsole.log('fake suite: skipped');\nprocess.exit(0);\n",
    );
    // runOne() spawns `node <script>` relative to REPO_ROOT as cwd — pass an
    // absolute path so cwd is irrelevant here. AWAITED before the `finally`
    // cleanup below runs — a fire-and-forget `return promise` inside a
    // try/finally lets `finally` delete the tmp dir (and the script file
    // inside it) before the spawned child has finished reading it, a real
    // race caught live while writing this fixture (ENOENT under load).
    const result = await runOne([script]);
    assert.equal(result.code, 0, "a self-skipping suite must still exit 0 (never a failure)");
    const note = skipNote(result.stdout);
    assert.equal(note, "reason=synthetic-test-fixture", "run_verify must be able to recover the skip reason from the suite's stdout");
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
});

// ── concrete instance: canary_codex.mjs's own no-binary skip paths ───────
await check("canary_codex.mjs default mode prints CANARY_SKIP and exits 0 when no codex binary is on PATH", () => {
  const r = runCanary([]);
  assert.equal(r.status, 0, `expected exit 0, got ${r.status}; stderr: ${r.stderr}`);
  assert.match(r.stdout, SKIP_MARKER_RE, `stdout must contain a CANARY_SKIP marker line:\n${r.stdout}`);
  assert.equal(skipNote(r.stdout), "reason=no-codex-binary mode=default");
});

await check("canary_codex.mjs --probe prints CANARY_SKIP and exits 0 when no codex binary is on PATH", () => {
  const r = runCanary(["--probe"]);
  assert.equal(r.status, 0, `expected exit 0, got ${r.status}; stderr: ${r.stderr}`);
  assert.match(r.stdout, SKIP_MARKER_RE, `stdout must contain a CANARY_SKIP marker line:\n${r.stdout}`);
  assert.equal(skipNote(r.stdout), "reason=no-codex-binary mode=probe");
});

await check("canary_codex.mjs --probe-selftest is unaffected (never touches codex, no marker expected)", () => {
  const r = runCanary(["--probe-selftest"]);
  assert.equal(r.status, 0, `probe-selftest must stay green regardless of codex on PATH; stderr: ${r.stderr}`);
  assert.equal(skipNote(r.stdout), null, "probe-selftest runs its offline invariant check, not the no-binary skip path — no marker expected");
});

// ── SUITES export/discovery untouched (must-have) ──────────────────────────
await check("SUITES export is still a plain array of discovered/extra suite entries (byte-shape untouched)", () => {
  assert.ok(Array.isArray(SUITES) && SUITES.length > 1, `expected the full discovered SUITES array, got length ${SUITES.length}`);
  assert.ok(
    SUITES.every((entry) => Array.isArray(entry) && typeof entry[0] === "string"),
    "every SUITES entry must still be an array whose first element is a string path",
  );
});

console.log(`\ntest_run_verify_impacted: ${passed} passed, ${failed} failed`);
if (failed) process.exit(1);
