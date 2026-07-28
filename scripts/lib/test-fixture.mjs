#!/usr/bin/env node
// test-fixture.mjs — shared temp-repo bootstrap, cell factory, and the
// check/assert runner plumbing used by
// packages/bee/tests/test_lib.mjs.
//
// Extracted 1:1 from that file's original top section (cs-1): same
// PASS/FAIL console output, same passed/failed counters, same exit-code
// contract. Zero test-behavior change — this module only relocates code
// that used to be inline.
//
// Import-path note: this file is only ever imported from the CANONICAL
// packages/bee/tests/test_lib.mjs, which is the only copy
// run_verify.mjs spawns (scripts/run_verify.mjs SUITES lists that exact
// path). The .claude-plugin/ and .codex-plugin/ trees carry byte-rendered
// mirror copies of test_lib.mjs that are never separately executed — they
// are only diffed for byte-identity (scripts/test_skill_render.mjs) — so a
// relative import that resolves correctly from the canonical tree and
// would NOT resolve from a mirror copy is safe and already the established
// pattern here (see the pre-existing `../../../../scripts/lib/run-module-
// worker.mjs` import a few lines above this one's call site).
//
// cfl-1: BEE_CHECK_ONLY=<needle|/regex/> scopes check() in this shared
// helper to matching-name checks (non-matching run no body, counted as
// skipped and named in the summary — never silently dropped); unset, this
// module's behavior is byte-identical to before this cell.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { writeJsonAtomic } from '../../packages/bee/lib/fsutil.mjs';

// ─── temp repo setup ────────────────────────────────────────────────────────

/**
 * Bootstraps a fresh temp repo for a test run: .git/, .bee/ (seeded with
 * onboarding.json), and a src/deep/nested tree for path-resolution tests.
 * Returns the repo root path. Caller owns cleanup (fs.rmSync) at the end of
 * the run, same as before extraction.
 */
export function makeTempRepo() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-test-'));
  fs.mkdirSync(path.join(root, '.git'), { recursive: true });
  fs.mkdirSync(path.join(root, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(root, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  fs.mkdirSync(path.join(root, 'src'), { recursive: true });
  fs.mkdirSync(path.join(root, 'src', 'deep', 'nested'), { recursive: true });
  return root;
}

export function makeCell(id, extra = {}) {
  return {
    id,
    feature: 'demo',
    title: `Cell ${id}`,
    lane: 'small',
    status: 'open',
    deps: [],
    action: 'Do the thing per D1.',
    verify: 'node -e "process.exit(0)"',
    ...extra,
  };
}

// ─── check/assert runner plumbing ──────────────────────────────────────────

let passed = 0;
let failed = 0;
let skipped = 0;

function recordPass(name) {
  passed += 1;
  console.log(`PASS  ${name}`);
}

function recordFailure(name, error) {
  failed += 1;
  console.log(`FAIL  ${name}`);
  console.log(`      ${error instanceof Error ? error.message : error}`);
}

function recordSkip(name) {
  skipped += 1;
  console.log(`SKIP  ${name}`);
}

// ─── BEE_CHECK_ONLY name filter (cfl-1) ────────────────────────────────────
// A name-pattern filter for the check() runner below: with BEE_CHECK_ONLY
// set, only checks whose name matches run — everything else is skipped, not
// silently passed. A value wrapped in slashes (/re/[flags]) is a regex;
// anything else is a case-insensitive substring match. Exported so suites
// with their OWN local check() (22 of them, not converted in this cell) can
// adopt the same predicate later.
export function checkOnlyPredicate(filterValue) {
  if (!filterValue) return null;
  const regexForm = /^\/(.*)\/([a-z]*)$/.exec(filterValue);
  if (regexForm) {
    let re;
    try {
      re = new RegExp(regexForm[1], regexForm[2]);
    } catch (error) {
      // An invalid /regex/ must never crash the module at import time (that
      // would take down every suite that imports this file, not just the
      // filtered run) — refuse with a typed message instead, same shape as
      // the zero-match refusal in printSummaryAndExit below.
      console.error(
        `test-fixture: BEE_CHECK_ONLY="${filterValue}" is not a valid regex — ${error.message}. FIX: check the filter syntax.`,
      );
      process.exit(1);
    }
    return (name) => re.test(name);
  }
  const needle = filterValue.toLowerCase();
  return (name) => name.toLowerCase().includes(needle);
}

// Computed once at module load — BEE_CHECK_ONLY is set by the process
// environment before a suite ever runs, same lifecycle as BEE_VERIFY_ONLY
// in scripts/run_verify.mjs.
const CHECK_ONLY = process.env.BEE_CHECK_ONLY || '';
const checkOnlyMatch = checkOnlyPredicate(CHECK_ONLY);

export function check(name, fn) {
  if (checkOnlyMatch && !checkOnlyMatch(name)) {
    recordSkip(name);
    return;
  }
  try {
    const result = fn();
    if (result && typeof result.then === 'function') {
      return result.then(
        () => recordPass(name),
        (error) => recordFailure(name, error),
      );
    }
    recordPass(name);
  } catch (error) {
    recordFailure(name, error);
  }
}

export function assert(condition, message) {
  if (!condition) throw new Error(message);
}

export function assertThrows(fn, needle, message) {
  try {
    fn();
  } catch (error) {
    const text = error instanceof Error ? error.message : String(error);
    assert(
      text.toLowerCase().includes(needle.toLowerCase()),
      `${message} — threw, but message "${text}" does not mention "${needle}"`,
    );
    return;
  }
  throw new Error(`${message} — expected an error, none thrown`);
}

// The async sibling of assertThrows (msh-5): startFeature (lib/state.mjs)
// now wraps its body in withStoreLock, so its refusals reject a Promise
// instead of throwing synchronously — same message-substring contract.
export async function assertRejects(fn, needle, message) {
  try {
    await fn();
  } catch (error) {
    const text = error instanceof Error ? error.message : String(error);
    assert(
      text.toLowerCase().includes(needle.toLowerCase()),
      `${message} — threw, but message "${text}" does not mention "${needle}"`,
    );
    return;
  }
  throw new Error(`${message} — expected an error, none thrown`);
}

/**
 * Prints the "<n> passed, <m> failed" summary and exits nonzero on any
 * failure — same contract as the two lines this replaces at the bottom of
 * test_lib.mjs. With BEE_CHECK_ONLY unset this is byte-identical to before
 * cfl-1: no skip counter, no extra text. With BEE_CHECK_ONLY set, the
 * summary also reports the skip count and the filter used, and a run that
 * matched zero checks (nothing ran, so it can never read as green) exits
 * non-zero with a typed message instead of the ordinary 0-passed/0-failed
 * summary.
 */
export function printSummaryAndExit() {
  if (!checkOnlyMatch) {
    console.log(`\n${passed} passed, ${failed} failed`);
    process.exit(failed > 0 ? 1 : 0);
  }
  if (passed + failed === 0) {
    console.error(
      `test-fixture: BEE_CHECK_ONLY="${CHECK_ONLY}" matched zero checks — refusing a silent trivial-green run. FIX: check the filter against a check's name.`,
    );
    process.exit(1);
  }
  console.log(`\n${passed} passed, ${failed} failed, ${skipped} skipped (BEE_CHECK_ONLY=${CHECK_ONLY})`);
  process.exit(failed > 0 ? 1 : 0);
}
