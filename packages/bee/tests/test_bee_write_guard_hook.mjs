#!/usr/bin/env node
// test_bee_write_guard_hook.mjs — end-to-end integration tests for
// hooks/bee-write-guard.mjs itself (harness-integration-3, D4). Spawns the
// real hook script as a subprocess with crafted PreToolUse stdin payloads —
// test_lib.mjs's unit tests exercise lib/guards.mjs's exported functions
// directly, but cannot reach this file's own dispatch logic (payload
// parsing, the shared `denial` variable, the outer try/catch). Every fixture
// repo is an isolated temp dir under os.tmpdir(): the real checkout's own
// .bee/ state is never touched.
//
// Covers must_haves:
//   (a) a malformed bee.mjs-shaped Bash call is denied, structured correction
//       on stderr, before it would execute
//   (b) the existing gate guard, reservation guard, and privacy/scout guard
//       behave exactly as before this cell — zero regression
//   (c) the new check's logic never overwrites or discards a denial already
//       computed by an existing check, even when forced to throw
//   (d) (cell wux-2, GH #31) a containment denial targeting a KNOWN sibling
//       worktree (or, inversely, the main checkout from a worktree-rooted
//       session) names it and both remedies; an unknown outside path, and
//       any worktree-grants.json read failure, both keep the ORIGINAL
//       generic containment message byte-for-byte — the deny decision itself
//       never changes, only the explanatory text.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { writeJsonAtomic } from '../lib/fsutil.mjs';
import { acquireLeases } from '../lib/lease-store.mjs';

const TESTS_DIR = path.dirname(fileURLToPath(import.meta.url));
const TEMPLATES_DIR = path.dirname(TESTS_DIR);
const TEMPLATES_LIB_DIR = path.join(TEMPLATES_DIR, 'lib');
const REPO_ROOT = path.resolve(TESTS_DIR, '..', '..', '..');
const HOOK_PATH = path.join(REPO_ROOT, 'hooks', 'bee-write-guard.mjs');

let passed = 0;
let failed = 0;

function check(name, fn) {
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

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

// ─── fixture repo builder ──────────────────────────────────────────────────
// Mirrors onboard_bee.mjs's own vendoring: the hook dynamically imports
// modules from `<root>/.bee/bin/lib/*.mjs` (never from the template tree
// directly), so a realistic fixture must vendor real copies there — the
// exact prerequisite this cell's own action names (validating iteration 1,
// Blocker 4).
// The hook's dynamic import graph (rooted at state.mjs) reaches deep into
// the lib directory's transitive closure, and a hand-maintained module list
// here went stale at least twice (missing modules throw at import time in
// the fixture root, which makes the hook's own resolver fail open — denial
// tests silently regress to exit 0, decision 39be1227). Kill the class, not
// the instance: copy the whole lib directory wholesale so any future module
// (and its transitive imports) is vendored automatically, with no list to
// maintain.
function vendoredLibModuleNames() {
  return fs.readdirSync(TEMPLATES_LIB_DIR).filter((name) => name.endsWith('.mjs'));
}

// A valid-shaped registry whose one entry throws when its `parameters`
// getter is read — forces check (d)'s own parsing logic to throw (must_have
// c) without any import-time syntax error, so the failure is squarely inside
// checkCliShape()'s body, not a broken module load.
const THROWING_REGISTRY_SOURCE = `
export const SCHEMA_VERSION = '1.0';
export const COMMAND_REGISTRY = [
  {
    name: 'cells.show',
    helper: 'bee_cells.mjs',
    invoke: 'bee cells show',
    description: 'Forced-failure registry entry (test fixture only).',
    get parameters() {
      throw new Error('forced parsing failure for test');
    },
    examples: ['show --id x'],
    deprecated: null,
  },
];
`;

function makeFixtureRoot({
  phase = 'swarming',
  approvedGates = { context: true, shape: true, execution: true, review: false },
  reservations = [],
  throwingRegistry = false,
} = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-write-guard-hook-test-'));
  fs.mkdirSync(path.join(root, '.bee', 'bin', 'lib'), { recursive: true });
  fs.mkdirSync(path.join(root, '.bee', 'logs'), { recursive: true });

  for (const name of vendoredLibModuleNames()) {
    if (throwingRegistry && name === 'command-registry.mjs') continue;
    fs.copyFileSync(path.join(TEMPLATES_LIB_DIR, name), path.join(root, '.bee', 'bin', 'lib', name));
  }
  if (throwingRegistry) {
    fs.writeFileSync(path.join(root, '.bee', 'bin', 'lib', 'command-registry.mjs'), THROWING_REGISTRY_SOURCE, 'utf8');
  }

  writeJsonAtomic(path.join(root, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  writeJsonAtomic(path.join(root, '.bee', 'state.json'), {
    schema_version: '1.0',
    phase,
    feature: 'demo',
    mode: null,
    approved_gates: approvedGates,
    workers: [],
    summary: '',
    next_action: '',
  });
  // multisession-native-16: `.bee/reservations.json` is a rebuildable
  // PROJECTION over lease-store.mjs now (see reservations.mjs's own module
  // header) — the hook's reservation guard (guards.mjs's checkWrite ->
  // findConflicts) reads lease-store's files directly, so a fixture
  // reservation must be seeded as a REAL lease rather than written to the
  // legacy JSON file, which is no longer consulted for live conflicts at
  // all. acquireLeases (lease-store.mjs) is synchronous, matching this
  // file's fully-sync check() harness (spawnSync-based, no async plumbing
  // anywhere else in this file) — reservations.mjs's own reserve() is async
  // and deliberately not used here. The field mapping below (workflow_id <-
  // cell, workspace_id <- 'agent:'+agent, epoch 0, mode 'write', a
  // sessionless sentinel) is a deliberate small duplicate of reserve()'s own
  // — see reservations.mjs's module header for the canonical mapping this
  // mirrors.
  for (const entry of reservations) {
    acquireLeases(root, [
      {
        type: 'path',
        id: entry.path,
        mode: 'write',
        workflow_id: entry.cell || 'demo-1',
        session_id: ' bee-reservation-sessionless ',
        workspace_id: `agent:${entry.agent}`,
        epoch: 0,
        ttl: Number.isFinite(entry.ttl_seconds) && entry.ttl_seconds > 0 ? entry.ttl_seconds : 3600,
        kind: 'lease',
      },
    ]);
  }
  return root;
}

/** Spawn the real hook script with a crafted PreToolUse-shaped stdin payload. */
function runHook(root, payload) {
  return spawnSync(process.execPath, [HOOK_PATH], {
    cwd: root,
    input: JSON.stringify({ cwd: root, ...payload }),
    encoding: 'utf8',
  });
}

function activeReservation({ agent, path: reservedPath }) {
  return {
    agent,
    cell: 'demo-1',
    path: reservedPath,
    ttl_seconds: 3600,
    reserved_at: new Date().toISOString(),
    released_at: null,
  };
}

// ─── sibling-worktree fixture builders (GH #31) ────────────────────────────
// Fabricates the SAME bidirectional gitdir shape test_state.mjs's own
// "resolveRoots validates linked-worktree backlinks" fixture uses (and
// hooks/test_hook_contracts.mjs's runWorktreeAdapterRows) — no real `git
// worktree add` needed, just the file/dir layout resolveRoots and the hook's
// own inline mirror of it read.

// An ORDINARY main-checkout fixture (via makeFixtureRoot) plus one GRANTED
// sibling worktree registered in its runtime/worktree-grants.json. The
// sibling's root is a real directory so realpath succeeds, but it needs no
// content of its own — deriveGrantedWorktreeRoot only ever reads the
// mainRoot-side "gitdir" pointer file, same as worktree-store.mjs's own
// resolveWorktreeById.
function makeSiblingWorktreeFixture({ grantsRaw } = {}) {
  const root = makeFixtureRoot();
  const siblingRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-write-guard-sibling-'));
  const worktreeId = 'wt-sibling-demo';
  const gitdir = path.join(root, '.git', 'worktrees', worktreeId);
  fs.mkdirSync(gitdir, { recursive: true });
  fs.writeFileSync(path.join(gitdir, 'gitdir'), `${path.join(siblingRoot, '.git')}\n`);
  // Reverse pointer (sibling's own ".git" file back to this same gitdir) —
  // the SAME bidirectional shape resolveWorktreeById/resolveRoots require,
  // so the hook's forward-AND-reverse-verified resolution actually resolves
  // this fixture's sibling instead of treating its link as unproven.
  fs.writeFileSync(path.join(siblingRoot, '.git'), `gitdir: ${gitdir}\n`);
  fs.mkdirSync(path.join(root, '.bee', 'runtime'), { recursive: true });
  fs.writeFileSync(
    path.join(root, '.bee', 'runtime', 'worktree-grants.json'),
    grantsRaw !== undefined ? grantsRaw : JSON.stringify({ [worktreeId]: true }),
  );
  return { root, siblingRoot, worktreeId };
}

// A full main-checkout fixture (mainRoot, via makeFixtureRoot) plus a
// SEPARATE workRoot whose own ".git" is a FILE pointing back at
// "<mainRoot>/.git/worktrees/<id>" — the session in this fixture is ROOTED
// IN the worktree (cwd = workRoot), the inverse of makeSiblingWorktreeFixture
// above (whose session stays rooted in the ordinary main checkout).
function makeWorktreeRootedFixture(opts = {}) {
  const mainRoot = makeFixtureRoot(opts);
  const workRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-write-guard-worktree-'));
  const worktreeId = 'wt-rooted-demo';
  const gitdir = path.join(mainRoot, '.git', 'worktrees', worktreeId);
  fs.mkdirSync(gitdir, { recursive: true });
  fs.writeFileSync(path.join(workRoot, '.git'), `gitdir: ${gitdir}\n`);
  fs.writeFileSync(path.join(gitdir, 'gitdir'), `${path.join(workRoot, '.git')}\n`);
  return { mainRoot, workRoot, worktreeId };
}

// ─── (a) CLI-shape validation: malformed calls denied, well-formed allowed ─

check('a malformed bee.mjs-shaped Bash call (missing required --id) is denied before execution', () => {
  const root = makeFixtureRoot();
  const result = runHook(root, {
    tool_name: 'Bash',
    tool_input: { command: 'node .bee/bin/bee_cells.mjs show' },
  });
  assert(result.status === 2, `expected exit 2, got ${result.status} (stderr: ${result.stderr})`);
  assert(result.stderr.includes('bee CLI-shape guard'), `expected CLI-shape guard reason, got: ${result.stderr}`);
  assert(result.stderr.includes('cells.show'), `expected the resolved command name, got: ${result.stderr}`);
  assert(result.stderr.includes('field: id'), `expected the missing field named, got: ${result.stderr}`);
});

check('ce-1: a call missing every required flag renders ALL problems joined, keeping the pinned substrings ("bee CLI-shape guard", entry name, "field: <first>")', () => {
  const root = makeFixtureRoot();
  const result = runHook(root, {
    tool_name: 'Bash',
    tool_input: { command: 'node .bee/bin/bee_cells.mjs tier' },
  });
  assert(result.status === 2, `expected exit 2, got ${result.status} (stderr: ${result.stderr})`);
  assert(result.stderr.includes('bee CLI-shape guard'), `expected CLI-shape guard reason, got: ${result.stderr}`);
  assert(result.stderr.includes('cells.tier'), `expected the resolved command name, got: ${result.stderr}`);
  assert(result.stderr.includes('field: id'), `expected the FIRST problem field named, got: ${result.stderr}`);
  const requiredMisses = result.stderr.match(/required, missing/g) || [];
  assert(requiredMisses.length === 2, `expected BOTH missing fields (id, tier) reported — one "required, missing" per problem, got ${requiredMisses.length} in: ${result.stderr}`);
  assert(result.stderr.includes('(--tier)'), `expected the second problem\'s own field annotated, got: ${result.stderr}`);
});

check('a well-formed bee.mjs-shaped Bash call is allowed (check (d) does not false-positive)', () => {
  const root = makeFixtureRoot();
  const result = runHook(root, {
    tool_name: 'Bash',
    tool_input: { command: 'node .bee/bin/bee_cells.mjs show --id demo-1' },
  });
  assert(result.status === 0, `expected exit 0, got ${result.status} (stderr: ${result.stderr})`);
});

check('a flag value that legitimately begins with "--" is consumed as the value, not misread as a new flag (P1 fix, review-phase-1.md)', () => {
  const root = makeFixtureRoot();
  const result = runHook(root, {
    tool_name: 'Bash',
    tool_input: {
      command: 'node .bee/bin/bee_decisions.mjs log --decision "--foo" --rationale bar',
    },
  });
  // Before the fix: parseCliFlags treated "--foo" as a new bare flag instead
  // of --decision's value, so --decision resolved to boolean true, failed
  // the string-type schema check, and the call was denied. bee.mjs's own
  // parseFlags always consumes the next token unconditionally — the two
  // parsers must agree, and both must ALLOW this well-formed call.
  assert(result.status === 0, `expected exit 0 (dispatcher accepts this call), got ${result.status} (stderr: ${result.stderr})`);
});

check('a legacy bee_reservations.mjs boolean flag does not over-consume a trailing positional token', () => {
  const root = makeFixtureRoot();
  const result = runHook(root, {
    tool_name: 'Bash',
    tool_input: { command: 'node .bee/bin/bee_reservations.mjs sweep --json notaboolean' },
  });
  // "notaboolean" is a non-flag token following a boolean flag, so it is
  // never consumed as its value (schema-driven parse) — sweep --json alone
  // is well-formed, so this call is expected to be ALLOWED, proving the
  // parser does not over-consume trailing positional noise.
  assert(result.status === 0, `expected exit 0 (json is a no-value boolean flag), got ${result.status} (stderr: ${result.stderr})`);
});

check('a valid 3-token helper call (state worker add, longest-prefix resolution) passes schema validation (du-6)', () => {
  const root = makeFixtureRoot();
  const result = runHook(root, {
    tool_name: 'Bash',
    tool_input: { command: 'node .bee/bin/bee_state.mjs worker add --nickname w1 --cell c1 --json' },
  });
  assert(result.status === 0, `expected exit 0, got ${result.status} (stderr: ${result.stderr})`);
});

check('an invalid 3-token helper call is BLOCKED with the schema-guard message, not silently fail-open (du-6)', () => {
  const root = makeFixtureRoot();
  const result = runHook(root, {
    tool_name: 'Bash',
    tool_input: {
      // Before the fix, resolveCliCommandName's hardcoded 2-token shape
      // resolved this to the bogus name "state.worker" (no such registry
      // entry), so check (d) broke out and fail-opened at exit 0 — losing
      // schema validation entirely for every 3-token command. --json is
      // boolean-typed; giving it a non-boolean value is what the extended
      // registry's own validate() catches once resolution correctly lands
      // on "state.worker.add".
      command: 'node .bee/bin/bee_state.mjs worker add --nickname w1 --cell c1 --json=notaboolean',
    },
  });
  assert(result.status === 2, `expected exit 2, got ${result.status} (stderr: ${result.stderr})`);
  assert(result.stderr.includes('bee CLI-shape guard'), `expected CLI-shape guard reason, got: ${result.stderr}`);
  assert(result.stderr.includes('state.worker.add'), `expected the resolved 3-token command name, got: ${result.stderr}`);
  assert(result.stderr.includes('field: json'), `expected the offending field named, got: ${result.stderr}`);
});

check('a valid 3-token dispatcher-shaped call (bee.mjs state worker add) passes schema validation (du-6)', () => {
  const root = makeFixtureRoot();
  const result = runHook(root, {
    tool_name: 'Bash',
    tool_input: { command: 'node .bee/bin/bee.mjs state worker add --nickname w1 --cell c1 --json' },
  });
  assert(result.status === 0, `expected exit 0, got ${result.status} (stderr: ${result.stderr})`);
});

check('an unrecognized (non-bee-shaped) Bash call is left alone by check (d)', () => {
  const root = makeFixtureRoot();
  const result = runHook(root, {
    tool_name: 'Bash',
    tool_input: { command: 'ls -la' },
  });
  assert(result.status === 0, `expected exit 0, got ${result.status} (stderr: ${result.stderr})`);
});

// ─── (b) existing checks unaffected — zero regression, via the real hook ──

check('direct-edit guard: a write to docs/backlog.md is denied naming the owning verbs (backlog-unification bu-3)', () => {
  const root = makeFixtureRoot();
  const result = runHook(root, {
    tool_name: 'Write',
    tool_input: { file_path: 'docs/backlog.md', content: 'x\n' },
  });
  assert(result.status === 2, `expected exit 2, got ${result.status} (stderr: ${result.stderr})`);
  assert(result.stderr.includes('direct-edit'), `expected the direct-edit guard identified, got: ${result.stderr}`);
  assert(result.stderr.includes('bee.mjs backlog pbi add'), `expected pbi add named, got: ${result.stderr}`);
  assert(result.stderr.includes('bee.mjs backlog pbi status'), `expected pbi status named, got: ${result.stderr}`);
  assert(result.stderr.includes('bee.mjs backlog pbi amend'), `expected pbi amend named, got: ${result.stderr}`);
  assert(result.stderr.includes('bee.mjs backlog render --write'), `expected render --write named, got: ${result.stderr}`);
});

check('direct-edit guard: a write elsewhere under docs/ still passes (rest of docs/ unaffected)', () => {
  const root = makeFixtureRoot();
  const result = runHook(root, {
    tool_name: 'Write',
    tool_input: { file_path: 'docs/specs/some-area.md', content: 'x\n' },
  });
  assert(result.status === 0, `expected exit 0, got ${result.status} (stderr: ${result.stderr})`);
});

check('gate guard (idle intake gate): a write outside the allowed prefixes is still denied', () => {
  const root = makeFixtureRoot({ phase: 'idle' });
  const result = runHook(root, {
    tool_name: 'Write',
    tool_input: { file_path: 'src/foo.js' },
  });
  assert(result.status === 2, `expected exit 2, got ${result.status} (stderr: ${result.stderr})`);
  assert(result.stderr.includes('bee intake gate'), `expected intake gate reason, got: ${result.stderr}`);
});

check('gate guard (idle intake gate): a write inside an allowed prefix is still allowed', () => {
  const root = makeFixtureRoot({ phase: 'idle' });
  const result = runHook(root, {
    tool_name: 'Write',
    tool_input: { file_path: 'docs/history/x.md' },
  });
  assert(result.status === 0, `expected exit 0, got ${result.status} (stderr: ${result.stderr})`);
});

check('gate guard (terminal intake gate): a source write after a feature closes is denied, gates still approved (c2c46488)', () => {
  const root = makeFixtureRoot({
    phase: 'compounding-complete',
    approvedGates: { context: true, shape: true, execution: true, review: true },
  });
  const result = runHook(root, {
    tool_name: 'Edit',
    tool_input: { file_path: 'assets/css/tasks.css' },
  });
  assert(result.status === 2, `expected exit 2, got ${result.status} (stderr: ${result.stderr})`);
  assert(result.stderr.includes('bee intake gate'), `expected intake gate reason, got: ${result.stderr}`);
  assert(
    result.stderr.includes('compounding-complete'),
    `the block must name the terminal phase it fired on, got: ${result.stderr}`,
  );
});

check('gate guard (terminal intake gate): docs/ stays writable after a feature closes (scribing/compounding)', () => {
  const root = makeFixtureRoot({
    phase: 'compounding-complete',
    approvedGates: { context: true, shape: true, execution: true, review: true },
  });
  const result = runHook(root, {
    tool_name: 'Write',
    tool_input: { file_path: 'docs/specs/tasks.md' },
  });
  assert(result.status === 0, `expected exit 0, got ${result.status} (stderr: ${result.stderr})`);
});

check('gate guard (gated phase): a write before execution approval is still denied', () => {
  const root = makeFixtureRoot({
    phase: 'exploring',
    approvedGates: { context: true, shape: false, execution: false, review: false },
  });
  const result = runHook(root, {
    tool_name: 'Write',
    tool_input: { file_path: 'src/foo.js' },
  });
  assert(result.status === 2, `expected exit 2, got ${result.status} (stderr: ${result.stderr})`);
  assert(result.stderr.includes('bee gate'), `expected gate reason, got: ${result.stderr}`);
});

check('gate guard (gated phase): a write after execution approval is still allowed', () => {
  const root = makeFixtureRoot({
    phase: 'exploring',
    approvedGates: { context: true, shape: true, execution: true, review: false },
  });
  const result = runHook(root, {
    tool_name: 'Write',
    tool_input: { file_path: 'src/foo.js' },
  });
  assert(result.status === 0, `expected exit 0, got ${result.status} (stderr: ${result.stderr})`);
});

check('reservation guard: a write conflicting with another agent\'s reservation is still denied', () => {
  const root = makeFixtureRoot({
    phase: 'swarming',
    reservations: [activeReservation({ agent: 'other-agent', path: 'src/reserved.js' })],
  });
  const result = runHook(root, {
    tool_name: 'Bash',
    tool_input: { command: 'rm src/reserved.js' },
    agent_name: 'me',
  });
  assert(result.status === 2, `expected exit 2, got ${result.status} (stderr: ${result.stderr})`);
  assert(result.stderr.includes('bee reservation conflict'), `expected reservation conflict reason, got: ${result.stderr}`);
});

check('reservation guard: a write by the reservation\'s own holder is still allowed', () => {
  const root = makeFixtureRoot({
    phase: 'swarming',
    reservations: [activeReservation({ agent: 'me', path: 'src/reserved.js' })],
  });
  const result = runHook(root, {
    tool_name: 'Bash',
    tool_input: { command: 'rm src/reserved.js' },
    agent_name: 'me',
  });
  assert(result.status === 0, `expected exit 0, got ${result.status} (stderr: ${result.stderr})`);
});

check('privacy guard: reading a secret-shaped file is still denied with the @@BEE_PRIVACY@@ marker', () => {
  const root = makeFixtureRoot();
  const result = runHook(root, {
    tool_name: 'Read',
    tool_input: { file_path: '.env' },
  });
  assert(result.status === 2, `expected exit 2, got ${result.status} (stderr: ${result.stderr})`);
  assert(result.stderr.includes('@@BEE_PRIVACY@@'), `expected the privacy marker, got: ${result.stderr}`);
});

check('scout guard: reading inside node_modules/ is still denied', () => {
  const root = makeFixtureRoot();
  const result = runHook(root, {
    tool_name: 'Read',
    tool_input: { file_path: 'node_modules/foo/index.js' },
  });
  assert(result.status === 2, `expected exit 2, got ${result.status} (stderr: ${result.stderr})`);
  assert(result.stderr.includes('bee scout guard'), `expected scout guard reason, got: ${result.stderr}`);
});

check('a plain source read is still allowed', () => {
  const root = makeFixtureRoot();
  const result = runHook(root, {
    tool_name: 'Read',
    tool_input: { file_path: 'src/index.js' },
  });
  assert(result.status === 0, `expected exit 0, got ${result.status} (stderr: ${result.stderr})`);
});

// ─── (c) a forced throw in check (d) never erases an existing denial ──────

check('a denial already computed by the reservation guard survives check (d) being forced to throw', () => {
  const root = makeFixtureRoot({
    phase: 'swarming',
    reservations: [activeReservation({ agent: 'other-agent', path: 'src/reserved.js' })],
    throwingRegistry: true,
  });
  // First segment (`rm src/reserved.js`) trips the reservation guard (check
  // b) BEFORE check (d) ever runs; the second segment is bee-cli-shaped and
  // resolves to the fixture's throwing registry entry, forcing check (d)'s
  // own parsing logic (the `entry.parameters` read inside checkCliShape) to
  // throw on the very same Bash call.
  const result = runHook(root, {
    tool_name: 'Bash',
    tool_input: { command: 'rm src/reserved.js && node .bee/bin/bee_cells.mjs show --id demo-1' },
    agent_name: 'me',
  });
  assert(result.status === 2, `expected exit 2 (original deny must survive), got ${result.status} (stderr: ${result.stderr})`);
  assert(
    result.stderr.includes('bee reservation conflict'),
    `expected the ORIGINAL reservation-conflict reason to still be reported, got: ${result.stderr}`,
  );
  assert(
    !result.stderr.includes('CLI-shape guard'),
    `check (d) must never have been allowed to assign its own denial once check (b) already denied, got: ${result.stderr}`,
  );
  // The forced throw is still visible in the crash log — fail-open for this
  // check alone is silent to the tool call, never silent in the audit trail.
  const crashLog = fs.readFileSync(path.join(root, '.bee', 'logs', 'hooks.jsonl'), 'utf8');
  assert(
    crashLog.includes('forced parsing failure for test'),
    `expected the forced throw to be logged to hooks.jsonl, got: ${crashLog}`,
  );
});

check('with no prior denial, a forced throw in check (d) fails open (allow) rather than crashing', () => {
  const root = makeFixtureRoot({ phase: 'swarming', throwingRegistry: true });
  const result = runHook(root, {
    tool_name: 'Bash',
    tool_input: { command: 'node .bee/bin/bee_cells.mjs show --id demo-1' },
  });
  assert(result.status === 0, `expected exit 0 (fail-open on the new check's own crash), got ${result.status} (stderr: ${result.stderr})`);
});

// ─── (d) sibling-worktree-aware containment denial message (GH #31) ───────
// Message-only: the deny decision (exit 2) is untouched in every row below —
// only the stderr text changes when the target proves to be a known
// sibling/main checkout instead of a plain unrecognizable outside path.

check('a denial targeting a granted sibling worktree names the worktree id and both remedies', () => {
  const { root, siblingRoot, worktreeId } = makeSiblingWorktreeFixture();
  const target = path.join(siblingRoot, 'src', 'foo.js');
  const result = runHook(root, {
    tool_name: 'Edit',
    tool_input: { file_path: target },
  });
  assert(result.status === 2, `expected exit 2, got ${result.status} (stderr: ${result.stderr})`);
  assert(result.stderr.includes(worktreeId), `expected the worktree id named, got: ${result.stderr}`);
  assert(
    result.stderr.includes(`cwd=${siblingRoot}`) || result.stderr.includes(siblingRoot),
    `expected the "open a session with cwd=<worktree root>" remedy, got: ${result.stderr}`,
  );
  assert(
    result.stderr.includes(`bee worktree merge --id ${worktreeId}`),
    `expected the "merge it back from main" remedy, got: ${result.stderr}`,
  );
});

check('a denial targeting a granted sibling worktree via a Bash-extracted target also names it (still exit 2)', () => {
  const { root, siblingRoot, worktreeId } = makeSiblingWorktreeFixture();
  const target = path.join(siblingRoot, 'src', 'foo.js');
  const result = runHook(root, {
    tool_name: 'Bash',
    tool_input: { command: `rm ${target}` },
  });
  assert(result.status === 2, `expected exit 2, got ${result.status} (stderr: ${result.stderr})`);
  assert(result.stderr.includes(worktreeId), `expected the worktree id named, got: ${result.stderr}`);
  assert(
    result.stderr.includes(`bee worktree merge --id ${worktreeId}`),
    `expected the "merge it back from main" remedy, got: ${result.stderr}`,
  );
});

check('an unknown outside path (no matching granted sibling) keeps the generic containment message', () => {
  const { root } = makeSiblingWorktreeFixture();
  const unrelated = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-write-guard-unrelated-'));
  const target = path.join(unrelated, 'foo.js');
  const result = runHook(root, {
    tool_name: 'Edit',
    tool_input: { file_path: target },
  });
  assert(result.status === 2, `expected exit 2, got ${result.status} (stderr: ${result.stderr})`);
  assert(
    result.stderr.includes('could not be canonically contained inside the physical worktree'),
    `expected the generic containment message, got: ${result.stderr}`,
  );
  assert(!result.stderr.includes('worktree "'), `expected NO worktree naming for an unknown path, got: ${result.stderr}`);
});

check('an unparseable grants file falls back to the generic message (fail-open to generic, never allow, never crash)', () => {
  const { root, siblingRoot } = makeSiblingWorktreeFixture({ grantsRaw: '{ not valid json' });
  const target = path.join(siblingRoot, 'src', 'foo.js');
  const result = runHook(root, {
    tool_name: 'Edit',
    tool_input: { file_path: target },
  });
  assert(result.status === 2, `expected exit 2 (never allow on a grants-read failure), got ${result.status} (stderr: ${result.stderr})`);
  assert(
    result.stderr.includes('could not be canonically contained inside the physical worktree'),
    `expected the generic containment message, got: ${result.stderr}`,
  );
  assert(!result.stderr.includes('worktree "'), `expected NO worktree naming when grants are unparseable, got: ${result.stderr}`);
});

check('a session rooted in a worktree denies a target inside the MAIN checkout with the inverse message', () => {
  const { mainRoot, workRoot } = makeWorktreeRootedFixture();
  const target = path.join(mainRoot, 'src', 'foo.js');
  const result = runHook(workRoot, {
    tool_name: 'Edit',
    tool_input: { file_path: target },
  });
  assert(result.status === 2, `expected exit 2, got ${result.status} (stderr: ${result.stderr})`);
  assert(
    result.stderr.includes('main checkout'),
    `expected the inverse "belongs to the main checkout" message, got: ${result.stderr}`,
  );
  assert(
    result.stderr.includes('session rooted there'),
    `expected the "run this from a session rooted there" remedy, got: ${result.stderr}`,
  );
});

// ─── summary ────────────────────────────────────────────────────────────────

console.log(`\n${passed} passed, ${failed} failed`);
process.exitCode = failed > 0 ? 1 : 0;
