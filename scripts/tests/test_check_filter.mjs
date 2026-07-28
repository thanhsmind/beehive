#!/usr/bin/env node
// test_check_filter.mjs — pins the safety properties of BEE_CHECK_ONLY
// (scripts/lib/test-fixture.mjs, cfl-1). P1 finding (review-p1-fixes F4):
// `rg BEE_CHECK_ONLY` across every suite hit only the implementation — the
// filter shipped with zero tests. Its safety properties were verified once
// by hand and pinned by nothing, so a regression that counts skips as
// passes, or lets a zero-match filter exit 0, would silently turn all
// shared-fixture suites trivially green. This suite pins:
//   (1) unset — byte-identical to pre-filter output/exit (no skip counter,
//       no extra text);
//   (2) a substring filter actually gates which check BODIES run (not just
//       which line prints) — skipped means never executed, never counted
//       as passed;
//   (3) a /regex/ filter is honored, anchors and flags included, and
//       differs from plain substring matching;
//   (4) a filter matching zero checks exits NON-ZERO with a typed refusal,
//       never a trivial green;
//   (5) an INVALID regex is a typed refusal, not an uncaught crash at
//       module load (p1-2 fix — the predicate builder used to call `new
//       RegExp(...)` unguarded, which threw a raw SyntaxError before any
//       check ever ran).
//
// Mechanism: BEE_CHECK_ONLY is read once at module load, so the same
// process/module instance can't be re-exercised under a different filter
// value. Each property is instead pinned by writing a small throwaway
// fixture suite into a temp dir — importing the REAL check/assert/
// printSummaryAndExit from scripts/lib/test-fixture.mjs — and running it as
// a genuine child OS process per BEE_CHECK_ONLY value, asserting on the
// child's stdout/stderr/exit code and a body-execution sentinel file. Same
// "each racer is its own OS process" discipline as test_claim_race.mjs.
//
// new_suite_reason: BEE_CHECK_ONLY is a harness-level mechanism embedded in
// the shared check() runner, not owned by any suite under test — hence a
// new standalone file rather than an addition to an existing suite.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const REPO_ROOT = path.join(path.dirname(__filename), '..', '..');
const TEST_FIXTURE_PATH = path.join(REPO_ROOT, 'scripts', 'lib', 'test-fixture.mjs');
const TEST_FIXTURE_URL = pathToFileURL(TEST_FIXTURE_PATH).href;

let passed = 0;
let failed = 0;

function checkLocal(name, fn) {
  try {
    fn();
    passed += 1;
    console.log(`PASS  ${name}`);
  } catch (error) {
    failed += 1;
    console.log(`FAIL  ${name}`);
    console.log(`      ${error instanceof Error ? error.message : error}`);
  }
}

function assertLocal(condition, message) {
  if (!condition) throw new Error(message);
}

// ─── throwaway fixture suite ────────────────────────────────────────────────
// Three named checks with deliberately overlapping name substrings so a
// substring filter and an anchored regex filter select different subsets:
//   apple-check      — passes
//   appleseed-check  — passes; a bare substring filter "apple" also matches
//                      this one, but an anchored /^apple-check$/ does not —
//                      that's what distinguishes regex from substring below.
//   banana-check     — FAILS on purpose (assert false) so the unfiltered
//                      run has something real to pin (non-zero exit,
//                      FAIL line + message) and so a filter that excludes
//                      it can be proven to genuinely skip it (green run)
//                      rather than accidentally still running it.
// Each body appends its own name to CFL_SENTINEL (when set) before
// asserting, so the test can prove a SKIPped check's body never ran at
// all — not merely that it printed SKIP instead of PASS.
function fixtureSuiteSource() {
  return `
import fs from 'node:fs';
import { check, assert, printSummaryAndExit } from ${JSON.stringify(TEST_FIXTURE_URL)};

const sentinelPath = process.env.CFL_SENTINEL;
function mark(name) {
  if (sentinelPath) fs.appendFileSync(sentinelPath, name + '\\n');
}

check('apple-check', () => { mark('apple-check'); assert(true, 'apple-check should pass'); });
check('appleseed-check', () => { mark('appleseed-check'); assert(true, 'appleseed-check should pass'); });
check('banana-check', () => { mark('banana-check'); assert(false, 'banana-check deliberately fails'); });

printSummaryAndExit();
`;
}

function makeFixtureDir() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'cfl-fixture-'));
  const suitePath = path.join(dir, 'fixture-suite.mjs');
  fs.writeFileSync(suitePath, fixtureSuiteSource());
  return { dir, suitePath };
}

function runFixture(suitePath, { checkOnly, sentinelPath } = {}) {
  const env = { ...process.env };
  delete env.BEE_CHECK_ONLY;
  if (checkOnly !== undefined) env.BEE_CHECK_ONLY = checkOnly;
  delete env.CFL_SENTINEL;
  if (sentinelPath) env.CFL_SENTINEL = sentinelPath;
  return spawnSync(process.execPath, [suitePath], { encoding: 'utf8', env });
}

function readSentinel(sentinelPath) {
  if (!fs.existsSync(sentinelPath)) return [];
  return fs.readFileSync(sentinelPath, 'utf8').split('\n').filter(Boolean);
}

function withFixture(fn) {
  const { dir, suitePath } = makeFixtureDir();
  const sentinelPath = path.join(dir, 'ran.log');
  try {
    fn({ suitePath, sentinelPath });
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

// ─── (1) unset BEE_CHECK_ONLY: byte-identical to pre-filter behavior ───────
checkLocal('unfiltered run is byte-identical to pre-filter output/exit', () => {
  withFixture(({ suitePath, sentinelPath }) => {
    const result = runFixture(suitePath, { sentinelPath });
    const expectedStdout =
      'PASS  apple-check\n' +
      'PASS  appleseed-check\n' +
      'FAIL  banana-check\n' +
      '      banana-check deliberately fails\n' +
      '\n' +
      '2 passed, 1 failed\n';
    assertLocal(
      result.stdout === expectedStdout,
      `unfiltered stdout must be byte-identical to the pre-filter format (no skip counter, no BEE_CHECK_ONLY suffix) — got:\n${result.stdout}`,
    );
    assertLocal(result.status === 1, `unfiltered run with a failing check must exit 1, got ${result.status}`);
    const ran = readSentinel(sentinelPath);
    assertLocal(
      ran.length === 3 &&
        ran.includes('apple-check') &&
        ran.includes('appleseed-check') &&
        ran.includes('banana-check'),
      `unfiltered run must execute all three check bodies, got: ${JSON.stringify(ran)}`,
    );
  });
});

// ─── (2) substring filter: only matching bodies run, rest counted skipped ──
checkLocal('substring filter runs only matching bodies, skips the rest by name', () => {
  withFixture(({ suitePath, sentinelPath }) => {
    const result = runFixture(suitePath, { checkOnly: 'apple', sentinelPath });
    assertLocal(
      result.status === 0,
      `"apple" matches only the two passing checks, expected exit 0, got ${result.status}`,
    );
    assertLocal(
      result.stdout.includes('PASS  apple-check'),
      `expected apple-check to run and pass:\n${result.stdout}`,
    );
    assertLocal(
      result.stdout.includes('PASS  appleseed-check'),
      `expected appleseed-check to run and pass (substring match):\n${result.stdout}`,
    );
    assertLocal(
      result.stdout.includes('SKIP  banana-check'),
      `expected banana-check to be SKIPped, not silently counted as passed:\n${result.stdout}`,
    );
    assertLocal(
      !result.stdout.includes('FAIL  banana-check'),
      `banana-check must not run (it would fail) when filtered out:\n${result.stdout}`,
    );
    assertLocal(
      result.stdout.includes('2 passed, 0 failed, 1 skipped (BEE_CHECK_ONLY=apple)'),
      `summary must report the skip count and name the filter used:\n${result.stdout}`,
    );
    const ran = readSentinel(sentinelPath);
    assertLocal(
      ran.length === 2 && ran.includes('apple-check') && ran.includes('appleseed-check') && !ran.includes('banana-check'),
      `a SKIPped check's body must never execute at all (skip must not be cosmetic) — got: ${JSON.stringify(ran)}`,
    );
  });
});

// ─── (3) regex filter: anchors and flags honored, differs from substring ──
checkLocal('slash-wrapped filter is honored as a regex, anchors included', () => {
  withFixture(({ suitePath, sentinelPath }) => {
    const result = runFixture(suitePath, { checkOnly: '/^apple-check$/', sentinelPath });
    assertLocal(
      result.status === 0,
      `anchored regex selects only the passing apple-check, expected exit 0, got ${result.status}`,
    );
    const ran = readSentinel(sentinelPath);
    assertLocal(
      ran.length === 1 && ran[0] === 'apple-check',
      `anchored /^apple-check$/ must run ONLY apple-check (a bare substring "apple" would also match appleseed-check) — got: ${JSON.stringify(ran)}`,
    );
  });
});

checkLocal('regex flags are honored', () => {
  withFixture(({ suitePath, sentinelPath }) => {
    const result = runFixture(suitePath, { checkOnly: '/^APPLE-CHECK$/i', sentinelPath });
    assertLocal(
      result.status === 0,
      `case-insensitive regex must still match lowercase apple-check, expected exit 0, got ${result.status}`,
    );
    const ran = readSentinel(sentinelPath);
    assertLocal(
      ran.length === 1 && ran[0] === 'apple-check',
      `the "i" flag must be passed through to RegExp (uppercase pattern must match the lowercase name) — got: ${JSON.stringify(ran)}`,
    );
  });
});

// ─── (4) zero-match filter refuses non-zero, never a trivial green ────────
checkLocal('zero-match filter exits non-zero with a typed refusal', () => {
  withFixture(({ suitePath, sentinelPath }) => {
    const result = runFixture(suitePath, { checkOnly: 'no-such-check-xyz', sentinelPath });
    assertLocal(
      result.status === 1,
      `a filter matching zero checks must NEVER exit 0 (that would read as a trivial green run), got ${result.status}`,
    );
    assertLocal(
      result.stderr.includes('matched zero checks'),
      `expected the typed zero-match refusal message on stderr, got:\n${result.stderr}`,
    );
    const ran = readSentinel(sentinelPath);
    assertLocal(
      ran.length === 0,
      `zero-match run must execute no check bodies at all — got: ${JSON.stringify(ran)}`,
    );
  });
});

// ─── (5) invalid regex is a typed refusal, not a module-load crash ────────
checkLocal('invalid /regex/ refuses cleanly instead of crashing at module load', () => {
  withFixture(({ suitePath }) => {
    const result = runFixture(suitePath, { checkOnly: '/[/' });
    assertLocal(
      result.status === 1,
      `an invalid regex filter must refuse with exit 1, not crash with some other code, got ${result.status}`,
    );
    assertLocal(
      result.stderr.includes('is not a valid regex'),
      `expected the typed invalid-regex refusal on stderr, not a bare uncaught-exception stack trace — got:\n${result.stderr}`,
    );
    assertLocal(
      result.stdout === '',
      `an invalid regex must refuse before any check runs (no PASS/SKIP/FAIL lines at all) — got:\n${result.stdout}`,
    );
  });
});

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed > 0 ? 1 : 0);
