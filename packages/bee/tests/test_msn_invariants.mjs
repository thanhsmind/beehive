#!/usr/bin/env node
// test_msn_invariants.mjs — multisession-native-23 (CONTEXT.md D8 stage 5,
// D9): the ACCEPTANCE suite for the whole multisession-native feature. One
// numbered, named entry per issue #56 invariant 1-15. Same PASS/FAIL/exit-1
// contract as every other suite here — see scripts/lib/test-fixture.mjs.
//
// Design (advisor digest slice5, condition D binding, msn-23's own cell
// text): this file is the INDEX, not a from-scratch reproof. Most
// invariants already have a per-slice test proving them; this suite REUSES
// those by import/execution rather than re-deriving the logic, but a reused
// entry must FAIL LOUD — never a silent pass — when:
//   (a) the underlying suite FILE is missing (fs.existsSync gate), or
//   (b) the SPECIFIC assertion text is no longer present in that file's
//       source (a content-marker regression — renamed/deleted without this
//       suite noticing), or
//   (c) the underlying suite does not pass AS A WHOLE, or
//   (d) that specific assertion's PASS line does not appear in the
//       underlying suite's actual runtime stdout (present in source but not
//       actually exercised/passing under that exact name — e.g. skipped).
// assertReusedInvariant() below is the one helper every reused entry goes
// through so all four checks are uniform. "Suite green" alone is never
// accepted as proof for a SPECIFIC invariant (must_haves prohibition) — (d)
// is what keeps a reused entry honest about proving THAT invariant, not
// merely "the file didn't explode".
//
// Underlying suites are executed at most ONCE each (runExternalSuite's
// cache) even though several invariants below point at the same file —
// re-running an already-proven-green suite per invariant would be pure
// waste.
//
// Fresh tests (gaps found during this cell's own research — the cell text's
// "5, 7, 15 likely" undersold it; 6 and 11 turned out to lack any existing
// race/lock-timing proof either):
//   5  — disjoint-resource lease acquisitions never contend, under a real
//        concurrent Worker barrier (race_lease_child.mjs 'disjoint-resources')
//        PLUS a structural source-scan proving acquireLeases's create path
//        never touches withStoreLock at all.
//   6  — same-resource lease acquisition has exactly one O_EXCL winner,
//        under a real concurrent Worker barrier (race_lease_child.mjs
//        'same-resource') — test_lease_store.mjs only ever proves this
//        sequentially (call, then a second call), never under genuine
//        concurrency.
//   7  — the advisory-allow+warning half is REUSED (test_guards.mjs already
//        proves it byte-for-byte); the merge-time conflict-detection half is
//        FRESH — a real two-worktree git fixture where both sides edit the
//        SAME normal file differently, checkWrite proves the advisory
//        allow+warning at write time, then mergeFeatureWorktree proves the
//        typed MERGE_CONFLICT catches it at merge time. Neither existing
//        suite chains these two facts together (test_worktree_store.mjs has
//        no MERGE_CONFLICT coverage at all — grepped, zero hits).
//   15 — write-capable ops refuse without identity WHERE identity is
//        mandatory today (lease acquire, ownership claim); the
//        workspace-scoped guard denial half is reused from test_guards.mjs's
//        existing deny-class-(c) proof. The legacy-compat carve-outs
//        (unfenced lease renew/release when presentedEpoch is omitted,
//        sessionless calls proceeding untouched, bee.mjs's writeHandoff
//        no-workflow fallback deferred to msn-24) are named explicitly in
//        this entry's own PASS-line output, per the msn-24 truth-chain
//        consistency requirement (advisor digest slice5) — never silently
//        dropped.
//
// Condition D (advisor digest slice5, binding): invariant 12 is UNPROVABLE
// universally — its entry below scopes to the two ENUMERATED long ops
// (queue processing, merge-verify child) plus a grep-guard scoped to the
// verify-child spawn site specifically (not a blanket "zero spawnSync under
// any lock in lib/" claim, which is FALSE today — worktree-store.mjs
// legitimately runs short git plumbing and companion start/teardown via
// spawnSync while holding 'worktree-admin', because those are fast/bounded,
// not the multi-minute op this invariant is about). "Not universal" is
// explicit inline on that entry.
//
// Final summary line format: "N/15 PASS" plus one line per invariant naming
// PASS/FAIL — this is the line msn-25's issue-56 closure quotes verbatim.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { assert } from '../../../scripts/lib/test-fixture.mjs';
import { runModuleWorker } from '../../../scripts/lib/run-module-worker.mjs';
import { acquireLeases } from '../lib/lease-store.mjs';
import { registerWorkspace, claimWriteOwnership } from '../lib/workspace-store.mjs';
import { checkWrite } from '../lib/guards.mjs';
import { defaultState, writeState } from '../lib/state.mjs';
import { createFeatureWorktree, mergeFeatureWorktree } from '../lib/worktree-store.mjs';
import { createSession } from '../lib/claims.mjs';

// Hermeticity (same discipline as every other suite here): never inherit the
// harness's own live session identity.
delete process.env.BEE_AGENT_NAME;
delete process.env.CLAUDE_CODE_SESSION_ID;
delete process.env.BEE_SESSION_ID;

const __dirname = path.dirname(fileURLToPath(import.meta.url));
// tests -> bee -> packages -> repo root (same 3-level convention every
// sibling suite here uses for its scripts/lib import).
const REPO_ROOT = path.resolve(__dirname, '../../..');
const LIB_DIR = path.join(REPO_ROOT, 'packages', 'bee', 'lib');

// ─── invariant-level tracking (the required "N/15" final line) ────────────
//
// The shared check()/printSummaryAndExit() from test-fixture.mjs count
// CHECK ROWS, not INVARIANTS — invariant 12 alone needs three rows (queue
// processing, merge-verify child, the grep-guard). msn-25's issue-56
// closure needs a line naming all 15 invariants by number, not a row count.
// invariantCheck() below keeps the exact same "PASS  <name>" / "FAIL  <name>"
// console contract check() uses (so this suite still reads like every
// sibling here, and so a FUTURE suite could content-marker-check THIS one
// the same way this suite content-marker-checks its own reused entries) but
// additionally rolls each row up under its invariant number.

const rows = [];
const invariantStatus = new Map();

async function invariantCheck(n, name, fn) {
  try {
    const result = fn();
    if (result && typeof result.then === 'function') await result;
    console.log(`PASS  ${name}`);
    rows.push({ name, passed: true, invariant: n });
    invariantStatus.set(n, invariantStatus.get(n) ?? true);
  } catch (error) {
    console.log(`FAIL  ${name}`);
    console.log(`      ${error instanceof Error ? error.message : error}`);
    rows.push({ name, passed: false, invariant: n });
    invariantStatus.set(n, false);
  }
}

// ─── the reuse-index machinery ──────────────────────────────────────────────

const externalSuiteCache = new Map();

/**
 * Runs an underlying suite file at most once; caches {missing, status,
 * stdout, stderr}. Deliberately a REAL child process (node:child_process),
 * never runModuleWorker/worker_threads: several of the suites this index
 * reuses (test_cli_state.mjs, test_claims.mjs) themselves spawn their OWN
 * nested Worker via runModuleWorker (to drive bee.mjs's real CLI, or their
 * own race-child harness) — running such a suite INSIDE another eval-mode
 * Worker double-nests that pattern and silently breaks it (every
 * spawnSync-backed helper inside the nested worker returns null; verified
 * empirically while building this suite — the same file passes 100% clean
 * as a top-level `node <file>` process). A real child process is also the
 * SAME mechanism run_verify.mjs itself uses for every suite (spawn, not
 * worker threads — scripts/run_verify.mjs's runOne()), so this is the more
 * faithful "actually run it the way it's normally run" proof anyway.
 */
async function runExternalSuite(relPathFromRepoRoot, { timeout = 120_000 } = {}) {
  if (externalSuiteCache.has(relPathFromRepoRoot)) return externalSuiteCache.get(relPathFromRepoRoot);
  const abs = path.join(REPO_ROOT, relPathFromRepoRoot);
  if (!fs.existsSync(abs)) {
    const missing = { missing: true, status: null, stdout: '', stderr: '' };
    externalSuiteCache.set(relPathFromRepoRoot, missing);
    return missing;
  }
  const result = spawnSync(process.execPath, [abs], { cwd: REPO_ROOT, encoding: 'utf8', timeout, env: process.env, maxBuffer: 64 * 1024 * 1024 });
  const entry = {
    missing: false,
    status: result.error && result.error.code === 'ETIMEDOUT' ? null : result.status,
    stdout: result.stdout || '',
    stderr: `${result.stderr || ''}${result.error ? `\n${result.error.stack || result.error.message || result.error}` : ''}`,
  };
  externalSuiteCache.set(relPathFromRepoRoot, entry);
  return entry;
}

/**
 * sourceHasMarker(source, marker) — a JS string LITERAL in source escapes
 * an apostrophe as `\'` (e.g. `heartbeat\'s` inside a single-quoted check()
 * name); the RUNTIME printed text (after JS evaluates the escape) has no
 * backslash (`heartbeat's`). Markers in this file are written as the
 * runtime/printed text (matching what the PASS line actually shows), so the
 * source-content check tries both forms rather than assuming source bytes
 * and runtime text are identical.
 */
function sourceHasMarker(source, marker) {
  return source.includes(marker) || source.replace(/\\'/g, "'").includes(marker);
}

/**
 * assertReusedInvariant(n, name, relPath, marker, opts) — registers ONE
 * check() entry that fails loud (never a silent pass) on every one of the
 * four gaps documented in this file's header. `passPrefix` accounts for the
 * two non-check()-based frameworks this suite also reuses (test-fixture.mjs's
 * check() prints "PASS  " — two spaces; scripts/tests/test_worktree_merge_queue.mjs's
 * own record() prints "PASS " — one space).
 */
async function assertReusedInvariant(n, name, relPathFromRepoRoot, marker, { passPrefix = 'PASS  ' } = {}) {
  await invariantCheck(n, `invariant ${n}: ${name} [reused from ${relPathFromRepoRoot}]`, async () => {
    const abs = path.join(REPO_ROOT, relPathFromRepoRoot);
    assert(
      fs.existsSync(abs),
      `FAIL LOUD: underlying suite "${relPathFromRepoRoot}" is MISSING — invariant ${n} cannot be verified by reuse. This is not a silent pass.`,
    );
    const source = fs.readFileSync(abs, 'utf8');
    assert(
      sourceHasMarker(source, marker),
      `FAIL LOUD: the specific assertion text for invariant ${n} is no longer present in "${relPathFromRepoRoot}" (renamed or removed without this index noticing). Looked for: ${JSON.stringify(marker)}`,
    );
    const result = await runExternalSuite(relPathFromRepoRoot);
    assert(!result.missing, `FAIL LOUD: underlying suite "${relPathFromRepoRoot}" vanished between the existence check and execution.`);
    assert(
      result.status === 0,
      `FAIL LOUD: underlying suite "${relPathFromRepoRoot}" did not pass as a whole (exit ${result.status}) — invariant ${n} cannot be trusted green off a red suite. stderr: ${(result.stderr || '').slice(0, 800)}`,
    );
    assert(
      result.stdout.includes(passPrefix + marker),
      `FAIL LOUD: invariant ${n}'s specific assertion did not print as PASSING in "${relPathFromRepoRoot}"'s actual output (present in source, but its runtime PASS line is missing — "suite green" alone is never accepted as proof of THIS invariant). Looked for: ${JSON.stringify(passPrefix + marker)}`,
    );
  });
}

// ─── invariants 1-3: workflow-store lock isolation (test_cli_state.mjs) ────

const CLI_STATE = 'packages/bee/tests/test_cli_state.mjs';

await assertReusedInvariant(
  1,
  'concurrent planning: two workflows plan without sharing a lock',
  CLI_STATE,
  "multisession-native-10 (invariant 1/2, C4): two DIFFERENT features' workflow-routed mutations never contend on a shared lock — holding feature A's workflow lock never blocks feature B's mutation (deterministic seam proof, msn-6-style)",
);

await assertReusedInvariant(
  2,
  'phase change isolation across workflows',
  CLI_STATE,
  "multisession-native-10 (invariant 1/2, C4): a lane mutation with a live workflow record never needs the shared 'state' lock — held externally for the whole call, the CLI mutation still completes near-instantly",
);

await assertReusedInvariant(
  3,
  'gate isolation cross-workflow: a plan-rev bump on W1 never touches W2',
  CLI_STATE,
  'multisession-native-9 (C2, binding, RED-then-GREEN): plan-rev bump flips the PROJECTED execution boolean for the bumped workflow only — a claim refuses on W1; W2 is untouched (invariant 3)',
);

// ─── invariant 4: heartbeat never erases a lane/workflow binding ──────────

const TEST_CLAIMS = 'packages/bee/tests/test_claims.mjs';

await assertReusedInvariant(
  4,
  'heartbeat never erases a lane/workflow binding',
  TEST_CLAIMS,
  "race: heartbeat-vs-bindSessionLane (D10a, forced interleaving) — a bind forced into heartbeat's read-modify-write window is never clobbered by the stale heartbeat write",
);

// ─── invariant 5: disjoint-resource cells never contend (FRESH) ──────────

const raceLeaseChild = path.join(__dirname, 'race_lease_child.mjs');

await invariantCheck(5, 'invariant 5: disjoint-resource lease acquisitions never contend — a held lease on one resource never blocks acquiring an unrelated one, under genuine concurrent execution [FRESH]', async () => {
  assert(fs.existsSync(raceLeaseChild), `FAIL LOUD: race_lease_child.mjs is missing — invariant 5 cannot be verified.`);
  const result = await runModuleWorker(raceLeaseChild, { cwd: REPO_ROOT, args: ['disjoint-resources'], timeout: 60_000 });
  assert(
    result.status === 0,
    `disjoint-resources race must exit 0, got ${result.status}. stdout: ${result.stdout} stderr: ${result.stderr}`,
  );
  assert(
    result.stdout.includes('PASS  disjoint-resources:'),
    `expected the race child's own PASS summary line, got: ${result.stdout}`,
  );

  // Structural seam-proof (C4 pattern, same static-source-scan discipline as
  // workflow-store.mjs:430 / lease-store.mjs's own module header): the
  // acquireLeases CREATE path never touches withStoreLock at all — the only
  // exclusion primitive on that path is per-resource O_EXCL file creation,
  // so two disjoint resources structurally cannot ever contend on a shared
  // lock, regardless of timing.
  const leaseStoreSource = fs.readFileSync(path.join(LIB_DIR, 'lease-store.mjs'), 'utf8');
  const fnStart = leaseStoreSource.indexOf('export function acquireLeases(');
  assert(fnStart !== -1, 'lease-store.mjs no longer exports acquireLeases at the expected shape — seam-proof cannot locate it.');
  const nextExportIdx = leaseStoreSource.indexOf('\nexport ', fnStart + 1);
  const fnBody = nextExportIdx === -1 ? leaseStoreSource.slice(fnStart) : leaseStoreSource.slice(fnStart, nextExportIdx);
  assert(
    !/withStoreLock\(/.test(fnBody),
    `seam-proof failed: acquireLeases's body now calls withStoreLock — disjoint resources could contend on a shared lock. Body:\n${fnBody}`,
  );
});

// ─── invariant 6: same-workspace same-file lease has exactly one winner (FRESH) ──

await invariantCheck(6, 'invariant 6: same-resource lease acquisition — exactly one O_EXCL winner under genuine concurrent execution [FRESH]', async () => {
  assert(fs.existsSync(raceLeaseChild), `FAIL LOUD: race_lease_child.mjs is missing — invariant 6 cannot be verified.`);
  const result = await runModuleWorker(raceLeaseChild, { cwd: REPO_ROOT, args: ['same-resource'], timeout: 60_000 });
  assert(
    result.status === 0,
    `same-resource race must exit 0, got ${result.status}. stdout: ${result.stdout} stderr: ${result.stderr}`,
  );
  assert(
    result.stdout.includes('PASS  same-resource:'),
    `expected the race child's own PASS summary line, got: ${result.stdout}`,
  );
});

// ─── invariant 7: cross-worktree same-file — advisory allow + merge-fence catch ──

const TEST_GUARDS = 'packages/bee/tests/test_guards.mjs';

function writeHoldsLedgerFor(dir, holds) {
  const runtime = path.join(dir, '.bee', 'runtime');
  fs.mkdirSync(runtime, { recursive: true });
  fs.writeFileSync(path.join(runtime, 'cross-worktree-holds.json'), JSON.stringify({ holds }, null, 2));
}

function git(cwd, args, { allowFailure = false } = {}) {
  const result = spawnSync('git', args, { cwd, encoding: 'utf8' });
  if (!allowFailure && result.status !== 0) {
    throw new Error(`git ${args.join(' ')} failed (${result.status}): ${result.stderr || result.stdout}`);
  }
  return result;
}

await invariantCheck(7, 'invariant 7: cross-worktree same-normal-file write is allowed WITH the advisory warning surfaced (reused: guards.mjs advisory path), AND merge-time conflict detection engages when the two sides genuinely diverge (FRESH: worktree-store.mjs merge fence) [reused + FRESH]', async () => {
  // ── half A (reused via content-marker + runtime-marker, guards.mjs advisory path) ──
  const guardsAbs = path.join(REPO_ROOT, TEST_GUARDS);
  const advisoryMarker =
    "checkWrite (xwh-4/msn-14): a foreign checkout's hold on a NORMAL path is advisory — allow:true with a warning naming the holding checkout, its feature, the expiry, and the merge-time consequence — phase-independent (swarming with execution approved)";
  assert(fs.existsSync(guardsAbs), `FAIL LOUD: underlying suite "${TEST_GUARDS}" is MISSING — invariant 7's advisory half cannot be verified.`);
  const guardsSource = fs.readFileSync(guardsAbs, 'utf8');
  assert(
    sourceHasMarker(guardsSource, advisoryMarker),
    `FAIL LOUD: the advisory-warning assertion is no longer present in "${TEST_GUARDS}" — invariant 7's reused half is broken.`,
  );
  const guardsResult = await runExternalSuite(TEST_GUARDS);
  assert(!guardsResult.missing && guardsResult.status === 0, `FAIL LOUD: "${TEST_GUARDS}" did not pass as a whole (exit ${guardsResult.status}).`);
  assert(
    guardsResult.stdout.includes(`PASS  ${advisoryMarker}`),
    `FAIL LOUD: the advisory-warning check's PASS line is missing from "${TEST_GUARDS}"'s actual output.`,
  );

  // Also demonstrate the same fact directly, self-contained, without
  // depending only on the other file's fixture helpers being unchanged —
  // the SAME technique test_guards.mjs uses (write the cross-worktree holds
  // ledger by hand, call checkWrite from an ordinary checkout so topology
  // resolves holder='main', a 'wt-featx' entry is foreign).
  const advisoryDir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-msn23-inv7-advisory-'));
  try {
    fs.mkdirSync(path.join(advisoryDir, '.bee'), { recursive: true });
    fs.writeFileSync(path.join(advisoryDir, '.bee', 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.1.0' }, null, 2));
    writeHoldsLedgerFor(advisoryDir, [
      {
        path: 'src/shared/normal.ts',
        holder: 'wt-shared',
        feature: 'feat-shared',
        session: null,
        cell: 'shared-1',
        ttl_seconds: 3600,
        mirrored_at: new Date().toISOString(),
        released_at: null,
      },
    ]);
    const state = { ...defaultState(), phase: 'swarming', approved_gates: { context: true, shape: true, execution: true, review: false } };
    const verdict = checkWrite(advisoryDir, state, 'src/shared/normal.ts', 'net-writer');
    assert(verdict.allow === true, `cross-worktree same-normal-file write must be ALLOWED, got ${JSON.stringify(verdict)}`);
    assert(typeof verdict.warning === 'string' && verdict.warning.length > 0, `the allow must never be silent — a warning is required, got ${JSON.stringify(verdict)}`);
  } finally {
    fs.rmSync(advisoryDir, { recursive: true, force: true });
  }

  // ── half B (FRESH: real two-worktree git fixture, genuine textual conflict) ──
  const mainRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-msn23-inv7-conflict-'));
  try {
    git(mainRoot, ['init', '-b', 'main']);
    git(mainRoot, ['config', 'user.email', 'bee@example.invalid']);
    git(mainRoot, ['config', 'user.name', 'Bee Test']);
    fs.writeFileSync(path.join(mainRoot, '.gitignore'), '.bee/\n');
    fs.writeFileSync(path.join(mainRoot, 'shared.txt'), 'base\n');
    git(mainRoot, ['add', '.gitignore', 'shared.txt']);
    git(mainRoot, ['commit', '-m', 'base']);

    const created = await createFeatureWorktree(mainRoot, { feature: 'inv7-conflict' });
    // The worktree side edits the SAME normal file...
    fs.writeFileSync(path.join(created.worktreeRoot, 'shared.txt'), 'worktree-side edit\n');
    git(created.worktreeRoot, ['add', 'shared.txt']);
    git(created.worktreeRoot, ['commit', '-m', 'worktree edit']);

    // ...while MAIN, independently, edits the same lines differently — the
    // exact "two checkouts touched the same normal file" scenario the
    // advisory allow above green-lights at write time.
    fs.writeFileSync(path.join(mainRoot, 'shared.txt'), 'main-side edit\n');
    git(mainRoot, ['add', 'shared.txt']);
    git(mainRoot, ['commit', '-m', 'main edit']);

    const preHead = git(mainRoot, ['rev-parse', 'HEAD']).stdout.trim();
    const merged = await mergeFeatureWorktree(mainRoot, { id: created.id });
    assert(
      merged.ok === false && merged.code === 'MERGE_CONFLICT',
      `a genuine textual conflict on a shared normal file must be caught at merge time as typed MERGE_CONFLICT, got ${JSON.stringify(merged)}`,
    );
    const headAfter = git(mainRoot, ['rev-parse', 'HEAD']).stdout.trim();
    assert(headAfter === preHead, 'main HEAD must be byte-unchanged after an aborted conflicted merge');
    assert(!fs.existsSync(path.join(mainRoot, '.git', 'MERGE_HEAD')), 'no MERGE_HEAD may linger after the abort');
  } finally {
    git(mainRoot, ['worktree', 'prune'], { allowFailure: true });
    fs.rmSync(mainRoot, { recursive: true, force: true });
  }
});

// ─── invariant 8: exclusive resources hard-block cross-worktree ──────────

await assertReusedInvariant(
  8,
  'exclusive resources (migrations, lockfiles, release-manifest.json, .bee/onboarding.json, generated/) hard-block cross-worktree',
  TEST_GUARDS,
  'checkWrite (xwh-4/msn-14): a foreign hold on an EXCLUSIVE-marked resource (built-in defaults: migrations, lockfiles, release-manifest.json, .bee/onboarding.json, generated/) still hard-denies cross-worktree, same reason shape as before this cell',
);

// ─── invariant 9: handoff isolation ────────────────────────────────────────

await assertReusedInvariant(
  9,
  "workflow A's open handoff never blocks workflow B, and A's own handoff is untouched by B's mailbox activity",
  CLI_STATE,
  "invariant 9 (D9, must-have truth): workflow A paused with an open handoff never blocks workflow B from starting, working, or adopting its OWN handoff — and A's own handoff is completely untouched by any of B's mailbox activity",
);

// ─── invariant 10: epoch fencing ───────────────────────────────────────────

await assertReusedInvariant(
  10,
  'renewClaimTTL refuses a stale presented epoch (CLAIM_FENCE_STALE)',
  TEST_CLAIMS,
  'invariant 10: renewClaimTTL refuses typed CLAIM_FENCE_STALE when the presented epoch is behind current fence_epoch; the current epoch succeeds',
);

// ─── invariant 11: hooks never wait on store locks ────────────────────────

await assertReusedInvariant(
  11,
  'checkWrite (the write-guard hook\'s core logic) never acquires any store lock — a write resolves promptly even while another process genuinely holds a lock file',
  TEST_GUARDS,
  'checkWrite (msn-21): the hook posture regression — checkWrite never acquires ANY store lock (readWorkspace/readSession are plain reads), so a write resolves promptly even while another process genuinely holds the workspace lock file',
);

// ─── invariant 12: no long op holds a filesystem lock (condition D: scoped, "not universal") ──

const TEST_INTEGRATION_QUEUE = 'packages/bee/tests/test_integration_queue.mjs';
const TEST_WORKTREE_MERGE_QUEUE = 'scripts/tests/test_worktree_merge_queue.mjs';

await assertReusedInvariant(
  12,
  "queue processing never holds the 'integration-queue' store lock while a merge is in flight",
  TEST_INTEGRATION_QUEUE,
  'runThroughQueue: the "integration-queue" store lock is never held while runMerge is in flight (invariant 12, this module\'s own boundary)',
);

await assertReusedInvariant(
  12,
  "the merge-verify child (a genuine multi-minute op) runs with neither 'worktree-admin' nor 'integration-queue' held",
  TEST_WORKTREE_MERGE_QUEUE,
  "(b) 'integration-queue' lock is ABSENT while merge A's verify child runs (new queue-lock proof, invariant 12)",
  { passPrefix: 'PASS ' },
);

await invariantCheck(
  12,
  "invariant 12: grep-guard — the enumerated long op (worktree-store.mjs's runVerifyChild, the multi-minute verify child) never uses spawnSync. NOT UNIVERSAL: this does NOT claim zero withStoreLock+spawnSync pairs anywhere in lib/ — short git plumbing (createFeatureWorktreeLocked's runGit calls) and companion process start/teardown legitimately run via spawnSync while 'worktree-admin' is held, because those are fast/bounded, not the long op this invariant is about. [FRESH]",
  () => {
    const worktreeStoreSource = fs.readFileSync(path.join(LIB_DIR, 'worktree-store.mjs'), 'utf8');
    const fnMarker = 'function runVerifyChild(';
    const fnStart = worktreeStoreSource.indexOf(fnMarker);
    assert(fnStart !== -1, 'worktree-store.mjs no longer defines runVerifyChild at the expected shape — grep-guard cannot locate it.');
    const nextFnIdx = worktreeStoreSource.indexOf('\nfunction ', fnStart + 1);
    const nextExportIdx = worktreeStoreSource.indexOf('\nexport ', fnStart + 1);
    const candidates = [nextFnIdx, nextExportIdx].filter((i) => i !== -1);
    const boundary = candidates.length ? Math.min(...candidates) : worktreeStoreSource.length;
    const fnBody = worktreeStoreSource.slice(fnStart, boundary);
    assert(/spawn\(/.test(fnBody) && !/spawnSync\(/.test(fnBody), `runVerifyChild must invoke the verify command via async spawn, never spawnSync. Body:\n${fnBody}`);
  },
);

// ─── invariants 13/14: overview rebuilds fully; delete loses nothing ─────

const TEST_STATE_PROJECTION = 'packages/bee/tests/test_state_projection.mjs';

await assertReusedInvariant(
  13,
  'overview (.bee/state.json) rebuilds fully from records after deletion',
  TEST_STATE_PROJECTION,
  'invariant 13/14: deleting .bee/state.json and rebuilding it (no overrides) reproduces byte-identical D1-owned content — "overview rebuilds fully from records"',
);

await assertReusedInvariant(
  14,
  'deleting a projection (lane file) loses nothing — rebuild reproduces it byte-identically',
  TEST_STATE_PROJECTION,
  'invariant 13/14: deleting .bee/lanes/<feature>.json and rebuilding it reproduces byte-identical content — "deleting a projection loses nothing"',
);

// ─── invariant 15: identity mandatory where it matters today (FRESH + reused) ──

await invariantCheck(
  15,
  'invariant 15: write-capable ops refuse without session_id+workflow_id+workspace_id WHERE identity is mandatory today (lease acquire, ownership claim, workspace-scoped guard denial) — legacy-compat carve-outs named explicitly, never silently dropped [FRESH + reused]',
  async () => {
    // (a) lease acquire, FRESH: acquireLeases refuses when session_id,
    // workflow_id, or workspace_id is missing — LEASE_INVALID_REQUEST, never
    // a silent fallback.
    const leaseRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-msn23-inv15-lease-'));
    try {
      for (const missingField of ['workflow_id', 'session_id', 'workspace_id']) {
        const req = { type: 'path', id: 'inv15/lease-target.txt', mode: 'write', workflow_id: 'wf', session_id: 'sess', workspace_id: 'main', epoch: 1 };
        delete req[missingField];
        let threw = null;
        try {
          acquireLeases(leaseRoot, [req]);
        } catch (error) {
          threw = error;
        }
        assert(threw && threw.code === 'LEASE_INVALID_REQUEST', `acquireLeases must refuse typed LEASE_INVALID_REQUEST when "${missingField}" is missing, got ${threw ? threw.code || threw.message : '(no throw)'}`);
        assert(new RegExp(missingField).test(threw.message), `the refusal must name the missing field "${missingField}", got: ${threw.message}`);
      }
    } finally {
      fs.rmSync(leaseRoot, { recursive: true, force: true });
    }

    // (b) ownership claim, FRESH: claimWriteOwnership refuses without a sessionId.
    const wsRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-msn23-inv15-ws-'));
    try {
      let threw = null;
      try {
        await claimWriteOwnership(wsRoot, 'main', null);
      } catch (error) {
        threw = error;
      }
      assert(threw && threw.code === 'WORKSPACE_INVALID_SESSION', `claimWriteOwnership must refuse typed WORKSPACE_INVALID_SESSION with no sessionId, got ${threw ? threw.code || threw.message : '(no throw)'}`);

      // Legacy-compat carve-out, named explicitly: a session-scoped
      // acquire/renew/release omitting presentedEpoch is BYTE-UNCHANGED
      // legacy behavior (lease-store.mjs's own module header) — never
      // fenced, and this cell does not tighten that. This is not a bug,
      // it is a documented exclusion.
      await registerWorkspace(wsRoot, { id: 'main', type: 'main', root: wsRoot });
      const owned = await claimWriteOwnership(wsRoot, 'main', 'sess-owner');
      assert(owned.ok === true && owned.owner === true, 'sanity: a real sessionId does successfully claim ownership (proves the refusal above was about identity, not a broken fixture)');
    } finally {
      fs.rmSync(wsRoot, { recursive: true, force: true });
    }

    // (c) workspace-scoped guard denial — REUSED via content-marker +
    // runtime-marker from test_guards.mjs's existing deny-class-(c) proof
    // (msn-21): a non-owner session's write into a workspace another LIVE
    // session already write-owns is denied.
    const guardDenyMarker =
      "checkWrite (msn-21, deny class (c)): a non-owner session refuses a write in a workspace another LIVE session already write-owns — named in the reason, allowed once the owner is stale, allowed for the owner itself, allowed when unregistered";
    const guardsAbs = path.join(REPO_ROOT, TEST_GUARDS);
    assert(fs.existsSync(guardsAbs), `FAIL LOUD: underlying suite "${TEST_GUARDS}" is MISSING — invariant 15's guard-denial half cannot be verified.`);
    const guardsSource = fs.readFileSync(guardsAbs, 'utf8');
    assert(sourceHasMarker(guardsSource, guardDenyMarker), `FAIL LOUD: the workspace-ownership-deny assertion is no longer present in "${TEST_GUARDS}".`);
    const guardsResult = await runExternalSuite(TEST_GUARDS);
    assert(!guardsResult.missing && guardsResult.status === 0, `FAIL LOUD: "${TEST_GUARDS}" did not pass as a whole (exit ${guardsResult.status}).`);
    assert(guardsResult.stdout.includes(`PASS  ${guardDenyMarker}`), `FAIL LOUD: the workspace-ownership-deny check's PASS line is missing from "${TEST_GUARDS}"'s actual output.`);

    // (d) the same guard denial, self-contained (does not depend solely on
    // the other file's private fixture helpers staying unchanged):
    // sessionless calls proceed UNTOUCHED — the legacy carve-out named
    // explicitly, never silently skipped.
    const guardRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-msn23-inv15-guard-'));
    try {
      fs.mkdirSync(path.join(guardRoot, '.bee'), { recursive: true });
      fs.writeFileSync(path.join(guardRoot, '.bee', 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.1.0' }, null, 2));
      // 'validating' + execution approved (never 'idle' — the intake gate
      // would block first; never 'swarming' — the workspace-ownership deny
      // class explicitly excludes it, guards.mjs's own `phase !== 'swarming'`
      // condition, "swarming reservations keep their existing guard
      // branches") — same placement discipline test_guards.mjs's own
      // ownershipRepo() fixture uses, so the ownership check is what
      // actually decides here, not an earlier gate branch.
      //
      // Written to DISK, not just held in memory: checkWrite's
      // resolveWriteRecord only uses the in-memory `state` param when
      // sessionId is absent (source:'default' passthrough). A
      // session-identified call instead goes through resolvePipeline(),
      // which falls back to readState(root) off .bee/state.json on disk the
      // moment the session has no lane binding (readSession returns null
      // for a session that was never created here) — so the on-disk record
      // is what actually governs the intruder's write below.
      const state = { ...defaultState(), phase: 'validating', approved_gates: { context: true, shape: true, execution: true, review: false } };
      writeState(guardRoot, state);
      // checkWorkspaceOwnership only treats an owner as a live blocker when
      // its SESSION RECORD exists and its heartbeat is fresh (a
      // never-created or gone-stale owner never blocks — "dead owner never
      // blocks" semantics, guards.mjs's own comment). A real live owner
      // needs an actual createSession() first, same as test_guards.mjs's
      // ownershipRepo()-adjacent fixtures do.
      createSession(guardRoot, { id: 'sess-live-owner' });
      await registerWorkspace(guardRoot, { id: 'main', type: 'main', root: guardRoot });
      await claimWriteOwnership(guardRoot, 'main', 'sess-live-owner');

      const sessionlessWrite = checkWrite(guardRoot, state, 'src/whatever.ts', null, {});
      assert(
        sessionlessWrite.allow === true,
        `LEGACY CARVE-OUT (documented, not tightened by this cell): a sessionless write must proceed untouched even when a live owner exists, got ${JSON.stringify(sessionlessWrite)}`,
      );

      const intruderWrite = checkWrite(guardRoot, state, 'src/whatever.ts', null, { sessionId: 'sess-intruder' });
      assert(
        intruderWrite.allow === false && intruderWrite.kind === 'workspace-ownership',
        `a session-identified write into a workspace a DIFFERENT live session owns must be denied, got ${JSON.stringify(intruderWrite)}`,
      );
    } finally {
      fs.rmSync(guardRoot, { recursive: true, force: true });
    }

    console.log(
      'invariant 15: LEGACY-COMPAT CARVE-OUTS (named explicitly, deliberately NOT tightened by this cell — tightening is post-feature work, D9): ' +
        '(1) lease renewLease/releaseLease omitting presentedEpoch is byte-unchanged UNFENCED legacy behavior; ' +
        '(2) a sessionless call to checkWrite/applyWritePolicy proceeds untouched, even into a live-owned workspace; ' +
        "(3) bee.mjs's writeHandoff no-workflow fallback (bee.mjs:3357) STAYS one more release per advisor digest slice5 condition E — reclassified as the projection writer, not removed; deferred to msn-24's own cell, consistent with msn-25's issue-56 closure disclosure.",
    );
  },
);

// ─── final summary: the verbatim line msn-25's issue-56 closure quotes ───
//
// Distinct from a check-row count (test-fixture.mjs's own printSummaryAndExit,
// intentionally not used here) — this rolls each row up under its INVARIANT
// number, since invariant 12 alone has three rows. Exit code is still the
// standard 0=pass/nonzero=fail contract run_verify.mjs relies on (it only
// ever reads the child's exit code, never parses suite stdout — confirmed
// against scripts/run_verify.mjs's runOne()).

const ALL_INVARIANTS = Array.from({ length: 15 }, (_, i) => i + 1);
const failedInvariants = ALL_INVARIANTS.filter((n) => invariantStatus.get(n) !== true);
const passedInvariants = ALL_INVARIANTS.filter((n) => invariantStatus.get(n) === true);
const missingInvariants = ALL_INVARIANTS.filter((n) => !invariantStatus.has(n));

if (missingInvariants.length > 0) {
  console.log(`\nSUITE BUG: invariant(s) ${missingInvariants.join(',')} never registered a single check() row — this suite's own coverage is incomplete.`);
}

if (failedInvariants.length === 0 && missingInvariants.length === 0) {
  console.log(`\n15/15 PASS: invariants ${ALL_INVARIANTS.join(',')} all green.`);
} else {
  console.log(
    `\n${passedInvariants.length}/15 PASS — FAILED: ${failedInvariants.length ? failedInvariants.join(',') : '(none)'}${
      missingInvariants.length ? `; MISSING: ${missingInvariants.join(',')}` : ''
    }`,
  );
}

const totalPassed = rows.filter((r) => r.passed).length;
const totalFailed = rows.filter((r) => !r.passed).length;
console.log(`${totalPassed} passed, ${totalFailed} failed`);
process.exit(failedInvariants.length > 0 || missingInvariants.length > 0 ? 1 : 0);
