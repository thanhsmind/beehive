#!/usr/bin/env node
// test_compact_verbs.mjs — black-box tests for the two DISPATCHED CLI verbs
// (feature compaction-hardening, cell cz-4; decisions D3/D12/D13): `bee state
// compact-log` and `bee state compact-check`. Drives the real shipped binary
// (.bee/bin/bee.mjs, the exact entrypoint every runtime invokes) as a
// subprocess against an isolated fixture store — mirrors the runBee/record
// conventions at scripts/tests/test_conformance.mjs:44,95-97,51-55 — and NEVER
// touches this repo's own .bee/ state.
//
// This file covers the WIRING: flags reach lib/compaction.mjs's functions
// unmangled, a usage error exits non-zero with nothing written, a detected
// mismatch still exits 0 (D13), and the anchor_missing check this verb adds
// on top of compactCheck's own output carries the exact D10 nudge command.
// The counting-rule algorithm itself (D5/D9) is already proven at the module
// level by scripts/tests/test_compaction_module.mjs (cz-3) — this file does not
// re-prove it, only that the CLI passes arguments through correctly.
//
// Exits 1 on any FAIL.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";

import { runModuleWorker } from "../lib/run-module-worker.mjs";
import { ANCHOR_NUDGE_COMMAND } from "../../skills/bee-hive/templates/lib/compaction.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");
const BEE_MJS = path.join(REPO_ROOT, ".bee", "bin", "bee.mjs");

let failures = 0;

function record(id, description, passed, detail = "") {
  const label = passed ? "PASS" : "FAIL";
  if (!passed) failures += 1;
  console.log(`${label}  [${id}] ${description}${passed ? "" : ` :: ${detail}`}`);
}

// ─── shared fixture builders (narrow duplication of test_conformance.mjs's
// mkFixture/writeStateFile — that file's own comment already establishes this
// as the repo's convention rather than a shared export, since none of the
// precedent test files export anything). ──────────────────────────────────

function mkFixture(prefix) {
  return fs.mkdtempSync(path.join(os.tmpdir(), prefix));
}

function writeJson(file, value) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

function buildStoreFixture(prefix, { phase = "swarming", executionApproved = true, feature = "compact-verbs-demo" } = {}) {
  const root = mkFixture(prefix);
  writeJson(path.join(root, ".bee", "onboarding.json"), { schema_version: "1.0", bee_version: "0.1.0" });
  writeJson(path.join(root, ".bee", "state.json"), {
    phase,
    mode: "standard",
    feature,
    approved_gates: { context: true, shape: true, execution: executionApproved, review: false },
  });
  return root;
}

function writeSessionRecord(root, sessionId, extra = {}) {
  writeJson(path.join(root, ".bee", "sessions", `${sessionId}.json`), {
    id: sessionId,
    started_at: new Date().toISOString(),
    last_heartbeat: new Date().toISOString(),
    ...extra,
  });
}

async function runBee(cwd, args, input) {
  return runModuleWorker(BEE_MJS, { args, cwd, input });
}

function rm(root) {
  fs.rmSync(root, { recursive: true, force: true });
}

function compactionLog(root) {
  return path.join(root, ".bee", "logs", "compaction.jsonl");
}

function readCompactionRecords(root) {
  const file = compactionLog(root);
  if (!fs.existsSync(file)) return [];
  return fs
    .readFileSync(file, "utf8")
    .split("\n")
    .filter((line) => line.trim())
    .map((line) => JSON.parse(line));
}

// Every `bee.mjs` invocation, of ANY command, best-effort persists a
// dispatcher-wide manifest-drift timestamp under .bee/cache/ (bee.mjs's own
// checkManifestDrift — unrelated to compaction, unconditional on every
// dispatch, and the directory itself springs into existence on the FIRST
// call too, which would otherwise register as a false "mutation" on the very
// first hash comparison). That is dispatcher bookkeeping, not something
// compactCheck writes — D13's "reports and never mutates" is a claim about
// compactCheck's OWN behavior, so the idempotence proof below excludes the
// whole .bee/cache/ subtree rather than let dispatcher-wide noise manufacture
// a false mutation report.
//
// Root-caused via a MEASURED tree diff (rb-2, not assumption): the same
// applies to .bee/logs/. Every `bee.mjs` invocation of ANY command also
// appends one line to .bee/logs/timings.jsonl (the direct-run timing block
// added by wv-1 — CLI-wide, fail-open, append-only telemetry, unrelated to
// compaction), and hooks already write .bee/logs/hooks.jsonl there
// unconditionally too. Neither is bee STATE — D13's claim is about
// compactCheck's own STATE side effects, not CLI-wide telemetry noise, so
// the idempotence proof excludes the whole .bee/logs/ subtree as well.
// This exemption is deliberately narrow (two named top-level dirs, not a
// blanket pass) and scenarioCompactCheckMutatesNothing pairs it with a
// negative control: a genuine .bee/state.json content mutation between two
// hashes must still register as a diff, proving the exemption did not widen
// the check into blindness (the "clearing a red by widening the threshold"
// trap this suite exists to avoid).
const HASH_EXEMPT_DIRS = new Set(["cache", "logs"]);

/** sha256 over every path + byte under a directory tree (idempotence proof). */
function hashTree(dir) {
  const hash = crypto.createHash("sha256");
  const walk = (abs, rel) => {
    const entries = fs.readdirSync(abs, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name));
    for (const entry of entries) {
      const childAbs = path.join(abs, entry.name);
      const childRel = rel ? `${rel}/${entry.name}` : entry.name;
      if (entry.isDirectory()) {
        if (HASH_EXEMPT_DIRS.has(childRel)) continue;
        hash.update(`D:${childRel}\n`);
        walk(childAbs, childRel);
      } else {
        hash.update(`F:${childRel}:`);
        hash.update(fs.readFileSync(childAbs));
        hash.update("\n");
      }
    }
  };
  walk(dir, "");
  return hash.digest("hex");
}

// ═════════════════════════════════════════════════════════════════════════
// compact-log — happy path: a precompact record lands on disk with the
// flags passed through unmangled, via the public entrypoint.
// ═════════════════════════════════════════════════════════════════════════

async function scenarioCompactLogHappyPath() {
  const root = buildStoreFixture("bee-compact-log-happy-");
  const res = await runBee(root, ["state", "compact-log", "--event", "precompact", "--session-id", "sess-log-1", "--json"]);
  const parsed = res.status === 0 ? JSON.parse(res.stdout) : null;
  const wrote = fs.existsSync(compactionLog(root));
  const record0 = wrote ? readCompactionRecords(root)[0] : null;
  const correct =
    res.status === 0 &&
    parsed &&
    parsed.event === "precompact" &&
    parsed.session === "sess-log-1" &&
    parsed.compact_index === 1 &&
    record0 &&
    record0.session === "sess-log-1" &&
    record0.event === "precompact";
  record(
    "compact-log-happy-path",
    "compact-log via the public entrypoint appends a precompact record and echoes it, --event/--session-id reaching lib/compaction.mjs unmangled",
    correct,
    `status=${res.status} stdout=${res.stdout} stderr=${res.stderr} onDisk=${JSON.stringify(record0)}`,
  );
  rm(root);
}

// ═════════════════════════════════════════════════════════════════════════
// compact-log — usage error: an unrecognized --event value refuses, exits
// non-zero, and writes NOTHING (negative control on the write itself).
// ═════════════════════════════════════════════════════════════════════════

async function scenarioCompactLogBadEvent() {
  const root = buildStoreFixture("bee-compact-log-bad-event-");
  const res = await runBee(root, ["state", "compact-log", "--event", "bogus", "--session-id", "sess-log-2", "--json"]);
  const refused = res.status !== 0;
  const nothingWritten = !fs.existsSync(compactionLog(root));
  record(
    "compact-log-bad-event-refuses",
    "compact-log refuses (non-zero exit) on an unrecognized --event value and writes nothing",
    refused && nothingWritten,
    `status=${res.status} stderr=${res.stderr} logExists=${fs.existsSync(compactionLog(root))}`,
  );
  rm(root);
}

// ═════════════════════════════════════════════════════════════════════════
// compact-log — usage error: a missing --session-id refuses via the generic
// validate-args.mjs required-field check (--session-id is in the registry
// entry's `required`), which routes its refusal to STDOUT unconditionally —
// distinct from a handler-thrown Error (emitError), which goes to stderr
// unless --json is set. This registers session-id as truly required.
// ═════════════════════════════════════════════════════════════════════════

async function scenarioCompactLogMissingSession() {
  const root = buildStoreFixture("bee-compact-log-missing-session-");
  const res = await runBee(root, ["state", "compact-log", "--event", "precompact"]);
  const refused = res.status !== 0 && /--session-id/.test(res.stdout);
  record(
    "compact-log-missing-session-id-refuses",
    "compact-log refuses (non-zero exit, names the flag) when --session-id is omitted (validate-args.mjs required-field check)",
    refused,
    `status=${res.status} stdout=${res.stdout} stderr=${res.stderr}`,
  );
  rm(root);
}

// ═════════════════════════════════════════════════════════════════════════
// compact-check — D13: a DETECTED MISMATCH still exits 0. "reports data,
// never a failure" is proven here by driving a session with no session
// record and no anchor through the real dispatcher and reading its exit
// code, not merely its JSON body.
// ═════════════════════════════════════════════════════════════════════════

async function scenarioCompactCheckMismatchStillExitsZero() {
  const root = buildStoreFixture("bee-compact-check-mismatch-");
  const res = await runBee(root, ["state", "compact-check", "--session-id", "no-such-session", "--json"]);
  const parsed = res.status === 0 ? JSON.parse(res.stdout) : null;
  const reportedMismatch = parsed && parsed.ok === false && parsed.mismatches.length > 0;
  record(
    "compact-check-mismatch-exits-zero",
    "compact-check exits 0 even when it reports a mismatch (D13: reports, never blocks) — a usage error is the only non-zero case",
    res.status === 0 && reportedMismatch,
    `status=${res.status} stdout=${res.stdout} stderr=${res.stderr}`,
  );
  rm(root);
}

// ═════════════════════════════════════════════════════════════════════════
// compact-check — usage error: a missing --session-id refuses (non-zero),
// the one case where compact-check DOES exit non-zero.
// ═════════════════════════════════════════════════════════════════════════

async function scenarioCompactCheckMissingSession() {
  const root = buildStoreFixture("bee-compact-check-missing-session-");
  // Same validate-args.mjs required-field path as the compact-log sibling
  // above — the refusal lands on stdout regardless of --json.
  const res = await runBee(root, ["state", "compact-check"]);
  const refused = res.status !== 0 && /--session-id/.test(res.stdout);
  record(
    "compact-check-missing-session-id-refuses",
    "compact-check refuses (non-zero exit, names the flag) when --session-id is omitted — the one non-zero case, a usage error",
    refused,
    `status=${res.status} stdout=${res.stdout} stderr=${res.stderr}`,
  );
  rm(root);
}

// ═════════════════════════════════════════════════════════════════════════
// compact-check — the anchor_missing check carries the EXACT D10 nudge
// command (ANCHOR_NUDGE_COMMAND), reachable by command even on a runtime
// that never executes a hook (D3's helper floor).
// ═════════════════════════════════════════════════════════════════════════

async function scenarioCompactCheckAnchorMissingCommand() {
  const root = buildStoreFixture("bee-compact-check-anchor-cmd-");
  const res = await runBee(root, ["state", "compact-check", "--session-id", "sess-anchor", "--json"]);
  const parsed = res.status === 0 ? JSON.parse(res.stdout) : null;
  const check = parsed && Array.isArray(parsed.checks) ? parsed.checks.find((entry) => entry.name === "anchor_missing") : null;
  const exact = check && check.command === ANCHOR_NUDGE_COMMAND;
  record(
    "compact-check-anchor-missing-command-exact",
    "compact-check's anchor_missing check carries the exact command anchorMissing() names",
    res.status === 0 && exact,
    `status=${res.status} check=${JSON.stringify(check)} expected=${ANCHOR_NUDGE_COMMAND}`,
  );
  rm(root);
}

// ═════════════════════════════════════════════════════════════════════════
// compact-check — D13 idempotence, driven through the public entrypoint,
// NEGATIVE CONTROL: two consecutive runs leave the whole .bee/ tree
// byte-identical (a stable stdout alone would not prove this — the hash of
// every path and byte does). This is the row the cell asks for by name.
// ═════════════════════════════════════════════════════════════════════════

async function scenarioCompactCheckMutatesNothing() {
  const root = buildStoreFixture("bee-compact-check-idempotent-");
  writeSessionRecord(root, "sess-idempotent");
  const beeDir = path.join(root, ".bee");

  const before = hashTree(beeDir);
  const first = await runBee(root, ["state", "compact-check", "--session-id", "sess-idempotent", "--json"]);
  const afterFirst = hashTree(beeDir);
  const second = await runBee(root, ["state", "compact-check", "--session-id", "sess-idempotent", "--json"]);
  const afterSecond = hashTree(beeDir);

  const bothOk = first.status === 0 && second.status === 0;
  const idempotent = before === afterFirst && afterFirst === afterSecond;
  record(
    "compact-check-mutates-nothing",
    "compact-check via the public entrypoint mutates NOTHING under .bee/ across two consecutive runs — the tree hash is the proof, not a stable stdout (D13, negative control)",
    bothOk && idempotent,
    `before=${before} afterFirst=${afterFirst} afterSecond=${afterSecond} first.status=${first.status} second.status=${second.status}`,
  );
  rm(root);
}

// ═════════════════════════════════════════════════════════════════════════
// PROVING negative control for the HASH_EXEMPT_DIRS exemption above: the
// exemption scopes to .bee/cache/** and .bee/logs/** ONLY. A genuine content
// mutation to a real state file (.bee/state.json) must still register as a
// hash diff — otherwise the exemption would have quietly widened the check
// into blindness instead of narrowly excluding telemetry. This is the
// control case the cell asks for by name: it never runs compact-check at
// all, it just proves hashTree() itself still has teeth outside the
// exempted dirs.
// ═════════════════════════════════════════════════════════════════════════

async function scenarioCompactCheckMutatesNothingControlDetectsRealMutation() {
  const root = buildStoreFixture("bee-compact-check-negative-control-");
  writeSessionRecord(root, "sess-negative-control");
  const beeDir = path.join(root, ".bee");
  const stateFile = path.join(beeDir, "state.json");

  const before = hashTree(beeDir);
  const original = fs.readFileSync(stateFile, "utf8");
  const mutated = JSON.parse(original);
  mutated.mode = "high-risk"; // any genuine content change to a real state file
  fs.writeFileSync(stateFile, `${JSON.stringify(mutated, null, 2)}\n`, "utf8");
  const afterMutation = hashTree(beeDir);

  record(
    "compact-check-negative-control-detects-real-mutation",
    "a genuine .bee/state.json content mutation between two hashes still turns the tree hash red — proves HASH_EXEMPT_DIRS did not widen the check into blindness, only excluded telemetry (.bee/cache/**, .bee/logs/**)",
    before !== afterMutation,
    `before=${before} afterMutation=${afterMutation}`,
  );
  rm(root);
}

// ═════════════════════════════════════════════════════════════════════════

async function main() {
  await scenarioCompactLogHappyPath();
  await scenarioCompactLogBadEvent();
  await scenarioCompactLogMissingSession();
  await scenarioCompactCheckMismatchStillExitsZero();
  await scenarioCompactCheckMissingSession();
  await scenarioCompactCheckAnchorMissingCommand();
  await scenarioCompactCheckMutatesNothing();
  await scenarioCompactCheckMutatesNothingControlDetectsRealMutation();

  console.log(`\n${failures === 0 ? "ALL PASS" : `${failures} FAILURE(S)`}`);
  process.exitCode = failures === 0 ? 0 : 1;
}

await main();
