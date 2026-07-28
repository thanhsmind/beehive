#!/usr/bin/env node
// test_impact_registry.mjs — proves scripts/impact_registry.mjs (cov-1,
// ci-owned-verify D3): the file→suite relatedness registry is DERIVED from
// run_verify.mjs's own SUITES discovery (never a hand list), captures all
// four documented edge types, is byte-deterministic, and its --check/--query
// verbs behave as specified.
//
// Fast: imports the module's exported functions directly instead of
// spawning a fresh `node scripts/impact_registry.mjs` subprocess for every
// case (buildRegistry() alone does the real work; the CLI wrapper around it
// is exercised separately, and only where the verb itself — --check's exit
// code, --query's stdout/stderr split — is what's under test).

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");
const REGISTRY_SCRIPT = path.join(__dirname, "..", "impact_registry.mjs");
const REGISTRY_JSON = path.join(__dirname, "..", "impact-registry.json");
const RUN_VERIFY_PATH = path.join(__dirname, "..", "run_verify.mjs");

const { buildRegistry, serializeRegistry, queryRegistry } = await import(
  pathToFileURL(REGISTRY_SCRIPT).href
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

function runCli(args) {
  return spawnSync(process.execPath, [REGISTRY_SCRIPT, ...args], {
    cwd: REPO_ROOT,
    encoding: "utf8",
    timeout: 30000,
  });
}

// ── determinism ──────────────────────────────────────────────────────────
await check("buildRegistry() run twice produces byte-identical serialized output", async () => {
  const a = serializeRegistry(await buildRegistry());
  const b = serializeRegistry(await buildRegistry());
  assert.equal(a, b, "two builds of the same tree must be byte-identical — no timestamps, no nondeterministic ordering");
});

// ── known static edge: test_config_validate.mjs -> state.mjs ──────────────
await check("a known static ESM import edge is captured: scripts/tests/test_config_validate.mjs -> .bee/bin/lib/state.mjs", async () => {
  const registry = await buildRegistry();
  const suites = registry.files[".bee/bin/lib/state.mjs"]?.all || [];
  assert.ok(
    suites.includes("scripts/tests/test_config_validate.mjs"),
    `expected scripts/tests/test_config_validate.mjs among the suites for .bee/bin/lib/state.mjs, got: ${JSON.stringify(suites)}`,
  );
});

// ── known spawn edge: test_worktree_cli.mjs spawns .bee/bin/bee.mjs ───────
await check("a known spawn-argv edge is captured: scripts/tests/test_worktree_cli.mjs -> .bee/bin/bee.mjs", async () => {
  const registry = await buildRegistry();
  const suites = registry.files[".bee/bin/bee.mjs"]?.all || [];
  assert.ok(
    suites.includes("scripts/tests/test_worktree_cli.mjs"),
    `expected scripts/tests/test_worktree_cli.mjs (spawnSync('node', [BEE_MJS, ...args])) among the suites for .bee/bin/bee.mjs, got ${suites.length} suites`,
  );
});

// ── known runModuleWorker edge: test_conformance.mjs -> bee.mjs ───────────
await check("a known runModuleWorker edge is captured: scripts/tests/test_conformance.mjs -> .bee/bin/bee.mjs", async () => {
  const registry = await buildRegistry();
  const suites = registry.files[".bee/bin/bee.mjs"]?.all || [];
  assert.ok(
    suites.includes("scripts/tests/test_conformance.mjs"),
    `expected scripts/tests/test_conformance.mjs (runModuleWorker(BEE_MJS, ...)) among the suites for .bee/bin/bee.mjs, got ${suites.length} suites`,
  );
});

// ── inherited closure: bee.mjs's own static imports flow to its spawners ──
await check("a spawned/worker-run bee.mjs inherits its own static import closure (state.mjs reachable via test_worktree_cli.mjs)", async () => {
  const registry = await buildRegistry();
  const suites = registry.files[".bee/bin/lib/state.mjs"]?.all || [];
  assert.ok(
    suites.includes("scripts/tests/test_worktree_cli.mjs"),
    `expected scripts/tests/test_worktree_cli.mjs to reach .bee/bin/lib/state.mjs THROUGH bee.mjs's own closure, got: ${JSON.stringify(suites)}`,
  );
});

// ── every suite maps to at least itself, DIRECTLY ──────────────────────────
// Iterates run_verify.mjs's own SUITES discovery rather than hardcoding one
// suite's filename — a hardcoded name goes stale the moment a suite is
// renamed/folded/pruned (as test_run_verify_only.mjs was, folded into
// test_run_verify_impacted.mjs), silently testing nothing once the file is
// gone instead of catching drift in the property itself.
await check("every discovered suite's own entry file maps to (at least) itself, and directly so", async () => {
  const registry = await buildRegistry();
  const { SUITES } = await import(pathToFileURL(RUN_VERIFY_PATH).href);
  assert.ok(SUITES.length > 0, "expected run_verify.mjs to discover at least one suite");
  for (const suiteEntry of SUITES) {
    const suitePath = suiteEntry[0];
    // Mirrors impact_registry.mjs's own (unexported) suiteLabel(): registry
    // 'all'/'direct' lists are keyed by suite LABEL, which is the entry-file
    // path plus any extra argv (e.g. EXTRA_SUITES entries like
    // ["scripts/release_manifest.mjs", "--selftest"]), not the bare path.
    const label = [suiteEntry[0], ...suiteEntry.slice(1)].join(" ");
    const entry = registry.files[suitePath];
    assert.ok(entry, `expected ${suitePath} to be a known registry entry`);
    assert.ok(
      entry.all.includes(label),
      `a suite's own file must always be in its own impacted set ('all'): ${label}`,
    );
    assert.ok(
      entry.direct.includes(label),
      `a suite's own path is direct too (must-have) — must also be in its own 'direct' set: ${label}`,
    );
  }
});

// ── edge DEPTH: direct vs transitive (impacted-level1 D1) ──────────────────
// scripts/tests/test_conformance.mjs never imports .bee/bin/lib/state.mjs itself —
// it only reaches .bee/bin/bee.mjs via runModuleWorker(BEE_MJS, ...) (a
// direct edge for test_conformance.mjs -> bee.mjs), and bee.mjs's own static
// import of state.mjs comes along for the ride. So state.mjs is reachable
// from test_conformance.mjs ONLY transitively: present in 'all', absent from
// 'direct'.
await check("edge depth: a transitive-only edge (state.mjs via bee.mjs's runModuleWorker closure) is in 'all' but absent from 'direct': scripts/tests/test_conformance.mjs -> .bee/bin/lib/state.mjs", async () => {
  const registry = await buildRegistry();
  const entry = registry.files[".bee/bin/lib/state.mjs"];
  assert.ok(entry, "expected .bee/bin/lib/state.mjs to be a known registry entry");
  assert.ok(
    entry.all.includes("scripts/tests/test_conformance.mjs"),
    `expected scripts/tests/test_conformance.mjs in 'all' (reachable transitively through bee.mjs), got: ${JSON.stringify(entry.all)}`,
  );
  assert.ok(
    !entry.direct.includes("scripts/tests/test_conformance.mjs"),
    `expected scripts/tests/test_conformance.mjs ABSENT from 'direct' (test_conformance.mjs's own source never mentions state.mjs), got: ${JSON.stringify(entry.direct)}`,
  );
});

// ── edge DEPTH: a suite that spawns bee.mjs directly (bee.mjs itself is direct) ──
await check("edge depth: bee.mjs itself is a DIRECT edge for its spawner: scripts/tests/test_worktree_cli.mjs -> .bee/bin/bee.mjs", async () => {
  const registry = await buildRegistry();
  const entry = registry.files[".bee/bin/bee.mjs"];
  assert.ok(entry, "expected .bee/bin/bee.mjs to be a known registry entry");
  assert.ok(
    entry.direct.includes("scripts/tests/test_worktree_cli.mjs"),
    `expected scripts/tests/test_worktree_cli.mjs (spawns bee.mjs directly) in 'direct' for .bee/bin/bee.mjs, got: ${JSON.stringify(entry.direct)}`,
  );
});

// ── direct is always a subset of all ────────────────────────────────────────
await check("determinism/shape: every file's 'direct' suite list is a subset of its 'all' list", async () => {
  const registry = await buildRegistry();
  for (const [file, entry] of Object.entries(registry.files)) {
    for (const suite of entry.direct) {
      assert.ok(
        entry.all.includes(suite),
        `direct-not-in-all violation for ${file}: ${suite} is in 'direct' but missing from 'all'`,
      );
    }
  }
});

// ── committed registry stays in sync with a fresh build ───────────────────
await check("the committed scripts/impact-registry.json matches a freshly-built registry (byte-identical)", async () => {
  const fresh = serializeRegistry(await buildRegistry());
  const onDisk = fs.readFileSync(REGISTRY_JSON, "utf8");
  assert.equal(
    onDisk,
    fresh,
    "the committed registry has drifted from a fresh build — run `node scripts/impact_registry.mjs --write` to regenerate",
  );
});

// ── CLI --write then --check is green ──────────────────────────────────────
await check("CLI: --write then --check exits 0 (registry matches what it just wrote)", async () => {
  const backup = fs.readFileSync(REGISTRY_JSON, "utf8");
  try {
    const w = runCli(["--write"]);
    assert.equal(w.status, 0, `--write should exit 0, got ${w.status}; stderr:\n${w.stderr}`);
    const c = runCli(["--check"]);
    assert.equal(c.status, 0, `--check should exit 0 right after --write, got ${c.status}; stderr:\n${c.stderr}`);
  } finally {
    fs.writeFileSync(REGISTRY_JSON, backup);
  }
});

// ── CLI --check catches drift ──────────────────────────────────────────────
await check("CLI: --check catches drift (corrupted registry) and names the regen command, exit 1", async () => {
  const backup = fs.readFileSync(REGISTRY_JSON, "utf8");
  try {
    fs.writeFileSync(REGISTRY_JSON, backup.replace(/\n$/, "") + "\n// drift\n");
    const c = runCli(["--check"]);
    assert.equal(c.status, 1, `expected exit 1 on drift, got ${c.status}; stdout:\n${c.stdout}\nstderr:\n${c.stderr}`);
    assert.match(
      c.stderr,
      /impact_registry\.mjs --write/,
      `expected the regen-command message on stderr:\n${c.stderr}`,
    );
  } finally {
    fs.writeFileSync(REGISTRY_JSON, backup);
  }
});

// ── CLI --check catches a missing registry file ────────────────────────────
await check("CLI: --check catches a missing registry file, exit 1", async () => {
  const backup = fs.readFileSync(REGISTRY_JSON, "utf8");
  try {
    fs.rmSync(REGISTRY_JSON);
    const c = runCli(["--check"]);
    assert.equal(c.status, 1, `expected exit 1 on missing registry, got ${c.status}`);
  } finally {
    fs.writeFileSync(REGISTRY_JSON, backup);
  }
});

// ── query: union of suites across multiple files ───────────────────────────
await check("query: union of suites for two files, sorted, newline-separated, exit 0", async () => {
  const registry = await buildRegistry();
  const { mappedSuites, unmapped } = queryRegistry(registry, [
    ".bee/bin/lib/state.mjs",
    "scripts/run_verify.mjs",
  ]);
  assert.equal(unmapped.length, 0, `expected both files mapped, got unmapped: ${JSON.stringify(unmapped)}`);
  assert.ok(mappedSuites.length > 1, `expected the union to contain multiple suites, got ${mappedSuites.length}`);
  const sorted = [...mappedSuites].sort();
  assert.deepEqual(mappedSuites, sorted, "mappedSuites must be sorted");
});

// ── query: unmapped file is noted, exit 0, never silent ────────────────────
await check("query: an unmapped file is noted loudly but still exits 0 (never silent)", async () => {
  const registry = await buildRegistry();
  const { mappedSuites, unmapped } = queryRegistry(registry, [
    "scripts/zz-definitely-not-a-real-file.mjs",
  ]);
  assert.equal(mappedSuites.length, 0);
  assert.deepEqual(unmapped, ["scripts/zz-definitely-not-a-real-file.mjs"]);
});

// ── CLI --query end-to-end: stdout has suites, stderr notes the unmapped one, exit 0 ──
await check("CLI: --query prints mapped suites on stdout and an UNMAPPED note on stderr for an unknown file, exit 0", async () => {
  const r = runCli(["--query", ".bee/bin/lib/state.mjs", "scripts/zz-definitely-not-a-real-file.mjs"]);
  assert.equal(r.status, 0, `expected exit 0 even with an unmapped file, got ${r.status}; stderr:\n${r.stderr}`);
  assert.match(r.stdout, /scripts\/tests\/test_config_validate\.mjs/, `expected a suite name on stdout:\n${r.stdout}`);
  assert.match(r.stderr, /UNMAPPED.*zz-definitely-not-a-real-file\.mjs/, `expected an UNMAPPED note on stderr:\n${r.stderr}`);
});

// ── query: --level 1 narrows to direct edges only ──────────────────────────
await check("query: level 1 is strictly narrower than the default (full/transitive) for a hub file (.bee/bin/lib/state.mjs)", async () => {
  const registry = await buildRegistry();
  const full = queryRegistry(registry, [".bee/bin/lib/state.mjs"]);
  const level1 = queryRegistry(registry, [".bee/bin/lib/state.mjs"], { level: 1 });
  assert.equal(full.unmapped.length, 0);
  assert.equal(level1.unmapped.length, 0, "a file known to the registry (has 'all' suites) is never unmapped at level 1");
  assert.ok(
    level1.mappedSuites.length < full.mappedSuites.length,
    `expected level-1 (direct only) to be strictly narrower than full/transitive for a hub file, got level1=${level1.mappedSuites.length} full=${full.mappedSuites.length}`,
  );
  for (const s of level1.mappedSuites) {
    assert.ok(full.mappedSuites.includes(s), `every level-1 suite must also appear in the full/transitive set: ${s}`);
  }
});

// ── query: a fully transitive-only file (zero direct suites) is not unmapped at level 1 ──
await check("query: a file with zero direct edges but nonzero transitive edges maps to zero suites at level 1, yet is NOT unmapped", async () => {
  const registry = await buildRegistry();
  const entry = registry.files[".bee/bin/lib/judge.mjs"];
  assert.ok(entry, "fixture assumption: .bee/bin/lib/judge.mjs must be a known registry entry");
  assert.equal(entry.direct.length, 0, "fixture assumption: .bee/bin/lib/judge.mjs must have zero direct suites (only bee.mjs imports it)");
  assert.ok(entry.all.length > 0, "fixture assumption: .bee/bin/lib/judge.mjs must be reachable transitively");
  const level1 = queryRegistry(registry, [".bee/bin/lib/judge.mjs"], { level: 1 });
  assert.equal(level1.mappedSuites.length, 0, "zero direct suites at level 1");
  assert.deepEqual(level1.unmapped, [], "a known-but-transitive-only file must NOT be reported as unmapped");
});

// ── CLI --query --level 1 end-to-end ────────────────────────────────────────
await check("CLI: --query --level 1 prints strictly fewer suites than the default query for the same hub file", async () => {
  const full = runCli(["--query", ".bee/bin/lib/state.mjs"]);
  const level1 = runCli(["--query", ".bee/bin/lib/state.mjs", "--level", "1"]);
  assert.equal(full.status, 0, `default query should exit 0, got ${full.status}; stderr:\n${full.stderr}`);
  assert.equal(level1.status, 0, `--level 1 query should exit 0, got ${level1.status}; stderr:\n${level1.stderr}`);
  const fullCount = full.stdout.trim().split("\n").filter(Boolean).length;
  const level1Count = level1.stdout.trim().split("\n").filter(Boolean).length;
  assert.ok(
    level1Count < fullCount,
    `expected --level 1 to print fewer suites than the default for a hub file, got level1=${level1Count} full=${fullCount}`,
  );
});

// ── test-economy D6 dependency: level-1 is always a subset of transitive ──
// run_verify.mjs's impacted cap (D6) falls back from a transitive selection
// to `queryRegistry(registry, changedList, { level: 1 })` over the SAME
// changed-file set. That fallback is only safe if level-1 never surfaces a
// suite absent from the transitive result — pin the subset invariant across
// a multi-file changed set (not just the single-hub-file case already
// covered above) so a future registry change can't silently widen it.
await check("test-economy D6: level-1 query over a multi-file changed set stays a subset of the transitive query for the same set", async () => {
  const registry = await buildRegistry();
  const changed = ["scripts/run_verify.mjs", ".bee/bin/lib/state.mjs"];
  const full = queryRegistry(registry, changed);
  const level1 = queryRegistry(registry, changed, { level: 1 });
  assert.ok(level1.mappedSuites.length > 0, "fixture assumption: expected at least one direct suite for this changed set");
  for (const s of level1.mappedSuites) {
    assert.ok(full.mappedSuites.includes(s), `every level-1 suite must also appear in the transitive set: ${s}`);
  }
});

// ── fx-2 (foundation-fixes): the worktree-store suite split lands both
// halves in the committed registry ─────────────────────────────────────────
await check("foundation-fixes fx-2: both split suite filenames (test_worktree_store.mjs topology half + test_worktree_store_merge.mjs merge half) are present in scripts/impact-registry.json", async () => {
  const onDisk = fs.readFileSync(REGISTRY_JSON, "utf8");
  assert.ok(
    onDisk.includes("packages/bee/tests/test_worktree_store.mjs"),
    "expected the topology-half suite filename (packages/bee/tests/test_worktree_store.mjs) in the committed registry",
  );
  assert.ok(
    onDisk.includes("packages/bee/tests/test_worktree_store_merge.mjs"),
    "expected the merge-half suite filename (packages/bee/tests/test_worktree_store_merge.mjs) in the committed registry",
  );
});

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed > 0 ? 1 : 0);
