#!/usr/bin/env node
// Proves the `bee worktree` CLI group (worktree-feature-parallelism Slice A)
// end-to-end against a REAL temp git repo + real `git worktree add`: register
// grants the current linked worktree its own store and bootstraps it, list
// reflects the registry, unregister removes the grant and resolveRoots falls
// back to main again. Runs the real dispatcher via spawnSync (no mocking),
// mirroring scripts/tests/test_worktree_grant_resolve.mjs's fixture pattern.

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { resolveRoots } from '../../.bee/bin/lib/state.mjs';
import { createFeatureWorktree, mergeFeatureWorktree } from '../../.bee/bin/lib/worktree-store.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..', '..');
const BEE_MJS = path.join(REPO_ROOT, '.bee', 'bin', 'bee.mjs');

const results = [];
function record(desc, passed, detail) {
  results.push({ desc, passed });
  console.log((passed ? 'PASS ' : 'FAIL ') + desc + (passed ? '' : ` -- ${detail}`));
}

function git(cwd, args) {
  const r = spawnSync('git', args, { cwd, encoding: 'utf8' });
  if (r.status !== 0) throw new Error(`git ${args.join(' ')} (cwd=${cwd}) failed: ${r.stderr}`);
  return r.stdout;
}

function bee(cwd, args) {
  return spawnSync('node', [BEE_MJS, ...args], { cwd, encoding: 'utf8' });
}

function verifiedId(wtPath) {
  const gitFile = fs.readFileSync(path.join(wtPath, '.git'), 'utf8').trim();
  const m = gitFile.match(/^gitdir:\s*(.+)$/);
  const gitdir = path.resolve(wtPath, m[1].trim());
  return path.basename(gitdir);
}

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-worktree-cli-'));
try {
  const main = path.join(tmp, 'main');
  fs.mkdirSync(main);
  git(main, ['init', '-q', '-b', 'main']);
  git(main, ['config', 'user.email', 's@e']);
  git(main, ['config', 'user.name', 's']);
  fs.writeFileSync(path.join(main, 'f'), 'x');
  git(main, ['add', '.']);
  git(main, ['commit', '-q', '-m', 'init']);

  const mainBeeDir = path.join(main, '.bee');
  fs.mkdirSync(mainBeeDir, { recursive: true });
  fs.writeFileSync(path.join(mainBeeDir, 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.0.0' }));
  fs.writeFileSync(path.join(mainBeeDir, 'config.json'), JSON.stringify({ commands: {} }));

  const wt = path.join(tmp, 'wt');
  git(main, ['worktree', 'add', '-q', '-b', 'feat', wt]);
  const id = verifiedId(wt);
  const grantsFile = path.join(mainBeeDir, 'runtime', 'worktree-grants.json');
  const fileInWt = path.join(wt, 'src.txt');

  // ── before register: resolveRoots reports linked-valid but falls back to
  // main (P40 default) — the grant does not exist yet. ─────────────────────
  {
    const r = resolveRoots(fileInWt);
    const ok = r.worktreeResolution === 'linked-valid' && fs.realpathSync(r.storeRoot) === fs.realpathSync(main);
    record('before register: resolveRoots falls back to the main store (not yet granted)', ok, JSON.stringify(r));
  }

  // ── register ────────────────────────────────────────────────────────────
  const registerResult = bee(wt, ['worktree', 'register', '--feature', 'demo-feature', '--json']);
  {
    const ok = registerResult.status === 0;
    record('worktree register exits 0', ok, `status=${registerResult.status} stdout=${registerResult.stdout} stderr=${registerResult.stderr}`);
  }
  let registerJson = null;
  try {
    registerJson = JSON.parse(registerResult.stdout);
  } catch {
    /* checked below */
  }
  {
    const ok = registerJson && registerJson.ok === true && registerJson.id === id && registerJson.bootstrap && registerJson.bootstrap.created === true;
    record('worktree register --json reports ok:true, the git-verified id, and bootstrap.created:true', ok, registerResult.stdout);
  }

  // ── the grant landed in the MAIN store's registry ─────────────────────────
  {
    const ok = fs.existsSync(grantsFile) && JSON.parse(fs.readFileSync(grantsFile, 'utf8'))[id] === true;
    record('grant appears in <main>/.bee/runtime/worktree-grants.json', ok, fs.existsSync(grantsFile) ? fs.readFileSync(grantsFile, 'utf8') : 'grants file missing');
  }

  // ── the worktree's own .bee/state.json was bootstrapped ──────────────────
  const worktreeStateFile = path.join(wt, '.bee', 'state.json');
  {
    const ok = fs.existsSync(worktreeStateFile);
    record("worktree's .bee/state.json was bootstrapped", ok, worktreeStateFile);
  }
  let worktreeState = null;
  if (fs.existsSync(worktreeStateFile)) worktreeState = JSON.parse(fs.readFileSync(worktreeStateFile, 'utf8'));
  {
    const ok =
      worktreeState &&
      worktreeState.feature === 'demo-feature' &&
      worktreeState.phase === 'idle' &&
      worktreeState.approved_gates &&
      Object.values(worktreeState.approved_gates).every((g) => g === false);
    record(
      'bootstrapped state.json has feature=demo-feature, phase=idle, every gate false',
      ok,
      JSON.stringify(worktreeState),
    );
  }
  {
    const ok = fs.existsSync(path.join(wt, '.bee', 'onboarding.json'));
    record("bootstrap copied onboarding.json from the main store", ok, path.join(wt, '.bee', 'onboarding.json'));
  }

  // ── resolveRoots(<file in worktree>) now resolves to the worktree's OWN
  // store, not main. ─────────────────────────────────────────────────────
  {
    const r = resolveRoots(fileInWt);
    const ok =
      r.worktreeResolution === 'linked-valid' &&
      r.storeRoot &&
      fs.realpathSync(r.storeRoot) === fs.realpathSync(wt) &&
      fs.realpathSync(r.storeRoot) !== fs.realpathSync(main) &&
      r.id === id;
    record('after register: resolveRoots resolves the worktree file to its OWN local store', ok, JSON.stringify(r));
  }

  // ── register is idempotent: re-running does not clobber the existing
  // worktree state.json. ────────────────────────────────────────────────────
  fs.writeFileSync(worktreeStateFile, `${JSON.stringify({ ...worktreeState, phase: 'swarming', sentinel: 'do-not-clobber' }, null, 2)}\n`);
  const reregisterResult = bee(wt, ['worktree', 'register', '--feature', 'demo-feature', '--json']);
  {
    const ok = reregisterResult.status === 0;
    record('re-running worktree register exits 0 (idempotent, not an error)', ok, `status=${reregisterResult.status} stderr=${reregisterResult.stderr}`);
  }
  {
    let reregisterJson = null;
    try {
      reregisterJson = JSON.parse(reregisterResult.stdout);
    } catch {
      /* checked below */
    }
    const ok = reregisterJson && reregisterJson.bootstrap && reregisterJson.bootstrap.created === false;
    record('re-running worktree register reports bootstrap.created:false (skipped)', ok, reregisterResult.stdout);
  }
  {
    const afterState = JSON.parse(fs.readFileSync(worktreeStateFile, 'utf8'));
    const ok = afterState.sentinel === 'do-not-clobber' && afterState.phase === 'swarming';
    record('re-running worktree register never overwrote the existing worktree state.json', ok, JSON.stringify(afterState));
  }
  // restore a clean state.json for the rest of the run (undo the sentinel mutation)
  fs.writeFileSync(worktreeStateFile, `${JSON.stringify(worktreeState, null, 2)}\n`);

  // ── list reflects the grant ───────────────────────────────────────────────
  const listResult = bee(wt, ['worktree', 'list', '--json']);
  {
    const ok = listResult.status === 0;
    record('worktree list exits 0', ok, `status=${listResult.status} stderr=${listResult.stderr}`);
  }
  {
    let listJson = null;
    try {
      listJson = JSON.parse(listResult.stdout);
    } catch {
      /* checked below */
    }
    const ok = listJson && listJson.grants && listJson.grants[id] === true;
    record('worktree list --json includes the registered id', ok, listResult.stdout);
  }

  // ── unregister (no --id: defaults to the current worktree's own id) ──────
  const unregisterResult = bee(wt, ['worktree', 'unregister', '--json']);
  {
    const ok = unregisterResult.status === 0;
    record('worktree unregister (no --id) exits 0', ok, `status=${unregisterResult.status} stdout=${unregisterResult.stdout} stderr=${unregisterResult.stderr}`);
  }
  {
    let unregisterJson = null;
    try {
      unregisterJson = JSON.parse(unregisterResult.stdout);
    } catch {
      /* checked below */
    }
    const ok = unregisterJson && unregisterJson.ok === true && unregisterJson.id === id;
    record('worktree unregister --json reports ok:true and the removed id', ok, unregisterResult.stdout);
  }
  {
    const grants = JSON.parse(fs.readFileSync(grantsFile, 'utf8'));
    const ok = !(id in grants);
    record('grant is gone from <main>/.bee/runtime/worktree-grants.json after unregister', ok, JSON.stringify(grants));
  }

  // ── list reflects the removal ─────────────────────────────────────────────
  const listAfterResult = bee(wt, ['worktree', 'list', '--json']);
  {
    const listAfterJson = JSON.parse(listAfterResult.stdout);
    const ok = listAfterJson.grants[id] !== true;
    record('worktree list --json no longer shows the id as granted', ok, listAfterResult.stdout);
  }

  // ── after unregister: resolveRoots falls back to main again ──────────────
  {
    const r = resolveRoots(fileInWt);
    const ok = r.storeRoot && fs.realpathSync(r.storeRoot) === fs.realpathSync(main);
    record('after unregister: resolveRoots falls back to the main store again', ok, JSON.stringify(r));
  }

  // ── register from an ordinary (non-worktree) checkout fails with a typed
  // error, never crashes uncaught. ──────────────────────────────────────────
  const ordinaryRegisterResult = bee(main, ['worktree', 'register', '--feature', 'x', '--json']);
  {
    const ok = ordinaryRegisterResult.status !== 0 && !/at Object|TypeError|ReferenceError/.test(ordinaryRegisterResult.stderr);
    record(
      'worktree register from an ordinary checkout fails with a typed (non-crash) error',
      ok,
      `status=${ordinaryRegisterResult.status} stdout=${ordinaryRegisterResult.stdout} stderr=${ordinaryRegisterResult.stderr}`,
    );
  }

  // ── unregister --id <explicit> works from anywhere (e.g. from main) ──────
  bee(wt, ['worktree', 'register', '--feature', 'demo-feature-2', '--json']);
  const explicitUnregisterResult = bee(main, ['worktree', 'unregister', '--id', id, '--json']);
  {
    const ok = explicitUnregisterResult.status === 0;
    record('worktree unregister --id <explicit> from the main checkout exits 0', ok, `status=${explicitUnregisterResult.status} stderr=${explicitUnregisterResult.stderr}`);
  }
  {
    const grants = JSON.parse(fs.readFileSync(grantsFile, 'utf8'));
    const ok = !(id in grants);
    record('explicit --id unregister removed the grant', ok, JSON.stringify(grants));
  }

  // ── GH #30 (wux-1): `bee status` inside an UNGRANTED linked worktree
  // (resolveRoots reports 'linked-valid' with storeRoot falling back to
  // mainRoot — exactly `wt`'s state right now, confirmed above) prints a loud
  // shares-main-store notice, in BOTH text and --json (`worktree_notice`
  // field). An ORDINARY checkout (main) and a GRANTED linked worktree must
  // show NO such notice at all — byte-identical to pre-cell output. ────────
  {
    const ordinaryStatusResult = bee(main, ['status', '--json']);
    let ordinaryStatusJson = null;
    try {
      ordinaryStatusJson = JSON.parse(ordinaryStatusResult.stdout);
    } catch {
      /* checked below */
    }
    const ok = ordinaryStatusResult.status === 0 && ordinaryStatusJson && !('worktree_notice' in ordinaryStatusJson);
    record('bee status --json from the ordinary (main) checkout has NO worktree_notice field', ok, ordinaryStatusResult.stdout);

    const ordinaryStatusText = bee(main, ['status']);
    const textOk = ordinaryStatusText.status === 0 && !/UNGRANTED/.test(ordinaryStatusText.stdout);
    record('bee status text from the ordinary (main) checkout has NO ungranted-worktree notice', textOk, ordinaryStatusText.stdout);
  }
  {
    const ungrantedStatusResult = bee(wt, ['status', '--json']);
    let ungrantedStatusJson = null;
    try {
      ungrantedStatusJson = JSON.parse(ungrantedStatusResult.stdout);
    } catch {
      /* checked below */
    }
    const ok =
      ungrantedStatusResult.status === 0 &&
      ungrantedStatusJson &&
      typeof ungrantedStatusJson.worktree_notice === 'string' &&
      /UNGRANTED/.test(ungrantedStatusJson.worktree_notice) &&
      /SHARES the main checkout's store/.test(ungrantedStatusJson.worktree_notice) &&
      /bee worktree new --feature/.test(ungrantedStatusJson.worktree_notice) &&
      /bee worktree register/.test(ungrantedStatusJson.worktree_notice);
    record(
      'bee status --json from an UNGRANTED linked worktree carries worktree_notice naming both remedies',
      ok,
      ungrantedStatusResult.stdout,
    );

    const ungrantedStatusText = bee(wt, ['status']);
    const textOk =
      ungrantedStatusText.status === 0 &&
      /UNGRANTED/.test(ungrantedStatusText.stdout) &&
      /SHARES the main checkout's store/.test(ungrantedStatusText.stdout) &&
      /bee worktree new --feature/.test(ungrantedStatusText.stdout) &&
      /bee worktree register/.test(ungrantedStatusText.stdout);
    record('bee status text from an UNGRANTED linked worktree prints the same notice', textOk, ungrantedStatusText.stdout);
  }
  {
    // grant `wt` so the SAME worktree, now granted, proves the notice is
    // grant-state-driven, not path-driven — then restore ungranted (matches
    // the invariant already established above, for whatever runs next).
    const grantResult = bee(wt, ['worktree', 'register', '--feature', 'wux-status-check', '--json']);
    const grantOk = grantResult.status === 0;
    record('(setup) re-registering wt to prove the granted case exits 0', grantOk, `status=${grantResult.status} stderr=${grantResult.stderr}`);

    const grantedStatusResult = bee(wt, ['status', '--json']);
    let grantedStatusJson = null;
    try {
      grantedStatusJson = JSON.parse(grantedStatusResult.stdout);
    } catch {
      /* checked below */
    }
    const ok = grantedStatusResult.status === 0 && grantedStatusJson && !('worktree_notice' in grantedStatusJson);
    record('bee status --json from a GRANTED linked worktree has NO worktree_notice field', ok, grantedStatusResult.stdout);

    const grantedStatusText = bee(wt, ['status']);
    const textOk = grantedStatusText.status === 0 && !/UNGRANTED/.test(grantedStatusText.stdout);
    record('bee status text from a GRANTED linked worktree has NO ungranted-worktree notice', textOk, grantedStatusText.stdout);

    const unregisterBackResult = bee(main, ['worktree', 'unregister', '--id', id, '--json']);
    record('(teardown) unregistering wt back to ungranted exits 0', unregisterBackResult.status === 0, `status=${unregisterBackResult.status} stderr=${unregisterBackResult.stderr}`);
  }

  // ── worktree new: create + register in one move (wsr-1, GH #21) ──────────
  const newFeature = 'wsr-new-demo';
  const newSibling = path.join(tmp, `${path.basename(main)}--wt--${newFeature}`);
  const newBranch = `wt/${newFeature}`;

  const newResult = bee(main, ['worktree', 'new', '--feature', newFeature, '--json']);
  {
    const ok = newResult.status === 0;
    record('worktree new exits 0', ok, `status=${newResult.status} stdout=${newResult.stdout} stderr=${newResult.stderr}`);
  }
  let newJson = null;
  try {
    newJson = JSON.parse(newResult.stdout);
  } catch {
    /* checked below */
  }
  let newId = null;
  {
    const ok = newJson && typeof newJson.id === 'string' && newJson.worktreeRoot && newJson.branch === newBranch;
    newId = newJson && newJson.id;
    record('worktree new --json reports id, worktreeRoot, and branch "wt/<feature>"', ok, newResult.stdout);
  }
  // ── GH #31 (wux-1): success output names the explicit next step — open a
  // NEW session cwd'd into the created worktree; this session stays on main;
  // merge back later with the exact "bee worktree merge --id <id>" command.
  // Both the --json `next_step` field and the text output carry it. ────────
  {
    const ok =
      newJson &&
      typeof newJson.next_step === 'string' &&
      newJson.next_step.includes(newJson.worktreeRoot) &&
      /open a new session/i.test(newJson.next_step) &&
      /stays on main/.test(newJson.next_step) &&
      newJson.next_step.includes(`bee worktree merge --id ${newJson.id}`);
    record('worktree new --json reports a next_step naming the worktree path and the merge-back command', ok, newResult.stdout);
  }
  {
    const newTextFeature = `${newFeature}-text`;
    const newTextSibling = path.join(tmp, `${path.basename(main)}--wt--${newTextFeature}`);
    const newTextResult = bee(main, ['worktree', 'new', '--feature', newTextFeature]);
    const textOk =
      newTextResult.status === 0 &&
      newTextResult.stdout.includes(newTextSibling) &&
      /open a new session/i.test(newTextResult.stdout) &&
      /stays on main/.test(newTextResult.stdout) &&
      /bee worktree merge --id /.test(newTextResult.stdout);
    record('worktree new text output names the next step (open a session, stays on main, merge-back command)', textOk, newTextResult.stdout);
  }
  {
    const ok = fs.existsSync(newSibling);
    record('worktree new created the sibling directory ../<repo-basename>--wt--<feature>', ok, newSibling);
  }
  {
    const newStateFile = path.join(newSibling, '.bee', 'state.json');
    const stateExists = fs.existsSync(newStateFile);
    let state = null;
    if (stateExists) state = JSON.parse(fs.readFileSync(newStateFile, 'utf8'));
    const ok =
      stateExists &&
      state.feature === newFeature &&
      state.phase === 'idle' &&
      state.approved_gates &&
      Object.values(state.approved_gates).every((g) => g === false);
    record("worktree new's created worktree has a bootstrapped idle .bee/state.json", ok, JSON.stringify(state));
  }
  {
    const ok = fs.existsSync(path.join(newSibling, '.bee', 'onboarding.json'));
    record('worktree new bootstrap copied onboarding.json from the main store', ok, path.join(newSibling, '.bee', 'onboarding.json'));
  }

  // ── worktree list shows the grant "worktree new" created ─────────────────
  {
    const listResult2 = bee(main, ['worktree', 'list', '--json']);
    let listJson2 = null;
    try {
      listJson2 = JSON.parse(listResult2.stdout);
    } catch {
      /* checked below */
    }
    const ok = listResult2.status === 0 && listJson2 && newId && listJson2.grants[newId] === true;
    record('worktree list shows the grant "worktree new" created', ok, listResult2.stdout);
  }

  // ── repeated "new" for the SAME feature typed-refuses (target dir already
  // exists), zero mutation. ─────────────────────────────────────────────────
  {
    const grantsSnapshot = fs.readFileSync(grantsFile, 'utf8');
    const repeatResult = bee(main, ['worktree', 'new', '--feature', newFeature, '--json']);
    const ok = repeatResult.status !== 0 && /WORKTREE_TARGET_EXISTS/.test(repeatResult.stdout + repeatResult.stderr);
    record(
      'repeated "worktree new" for the same feature typed-refuses (WORKTREE_TARGET_EXISTS)',
      ok,
      `status=${repeatResult.status} stdout=${repeatResult.stdout} stderr=${repeatResult.stderr}`,
    );
    const grantsAfter = fs.readFileSync(grantsFile, 'utf8');
    record('repeated "worktree new" refusal is zero-mutation (grants file unchanged)', grantsAfter === grantsSnapshot, grantsAfter);
  }

  // ── bad slugs refuse (WORKTREE_INVALID_SLUG), zero mutation ──────────────
  const badSlugs = ['../../etc', '-leading-dash', 'Uppercase', 'has space', 'semi;colon', 'trailing/'];
  for (const badSlug of badSlugs) {
    const before = fs.readFileSync(grantsFile, 'utf8');
    const beforeEntries = fs.readdirSync(tmp).sort();
    const badResult = bee(main, ['worktree', 'new', '--feature', badSlug, '--json']);
    const ok = badResult.status !== 0 && /WORKTREE_INVALID_SLUG/.test(badResult.stdout + badResult.stderr);
    record(
      `worktree new refuses bad slug ${JSON.stringify(badSlug)} (WORKTREE_INVALID_SLUG)`,
      ok,
      `status=${badResult.status} stdout=${badResult.stdout} stderr=${badResult.stderr}`,
    );
    const after = fs.readFileSync(grantsFile, 'utf8');
    const afterEntries = fs.readdirSync(tmp).sort();
    const mutationOk = after === before && JSON.stringify(afterEntries) === JSON.stringify(beforeEntries);
    record(
      `worktree new bad slug ${JSON.stringify(badSlug)} refusal is zero-mutation (no new dir, grants unchanged)`,
      mutationOk,
      `before=${JSON.stringify(beforeEntries)} after=${JSON.stringify(afterEntries)}`,
    );
  }

  // ── running "new" from inside a linked worktree refuses (typed, non-crash),
  // zero mutation. ──────────────────────────────────────────────────────────
  {
    const before = fs.readFileSync(grantsFile, 'utf8');
    const beforeEntries = fs.readdirSync(tmp).sort();
    const fromWtResult = bee(wt, ['worktree', 'new', '--feature', 'wsr-from-linked', '--json']);
    const ok = fromWtResult.status !== 0 && !/at Object|TypeError|ReferenceError/.test(fromWtResult.stderr);
    record(
      'worktree new run from inside a linked worktree refuses (typed, non-crash)',
      ok,
      `status=${fromWtResult.status} stdout=${fromWtResult.stdout} stderr=${fromWtResult.stderr}`,
    );
    const after = fs.readFileSync(grantsFile, 'utf8');
    const afterEntries = fs.readdirSync(tmp).sort();
    const mutationOk = after === before && JSON.stringify(afterEntries) === JSON.stringify(beforeEntries);
    record('worktree new run from inside a linked worktree is zero-mutation', mutationOk, `before=${JSON.stringify(beforeEntries)} after=${JSON.stringify(afterEntries)}`);
  }

  // ── --base-ref is honored (branches off an explicit ref, not just HEAD) ──
  {
    const baseRefFeature = 'wsr-base-ref-demo';
    const baseRefResult = bee(main, ['worktree', 'new', '--feature', baseRefFeature, '--base-ref', 'main', '--json']);
    const ok = baseRefResult.status === 0;
    record('worktree new --base-ref main exits 0', ok, `status=${baseRefResult.status} stdout=${baseRefResult.stdout} stderr=${baseRefResult.stderr}`);
    let baseRefJson = null;
    try {
      baseRefJson = JSON.parse(baseRefResult.stdout);
    } catch {
      /* checked below */
    }
    const mainHeadSha = git(main, ['rev-parse', 'main']).trim();
    const shaOk = baseRefJson && baseRefJson.baseRefSha === mainHeadSha;
    record('worktree new --base-ref main reports the RESOLVED sha (not the ref string) in JSON output', shaOk, baseRefResult.stdout);
  }

  // ── p162-3 (decision D3 / advisor R8): base-ref is validated via `git
  // rev-parse --verify --end-of-options "<ref>^{commit}"`, NOT `git
  // check-ref-format` — check-ref-format only checks ref-NAME syntax and
  // wrongly rejects/mishandles commit-ish forms like `HEAD~1`, a short sha,
  // and `<tag>^{commit}`. A second real commit + a tag give every accepted
  // form something concrete to resolve against. ────────────────────────────
  git(main, ['commit', '--allow-empty', '-q', '-m', 'second commit for base-ref resolution tests']);
  git(main, ['tag', 'v1.2.0']);
  const baseRefShortSha = git(main, ['rev-parse', '--short', 'HEAD']).trim();
  const baseRefHeadSha = git(main, ['rev-parse', 'HEAD']).trim();
  const baseRefParentSha = git(main, ['rev-parse', 'HEAD~1']).trim();

  const acceptedBaseRefs = [
    { ref: 'HEAD', label: 'HEAD', expectSha: baseRefHeadSha },
    { ref: 'HEAD~1', label: 'HEAD~1', expectSha: baseRefParentSha },
    { ref: baseRefShortSha, label: 'a short sha', expectSha: baseRefHeadSha },
    { ref: 'v1.2.0^{commit}', label: 'a tag with ^{commit}', expectSha: baseRefHeadSha },
  ];
  acceptedBaseRefs.forEach(({ ref, label, expectSha }, i) => {
    const feature = `wsr-baseref-ok-${i}`;
    const result = bee(main, ['worktree', 'new', '--feature', feature, '--base-ref', ref, '--json']);
    const ok = result.status === 0;
    record(`worktree new --base-ref ${JSON.stringify(ref)} (${label}) exits 0`, ok, `status=${result.status} stdout=${result.stdout} stderr=${result.stderr}`);
    let json = null;
    try {
      json = JSON.parse(result.stdout);
    } catch {
      /* checked below */
    }
    const shaOk = json && json.baseRefSha === expectSha;
    record(`worktree new --base-ref ${JSON.stringify(ref)} (${label}) resolves to the expected commit sha`, shaOk, `expected=${expectSha} got=${json && json.baseRefSha} stdout=${result.stdout}`);
  });

  // ── nonexistent ref, syntactically-invalid garbage, and a leading-dash
  // injection string all refuse the SAME way: typed WORKTREE_BASE_NOT_FOUND,
  // zero mutation. There is no cheap syntax-only pre-check left standing
  // once validation is `rev-parse --verify` — a bad-syntax ref and a
  // well-formed-but-missing ref both fail resolution identically, so the
  // old WORKTREE_INVALID_BASE_REF code (which assumed a syntax check could
  // be told apart from an existence check) is retired; everything that
  // fails to resolve collapses into WORKTREE_BASE_NOT_FOUND. ───────────────
  const rejectedBaseRefs = [
    { ref: 'nonexistent-ref-xyz-zzz', label: 'a nonexistent ref' },
    { ref: 'not a ref!!', label: 'syntactically invalid garbage' },
    { ref: '--upload-pack=/bin/sh', label: 'a leading-dash injection string' },
  ];
  rejectedBaseRefs.forEach(({ ref, label }, i) => {
    const feature = `wsr-baseref-bad-${i}`;
    const before = fs.readFileSync(grantsFile, 'utf8');
    const beforeEntries = fs.readdirSync(tmp).sort();
    const result = bee(main, ['worktree', 'new', '--feature', feature, '--base-ref', ref, '--json']);
    const ok = result.status !== 0 && /WORKTREE_BASE_NOT_FOUND/.test(result.stdout + result.stderr);
    record(`worktree new refuses --base-ref ${JSON.stringify(ref)} (${label}) typed WORKTREE_BASE_NOT_FOUND`, ok, `status=${result.status} stdout=${result.stdout} stderr=${result.stderr}`);
    const after = fs.readFileSync(grantsFile, 'utf8');
    const afterEntries = fs.readdirSync(tmp).sort();
    const mutationOk = after === before && JSON.stringify(afterEntries) === JSON.stringify(beforeEntries);
    record(`worktree new --base-ref ${JSON.stringify(ref)} (${label}) refusal is zero-mutation`, mutationOk, `before=${JSON.stringify(beforeEntries)} after=${JSON.stringify(afterEntries)}`);
  });

  // ── injected post-add failure rolls back (dir + grant + branch) and
  // reports typed — exercised directly against createFeatureWorktree (the
  // CLI's argv surface has no fault-injection hook), reusing the SAME real
  // temp repo the rest of this script already proved `bee worktree new`
  // against. ─────────────────────────────────────────────────────────────
  {
    const rollbackFeature = 'wsr-rollback-demo';
    const rollbackSibling = path.join(tmp, `${path.basename(main)}--wt--${rollbackFeature}`);
    const rollbackBranch = `wt/${rollbackFeature}`;
    let caught = null;
    try {
      // hardening-4b: createFeatureWorktree is now async (withStoreLock-
      // wrapped) — its throw is a rejected promise now, so this needs an
      // await in front of it to still land in the catch below (same reason
      // mergeFeatureWorktree needed one at xwh-2, see the comment near its
      // own await a few hundred lines down).
      await createFeatureWorktree(main, {
        feature: rollbackFeature,
        _bootstrapWorktreeStore: () => {
          throw new Error('injected-bootstrap-failure');
        },
      });
    } catch (error) {
      caught = error;
    }
    {
      const ok = caught && caught.code === 'WORKTREE_POST_ADD_FAILED' && /injected-bootstrap-failure/.test(caught.message);
      record(
        'injected post-add failure throws a typed WORKTREE_POST_ADD_FAILED error naming the underlying cause',
        ok,
        caught ? `${caught.code}: ${caught.message}` : 'no error thrown',
      );
    }
    {
      const ok = !fs.existsSync(rollbackSibling);
      record('injected post-add failure rolls back: the sibling directory was removed', ok, rollbackSibling);
    }
    {
      const branchList = git(main, ['branch', '--list', rollbackBranch]).trim();
      record('injected post-add failure rolls back: the branch was removed too', branchList.length === 0, `git branch --list ${rollbackBranch} -> "${branchList}"`);
    }
    {
      const grants = JSON.parse(fs.readFileSync(grantsFile, 'utf8'));
      const likelyId = `${path.basename(main)}--wt--${rollbackFeature}`;
      const ok = !(likelyId in grants);
      record('injected post-add failure rolls back: no grant was left behind', ok, JSON.stringify(grants));
    }
  }
} finally {
  fs.rmSync(tmp, { recursive: true, force: true });
}

// ─── "bee worktree merge --id <id>" (GH #21, decision D8; reworked into a
// STAGED transaction by decision D2-REVISED, user review P1-2): merge-back
// with the semantic-conflict alarm (wsr-2) ─────────────────────────────────
// Every fixture below is set up so its own assertions are self-contained —
// none of them depend on state a PRIOR test happened to leave behind
// (advisor R6 fixture-chain warning). This matters concretely for mainA: the
// old contract's "merge commit is never rolled back" meant a red-verify test
// that flipped flag.txt to "on" left it flipped for every later test on the
// same fixture; under the new abort-on-red contract that flip is undone the
// moment bee runs "git merge --abort", so any later test that wants a red
// verify creates its OWN worktree that flips flag.txt itself, rather than
// reusing residue from an earlier case.
//
// A production-shaped .gitignore (byte-copy of this repo's own top-level
// .bee/* ignore rules) is committed into the fixture main repo so a freshly
// bootstrapped worktree store's cache/runtime-tier files (state.json,
// runtime/) are genuinely invisible to "git status --porcelain" — the same
// way they are in the real repo — while onboarding.json/config.json are
// TRACKED (inherited by every worktree via the normal git checkout), proving
// decision D8a for real rather than special-casing the test.
const BEE_GITIGNORE = [
  '.bee/state.json',
  '.bee/reservations.json',
  '.bee/workers/',
  '.bee/logs/',
  '.bee/capture-queue.jsonl',
  '.bee/feedback-digest.json',
  '.bee/.inject-cache.json',
  '.bee/HANDOFF.json',
  '.bee/spikes/',
  '.bee/manifest-hash.json',
  '.bee/sessions/',
  '.bee/claims/',
  '.bee/runtime/',
  '.bee/cache/',
  // hardening-4b: withStoreLock's lockfiles (.bee/locks/*.lock) — the
  // worktree-admin mutex now HOLDS its lock for the entire create/merge
  // operation, so leaving this ignored would make the lock file itself
  // register as untracked dirt to the D8a `git status --porcelain` checks
  // this same operation runs.
  '.bee/locks/',
  '',
].join('\n');

function initMergeFixtureMain(main, { verifyScript } = {}) {
  fs.mkdirSync(main, { recursive: true });
  git(main, ['init', '-q', '-b', 'main']);
  git(main, ['config', 'user.email', 's@e']);
  git(main, ['config', 'user.name', 's']);
  fs.writeFileSync(path.join(main, '.gitignore'), BEE_GITIGNORE);
  fs.mkdirSync(path.join(main, '.bee'), { recursive: true });
  fs.writeFileSync(path.join(main, '.bee', 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.0.0' }));
  const config = verifyScript ? { commands: { verify: `node ${verifyScript}` } } : { commands: {} };
  fs.writeFileSync(path.join(main, '.bee', 'config.json'), JSON.stringify(config));
  fs.writeFileSync(path.join(main, 'f'), 'x');
  if (verifyScript) {
    fs.writeFileSync(
      path.join(main, verifyScript),
      "import fs from 'node:fs';\nconst v = fs.readFileSync('flag.txt', 'utf8').trim();\nprocess.exit(v === 'off' ? 0 : 1);\n",
    );
    fs.writeFileSync(path.join(main, 'flag.txt'), 'off');
  }
  git(main, ['add', '.']);
  git(main, ['commit', '-q', '-m', 'init']);
}

function mergeNewWorktree(main, feature) {
  const r = bee(main, ['worktree', 'new', '--feature', feature, '--json']);
  if (r.status !== 0) throw new Error(`worktree new --feature ${feature} failed: status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
  return JSON.parse(r.stdout);
}

// The three-part "main was left byte-untouched" proof (decision D2-REVISED)
// bee itself runs after every "git merge --abort" — asserted independently
// here rather than trusted, exactly like the library code does it: HEAD
// unchanged, no live MERGE_HEAD/MERGE_MSG, and a clean tracked status.
function threePartProof(main, preHead) {
  const headNow = git(main, ['rev-parse', 'HEAD']).trim();
  const mergeHeadGone = !fs.existsSync(path.join(main, '.git', 'MERGE_HEAD'));
  const mergeMsgGone = !fs.existsSync(path.join(main, '.git', 'MERGE_MSG'));
  const statusClean = git(main, ['status', '--porcelain', '--untracked-files=no']).trim() === '';
  return {
    ok: headNow === preHead && mergeHeadGone && mergeMsgGone && statusClean,
    headNow,
    mergeHeadGone,
    mergeMsgGone,
    statusClean,
  };
}

const mergeTmp = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-worktree-merge-'));
try {
  // ── fixture A: verify checks flag.txt === "off" — proves the green path,
  // the semantic-conflict alarm (MERGE_VERIFY_RED), and D8a (bootstrapped
  // store + committed work merges cleanly) all in one repo. ────────────────
  const mainA = path.join(mergeTmp, 'mainA');
  initMergeFixtureMain(mainA, { verifyScript: 'verify.mjs' });

  // ── green path: committed work in the worktree, verify still passes
  // (flag.txt untouched) — also the D8a proof (a bootstrapped .bee store,
  // gitignored, does not make the worktree "dirty"). ────────────────────────
  const greenCreated = mergeNewWorktree(mainA, 'wsr-merge-green');
  fs.writeFileSync(path.join(greenCreated.worktreeRoot, 'feature.txt'), 'hello from the worktree\n');
  git(greenCreated.worktreeRoot, ['add', 'feature.txt']);
  git(greenCreated.worktreeRoot, ['commit', '-q', '-m', 'feature work']);
  {
    const ok =
      fs.existsSync(path.join(greenCreated.worktreeRoot, '.bee', 'state.json')) &&
      !fs.existsSync(path.join(greenCreated.worktreeRoot, '.git')) === false; // sanity: worktree is real
    record('green-path fixture has a bootstrapped .bee/state.json (D8a setup sanity)', ok, greenCreated.worktreeRoot);
  }
  const greenResult = bee(mainA, ['worktree', 'merge', '--id', greenCreated.id, '--json']);
  {
    const ok = greenResult.status === 0;
    record('worktree merge (green path) exits 0', ok, `status=${greenResult.status} stdout=${greenResult.stdout} stderr=${greenResult.stderr}`);
  }
  let greenJson = null;
  try {
    greenJson = JSON.parse(greenResult.stdout);
  } catch {
    /* checked below */
  }
  {
    const ok = greenJson && greenJson.ok === true && greenJson.merged === true && greenJson.verify === 'green' && greenJson.branch === `wt/wsr-merge-green`;
    record(
      'worktree merge --json reports ok:true, merged:true, verify:"green" (D8a: bootstrapped-store-only worktree merges — not dirty)',
      ok,
      greenResult.stdout,
    );
  }
  {
    const log = git(mainA, ['log', '--oneline', '-1']).trim();
    const ok = /Merge worktree/.test(git(mainA, ['log', '-1', '--pretty=%B']));
    record('green-path merge landed a real merge commit on main naming the id', ok, log);
  }
  {
    const ok = fs.existsSync(path.join(mainA, 'feature.txt'));
    record('green-path merge brought the worktree\'s committed file into main', ok, mainA);
  }
  {
    // greenCreated's branch never diverged from main's own history (main
    // gained no commits of its own between "worktree new" and this merge),
    // so this IS the fast-forward-eligible case: "git merge --no-ff
    // --no-commit" must still stage (and, once committed, still yield) a
    // TRUE two-parent merge commit rather than silently fast-forwarding the
    // ref — the explicit --no-ff/--no-commit interaction the rework depends
    // on (main is never left mid-merge, but it also never loses the "always
    // a real merge commit" guarantee `--no-ff` alone used to provide).
    const parents = git(mainA, ['log', '-1', '--pretty=%P']).trim().split(/\s+/).filter(Boolean);
    record('green-path (fast-forward-eligible) merge still produced a TRUE 2-parent merge commit, not a fast-forward', parents.length === 2, `parents=${JSON.stringify(parents)}`);
  }

  // ── semantic-conflict alarm: a SECOND worktree changes flag.txt (no file
  // main also touches, so the merge is textually clean) to "on" — verify
  // runs against the merged-but-UNCOMMITTED tree and goes red BEFORE any
  // commit ever exists, so the merge is aborted and main is proven
  // byte-untouched (decision D2-REVISED — supersedes the old "merge commit
  // is never rolled back" contract: under the new invariant there is no
  // merge commit to roll back in the first place). ──────────────────────────
  const alarmPreHead = git(mainA, ['log', '-1', '--pretty=%H']).trim();
  const alarmCreated = mergeNewWorktree(mainA, 'wsr-merge-alarm');
  fs.writeFileSync(path.join(alarmCreated.worktreeRoot, 'flag.txt'), 'on');
  git(alarmCreated.worktreeRoot, ['commit', '-am', 'flip the flag']);
  const alarmResult = bee(mainA, ['worktree', 'merge', '--id', alarmCreated.id, '--json']);
  {
    const ok = alarmResult.status === 1;
    record('semantic-conflict alarm: worktree merge exits 1 on a red post-merge verify', ok, `status=${alarmResult.status} stdout=${alarmResult.stdout}`);
  }
  let alarmJson = null;
  try {
    alarmJson = JSON.parse(alarmResult.stdout);
  } catch {
    /* checked below */
  }
  {
    const ok = alarmJson && alarmJson.ok === false && alarmJson.code === 'MERGE_VERIFY_RED' && alarmJson.merged === false && alarmJson.verify === 'red' && typeof alarmJson.output_tail === 'string';
    record('semantic-conflict alarm: --json reports ok:false, code:"MERGE_VERIFY_RED", merged:false (never committed), verify:"red", and an output_tail', ok, alarmResult.stdout);
  }
  {
    const ok = /semantic-conflict alarm/.test(JSON.stringify(alarmJson));
    record('semantic-conflict alarm result names it a semantic-conflict alarm in prose', ok, JSON.stringify(alarmJson));
  }
  {
    const ok = /byte-untouched/.test(JSON.stringify(alarmJson)) && /no merge commit exists/.test(JSON.stringify(alarmJson));
    record('semantic-conflict alarm result states in prose that main was left byte-untouched and no merge commit exists', ok, JSON.stringify(alarmJson));
  }
  {
    const proof = threePartProof(mainA, alarmPreHead);
    record('semantic-conflict alarm: the staged merge was aborted and main passes the three-part untouched proof (HEAD unchanged, no MERGE_HEAD/MERGE_MSG, clean status)', proof.ok, JSON.stringify(proof));
  }
  {
    // Do NOT assert "no /Merge worktree/ in the log" here — mainA's HEAD
    // already carries the EARLIER green-path merge commit (also named
    // "Merge worktree ..."), so that phrase legitimately appears regardless
    // of this attempt. HEAD-equality to the pre-attempt value is already
    // proven above by threePartProof; this just re-confirms the visible
    // symptom: flag.txt is back to "off" (the abort undid the staged flip),
    // and this attempt's OWN branch name is nowhere in the log (it was
    // never committed).
    const log = git(mainA, ['log', '-1', '--pretty=%B']);
    const flagContent = fs.readFileSync(path.join(mainA, 'flag.txt'), 'utf8').trim();
    const ok = !/wsr-merge-alarm/.test(log) && flagContent === 'off';
    record('semantic-conflict alarm: no merge commit for THIS attempt landed, and flag.txt reverted to "off" on main (the abort undid the staged flip)', ok, `log=${log} flag=${flagContent}`);
  }

  // ── unknown/ungranted id: typed, zero-mutation. ───────────────────────────
  {
    const before = git(mainA, ['log', '-1', '--pretty=%H']);
    const r = bee(mainA, ['worktree', 'merge', '--id', 'not-a-real-id', '--json']);
    const ok = r.status !== 0 && /WORKTREE_MERGE_UNKNOWN_ID/.test(r.stdout + r.stderr);
    record('worktree merge refuses an unknown/ungranted id (WORKTREE_MERGE_UNKNOWN_ID)', ok, `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
    const after = git(mainA, ['log', '-1', '--pretty=%H']);
    record('unknown-id refusal is zero-mutation (main HEAD unchanged)', before === after, `${before} vs ${after}`);
  }

  // ── fixture B: no commands.verify recorded — used for the conflict case,
  // the remaining pre-flight refusals, and cleanup (verify:'skipped'). ──────
  const mainB = path.join(mergeTmp, 'mainB');
  initMergeFixtureMain(mainB);
  const grantsFileB = path.join(mainB, '.bee', 'runtime', 'worktree-grants.json');

  // ── MERGE_CONFLICT: main and the worktree both edit the SAME line of a
  // tracked file after branching -> a real textual conflict. Under the
  // staged-transaction rework bee itself runs "git merge --abort" and
  // proves main untouched — there is no conflict state left for a human to
  // resolve on main anymore (it never gets that far); a human resolves the
  // conflict in the WORKTREE and retries the merge instead. ────────────────
  const conflictCreated = mergeNewWorktree(mainB, 'wsr-merge-conflict');
  fs.writeFileSync(path.join(conflictCreated.worktreeRoot, 'f'), 'from the worktree\n');
  git(conflictCreated.worktreeRoot, ['commit', '-am', 'worktree edits f']);
  fs.writeFileSync(path.join(mainB, 'f'), 'from main\n');
  git(mainB, ['commit', '-am', 'main edits f too']);
  const conflictPreHead = git(mainB, ['log', '-1', '--pretty=%H']).trim(); // AFTER "main edits f too" landed — the real pre-merge-attempt HEAD
  const conflictResult = bee(mainB, ['worktree', 'merge', '--id', conflictCreated.id, '--json']);
  {
    const ok = conflictResult.status !== 0;
    record('worktree merge exits non-zero on a real textual conflict', ok, `status=${conflictResult.status} stdout=${conflictResult.stdout}`);
  }
  let conflictJson = null;
  try {
    conflictJson = JSON.parse(conflictResult.stdout);
  } catch {
    /* checked below */
  }
  {
    const ok = conflictJson && conflictJson.ok === false && conflictJson.code === 'MERGE_CONFLICT';
    record('worktree merge --json reports ok:false, code:"MERGE_CONFLICT"', ok, conflictResult.stdout);
  }
  {
    const ok = /byte-untouched/.test(JSON.stringify(conflictJson));
    record('MERGE_CONFLICT result states in prose that main was left byte-untouched', ok, JSON.stringify(conflictJson));
  }
  {
    // conflictPreHead was captured AFTER "main edits f too" landed, i.e. the
    // real pre-merge-attempt HEAD — bee already ran "git merge --abort"
    // internally by the time this returns, so main must already be back to
    // that exact state (no manual cleanup abort needed or possible: there is
    // nothing left to abort).
    const proof = threePartProof(mainB, conflictPreHead);
    record('MERGE_CONFLICT: bee already aborted the staged merge and main passes the three-part untouched proof (HEAD unchanged, no MERGE_HEAD/MERGE_MSG, clean status)', proof.ok, JSON.stringify(proof));
  }

  // ── dirty MAIN refuses, zero-mutation. ────────────────────────────────────
  const dirtyMainCreated = mergeNewWorktree(mainB, 'wsr-merge-dirty-main');
  fs.writeFileSync(path.join(mainB, 'f'), 'uncommitted main edit\n');
  {
    const before = git(mainB, ['log', '-1', '--pretty=%H']);
    const r = bee(mainB, ['worktree', 'merge', '--id', dirtyMainCreated.id, '--json']);
    const ok = r.status !== 0 && /WORKTREE_MERGE_MAIN_DIRTY/.test(r.stdout + r.stderr);
    record('worktree merge refuses a dirty MAIN tree (WORKTREE_MERGE_MAIN_DIRTY)', ok, `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
    const after = git(mainB, ['log', '-1', '--pretty=%H']);
    record('dirty-main refusal is zero-mutation (main HEAD unchanged)', before === after, `${before} vs ${after}`);
  }
  git(mainB, ['checkout', '--', 'f']);

  // ── dirty WORKTREE (a real tracked-file edit, not the bootstrapped store)
  // refuses, zero-mutation. ──────────────────────────────────────────────────
  const dirtyWtCreated = mergeNewWorktree(mainB, 'wsr-merge-dirty-wt');
  fs.writeFileSync(path.join(dirtyWtCreated.worktreeRoot, 'f'), 'uncommitted worktree edit\n');
  {
    const before = git(mainB, ['log', '-1', '--pretty=%H']);
    const r = bee(mainB, ['worktree', 'merge', '--id', dirtyWtCreated.id, '--json']);
    const ok = r.status !== 0 && /WORKTREE_MERGE_WORKTREE_DIRTY/.test(r.stdout + r.stderr);
    record('worktree merge refuses a dirty WORKTREE tree (WORKTREE_MERGE_WORKTREE_DIRTY)', ok, `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
    const after = git(mainB, ['log', '-1', '--pretty=%H']);
    record('dirty-worktree refusal is zero-mutation (main HEAD unchanged)', before === after, `${before} vs ${after}`);
  }

  // ── detached HEAD in the worktree refuses, zero-mutation. ─────────────────
  const detachedCreated = mergeNewWorktree(mainB, 'wsr-merge-detached');
  git(detachedCreated.worktreeRoot, ['checkout', '--detach', '-q']);
  {
    const before = git(mainB, ['log', '-1', '--pretty=%H']);
    const r = bee(mainB, ['worktree', 'merge', '--id', detachedCreated.id, '--json']);
    const ok = r.status !== 0 && /WORKTREE_MERGE_DETACHED_HEAD/.test(r.stdout + r.stderr);
    record('worktree merge refuses a detached-HEAD worktree (WORKTREE_MERGE_DETACHED_HEAD)', ok, `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
    const after = git(mainB, ['log', '-1', '--pretty=%H']);
    record('detached-HEAD refusal is zero-mutation (main HEAD unchanged)', before === after, `${before} vs ${after}`);
  }

  // ── worktree checked out to a branch other than its expected wt/<slug>
  // refuses, zero-mutation. ──────────────────────────────────────────────────
  const wrongBranchCreated = mergeNewWorktree(mainB, 'wsr-merge-wrong-branch');
  git(wrongBranchCreated.worktreeRoot, ['checkout', '-q', '-b', 'not-the-expected-branch']);
  {
    const before = git(mainB, ['log', '-1', '--pretty=%H']);
    const r = bee(mainB, ['worktree', 'merge', '--id', wrongBranchCreated.id, '--json']);
    const ok = r.status !== 0 && /WORKTREE_MERGE_BRANCH_MISMATCH/.test(r.stdout + r.stderr);
    record('worktree merge refuses a worktree checked out to an unexpected branch (WORKTREE_MERGE_BRANCH_MISMATCH)', ok, `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
    const after = git(mainB, ['log', '-1', '--pretty=%H']);
    record('wrong-branch refusal is zero-mutation (main HEAD unchanged)', before === after, `${before} vs ${after}`);
  }

  // ── own-worktree merge collapses into the not-ordinary refusal (advisor
  // R5 / decision D8: there is no separate "own worktree" code — running
  // merge from inside ANY linked worktree, including the one named by --id,
  // already fails isOrdinaryCheckout(mainRoot)). CLI path first (already
  // refuses because process.cwd() is not ordinary), then the direct-lib path
  // (mainRoot = the worktree's own root, id = its own id) to prove the SAME
  // guard is what protects a worktree from merging itself, not a phantom
  // second code. ────────────────────────────────────────────────────────────
  const ownCreated = mergeNewWorktree(mainB, 'wsr-merge-own');
  {
    const r = bee(ownCreated.worktreeRoot, ['worktree', 'merge', '--id', ownCreated.id, '--json']);
    const ok = r.status !== 0 && !/at Object|TypeError|ReferenceError/.test(r.stderr);
    record('worktree merge run from inside a linked worktree refuses (typed, non-crash) — CLI path', ok, `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
  }
  {
    let caught = null;
    try {
      // xwh-2: mergeFeatureWorktree is now async (its --cleanup path awaits
      // releaseAllForHolder) — every early, zero-mutation refusal (this one
      // included) now rejects instead of throwing synchronously, so this
      // needs an await in front of it to still land in the catch below.
      await mergeFeatureWorktree(ownCreated.worktreeRoot, { id: ownCreated.id });
    } catch (error) {
      caught = error;
    }
    const ok = caught && caught.code === 'WORKTREE_MERGE_CALLER_NOT_ORDINARY';
    record(
      'direct-lib own-id path: mergeFeatureWorktree(worktreeRoot, {id: its own id}) refuses WORKTREE_MERGE_CALLER_NOT_ORDINARY (own-worktree merge collapses into not-ordinary)',
      ok,
      caught ? `${caught.code}: ${caught.message}` : 'no error thrown',
    );
  }

  // ── cleanup: green (verify:'skipped', no commands.verify in fixture B) +
  // --cleanup removes the worktree, deletes the branch, and drops the grant.
  // ───────────────────────────────────────────────────────────────────────
  const cleanupCreated = mergeNewWorktree(mainB, 'wsr-merge-cleanup');
  fs.writeFileSync(path.join(cleanupCreated.worktreeRoot, 'cleanup-work.txt'), 'work to merge\n');
  git(cleanupCreated.worktreeRoot, ['add', 'cleanup-work.txt']);
  git(cleanupCreated.worktreeRoot, ['commit', '-q', '-m', 'cleanup fixture work']);
  const cleanupResult = bee(mainB, ['worktree', 'merge', '--id', cleanupCreated.id, '--cleanup', '--json']);
  {
    const ok = cleanupResult.status === 0;
    record('worktree merge --cleanup (green/skipped path) exits 0', ok, `status=${cleanupResult.status} stdout=${cleanupResult.stdout} stderr=${cleanupResult.stderr}`);
  }
  let cleanupJson = null;
  try {
    cleanupJson = JSON.parse(cleanupResult.stdout);
  } catch {
    /* checked below */
  }
  {
    const ok = cleanupJson && cleanupJson.ok === true && cleanupJson.verify === 'skipped' && cleanupJson.cleanup && cleanupJson.cleanup.ok === true && cleanupJson.cleanup.removed === true && cleanupJson.cleanup.branch_deleted === true;
    record('worktree merge --cleanup reports verify:"skipped" and cleanup:{ok:true, removed:true, branch_deleted:true}', ok, cleanupResult.stdout);
  }
  {
    // Extending --cleanup eligibility to verify:'skipped' is a D8-gap
    // resolution (advisor-confirmed), conditioned on never being silent: a
    // cleanup that ran with no semantic gate must say so loudly.
    const ok = cleanupJson && cleanupJson.cleanup && /verify skipped.*no semantic gate|verify skipped — no commands\.verify recorded; cleaned up unchecked\./.test(cleanupJson.cleanup.warning || '');
    record('worktree merge --cleanup on a skipped verify carries a loud "cleaned up unchecked" warning', ok, JSON.stringify(cleanupJson.cleanup));
  }
  {
    const ok = !fs.existsSync(cleanupCreated.worktreeRoot);
    record('worktree merge --cleanup actually removed the worktree directory', ok, cleanupCreated.worktreeRoot);
  }
  {
    const branchList = git(mainB, ['branch', '--list', `wt/wsr-merge-cleanup`]).trim();
    record('worktree merge --cleanup deleted the branch (git branch -d, not -D)', branchList.length === 0, `git branch --list -> "${branchList}"`);
  }
  {
    const grants = JSON.parse(fs.readFileSync(grantsFileB, 'utf8'));
    const ok = !(cleanupCreated.id in grants);
    record('worktree merge --cleanup dropped the id from the MAIN store\'s grant registry', ok, JSON.stringify(grants));
  }

  // ── cleanup refusal: untracked file at a tracked path blocks cleanup, but
  // the merge result itself is still reported ok (D8b). The worktree MUST be
  // clean to pass merge's own pre-flight dirty check in the first place, so
  // the only honest way to exercise cleanup's SEPARATE freshness re-check is
  // a file dropped as a SIDE EFFECT of the (green) verify step itself —
  // exactly the race the re-check exists to catch (verify can run for a
  // while; the worktree is not re-frozen while it runs). A dedicated fixture
  // (mainC) records a verify command that always exits 0 but plants a file
  // at the path named by $BEE_TEST_LEFTOVER_PATH, if set. ──────────────────
  const mainC = path.join(mergeTmp, 'mainC');
  fs.mkdirSync(mainC, { recursive: true });
  git(mainC, ['init', '-q', '-b', 'main']);
  git(mainC, ['config', 'user.email', 's@e']);
  git(mainC, ['config', 'user.name', 's']);
  fs.writeFileSync(path.join(mainC, '.gitignore'), BEE_GITIGNORE);
  fs.mkdirSync(path.join(mainC, '.bee'), { recursive: true });
  fs.writeFileSync(path.join(mainC, '.bee', 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.0.0' }));
  fs.writeFileSync(path.join(mainC, '.bee', 'config.json'), JSON.stringify({ commands: { verify: 'node verify-leftover.mjs' } }));
  fs.writeFileSync(path.join(mainC, 'f'), 'x');
  fs.writeFileSync(
    path.join(mainC, 'verify-leftover.mjs'),
    "import fs from 'node:fs';\nconst target = process.env.BEE_TEST_LEFTOVER_PATH;\nif (target) fs.writeFileSync(target, 'oops, forgot to commit this\\n');\nprocess.exit(0);\n",
  );
  git(mainC, ['add', '.']);
  git(mainC, ['commit', '-q', '-m', 'init']);

  const cleanupDirtyCreated = mergeNewWorktree(mainC, 'wsr-merge-cleanup-dirty');
  fs.writeFileSync(path.join(cleanupDirtyCreated.worktreeRoot, 'merge-work.txt'), 'more work\n');
  git(cleanupDirtyCreated.worktreeRoot, ['add', 'merge-work.txt']);
  git(cleanupDirtyCreated.worktreeRoot, ['commit', '-q', '-m', 'more cleanup fixture work']);
  const leftoverPath = path.join(cleanupDirtyCreated.worktreeRoot, 'leftover.txt');
  process.env.BEE_TEST_LEFTOVER_PATH = leftoverPath;
  let cleanupDirtyResult;
  try {
    cleanupDirtyResult = bee(mainC, ['worktree', 'merge', '--id', cleanupDirtyCreated.id, '--cleanup', '--json']);
  } finally {
    delete process.env.BEE_TEST_LEFTOVER_PATH;
  }
  {
    const ok = cleanupDirtyResult.status === 0;
    record('worktree merge --cleanup: the underlying merge itself still exits 0 even when cleanup is refused', ok, `status=${cleanupDirtyResult.status} stdout=${cleanupDirtyResult.stdout}`);
  }
  let cleanupDirtyJson = null;
  try {
    cleanupDirtyJson = JSON.parse(cleanupDirtyResult.stdout);
  } catch {
    /* checked below */
  }
  {
    const ok = cleanupDirtyJson && cleanupDirtyJson.ok === true && cleanupDirtyJson.verify === 'green' && cleanupDirtyJson.cleanup && cleanupDirtyJson.cleanup.ok === false && cleanupDirtyJson.cleanup.code === 'WORKTREE_MERGE_CLEANUP_DIRTY';
    record('worktree merge --cleanup refuses (typed WORKTREE_MERGE_CLEANUP_DIRTY) when untracked files remain at tracked paths, merge result still ok:true', ok, cleanupDirtyResult.stdout);
  }
  {
    const ok = fs.existsSync(cleanupDirtyCreated.worktreeRoot);
    record('cleanup refusal left the worktree directory in place (no partial removal)', ok, cleanupDirtyCreated.worktreeRoot);
  }
  {
    const ok = fs.existsSync(leftoverPath);
    record('the untracked leftover file (planted by verify, simulating a race) is still there — cleanup did not silently delete it', ok, leftoverPath);
  }

  // ── post-commit guard: a verify command that mutates a TRACKED file
  // (without bee's knowledge) still lets the green merge commit land — but
  // the result carries a typed "verify_mutated_tracked_files" warning
  // instead of silently treating the working tree as equivalent to the
  // commit (D2-REVISED step 6). A dedicated fixture (mainD) records a verify
  // command that unconditionally overwrites tracked file "f" and exits 0.
  const mainD = path.join(mergeTmp, 'mainD');
  fs.mkdirSync(mainD, { recursive: true });
  git(mainD, ['init', '-q', '-b', 'main']);
  git(mainD, ['config', 'user.email', 's@e']);
  git(mainD, ['config', 'user.name', 's']);
  fs.writeFileSync(path.join(mainD, '.gitignore'), BEE_GITIGNORE);
  fs.mkdirSync(path.join(mainD, '.bee'), { recursive: true });
  fs.writeFileSync(path.join(mainD, '.bee', 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.0.0' }));
  fs.writeFileSync(path.join(mainD, '.bee', 'config.json'), JSON.stringify({ commands: { verify: 'node verify-mutate.mjs' } }));
  fs.writeFileSync(path.join(mainD, 'f'), 'x');
  fs.writeFileSync(
    path.join(mainD, 'verify-mutate.mjs'),
    "import fs from 'node:fs';\nfs.writeFileSync('f', 'mutated-by-verify\\n');\nprocess.exit(0);\n",
  );
  git(mainD, ['add', '.']);
  git(mainD, ['commit', '-q', '-m', 'init']);

  const mutateCreated = mergeNewWorktree(mainD, 'wsr-merge-mutate');
  fs.writeFileSync(path.join(mutateCreated.worktreeRoot, 'other.txt'), 'unrelated file\n');
  git(mutateCreated.worktreeRoot, ['add', 'other.txt']);
  git(mutateCreated.worktreeRoot, ['commit', '-q', '-m', 'mutate fixture work']);
  const mutateResult = bee(mainD, ['worktree', 'merge', '--id', mutateCreated.id, '--json']);
  {
    const ok = mutateResult.status === 0;
    record('worktree merge (verify mutates a tracked file, but still exits green) still exits 0', ok, `status=${mutateResult.status} stdout=${mutateResult.stdout} stderr=${mutateResult.stderr}`);
  }
  let mutateJson = null;
  try {
    mutateJson = JSON.parse(mutateResult.stdout);
  } catch {
    /* checked below */
  }
  {
    const ok = mutateJson && mutateJson.ok === true && mutateJson.merged === true && mutateJson.verify === 'green' && mutateJson.warning && mutateJson.warning.code === 'verify_mutated_tracked_files';
    record('verify mutating a tracked file on a green merge attaches a typed warning.code:"verify_mutated_tracked_files" instead of silently claiming tree === commit', ok, mutateResult.stdout);
  }
  {
    const ok = /Merge worktree/.test(git(mainD, ['log', '-1', '--pretty=%B']));
    record('the merge commit itself still landed despite the post-commit tracked-file-mutation warning', ok, git(mainD, ['log', '--oneline', '-1']));
  }
  {
    const content = fs.readFileSync(path.join(mainD, 'f'), 'utf8');
    const status = git(mainD, ['status', '--porcelain', '--untracked-files=no']);
    const ok = content === 'mutated-by-verify\n' && /^ M f/m.test(status);
    record('the tracked-file mutation from verify is really sitting dirty on top of the merge commit (not silently absorbed)', ok, `content=${JSON.stringify(content)} status=${JSON.stringify(status)}`);
  }
  git(mainD, ['checkout', '--', 'f']); // test cleanup only (restores mainD's tree), not bee behavior

  // ── no cleanup on MERGE_CONFLICT, MERGE_VERIFY_RED, or ALREADY_UP_TO_DATE,
  // even with --cleanup passed (D2-REVISED step 7: strictly post-commit). ───
  {
    const noCleanupConflictCreated = mergeNewWorktree(mainB, 'wsr-merge-no-cleanup-conflict');
    fs.writeFileSync(path.join(noCleanupConflictCreated.worktreeRoot, 'f'), 'worktree edit again\n');
    git(noCleanupConflictCreated.worktreeRoot, ['commit', '-am', 'worktree edits f again']);
    fs.writeFileSync(path.join(mainB, 'f'), 'main edit again\n');
    git(mainB, ['commit', '-am', 'main edits f again too']);
    const attemptPreHead = git(mainB, ['log', '-1', '--pretty=%H']).trim(); // after "main edits f again too" landed
    const r = bee(mainB, ['worktree', 'merge', '--id', noCleanupConflictCreated.id, '--cleanup', '--json']);
    const parsed = JSON.parse(r.stdout);
    const ok = parsed.ok === false && parsed.code === 'MERGE_CONFLICT' && !('cleanup' in parsed);
    record('MERGE_CONFLICT + --cleanup never attempts cleanup (no .cleanup field on the result)', ok, r.stdout);
    const stillThere = fs.existsSync(noCleanupConflictCreated.worktreeRoot);
    record('MERGE_CONFLICT + --cleanup left the worktree in place', stillThere, noCleanupConflictCreated.worktreeRoot);
    // bee already ran "git merge --abort" internally (proven above by the
    // dedicated MERGE_CONFLICT case) — nothing is left for the test to
    // clean up by hand here.
    const proof = threePartProof(mainB, attemptPreHead);
    record('MERGE_CONFLICT + --cleanup: main still passes the three-part untouched proof after the aborted attempt', proof.ok, JSON.stringify(proof));
  }
  {
    // Self-contained (advisor R6): this worktree flips flag.txt to "on"
    // itself rather than relying on a previous test's red-verify leftover —
    // under the new abort-on-red contract, mainA's flag.txt reverts to
    // "off" the moment ANY red-verify merge attempt is aborted, so no such
    // leftover exists to depend on anymore.
    const noCleanupRedCreated = mergeNewWorktree(mainA, 'wsr-merge-no-cleanup-red');
    fs.writeFileSync(path.join(noCleanupRedCreated.worktreeRoot, 'flag.txt'), 'on');
    git(noCleanupRedCreated.worktreeRoot, ['commit', '-am', 'flip the flag again']);
    const r = bee(mainA, ['worktree', 'merge', '--id', noCleanupRedCreated.id, '--cleanup', '--json']);
    const parsed = JSON.parse(r.stdout);
    const ok = parsed.ok === false && parsed.code === 'MERGE_VERIFY_RED' && !('cleanup' in parsed);
    record('MERGE_VERIFY_RED + --cleanup never attempts cleanup (no .cleanup field on the result)', ok, r.stdout);
    const stillThere = fs.existsSync(noCleanupRedCreated.worktreeRoot);
    record('MERGE_VERIFY_RED + --cleanup left the worktree in place', stillThere, noCleanupRedCreated.worktreeRoot);
    const flagContent = fs.readFileSync(path.join(mainA, 'flag.txt'), 'utf8').trim();
    record('MERGE_VERIFY_RED + --cleanup: the abort still reverted flag.txt to "off" on main (cleanup skip did not skip the abort)', flagContent === 'off', flagContent);
  }
  // ── ALREADY_UP_TO_DATE: a worktree with no new commits vs main is a typed
  // no-op — a commit is never attempted. --cleanup, however, DOES run here.
  //
  // issues-46-53 D3 + D8 (#47) — WHY THE OLD ASSERTION HERE WAS WRONG. This
  // block used to assert `!('cleanup' in parsed)` and "already-up-to-date
  // merge left the worktree in place (--cleanup never runs on a no-op)",
  // pinning the blanket rule "cleanup is strictly post-commit". That
  // expectation encoded the defect: it treated the COMMIT as the safety
  // property, when the real property is "the worktree holds nothing that would
  // be lost". ALREADY_UP_TO_DATE means the branch holds nothing main lacks,
  // and the WORKTREE_MERGE_WORKTREE_DIRTY pre-check upstream has already
  // proved the worktree carries no uncommitted work — so the flag was merely
  // being accepted and then silently dropped (exit 0, nothing removed, not one
  // word about it). The rule stays exactly right for MERGE_CONFLICT and
  // MERGE_VERIFY_RED, both asserted directly above and deliberately left
  // untouched: work is NOT integrated on either of those paths, so removing
  // the worktree would destroy the only copy of it. ────────────────────────
  {
    const noopCreated = mergeNewWorktree(mainB, 'wsr-merge-noop');
    const before = git(mainB, ['log', '-1', '--pretty=%H']).trim();
    const r = bee(mainB, ['worktree', 'merge', '--id', noopCreated.id, '--cleanup', '--json']);
    {
      const ok = r.status === 0;
      record('worktree merge on an already-up-to-date branch exits 0', ok, `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
    }
    let parsed = null;
    try {
      parsed = JSON.parse(r.stdout);
    } catch {
      /* checked below */
    }
    {
      const ok = parsed && parsed.ok === true && parsed.merged === false && parsed.code === 'ALREADY_UP_TO_DATE' && parsed.verify === 'skipped';
      record('worktree merge --json reports ok:true, merged:false, code:"ALREADY_UP_TO_DATE", verify:"skipped"', ok, r.stdout);
    }
    {
      const ok = parsed && parsed.cleanup && parsed.cleanup.ok === true && parsed.cleanup.removed === true && parsed.cleanup.branch_deleted === true;
      record('(#47) ALREADY_UP_TO_DATE + --cleanup RUNS cleanup — the flag is no longer silently dropped', ok, r.stdout);
    }
    {
      // The "cleaned up unchecked" warning means "no commands.verify was
      // recorded, so this ran with no semantic gate" — a lie on this path,
      // where verify was skipped only because nothing was merged for it to
      // check. code:"ALREADY_UP_TO_DATE" + merged:false already say that.
      const ok = parsed && parsed.cleanup && !('warning' in parsed.cleanup);
      record('(#47) the no-op cleanup carries no "cleaned up unchecked" warning — nothing was merged, so no verify gate was owed', ok, JSON.stringify(parsed && parsed.cleanup));
    }
    const after = git(mainB, ['log', '-1', '--pretty=%H']).trim();
    record('already-up-to-date merge never committed anything (main HEAD unchanged)', before === after, `${before} vs ${after}`);
    {
      const ok = !fs.existsSync(noopCreated.worktreeRoot);
      record('(#47) ALREADY_UP_TO_DATE + --cleanup actually removed the worktree directory', ok, noopCreated.worktreeRoot);
    }
    {
      const branchList = git(mainB, ['branch', '--list', 'wt/wsr-merge-noop']).trim();
      record('(#47) ALREADY_UP_TO_DATE + --cleanup deleted the branch (git branch -d, not -D)', branchList.length === 0, `git branch --list -> "${branchList}"`);
    }
    {
      const grants = JSON.parse(fs.readFileSync(grantsFileB, 'utf8'));
      record("(#47) ALREADY_UP_TO_DATE + --cleanup dropped the id from the MAIN store's grant registry", !(noopCreated.id in grants), JSON.stringify(grants));
    }
  }
  {
    // The FLAG is what removes the worktree, not the no-op path itself:
    // without --cleanup the same no-op still leaves everything in place and
    // only suggests the command.
    const noopKeepCreated = mergeNewWorktree(mainB, 'wsr-merge-noop-keep');
    const r = bee(mainB, ['worktree', 'merge', '--id', noopKeepCreated.id, '--json']);
    const parsed = JSON.parse(r.stdout);
    const ok = parsed.code === 'ALREADY_UP_TO_DATE' && !('cleanup' in parsed) && typeof parsed.cleanup_suggested_command === 'string';
    record('(#47) an already-up-to-date merge WITHOUT --cleanup removes nothing and only suggests the cleanup command', ok, r.stdout);
    record('(#47) without --cleanup, the already-up-to-date worktree is left in place', fs.existsSync(noopKeepCreated.worktreeRoot), noopKeepCreated.worktreeRoot);
  }
  {
    // The CLI's TEXT output half of #47: the no-op branch used to print one
    // headline and stop, so a dropped --cleanup was invisible there too.
    const noopTextCreated = mergeNewWorktree(mainB, 'wsr-merge-noop-text');
    const r = bee(mainB, ['worktree', 'merge', '--id', noopTextCreated.id, '--cleanup']);
    const ok = r.status === 0 && /nothing to merge; no commit was made\./.test(r.stdout) && /cleanup: worktree removed, branch deleted\./.test(r.stdout);
    record('(#47) the no-op TEXT output says out loud what --cleanup did, instead of staying silent about the flag', ok, `status=${r.status} stdout=${JSON.stringify(r.stdout)}`);
  }

  // ── without --cleanup, a green/skipped result only SUGGESTS the cleanup
  // command — nothing is removed. ────────────────────────────────────────────
  {
    const suggestCreated = mergeNewWorktree(mainB, 'wsr-merge-suggest');
    fs.writeFileSync(path.join(suggestCreated.worktreeRoot, 'suggest-work.txt'), 'x\n');
    git(suggestCreated.worktreeRoot, ['add', 'suggest-work.txt']);
    git(suggestCreated.worktreeRoot, ['commit', '-q', '-m', 'suggest fixture work']);
    const r = bee(mainB, ['worktree', 'merge', '--id', suggestCreated.id, '--json']);
    const parsed = JSON.parse(r.stdout);
    const ok = parsed.ok === true && !('cleanup' in parsed) && typeof parsed.cleanup_suggested_command === 'string' && parsed.cleanup_suggested_command.includes(suggestCreated.id);
    record('worktree merge without --cleanup only suggests the cleanup command (no .cleanup field, cleanup_suggested_command present)', ok, r.stdout);
    const stillThere = fs.existsSync(suggestCreated.worktreeRoot);
    record('without --cleanup, the worktree was left in place', stillThere, suggestCreated.worktreeRoot);
  }

  // ── test-simple (decision 412e9b3a, supersedes cov-4/D5): the merge gate
  // is `commands.verify` — the last net over the merged tree — with a
  // STRING commands.test as the fallback only when no verify is recorded.
  // (commands.test may be an ARRAY for the dev-loop runner; an array is
  // never spawned as one shell command here.) Both fixtures below write a
  // distinct marker file from whichever script actually ran, so "which
  // command executed" is proven by a file on disk rather than inferred from
  // the green/red outcome alone. ────────────────────────────────────────
  {
    // commands.test AND commands.verify both configured, and they disagree:
    // commands.test exits 0 (would report green), commands.verify exits 1
    // (reports red/MERGE_VERIFY_RED). A RED merge result here is only
    // possible if commands.verify — the merge-time chain — is what ran.
    const mainTestPref = path.join(mergeTmp, 'mainTestPref');
    fs.mkdirSync(mainTestPref, { recursive: true });
    git(mainTestPref, ['init', '-q', '-b', 'main']);
    git(mainTestPref, ['config', 'user.email', 's@e']);
    git(mainTestPref, ['config', 'user.name', 's']);
    fs.writeFileSync(path.join(mainTestPref, '.gitignore'), BEE_GITIGNORE);
    fs.mkdirSync(path.join(mainTestPref, '.bee'), { recursive: true });
    fs.writeFileSync(path.join(mainTestPref, '.bee', 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.0.0' }));
    fs.writeFileSync(
      path.join(mainTestPref, '.bee', 'config.json'),
      JSON.stringify({ commands: { test: 'node test-cmd.mjs', verify: 'node verify-cmd.mjs' } }),
    );
    fs.writeFileSync(path.join(mainTestPref, 'f'), 'x');
    fs.writeFileSync(
      path.join(mainTestPref, 'test-cmd.mjs'),
      "import fs from 'node:fs';\nfs.writeFileSync('test-cmd-ran.txt', 'x\\n');\nprocess.exit(0);\n",
    );
    fs.writeFileSync(
      path.join(mainTestPref, 'verify-cmd.mjs'),
      "import fs from 'node:fs';\nfs.writeFileSync('verify-cmd-ran.txt', 'x\\n');\nprocess.exit(1);\n",
    );
    git(mainTestPref, ['add', '.']);
    git(mainTestPref, ['commit', '-q', '-m', 'init']);

    const prefCreated = mergeNewWorktree(mainTestPref, 'wsr-merge-test-pref');
    fs.writeFileSync(path.join(prefCreated.worktreeRoot, 'pref-work.txt'), 'x\n');
    git(prefCreated.worktreeRoot, ['add', 'pref-work.txt']);
    git(prefCreated.worktreeRoot, ['commit', '-q', '-m', 'test-pref fixture work']);
    const prefResult = bee(mainTestPref, ['worktree', 'merge', '--id', prefCreated.id, '--json']);
    let prefJson = null;
    try {
      prefJson = JSON.parse(prefResult.stdout);
    } catch {
      /* checked below */
    }
    {
      const ok = prefResult.status !== 0 && prefJson && prefJson.ok === false && prefJson.code === 'MERGE_VERIFY_RED';
      record('(test-simple) worktree merge with BOTH commands.test and commands.verify configured — a RED result is only possible if commands.verify (the merge-time chain) ran, not commands.test', ok, prefResult.stdout);
    }
    {
      const testRan = fs.existsSync(path.join(mainTestPref, 'test-cmd-ran.txt'));
      const verifyRan = fs.existsSync(path.join(mainTestPref, 'verify-cmd-ran.txt'));
      record('(test-simple) commands.verify is preferred over commands.test at merge: the verify-cmd marker exists and the test-cmd marker does not', verifyRan && !testRan, `test-cmd-ran=${testRan} verify-cmd-ran=${verifyRan}`);
    }
  }
  {
    // commands.test ABSENT, only commands.verify configured — merge must
    // still fall back to commands.verify (the pre-existing contract, now
    // proven directly by the same marker-file technique rather than only
    // inferred from mainA's earlier green/red assertions).
    const mainFallback = path.join(mergeTmp, 'mainTestFallback');
    fs.mkdirSync(mainFallback, { recursive: true });
    git(mainFallback, ['init', '-q', '-b', 'main']);
    git(mainFallback, ['config', 'user.email', 's@e']);
    git(mainFallback, ['config', 'user.name', 's']);
    fs.writeFileSync(path.join(mainFallback, '.gitignore'), BEE_GITIGNORE);
    fs.mkdirSync(path.join(mainFallback, '.bee'), { recursive: true });
    fs.writeFileSync(path.join(mainFallback, '.bee', 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.0.0' }));
    fs.writeFileSync(path.join(mainFallback, '.bee', 'config.json'), JSON.stringify({ commands: { verify: 'node verify-cmd.mjs' } }));
    fs.writeFileSync(path.join(mainFallback, 'f'), 'x');
    fs.writeFileSync(
      path.join(mainFallback, 'verify-cmd.mjs'),
      "import fs from 'node:fs';\nfs.writeFileSync('verify-cmd-ran.txt', 'x\\n');\nprocess.exit(0);\n",
    );
    git(mainFallback, ['add', '.']);
    git(mainFallback, ['commit', '-q', '-m', 'init']);

    const fallbackCreated = mergeNewWorktree(mainFallback, 'wsr-merge-test-fallback');
    fs.writeFileSync(path.join(fallbackCreated.worktreeRoot, 'fallback-work.txt'), 'x\n');
    git(fallbackCreated.worktreeRoot, ['add', 'fallback-work.txt']);
    git(fallbackCreated.worktreeRoot, ['commit', '-q', '-m', 'test-fallback fixture work']);
    const fallbackResult = bee(mainFallback, ['worktree', 'merge', '--id', fallbackCreated.id, '--json']);
    let fallbackJson = null;
    try {
      fallbackJson = JSON.parse(fallbackResult.stdout);
    } catch {
      /* checked below */
    }
    {
      const ok = fallbackResult.status === 0 && fallbackJson && fallbackJson.ok === true && fallbackJson.verify === 'green';
      record('(cov-4) worktree merge with only commands.verify configured (no commands.test) still exits green', ok, fallbackResult.stdout);
    }
    {
      const verifyRan = fs.existsSync(path.join(mainFallback, 'verify-cmd-ran.txt'));
      record('(cov-4) with commands.test absent, merge falls back to running commands.verify (the verify-cmd marker exists)', verifyRan, `verify-cmd-ran=${verifyRan}`);
    }
  }
  {
    // P1-4 regression: a REAL staged merge (git merge --no-ff --no-commit,
    // exactly the state mergeFeatureWorktree gates in) with a
    // registry-mapped changed file must show the gate actually enumerating
    // the STAGED changes and selecting >0 suites — never the trivial
    // "0 suites" pass a wiring bug (wrong cwd, reading committed HEAD
    // instead of the staged index, an empty/stale registry) would produce
    // silently. Wiring the real scripts/run_verify.mjs + the real
    // scripts/impact-registry.json into this temp-repo fixture is
    // disproportionate (it would need the whole repo's suite tree present
    // under the fixture root) — this uses a small stub commands.test that
    // does the same two things the real gate does: (1) reads `git status
    // --porcelain` for the currently staged/working changes, exactly like
    // run_verify.mjs's statusPorcelainFiles(), and (2) maps changed paths
    // through a tiny on-disk registry file, exactly like
    // impact_registry.mjs's queryRegistry(). The property under test is the
    // WIRING (does the gate see the staged merge state and pick >0 suites
    // for a mapped file), not run_verify.mjs's own suite-selection logic,
    // which is exercised directly by scripts/tests/test_impact_registry.mjs and
    // scripts/tests/test_run_verify_impacted.mjs elsewhere in the chain.
    const mainP14 = path.join(mergeTmp, 'mainP14');
    fs.mkdirSync(mainP14, { recursive: true });
    git(mainP14, ['init', '-q', '-b', 'main']);
    git(mainP14, ['config', 'user.email', 's@e']);
    git(mainP14, ['config', 'user.name', 's']);
    fs.writeFileSync(path.join(mainP14, '.gitignore'), BEE_GITIGNORE);
    fs.mkdirSync(path.join(mainP14, '.bee'), { recursive: true });
    fs.writeFileSync(path.join(mainP14, '.bee', 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.0.0' }));
    fs.writeFileSync(path.join(mainP14, '.bee', 'config.json'), JSON.stringify({ commands: { test: 'node impacted-stub.mjs' } }));
    fs.writeFileSync(path.join(mainP14, 'f'), 'x');
    // tiny stand-in for scripts/impact-registry.json: maps one known changed
    // path to one known suite label.
    fs.writeFileSync(path.join(mainP14, 'stub-registry.json'), JSON.stringify({ 'mapped.txt': 'fake-suite-A' }));
    fs.writeFileSync(
      path.join(mainP14, 'impacted-stub.mjs'),
      [
        "import fs from 'node:fs';",
        "import { execSync } from 'node:child_process';",
        "const out = execSync('git status --porcelain', { encoding: 'utf8' });",
        "const changedFiles = out.split('\\n').filter(Boolean).map((l) => l.slice(3).trim());",
        "const registry = JSON.parse(fs.readFileSync('stub-registry.json', 'utf8'));",
        "const suites = [...new Set(changedFiles.map((f) => registry[f]).filter(Boolean))];",
        "fs.writeFileSync('impacted-result.json', JSON.stringify({ changedFiles, suites }));",
        "console.log(`IMPACTED RUN: ${suites.length} suite(s) from ${changedFiles.length} changed file(s)`);",
        'process.exit(0);',
        '',
      ].join('\n'),
    );
    git(mainP14, ['add', '.']);
    git(mainP14, ['commit', '-q', '-m', 'init']);

    const p14Created = mergeNewWorktree(mainP14, 'wsr-merge-p14');
    fs.writeFileSync(path.join(p14Created.worktreeRoot, 'mapped.txt'), 'registry-mapped change\n');
    git(p14Created.worktreeRoot, ['add', 'mapped.txt']);
    git(p14Created.worktreeRoot, ['commit', '-q', '-m', 'add a registry-mapped file']);
    const p14Result = bee(mainP14, ['worktree', 'merge', '--id', p14Created.id, '--json']);
    {
      const ok = p14Result.status === 0 && JSON.parse(p14Result.stdout).ok === true;
      record('(cov-4 P1-4) worktree merge with a registry-mapped changed file exits 0', ok, `status=${p14Result.status} stdout=${p14Result.stdout} stderr=${p14Result.stderr}`);
    }
    const p14ResultFile = path.join(mainP14, 'impacted-result.json');
    let p14Stub = null;
    try {
      p14Stub = JSON.parse(fs.readFileSync(p14ResultFile, 'utf8'));
    } catch {
      /* checked below */
    }
    {
      const ok = p14Stub !== null && p14Stub.changedFiles.includes('mapped.txt');
      record('(cov-4 P1-4) the gate ran against the STAGED merge state — it enumerated the staged "mapped.txt" change, not an empty/committed tree', ok, JSON.stringify(p14Stub));
    }
    {
      const ok = p14Stub !== null && p14Stub.suites.length > 0 && p14Stub.suites.includes('fake-suite-A');
      record('(cov-4 P1-4) a registry-mapped changed file selects >0 suites — never the trivial "0 suites" pass', ok, JSON.stringify(p14Stub));
    }
  }

  // ── #46 (issues-46-53 D4): the worktree's identity is fixed at creation and
  // is no longer read from the MUTABLE state.feature. The paved road creates
  // the worktree at session-scout time, BEFORE exploring settles the feature's
  // real name — so the rename below is the designed-in case, not user error.
  // The directory, the branch and the feature all came from ONE slug; only
  // state.feature drifts, and merge used to derive its expected branch from
  // that drifted field and then blame the BRANCH — the one thing the user must
  // not change. `identityRel` is the immutable record; it lives under
  // .bee/runtime/ (gitignored everywhere) so it can never make the worktree
  // read dirty to merge's own `git status --porcelain` pre-check. ───────────
  const identityRel = path.join('.bee', 'runtime', 'worktree-identity.json');
  /** The real drift path: `bee state set --feature <new>` inside the worktree
   * (--owner is required by state set; a freshly bootstrapped worktree is on
   * phase "idle"). */
  function renameWorktreeFeature(worktreeRoot, newFeature) {
    const r = bee(worktreeRoot, ['state', 'set', '--feature', newFeature, '--owner', 'idle']);
    if (r.status !== 0) throw new Error(`state set --feature ${newFeature} failed: ${r.stdout} ${r.stderr}`);
  }
  {
    const renamed = mergeNewWorktree(mainB, 'wsr-scout-guess');
    {
      const identityFile = path.join(renamed.worktreeRoot, identityRel);
      const parsed = fs.existsSync(identityFile) ? JSON.parse(fs.readFileSync(identityFile, 'utf8')) : null;
      record('(#46) worktree new records the creation slug immutably in .bee/runtime/worktree-identity.json', parsed && parsed.feature === 'wsr-scout-guess', JSON.stringify(parsed));
    }
    fs.writeFileSync(path.join(renamed.worktreeRoot, 'renamed-work.txt'), 'work done under the real name\n');
    git(renamed.worktreeRoot, ['add', 'renamed-work.txt']);
    git(renamed.worktreeRoot, ['commit', '-q', '-m', 'work under the settled name']);
    renameWorktreeFeature(renamed.worktreeRoot, 'wsr-settled-name');
    {
      const state = JSON.parse(fs.readFileSync(path.join(renamed.worktreeRoot, '.bee', 'state.json'), 'utf8'));
      const identity = JSON.parse(fs.readFileSync(path.join(renamed.worktreeRoot, identityRel), 'utf8'));
      const ok = state.feature === 'wsr-settled-name' && identity.feature === 'wsr-scout-guess';
      record('(#46) a later rename moves state.feature and leaves the immutable creation slug alone', ok, JSON.stringify({ state: state.feature, identity: identity.feature }));
    }
    {
      const dirty = git(renamed.worktreeRoot, ['status', '--porcelain']).trim();
      record('(#46) the immutable record is gitignored — it never makes the worktree read dirty to merge\'s pre-check', dirty === '', JSON.stringify(dirty));
    }
    const r = bee(mainB, ['worktree', 'merge', '--id', renamed.id, '--json']);
    {
      const ok = r.status === 0 && JSON.parse(r.stdout).ok === true && JSON.parse(r.stdout).merged === true;
      record('(#46) a worktree whose feature was renamed after creation now MERGES — the expected branch comes from the immutable slug, not the drifted field', ok, `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);
    }
    record('(#46) the renamed worktree\'s committed work actually landed on main', fs.existsSync(path.join(mainB, 'renamed-work.txt')), mainB);
  }
  {
    // Drift AND a genuinely different checked-out branch: merge still refuses
    // (it cannot guess), but the message now names the field that drifted and
    // says outright not to rename the branch to match.
    const driftCreated = mergeNewWorktree(mainB, 'wsr-drift-blame');
    renameWorktreeFeature(driftCreated.worktreeRoot, 'wsr-drift-renamed');
    git(driftCreated.worktreeRoot, ['checkout', '-q', '-b', 'something-else-entirely']);
    const r = bee(mainB, ['worktree', 'merge', '--id', driftCreated.id, '--json']);
    const out = r.stdout + r.stderr;
    const ok =
      r.status !== 0 &&
      /WORKTREE_MERGE_BRANCH_MISMATCH/.test(out) &&
      /FEATURE FIELD drifted after creation/.test(out) &&
      /wsr-drift-blame/.test(out) &&
      /wsr-drift-renamed/.test(out) &&
      /Do NOT rename the branch to match/.test(out);
    record('(#46) when the two disagree, the refusal names the DRIFTED FIELD (quoting both values) instead of blaming the unchangeable branch', ok, out);
  }
  {
    // A pre-existing worktree — one created before the immutable record
    // shipped — must degrade to EXACTLY today's behavior: same accept, same
    // refuse, never a crash and never a new refusal. Simulated by deleting the
    // record, which is what such a worktree genuinely looks like on disk.
    const legacyOk = mergeNewWorktree(mainB, 'wsr-legacy-ok');
    fs.rmSync(path.join(legacyOk.worktreeRoot, identityRel), { force: true });
    fs.writeFileSync(path.join(legacyOk.worktreeRoot, 'legacy-work.txt'), 'x\n');
    git(legacyOk.worktreeRoot, ['add', 'legacy-work.txt']);
    git(legacyOk.worktreeRoot, ['commit', '-q', '-m', 'legacy work']);
    const r = bee(mainB, ['worktree', 'merge', '--id', legacyOk.id, '--json']);
    const ok = r.status === 0 && JSON.parse(r.stdout).merged === true;
    record('(#46) a worktree with NO immutable record and no rename merges exactly as before (degrades, never crashes)', ok, `status=${r.status} stdout=${r.stdout} stderr=${r.stderr}`);

    const legacyDrift = mergeNewWorktree(mainB, 'wsr-legacy-drift');
    fs.rmSync(path.join(legacyDrift.worktreeRoot, identityRel), { force: true });
    renameWorktreeFeature(legacyDrift.worktreeRoot, 'wsr-legacy-renamed');
    fs.rmSync(path.join(legacyDrift.worktreeRoot, identityRel), { force: true });
    const r2 = bee(mainB, ['worktree', 'merge', '--id', legacyDrift.id, '--json']);
    // Read the message out of the JSON envelope rather than the raw stdout —
    // the quoted branch names are backslash-escaped in the raw bytes.
    let out2 = r2.stdout + r2.stderr;
    try {
      out2 = JSON.parse(r2.stdout).error;
    } catch {
      /* fall back to the raw text; the assertions below fail loudly either way */
    }
    const sameAsBefore = r2.status !== 0 && /WORKTREE_MERGE_BRANCH_MISMATCH/.test(out2) && /not its expected "wt\/wsr-legacy-renamed" branch/.test(out2);
    record('(#46) a worktree with NO immutable record that WAS renamed still refuses exactly as it did before — no new refusal, no crash', sameAsBefore, out2);
    const namesTheField = /MUTABLE \.bee\/state\.json "feature" field/.test(out2) && /The branch name is fixed at creation; do not rename it to match/.test(out2);
    record('(#46) even in that degraded case the message points at the mutable field rather than at the branch', namesTheField, out2);
  }

  // ── hardening-4c (issue #56 3.9 / CONTEXT.md D10b): the 'worktree-admin'
  // lock is now RELEASED around the verify child instead of held across it
  // — the exact opposite of what the old "D2 regression" sleep-based test
  // used to assert here (it proved the lock stayed held for the verify's
  // whole 45s duration, past STALE_MS, and that a second lock attempt was
  // refused busy during that window). hardening-4c's whole point is that
  // this is no longer true: a verify-configured merge releases the lock for
  // P2 and re-acquires it for P3. That old test is replaced by the two
  // deterministic-seam tests below — no sleeps, no timing windows: an
  // INJECTED verify script self-checks/self-tampers from INSIDE its own
  // run instead (same seam discipline as lock.mjs's own
  // `_takeoverSeam`/`_postRenameSeam`, precedent cited by advisor condition
  // C4). ───────────────────────────────────────────────────────────────────

  // ── test: the lock file is ABSENT while the verify child runs. The
  // injected verify script computes bee's own real lockFilePath(mainRoot,
  // 'worktree-admin') and records whether it observed the file mid-run,
  // from INSIDE the P2 window — not timed/polled from outside. ────────────
  {
    const mainLock = path.join(mergeTmp, 'mainLockCheck');
    fs.mkdirSync(mainLock, { recursive: true });
    git(mainLock, ['init', '-q', '-b', 'main']);
    git(mainLock, ['config', 'user.email', 's@e']);
    git(mainLock, ['config', 'user.name', 's']);
    fs.writeFileSync(path.join(mainLock, '.gitignore'), BEE_GITIGNORE);
    fs.mkdirSync(path.join(mainLock, '.bee'), { recursive: true });
    fs.writeFileSync(path.join(mainLock, '.bee', 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.0.0' }));
    fs.writeFileSync(path.join(mainLock, '.bee', 'config.json'), JSON.stringify({ commands: { verify: 'node verify-lock-check.mjs' } }));
    // pathToFileURL: the injected script's static import of a bare absolute
    // win32 path would parse as protocol "d:" — a file:// URL specifier is
    // valid on every platform.
    const lockMjsAbs = pathToFileURL(path.join(REPO_ROOT, '.bee', 'bin', 'lib', 'lock.mjs')).href;
    fs.writeFileSync(
      path.join(mainLock, 'verify-lock-check.mjs'),
      [
        "import fs from 'node:fs';",
        `import { lockFilePath } from ${JSON.stringify(lockMjsAbs)};`,
        "const lockPath = lockFilePath(process.cwd(), 'worktree-admin');",
        "fs.writeFileSync('lock-observed.json', JSON.stringify({ existedDuringVerify: fs.existsSync(lockPath), lockPath }));",
        'process.exit(0);',
        '',
      ].join('\n'),
    );
    fs.writeFileSync(path.join(mainLock, 'f'), 'x');
    git(mainLock, ['add', '.']);
    git(mainLock, ['commit', '-q', '-m', 'init']);

    const lockCheckCreated = mergeNewWorktree(mainLock, 'wsr-lock-check');
    fs.writeFileSync(path.join(lockCheckCreated.worktreeRoot, 'work.txt'), 'x\n');
    git(lockCheckCreated.worktreeRoot, ['add', 'work.txt']);
    git(lockCheckCreated.worktreeRoot, ['commit', '-q', '-m', 'lock-check fixture work']);

    const lockCheckResult = bee(mainLock, ['worktree', 'merge', '--id', lockCheckCreated.id, '--json']);
    {
      const ok = lockCheckResult.status === 0;
      record(
        '(hardening-4c) lock-check merge exits 0 (fixture sanity)',
        ok,
        `status=${lockCheckResult.status} stdout=${lockCheckResult.stdout} stderr=${lockCheckResult.stderr}`,
      );
    }
    const observedPath = path.join(mainLock, 'lock-observed.json');
    let observed = null;
    try {
      observed = JSON.parse(fs.readFileSync(observedPath, 'utf8'));
    } catch {
      /* checked below */
    }
    record(
      '(hardening-4c) the injected verify script ran and recorded an observation (fixture sanity)',
      observed !== null,
      observed === null ? `missing/unreadable ${observedPath}` : JSON.stringify(observed),
    );
    record(
      "(hardening-4c / issue #56 3.9) the 'worktree-admin' lock file was ABSENT while the verify child ran (P2) — the lock is released for verify, not held across it",
      observed !== null && observed.existedDuringVerify === false,
      observed ? JSON.stringify(observed) : '(no observation recorded)',
    );
  }

  // ── test: P3 fence drift. The injected verify script tampers the STAGED
  // TREE (stages a new file via `git add`) DURING P2, while the lock is
  // released. HEAD and MERGE_HEAD never move — exactly the case advisor
  // condition C2 calls out as the reason a HEAD-only re-check is not
  // enough — but the fence's `git write-tree` identity check must catch
  // it, abort the staged merge, and throw a typed refusal; main must be
  // left byte-untouched. ────────────────────────────────────────────────
  {
    const mainFence = path.join(mergeTmp, 'mainFenceDrift');
    fs.mkdirSync(mainFence, { recursive: true });
    git(mainFence, ['init', '-q', '-b', 'main']);
    git(mainFence, ['config', 'user.email', 's@e']);
    git(mainFence, ['config', 'user.name', 's']);
    fs.writeFileSync(path.join(mainFence, '.gitignore'), BEE_GITIGNORE);
    fs.mkdirSync(path.join(mainFence, '.bee'), { recursive: true });
    fs.writeFileSync(path.join(mainFence, '.bee', 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.0.0' }));
    fs.writeFileSync(path.join(mainFence, '.bee', 'config.json'), JSON.stringify({ commands: { verify: 'node verify-tamper.mjs' } }));
    fs.writeFileSync(
      path.join(mainFence, 'verify-tamper.mjs'),
      [
        "import fs from 'node:fs';",
        "import { execSync } from 'node:child_process';",
        "fs.writeFileSync('tamper.txt', 'tampered\\n');",
        "execSync('git add tamper.txt');",
        'process.exit(0);',
        '',
      ].join('\n'),
    );
    fs.writeFileSync(path.join(mainFence, 'f'), 'x');
    git(mainFence, ['add', '.']);
    git(mainFence, ['commit', '-q', '-m', 'init']);

    const fenceCreated = mergeNewWorktree(mainFence, 'wsr-fence-drift');
    fs.writeFileSync(path.join(fenceCreated.worktreeRoot, 'work.txt'), 'x\n');
    git(fenceCreated.worktreeRoot, ['add', 'work.txt']);
    git(fenceCreated.worktreeRoot, ['commit', '-q', '-m', 'fence-drift fixture work']);
    const fencePreHead = git(mainFence, ['log', '-1', '--pretty=%H']).trim();

    const fenceResult = bee(mainFence, ['worktree', 'merge', '--id', fenceCreated.id, '--json']);
    {
      const ok = fenceResult.status !== 0 && /WORKTREE_MERGE_FENCE_DRIFT/.test(fenceResult.stdout + fenceResult.stderr);
      record(
        '(hardening-4c / advisor C2) P3 fence catches a staged-tree tamper made during the unlocked verify window and refuses (typed WORKTREE_MERGE_FENCE_DRIFT)',
        ok,
        `status=${fenceResult.status} stdout=${fenceResult.stdout} stderr=${fenceResult.stderr}`,
      );
    }
    {
      const proof = threePartProof(mainFence, fencePreHead);
      record('(hardening-4c / advisor C2) main is left byte-untouched after the fence-drift abort (three-part proof)', proof.ok, JSON.stringify(proof));
    }
    {
      const ok = !fs.existsSync(path.join(mainFence, 'tamper.txt'));
      record('(hardening-4c / advisor C2) the tampered file never lands on main', ok, path.join(mainFence, 'tamper.txt'));
    }
  }

  // ── no-test-repos D1/D2 (decision 55b951e1): commands.verify === 'none'
  // must NEVER be spawned as a shell command — handleWorktreeMerge maps the
  // sentinel to `undefined` so the existing verifySkipped loud-warning path
  // fires instead. Proven two ways: (a) the merge succeeds and reports
  // verify:"skipped" — had the literal string "none" been spawned via
  // spawnSync(..., { shell: true }), the shell would report "command not
  // found" (non-zero exit), which the verify gate treats as
  // MERGE_VERIFY_RED and aborts the merge; ok:true + verify:"skipped" is
  // only reachable by never spawning it. (b) --cleanup's loud "cleaned up
  // unchecked" warning fires, exactly as it does for a repo with no
  // commands.verify recorded at all. ──────────────────────────────────────
  {
    const mainNoTest = path.join(mergeTmp, 'mainNoTest');
    fs.mkdirSync(mainNoTest, { recursive: true });
    git(mainNoTest, ['init', '-q', '-b', 'main']);
    git(mainNoTest, ['config', 'user.email', 's@e']);
    git(mainNoTest, ['config', 'user.name', 's']);
    fs.writeFileSync(path.join(mainNoTest, '.gitignore'), BEE_GITIGNORE);
    fs.mkdirSync(path.join(mainNoTest, '.bee'), { recursive: true });
    fs.writeFileSync(path.join(mainNoTest, '.bee', 'onboarding.json'), JSON.stringify({ schema_version: '1.0', bee_version: '0.0.0' }));
    fs.writeFileSync(path.join(mainNoTest, '.bee', 'config.json'), JSON.stringify({ commands: { verify: 'none' } }));
    fs.writeFileSync(path.join(mainNoTest, 'f'), 'x');
    git(mainNoTest, ['add', '.']);
    git(mainNoTest, ['commit', '-q', '-m', 'init']);

    const noTestCreated = mergeNewWorktree(mainNoTest, 'wsr-no-test-sentinel');
    fs.writeFileSync(path.join(noTestCreated.worktreeRoot, 'no-test-work.txt'), 'work under a no-test declaration\n');
    git(noTestCreated.worktreeRoot, ['add', 'no-test-work.txt']);
    git(noTestCreated.worktreeRoot, ['commit', '-q', '-m', 'no-test fixture work']);
    const noTestResult = bee(mainNoTest, ['worktree', 'merge', '--id', noTestCreated.id, '--cleanup', '--json']);
    let noTestJson = null;
    try {
      noTestJson = JSON.parse(noTestResult.stdout);
    } catch {
      /* checked below */
    }
    {
      const ok =
        noTestResult.status === 0 &&
        noTestJson &&
        noTestJson.ok === true &&
        noTestJson.merged === true &&
        noTestJson.verify === 'skipped';
      record(
        'no-test-repos D1/D2: worktree merge with commands.verify:"none" merges and reports verify:"skipped" — the sentinel is never spawned as a shell command',
        ok,
        noTestResult.stdout,
      );
    }
    {
      const ok =
        noTestJson &&
        noTestJson.cleanup &&
        noTestJson.cleanup.ok === true &&
        /verify skipped — no commands\.verify recorded; cleaned up unchecked\./.test(noTestJson.cleanup.warning || '');
      record(
        'no-test-repos D1/D2: --cleanup after a sentinel-declared merge carries the same loud "cleaned up unchecked" warning as an unconfigured repo (verifySkipped path)',
        ok,
        JSON.stringify(noTestJson && noTestJson.cleanup),
      );
    }
    {
      const landed = fs.existsSync(path.join(mainNoTest, 'no-test-work.txt'));
      record("no-test-repos D1/D2: the worktree's committed work actually landed on main despite the sentinel", landed, mainNoTest);
    }
  }
} finally {
  fs.rmSync(mergeTmp, { recursive: true, force: true });
}

const failed = results.filter((r) => !r.passed);
console.log(`SUMMARY: ${results.length - failed.length}/${results.length} passed`);
process.exit(failed.length ? 7 : 0);
