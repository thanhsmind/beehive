#!/usr/bin/env node
// test_state.mjs — state.mjs lib contract tests (state/readStateStrict/lanes),
// split out of test_lib.mjs (cs-2b) to shrink the monolith. Same PASS/FAIL/
// exit-1 contract as every other suite here — see scripts/lib/test-fixture.mjs.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  makeTempRepo,
  makeCell,
  check,
  assert,
  assertThrows,
  assertRejects,
  printSummaryAndExit,
} from '../../../scripts/lib/test-fixture.mjs';
import {
  findRepoRoot,
  resolveRoots,
  resolveContext,
  WorktreeLinkInvalidError,
  readState,
  readStateStrict,
  gateApproved,
  startFeature,
  readConfig,
  localConfigPath,
  mergeConfigOverlay,
} from '../lib/state.mjs';
import { readBacklogCounts } from '../lib/backlog.mjs';
import { reserve } from '../lib/reservations.mjs';
// multisession-native-16: reservations.mjs is now a shim over lease-store.mjs
// — a test that needs to force a reservation into the past (to prove
// startFeature's expired-hold-never-blocks precondition) backdates the
// underlying LEASE directly via renewLease, since `.bee/reservations.json`
// is only a rebuildable projection nothing here reads live anymore.
import { renewLease } from '../lib/lease-store.mjs';
import { createSession, heartbeatSession, claimCellFile, sessionPath } from '../lib/claims.mjs';
import { listWorkflows, updateWorkflow } from '../lib/workflow-store.mjs';
import { acquireStoreLockOnceSync } from '../lib/lock.mjs';
// fsh-3 (lane store): namespace imports so a not-yet-implemented export fails
// its own row ("… is not a function") instead of crashing the whole module
// graph at import time — the RED-first evidence stays per-row.
import * as laneStore from '../lib/state.mjs';
import * as laneBinding from '../lib/claims.mjs';
import { shouldInject, markInjected } from '../lib/inject.mjs';
import { readJson, writeJsonAtomic } from '../lib/fsutil.mjs';

// Hermeticity (hardening-1-7-10 D1, defense in depth): this suite must never
// inherit the harness's own session identity. run_verify.mjs already scrubs
// both vars for every child suite it spawns; deleting them again here means
// a bare `node packages/bee/tests/test_state.mjs`, run directly
// from inside a live Claude Code or bee session, is equally hermetic instead
// of silently depending on whatever session happens to be live.
delete process.env.CLAUDE_CODE_SESSION_ID;
delete process.env.BEE_SESSION_ID;

const root = makeTempRepo();

// Self-containment fix (cs-2b split): makeStateRepo is defined in test_lib.mjs's
// "bee.mjs state CLI" section (now test_cli_state.mjs, a different file) and
// makeCellFile is defined in its "bee.mjs state start-feature" section (same
// new file) — both were only reachable here via function-declaration hoisting
// across the whole monolith. The lanes rows below need both. Verbatim copies,
// same shape, same behavior, zero check weakened.
function makeStateRepo(prefix) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  return dir;
}

function makeCellFile(dir, id, extra = {}) {
  fs.mkdirSync(path.join(dir, '.bee', 'cells'), { recursive: true });
  const cell = {
    id,
    feature: 'old-feature',
    title: `Cell ${id}`,
    lane: 'tiny',
    status: 'open',
    deps: [],
    action: 'do it',
    verify: 'node -e "process.exit(0)"',
    trace: {},
    ...extra,
  };
  writeJsonAtomic(path.join(dir, '.bee', 'cells', `${id}.json`), cell);
  return cell;
}

// ─── state ──────────────────────────────────────────────────────────────────

await check('readJson strips a leading UTF-8 BOM (GitHub #9) and warns instead of silently swallowing malformed JSON (GitHub #13)', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-readjson-bom-'));
  // 1. A BOM-prefixed file (what PowerShell `Set-Content -Encoding UTF8` writes)
  //    must parse cleanly rather than throwing "Unexpected token '﻿'".
  const bomFile = path.join(dir, 'bom.json');
  fs.writeFileSync(bomFile, `\uFEFF${JSON.stringify({ a: 1 })}`, 'utf8');
  const parsed = readJson(bomFile, null);
  assert(parsed && parsed.a === 1, `BOM-prefixed JSON should parse, got ${JSON.stringify(parsed)}`);
  // 2. Malformed JSON returns the fallback AND emits a stderr warning naming the
  //    file — a corrupt config must never be silently indistinguishable from an
  //    absent one.
  const badFile = path.join(dir, 'bad.json');
  fs.writeFileSync(badFile, '{ not valid', 'utf8');
  const warnings = [];
  const origWarn = console.warn;
  console.warn = (...a) => warnings.push(a.join(' '));
  try {
    const fallback = readJson(badFile, { def: true });
    assert(fallback && fallback.def === true, 'malformed JSON should return the fallback');
  } finally {
    console.warn = origWarn;
  }
  assert(
    warnings.some((w) => w.includes('could not parse JSON') && w.includes(badFile)),
    `expected a warning naming the malformed file, got ${JSON.stringify(warnings)}`,
  );
  fs.rmSync(dir, { recursive: true, force: true });
});

await check('resolveProductRoot: unset->bee root (FREEZE, zero change); set->resolved; set-but-missing->loud warn (GitHub #14)', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-productroot-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  const cfg = path.join(dir, '.bee', 'config.json');

  // FREEZE 1: no config file at all -> product root IS the bee root (byte-identical to today).
  assert(laneStore.resolveProductRoot(dir) === dir, 'no config -> bee root');
  // FREEZE 2: config present, no product_root key -> still the bee root.
  writeJsonAtomic(cfg, { hooks: {} });
  assert(laneStore.resolveProductRoot(dir) === dir, 'config without product_root -> bee root');
  // FREEZE 3: empty-string product_root is treated as unset.
  writeJsonAtomic(cfg, { product_root: '' });
  assert(laneStore.resolveProductRoot(dir) === dir, 'empty product_root -> bee root');

  // SET + exists (relative) -> resolves under the bee root.
  fs.mkdirSync(path.join(dir, 'repo'), { recursive: true });
  writeJsonAtomic(cfg, { product_root: 'repo' });
  assert(laneStore.resolveProductRoot(dir) === path.resolve(dir, 'repo'), 'relative product_root resolves under bee root');
  // SET + exists (absolute) -> honored as-is.
  writeJsonAtomic(cfg, { product_root: path.join(dir, 'repo') });
  assert(laneStore.resolveProductRoot(dir) === path.join(dir, 'repo'), 'absolute product_root honored');

  // SET + missing -> loud stderr warning (not silent) AND returns the resolved missing path.
  writeJsonAtomic(cfg, { product_root: 'nope' });
  const warnings = [];
  const origWarn = console.warn;
  console.warn = (...a) => warnings.push(a.join(' '));
  let missingResolved;
  try {
    missingResolved = laneStore.resolveProductRoot(dir);
  } finally {
    console.warn = origWarn;
  }
  assert(missingResolved === path.resolve(dir, 'nope'), 'set-but-missing returns resolved path (reads find nothing)');
  assert(
    warnings.some((w) => w.includes('product_root') && w.includes('#14')),
    `set-but-missing must warn loudly naming product_root, got ${JSON.stringify(warnings)}`,
  );
  fs.rmSync(dir, { recursive: true, force: true });
});

await check('backlog reads resolve against product_root when set — the repo-divorce topology (GitHub #14)', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-productroot-int-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  // Workshop-side docs/ is EMPTY (the bug: reads landed here and found nothing).
  // The real product docs live one dir down in ./repo.
  fs.mkdirSync(path.join(dir, 'repo', 'docs'), { recursive: true });
  fs.writeFileSync(
    path.join(dir, 'repo', 'docs', 'backlog.md'),
    '# Backlog\n\n| ID | Story | Status |\n|----|-------|--------|\n| 1 | A | done |\n| 2 | B | proposed |\n| 3 | C | in-flight |\n',
    'utf8',
  );
  // Without product_root: workshop docs/backlog.md absent -> null (today's silent behavior).
  assert(readBacklogCounts(dir) === null, 'no product_root + no workshop backlog -> null (unchanged)');
  // With product_root=repo: the same call now reads ./repo/docs/backlog.md.
  writeJsonAtomic(path.join(dir, '.bee', 'config.json'), { product_root: 'repo' });
  const counts = readBacklogCounts(dir);
  assert(counts && counts.done === 1 && counts.proposed === 1 && counts.inFlight === 1,
    `backlog counts should come from ./repo/docs/backlog.md, got ${JSON.stringify(counts)}`);
  fs.rmSync(dir, { recursive: true, force: true });
});

await check('inject cache re-homes to .bee/cache/ and migrates from the legacy .bee/ root file (GitHub #11)', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cache-migrate-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  const legacy = path.join(dir, '.bee', '.inject-cache.json');
  const fresh = path.join(dir, '.bee', 'cache', 'inject-cache.json');
  // Seed the legacy-location cache with a prior injection record.
  fs.writeFileSync(legacy, JSON.stringify({ preamble: { hash: 'h1', at: new Date().toISOString() } }), 'utf8');
  // shouldInject honors the legacy history via fallback read: same hash -> no re-inject.
  assert(shouldInject(dir, 'preamble', 'h1') === false, 'legacy dedup history honored via fallback');
  // markInjected writes to the NEW location and removes the legacy file.
  markInjected(dir, 'preamble', 'h2');
  assert(fs.existsSync(fresh), 'inject cache now lives under .bee/cache/');
  assert(!fs.existsSync(legacy), 'legacy .bee/.inject-cache.json is cleaned up');
  fs.rmSync(dir, { recursive: true, force: true });
});

await check('findRepoRoot walks up from a nested dir', async () => {
  const found = findRepoRoot(path.join(root, 'src', 'deep', 'nested'));
  assert(found === root, `expected ${root}, got ${found}`);
});

await check('resolveRoots keeps ordinary repositories ordinary', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-root-ordinary-'));
  fs.mkdirSync(path.join(dir, '.git'));
  const roots = resolveRoots(path.join(dir, 'child'));
  assert(roots.storeRoot === dir && roots.workRoot === dir, 'ordinary roots should point at checkout');
  assert(roots.worktreeResolution === 'ordinary', 'ordinary resolution expected');
});

await check('resolveRoots validates linked-worktree backlinks and shares main store', async () => {
  const main = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-root-main-'));
  const work = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-root-work-'));
  const id = 'fixture';
  const gitdir = path.join(main, '.git', 'worktrees', id);
  fs.mkdirSync(gitdir, { recursive: true });
  fs.writeFileSync(path.join(work, '.git'), `gitdir: ${gitdir}\n`);
  fs.writeFileSync(path.join(gitdir, 'gitdir'), path.join(work, '.git') + '\n');
  const roots = resolveRoots(work);
  assert(roots.storeRoot === main && roots.workRoot === work, 'linked roots should split store/work roots');
  assert(roots.worktreeResolution === 'linked-valid', 'linked-valid resolution expected');
});

await check('linked-shaped invalid metadata fails closed with typed error', async () => {
  const work = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-root-invalid-'));
  fs.writeFileSync(path.join(work, '.git'), 'gitdir: /tmp/forged/.git/worktrees/x\n');
  let caught;
  try { resolveRoots(work); } catch (err) { caught = err; }
  assert(caught instanceof WorktreeLinkInvalidError, 'expected typed worktree error');
  assert(caught.code === 'WORKTREE_LINK_INVALID', 'expected stable error code');
  let rootError;
  try { findRepoRoot(work); } catch (err) { rootError = err; }
  assert(rootError && rootError.code === 'WORKTREE_LINK_INVALID', 'findRepoRoot must fail closed');
});

// ─── resolveContext (multisession-native-17, CONTEXT.md D2) ────────────────
// Two-topology proof, same manual worktree-fixture pattern as the resolveRoots
// tests just above (main .git/worktrees/<id> + reverse gitdir pointer) —
// lighter-weight than test_worktree_cli.mjs's real `git worktree add` spawn,
// consistent with how this file already proves resolveRoots itself.

await check('resolveContext: main checkout — control/local/git roots, workspaceId main, worktreeId null', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-ctx-main-'));
  fs.mkdirSync(path.join(dir, '.git'));
  const ctx = resolveContext(path.join(dir, 'child'));
  assert(ctx.workspaceRoot === dir, `workspaceRoot should be the checkout root, got ${ctx.workspaceRoot}`);
  assert(ctx.gitCommonDir === path.join(dir, '.git'), `gitCommonDir should be the checkout's own .git, got ${ctx.gitCommonDir}`);
  assert(ctx.controlRoot === dir, `unexpected controlRoot ${ctx.controlRoot} (msn-18a: controlRoot is mainRoot itself, not a runtime/control subdir)`);
  assert(ctx.localRuntimeRoot === path.join(dir, '.bee', 'runtime', 'local'), `unexpected localRuntimeRoot ${ctx.localRuntimeRoot}`);
  assert(ctx.projectRoot === dir, `projectRoot should default to workspaceRoot, got ${ctx.projectRoot}`);
  assert(ctx.workspaceId === 'main', `workspaceId should be 'main', got ${ctx.workspaceId}`);
  assert(ctx.worktreeId === null, `worktreeId should be null for the main checkout, got ${ctx.worktreeId}`);
});

await check('resolveContext: unregistered linked worktree — control root shared at MAIN, local root at the worktree, workspaceId still main', async () => {
  const main = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-ctx-main2-'));
  const work = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-ctx-work-'));
  const id = 'ctx-fixture-unregistered';
  const gitdir = path.join(main, '.git', 'worktrees', id);
  fs.mkdirSync(gitdir, { recursive: true });
  fs.writeFileSync(path.join(work, '.git'), `gitdir: ${gitdir}\n`);
  fs.writeFileSync(path.join(gitdir, 'gitdir'), path.join(work, '.git') + '\n');

  const ctx = resolveContext(work);
  assert(ctx.workspaceRoot === work, `workspaceRoot should be the worktree's own checkout, got ${ctx.workspaceRoot}`);
  assert(ctx.gitCommonDir === path.join(main, '.git'), `gitCommonDir should be the MAIN checkout's .git, got ${ctx.gitCommonDir}`);
  assert(ctx.controlRoot === main, `controlRoot should be shared at MAIN itself (msn-18a), got ${ctx.controlRoot}`);
  assert(ctx.localRuntimeRoot === path.join(work, '.bee', 'runtime', 'local'), `localRuntimeRoot should stay per-checkout (worktree), got ${ctx.localRuntimeRoot}`);
  assert(ctx.worktreeId === id, `worktreeId should report the git-verified id, got ${ctx.worktreeId}`);
  assert(ctx.workspaceId === 'main', `unregistered worktree's workspaceId should fall back to 'main' (P40 default), got ${ctx.workspaceId}`);
});

await check('resolveContext: REGISTERED (granted) linked worktree — workspaceId becomes its own id, controlRoot stays shared at MAIN', async () => {
  const main = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-ctx-main3-'));
  const work = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-ctx-work2-'));
  const id = 'ctx-fixture-registered';
  const gitdir = path.join(main, '.git', 'worktrees', id);
  fs.mkdirSync(gitdir, { recursive: true });
  fs.writeFileSync(path.join(work, '.git'), `gitdir: ${gitdir}\n`);
  fs.writeFileSync(path.join(gitdir, 'gitdir'), path.join(work, '.git') + '\n');
  // Grant the worktree its own workspace via the MAIN store's registry —
  // the same registry decideWorktreeStore/readGrants read (worktree-store.mjs).
  fs.mkdirSync(path.join(main, '.bee', 'runtime'), { recursive: true });
  fs.writeFileSync(path.join(main, '.bee', 'runtime', 'worktree-grants.json'), JSON.stringify({ [id]: true }));

  const ctx = resolveContext(work);
  assert(ctx.workspaceId === id, `registered worktree's workspaceId should be its own id, got ${ctx.workspaceId}`);
  assert(ctx.worktreeId === id, `worktreeId should still be the git-verified id, got ${ctx.worktreeId}`);
  assert(ctx.controlRoot === main, `controlRoot must stay shared at MAIN itself even when granted (msn-18a), got ${ctx.controlRoot}`);
  assert(ctx.localRuntimeRoot === path.join(work, '.bee', 'runtime', 'local'), `localRuntimeRoot must stay the worktree's own checkout, got ${ctx.localRuntimeRoot}`);

  // workspaceId is stable across repeated calls (must_have).
  const again = resolveContext(work);
  assert(again.workspaceId === ctx.workspaceId, 'workspaceId must be stable across repeated resolveContext calls');
});

// BINDING (advisor-digest-slice4, msn-18 re-slice): the topology test data-
// plane isolation must not silently collapse when the control plane
// collapses onto mainRoot (msn-18a's controlRoot fix above) — a linked
// worktree's localRuntimeRoot must resolve to a DISTINCT subtree under the
// WORKTREE's own root, never main's, even though controlRoot now equals
// main's bare checkout root for both.
await check('resolveContext BINDING: localRuntimeRoot is a DISTINCT per-worktree subtree, never collapsing onto main (data-plane isolation)', async () => {
  const main = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-ctx-distinct-main-'));
  const work = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-ctx-distinct-work-'));
  fs.mkdirSync(path.join(main, '.git'));
  const id = 'ctx-fixture-distinct';
  const gitdir = path.join(main, '.git', 'worktrees', id);
  fs.mkdirSync(gitdir, { recursive: true });
  fs.writeFileSync(path.join(work, '.git'), `gitdir: ${gitdir}\n`);
  fs.writeFileSync(path.join(gitdir, 'gitdir'), path.join(work, '.git') + '\n');

  const mainCtx = resolveContext(main);
  const workCtx = resolveContext(work);

  // Control plane collapses onto the SAME shared root for both (msn-18a).
  assert(mainCtx.controlRoot === main, `main's own controlRoot should be itself, got ${mainCtx.controlRoot}`);
  assert(workCtx.controlRoot === mainCtx.controlRoot, `worktree controlRoot must equal main's controlRoot, got ${workCtx.controlRoot} vs ${mainCtx.controlRoot}`);

  // Local runtime plane must NOT collapse — each checkout gets its own
  // subtree, and the worktree's lives under the WORKTREE's root, never main's.
  assert(workCtx.localRuntimeRoot !== mainCtx.localRuntimeRoot, `worktree localRuntimeRoot must differ from main's, both resolved to ${workCtx.localRuntimeRoot}`);
  assert(workCtx.localRuntimeRoot === path.join(work, '.bee', 'runtime', 'local'), `worktree localRuntimeRoot must nest under the WORKTREE's own root, got ${workCtx.localRuntimeRoot}`);
  assert(!workCtx.localRuntimeRoot.startsWith(main + path.sep), `worktree localRuntimeRoot must never nest under main's root, got ${workCtx.localRuntimeRoot}`);
});

await check('resolveRoots compat wrapper stays byte-identical after the resolveContext refactor', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-ctx-wrapper-'));
  fs.mkdirSync(path.join(dir, '.git'));
  const roots = resolveRoots(path.join(dir, 'child'));
  assert(Object.keys(roots).sort().join(',') === 'storeRoot,workRoot,worktreeResolution', `resolveRoots must keep its exact original key set, got ${Object.keys(roots).sort().join(',')}`);
  assert(roots.storeRoot === dir && roots.workRoot === dir && roots.worktreeResolution === 'ordinary', 'resolveRoots wrapper must still resolve an ordinary checkout unchanged');

  // Cross-check resolveContext agrees on the same underlying checkout.
  const ctx = resolveContext(path.join(dir, 'child'));
  assert(ctx.workspaceRoot === roots.workRoot, 'resolveContext.workspaceRoot must agree with resolveRoots.workRoot for the same cwd');
});

await check('readState returns defaults when state.json missing', async () => {
  const state = readState(root);
  assert(state.phase === 'idle', `default phase should be idle, got ${state.phase}`);
  assert(gateApproved(state, 'execution') === false, 'execution gate should default false');
});

// ─── readStateStrict (review P1-1: a present-but-corrupt state.json must ────
// fail loud, never be silently clobbered to defaults by a bee_state mutation).
// readState itself stays fail-open — hooks and bee_status depend on that
// shape — so these tests pin readStateStrict's distinct absent-vs-corrupt
// behavior AND that readState's own semantics are unchanged.

await check('readStateStrict returns defaults when state.json is absent (same as readState)', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-strict-absent-'));
  try {
    const state = readStateStrict(dir);
    assert(state.phase === 'idle', `default phase should be idle, got ${state.phase}`);
    assert(gateApproved(state, 'execution') === false, 'execution gate should default false');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('readStateStrict throws on a present-but-unparseable state.json, naming the file and a FIX', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-strict-corrupt-'));
  try {
    fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
    fs.writeFileSync(path.join(dir, '.bee', 'state.json'), '{ not valid json', 'utf8');
    let threw = null;
    try {
      readStateStrict(dir);
    } catch (err) {
      threw = err instanceof Error ? err.message : String(err);
    }
    assert(threw !== null, 'readStateStrict throws on unparseable JSON');
    assert(/state\.json/.test(threw), `error names the state.json file, got ${threw}`);
    assert(/not valid json/i.test(threw), `error says the file is not valid JSON, got ${threw}`);
    assert(/refuses to rebuild state from defaults/i.test(threw), `error says the CLI refuses to rebuild from defaults, got ${threw}`);
    assert(/FIX:/.test(threw), `error carries a FIX:, got ${threw}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('readStateStrict throws when state.json parses but is not a JSON object', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-strict-nonobject-'));
  try {
    fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
    fs.writeFileSync(path.join(dir, '.bee', 'state.json'), '[1,2,3]', 'utf8');
    assertThrows(
      () => readStateStrict(dir),
      'not a json object',
      'readStateStrict rejects a non-object JSON value (an array)',
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('readState (non-strict) still returns defaults for the same corrupt input — fail-open shape unchanged', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-nonstrict-corrupt-'));
  try {
    fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
    fs.writeFileSync(path.join(dir, '.bee', 'state.json'), '{ not valid json', 'utf8');
    const state = readState(dir);
    assert(state.phase === 'idle', `readState should fail open to defaults, got phase ${state.phase}`);
    assert(gateApproved(state, 'execution') === false, 'execution gate should default false');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── lanes (fsh-3, fresh-session-handoff S2): per-feature lane records beside ─
// the default pipeline. Additive by design (D4): a repo with no .bee/lanes/
// and no bound session behaves byte-identically to today — the pre-existing
// state/start-feature rows above are the parity proof and are never modified,
// only extended by the rows below. A LANE start's preconditions are the
// validated Q4 set: same-feature nonterminal cells, feature-attributed
// handoff/workers (attribution DERIVED from existing fields — handoff.feature,
// worker→cell→feature — no new fields invented), and a global declared-paths
// vs other-session-holds overlap check.

function laneFile(dir, feature) {
  return path.join(dir, '.bee', 'lanes', `${feature}.json`);
}

function writeLaneFixture(dir, feature, extra = {}) {
  laneStore.writeLane(dir, {
    schema_version: '1.0',
    feature,
    mode: null,
    phase: 'idle',
    approved_gates: { context: false, shape: false, execution: false, review: false },
    summary: '',
    next_action: '',
    created_at: new Date().toISOString(),
    ...extra,
  });
}

await check('lanes: writeLane/readLane round-trip at .bee/lanes/<feature>.json (unicode/space names included); missing lane reads null; listLanes enumerates; removeLane deletes', async () => {
  const dir = makeStateRepo('bee-lane-crud-');
  try {
    writeLaneFixture(dir, 'lane-a', { mode: 'standard', phase: 'exploring', summary: 'sum', next_action: 'next' });
    assert(fs.existsSync(laneFile(dir, 'lane-a')), 'lane record lives at .bee/lanes/<feature>.json');
    assert(laneStore.lanePath(dir, 'lane-a') === laneFile(dir, 'lane-a'), 'lanePath resolves under .bee/lanes');
    const lane = laneStore.readLane(dir, 'lane-a');
    assert(lane && lane.feature === 'lane-a', 'feature round-trips');
    assert(lane.mode === 'standard' && lane.phase === 'exploring', 'mode/phase round-trip');
    assert(lane.approved_gates && lane.approved_gates.execution === false, 'gates round-trip');
    assert(typeof lane.created_at === 'string' && !Number.isNaN(Date.parse(lane.created_at)), 'created_at is a timestamp');
    assert(laneStore.readLane(dir, 'lane-ghost') === null, 'missing lane reads null, never a guessed default');
    writeLaneFixture(dir, 'tính năng á'); // input-extremes probe: spaces + unicode
    assert(laneStore.readLane(dir, 'tính năng á').feature === 'tính năng á', 'unicode/space feature names round-trip');
    const listed = laneStore.listLanes(dir).map((l) => l.feature).sort();
    assert(JSON.stringify(listed) === JSON.stringify(['lane-a', 'tính năng á'].sort()), `listLanes enumerates lane records, got ${JSON.stringify(listed)}`);
    laneStore.removeLane(dir, 'tính năng á');
    assert(!fs.existsSync(laneFile(dir, 'tính năng á')), 'removeLane deletes the record');
    assertThrows(() => laneStore.lanePath(dir, '../evil'), 'plain id', 'path-shaped lane names are rejected as bad arguments');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('lanes: readLane/listLanes are fail-open for display — a corrupt lane file warns, is skipped, and stays untouched on disk', async () => {
  const dir = makeStateRepo('bee-lane-corrupt-read-');
  try {
    writeLaneFixture(dir, 'lane-ok');
    fs.writeFileSync(laneFile(dir, 'lane-bad'), '{ not json', 'utf8');
    const before = fs.readFileSync(laneFile(dir, 'lane-bad'), 'utf8');
    const warnings = [];
    const origWarn = console.warn;
    console.warn = (...a) => warnings.push(a.join(' '));
    let read;
    let listed;
    try {
      read = laneStore.readLane(dir, 'lane-bad');
      listed = laneStore.listLanes(dir);
    } finally {
      console.warn = origWarn;
    }
    assert(read === null, 'corrupt lane reads null for display');
    assert(warnings.some((w) => w.includes('lane-bad')), `a warning names the corrupt lane, got ${JSON.stringify(warnings)}`);
    assert(listed.length === 1 && listed[0].feature === 'lane-ok', 'listLanes skips the corrupt record and keeps the healthy one');
    assert(fs.readFileSync(laneFile(dir, 'lane-bad'), 'utf8') === before, 'corrupt file untouched by fail-open reads');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('lanes: readLaneStrict refuses loudly on a present-but-corrupt lane file (untouched); a missing lane reads null (creation is the caller\'s explicit move)', async () => {
  const dir = makeStateRepo('bee-lane-strict-');
  try {
    fs.mkdirSync(path.join(dir, '.bee', 'lanes'), { recursive: true });
    fs.writeFileSync(laneFile(dir, 'lane-bad'), '{ not json', 'utf8');
    const before = fs.readFileSync(laneFile(dir, 'lane-bad'), 'utf8');
    assertThrows(() => laneStore.readLaneStrict(dir, 'lane-bad'), 'lane', 'corrupt lane refuses loudly for mutation');
    assert(fs.readFileSync(laneFile(dir, 'lane-bad'), 'utf8') === before, 'refusal leaves the corrupt file untouched');
    // a record whose feature field names ANOTHER feature is corrupt, never trusted
    writeLaneFixture(dir, 'lane-lies');
    const lying = readJson(laneFile(dir, 'lane-lies'), null);
    writeJsonAtomic(laneFile(dir, 'lane-lies'), { ...lying, feature: 'someone-else' });
    assertThrows(() => laneStore.readLaneStrict(dir, 'lane-lies'), 'lane', 'a feature-mismatched record refuses under strict');
    assert(laneStore.readLaneStrict(dir, 'lane-ghost') === null, 'missing lane is null under strict too');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('lanes: createSession OMITS the lane key when unbound; bindSessionLane writes it, unbindSessionLane removes the key entirely; ghost session is typed SESSION_MISSING', async () => {
  const dir = makeStateRepo('bee-lane-bind-');
  try {
    const made = laneBinding.createSession(dir, { id: 'sess-bind' });
    assert(made.ok === true, 'session created');
    const rawUnbound = readJson(sessionPath(dir, 'sess-bind'), null);
    assert(rawUnbound && !('lane' in rawUnbound), 'unbound session record has NO lane key (pre-existing session-shape rows stay green)');
    const bound = laneBinding.bindSessionLane(dir, 'sess-bind', 'lane-a');
    assert(bound.ok === true && bound.session.lane === 'lane-a', 'bind returns the bound record');
    const rawBound = readJson(sessionPath(dir, 'sess-bind'), null);
    assert(rawBound.lane === 'lane-a' && rawBound.id === 'sess-bind', 'lane binding persisted beside the session identity');
    laneBinding.heartbeatSession(dir, 'sess-bind');
    assert(readJson(sessionPath(dir, 'sess-bind'), null).lane === 'lane-a', 'the binding survives a heartbeat rewrite');
    const unbound = laneBinding.unbindSessionLane(dir, 'sess-bind');
    assert(unbound.ok === true, 'unbind ok');
    const rawAfter = readJson(sessionPath(dir, 'sess-bind'), null);
    assert(rawAfter && !('lane' in rawAfter), 'unbind removes the key entirely, not lane:null');
    const ghost = laneBinding.bindSessionLane(dir, 'sess-ghost', 'lane-a');
    assert(ghost.ok === false && ghost.code === 'SESSION_MISSING' && typeof ghost.reason === 'string', 'binding a missing session is a typed failure — no throw');
    assertThrows(() => laneBinding.bindSessionLane(dir, 'sess-bind', '../evil'), 'plain id', 'path-shaped lane names are rejected as bad arguments');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('lanes: resolvePipeline — no sessionId, unknown session, or unbound session resolves to the DEFAULT record; a bound session resolves to its lane record', async () => {
  const dir = makeStateRepo('bee-lane-resolve-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { schema_version: '1.0', phase: 'swarming', feature: 'default-feat', workers: [] });
    const bare = laneStore.resolvePipeline(dir);
    assert(bare.ok === true && bare.source === 'default' && bare.record.feature === 'default-feat', 'no sessionId → the default record');
    const unknown = laneStore.resolvePipeline(dir, { sessionId: 'sess-nobody' });
    assert(unknown.ok === true && unknown.source === 'default', 'unknown session → default, resolution never guesses a lane');
    laneBinding.createSession(dir, { id: 'sess-r' });
    const unbound = laneStore.resolvePipeline(dir, { sessionId: 'sess-r' });
    assert(unbound.ok === true && unbound.source === 'default', 'unbound session → default');
    writeLaneFixture(dir, 'lane-r', { phase: 'planning', mode: 'standard' });
    laneBinding.bindSessionLane(dir, 'sess-r', 'lane-r');
    const bound = laneStore.resolvePipeline(dir, { sessionId: 'sess-r' });
    assert(bound.ok === true && bound.source === 'lane', `bound session → lane source, got ${JSON.stringify(bound)}`);
    assert(bound.feature === 'lane-r' && bound.record.feature === 'lane-r' && bound.record.phase === 'planning', 'the bound lane record is returned');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('lanes: resolvePipeline — a binding to a missing or corrupt lane is a TYPED refusal naming the lane, never a silent fall-back to the default', async () => {
  const dir = makeStateRepo('bee-lane-resolve-refuse-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { schema_version: '1.0', phase: 'idle', feature: null, workers: [] });
    laneBinding.createSession(dir, { id: 'sess-m' });
    laneBinding.bindSessionLane(dir, 'sess-m', 'lane-ghost');
    const missing = laneStore.resolvePipeline(dir, { sessionId: 'sess-m' });
    assert(missing.ok === false && missing.code === 'LANE_MISSING', `missing lane is a typed refusal, got ${JSON.stringify(missing)}`);
    assert(typeof missing.reason === 'string' && missing.reason.includes('lane-ghost'), 'reason names the missing lane');
    fs.mkdirSync(path.join(dir, '.bee', 'lanes'), { recursive: true });
    fs.writeFileSync(laneFile(dir, 'lane-ghost'), '{ not json', 'utf8');
    const warnings = [];
    const origWarn = console.warn;
    console.warn = (...a) => warnings.push(a.join(' '));
    let corrupt;
    try {
      corrupt = laneStore.resolvePipeline(dir, { sessionId: 'sess-m' });
    } finally {
      console.warn = origWarn;
    }
    assert(corrupt.ok === false && corrupt.code === 'LANE_CORRUPT', `corrupt lane is a typed refusal, got ${JSON.stringify(corrupt)}`);
    assert(typeof corrupt.reason === 'string' && corrupt.reason.includes('lane-ghost'), 'reason names the corrupt lane');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('lanes: startFeature lane mode creates the lane with all four gates false while state.json and every other lane stay byte-identical (D4 zero-touch)', async () => {
  const dir = makeStateRepo('bee-lane-start-ok-');
  try {
    const statePath = path.join(dir, '.bee', 'state.json');
    writeJsonAtomic(statePath, {
      schema_version: '1.0',
      phase: 'swarming', // the DEFAULT pipeline is mid-flight — a lane start must not care and must not touch it
      feature: 'default-feat',
      mode: 'standard',
      approved_gates: { context: true, shape: true, execution: true, review: false },
      workers: [],
    });
    writeLaneFixture(dir, 'lane-other', { phase: 'planning' });
    const stateBefore = fs.readFileSync(statePath, 'utf8');
    const otherBefore = fs.readFileSync(laneFile(dir, 'lane-other'), 'utf8');
    const record = await startFeature(dir, { feature: 'lane-new', mode: 'high-risk', phase: 'exploring', lane: true });
    assert(record.feature === 'lane-new' && record.mode === 'high-risk' && record.phase === 'exploring', 'lane record carries feature/mode/phase');
    assert(Object.values(record.approved_gates).every((v) => v === false), `all four gates start false — a lane never inherits approvals, got ${JSON.stringify(record.approved_gates)}`);
    assert(typeof record.created_at === 'string' && !Number.isNaN(Date.parse(record.created_at)), 'created_at stamped');
    assert(fs.existsSync(laneFile(dir, 'lane-new')), 'lane record written to .bee/lanes/');
    assert(fs.readFileSync(statePath, 'utf8') === stateBefore, 'the DEFAULT record is byte-identical — a lane start never touches state.json');
    assert(fs.readFileSync(laneFile(dir, 'lane-other'), 'utf8') === otherBefore, 'every other lane byte-identical');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('lanes: a lane start refuses while THIS feature has nonterminal cells, and is never blocked by another feature\'s nonterminal cells', async () => {
  const dir = makeStateRepo('bee-lane-start-cells-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { schema_version: '1.0', phase: 'swarming', feature: 'default-feat', workers: [] });
    makeCellFile(dir, 'mine-1', { feature: 'lane-c', status: 'open' });
    makeCellFile(dir, 'other-1', { feature: 'elsewhere', status: 'claimed' });
    await assertRejects(
      () => startFeature(dir, { feature: 'lane-c', lane: true }),
      'mine-1',
      'a same-feature nonterminal cell refuses the lane start',
    );
    assert(!fs.existsSync(laneFile(dir, 'lane-c')), 'refusal writes nothing');
    const record = await startFeature(dir, { feature: 'lane-d', lane: true });
    assert(record.feature === 'lane-d', 'another feature\'s nonterminal (even claimed) cell never blocks an unrelated lane start');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('lanes: a global HANDOFF blocks a lane start only when its feature names this lane', async () => {
  const dir = makeStateRepo('bee-lane-start-handoff-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { schema_version: '1.0', phase: 'idle', feature: null, workers: [] });
    writeJsonAtomic(path.join(dir, '.bee', 'HANDOFF.json'), { feature: 'lane-e', cell: 'x', done: [], remaining: [] });
    await assertRejects(() => startFeature(dir, { feature: 'lane-e', lane: true }), 'HANDOFF', 'a handoff naming THIS feature blocks its lane start');
    assert(!fs.existsSync(laneFile(dir, 'lane-e')), 'refusal writes nothing');
    const unrelated = await startFeature(dir, { feature: 'lane-f', lane: true });
    assert(unrelated.feature === 'lane-f', 'a handoff for another feature does not block this lane');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// F5 (advisor consult slice 2, binding, multisession-native-6): the DEFAULT
// (non-lane) start now mirrors the lane path's exact per-feature HANDOFF
// scoping instead of any-handoff-blocks — see startFeature's own header
// comment in lib/state.mjs for the residual-coupling note (one shared
// .bee/HANDOFF.json file until slice 3's per-workflow mailboxes, msn-15).
await check('F5: the DEFAULT (non-lane) start blocks only when .bee/HANDOFF.json names THIS feature; an unrelated-feature handoff no longer blocks it', async () => {
  const dir = makeStateRepo('bee-default-start-handoff-scoped-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { schema_version: '1.0', phase: 'idle', feature: null, workers: [] });
    writeJsonAtomic(path.join(dir, '.bee', 'HANDOFF.json'), { feature: 'other-feat', cell: 'x', done: [], remaining: [] });
    const unrelated = await startFeature(dir, { feature: 'new-feat' });
    assert(unrelated.feature === 'new-feat', 'a handoff naming a DIFFERENT feature no longer blocks the default start (F5)');

    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { schema_version: '1.0', phase: 'idle', feature: null, workers: [] });
    writeJsonAtomic(path.join(dir, '.bee', 'HANDOFF.json'), { feature: 'same-feat', cell: 'x', done: [], remaining: [] });
    const before = fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8');
    await assertRejects(
      () => startFeature(dir, { feature: 'same-feat' }),
      'HANDOFF',
      'a handoff naming THIS feature still blocks the default start (SAME-feature refusal semantics preserved)',
    );
    assert(fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8') === before, 'refusal writes nothing');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// multisession-native-8 (D6): "worker" is now a derived view (live-heartbeat
// session x active claim, claims.mjs activeWorkers) — never state.json's
// hand-mutated `workers` array. A hand-written entry alone (no live session)
// must never block; a genuinely live OTHER session with a claim on this
// feature's cell still blocks exactly as before (worker session -> claim ->
// cell.feature).
await check('lanes: an OTHER live worker session blocks a lane start only when its claimed cell derives to this lane\'s feature (derived view, msn-8/D6)', async () => {
  const dir = makeStateRepo('bee-lane-start-worker-');
  try {
    makeCellFile(dir, 'wcell-1', { feature: 'lane-h', status: 'capped' }); // terminal, so precondition (a) passes — isolates the worker check
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'swarming',
      feature: 'default-feat',
      workers: [],
    });
    createSession(dir, { id: 'busy-sess' });
    claimCellFile(dir, 'busy-sess', 'wcell-1');
    await assertRejects(
      () => startFeature(dir, { feature: 'lane-h', lane: true, sessionId: 'caller' }),
      'worker',
      'a live OTHER session claiming this feature\'s cell blocks the lane start',
    );
    assert(!fs.existsSync(laneFile(dir, 'lane-h')), 'refusal writes nothing');
    const unrelated = await startFeature(dir, { feature: 'lane-i', lane: true, sessionId: 'caller' });
    assert(unrelated.feature === 'lane-i', 'a worker on another feature\'s cell never blocks this lane');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('lanes: a hand-written state.workers entry alone (no live session) never blocks a lane start (D6 demotion, msn-8)', async () => {
  const dir = makeStateRepo('bee-lane-start-worker-ghost-');
  try {
    makeCellFile(dir, 'wcell-2', { feature: 'lane-ghost', status: 'capped' });
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'swarming',
      feature: 'default-feat',
      workers: [{ nickname: 'ghost', cell: 'wcell-2', tier: 'generation', status: 'in-flight' }],
    });
    const started = await startFeature(dir, { feature: 'lane-ghost', lane: true, sessionId: 'caller' });
    assert(started.feature === 'lane-ghost', 'a hand-written workers entry with no live session must never block');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('lanes: a lane start declaring intended paths refuses on overlap with ANOTHER session\'s active holds (claimed-cell files or reservations); own and expired holds never block', async () => {
  const dir = makeStateRepo('bee-lane-start-holds-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { schema_version: '1.0', phase: 'idle', feature: null, workers: [] });
    laneBinding.createSession(dir, { id: 'sess-me' });
    laneBinding.createSession(dir, { id: 'sess-them' });
    makeCellFile(dir, 'held-cell', { feature: 'elsewhere', status: 'capped', files: ['src/app.ts'] });
    const held = claimCellFile(dir, 'sess-them', 'held-cell');
    assert(held.ok === true, 'precondition: another session holds a claim whose cell files include src/app.ts');
    await assertRejects(
      () => startFeature(dir, { feature: 'lane-j', lane: true, sessionId: 'sess-me', paths: ['src/app.ts'] }),
      'sess-them',
      'overlap with another session\'s claim-held files refuses, naming the holder',
    );
    assert(!fs.existsSync(laneFile(dir, 'lane-j')), 'refusal writes nothing');
    const own = await startFeature(dir, { feature: 'lane-k', lane: true, sessionId: 'sess-them', paths: ['src/app.ts'] });
    assert(own.feature === 'lane-k', 'the holder\'s own session is never blocked by its own claim');
    // Explicit session (hardening-1-7-10 D1): two live sessions already exist
    // in this fixture (sess-me, sess-them), so a sessionless reserve() would
    // hit the typed SESSION_REQUIRED refusal hermetically (no env fallback to
    // silently supply an identity) and never create the hold at all — the
    // test's intent is proving reservation-overlap refusal, not exercising
    // the sessionless path (that is covered elsewhere), so give it a real
    // session identity and assert the hold actually landed.
    const reserved = await reserve(dir, { agent: 'worker-z', cell: 'z-1', path: 'src/lib/*', session: 'sess-them' });
    assert(reserved.ok === true, `precondition: worker-z's reservation must actually be created, got ${JSON.stringify(reserved)}`);
    await assertRejects(
      () => startFeature(dir, { feature: 'lane-l', lane: true, sessionId: 'sess-me', paths: ['src/lib/util.ts'] }),
      'worker-z',
      'overlap with an active reservation refuses, naming the holder',
    );
    // Backdate the underlying lease itself (msn-16: reservationsPath is a
    // rebuildable projection, not the live store) so its expires_at falls
    // well before the REAL current time — a 60s ttl anchored 2h in the past.
    await renewLease(dir, { type: 'path', id: 'src/lib/*' }, { ttl: 60, now: Date.now() - 7200 * 1000 });
    const expired = await startFeature(dir, { feature: 'lane-l', lane: true, sessionId: 'sess-me', paths: ['src/lib/util.ts'] });
    assert(expired.feature === 'lane-l', 'an expired hold never blocks');
    const undeclared = await startFeature(dir, { feature: 'lane-m', lane: true, sessionId: 'sess-me' });
    assert(undeclared.feature === 'lane-m', 'no declared paths → the holds check is skipped by contract');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('lanes: restarting a terminal lane resets exactly its four gates (created_at preserved); a mid-flight lane refuses; a corrupt lane file refuses loudly untouched', async () => {
  const dir = makeStateRepo('bee-lane-restart-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { schema_version: '1.0', phase: 'idle', feature: null, workers: [] });
    const born = '2026-01-01T00:00:00.000Z';
    writeLaneFixture(dir, 'lane-n', {
      phase: 'compounding-complete',
      mode: 'standard',
      approved_gates: { context: true, shape: true, execution: true, review: true },
      created_at: born,
    });
    const restarted = await startFeature(dir, { feature: 'lane-n', mode: 'tiny', phase: 'exploring', lane: true });
    assert(Object.values(restarted.approved_gates).every((v) => v === false), 'restart resets all four gates — spec R1 applied per lane');
    assert(restarted.created_at === born, `created_at survives a restart, got ${restarted.created_at}`);
    assert(restarted.mode === 'tiny' && restarted.phase === 'exploring', 'mode/phase refreshed');
    writeLaneFixture(dir, 'lane-o', { phase: 'swarming' });
    const midBefore = fs.readFileSync(laneFile(dir, 'lane-o'), 'utf8');
    await assertRejects(() => startFeature(dir, { feature: 'lane-o', lane: true }), 'phase', 'a mid-flight lane refuses its own restart');
    assert(fs.readFileSync(laneFile(dir, 'lane-o'), 'utf8') === midBefore, 'refusal leaves the lane untouched');
    fs.writeFileSync(laneFile(dir, 'lane-p'), '{ not json', 'utf8');
    const corruptBefore = fs.readFileSync(laneFile(dir, 'lane-p'), 'utf8');
    await assertRejects(() => startFeature(dir, { feature: 'lane-p', lane: true }), 'lane', 'a corrupt lane file refuses the mutation loudly');
    assert(fs.readFileSync(laneFile(dir, 'lane-p'), 'utf8') === corruptBefore, 'corrupt file untouched');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── multisession-native-6: startFeature creates a workflow record ────────
// D1 (CONTEXT.md), advisor consult slice 2 (C1/C4, the cell's binding
// conditions). Every startFeature call — default or lane — now also creates
// a .bee/runtime/workflows/<id>/state.json record via workflow-store.mjs,
// alongside its unchanged legacy write.

await check('msn-6 (cell truth): two DIFFERENT features started concurrently do not contend on a shared lock for the workflow-record-creation step (deterministic seam proof, msn-5-style)', async () => {
  const dir = makeStateRepo('bee-start-workflow-concurrency-');
  try {
    // Seed feature A's workflow record via a real lane start, then hold ITS
    // workflow:<id> lock externally. A single-attempt update against that
    // held lock must see LOCK_BUSY (negative control — proves the harness is
    // meaningful), while starting feature B (a DIFFERENT feature) must
    // succeed without waiting on it at all — proving the two never share a
    // lock for the workflow-record-creation step. The legacy lane-file write
    // for each call still serializes on the 'state' lock (an honest,
    // documented caveat until msn-7/10 retire it) — this test isolates the
    // workflow-record step specifically, exactly the "cell truth" it must
    // prove.
    await startFeature(dir, { feature: 'concur-a', lane: true });
    const wfA = listWorkflows(dir).workflows.find((wf) => wf.feature === 'concur-a');
    assert(wfA, 'feature A got a workflow record');

    const held = acquireStoreLockOnceSync(dir, `workflow:${wfA.id}`);
    assert(held.acquired, "precondition: test can acquire feature A's workflow lock directly");
    try {
      await assertRejects(
        () => updateWorkflow(dir, wfA.id, { phase: 'planning' }, { maxAttempts: 1 }),
        'busy',
        "sanity control: A's workflow lock really is held",
      );

      const recordB = await startFeature(dir, { feature: 'concur-b', lane: true });
      assert(recordB.feature === 'concur-b', "feature B's lane start succeeded without waiting on A's workflow lock");
      const wfB = listWorkflows(dir).workflows.find((wf) => wf.feature === 'concur-b');
      assert(wfB, "feature B got its own workflow record, created independently of A's held lock");
      assert(wfB.id !== wfA.id, 'A and B are distinct workflow records');
    } finally {
      held.release();
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('msn-6 C1: a mid-flight legacy state.json is idempotently seeded into a workflow record the first time ANY feature is started via the lane path; re-running never duplicates it', async () => {
  const dir = makeStateRepo('bee-start-c1-seed-');
  try {
    // A live legacy pipeline: the DEFAULT state.json is mid-flight for
    // "old-feature", and this is a fresh repo with zero workflow records —
    // exactly C1's dangerous moment.
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
      schema_version: '1.0',
      phase: 'swarming',
      feature: 'old-feature',
      mode: 'standard',
      approved_gates: { context: true, shape: true, execution: false, review: false },
      workers: [],
      summary: 'mid-flight legacy work',
      next_action: 'keep swarming old-feature',
    });
    assert(listWorkflows(dir).workflows.length === 0, 'precondition: zero workflow records exist yet');

    // The DEFAULT (non-lane) path would itself refuse here (mid-flight phase
    // check) — only the LANE path can start an UNRELATED feature while the
    // legacy default record is mid-flight, which is exactly the scenario
    // that could silently lose "old-feature" if C1 did not seed it.
    const started = await startFeature(dir, { feature: 'new-lane-feature', lane: true });
    assert(started.feature === 'new-lane-feature', 'the unrelated lane feature started normally');

    const afterFirst = listWorkflows(dir).workflows;
    const seeded = afterFirst.find((wf) => wf.feature === 'old-feature');
    assert(seeded, `old-feature must survive as a materialized workflow record, got ${JSON.stringify(afterFirst.map((w) => w.feature))}`);
    assert(seeded.phase === 'swarming', 'materialized record carries the legacy phase');
    assert(seeded.mode === 'standard', 'materialized record carries the legacy mode');
    assert(seeded.status === 'active', 'materialized record is active, never silently closed');
    assert(seeded.gates.context.approved === true && seeded.gates.shape.approved === true, 'approved gates carried over');
    assert(seeded.gates.execution.approved === false && seeded.gates.review.approved === false, 'unapproved gates carried over as false');
    assert(seeded.gates.context.approved_for_plan_rev === null, 'materialized gates map to approved_for_plan_rev: null (no plan-rev history to carry)');

    const newWf = afterFirst.find((wf) => wf.feature === 'new-lane-feature');
    assert(newWf, "the newly-started feature also got its own workflow record");
    assert(newWf.id !== seeded.id, "the seeded record and the new feature's record are distinct");

    // Idempotency: starting a THIRD unrelated feature must never re-seed or
    // duplicate old-feature's workflow record.
    await startFeature(dir, { feature: 'third-lane-feature', lane: true });
    const afterThird = listWorkflows(dir).workflows;
    const oldFeatureCount = afterThird.filter((wf) => wf.feature === 'old-feature').length;
    assert(oldFeatureCount === 1, `old-feature must never be duplicated by a later start, got ${oldFeatureCount} records`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── D1 (foundation-fixes): startFeature closes the outgoing feature's live
// workflow record(s) on the DEFAULT (non-lane) path — a chain of starts
// leaves exactly the newest feature's record active. ────────────────────────

await check('D1 (foundation-fixes): a chain of 3 default-path startFeature calls leaves exactly the last workflow record active; the prior two are closed', async () => {
  const dir = makeStateRepo('bee-start-close-outgoing-chain-');
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { schema_version: '1.0', phase: 'idle', feature: null, workers: [] });
    await startFeature(dir, { feature: 'chain-a' });

    // Simulate feature A reaching its terminal phase (compounding-complete)
    // before feature B starts — the real-world shape a fresh startFeature
    // call always sees (D1's own precondition requires idle or
    // compounding-complete). feature stays SET on state.json exactly like a
    // just-closed real feature would leave it.
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { schema_version: '1.0', phase: 'compounding-complete', feature: 'chain-a', workers: [] });
    await startFeature(dir, { feature: 'chain-b' });

    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { schema_version: '1.0', phase: 'compounding-complete', feature: 'chain-b', workers: [] });
    await startFeature(dir, { feature: 'chain-c' });

    const workflows = listWorkflows(dir).workflows;
    const a = workflows.filter((wf) => wf.feature === 'chain-a');
    const b = workflows.filter((wf) => wf.feature === 'chain-b');
    const c = workflows.filter((wf) => wf.feature === 'chain-c');
    assert(a.length === 1 && a[0].status === 'closed', `chain-a's workflow record must be closed, got ${JSON.stringify(a)}`);
    assert(b.length === 1 && b[0].status === 'closed', `chain-b's workflow record must be closed, got ${JSON.stringify(b)}`);
    assert(c.length === 1 && c[0].status === 'active', `chain-c (the last-started feature) must still be active, got ${JSON.stringify(c)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── config.local.json overlay (hardening-8) ───────────────────────────────
// Machine-local values (today: dogfood_repos absolute paths) live in a
// gitignored .bee/config.local.json sibling, deep-merged OVER the tracked
// .bee/config.json by readConfig — overlay wins, absent overlay is
// byte-identical to today (D4 zero-overlay parity), and arrays replace
// wholesale rather than merging element-by-element.

function writeConfigFixture(dir, tracked, overlay) {
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'config.json'), tracked);
  if (overlay !== undefined) writeJsonAtomic(localConfigPath(dir), overlay);
}

await check('readConfig: absent .bee/config.local.json is byte-identical to today (no overlay file at all)', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-config-overlay-absent-'));
  try {
    writeConfigFixture(dir, {
      gate_bypass: 'normal',
      dogfood_repos: [{ path: dir, label: 'tracked-only' }],
    });
    assert(!fs.existsSync(localConfigPath(dir)), 'precondition: no overlay file exists');
    const config = readConfig(dir);
    assert(config.gate_bypass === 'normal', `gate_bypass must come from the tracked file unchanged, got ${config.gate_bypass}`);
    assert(config.dogfood_repos.length === 1 && config.dogfood_repos[0].label === 'tracked-only', 'dogfood_repos must be exactly the tracked list when no overlay exists');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('readConfig: an overlay value WINS over the tracked value for the same key', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-config-overlay-wins-'));
  try {
    writeConfigFixture(
      dir,
      { gate_bypass: 'off', product_root: 'tracked-root' },
      { gate_bypass: 'total' },
    );
    const config = readConfig(dir);
    assert(config.gate_bypass === 'total', `overlay must win: expected "total", got ${config.gate_bypass}`);
    assert(config.product_root === 'tracked-root', 'a key the overlay never mentions must still come from the tracked file');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('readConfig: dogfood_repos overlay REPLACES the tracked array wholesale (never concatenated/interleaved)', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-config-overlay-array-'));
  try {
    writeConfigFixture(
      dir,
      { dogfood_repos: [{ path: dir, label: 'tracked-a' }, { path: dir, label: 'tracked-b' }] },
      { dogfood_repos: [{ path: dir, label: 'local-only' }] },
    );
    const config = readConfig(dir);
    assert(config.dogfood_repos.length === 1, `overlay array must fully replace the tracked array (expected 1 entry, got ${config.dogfood_repos.length})`);
    assert(config.dogfood_repos[0].label === 'local-only', `expected the overlay's own entry to survive, got ${JSON.stringify(config.dogfood_repos)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ag-2: a dead/unreadable dogfood_repos entry warns via console.warn on every
// readConfig call. Inside a hook run that fires on EVERY hook invocation and
// pollutes unrelated hook stderr; hooks/adapter.mjs's readHookContext marks
// BEE_HOOK_CONTEXT once, before any lib import, so normalizeDogfoodRepos can
// suppress the warn while still skipping the dead entry identically. Plain
// CLI runs (status, onboarding — no adapter) never set the flag and keep the
// warning verbatim.

await check('readConfig: dogfood_repos dead entry inside hook context (BEE_HOOK_CONTEXT set) — no warn output, entry still skipped', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-config-dogfood-hookctx-'));
  try {
    const deadPath = path.join(dir, 'does-not-exist');
    writeConfigFixture(dir, { dogfood_repos: [{ path: deadPath, label: 'dead' }] });
    const warnings = [];
    const origWarn = console.warn;
    console.warn = (...a) => warnings.push(a.join(' '));
    const origFlag = process.env.BEE_HOOK_CONTEXT;
    process.env.BEE_HOOK_CONTEXT = '1';
    let config;
    try {
      config = readConfig(dir);
    } finally {
      console.warn = origWarn;
      if (origFlag === undefined) delete process.env.BEE_HOOK_CONTEXT;
      else process.env.BEE_HOOK_CONTEXT = origFlag;
    }
    assert(config.dogfood_repos.length === 0, `dead entry must still be skipped under hook context, got ${JSON.stringify(config.dogfood_repos)}`);
    assert(warnings.length === 0, `hook context must suppress the dogfood_repos warn entirely, got ${JSON.stringify(warnings)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('readConfig: dogfood_repos dead entry with no BEE_HOOK_CONTEXT (plain CLI run) — warning still fires verbatim', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-config-dogfood-cli-'));
  try {
    const deadPath = path.join(dir, 'does-not-exist');
    writeConfigFixture(dir, { dogfood_repos: [{ path: deadPath, label: 'dead' }] });
    const warnings = [];
    const origWarn = console.warn;
    console.warn = (...a) => warnings.push(a.join(' '));
    const origFlag = process.env.BEE_HOOK_CONTEXT;
    delete process.env.BEE_HOOK_CONTEXT;
    let config;
    try {
      config = readConfig(dir);
    } finally {
      console.warn = origWarn;
      if (origFlag !== undefined) process.env.BEE_HOOK_CONTEXT = origFlag;
    }
    assert(config.dogfood_repos.length === 0, `dead entry must still be skipped without hook context, got ${JSON.stringify(config.dogfood_repos)}`);
    assert(
      warnings.some((w) => w.includes('dogfood_repos: skipping') && w.includes(deadPath)),
      `expected the verbatim dogfood_repos warning naming the dead path, got ${JSON.stringify(warnings)}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('readConfig: overlay deep-merges nested objects (a partial hooks override leaves untouched siblings from the tracked file)', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-config-overlay-nested-'));
  try {
    writeConfigFixture(
      dir,
      { hooks: { 'session-init': true, 'write-guard': true } },
      { hooks: { 'write-guard': false } },
    );
    const config = readConfig(dir);
    assert(config.hooks['write-guard'] === false, 'overlay leaf must win inside a nested object');
    assert(config.hooks['session-init'] === true, 'a sibling key inside the same nested object the overlay never touched must survive from the tracked file');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('readConfig: a malformed (non-object) .bee/config.local.json degrades to "absent overlay", never throws', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-config-overlay-malformed-'));
  try {
    writeConfigFixture(dir, { gate_bypass: 'normal' });
    fs.writeFileSync(localConfigPath(dir), '[ "not", "an", "object" ]\n');
    const config = readConfig(dir);
    assert(config.gate_bypass === 'normal', 'a malformed overlay must never override or crash — tracked value stands');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('mergeConfigOverlay: pure function — base is never mutated, arrays replace, plain objects merge recursively', () => {
  const base = { a: 1, nested: { x: 1, y: 2 }, list: [1, 2, 3] };
  const baseSnapshot = JSON.parse(JSON.stringify(base));
  const merged = mergeConfigOverlay(base, { nested: { y: 99 }, list: [7] });
  assert(JSON.stringify(base) === JSON.stringify(baseSnapshot), 'base object must never be mutated by the merge');
  assert(merged.a === 1, 'a key the overlay never mentions passes through unchanged');
  assert(merged.nested.x === 1 && merged.nested.y === 99, 'nested object merges: untouched sibling survives, overlaid leaf wins');
  assert(Array.isArray(merged.list) && merged.list.length === 1 && merged.list[0] === 7, 'array overlay replaces wholesale, never concatenates');
  assert(mergeConfigOverlay(base, undefined) === base, 'an undefined overlay returns the base object unchanged (identity, not a copy)');
  assert(mergeConfigOverlay(base, null) === base, 'a null overlay returns the base object unchanged');
});

await check(
  'grep-audit: production writers of the legacy .bee/HANDOFF.json are exactly {rebuildHandoffProjection, writeHandoff C1 fallback, adoptHandoff C1 fallback} (multisession-native-24, advisor slice 5 condition E)',
  () => {
    const packagesBeeRoot = fileURLToPath(new URL('..', import.meta.url));
    const repoRoot = findRepoRoot(packagesBeeRoot);
    if (!repoRoot) return; // no repo context to census against (bare checkout)

    const libDir = path.join(repoRoot, 'packages', 'bee', 'lib');
    const beeMjsPath = path.join(repoRoot, 'packages', 'bee', 'bee.mjs');

    // A "mutation" of the legacy file: writeJsonAtomic(handoffPath(...)) or
    // fs.rmSync(handoffPath(...)) — the two primitives every sanctioned
    // writer uses (readHandoff's own readJson(handoffPath(...)) never
    // matches, so read-only call sites never trip this audit).
    const MUTATION_RE = /(?:writeJsonAtomic|fs\.rmSync)\(\s*handoffPath\(/g;

    function enclosingFunctionName(text, matchIndex) {
      // Walk backwards from the match to the nearest preceding top-level
      // `[export] [async] function <name>(` declaration — sufficient for
      // this file's flat, non-nested-function style; a mis-attribution
      // would surface as an unexpected name in the assertion below, not a
      // silent pass.
      const before = text.slice(0, matchIndex);
      const fnRe = /(?:export\s+)?(?:async\s+)?function\s+([A-Za-z0-9_]+)\s*\(/g;
      let name = null;
      let m;
      while ((m = fnRe.exec(before))) name = m[1];
      return name;
    }

    const hits = [];
    for (const file of fs.readdirSync(libDir).filter((f) => f.endsWith('.mjs'))) {
      const text = fs.readFileSync(path.join(libDir, file), 'utf8');
      let m;
      MUTATION_RE.lastIndex = 0;
      while ((m = MUTATION_RE.exec(text))) hits.push({ file, fn: enclosingFunctionName(text, m.index) });
    }
    // bee.mjs must never mutate the legacy file directly — every mutation
    // goes through one of the sanctioned lib functions above.
    if (fs.existsSync(beeMjsPath)) {
      const beeText = fs.readFileSync(beeMjsPath, 'utf8');
      let m;
      MUTATION_RE.lastIndex = 0;
      while ((m = MUTATION_RE.exec(beeText))) hits.push({ file: 'bee.mjs', fn: enclosingFunctionName(beeText, m.index) });
    }

    assert(hits.length > 0, 'grep-audit found zero handoffPath mutation sites — a broken regex would silently pass this sweep');

    const SANCTIONED = new Set([
      'state.mjs::writeHandoff',
      'state.mjs::adoptHandoff',
      'state-projection.mjs::rebuildHandoffProjection',
    ]);
    const unsanctioned = hits.map((h) => `${h.file}::${h.fn}`).filter((key) => !SANCTIONED.has(key));

    assert(
      unsanctioned.length === 0,
      `unsanctioned .bee/HANDOFF.json writer(s) found: ${unsanctioned.join(', ')} — the only allowed writers are ` +
        'rebuildHandoffProjection (state-projection.mjs) and the C1 no-workflow-records fallback pair ' +
        'writeHandoff/adoptHandoff (state.mjs, each carrying a dated deprecation note).',
    );
  },
);

printSummaryAndExit();
