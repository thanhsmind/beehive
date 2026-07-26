#!/usr/bin/env node
// Tests for scripts/ledger_parity.mjs — the guard for the THIRD copy
// relationship (lpsp-1): `.bee/bin/**` <-> the per-file sha256 hashes
// recorded in `.bee/onboarding.json`'s `managed.lib` / `managed.helpers`
// maps. Before this cell, nothing checked that relationship: a release could
// tag a commit whose ledger was never refreshed by self-onboard, and the
// ledger would then falsely report drift (or falsely report none) to every
// new session. USER-REPORTED, hand-fixed in 6412017: a clean checkout of
// main reported drift:true for .bee/bin/lib/lock.mjs at v1.9.0 while the
// files were byte-identical to source — only the tracked onboarding.json
// ledger was stale, because a release ran onboarding, then the tag was
// re-pointed to a later fix commit that changed .bee/bin/** without
// re-onboarding.
//
// Builds disposable fixture repos under a tmpdir (never touches the real
// checkout) shaped like `<root>/.bee/onboarding.json` +
// `<root>/.bee/bin/lib/*.mjs` [+ `<root>/.bee/bin/*.mjs` helpers], and drives
// computeLedgerParity(root) directly (no subprocess needed — the checker
// exports its core function for exactly this).
//
// Cases (must_haves): stale recorded hash -> FAIL; claimed file missing ->
// FAIL; unrecorded extra .mjs in the fully-managed lib dir -> FAIL; a
// freshly-onboarded tree -> PASS.

import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";

import { computeLedgerParity } from "../ledger_parity.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

function sha256(text) {
  return crypto.createHash("sha256").update(text, "utf8").digest("hex");
}

// Builds a fixture repo: `<root>/.bee/bin/lib/<name>` for each of `libFiles`
// (name -> content), `<root>/.bee/bin/<name>` for each of `helperFiles`, and
// `<root>/.bee/onboarding.json` with `managed.lib`/`managed.helpers` computed
// from `recordedLibFiles`/`recordedHelperFiles` (defaults to the live
// content — the "freshly onboarded" case) so a test can diverge the ledger
// from disk on purpose.
function buildFixture({
  libFiles = {},
  helperFiles = {},
  recordedLibFiles = libFiles,
  recordedHelperFiles = helperFiles,
  omitFromDisk = [], // rel lib file names to record but NEVER write (missing-file case)
}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "ledger-parity-fixture-"));
  const libDir = path.join(root, ".bee", "bin", "lib");
  fs.mkdirSync(libDir, { recursive: true });
  for (const [name, content] of Object.entries(libFiles)) {
    if (omitFromDisk.includes(name)) continue;
    fs.writeFileSync(path.join(libDir, name), content, "utf8");
  }
  for (const [name, content] of Object.entries(helperFiles)) {
    fs.writeFileSync(path.join(root, ".bee", "bin", name), content, "utf8");
  }
  const managed = {
    lib: Object.fromEntries(Object.entries(recordedLibFiles).map(([n, c]) => [n, sha256(c)])),
    helpers: Object.fromEntries(Object.entries(recordedHelperFiles).map(([n, c]) => [n, sha256(c)])),
  };
  fs.writeFileSync(
    path.join(root, ".bee", "onboarding.json"),
    JSON.stringify({ schema_version: "1.0", bee_version: "9.9.9", managed }, null, 2),
    "utf8",
  );
  return root;
}

function cleanup(root) {
  fs.rmSync(root, { recursive: true, force: true });
}

let failures = 0;
async function test(name, fn) {
  try {
    await fn();
    console.log(`PASS ${name}`);
  } catch (err) {
    failures += 1;
    console.error(`FAIL ${name}`);
    console.error(err && err.stack ? err.stack : err);
  }
}

await test("freshly-onboarded tree PASSES", async () => {
  const root = buildFixture({
    libFiles: { "alpha.mjs": "export const alpha = 1;\n", "beta.mjs": "export const beta = 2;\n" },
    helperFiles: { "bee.mjs": "console.log('helper');\n" },
  });
  try {
    const result = await computeLedgerParity(root);
    assert.equal(result.ok, true, `expected ok:true, got ${JSON.stringify(result)}`);
    assert.deepEqual(result.stale, []);
    assert.deepEqual(result.missing, []);
    assert.deepEqual(result.extra, []);
  } finally {
    cleanup(root);
  }
});

await test("stale recorded hash FAILS and names the file", async () => {
  const root = buildFixture({
    libFiles: { "alpha.mjs": "export const alpha = 1;\n" },
    // recorded hash is for OLD content; disk now holds NEW content — models
    // a self-onboard that ran before a later commit touched .bee/bin/lib/*.
    recordedLibFiles: { "alpha.mjs": "export const alpha = 0; // old\n" },
  });
  try {
    const result = await computeLedgerParity(root);
    assert.equal(result.ok, false);
    assert.deepEqual(result.stale, [".bee/bin/lib/alpha.mjs"]);
    assert.deepEqual(result.missing, []);
    assert.deepEqual(result.extra, []);
  } finally {
    cleanup(root);
  }
});

await test("claimed file missing FAILS and names the file", async () => {
  const root = buildFixture({
    libFiles: { "alpha.mjs": "export const alpha = 1;\n" },
    omitFromDisk: ["alpha.mjs"],
  });
  try {
    const result = await computeLedgerParity(root);
    assert.equal(result.ok, false);
    assert.deepEqual(result.missing, [".bee/bin/lib/alpha.mjs"]);
    assert.deepEqual(result.stale, []);
    assert.deepEqual(result.extra, []);
  } finally {
    cleanup(root);
  }
});

await test("unrecorded extra .mjs in the fully-managed lib dir FAILS", async () => {
  const root = buildFixture({
    libFiles: { "alpha.mjs": "export const alpha = 1;\n" },
  });
  try {
    // Drop an extra .mjs straight onto disk that the ledger never recorded —
    // models a new lib file added without re-running self-onboard.
    fs.writeFileSync(path.join(root, ".bee", "bin", "lib", "rogue.mjs"), "export const rogue = true;\n", "utf8");
    const result = await computeLedgerParity(root);
    assert.equal(result.ok, false);
    assert.deepEqual(result.extra, [".bee/bin/lib/rogue.mjs"]);
    assert.deepEqual(result.stale, []);
    assert.deepEqual(result.missing, []);
  } finally {
    cleanup(root);
  }
});

await test("missing onboarding.json ledger FAILS with a clear error, never throws", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "ledger-parity-fixture-"));
  try {
    const result = await computeLedgerParity(root);
    assert.equal(result.ok, false);
    assert.ok(result.error && /ledger/i.test(result.error), `expected a ledger-related error, got ${JSON.stringify(result)}`);
  } finally {
    cleanup(root);
  }
});

await test("THIS repo's real ledger currently passes (proves the check runs against reality, not just fixtures)", async () => {
  const REPO_ROOT = path.join(__dirname, "..", "..");
  const result = await computeLedgerParity(REPO_ROOT);
  assert.equal(result.ok, true, `expected THIS repo's ledger to be parity-clean, got ${JSON.stringify(result)}`);
});

if (failures > 0) {
  console.error(`\nFAIL scripts/tests/test_ledger_parity.mjs: ${failures} failure(s)`);
  process.exit(1);
}
console.log("\nPASS scripts/tests/test_ledger_parity.mjs");
process.exit(0);
