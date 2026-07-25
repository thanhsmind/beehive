#!/usr/bin/env node
// Guard: commands.verify in .bee/config.json must always run every mandatory
// suite. TEST-11/D-14 (SPEC §11.2, decision ed0b2920): the CLI registry
// contract test (test_bee_cli.mjs) is a public-command contract and must
// never silently fall out of the release verify chain again — this script
// is the mechanical guard that catches that regression at verify time.
//
// commands.verify now points at scripts/run_verify.mjs (a parallel runner)
// instead of an inline `&&`-chain string, so the check below loads that
// runner's exported SUITES list.
//
// cs-4 (contention-split) flip: run_verify.mjs's SUITES array is no longer
// hand-written — it's discovered by globbing `test_*.mjs` under a fixed set
// of roots, which is exactly why the OLD exact-membership check here (a
// mandatory path must appear verbatim in a hand-maintained array) stopped
// being the right shape: there is no hand-maintained array left to drift.
// The guard now asserts two INDEPENDENT things instead:
//   (1) floor count — the discovered suite total never silently drops below
//       a frozen historical floor (catches suites vanishing in bulk, e.g. a
//       discovery root typo'd or a directory emptied);
//   (2) per-suite rename/drop protection for a curated MANDATORY_SUITES
//       list — each must (a) still exist on disk under its recorded path
//       (a rename or delete is caught even though discovery would just
//       quietly stop finding it) and (b) still appear in the SUITES array
//       run_verify.mjs actually exports (catches the one way a suite can
//       exist on disk yet never run: landing in run_verify.mjs's own
//       EXCLUDE list without anyone updating this guard).
// A deleted or excluded mandatory suite still fails loudly either way.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");
const CONFIG_PATH = path.join(REPO_ROOT, ".bee", "config.json");
const RUNNER_PATH = path.join(REPO_ROOT, "scripts", "run_verify.mjs");

// The suites that MUST always run as part of commands.verify — full,
// explicit repo-relative paths (cs-2b: the old test_lib.mjs monolith is gone,
// split across six files; substring matching on bare basenames like "test_lib"
// is retired in favor of exact membership so a rename or a near-miss path
// can never silently satisfy an entry it was not meant to). Unchanged by the
// cs-4 discovery flip — this list is the curated subset that gets the
// stronger per-suite protection on top of the floor count below.
const MANDATORY_SUITES = [
  "packages/bee/tests/test_cli_state.mjs",
  "packages/bee/tests/test_cli_cells.mjs",
  "packages/bee/tests/test_state.mjs",
  "packages/bee/tests/test_guards.mjs",
  "packages/bee/tests/test_backlog_capture.mjs",
  "packages/bee/tests/test_misc.mjs",
  "packages/bee/tests/test_cells.mjs",
  "packages/bee/tests/test_reservations.mjs",
  "packages/bee/tests/test_claims.mjs",
  "packages/bee/tests/test_feedback.mjs",
  "packages/bee/tests/test_reviews.mjs",
  "scripts/tests/test_skill_render.mjs",
  "packages/bee/scripts/test_onboard_bee.mjs",
  "packages/bee/scripts/test_plugin_distribution.mjs",
  "scripts/tests/test_portable_paths.mjs",
  "packages/bee/hooks/test_model_guard.mjs",
  "packages/bee/hooks/test_write_guard.mjs",
  "packages/bee/hooks/test_hook_contracts.mjs",
  "packages/bee/tests/test_bee_cli.mjs",
  "scripts/tests/test_state_write_concurrency.mjs",
  "scripts/tests/test_installers_e2e.mjs",
  "scripts/tests/test_conformance.mjs",
  "scripts/tests/test_agents_budget.mjs",
  "packages/bee/tests/test_recovery.mjs",
  // okf-3: knowledge check joins the chain as a chain-failing suite (D22/D34)
  // — pinned here so it can never silently drop out of commands.verify again.
  // okf-4: the same pinned path also carries `knowledge index --check`
  // (D21 stale-generated-index freshness) — one dispatcher, two chain
  // entries, both protected by this exact-path pin.
  ".bee/bin/bee.mjs",
  // okf-5: the advisor-protocol migration coverage gate (D35) — pinned so the
  // `okf_migrate --check advisor-protocol` chain entry can never silently
  // drop out of commands.verify (it does not match the test_*.mjs discovery
  // glob, so EXTRA_SUITES membership is its only way into the chain).
  "scripts/okf_migrate.mjs",
  // f2-1b (F8): the suite that proves that coverage gate is HONEST — pins are
  // content-addressed and fully asserted, an empty/mismatched/unresolvable/
  // unscheme'd extraction exits 1 instead of reading as 0/0 green, and
  // onboarding.md's non-zero unparsed-block count proves the extractor is no
  // longer format-blind. The gate above can pass while lying; this is the
  // suite that catches that, so it must never drop out of the chain either.
  "scripts/tests/test_okf_pins.mjs",
  // f3-2 (G1/G8): the compatibility fallback's ONLY end-to-end proof. bee's
  // own checkout HAS a bundle and G2 forbids new prose under docs/specs/ here,
  // so this repo structurally cannot exercise "a host repo that never
  // migrated keeps working" — only the bundle-LESS fixture inside this suite
  // can. Dropping it out of the chain would leave the guarantee as unasserted
  // prose again, which is exactly how it would rot in one release with no
  // error (advisor-digest-f3 finding 1).
  "packages/bee/tests/test_bundle_mode.mjs",
  // f3-4 (G2): the fence that makes `docs/specs/` read-only for NEW content.
  // Pinned exact-path because it does not match the `test_*.mjs` discovery
  // glob — EXTRA_SUITES membership is its only way into the chain, and a
  // guard that silently stops running is worse than no guard: new prose would
  // land in the compatibility surface again with nothing to say so.
  "scripts/okf_specs_fence.mjs",
  // f4-6: the fence over the INSTRUCTION layer. Pinned exact-path for the same
  // reason as its sibling — it does not match the `test_*.mjs` discovery glob,
  // so EXTRA_SUITES membership is its only way into the chain. It exists
  // because three hand audits each found instruction-layer misroutes the
  // previous one missed while the chain stayed green; a guard against that
  // failure mode which can itself silently stop running would be the same
  // failure mode wearing a script.
  "scripts/okf_instructions_fence.mjs",
  // i-1 (issues-46-53 D1/D2): the docs/backlog.md PBI-id uniqueness gate.
  // Pinned exact-path for the same reason as the fence scripts above — its
  // filename doesn't match the `test_*.mjs` discovery glob, so EXTRA_SUITES
  // membership is its only route into the chain, and a duplicate-id gate
  // that can silently stop running protects nothing (exactly the failure
  // mode #49 itself was).
  "scripts/backlog_uniqueness.mjs",
];

// f2-3 (F6): a migration cell's coverage gate must be in the chain, and the
// MANDATORY_SUITES pin above is an exact-PATH pin — "scripts/okf_migrate.mjs"
// already protects the dispatcher from dropping out, but not any individual
// `--check <area>` argument variant riding it. These are the argument variants
// the chain must carry, checked verbatim against EXTRA_SUITES so that deleting
// one area's gate while leaving another's is a loud failure instead of a
// silent hole in coverage.
export const MANDATORY_SUITE_ARGS = [
  ["scripts/okf_migrate.mjs", "--check", "advisor-protocol"],
  ["scripts/okf_migrate.mjs", "--check-patterns"],
  ["scripts/okf_migrate.mjs", "--check", "doctrine-layer"],
  ["scripts/okf_migrate.mjs", "--check", "decision-memory"],
  ["scripts/okf_migrate.mjs", "--check", "verify-pipeline"],
  ["scripts/okf_migrate.mjs", "--check", "performance-log"],
  ["scripts/okf_migrate.mjs", "--check", "feedback-digest"],
  ["scripts/okf_migrate.mjs", "--check", "onboarding"],
  ["scripts/okf_migrate.mjs", "--check", "hook-runtime"],
  // f2-11 (F6/F9): worktree-parallelism's own D35 coverage gate, and the first
  // user of a THIRD anchor scheme. This is the one area of the eleven that
  // genuinely has no numbered anchors — no `B*`/`R*`/`E*`/`P*` id anywhere in
  // its 225 lines and none of the four anchor-bearing nine-section headings —
  // so `ba-nine-section` derives 0 anchors AND 0 unparsed blocks from it. F9
  // forbids forcing it into that shape and D10 forbids inventing ids the
  // source never had, so `narrative-sections` derives the anchors from the
  // source's OWN `## ` headings, slugified: 10 anchors, unparsed_blocks: 2
  // (the `**Area:**` / `**Status:**` preamble lines, which belong to no
  // section and are reported rather than invented into anchors). Seven
  // concepts, split by TOPIC (purpose and boundary; the trust model; entering;
  // returning and the merge gate; routing and visibility; the cross-worktree
  // holds ledger; store tiers and the implementation map).
  ["scripts/okf_migrate.mjs", "--check", "worktree-parallelism"],
  // f2-13 (F6/F9/F10): workflow-state's own D35 coverage gate — the last area
  // of the migration and the largest (140 anchors from a 1464-line source),
  // migrated across several cells (F10) and riding a REPAIRED pin: the source
  // shipped `R19`/`R20`/`R21` twice each, so the second occurrence of each id
  // in document order was renumbered `R19a`/`R20a`/`R21a` before the pin was
  // captured. Fifteen concepts — D30's nine behavior clusters as the spine,
  // with the two that would have swallowed 35+ anchors apiece once the rules,
  // edge cases and pointer bullets were distributed split by topic inside
  // themselves rather than left as dumping grounds.
  ["scripts/okf_migrate.mjs", "--check", "workflow-state"],
  // f3-5 (G6): okf-profile's own D35 coverage gate — the TWELFTH and last area
  // migration, and the only one whose source DEFINES the bundle it moved into.
  // 24 anchors (13 B including the refinement-suffixed B6b / 0 R / 4 E / 7 P)
  // from blob 9267d3e at 53d8111, unparsed_blocks: 17. `rules: 0` is measured,
  // not missing: the nine `## Business Rules` bullets carry no `R` id, so none
  // was invented (D10) and they are counted unparsed while still being re-homed
  // verbatim. Five concepts, split by topic.
  ["scripts/okf_migrate.mjs", "--check", "okf-profile"],
  // f3-4 (G2): the exact-PATH pin above protects the fence script itself, but
  // not the `--selftest` variant riding it — and the selftest is the only
  // thing that proves the fence BITES (a bare live run passes trivially the
  // moment docs/specs/ happens to be clean, which is the normal state). Both
  // must be in the chain, so both are pinned: the argument variant here, the
  // bare live run by the path pin.
  ["scripts/okf_specs_fence.mjs", "--selftest"],
  // f4-6: same shape, same reason — the bare live run of the instruction fence
  // passes trivially whenever the instruction layer happens to be clean (its
  // normal state), so the `--selftest` variant is the only thing proving the
  // fence still bites, that it stays inert with no bundle, and that its four
  // exemption classes still hold. The path pin above protects the script; this
  // pins the argument variant riding it.
  ["scripts/okf_instructions_fence.mjs", "--selftest"],
  // i-1 (issues-46-53 D1/D2): the exact-PATH pin above protects the backlog
  // uniqueness gate script itself, but not the `--check` variant riding it —
  // and a bare run with no args does not exercise the gate at all. Both must
  // be in the chain; the argument variant is pinned here.
  ["scripts/backlog_uniqueness.mjs", "--check"],
];

// Floor count: total discovered suites must never silently drop below this
// number. Frozen at the count run_verify.mjs actually discovered the moment
// this guard flipped from hand-listing to convention-based discovery (cs-4,
// 2026-07-20): 47 suites from the old hand-written array, plus 3 real
// `test_*.mjs` files discovery found sitting in the repo that the old array
// had silently never run (scripts/test_config_samples_safe.mjs,
// packages/bee/tests/test_perf.mjs, hooks/test_bypass_stop_net.mjs)
// = 50. Bump this UP whenever a suite is intentionally added; it should
// never need to go down.
// f2-1b: +1 for scripts/test_okf_pins.mjs (the derived-pin honesty suite) = 51.
// f2-3: +1 for the `okf_migrate --check doctrine-layer` coverage gate = 52.
// f2-5: +1 for the `okf_migrate --check decision-memory` coverage gate = 53.
// f2-6: +1 for the `okf_migrate --check verify-pipeline` coverage gate = 54.
// f2-7: +1 for the `okf_migrate --check performance-log` coverage gate = 55.
// f2-8: +1 for the `okf_migrate --check feedback-digest` coverage gate = 56.
// f2-9: +1 for the `okf_migrate --check onboarding` coverage gate = 57.
// f2-10: +1 for the `okf_migrate --check hook-runtime` coverage gate = 58.
// f2-11: +1 for the `okf_migrate --check worktree-parallelism` coverage gate = 59.
// f2-13: +1 for the `okf_migrate --check workflow-state` coverage gate = 60 —
// the last area of the migration, so this is the last of these bumps.
// f3-4: +2 for the docs/specs read-only fence — `okf_specs_fence --selftest`
// (the fixtures that prove it bites, and that it stays inert with no bundle)
// and the bare live run against this repo's own docs/specs/ = 62.
// f4-6: +2 for the INSTRUCTION-layer fence — `okf_instructions_fence --selftest`
// (the fixtures that prove it bites, that it stays inert with no bundle, and
// that all four exemption classes hold) and the bare live run against this
// repo's own skills/, hooks/ and AGENTS.md = 64.
// i-1 (issues-46-53 D1/D2): +1 for the docs/backlog.md PBI-id uniqueness
// gate (`backlog_uniqueness --check`) = 65.
const SUITE_FLOOR_COUNT = 65;

/**
 * Checks a discovered suite list against a mandatory list and a floor count.
 * `suitePaths` is a flat list of repo-relative suite path strings (e.g.
 * "packages/bee/tests/test_cells.mjs"). Returns
 * { ok, belowFloor, count, missingOnDisk, missingFromSuites } —
 * missingOnDisk / missingFromSuites are `mandatory`-ordered subsets that
 * failed each respective check (no substring matching — a mandatory path
 * must appear verbatim).
 */
function checkDiscovery(suitePaths, mandatory, floor) {
  const paths = Array.isArray(suitePaths) ? suitePaths : [];
  const present = new Set(paths);

  const belowFloor = paths.length < floor;
  const missingOnDisk = mandatory.filter(
    (suite) => !fs.existsSync(path.join(REPO_ROOT, suite)),
  );
  const missingFromSuites = mandatory.filter((suite) => !present.has(suite));

  return {
    ok: !belowFloor && missingOnDisk.length === 0 && missingFromSuites.length === 0,
    belowFloor,
    count: paths.length,
    missingOnDisk,
    missingFromSuites,
  };
}

/**
 * The mandatory ARGUMENT VARIANTS missing from a discovered suite list.
 * `suites` is the raw SUITES array (entries are string or string[]); each
 * mandatory entry must appear as an exact, element-for-element match — a
 * dispatcher path alone never satisfies it, which is the whole point.
 */
function missingArgVariants(suites, mandatoryArgs) {
  const present = new Set(
    (Array.isArray(suites) ? suites : []).map((entry) =>
      JSON.stringify(Array.isArray(entry) ? entry : [entry]),
    ),
  );
  return mandatoryArgs.filter((entry) => !present.has(JSON.stringify(entry)));
}

// ─── internal self-test: prove the checker actually bites, all three ways ─
// Never touches the real runner or the real MANDATORY_SUITES — synthetic
// data only, so a rubber-stamp checker can't hide behind reality happening
// to be fine.
{
  let selfTestFailed = false;

  // (a) floor count: fewer discovered suites than the floor requires.
  const floorResult = checkDiscovery(["a.mjs", "b.mjs", "c.mjs"], [], 10);
  if (floorResult.ok || !floorResult.belowFloor) {
    console.error("FAIL test_verify_manifest: internal self-test did not catch a suite count below the floor");
    console.error(`      checker result: ${JSON.stringify(floorResult)}`);
    selfTestFailed = true;
  }

  // (b) membership: a mandatory suite that exists on disk (this very file)
  // but is missing from the discovered list.
  const presentButUnwired = "scripts/tests/test_verify_manifest.mjs";
  const dummyFloorSuites = Array.from({ length: 10 }, (_, i) => `dummy_${i}.mjs`);
  const membershipResult = checkDiscovery(dummyFloorSuites, [presentButUnwired], 10);
  if (membershipResult.ok || !membershipResult.missingFromSuites.includes(presentButUnwired)) {
    console.error("FAIL test_verify_manifest: internal self-test did not catch a mandatory suite missing from the discovered SUITES array");
    console.error(`      checker result: ${JSON.stringify(membershipResult)}`);
    selfTestFailed = true;
  }

  // (c) on-disk existence: a mandatory suite path that does not exist,
  // even though it IS present in the discovered list (renamed/deleted, but
  // some stale entry still names the old path).
  const neverExists = "scripts/__cs4_selftest_never_exists__.mjs";
  const dummyFloorSuitesWithGhost = [neverExists, ...Array.from({ length: 9 }, (_, i) => `dummy_${i}.mjs`)];
  const onDiskResult = checkDiscovery(dummyFloorSuitesWithGhost, [neverExists], 10);
  if (onDiskResult.ok || !onDiskResult.missingOnDisk.includes(neverExists)) {
    console.error("FAIL test_verify_manifest: internal self-test did not catch a mandatory suite missing on disk");
    console.error(`      checker result: ${JSON.stringify(onDiskResult)}`);
    selfTestFailed = true;
  }

  // (d) argument variant: the dispatcher path IS present, but the specific
  // `--check <area>` variant that carries one area's coverage gate is not.
  // This is the hole the exact-PATH pin cannot see (f2-3).
  const argVariantResult = missingArgVariants(
    [["scripts/okf_migrate.mjs", "--check", "advisor-protocol"], "scripts/okf_migrate.mjs"],
    [["scripts/okf_migrate.mjs", "--check", "doctrine-layer"]],
  );
  if (argVariantResult.length !== 1) {
    console.error("FAIL test_verify_manifest: internal self-test did not catch a mandatory suite ARGUMENT VARIANT missing while its dispatcher path was present");
    console.error(`      checker result: ${JSON.stringify(argVariantResult)}`);
    selfTestFailed = true;
  }

  if (selfTestFailed) {
    process.exit(1);
  }

  console.log("PASS test_verify_manifest: internal self-test — checker correctly flags floor-count, membership, on-disk-existence, and argument-variant regressions");
}

// ─── real check: THIS repo's real .bee/config.json + run_verify.mjs ───────

if (!fs.existsSync(CONFIG_PATH)) {
  console.error(`FAIL test_verify_manifest: ${CONFIG_PATH} does not exist`);
  process.exit(1);
}

let config;
try {
  config = JSON.parse(fs.readFileSync(CONFIG_PATH, "utf8"));
} catch (error) {
  console.error(`FAIL test_verify_manifest: could not parse ${CONFIG_PATH}: ${error.message}`);
  process.exit(1);
}

const verifyString = config?.commands?.verify;
if (typeof verifyString !== "string" || !verifyString.trim()) {
  console.error("FAIL test_verify_manifest: .bee/config.json commands.verify is missing or empty");
  process.exit(1);
}

if (!verifyString.includes("run_verify.mjs")) {
  console.error(`FAIL test_verify_manifest: .bee/config.json commands.verify does not invoke scripts/run_verify.mjs`);
  console.error(`      commands.verify: ${verifyString}`);
  process.exit(1);
}

if (!fs.existsSync(RUNNER_PATH)) {
  console.error(`FAIL test_verify_manifest: ${RUNNER_PATH} does not exist`);
  process.exit(1);
}

let runnerModule;
try {
  runnerModule = await import(pathToFileURL(RUNNER_PATH).href);
} catch (error) {
  console.error(`FAIL test_verify_manifest: could not import ${RUNNER_PATH}: ${error.message}`);
  process.exit(1);
}

const suites = runnerModule.SUITES;
if (!Array.isArray(suites) || suites.length === 0) {
  console.error("FAIL test_verify_manifest: scripts/run_verify.mjs does not export a non-empty SUITES array");
  process.exit(1);
}

const suitePaths = suites.map((entry) => (Array.isArray(entry) ? entry[0] : entry));

const result = checkDiscovery(suitePaths, MANDATORY_SUITES, SUITE_FLOOR_COUNT);
if (!result.ok) {
  if (result.belowFloor) {
    console.error(`FAIL test_verify_manifest: run_verify.mjs discovered only ${result.count} suite(s), below the frozen floor of ${SUITE_FLOOR_COUNT}`);
  }
  if (result.missingOnDisk.length > 0) {
    console.error(`FAIL test_verify_manifest: ${result.missingOnDisk.length} mandatory suite(s) no longer exist on disk: ${result.missingOnDisk.join(", ")}`);
  }
  if (result.missingFromSuites.length > 0) {
    console.error(`FAIL test_verify_manifest: ${result.missingFromSuites.length} mandatory suite(s) exist on disk but are missing from run_verify.mjs's discovered SUITES array: ${result.missingFromSuites.join(", ")}`);
  }
  console.error(`      SUITES (${suitePaths.length}): ${JSON.stringify(suitePaths)}`);
  process.exit(1);
}

const missingArgs = missingArgVariants(suites, MANDATORY_SUITE_ARGS);
if (missingArgs.length > 0) {
  console.error(`FAIL test_verify_manifest: ${missingArgs.length} mandatory suite ARGUMENT VARIANT(s) are missing from run_verify.mjs's SUITES array: ${missingArgs.map((e) => e.join(" ")).join(" | ")}`);
  console.error(`      a coverage gate whose dispatcher is pinned but whose own --check entry was dropped protects nothing (F6)`);
  process.exit(1);
}

console.log(`PASS test_verify_manifest: run_verify.mjs discovered ${result.count} suites (floor ${SUITE_FLOOR_COUNT}); all ${MANDATORY_SUITES.length} mandatory suites present on disk and wired in, exact-path matched (${MANDATORY_SUITES.join(", ")}); all ${MANDATORY_SUITE_ARGS.length} mandatory argument variants wired in (${MANDATORY_SUITE_ARGS.map((e) => e.join(" ")).join(" | ")})`);
