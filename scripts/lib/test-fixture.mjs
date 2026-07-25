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

function recordPass(name) {
  passed += 1;
  console.log(`PASS  ${name}`);
}

function recordFailure(name, error) {
  failed += 1;
  console.log(`FAIL  ${name}`);
  console.log(`      ${error instanceof Error ? error.message : error}`);
}

export function check(name, fn) {
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
 * test_lib.mjs.
 */
export function printSummaryAndExit() {
  console.log(`\n${passed} passed, ${failed} failed`);
  process.exit(failed > 0 ? 1 : 0);
}
