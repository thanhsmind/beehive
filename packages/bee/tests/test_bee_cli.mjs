#!/usr/bin/env node
// test_bee_cli.mjs — self-contained contract tests for the shared command
// registry and args validator (no framework). Creates a temp repo under
// os.tmpdir() (mirrors test_lib.mjs's isolation pattern) and NEVER runs a
// registry example against this checkout's real .bee/ state — several
// examples are state-mutating cell/decision/reservation operations that
// would corrupt this repo's own tracking data if run for real here.
//
// Covers:
//   1. every COMMAND_REGISTRY entry's `parameters` is valid JSON-Schema (D3 shape)
//   2. validate() rejects a missing required field with the structured
//      {ok:false, error:{field, reason, command}} shape, and never throws
//   3. every entry's examples[] executes successfully against the real
//      underlying helper script, inside the isolated temp repo

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { runModuleWorker } from '../../../scripts/lib/run-module-worker.mjs';

import { SCHEMA_VERSION, COMMAND_REGISTRY } from '../lib/command-registry.mjs';
import { validate, isValidParameterSchema } from '../lib/validate-args.mjs';
import { addCell, updateCell, deriveRegenGuards, regenObligationRefusal } from '../lib/cells.mjs';
import { createSession, bindSessionLane } from '../lib/claims.mjs';
import { writeJsonAtomic, hashFile, appendJsonl } from '../lib/fsutil.mjs';
import { defaultState, writeState, writeLane, BEE_VERSION } from '../lib/state.mjs';
import { listWorkflows } from '../lib/workflow-store.mjs';
import { buildSessionPreamble } from '../lib/inject.mjs';
import { mirrorHold, findForeignHolds } from '../lib/worktree-holds.mjs';
import { ANCHOR_NUDGE_COMMAND } from '../lib/compaction.mjs';
import { encodeProjectDir } from '../lib/perf.mjs';
import { emitFrontmatter } from '../lib/knowledge.mjs';
import {
  splitCommandTokens,
  resolveCommand,
  parseFlags,
  nearestCommandName,
  deprecatedRedirect,
  computeManifestHash,
  manifestLintWarning,
  judgeStandardWarning,
} from '../bee.mjs';

const TESTS_DIR = path.dirname(fileURLToPath(import.meta.url));
const TEMPLATES_DIR = path.dirname(TESTS_DIR);

// Declared here (not near their first heavy use further down) so that
// runExample — called from check() blocks starting near the top of the
// file — can reference BEE_MJS without a temporal-dead-zone ReferenceError.
const BEE_MJS = path.join(TEMPLATES_DIR, 'bee.mjs');

let passed = 0;
let failed = 0;

async function check(name, fn) {
  try {
    await fn();
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

function entryByName(name) {
  const entry = COMMAND_REGISTRY.find((e) => e.name === name);
  assert(entry, `registry is missing entry "${name}"`);
  return entry;
}

// Tokenize a shell-like example string: whitespace-separated tokens, with
// "double-quoted segments" kept as one token. Every example in the registry
// deliberately avoids nested quotes, so this stays simple on purpose.
function tokenize(exampleString) {
  const tokens = [];
  const re = /"([^"]*)"|(\S+)/g;
  let match;
  while ((match = re.exec(exampleString)) !== null) {
    tokens.push(match[1] !== undefined ? match[1] : match[2]);
  }
  return tokens;
}

// ─── isolated temp repo (mirrors test_lib.mjs's os.tmpdir() pattern) ───────

const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-test-'));
fs.mkdirSync(path.join(root, '.bee'), { recursive: true });
writeJsonAtomic(path.join(root, '.bee', 'onboarding.json'), {
  schema_version: '1.0',
  bee_version: '0.1.0',
});
// cells.claim refuses unless Gate 3 (execution) is approved; the example
// sequence below claims a cell, so the fixture repo must already be past
// that gate.
writeState(root, {
  ...defaultState(),
  phase: 'swarming',
  feature: 'demo',
  approved_gates: { context: true, shape: true, execution: true, review: false },
});

// perf group: redirect the global perf log and the Claude transcript root into
// the temp repo so perf examples never touch the real ~/.config/beehive or
// ~/.claude. runModuleWorker inherits process.env by default, so these reach
// the dispatched worker. With no transcript under the fake CLAUDE_CONFIG_DIR,
// perf resolves an empty window and degrades to zeroed metrics (never throws).
process.env.BEEHIVE_PERF_DIR = path.join(root, 'perf-global');
process.env.CLAUDE_CONFIG_DIR = path.join(root, 'fake-claude');

// decision-propagation dp-5 (CONTEXT D7c): unlike supersede/redact, the
// registry's `decisions tag` example validates that --target actually
// resolves to a decide/supersede event — a random/placeholder id would
// refuse. Decision ids are crypto.randomUUID()-generated at write time, so
// the registry's example string (a fixed, documentation-friendly zero id,
// matching the supersede/redact convention) can only succeed if a matching
// event is pre-seeded here, before any example runs — mirrors the "demo-1"
// cell fixture's same well-known-id shape, one level down at the decisions
// store. Harmless to the later decisions.supersede/redact examples (which
// also target this same zero id): decisions tag's target resolution reads
// the raw active+archive union by TYPE only, never by superseded/redacted/
// archived status, so this event stays a valid tag target throughout the
// whole example chain below regardless of what happens to it meanwhile.
appendJsonl(path.join(root, '.bee', 'decisions.jsonl'), {
  id: '00000000-0000-0000-0000-000000000000',
  type: 'decide',
  date: new Date().toISOString(),
  decision: 'Fixture target for the decisions.tag registry example',
  rationale: 'decision-propagation dp-5: a well-known id the tag example can always resolve',
  scope: 'repo',
});

const executedNames = new Set();

/** Run the executable-th (default 0) example of a registry entry inside `root`.
 * P1 fix (review-phase-1.md): examples are now full dispatcher-form commands
 * ("bee cells show --id demo-1 --json"), consistent with each entry's own
 * `invoke` string. Execute them through the real dispatcher (bee.mjs) — the
 * surface the manifest actually advertises — rather than the legacy helper,
 * which the manifest-as-tested-contract claim did not previously cover. */
async function runExample(entryName, { exampleIndex = 0, cwd = root } = {}) {
  const entry = entryByName(entryName);
  executedNames.add(entry.name);
  const exampleString = entry.examples[exampleIndex];
  assert(typeof exampleString === 'string' && exampleString.trim(), `${entry.name}: examples[${exampleIndex}] must be a non-empty string`);
  const tokens = tokenize(exampleString);
  assert(tokens[0] === 'bee', `${entry.name}: example must be full dispatcher-form starting with "bee", got "${exampleString}"`);
  const args = tokens.slice(1);
  const result = await runModuleWorker(BEE_MJS, {
    args,
    cwd,
  });
  return { entry, result };
}

async function assertExampleOk(entryName, opts) {
  const { entry, result } = await runExample(entryName, opts);
  assert(
    result.status === 0,
    `${entry.name} example "${entry.examples[0]}" exited ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`,
  );
  return result;
}

// ─── registry shape (D3: JSON-Schema parameters, no bespoke format) ────────

await check('SCHEMA_VERSION is the top-level manifest field, not per-entry', async () => {
  assert(SCHEMA_VERSION === '1.0', `expected "1.0", got ${SCHEMA_VERSION}`);
  assert(
    COMMAND_REGISTRY.every((entry) => entry.schema_version === undefined),
    'schema_version must never appear on a per-entry basis',
  );
});

await check('every registry entry has the required manifest fields, no TODO/stub entries', async () => {
  assert(Array.isArray(COMMAND_REGISTRY) && COMMAND_REGISTRY.length > 0, 'registry must be a non-empty array');
  for (const entry of COMMAND_REGISTRY) {
    assert(typeof entry.name === 'string' && entry.name.trim(), `entry missing a name: ${JSON.stringify(entry)}`);
    assert(typeof entry.invoke === 'string' && entry.invoke.trim(), `${entry.name}: missing invoke`);
    assert(typeof entry.description === 'string' && entry.description.trim(), `${entry.name}: missing description`);
    assert(Array.isArray(entry.examples) && entry.examples.length > 0, `${entry.name}: examples must be non-empty`);
    assert('deprecated' in entry, `${entry.name}: deprecated field must be present (null when not deprecated)`);
  }
});

await check('every registry entry\'s parameters is valid JSON-Schema (D3 shape: type/properties/required)', async () => {
  for (const entry of COMMAND_REGISTRY) {
    assert(isValidParameterSchema(entry.parameters), `${entry.name}: parameters is not valid JSON-Schema — ${JSON.stringify(entry.parameters)}`);
    assert(entry.parameters.type === 'object', `${entry.name}: parameters.type must be "object"`);
  }
});

await check('registry names are unique and dot-namespaced by group (status, cells.*, reservations.*, decisions.*, state.*, backlog.*, capture.*, reviews.*, feedback.*, perf.*, worktree.*)', async () => {
  const names = COMMAND_REGISTRY.map((e) => e.name);
  assert(new Set(names).size === names.length, `duplicate names in registry: ${names.join(', ')}`);
  const groups = new Set(names.map((n) => (n.includes('.') ? n.split('.')[0] : n)));
  for (const group of groups) {
    assert(['status', 'doctor', 'cells', 'reservations', 'decisions', 'state', 'backlog', 'capture', 'reviews', 'feedback', 'perf', 'worktree', 'herding', 'config', 'dispatch', 'recovery', 'tmp', 'knowledge', 'intent'].includes(group), `unexpected group "${group}"`);
  }
});

await check('registry covers every subcommand of the 4 existing helpers', async () => {
  const names = new Set(COMMAND_REGISTRY.map((e) => e.name));
  const expected = [
    'status',
    'cells.list', 'cells.ready', 'cells.show', 'cells.add', 'cells.update', 'cells.claim',
    'cells.verify', 'cells.cap', 'cells.block', 'cells.drop', 'cells.tier', 'cells.judge',
    'reservations.reserve', 'reservations.release', 'reservations.list', 'reservations.sweep',
    'decisions.log', 'decisions.supersede', 'decisions.redact', 'decisions.active', 'decisions.search',
  ];
  for (const name of expected) {
    assert(names.has(name), `registry is missing subcommand "${name}"`);
  }
});

// ─── DA5: registry <-> runtime-verb bijection (drift guard) ────────────────
// Derives each group's verb list from RUNTIME BEHAVIOR — the "Unknown
// command ... Use: v1, v2, ..." contract line bee.mjs's own dispatcher
// already prints for an unrecognized top-level command in that group — never
// by reading/grepping bee.mjs's own source. Critical pattern 20260710: a
// drift guard that greps a module's own source pins syntax, not behavior,
// and pinned syntax can be the bug. This is the exact gap the PR shipped
// with: bee_cells.mjs's `update` verb existed on the helper but had no
// matching registry entry. The 9 bee_*.mjs shims are retired (shim-retire
// D1/D5) — the probe now spawns bee.mjs directly with the group token
// prepended, exactly what each shim used to do internally, so the observed
// "Unknown command" contract line is unchanged.

const GROUP_NAMES = ['cells', 'reservations', 'decisions', 'state', 'backlog', 'capture', 'reviews', 'feedback', 'perf', 'worktree', 'dispatch', 'recovery', 'tmp', 'knowledge'];

// Parse ONLY the stderr line that starts with "Unknown command" (trap t2:
// bee.mjs's own `cells update` verb separately emits an unrelated
// flag-level "Use: --id ID --file ..." line; anchoring on any "Use:"
// substring, rather than this specific contract line, would risk picking
// that one up under a different argv). Run inside `root`, an already
// bee-onboarded temp repo (created above) — bee.mjs refuses to run outside a
// bee repo root at all, so probing needs a real one, not a mutation of it
// (an unrecognized command never reaches any handler).
async function groupRuntimeVerbs(group) {
  const result = await runModuleWorker(BEE_MJS, {
    args: [group, '__bee_bijection_probe__'],
    cwd: root,
  });
  const contractLine = (result.stderr || '').split('\n').find((line) => line.startsWith('Unknown command'));
  assert(
    contractLine,
    `bee.mjs ${group}: expected a stderr line starting with "Unknown command" for an unrecognized top-level command, got stdout=${result.stdout} stderr=${result.stderr}`,
  );
  // Stop at the FIRST verb-list-terminating period, not necessarily end of
  // line: the reviews group's default message appends a trailing "(review
  // modes: ...)" annotation AFTER the verb list's own period (dispatcher-
  // unify du-3) — a greedy-to-end-of-line capture would swallow that
  // annotation as bogus extra "verbs". Every other group's Use: line puts
  // its own terminating period at the true end of the string, so this is a
  // no-op there (trap t1 still applies: without stopping at the period, the
  // last verb would parse as e.g. "judge.").
  const match = contractLine.match(/Use: (.+?)\.(?:\s|$)/);
  assert(match, `bee.mjs ${group}: "Unknown command" line has no "Use: ..." verb-list clause: ${contractLine}`);
  // Each comma-separated segment's FIRST word is the runtime verb: every
  // group spells a single-word verb per segment except the reviews group's
  // nested "candidate add" (two words) — collapsing to its first word
  // matches the registry-side collapse (name.split('.')[0] on the nested
  // "candidate.add" segment -> "candidate", dispatcher-unify du-3).
  return match[1]
    .split(',')
    .map((v) => v.trim().split(/\s+/)[0])
    .filter(Boolean);
}

await check('DA5 bijection: every runtime verb of bee.mjs cells/reservations/decisions/state/backlog/capture/reviews/feedback has a matching registry entry, and vice versa', async () => {
  for (const group of GROUP_NAMES) {
    const runtimeVerbs = new Set(await groupRuntimeVerbs(group));
    assert(runtimeVerbs.size > 0, `bee.mjs ${group}: parsed zero runtime verbs — the parser is broken, not the dispatcher`);
    // Collapse nested verbs to their top-level segment (state.worker.add ->
    // worker) so the bijection matches the dispatcher's runtime "Use:" line,
    // which lists only top-level verbs. For flat groups (cells/reservations/
    // decisions) this is a no-op — every verb is already single-segment.
    const registryVerbs = new Set(
      COMMAND_REGISTRY.filter((e) => e.name.startsWith(`${group}.`)).map(
        (e) => e.name.slice(group.length + 1).split('.')[0],
      ),
    );

    // (a) every runtime verb has a registry entry named `<group>.<verb>`
    const missingInRegistry = [...runtimeVerbs].filter((v) => !registryVerbs.has(v));
    assert(
      missingInRegistry.length === 0,
      `${group}: verb(s) [${missingInRegistry.join(', ')}] exist on the bee.mjs ${group} dispatcher (runtime) but have no "${group}.<verb>" entry in COMMAND_REGISTRY — registry side owns the fix (this is the exact cells.update gap the PR shipped with)`,
    );

    // (b) every registry `<group>.*` entry corresponds to a runtime verb
    const extraInRegistry = [...registryVerbs].filter((v) => !runtimeVerbs.has(v));
    assert(
      extraInRegistry.length === 0,
      `${group}: registry entr(y/ies) [${extraInRegistry.map((v) => `${group}.${v}`).join(', ')}] have no matching runtime verb on the bee.mjs ${group} dispatcher — registry side owns the fix (stale entry, or the dispatcher renamed/dropped this verb)`,
    );
  }
});

await check('DA5 bijection: the only dot-free registry entries are "status" and "doctor", and every entry\'s group is one of status|doctor|cells|reservations|decisions|state|backlog|capture|reviews|feedback|perf|worktree|config', async () => {
  const allowedGroups = new Set(['status', 'doctor', 'cells', 'reservations', 'decisions', 'state', 'backlog', 'capture', 'reviews', 'feedback', 'perf', 'worktree', 'herding', 'config', 'dispatch', 'recovery', 'tmp', 'knowledge', 'intent']);
  const allowedDotFree = new Set(['status', 'doctor']);
  for (const entry of COMMAND_REGISTRY) {
    const group = entry.name.includes('.') ? entry.name.split('.')[0] : entry.name;
    assert(allowedGroups.has(group), `${entry.name}: group "${group}" is not one of status|doctor|cells|reservations|decisions|state|backlog|capture|reviews|feedback|perf|worktree|herding|config|dispatch|tmp`);
    if (!entry.name.includes('.')) {
      assert(allowedDotFree.has(entry.name), `dot-free registry entry "${entry.name}" is not one of status|doctor — only those may be dot-free`);
    }
  }
});

// ─── validate-args.mjs: structured rejection, never a throw ────────────────

await check('validate() rejects a missing required field with the structured {field,reason,command} shape', async () => {
  const showEntry = entryByName('cells.show');
  const result = validate(showEntry, {});
  assert(result.ok === false, 'missing required "id" must not validate ok');
  assert(result.error.field === 'id', `error.field should be "id", got ${JSON.stringify(result.error)}`);
  assert(result.error.reason === 'required, missing', `error.reason should name the miss, got ${result.error.reason}`);
  assert(result.error.command === 'cells.show', `error.command should be "cells.show", got ${result.error.command}`);
});

await check('validate() accepts a call with every required field present', async () => {
  const claimEntry = entryByName('cells.claim');
  const result = validate(claimEntry, { id: 'demo-1', worker: 'worker-a' });
  assert(result.ok === true, `expected ok:true, got ${JSON.stringify(result)}`);
});

await check('validate() flags a wrong-typed value without throwing', async () => {
  const tierEntry = entryByName('cells.tier');
  const result = validate(tierEntry, { id: 'demo-1', tier: 42 });
  assert(result.ok === false, 'a number where a string tier is expected must not validate ok');
  assert(result.error.field === 'tier', `error.field should be "tier", got ${JSON.stringify(result.error)}`);
  assert(result.error.command === 'cells.tier', 'error.command should name the command');
});

await check('validate() never throws on a malformed commandEntry', async () => {
  const result = validate({ name: 'bogus' }, { anything: 'x' });
  assert(result.ok === false, 'a command with no parameters schema must not validate ok');
  assert(result.error.command === 'bogus', 'error.command still names the command');
});

await check('validate() problems[] names every missing required field, not just the first (ce-1 batch validation)', async () => {
  const tierEntry = entryByName('cells.tier');
  const result = validate(tierEntry, {});
  assert(result.ok === false, 'missing both required fields must not validate ok');
  assert(Array.isArray(result.problems), `problems must be an array, got ${JSON.stringify(result)}`);
  assert(result.problems.length === 2, `expected 2 problems (id + tier), got ${JSON.stringify(result.problems)}`);
  assert(
    result.problems.every((p) => p.reason === 'required, missing'),
    `every problem should be a required miss, got ${JSON.stringify(result.problems)}`,
  );
  const fields = result.problems.map((p) => p.field);
  assert(fields.includes('id') && fields.includes('tier'), `expected both "id" and "tier" named, got ${JSON.stringify(fields)}`);
  // error stays FIRST-problem-shaped (test_bee_cli.mjs:325 discipline) —
  // schema.required order is ['id', 'tier'], so "id" comes first.
  assert(result.error.field === 'id' && result.error.reason === 'required, missing', `error should still be the first problem, got ${JSON.stringify(result.error)}`);
});

await check('validate() enum support: an out-of-enum value is rejected with a reason naming the allowed values', async () => {
  const tierEntry = entryByName('cells.tier');
  const result = validate(tierEntry, { id: 'demo-1', tier: 'bogus-tier' });
  assert(result.ok === false, 'an out-of-enum tier must not validate ok');
  assert(result.error.field === 'tier', `error.field should be "tier", got ${JSON.stringify(result.error)}`);
  assert(/extraction/.test(result.error.reason) && /generation/.test(result.error.reason) && /ceiling/.test(result.error.reason), `reason should name the allowed tiers, got: ${result.error.reason}`);
  assert(result.problems.length === 1 && result.problems[0].field === 'tier', `problems should carry the single enum problem, got ${JSON.stringify(result.problems)}`);
});

await check('validate() problems[] combines a missing required field with an out-of-enum value in one refusal', async () => {
  const tierEntry = entryByName('cells.tier');
  const result = validate(tierEntry, { tier: 'bogus-tier' });
  assert(result.ok === false, 'missing id + bogus tier must not validate ok');
  assert(result.problems.length === 2, `expected 2 problems (missing id, invalid tier), got ${JSON.stringify(result.problems)}`);
  assert(result.problems[0].field === 'id' && result.problems[0].reason === 'required, missing', `first problem should be the missing id, got ${JSON.stringify(result.problems[0])}`);
  assert(result.problems[1].field === 'tier' && /extraction/.test(result.problems[1].reason), `second problem should name the tier enum, got ${JSON.stringify(result.problems[1])}`);
});

await check('isValidParameterSchema() rejects a bespoke (non-JSON-Schema) shape', async () => {
  assert(isValidParameterSchema({ id: 'string', worker: 'string' }) === false, 'a flat key->type map is not the D3 shape');
  assert(isValidParameterSchema({ type: 'object', properties: {}, required: ['missing'] }) === false, 'required field absent from properties must fail');
  assert(isValidParameterSchema({ type: 'object', properties: { id: { type: 'string' } }, required: [] }) === true, 'a minimal valid schema passes');
});

// ─── examples[] are tested contracts: every one runs for real, isolated ────
// Order matters here (unlike the registry's own array order): cells.add must
// run before show/claim/verify/cap/judge/tier/block/drop can succeed against
// the same fixture cell, and cells.claim needs the Gate-3 state written above.

await check('cells.add example creates the fixture cell used by the rest of the chain', async () => {
  const cellFixture = {
    id: 'demo-1',
    feature: 'demo',
    title: 'Demo cell for registry example test',
    lane: 'small',
    action: 'Exercise every cells.* example against a real fixture cell.',
    verify: 'node -e "process.exit(0)"',
  };
  fs.writeFileSync(path.join(root, 'cell-demo-1.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
  await assertExampleOk('cells.add');
  assert(fs.existsSync(path.join(root, '.bee', 'cells', 'demo-1.json')), 'demo-1 cell file should now exist');
});

await check('cells.list example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('cells.list');
  assert(result.stdout.includes('demo-1'), `expected demo-1 in list output, got ${result.stdout}`);
});

await check('cells.ready example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('cells.ready');
  assert(result.stdout.includes('demo-1'), `demo-1 should be ready (open, no deps), got ${result.stdout}`);
});

await check('cells.show example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('cells.show');
  assert(JSON.parse(result.stdout).id === 'demo-1', 'show should return the demo-1 cell');
});

await check('cells.update example runs through the real dispatcher', async () => {
  const patch = { title: 'Demo cell for registry example test (updated)' };
  fs.writeFileSync(path.join(root, 'cell-demo-1-update.json'), JSON.stringify(patch, null, 2), 'utf8');
  const result = await assertExampleOk('cells.update');
  const updated = JSON.parse(result.stdout);
  assert(updated.id === 'demo-1', `expected demo-1, got ${result.stdout}`);
  assert(updated.title === patch.title, `expected patched title, got ${result.stdout}`);
});

await check('cells.claim example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('cells.claim');
  assert(JSON.parse(result.stdout).status === 'claimed', 'demo-1 should now be claimed');
});

await check('cells.verify example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('cells.verify');
  assert(JSON.parse(result.stdout).trace.verify_passed === true, 'verify_passed should be true');
});

await check('cells.cap example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('cells.cap');
  assert(JSON.parse(result.stdout).status === 'capped', 'demo-1 should now be capped');
});

await check('cells.judge example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('cells.judge');
  assert(JSON.parse(result.stdout).hits.length === 0, 'a cell.json fixture file is not a frozen-judge pattern hit');
});

await check('cells.tier example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('cells.tier');
  assert(JSON.parse(result.stdout).tier === 'generation', 'demo-1 tier should now be "generation"');
});

await check('cells.block example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('cells.block');
  assert(JSON.parse(result.stdout).status === 'blocked', 'demo-1 should now be blocked');
});

await check('cells.drop example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('cells.drop');
  assert(JSON.parse(result.stdout).status === 'dropped', 'demo-1 should now be dropped');
});

// cells.claim-next (fresh-session-handoff fsh-11, D2/D4) needs its OWN ready
// cell — demo-1 is dropped by this point in the chain — added directly via
// addCell (not through the dispatcher, so it never consumes a registry
// example slot of its own). The fixture repo's default pipeline (feature
// "demo") already has execution approved from the root setup above, and
// "sess-claim-next" has no prior session record, so resolvePipeline resolves
// it straight to that default pipeline (D4 zero-lane parity).
await check('cells.claim-next example runs through the real dispatcher (own-lane default-pipeline pick, no prior session/lane state)', async () => {
  addCell(root, {
    id: 'demo-2',
    feature: 'demo',
    title: 'Demo cell for claim-next registry example test',
    lane: 'small',
    action: 'Exercise the cells.claim-next example against a real fixture cell.',
    verify: 'node -e "process.exit(0)"',
  });
  const result = await assertExampleOk('cells.claim-next');
  const parsed = JSON.parse(result.stdout);
  assert(parsed.ok === true && parsed.cell.id === 'demo-2', `expected demo-2 claimed, got ${result.stdout}`);
  assert(parsed.cell.status === 'claimed', 'demo-2 should now be claimed');
});

// D1: cells.schedule — plan-time only, read-only. demo-1 is dropped by this
// point in the chain (excluded from the schedulable node-set); demo-2 is
// claimed with no deps and no files, so it lands alone in wave 1 with clean
// diagnostics. The example omits --feature (schedules every cell), matching
// handleCellsReady's own no-fallback resolution: this fixture repo only has
// "demo" cells, so that is exactly wave 1's content.
await check('cells.schedule example runs through the real dispatcher (D1: waves + diagnostics, exact computeSchedule shape)', async () => {
  const result = await assertExampleOk('cells.schedule');
  const parsed = JSON.parse(result.stdout);
  assert(Array.isArray(parsed.waves), `expected a waves array, got ${result.stdout}`);
  assert(
    parsed.waves.length === 1 && parsed.waves[0].length === 1 && parsed.waves[0][0] === 'demo-2',
    `expected demo-2 alone in wave 1, got ${JSON.stringify(parsed.waves)}`,
  );
  assert(
    Array.isArray(parsed.diagnostics.cycles) && parsed.diagnostics.cycles.length === 0,
    `expected zero cycles, got ${JSON.stringify(parsed.diagnostics.cycles)}`,
  );
  assert(
    Array.isArray(parsed.diagnostics.unsatisfiable_deps) && parsed.diagnostics.unsatisfiable_deps.length === 0,
    `expected zero unsatisfiable deps, got ${JSON.stringify(parsed.diagnostics.unsatisfiable_deps)}`,
  );
});

await check('cells.schedule on an empty/zero-cell store exits 0 with empty waves (no crash, no refusal)', async () => {
  const emptyRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-schedule-empty-'));
  fs.mkdirSync(path.join(emptyRoot, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(emptyRoot, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  writeState(emptyRoot, {
    ...defaultState(),
    phase: 'swarming',
    feature: 'empty-demo',
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });
  const result = await runModuleWorker(BEE_MJS, {
    args: ['cells', 'schedule', '--json'],
    cwd: emptyRoot,
  });
  assert(result.status === 0, `expected exit 0 on an empty store, got ${result.status}: stderr=${result.stderr}`);
  const parsed = JSON.parse(result.stdout);
  assert(Array.isArray(parsed.waves) && parsed.waves.length === 0, `expected empty waves, got ${result.stdout}`);
});

// cells.reopen / cells.unclaim (GitHub #12) run LAST in the demo-1 chain: demo-1
// is "dropped" by this point (excluded from the schedule assertions above), so
// reopening it here does not disturb them.
await check('cells.reopen example runs through the real dispatcher (dropped -> open)', async () => {
  const result = await assertExampleOk('cells.reopen');
  const cell = JSON.parse(result.stdout);
  assert(cell.status === 'open', `demo-1 should be open after reopen, got ${cell.status}`);
  assert(cell.trace.verify_passed !== true, 'reopen must clear a stale passing verify');
});

await check('cells.unclaim example runs through the real dispatcher (claimed -> open)', async () => {
  await assertExampleOk('cells.claim'); // demo-1 is open after reopen; re-claim it
  const result = await assertExampleOk('cells.unclaim');
  const cell = JSON.parse(result.stdout);
  assert(cell.status === 'open', `demo-1 should be open after unclaim, got ${cell.status}`);
  assert(!cell.trace.worker, 'unclaim must release the worker');
});

// D2 + GH #27.4 (D-GHF-C): cells.reset-budget's registry example now runs
// against a deliberately budget-blocked demo-1 — resetCellBudget refuses
// (typed RESET_NOT_NEEDED) on a healthy cell, so the dispatcher-wiring proof
// must first close the door for real. The forced attempts below are
// injected directly (rather than via a claim/verify/unclaim loop) so this
// test stays independent of exactly how many ledger entries the claim/
// verify/block/drop chain above already left behind. The full exhaustion/
// refusal/reopen behavior is covered end to end in test_lib.mjs; this test
// proves the registry example (including its --operator actor) runs
// through the real dispatcher (registry -> handler -> resetCellBudget).
await check('cells.reset-budget example runs through the real dispatcher, after the door is actually closed by CELL_BUDGET_EXHAUSTED', async () => {
  const cellFile = path.join(root, '.bee', 'cells', 'demo-1.json');
  const demo1 = JSON.parse(fs.readFileSync(cellFile, 'utf8'));
  const forcedAttempts = [0, 1, 2, 3].map((i) => ({
    n: i + 1,
    at: new Date(Date.now() - (10 - i) * 1000).toISOString(),
    claim_session: `sess-reset-example-${i}`,
    claimed_at: new Date(Date.now() - (10 - i) * 1000).toISOString(),
    worker: 'w',
    verdict: 'blocked',
    failure_signature: `forced-reset-example-${i}`,
    note: null,
  }));
  demo1.trace = { ...(demo1.trace || {}), attempts: [...((demo1.trace && demo1.trace.attempts) || []), ...forcedAttempts] };
  fs.writeFileSync(cellFile, JSON.stringify(demo1, null, 2), 'utf8');

  const result = await assertExampleOk('cells.reset-budget');
  const cell = JSON.parse(result.stdout);
  assert(cell.id === 'demo-1', `expected demo-1, got ${result.stdout}`);
  assert(
    Array.isArray(cell.trace.budget_resets) && cell.trace.budget_resets.length === 1,
    `expected one budget_resets entry, got ${JSON.stringify(cell.trace.budget_resets)}`,
  );
  assert(
    typeof cell.trace.budget_resets[0].by_actor === 'string' && cell.trace.budget_resets[0].by_actor,
    `expected the example's --operator to land as by_actor, got ${JSON.stringify(cell.trace.budget_resets[0])}`,
  );
});

// D5 (self-correcting-loop): cells.judge-record's registry example, run
// against demo-1 with --builder-model/--judge-model both present and
// differing — exercises the full dispatcher wiring (registry -> handler ->
// recordJudgeVerdict -> validateJudgeVerdict/deriveModelIndependence) and
// proves the CLI's "flag presence implies pinned" derivation end to end;
// the pure-function accept/reject/independence rows are covered exhaustively
// in test_lib.mjs.
await check('cells.judge-record example runs through the real dispatcher, validates the --file payload, and stamps model_independence from --builder-model/--judge-model presence', async () => {
  const verdict = {
    schema: 'judge-verdict/1',
    verdict: 'PASS',
    checks: [{ id: 'must_haves', status: 'PASS', evidence: 'diff matches CONTEXT D5 citations' }],
    fixability: 'automatic',
    confidence: 'high',
  };
  fs.writeFileSync(path.join(root, 'verdict-demo-1.json'), JSON.stringify(verdict), 'utf8');
  const result = await assertExampleOk('cells.judge-record');
  const cell = JSON.parse(result.stdout);
  assert(cell.id === 'demo-1', `expected demo-1, got ${result.stdout}`);
  const entries = cell.trace.semantic_judge;
  assert(Array.isArray(entries) && entries.length === 1, `expected one semantic_judge entry, got ${JSON.stringify(entries)}`);
  assert(entries[0].builder_model === 'sonnet' && entries[0].judge_model === 'opus', `expected the --builder-model/--judge-model flags stored verbatim, got ${JSON.stringify(entries[0])}`);
  assert(entries[0].model_independence === 'confirmed', `two differing --*-model flags must derive confirmed (CLI-level pinned-by-presence), got ${entries[0].model_independence}`);
});

await check('cells.judge-record refuses (non-zero exit) a free-prose --file payload, and leaves the ledger untouched', async () => {
  fs.writeFileSync(path.join(root, 'verdict-demo-1-bad.json'), 'looks fine to me', 'utf8');
  const result = await runModuleWorker(BEE_MJS, {
    args: ['cells', 'judge-record', '--id', 'demo-1', '--file', 'verdict-demo-1-bad.json', '--json'],
    cwd: root,
  });
  assert(result.status !== 0, `a free-prose verdict payload must be refused, got exit ${result.status}: stdout=${result.stdout}`);
  // --json routes a thrown error's message to stdout as {"error": "..."} (emitError), not stderr.
  assert(/verdict rejected/i.test(result.stdout), `expected a "verdict rejected" refusal, got stdout=${result.stdout} stderr=${result.stderr}`);
});

// cells-archive-2: cells.archive / cells.unarchive round trip. Uses its OWN
// feature ("demo-archive", distinct from the fixture repo's active "demo"
// feature) with two directly-added, already-terminal cells (one capped, one
// dropped — addCell accepts an explicit status, skipping the claim/verify/cap
// dance this fixture does not need) so the archive precondition ("every cell
// capped or dropped") is met without disturbing the demo-1/demo-2 chain above.
await check('cells.archive / cells.unarchive round trip through the real dispatcher (archive-aware CLI)', async () => {
  const archFeature = 'demo-archive';
  addCell(root, {
    id: 'archv-1',
    feature: archFeature,
    title: 'Archive fixture cell 1 (capped)',
    lane: 'small',
    action: 'Fixture cell for the cells.archive/unarchive round trip.',
    verify: 'node -e "process.exit(0)"',
    status: 'capped',
  });
  addCell(root, {
    id: 'archv-2',
    feature: archFeature,
    title: 'Archive fixture cell 2 (dropped)',
    lane: 'small',
    action: 'Fixture cell for the cells.archive/unarchive round trip.',
    verify: 'node -e "process.exit(0)"',
    status: 'dropped',
  });
  assert(fs.existsSync(path.join(root, '.bee', 'cells', 'archv-1.json')), 'archv-1 should exist in the active dir before archiving');
  assert(fs.existsSync(path.join(root, '.bee', 'cells', 'archv-2.json')), 'archv-2 should exist in the active dir before archiving');

  const archiveResult = await assertExampleOk('cells.archive');
  const archived = JSON.parse(archiveResult.stdout);
  assert(archived.feature === archFeature, `expected feature "${archFeature}", got ${archiveResult.stdout}`);
  assert(
    [...archived.moved].sort().join(',') === 'archv-1,archv-2',
    `expected both fixture cells moved, got ${JSON.stringify(archived.moved)}`,
  );
  assert(archived.counts.capped === 1 && archived.counts.dropped === 1, `expected counts capped=1 dropped=1, got ${JSON.stringify(archived.counts)}`);
  assert(!fs.existsSync(path.join(root, '.bee', 'cells', 'archv-1.json')), 'archv-1 should be moved out of the active dir');
  assert(!fs.existsSync(path.join(root, '.bee', 'cells', 'archv-2.json')), 'archv-2 should be moved out of the active dir');
  assert(fs.existsSync(path.join(root, '.bee', 'cells', 'archive', archFeature, 'archv-1.json')), 'archv-1 should now live under .bee/cells/archive/demo-archive/');
  assert(fs.existsSync(path.join(root, '.bee', 'cells', 'archive', archFeature, 'archv-2.json')), 'archv-2 should now live under .bee/cells/archive/demo-archive/');

  const statusAfterArchive = await runModuleWorker(BEE_MJS, { args: ['status', '--json'], cwd: root });
  assert(statusAfterArchive.status === 0, `status after archive should exit 0, got ${statusAfterArchive.status}: ${statusAfterArchive.stderr}`);
  const statusParsed = JSON.parse(statusAfterArchive.stdout);
  assert(
    statusParsed.cells.archived && statusParsed.cells.archived.capped >= 1 && statusParsed.cells.archived.dropped >= 1,
    `expected an honest archived figure sourced from the summary ledger (no dir scan), got ${JSON.stringify(statusParsed.cells)}`,
  );

  const activeArchiveOnDemo = await runModuleWorker(BEE_MJS, {
    args: ['cells', 'archive', '--feature', 'demo', '--json'],
    cwd: root,
  });
  assert(activeArchiveOnDemo.status !== 0, `archiving the ACTIVE feature ("demo") must be refused, got exit ${activeArchiveOnDemo.status}`);

  const unarchiveResult = await assertExampleOk('cells.unarchive');
  const unarchived = JSON.parse(unarchiveResult.stdout);
  assert(unarchived.feature === archFeature, `expected feature "${archFeature}", got ${unarchiveResult.stdout}`);
  assert(
    [...unarchived.moved].sort().join(',') === 'archv-1,archv-2',
    `expected both fixture cells restored, got ${JSON.stringify(unarchived.moved)}`,
  );
  assert(fs.existsSync(path.join(root, '.bee', 'cells', 'archv-1.json')), 'archv-1 should be restored to the active dir');
  assert(fs.existsSync(path.join(root, '.bee', 'cells', 'archv-2.json')), 'archv-2 should be restored to the active dir');
  assert(!fs.existsSync(path.join(root, '.bee', 'cells', 'archive', archFeature)), 'the now-empty archive/demo-archive dir should be removed after unarchive');
});

await check('reservations.reserve example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('reservations.reserve');
  assert(JSON.parse(result.stdout).ok === true, 'reserve should succeed on a fresh path');
});

await check('reservations.reserve --session example (examples[1]) stamps the reservation with the owning session id (D3)', async () => {
  const result = await assertExampleOk('reservations.reserve', { exampleIndex: 1 });
  const parsed = JSON.parse(result.stdout);
  assert(parsed.ok === true, 'session-owned reserve should succeed on a fresh path');
  assert(parsed.reservation.session === 'sess-fsh7', `expected the reservation to carry session "sess-fsh7", got ${result.stdout}`);
});

await check('reservations.list example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('reservations.list');
  assert(result.stdout.includes('worker-a'), `expected the reservation just made, got ${result.stdout}`);
});

await check('reservations.release example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('reservations.release');
  assert(JSON.parse(result.stdout).released >= 1, 'release should free at least the one reservation just made');
});

await check('reservations.sweep example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('reservations.sweep');
  assert(typeof JSON.parse(result.stdout).released === 'number', 'sweep should report a released count');
});

await check('decisions.log example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('decisions.log');
  assert(typeof JSON.parse(result.stdout).id === 'string', 'log should return the new decision id');
});

await check('decisions.active example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('decisions.active');
  assert(JSON.parse(result.stdout).decisions.length >= 1, 'the decision just logged should be active');
});

await check('decisions.search example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('decisions.search');
  assert(JSON.parse(result.stdout).decisions.length >= 1, 'search for "registry" should match the decision just logged');
});

await check('decisions.supersede example runs through the real dispatcher (arbitrary id — event-sourced, no existence check)', async () => {
  const result = await assertExampleOk('decisions.supersede');
  assert(typeof JSON.parse(result.stdout).id === 'string', 'supersede should return the new event id');
});

await check('decisions.redact example runs through the real dispatcher (arbitrary id — event-sourced, no existence check)', async () => {
  const result = await assertExampleOk('decisions.redact');
  assert(typeof JSON.parse(result.stdout).id === 'string', 'redact should return the new event id');
});

await check('decisions.archive example runs through the real dispatcher (decision-propagation dp-3) — the decide event logged above is strictly older than the far-future --before cutoff, so it always has something to archive', async () => {
  const result = await assertExampleOk('decisions.archive');
  const payload = JSON.parse(result.stdout);
  assert(Array.isArray(payload.archived) && payload.archived.length >= 1, `archive should report at least 1 archived event, got ${result.stdout}`);
  assert(fs.existsSync(path.join(root, '.bee', 'decisions-archive.jsonl')), 'archive should create .bee/decisions-archive.jsonl');
});

await check('decisions.tag example runs through the real dispatcher (decision-propagation dp-5) — resolves the pre-seeded fixture target even after archive/supersede/redact touched it', async () => {
  // The fixture target (id 00000000-...) was ALSO the placeholder id for the
  // decisions.supersede/redact examples above, so by this point it is both
  // superseded and redacted — correctly excluded from active/search output
  // regardless of any tag overlay (that exclusion is upstream of the
  // overlay, unrelated to this example). This check only proves the
  // registry's own example round-trips end-to-end through the dispatcher;
  // the overlay-visible-in-search behavior is covered exhaustively by
  // test_decisions_propagation.mjs's dp-5 section against a non-redacted
  // target.
  const result = await assertExampleOk('decisions.tag');
  const event = JSON.parse(result.stdout);
  assert(event.type === 'tag', `expected a tag event, got ${result.stdout}`);
  assert(event.target === '00000000-0000-0000-0000-000000000000', `expected the fixture target id, got ${event.target}`);
  assert(event.tags.join(',') === 'billing,recall', `expected tags billing,recall, got ${JSON.stringify(event.tags)}`);
  assert(event.scope === 'billing', `expected scope billing, got ${event.scope}`);
});

await check('decisions.render example runs through the real dispatcher (decision-propagation dp-4) — writes docs/decisions/index.md from whatever the fixture chain above has logged so far', async () => {
  const result = await assertExampleOk('decisions.render');
  const payload = JSON.parse(result.stdout);
  assert(typeof payload.path === 'string' && payload.path.length > 0, `render should report a path, got ${result.stdout}`);
  assert(typeof payload.count === 'number', `render should report a numeric count, got ${result.stdout}`);
  assert(fs.existsSync(path.join(root, 'docs', 'decisions', 'index.md')), 'render should write docs/decisions/index.md');
});

await check('status example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('status');
  assert(JSON.parse(result.stdout).phase === 'swarming', 'status should reflect the fixture repo\'s phase');
});

// ─── lpsp-2 (P2): default `lanes` summarizes (active lane in full + counts +
// ids for the rest); `--lanes-full` restores today's full per-lane array.
// USER-REPORTED: the `lanes` block alone was 58% of a full `status --json`
// payload on this repo, paid on every session start (AGENTS.md step 3). The
// HARD CONSTRAINT under test: every OTHER top-level field a router needs
// (phase/mode/feature/gates/cells/recommended_next) must be byte-identical
// whether or not --lanes-full is passed — only `lanes` itself may change
// shape/size. A dedicated fixture repo (not the shared `root`/`rootState`)
// keeps this independent of every other example's mutations.

const rootLanes = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-status-lanes-cli-'));
fs.mkdirSync(path.join(rootLanes, '.bee'), { recursive: true });
writeJsonAtomic(path.join(rootLanes, '.bee', 'onboarding.json'), {
  schema_version: '1.0',
  bee_version: '0.1.0',
});
writeState(rootLanes, {
  ...defaultState(),
  phase: 'swarming',
  feature: 'demo-lanes',
  approved_gates: { context: true, shape: true, execution: true, review: false },
});
// Two lane records: "lane-active" is bound to the sole live session in this
// checkout (so buildStatus can identify it as THE active lane, same
// resolveSessionId root-inference path claims/reservations already use with
// no --session-id/env identity supplied); "lane-other" stays unbound — it is
// exactly the kind of historical record the default payload must summarize,
// not carry in full.
writeLane(rootLanes, {
  schema_version: '1.0',
  feature: 'lane-active',
  mode: 'standard',
  phase: 'swarming',
  approved_gates: { context: true, shape: true, execution: true, review: false },
  summary: 'the lane this session is actually working',
  next_action: 'keep swarming',
  created_at: new Date().toISOString(),
});
writeLane(rootLanes, {
  schema_version: '1.0',
  feature: 'lane-other',
  mode: 'standard',
  phase: 'exploring',
  approved_gates: { context: true, shape: false, execution: false, review: false },
  summary: 'a DIFFERENT lane — full record must NOT leak into the default payload',
  next_action: 'lock decisions',
  created_at: new Date().toISOString(),
});
// runModuleWorker inherits process.env by default (see the BEEHIVE_PERF_DIR
// comment above) — when THIS test process is itself running inside a bee
// session (BEE_SESSION_ID/CLAUDE_CODE_SESSION_ID set, e.g. under bee-swarming),
// that ambient id reaches the spawned `bee status` call too and wins over
// root-inference in resolveSessionId's own precedence chain. The fixture
// session must therefore be created under WHATEVER id resolveSessionId will
// actually resolve for that spawned call — the real id when ambient, else the
// literal fallback id root-inference then adopts (exactly one fresh session
// on disk) — never a fixed literal that only happens to work standalone.
const laneEffectiveSessionId = process.env.BEE_SESSION_ID?.trim() || process.env.CLAUDE_CODE_SESSION_ID?.trim() || 'sess-lanes-cli';
const laneSession = createSession(rootLanes, { id: laneEffectiveSessionId });
assert(laneSession.ok, `fixture session creation should succeed: ${JSON.stringify(laneSession)}`);
const laneBind = bindSessionLane(rootLanes, laneEffectiveSessionId, 'lane-active');
assert(laneBind.ok, `fixture session->lane bind should succeed: ${JSON.stringify(laneBind)}`);

const ROUTING_FIELDS = ['phase', 'mode', 'feature', 'gates', 'cells', 'recommended_next'];

await check('status --json (default) summarizes lanes: active lane in full, counts+ids for the rest, no historical full records', async () => {
  const result = await runBee(['status', '--json'], rootLanes);
  assert(result.status === 0, `status --json should succeed: ${result.stderr}`);
  const payload = JSON.parse(result.stdout);
  assert(!Array.isArray(payload.lanes), `default lanes must NOT be the old full-array shape, got ${JSON.stringify(payload.lanes)}`);
  assert(payload.lanes && typeof payload.lanes === 'object', `default lanes must be a summary object, got ${JSON.stringify(payload.lanes)}`);
  assert(payload.lanes.active && payload.lanes.active.feature === 'lane-active', `default lanes.active should be the session-bound lane in full, got ${JSON.stringify(payload.lanes.active)}`);
  assert(payload.lanes.active.approved_gates && payload.lanes.active.approved_gates.execution === true, 'active lane record keeps its own full approved_gates');
  assert(payload.lanes.active.summary === 'the lane this session is actually working', 'active lane keeps its full record (summary field) — it is the one thing a session routes on');
  assert(payload.lanes.counts && payload.lanes.counts.exploring === 1, `counts by phase for the rest, got ${JSON.stringify(payload.lanes.counts)}`);
  assert(Array.isArray(payload.lanes.ids) && payload.lanes.ids.includes('lane-other') && !payload.lanes.ids.includes('lane-active'), `ids should name the non-active lane(s) only, got ${JSON.stringify(payload.lanes.ids)}`);
  const stringified = JSON.stringify(payload.lanes);
  assert(!stringified.includes('lock decisions'), `default payload must not carry lane-other's full record (its next_action leaked), got ${stringified}`);
  assert(!stringified.includes('DIFFERENT lane'), `default payload must not carry lane-other's full record (its summary text leaked), got ${stringified}`);
});

await check('status --lanes-full --json restores today\'s full per-lane array, byte-unchanged shape', async () => {
  const result = await runBee(['status', '--lanes-full', '--json'], rootLanes);
  assert(result.status === 0, `status --lanes-full --json should succeed: ${result.stderr}`);
  const payload = JSON.parse(result.stdout);
  assert(Array.isArray(payload.lanes) && payload.lanes.length === 2, `--lanes-full should restore the full array of both lane records, got ${JSON.stringify(payload.lanes)}`);
  const other = payload.lanes.find((l) => l.feature === 'lane-other');
  assert(other && other.summary === 'a DIFFERENT lane — full record must NOT leak into the default payload', `--lanes-full row must carry its full summary text, got ${JSON.stringify(other)}`);
  assert(Array.isArray(other.bound_sessions) && other.bound_sessions.length === 0, 'lane-other has no bound session');
  const active = payload.lanes.find((l) => l.feature === 'lane-active');
  assert(Array.isArray(active.bound_sessions) && active.bound_sessions.includes(laneEffectiveSessionId), `--lanes-full row still carries bound_sessions exactly as today, got ${JSON.stringify(active)}`);
});

await check('a router reading phase/mode/feature/gates/cells/recommended_next sees byte-identical values with or without --lanes-full', async () => {
  const withoutFlag = JSON.parse((await runBee(['status', '--json'], rootLanes)).stdout);
  const withFlag = JSON.parse((await runBee(['status', '--lanes-full', '--json'], rootLanes)).stdout);
  for (const field of ROUTING_FIELDS) {
    assert(
      JSON.stringify(withoutFlag[field]) === JSON.stringify(withFlag[field]),
      `field "${field}" must be byte-identical regardless of --lanes-full — default=${JSON.stringify(withoutFlag[field])} vs --lanes-full=${JSON.stringify(withFlag[field])}`,
    );
  }
});

await check('status --lanes-full is a registered flag with a runnable example', async () => {
  const entry = entryByName('status');
  assert(entry.parameters.properties['lanes-full'], 'registry entry "status" must register a --lanes-full property');
  assert(entry.parameters.properties['lanes-full'].type === 'boolean', '--lanes-full must be typed boolean');
  const idx = entry.examples.findIndex((ex) => ex.includes('--lanes-full'));
  assert(idx !== -1, `status registry entry must carry a runnable --lanes-full example, got ${JSON.stringify(entry.examples)}`);
  const result = await assertExampleOk('status', { exampleIndex: idx, cwd: rootLanes });
  assert(Array.isArray(JSON.parse(result.stdout).lanes), `the registered --lanes-full example should actually restore the full array, got ${result.stdout}`);
});

// ─── state.* examples: run in a dedicated fresh repo (dispatcher-unify du-1) ─
// State verbs mutate .bee/state.json, so they get their own isolated repo,
// never the demo-1 fixture chain. Order matters: start-feature requires a
// clean idle workspace, so it runs first, before any other state mutation.

const rootState = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-state-example-'));
fs.mkdirSync(path.join(rootState, '.bee'), { recursive: true });
writeJsonAtomic(path.join(rootState, '.bee', 'onboarding.json'), {
  schema_version: '1.0',
  bee_version: '0.1.0',
});

await check('state.start-feature example runs through the real dispatcher (clean idle repo)', async () => {
  const result = await assertExampleOk('state.start-feature', { cwd: rootState });
  assert(JSON.parse(result.stdout).feature === 'newf', `expected feature newf, got ${result.stdout}`);
});

await check('state.set example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('state.set', { cwd: rootState });
  assert(JSON.parse(result.stdout).phase === 'planning', `expected phase planning, got ${result.stdout}`);
});

await check('state.gate example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('state.gate', { cwd: rootState });
  assert(JSON.parse(result.stdout).approved_gates.execution === true, `expected execution approved, got ${result.stdout}`);
});

// explicit-triage D1/D2 (cell et-1): the registry-completeness invariant
// ("every registry entry had its example executed at least once") demands at
// least this much for the new verb — deeper coverage (each enum refusal, the
// empty---flags round-trip, --show, the D3 claim warning, the preamble line)
// is et-3's own cell.
await check('state.route example runs through the real dispatcher (registry-completeness — deeper coverage is et-3\'s own cell)', async () => {
  const result = await assertExampleOk('state.route', { cwd: rootState });
  const route = JSON.parse(result.stdout);
  assert(
    route.class === 'feature' &&
      route.lane === 'standard' &&
      Array.isArray(route.flags) &&
      route.flags.includes('multi-domain') &&
      route.product_files === 7,
    `expected the route record to round-trip class/lane/flags/product_files, got ${result.stdout}`,
  );
});

// ─── explicit-triage et-3: deep behavioral net for `state route` (D1-D4) ───
// Assertions a-e per docs/history/explicit-triage/CONTEXT.md and cell et-3.
// Every fixture below is its own hermetic temp repo built through the REAL
// dispatcher (start-feature + gate), never the shared rootState chain above
// and never live .bee/ — et-1's registry-example check (just above) already
// covers the shallow "the example runs" case; this covers the enum
// refusals, the claim-warning D3 toggle, the preamble D2 line, and the D4
// re-lane rewrite.

async function routeFixtureRepo(feature) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-route-net-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  const started = await runBee(['state', 'start-feature', '--feature', feature, '--json'], dir);
  assert(started.status === 0, `fixture start-feature(${feature}) failed: ${started.stdout}${started.stderr}`);
  const gated = await runBee(['state', 'gate', '--name', 'execution', '--approved', 'true', '--json'], dir);
  assert(gated.status === 0, `fixture gate(${feature}) failed: ${gated.stdout}${gated.stderr}`);
  return dir;
}

function workflowRouteFor(dir, feature) {
  const wf = listWorkflows(dir).workflows.find((w) => w.feature === feature && w.status !== 'closed');
  assert(wf, `expected a live workflow record for feature "${feature}"`);
  return wf.route ?? null;
}

// (b) bad class, bad lane, bad flag, negative files each typed-refused with
// nothing written — each against its OWN never-set feature, so "nothing
// written" is unambiguous (a stale prior value could never masquerade as a
// refused write).

await check('state route --set: bad --class is typed-refused, nothing written (D1)', async () => {
  const dir = await routeFixtureRepo('rt-refuse-class');
  const refused = await runBee(
    ['state', 'route', '--set', '--class', 'bogus', '--lane', 'standard', '--flags', 'multi-domain', '--files', '3'],
    dir,
  );
  assert(refused.status !== 0, `expected non-zero exit, got ${refused.status}: ${refused.stdout}${refused.stderr}`);
  assert(
    /--class "bogus".*must be one of/.test(refused.stdout + refused.stderr),
    `expected a typed --class refusal naming the legal set, got: ${refused.stdout}${refused.stderr}`,
  );
  const show = await runBee(['state', 'route', '--show'], dir);
  assert(show.stdout.trim() === 'No route recorded.', `expected nothing written, got: ${show.stdout}`);
});

await check('state route --set: bad --lane is typed-refused, nothing written (D1)', async () => {
  const dir = await routeFixtureRepo('rt-refuse-lane');
  const refused = await runBee(
    ['state', 'route', '--set', '--class', 'feature', '--lane', 'bogus', '--flags', 'multi-domain', '--files', '3'],
    dir,
  );
  assert(refused.status !== 0, `expected non-zero exit, got ${refused.status}: ${refused.stdout}${refused.stderr}`);
  assert(
    /--lane "bogus".*must be one of/.test(refused.stdout + refused.stderr),
    `expected a typed --lane refusal naming the legal set, got: ${refused.stdout}${refused.stderr}`,
  );
  const show = await runBee(['state', 'route', '--show'], dir);
  assert(show.stdout.trim() === 'No route recorded.', `expected nothing written, got: ${show.stdout}`);
});

await check('state route --set: bad --flags entry is typed-refused, nothing written (D1)', async () => {
  const dir = await routeFixtureRepo('rt-refuse-flags');
  const refused = await runBee(
    ['state', 'route', '--set', '--class', 'feature', '--lane', 'standard', '--flags', 'not-a-real-flag', '--files', '3'],
    dir,
  );
  assert(refused.status !== 0, `expected non-zero exit, got ${refused.status}: ${refused.stdout}${refused.stderr}`);
  assert(
    /invalid flag\(s\) not-a-real-flag/.test(refused.stdout + refused.stderr),
    `expected a typed --flags refusal naming the bad flag, got: ${refused.stdout}${refused.stderr}`,
  );
  const show = await runBee(['state', 'route', '--show'], dir);
  assert(show.stdout.trim() === 'No route recorded.', `expected nothing written, got: ${show.stdout}`);
});

await check('state route --set: negative --files is typed-refused, nothing written (D1)', async () => {
  const dir = await routeFixtureRepo('rt-refuse-files');
  const refused = await runBee(
    ['state', 'route', '--set', '--class', 'feature', '--lane', 'standard', '--flags', 'multi-domain', '--files', '-1'],
    dir,
  );
  assert(refused.status !== 0, `expected non-zero exit, got ${refused.status}: ${refused.stdout}${refused.stderr}`);
  assert(
    /--files "-1".*non-negative integer/.test(refused.stdout + refused.stderr),
    `expected a typed --files refusal, got: ${refused.stdout}${refused.stderr}`,
  );
  const show = await runBee(['state', 'route', '--show'], dir);
  assert(show.stdout.trim() === 'No route recorded.', `expected nothing written, got: ${show.stdout}`);
});

// (a) valid --set round-trips on the active feature's workflow record, via
// --show, and via status --json (D1/D2) — plus (d) the preamble's Route
// line, absent before any --set and present in the exact D2 format after —
// and (e) a second --set (re-lane demotion) rewriting the SAME record.
// Sequenced in ONE fixture repo on purpose: (e) can only prove "the SAME
// record, not a second one" by observing the record this exact test built.

const rtRoot = await routeFixtureRepo('rt-full-cycle');

await check('state route: preamble carries no Route line before any route is recorded (D2 zero-cost-when-absent)', async () => {
  const preamble = buildSessionPreamble(rtRoot);
  assert(!preamble.includes('- Route:'), `expected no Route line before a route is recorded, got:\n${preamble}`);
});

let firstRoute = null;
await check('state route --set: valid input round-trips via --show, status --json, and the underlying workflow record (D1)', async () => {
  const setResult = await runBee(
    [
      'state', 'route', '--set',
      '--class', 'feature',
      '--lane', 'standard',
      '--flags', 'multi-domain,data-model',
      '--files', '3',
      '--rationale', 'et-3 fixture',
      '--json',
    ],
    rtRoot,
  );
  assert(setResult.status === 0, `--set should succeed, got ${setResult.status}: ${setResult.stdout}${setResult.stderr}`);
  firstRoute = JSON.parse(setResult.stdout);
  assert(
    firstRoute.class === 'feature' &&
      firstRoute.lane === 'standard' &&
      firstRoute.flags.join(',') === 'multi-domain,data-model' &&
      firstRoute.product_files === 3,
    `expected the set result to carry the recorded fields, got ${setResult.stdout}`,
  );

  const showResult = await runBee(['state', 'route', '--show', '--json'], rtRoot);
  assert(showResult.status === 0, `--show should succeed, got ${showResult.status}: ${showResult.stdout}${showResult.stderr}`);
  assert(
    JSON.parse(showResult.stdout).updated_at === firstRoute.updated_at,
    `--show should return the SAME record --set just wrote, got ${showResult.stdout}`,
  );

  const statusResult = await runBee(['status', '--json'], rtRoot);
  assert(statusResult.status === 0, `status --json should succeed, got ${statusResult.status}: ${statusResult.stderr}`);
  const statusRoute = JSON.parse(statusResult.stdout).route;
  assert(
    statusRoute && statusRoute.class === 'feature' && statusRoute.lane === 'standard' && statusRoute.product_files === 3,
    `expected status --json to carry the route block, got ${statusResult.stdout}`,
  );

  const wfRoute = workflowRouteFor(rtRoot, 'rt-full-cycle');
  assert(
    wfRoute && wfRoute.updated_at === firstRoute.updated_at && wfRoute.lane === 'standard',
    `expected the underlying workflow record to carry the SAME route (belt-and-suspenders, D1), got ${JSON.stringify(wfRoute)}`,
  );
});

await check('state route: preamble carries the exact "- Route: ..." line once a route is recorded (D2)', async () => {
  const preamble = buildSessionPreamble(rtRoot);
  assert(
    preamble.includes('- Route: class=feature | lane=standard | flags=2 [multi-domain,data-model] | files=3'),
    `expected the exact D2-formatted Route line, got:\n${preamble}`,
  );
});

await check('state route --set: a second --set (re-lane demotion) rewrites the SAME record — lane changes, one record, updated_at moves (D4)', async () => {
  await new Promise((resolve) => setTimeout(resolve, 5)); // guarantee updated_at actually moves
  const setResult = await runBee(
    ['state', 'route', '--set', '--class', 'feature', '--lane', 'tiny', '--flags', '', '--files', '0', '--json'],
    rtRoot,
  );
  assert(setResult.status === 0, `re-lane --set should succeed, got ${setResult.status}: ${setResult.stdout}${setResult.stderr}`);
  const secondRoute = JSON.parse(setResult.stdout);
  assert(
    secondRoute.lane === 'tiny' && secondRoute.flags.length === 0 && secondRoute.product_files === 0,
    `expected the demoted lane/flags/files, got ${setResult.stdout}`,
  );
  assert(secondRoute.updated_at !== firstRoute.updated_at, `updated_at must move on re-lane, still ${secondRoute.updated_at}`);

  const showResult = await runBee(['state', 'route', '--show', '--json'], rtRoot);
  const shown = JSON.parse(showResult.stdout);
  assert(
    shown.lane === 'tiny' && shown.updated_at === secondRoute.updated_at,
    `--show must reflect the rewritten record, got ${showResult.stdout}`,
  );

  const wfRoute = workflowRouteFor(rtRoot, 'rt-full-cycle');
  assert(
    wfRoute && wfRoute.lane === 'tiny' && wfRoute.updated_at === secondRoute.updated_at,
    `expected the SAME workflow record rewritten in place (one record, not a second), got ${JSON.stringify(wfRoute)}`,
  );

  const preamble = buildSessionPreamble(rtRoot);
  assert(
    preamble.includes('- Route: class=feature | lane=tiny | flags=0 [] | files=0'),
    `expected the preamble Route line to reflect the rewritten record, got:\n${preamble}`,
  );
});

// (c) `cells claim` (D3): ONE stderr warning when the claimed cell's feature
// has no route yet, and never a refusal; silent once a route is recorded.

await check('cells claim: warns ONCE on stderr when the claimed cell\'s feature has no route record (D3, soft enforcement, never a refusal)', async () => {
  const dir = await routeFixtureRepo('rt-warn-none');
  addCell(dir, {
    id: 'rt-warn-none-1',
    feature: 'rt-warn-none',
    title: 'et-3 fixture cell (no route)',
    lane: 'small',
    action: 'Exercise the D3 claim-warning behavior with no route recorded.',
    verify: 'node -e "process.exit(0)"',
  });
  const claimed = await runBee(['cells', 'claim', '--id', 'rt-warn-none-1', '--worker', 'et3-fixture', '--json'], dir);
  assert(claimed.status === 0, `claim should still succeed (D3 never refuses), got ${claimed.status}: ${claimed.stdout}${claimed.stderr}`);
  assert(JSON.parse(claimed.stdout).status === 'claimed', `expected the cell claimed, got ${claimed.stdout}`);
  const warnings = claimed.stderr.split('\n').filter((line) => line.startsWith('WARNING: cell "rt-warn-none-1"'));
  assert(warnings.length === 1, `expected exactly ONE D3 warning line, got ${warnings.length}: ${claimed.stderr}`);
  assert(/no route record/.test(warnings[0]), `expected the warning to name the missing route, got: ${warnings[0]}`);
});

await check('cells claim: silent (no D3 warning) when the claimed cell\'s feature already has a recorded route', async () => {
  const dir = await routeFixtureRepo('rt-warn-quiet');
  const setResult = await runBee(
    ['state', 'route', '--set', '--class', 'feature', '--lane', 'standard', '--flags', '', '--files', '1', '--json'],
    dir,
  );
  assert(setResult.status === 0, `fixture route --set should succeed, got ${setResult.status}: ${setResult.stdout}${setResult.stderr}`);
  addCell(dir, {
    id: 'rt-warn-quiet-1',
    feature: 'rt-warn-quiet',
    title: 'et-3 fixture cell (routed)',
    lane: 'small',
    action: 'Exercise the D3 claim-warning behavior with a route already recorded.',
    verify: 'node -e "process.exit(0)"',
  });
  const claimed = await runBee(['cells', 'claim', '--id', 'rt-warn-quiet-1', '--worker', 'et3-fixture', '--json'], dir);
  assert(claimed.status === 0, `claim should succeed, got ${claimed.status}: ${claimed.stdout}${claimed.stderr}`);
  assert(JSON.parse(claimed.stdout).status === 'claimed', `expected the cell claimed, got ${claimed.stdout}`);
  assert(!/WARNING: cell/.test(claimed.stderr), `expected NO D3 warning when a route is recorded, got: ${claimed.stderr}`);
});

await check('state.worker.add example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('state.worker.add', { cwd: rootState });
  assert(JSON.parse(result.stdout).workers.some((w) => w.nickname === 'w1'), `expected worker w1, got ${result.stdout}`);
});

await check('state.worker.update example runs through the real dispatcher (w1 added above)', async () => {
  const result = await assertExampleOk('state.worker.update', { cwd: rootState });
  assert(JSON.parse(result.stdout).workers.find((w) => w.nickname === 'w1').status === 'done', `expected w1 status done, got ${result.stdout}`);
});

await check('state.worker.remove example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('state.worker.remove', { cwd: rootState });
  assert(!JSON.parse(result.stdout).workers.some((w) => w.nickname === 'w1'), `expected w1 removed, got ${result.stdout}`);
});

await check('state.worker.clear example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('state.worker.clear', { cwd: rootState });
  assert(JSON.parse(result.stdout).workers.length === 0, `expected empty workers, got ${result.stdout}`);
});

await check('state.worker.prune example runs through the real dispatcher (no workers dir -> 0 pruned)', async () => {
  const result = await assertExampleOk('state.worker.prune', { cwd: rootState });
  assert(JSON.parse(result.stdout).pruned.length === 0, `expected 0 pruned, got ${result.stdout}`);
});

await check('state.scribing-run example runs through the real dispatcher (from an executed phase — chain-integrity D3)', async () => {
  // The shared rootState sits at `planning` from the state.set example above.
  // scribing-run used to advance to `compounding` from ANY phase; it now demands
  // a phase where execution actually happened. Walking the legal path first is
  // the point, not a workaround: this check now also proves swarming ->
  // scribing-run -> compounding runs end to end through the real dispatcher.
  const advance = await runBee(['state', 'set', '--owner', 'planning', '--phase', 'swarming', '--json'], rootState);
  assert(advance.status === 0, `advancing to swarming should succeed: ${advance.stderr}`);
  const result = await assertExampleOk('state.scribing-run', { cwd: rootState });
  assert(JSON.parse(result.stdout).phase === 'compounding', `expected phase compounding, got ${result.stdout}`);
});

// compounding-gate D1 (cell cg-1): the registry-completeness invariant
// ("every registry entry had its example executed at least once") demands at
// least this much for the new verb — deeper coverage (wrong-phase refusal,
// --waive-compounding audit logging, freshness edge cases) is cg-2's own cell.
await check('state.compounding-run example runs through the real dispatcher — does NOT advance phase (compounding-gate D1)', async () => {
  // rootState is now `compounding` (feature "newf") after the scribing-run
  // example above — the ONLY phase compounding-run is legal from.
  const result = await assertExampleOk('state.compounding-run', { cwd: rootState });
  const after = JSON.parse(result.stdout);
  assert(after.phase === 'compounding', `compounding-run must not advance phase, got ${result.stdout}`);
  assert(
    after.last_compounding_run && after.last_compounding_run.feature === 'newf',
    `expected last_compounding_run stamped for "newf", got ${result.stdout}`,
  );
});

await check('state.scribing-run is REFUSED from a phase where nothing was executed (chain-integrity D3)', async () => {
  const refused = await runBee(
    ['state', 'scribing-run', '--feature', 'newf', '--areas', 'x', '--next-action', 'n', '--json'],
    rootState,
  );
  // rootState is now `compounding` — not an executed phase.
  assert(refused.status !== 0, `scribing-run from compounding should be refused, got ${refused.stdout}`);
  // --json routes the failure to stdout as {"error": ...}; bare runs use stderr.
  assert(
    /scribing-run: refused from phase/.test(refused.stdout + refused.stderr),
    `expected the D3 refusal, got: ${refused.stdout}${refused.stderr}`,
  );
});

await check('state set --phase compounding-complete is REFUSED from swarming — the exact post-mortem call (chain-integrity D1-REVISED)', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-tail-guard-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  try {
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), { phase: 'swarming' });
    const refused = await runBee(['state', 'set', '--owner', 'swarming', '--phase', 'compounding-complete', '--json'], dir);
    assert(refused.status !== 0, 'swarming -> compounding-complete must be refused');
    // --json routes the failure to stdout as {"error": ...}; bare runs use stderr.
    assert(
      /may only be entered from/.test(refused.stdout + refused.stderr),
      `expected the tail-guard refusal, got: ${refused.stdout}${refused.stderr}`,
    );
    assert(
      JSON.parse(fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8')).phase === 'swarming',
      'a refused close must leave the phase untouched — no partial write',
    );

    // `compounding` is never settable directly: only a real scribing run yields it.
    const direct = await runBee(['state', 'set', '--owner', 'swarming', '--phase', 'compounding', '--json'], dir);
    assert(direct.status !== 0, '--phase compounding must be refused outright');
    assert(
      /scribing-run/.test(direct.stdout + direct.stderr),
      `the refusal must name scribing-run as the way, got: ${direct.stdout}${direct.stderr}`,
    );

    // Backward moves and the de-facto abandon verb stay legal (hive law 5).
    assert((await runBee(['state', 'set', '--owner', 'swarming', '--phase', 'planning', '--json'], dir)).status === 0, 'backward move must stay legal');
    assert((await runBee(['state', 'set', '--owner', 'planning', '--phase', 'idle', '--json'], dir)).status === 0, '--phase idle must stay legal');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── state.lanes / state.set|gate|scribing-run --lane / state.session.* :
// fresh-session-handoff fsh-4 (D2/D4) CLI surface over fsh-3's lane store +
// session→lane binding. Lane records live at .bee/lanes/<feature>.json,
// entirely separate from rootState's default state.json above, so these
// checks can run in any order relative to the default-pipeline checks
// above/below without disturbing either.

await check('state.start-feature --as-lane example (examples[1]) starts a lane record beside the untouched default state.json', async () => {
  const beforeDefault = fs.readFileSync(path.join(rootState, '.bee', 'state.json'), 'utf8');
  const result = await assertExampleOk('state.start-feature', { exampleIndex: 1, cwd: rootState });
  const lane = JSON.parse(result.stdout);
  assert(lane.feature === 'demo-lane', `expected lane feature demo-lane, got ${result.stdout}`);
  assert(lane.approved_gates.execution === false, `expected a fresh lane's gates all reset, got ${result.stdout}`);
  assert(fs.existsSync(path.join(rootState, '.bee', 'lanes', 'demo-lane.json')), 'lane file should now exist');
  const afterDefault = fs.readFileSync(path.join(rootState, '.bee', 'state.json'), 'utf8');
  assert(beforeDefault === afterDefault, 'default state.json must stay byte-untouched by a lane-mode start (D4)');
});

await check('state.lanes example lists the demo-lane record just started', async () => {
  const result = await assertExampleOk('state.lanes', { cwd: rootState });
  const lanes = JSON.parse(result.stdout);
  assert(Array.isArray(lanes) && lanes.some((l) => l.feature === 'demo-lane'), `expected demo-lane in lanes list, got ${result.stdout}`);
});

await check('state.set --lane example (examples[1]) routes the mutation to the lane record, not state.json', async () => {
  const beforeDefault = fs.readFileSync(path.join(rootState, '.bee', 'state.json'), 'utf8');
  const result = await assertExampleOk('state.set', { exampleIndex: 1, cwd: rootState });
  const lane = JSON.parse(result.stdout);
  assert(lane.feature === 'demo-lane' && lane.phase === 'planning', `expected lane phase planning, got ${result.stdout}`);
  const afterDefault = fs.readFileSync(path.join(rootState, '.bee', 'state.json'), 'utf8');
  assert(beforeDefault === afterDefault, 'default state.json must stay byte-untouched by a --lane routed set');
});

await check('state.gate --lane example (examples[1]) approves a gate on the lane record only', async () => {
  const result = await assertExampleOk('state.gate', { exampleIndex: 1, cwd: rootState });
  const lane = JSON.parse(result.stdout);
  assert(lane.feature === 'demo-lane' && lane.approved_gates.execution === true, `expected lane execution gate approved, got ${result.stdout}`);
});

// state.plan-rev.bump (multisession-native-9, D7/C2): exercised against its
// OWN isolated fixture — never rootState's shared demo-lane chain above,
// since bumping plan_rev deliberately flips demo-lane's projected execution
// boolean false, which every later rootState check (scribing-run, rebuild-
// projections, session bind, ...) assumes stays true. The full C2 proof
// (claim refusal + cross-workflow isolation, invariant 3) lives in
// test_cli_state.mjs, right beside msn-7's own lane-gate test.
await check('state.plan-rev.bump example (multisession-native-9) bumps a freshly-started lane workflow\'s plan_rev by 1, rebuilding its projection', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-plan-rev-bump-example-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  writeState(dir, defaultState());
  const started = await runBee(['state', 'start-feature', '--feature', 'demo-lane', '--as-lane', '--json'], dir);
  assert(started.status === 0, `start-feature --as-lane should succeed: ${started.stderr}`);
  const result = await assertExampleOk('state.plan-rev.bump', { cwd: dir });
  const out = JSON.parse(result.stdout);
  assert(out.feature === 'demo-lane' && out.plan_rev === 1, `expected demo-lane's plan_rev bumped to 1, got ${result.stdout}`);
});

await check('state.scribing-run --lane example (examples[1]) stamps the lane record only', async () => {
  // Same D3 rule on the lane record: the tail guard reads `from` off whichever
  // record is being mutated, so the lane must reach an executed phase too.
  const advance = await runBee(['state', 'set', '--lane', 'demo-lane', '--owner', 'planning', '--phase', 'swarming', '--json'], rootState);
  assert(advance.status === 0, `advancing the lane to swarming should succeed: ${advance.stderr}`);
  const result = await assertExampleOk('state.scribing-run', { exampleIndex: 1, cwd: rootState });
  const lane = JSON.parse(result.stdout);
  assert(
    lane.feature === 'demo-lane' && lane.phase === 'compounding' && lane.last_scribing_run.feature === 'demo-lane',
    `expected lane scribing stamp, got ${result.stdout}`,
  );
});

await check('state.rebuild-projections example (multisession-native-7/10) rebuilds demo-lane\'s projection AND state.json\'s, both from their own live workflow record — rootState\'s default feature ("newf") has been kept in sync at every default-path mutation since msn-10, so its rebuild is now authoritative even though it is live and non-idle', async () => {
  const beforeDefault = fs.readFileSync(path.join(rootState, '.bee', 'state.json'), 'utf8');
  const beforeLane = fs.readFileSync(path.join(rootState, '.bee', 'lanes', 'demo-lane.json'), 'utf8');
  const wfNewf = listWorkflows(rootState).workflows.find((wf) => wf.feature === 'newf');
  assert(wfNewf, 'precondition: rootState\'s default feature has a live workflow record (msn-6)');

  const result = await assertExampleOk('state.rebuild-projections', { cwd: rootState });
  const parsed = JSON.parse(result.stdout);
  assert(
    parsed.state.authoritative === true && parsed.state.source === wfNewf.id,
    `multisession-native-10: state.json's live default feature ("newf") now has its OWN workflow record kept in sync — rebuild must be authoritative and sourced from it, got ${result.stdout}`,
  );
  assert(
    parsed.lanes.some((l) => l.authoritative === true && l.lane && l.lane.feature === 'demo-lane'),
    `expected demo-lane's projection among the rebuilt lanes (lanes are always kept in sync in this cell), got ${result.stdout}`,
  );
  const afterDefault = fs.readFileSync(path.join(rootState, '.bee', 'state.json'), 'utf8');
  const afterLane = fs.readFileSync(path.join(rootState, '.bee', 'lanes', 'demo-lane.json'), 'utf8');
  assert(afterDefault === beforeDefault, 'rebuilding must be idempotent: state.json is already what its workflow record derives (kept in sync at every mutation since msn-10), so bytes are unchanged');
  assert(afterLane === beforeLane, 'rebuilding must be idempotent: demo-lane.json is already what the projection derives, so bytes are unchanged');
});

await check('state.set --lane refuses loudly when the named lane does not exist, no partial write (must-have truth)', async () => {
  const result = await runModuleWorker(BEE_MJS, {
    args: ['state', 'set', '--lane', 'ghost-lane', '--owner', 'exploring', '--phase', 'planning'],
    cwd: rootState,
  });
  assert(result.status !== 0, `expected non-zero exit, got ${result.status}`);
  assert(/ghost-lane/.test(result.stderr) && /does not exist/.test(result.stderr), `expected a named-lane refusal, got stderr=${result.stderr}`);
  assert(!fs.existsSync(path.join(rootState, '.bee', 'lanes', 'ghost-lane.json')), 'no partial lane file should be created on refusal');
});

await check('state.gate --lane refuses loudly over a corrupt lane record, file left byte-untouched (must-have truth)', async () => {
  const corruptPath = path.join(rootState, '.bee', 'lanes', 'corrupt-lane.json');
  fs.writeFileSync(corruptPath, '{ this is not a valid lane record', 'utf8');
  const before = fs.readFileSync(corruptPath, 'utf8');
  const result = await runModuleWorker(BEE_MJS, {
    args: ['state', 'gate', '--lane', 'corrupt-lane', '--name', 'execution', '--approved', 'true'],
    cwd: rootState,
  });
  assert(result.status !== 0, `expected non-zero exit, got ${result.status}`);
  const after = fs.readFileSync(corruptPath, 'utf8');
  assert(before === after, 'corrupt lane file must be byte-identical after the refused mutation');
});

await check('state.set --lane refuses when combined with --feature (a lane\'s identity is not a mutable field)', async () => {
  const result = await runModuleWorker(BEE_MJS, {
    args: ['state', 'set', '--lane', 'demo-lane', '--owner', 'planning', '--feature', 'renamed-lane', '--phase', 'planning'],
    cwd: rootState,
  });
  assert(result.status !== 0, `expected non-zero exit, got ${result.status}`);
  assert(/--feature/.test(result.stderr) && /--lane/.test(result.stderr), `expected a --feature/--lane conflict refusal, got stderr=${result.stderr}`);
});

await check('state.session.list example lists a manually-seeded session record', async () => {
  writeJsonAtomic(path.join(rootState, '.bee', 'sessions', 'sess-demo.json'), {
    id: 'sess-demo',
    started_at: new Date().toISOString(),
    last_heartbeat: new Date().toISOString(),
  });
  const result = await assertExampleOk('state.session.list', { cwd: rootState });
  assert(result.stdout.includes('sess-demo'), `expected sess-demo in session list, got ${result.stdout}`);
});

await check('state.session.bind example binds the seeded session to demo-lane', async () => {
  const result = await assertExampleOk('state.session.bind', { cwd: rootState });
  const session = JSON.parse(result.stdout);
  assert(session.id === 'sess-demo' && session.lane === 'demo-lane', `expected sess-demo bound to demo-lane, got ${result.stdout}`);
});

await check('state.session.unbind example removes the binding (lane key omitted, not null)', async () => {
  const result = await assertExampleOk('state.session.unbind', { cwd: rootState });
  const session = JSON.parse(result.stdout);
  assert(session.id === 'sess-demo' && !('lane' in session), `expected the lane key omitted after unbind, got ${result.stdout}`);
});

// ─── state.handoff.*: fresh-session-handoff fsh-9 (D1) — the guarded two-kind
// handoff lifecycle CLI surface. Uses its own prev/next cell + claim fixtures
// inside rootState so it never disturbs the demo-lane/session rows above.

await check('state.handoff.write --kind pause example (examples[0]) writes a free-form pause handoff', async () => {
  const result = await assertExampleOk('state.handoff.write', { cwd: rootState });
  const record = JSON.parse(result.stdout);
  assert(record.kind === 'pause', `expected a pause handoff, got ${result.stdout}`);
  assert(fs.existsSync(path.join(rootState, '.bee', 'HANDOFF.json')), 'HANDOFF.json should now exist');
});

await check('state.handoff.show example shows the pause handoff just written', async () => {
  const result = await assertExampleOk('state.handoff.show', { cwd: rootState });
  const record = JSON.parse(result.stdout);
  assert(record.kind === 'pause', `expected pause kind on show, got ${result.stdout}`);
});

await check('state.handoff.write --kind planned-next example (examples[1]) succeeds once its cap/claim fixtures are seeded, carries writer_session/previous_cell/next_cell', async () => {
  writeJsonAtomic(path.join(rootState, '.bee', 'cells', 'handoff-prev.json'), {
    id: 'handoff-prev',
    status: 'capped',
    trace: { verify_passed: true },
  });
  writeJsonAtomic(path.join(rootState, '.bee', 'claims', 'handoff-next.json'), {
    cell: 'handoff-next',
    session: 'sess-handoff-writer',
    ttl_seconds: 3600,
    claimed_at: new Date().toISOString(),
  });
  const result = await assertExampleOk('state.handoff.write', { exampleIndex: 1, cwd: rootState });
  const record = JSON.parse(result.stdout);
  assert(
    record.kind === 'planned-next' &&
      record.writer_session === 'sess-handoff-writer' &&
      record.previous_cell === 'handoff-prev' &&
      record.next_cell === 'handoff-next',
    `expected the carried planned-next identifiers, got ${result.stdout}`,
  );
});

await check('state.handoff.write --kind planned-next refuses (typed, non-zero exit) when the previous cell is not capped, no partial file (must-have truth)', async () => {
  const result = await runModuleWorker(BEE_MJS, {
    args: [
      'state',
      'handoff',
      'write',
      '--kind',
      'planned-next',
      '--writer-session',
      'sess-handoff-writer',
      '--previous-cell',
      'ghost-cell',
      '--next-cell',
      'handoff-next',
    ],
    cwd: rootState,
  });
  assert(result.status !== 0, `expected non-zero exit, got ${result.status}`);
  assert(/capped/.test(result.stderr), `expected a capped-precondition refusal, got stderr=${result.stderr}`);
});

await check('state.handoff.adopt example transfers the carried claim and clears the handoff', async () => {
  const result = await assertExampleOk('state.handoff.adopt', { cwd: rootState });
  const parsed = JSON.parse(result.stdout);
  assert(parsed.ok === true, `expected adoption to succeed, got ${result.stdout}`);
  assert(!fs.existsSync(path.join(rootState, '.bee', 'HANDOFF.json')), 'handoff should be cleared after adopt');
  const claim = JSON.parse(fs.readFileSync(path.join(rootState, '.bee', 'claims', 'handoff-next.json'), 'utf8'));
  assert(claim.session === 'sess-handoff-adopter', `expected the claim transferred to the adopting session, got ${JSON.stringify(claim)}`);
});

await check('state.handoff.show reports no handoff (null result) once cleared; the text form (no --json) prints "No handoff."', async () => {
  const result = await assertExampleOk('state.handoff.show', { cwd: rootState });
  assert(JSON.parse(result.stdout) === null, `expected a null result once cleared, got ${result.stdout}`);
  const textResult = await runModuleWorker(BEE_MJS, {
    args: ['state', 'handoff', 'show'],
    cwd: rootState,
  });
  assert(/No handoff\./.test(textResult.stdout), `expected "No handoff." in the text render, got stdout=${textResult.stdout}`);
});

// ─── backlog.* / capture.* examples: run in a dedicated fresh repo
// (dispatcher-unify du-2). Neither group touches .bee/state.json or the
// demo-1/demo-2 cell fixtures, so they get their own isolated repo with a
// docs/backlog.md table and a README.md heading for the badges pass to
// insert under.

const rootBacklogCapture = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-backlog-capture-example-'));
fs.mkdirSync(path.join(rootBacklogCapture, '.bee'), { recursive: true });
writeJsonAtomic(path.join(rootBacklogCapture, '.bee', 'onboarding.json'), {
  schema_version: '1.0',
  bee_version: '0.1.0',
});
fs.mkdirSync(path.join(rootBacklogCapture, 'docs'), { recursive: true });
fs.writeFileSync(
  path.join(rootBacklogCapture, 'docs', 'backlog.md'),
  '# Backlog\n\n| ID | Story | Status |\n|----|-------|--------|\n| 1 | A | done |\n| 2 | B | proposed |\n| 3 | C | in-flight |\n',
  'utf8',
);
fs.writeFileSync(path.join(rootBacklogCapture, 'README.md'), '# Demo repo\n', 'utf8');

await check('backlog.counts example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('backlog.counts', { cwd: rootBacklogCapture });
  const counts = JSON.parse(result.stdout);
  assert(counts.done === 1 && counts.proposed === 1 && counts.inFlight === 1, `expected 1/1/1, got ${result.stdout}`);
});

await check('backlog.rank example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('backlog.rank', { cwd: rootBacklogCapture });
  assert(Array.isArray(JSON.parse(result.stdout).order), `expected an order array, got ${result.stdout}`);
});

await check('backlog.badges example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('backlog.badges', { cwd: rootBacklogCapture });
  assert(typeof JSON.parse(result.stdout).badges === 'string', `expected a badges string, got ${result.stdout}`);
});

await check('backlog.add example runs through the real dispatcher and appends to .bee/backlog.jsonl', async () => {
  const result = await assertExampleOk('backlog.add', { cwd: rootBacklogCapture });
  const row = JSON.parse(result.stdout);
  assert(row.type === 'friction' && row.severity === 'P2', `expected the example row, got ${result.stdout}`);
  assert(fs.existsSync(path.join(rootBacklogCapture, '.bee', 'backlog.jsonl')), 'backlog.jsonl should now exist');
});

await check('backlog.propose example runs through the real dispatcher and appends a proposed PBI to the fold', async () => {
  const result = await assertExampleOk('backlog.propose', { cwd: rootBacklogCapture });
  const row = JSON.parse(result.stdout);
  assert(/^p-[0-9a-f]{8}$/.test(row.id), `expected a generated p-<8hex> id, got ${result.stdout}`);
  assert(row.feature === 'backlog-submit-command', `expected the example's --feature carried through, got ${result.stdout}`);

  // propose writes the event-sourced fold, never the generated docs/backlog.md
  // view — the proposal is visible through "backlog pbi list", and the table
  // only changes when "backlog render --write" regenerates it.
  const listResult = await runModuleWorker(BEE_MJS, {
    args: ['backlog', 'pbi', 'list', '--json'],
    cwd: rootBacklogCapture,
  });
  assert(listResult.status === 0, `pbi list failed: stdout=${listResult.stdout} stderr=${listResult.stderr}`);
  const proposed = JSON.parse(listResult.stdout).find((item) => item.id === row.id);
  assert(proposed, `expected ${row.id} in the fold, got ${listResult.stdout}`);
  assert(proposed.status === 'proposed', `expected status proposed, got ${JSON.stringify(proposed)}`);
  assert(proposed.title === row.story, `expected --story stored as the PBI title, got ${JSON.stringify(proposed)}`);
});

await check('backlog.pbi.add example runs through the real dispatcher and prints a generated id', async () => {
  const result = await assertExampleOk('backlog.pbi.add', { cwd: rootBacklogCapture });
  const item = JSON.parse(result.stdout);
  assert(typeof item.id === 'string' && /^p-[0-9a-f]{8}$/.test(item.id), `expected a generated p-<8hex> id, got ${result.stdout}`);
  assert(item.title === 'Unify the backlog', `expected the example title, got ${result.stdout}`);
});

// Seed the fixed id the pbi.status/pbi.amend/pbi.list examples below hardcode
// (--id p-a1b2c3d4, same documentation-friendly well-known-id convention as
// the decisions.tag fixture near the top of this file) — run directly through
// the dispatcher, not assertExampleOk('backlog.pbi.add', ...) again, which
// would re-run examples[0] and mint a fresh random id instead of this one.
const seedPbiResult = await runModuleWorker(BEE_MJS, {
  args: ['backlog', 'pbi', 'add', '--id', 'p-a1b2c3d4', '--title', 'Seed PBI for the pbi.status/amend/list examples', '--json'],
  cwd: rootBacklogCapture,
});
assert(seedPbiResult.status === 0, `seeding p-a1b2c3d4 failed: stdout=${seedPbiResult.stdout} stderr=${seedPbiResult.stderr}`);

await check('backlog.pbi.status example runs through the real dispatcher and flips status+feature', async () => {
  await assertExampleOk('backlog.pbi.status', { cwd: rootBacklogCapture });
  const listResult = await runModuleWorker(BEE_MJS, {
    args: ['backlog', 'pbi', 'list', '--json'],
    cwd: rootBacklogCapture,
  });
  assert(listResult.status === 0, `pbi list failed: stdout=${listResult.stdout} stderr=${listResult.stderr}`);
  const items = JSON.parse(listResult.stdout);
  const seeded = items.find((item) => item.id === 'p-a1b2c3d4');
  assert(seeded, `expected p-a1b2c3d4 in the fold, got ${listResult.stdout}`);
  assert(seeded.status === 'in-flight', `expected status in-flight, got ${JSON.stringify(seeded)}`);
  assert(seeded.feature === 'backlog-unification', `expected feature backlog-unification, got ${JSON.stringify(seeded)}`);
});

await check('backlog.pbi.amend example runs through the real dispatcher and updates cos', async () => {
  const result = await assertExampleOk('backlog.pbi.amend', { cwd: rootBacklogCapture });
  const item = JSON.parse(result.stdout);
  assert(item.id === 'p-a1b2c3d4' && item.cos === 'revised CoS text', `expected the amended cos, got ${result.stdout}`);
});

await check('backlog.pbi.list example runs through the real dispatcher and filters to in-flight', async () => {
  const result = await assertExampleOk('backlog.pbi.list', { cwd: rootBacklogCapture });
  const items = JSON.parse(result.stdout);
  assert(Array.isArray(items) && items.some((item) => item.id === 'p-a1b2c3d4'), `expected p-a1b2c3d4 in the in-flight filter, got ${result.stdout}`);
  assert(items.every((item) => item.status === 'in-flight'), `expected every row filtered to in-flight, got ${result.stdout}`);
});

await check('backlog.render examples (--write then --check) run through the real dispatcher and land Current', async () => {
  const writeResult = await assertExampleOk('backlog.render', { exampleIndex: 1, cwd: rootBacklogCapture });
  assert(/^(Rendered|Already current): docs\/backlog\.md$/.test(writeResult.stdout.trim()), `expected the write confirmation, got ${writeResult.stdout}`);
  assert(
    fs.readFileSync(path.join(rootBacklogCapture, 'docs', 'backlog.md'), 'utf8').includes('p-a1b2c3d4'),
    'docs/backlog.md should now list the seeded PBI',
  );

  const checkResult = await assertExampleOk('backlog.render', { exampleIndex: 0, cwd: rootBacklogCapture });
  assert(checkResult.stdout.trim() === 'Current: docs/backlog.md', `expected --check to report Current after the write, got ${checkResult.stdout}`);
});

// sqs-b2-fix: backlog.findings' own registry example targets --feature
// state-query-surface, a feature slug that has no natural row on this
// fixture's backlog.jsonl otherwise (backlog.pbi.status's own
// "backlog-unification" feature example is a kind:'pbi' row, which
// isBacklogFindingRow always excludes) — so seed one matching friction row
// directly, mirroring capture.flush's seed-then-assertExampleOk pattern above.
await check('backlog.findings example runs through the real dispatcher and returns the seeded friction row for the feature', async () => {
  fs.appendFileSync(
    path.join(rootBacklogCapture, '.bee', 'backlog.jsonl'),
    `${JSON.stringify({ type: 'friction', severity: 'P2', title: 'grep is slow over the seeded fixture repo', detail: 'ripgrep search took too long', feature: 'state-query-surface' })}\n`,
  );
  const result = await assertExampleOk('backlog.findings', { cwd: rootBacklogCapture });
  const { findings } = JSON.parse(result.stdout);
  assert(Array.isArray(findings) && findings.length === 1, `expected exactly the one seeded finding, got ${result.stdout}`);
  assert(findings[0].title === 'grep is slow over the seeded fixture repo', `expected the seeded row's title, got ${result.stdout}`);

  const { entry, result: textResult } = await runExample('backlog.findings', { exampleIndex: 1, cwd: rootBacklogCapture });
  assert(textResult.status === 0, `${entry.name} example "${entry.examples[1]}" exited ${textResult.status}: stdout=${textResult.stdout} stderr=${textResult.stderr}`);
  const textFindings = JSON.parse(textResult.stdout).findings;
  assert(Array.isArray(textFindings) && textFindings.length === 1, `expected the --text grep filter to still match the seeded row, got ${textResult.stdout}`);
});

await check('capture.add example runs through the real dispatcher and returns a stub id', async () => {
  const result = await assertExampleOk('capture.add', { cwd: rootBacklogCapture });
  const stub = JSON.parse(result.stdout);
  assert(typeof stub.id === 'string' && stub.id, `expected a stub id, got ${result.stdout}`);
});

await check('capture.list example runs through the real dispatcher and includes the stub just added', async () => {
  const result = await assertExampleOk('capture.list', { cwd: rootBacklogCapture });
  const listed = JSON.parse(result.stdout);
  assert(listed.count >= 1, `expected at least 1 pending stub, got ${result.stdout}`);
});

await check('capture.flush example runs through the real dispatcher against a pre-seeded stub id', async () => {
  // flushCaptureStub refuses an id with no matching pending stub (lib/capture.mjs,
  // never edited by this cell) — capture.add's own example generates a random
  // crypto.randomUUID(), so the literal fixed id in capture.flush's own
  // registry example is seeded directly into the queue file here first.
  const seededId = '00000000-0000-0000-0000-000000000000';
  fs.appendFileSync(
    path.join(rootBacklogCapture, '.bee', 'capture-queue.jsonl'),
    `${JSON.stringify({ kind: 'stub', id: seededId, at: new Date().toISOString(), outcome: 'seeded for capture.flush example', dids: [], area: null, files: [], lane: null })}\n`,
    'utf8',
  );
  const result = await assertExampleOk('capture.flush', { cwd: rootBacklogCapture });
  const record = JSON.parse(result.stdout);
  assert(record.id === seededId, `expected the seeded stub id flushed, got ${result.stdout}`);
});

// ─── intent.* examples (intent-anchor ia-1): its own isolated repo, since
// every verb reads/writes .bee/intent/ and the set→show→advance→clear order
// is the anchor's real lifecycle. ──────────────────────────────────────────

const rootIntent = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-intent-example-'));
fs.mkdirSync(path.join(rootIntent, '.bee'), { recursive: true });
writeJsonAtomic(path.join(rootIntent, '.bee', 'onboarding.json'), {
  schema_version: '1.0',
  bee_version: '0.1.0',
});

await check('intent.set example runs through the real dispatcher and stores the request VERBATIM', async () => {
  const result = await assertExampleOk('intent.set', { cwd: rootIntent });
  const anchor = JSON.parse(result.stdout);
  assert(anchor.request === 'example verbatim request', `expected the verbatim request, got ${result.stdout}`);
  assert(anchor.acceptance === 'example acceptance criteria', `expected the acceptance text, got ${result.stdout}`);
  assert(anchor.next_action === 'example next step', `expected the next action, got ${result.stdout}`);
  // D2: no active feature in this fixture, so the anchor still lands.
  assert(typeof anchor.key === 'string' && anchor.key, 'the anchor must carry the key it landed on');
  const onDisk = JSON.parse(
    fs.readFileSync(path.join(rootIntent, '.bee', 'intent', `${anchor.key}.json`), 'utf8'),
  );
  assert(onDisk.request === 'example verbatim request', 'the anchor is on disk under its key');
});

await check('intent.show example runs through the real dispatcher and returns the stored anchor', async () => {
  const result = await assertExampleOk('intent.show', { cwd: rootIntent });
  const anchor = JSON.parse(result.stdout);
  assert(anchor && anchor.request === 'example verbatim request', `expected the anchor, got ${result.stdout}`);
});

await check('intent.advance example moves next_action ONLY (D1: request/acceptance immutable)', async () => {
  const result = await assertExampleOk('intent.advance', { cwd: rootIntent });
  const anchor = JSON.parse(result.stdout);
  assert(anchor.next_action === 'example advanced next step', `expected the advanced next action, got ${result.stdout}`);
  assert(anchor.request === 'example verbatim request', 'advance must never touch the request');
  assert(anchor.acceptance === 'example acceptance criteria', 'advance must never touch acceptance');
});

await check('intent.clear example removes the anchor and leaves show reporting null', async () => {
  const result = await assertExampleOk('intent.clear', { cwd: rootIntent });
  assert(JSON.parse(result.stdout).cleared === true, `expected cleared:true, got ${result.stdout}`);
  const after = await runModuleWorker(BEE_MJS, { args: ['intent', 'show', '--json'], cwd: rootIntent });
  assert(after.status === 0, `intent show must stay green with no anchor, got ${after.status}`);
  assert(JSON.parse(after.stdout) === null, `expected null with no anchor, got ${after.stdout}`);
});

// ─── chain-integrity D2/D4: scribing debt is a WALL at the close boundary ────
// The post-mortem's real damage: six capped behavior_change cells whose settled
// behavior never reached docs/specs/, while `last_scribing_run` stayed null and
// the feature was marked closed anyway. That state used to be perfectly valid.

function makeDebtRepo() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-scribing-debt-'));
  fs.mkdirSync(path.join(dir, '.bee', 'cells'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  // At `compounding`, so the tail-guard predecessor check passes and the DEBT
  // check is the only thing left standing between here and the terminal phase.
  // compounding-gate D2 (cell cg-1): stamp a prior last_scribing_run/
  // last_compounding_run pair too — real usage never reaches phase
  // "compounding" without a genuine `state scribing-run`, and the tail guard
  // now ALSO requires a matching, at-or-after last_compounding_run. Both are
  // stamped well BEFORE the cells below, so scribingDebt's own threshold math
  // (cappedAt > threshold) is untouched — these tests exist to prove the DEBT
  // wall, not this orthogonal precondition.
  const priorRunAt = new Date(Date.now() - 60000).toISOString();
  writeJsonAtomic(path.join(dir, '.bee', 'state.json'), {
    phase: 'compounding',
    feature: 'demo',
    last_scribing_run: { feature: 'demo', at: priorRunAt },
    last_compounding_run: { feature: 'demo', at: priorRunAt },
  });
  for (const id of ['d-1', 'd-2']) {
    writeJsonAtomic(path.join(dir, '.bee', 'cells', `${id}.json`), {
      id,
      feature: 'demo',
      status: 'capped',
      trace: { behavior_change: true, capped_at: new Date().toISOString() },
    });
  }
  return dir;
}

await check('state set --phase compounding-complete is REFUSED while capped behavior_change cells are unscribed, naming every cell (chain-integrity D2)', async () => {
  const dir = makeDebtRepo();
  try {
    const refused = await runBee(['state', 'set', '--owner', 'compounding', '--phase', 'compounding-complete', '--json'], dir);
    assert(refused.status !== 0, 'closing with scribing debt must be refused');
    const out = refused.stdout + refused.stderr;
    assert(/d-1/.test(out) && /d-2/.test(out), `the refusal must name every unscribed cell, got: ${out}`);
    assert(/waive-scribing-debt/.test(out), `the refusal must disclose the sanctioned door, got: ${out}`);
    assert(
      JSON.parse(fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8')).phase === 'compounding',
      'a refused close must leave the phase untouched — no partial write',
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('--waive-scribing-debt permits the close but is never silent: it logs a decision naming the waived cells (chain-integrity D4)', async () => {
  const dir = makeDebtRepo();
  try {
    const ok = await runBee(['state', 'set', '--owner', 'compounding', '--phase', 'compounding-complete', '--waive-scribing-debt', '--json'], dir);
    assert(ok.status === 0, `the waiver must permit the close, got: ${ok.stdout}${ok.stderr}`);
    assert(
      JSON.parse(fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8')).phase === 'compounding-complete',
      'the waived close must actually write the terminal phase',
    );
    const log = fs.readFileSync(path.join(dir, '.bee', 'decisions.jsonl'), 'utf8');
    assert(/d-1/.test(log) && /d-2/.test(log), `the waiver decision must name every waived cell, got: ${log}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('--waive-scribing-debt under a taxonomy: the waiver audit itself does not throw DECISIONS_UNTAGGED_REFUSED, and lands correctly tagged (jrt-2 — this was the fifth untagged internal logDecision call, and the worst-placed of the five: it fires after the state write already landed)', async () => {
  const dir = makeDebtRepo();
  try {
    fs.mkdirSync(path.join(dir, 'docs', 'decisions'), { recursive: true });
    writeJsonAtomic(path.join(dir, 'docs', 'decisions', 'taxonomy.json'), {
      schema_version: 1,
      tags: [
        { name: 'scribing', description: 'Spec sync and BA capture' },
        { name: 'state', description: 'Runtime state and phases' },
      ],
      candidates: [],
    });
    const ok = await runBee(['state', 'set', '--owner', 'compounding', '--phase', 'compounding-complete', '--waive-scribing-debt', '--json'], dir);
    assert(
      ok.status === 0,
      `before the jrt-2 fix this threw DECISIONS_UNTAGGED_REFUSED after the state write already landed — got status ${ok.status}: ${ok.stdout}${ok.stderr}`,
    );
    assert(
      JSON.parse(fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8')).phase === 'compounding-complete',
      'the waived close must still write the terminal phase under a taxonomy',
    );
    const log = fs.readFileSync(path.join(dir, '.bee', 'decisions.jsonl'), 'utf8');
    const event = JSON.parse(log.trim().split('\n').pop());
    assert(/d-1/.test(event.decision) && /d-2/.test(event.decision), `the waiver decision must name every waived cell, got: ${event.decision}`);
    assert(
      Array.isArray(event.tags) && event.tags.includes('scribing') && event.tags.includes('state'),
      `waiver decision should be tagged scribing+state (what the event IS), got ${JSON.stringify(event.tags)}`,
    );
    assert(/docs\/specs\//.test(event.decision), 'this fixture has no bundle — the decision text still names docs/specs/, unchanged');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('--waive-scribing-debt in a repo with a knowledge bundle: the decision text names the bundle, not docs/specs/ (jrt-2 — the second, separate defect on the same line)', async () => {
  const dir = makeDebtRepo();
  try {
    const bee = { id: 'demo-area-overview', lifecycle: 'active', areas: ['demo'], authoritative_for: 'demo: purpose' };
    const data = { type: 'bee.area', title: 'Demo area — purpose', description: 'A fixture concept.', tags: ['demo'], timestamp: '2026-07-22', bee };
    fs.mkdirSync(path.join(dir, 'docs', 'knowledge', 'areas', 'demo'), { recursive: true });
    fs.writeFileSync(
      path.join(dir, 'docs', 'knowledge', 'areas', 'demo', 'overview.md'),
      `${emitFrontmatter(data)}\n# Demo area — purpose\n\nBody.\n`,
      'utf8',
    );
    const ok = await runBee(['state', 'set', '--owner', 'compounding', '--phase', 'compounding-complete', '--waive-scribing-debt', '--json'], dir);
    assert(ok.status === 0, `the waiver must still permit the close in a bundle repo, got: ${ok.stdout}${ok.stderr}`);
    const log = fs.readFileSync(path.join(dir, '.bee', 'decisions.jsonl'), 'utf8');
    const event = JSON.parse(log.trim().split('\n').pop());
    assert(
      /docs\/knowledge/.test(event.decision) && !/NOT in docs\/specs\//.test(event.decision),
      `in a bundle repo the waiver text must name docs/knowledge/, not the unconditional docs/specs/ wording — got: ${event.decision}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('a close with ZERO scribing debt passes and writes no waiver decision (chain-integrity D2)', async () => {
  const dir = makeDebtRepo();
  try {
    // Stamp a scribing run that post-dates both cells: debt cleared honestly.
    const state = JSON.parse(fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8'));
    state.last_scribing_run = { feature: 'demo', at: new Date(Date.now() + 60_000).toISOString() };
    // compounding-gate D2 (cell cg-1): the tail guard's own precondition
    // needs a last_compounding_run at/after the just-updated scribing run.
    state.last_compounding_run = { feature: 'demo', at: new Date(Date.now() + 120_000).toISOString() };
    writeJsonAtomic(path.join(dir, '.bee', 'state.json'), state);
    const ok = await runBee(['state', 'set', '--owner', 'compounding', '--phase', 'compounding-complete', '--json'], dir);
    assert(ok.status === 0, `a debt-free close must pass, got: ${ok.stdout}${ok.stderr}`);
    assert(
      !fs.existsSync(path.join(dir, '.bee', 'decisions.jsonl')),
      'a debt-free close must not log a waiver decision — nothing was waived',
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('capture.count example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('capture.count', { cwd: rootBacklogCapture });
  assert(typeof JSON.parse(result.stdout).count === 'number', `expected a numeric count, got ${result.stdout}`);
});

// ─── capture add --source CLI flag + capture list [mined] marker + flush
// works identically (transcript-recovery D6: mined-unconfirmed = a source:
// "mined" stub sitting unflushed; the normal flush IS the confirmation) ─────

await check('capture add --source persists provenance; capture list marks [mined]; flush works identically (D6)', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-capture-source-cli-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  try {
    const added = await runBee(
      ['capture', 'add', '--outcome', 'mined from crashed session', '--source', 'mined', '--json'],
      dir,
    );
    assert(added.status === 0, `capture add --source failed: ${added.stdout}${added.stderr}`);
    const stub = JSON.parse(added.stdout);
    assert(stub.source === 'mined', `expected source "mined" persisted, got ${added.stdout}`);

    const addedPlain = await runBee(['capture', 'add', '--outcome', 'ordinary settlement', '--json'], dir);
    assert(addedPlain.status === 0, `capture add without --source failed: ${addedPlain.stdout}${addedPlain.stderr}`);
    const plainStub = JSON.parse(addedPlain.stdout);
    assert(!('source' in plainStub), `an ordinary stub must not carry a source key, got ${addedPlain.stdout}`);

    const listed = await runBee(['capture', 'list'], dir);
    assert(listed.status === 0, `capture list failed: ${listed.stdout}${listed.stderr}`);
    assert(
      /mined from crashed session[^\n]*\[mined\]/.test(listed.stdout),
      `mined stub must render a [mined] marker, got: ${listed.stdout}`,
    );
    assert(
      !/ordinary settlement[^\n]*\[mined\]/.test(listed.stdout),
      `ordinary stub must not render a [mined] marker, got: ${listed.stdout}`,
    );

    // flush works identically for a mined stub — zero special-casing (D6)
    const flushed = await runBee(['capture', 'flush', '--id', stub.id, '--json'], dir);
    assert(flushed.status === 0, `flush of a mined stub failed: ${flushed.stdout}${flushed.stderr}`);
    const record = JSON.parse(flushed.stdout);
    assert(record.id === stub.id, `flush must confirm the mined stub id, got ${flushed.stdout}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── reviews.* / feedback.* examples: run in a dedicated fresh repo
// (dispatcher-unify du-3). reviews.create's A10 preflight requires a real
// capped behavior_change cell WITH recorded verification_evidence in scope,
// so a fixture cell ("ok-1") is built here through the real dispatcher
// (add/claim/verify/cap) before the reviews.create example runs. feedback's
// digest/count/collect/rank examples run over whatever sources are in scope
// in this same repo (an empty/near-empty source set is fine — buildDigest
// degrades to a low-count snapshot rather than throwing).

const rootReviewsFeedback = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-reviews-feedback-example-'));
fs.mkdirSync(path.join(rootReviewsFeedback, '.bee'), { recursive: true });
writeJsonAtomic(path.join(rootReviewsFeedback, '.bee', 'onboarding.json'), {
  schema_version: '1.0',
  bee_version: '0.1.0',
});
writeState(rootReviewsFeedback, {
  ...defaultState(),
  phase: 'swarming',
  feature: 'demo3',
  approved_gates: { context: true, shape: true, execution: true, review: false },
});

async function runBeeReviewsFeedbackFixture(args) {
  return await runModuleWorker(BEE_MJS, { args, cwd: rootReviewsFeedback });
}

await check('reviews fixture setup: a capped behavior_change cell ("ok-1") with recorded verification_evidence exists in scope', async () => {
  const cellFixture = {
    id: 'ok-1',
    feature: 'demo3',
    title: 'Fixture cell for reviews.* registry examples',
    lane: 'small',
    action: 'Exercise every reviews.* example against a real fixture cell.',
    verify: 'node -e "process.exit(0)"',
    behavior_change: true,
  };
  fs.writeFileSync(path.join(rootReviewsFeedback, 'cell-ok-1.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
  const added = await runBeeReviewsFeedbackFixture(['cells', 'add', '--file', 'cell-ok-1.json', '--json']);
  assert(added.status === 0, `cells add setup failed: ${added.status}: stdout=${added.stdout} stderr=${added.stderr}`);

  const claimed = await runBeeReviewsFeedbackFixture(['cells', 'claim', '--id', 'ok-1', '--worker', 'worker-rev', '--json']);
  assert(claimed.status === 0, `cells claim setup failed: ${claimed.status}: stdout=${claimed.stdout} stderr=${claimed.stderr}`);

  const verified = await runBeeReviewsFeedbackFixture(['cells', 'verify', '--id', 'ok-1', '--command', 'node -e 0', '--output', 'ok', '--passed', 'true', '--json']);
  assert(verified.status === 0, `cells verify setup failed: ${verified.status}: stdout=${verified.stdout} stderr=${verified.stderr}`);

  const capped = await runModuleWorker(BEE_MJS, {
    args: ['cells', 'cap', '--id', 'ok-1', '--outcome', 'done', '--files', 'a.js', '--behavior-change', '--evidence-stdin', '--json'],
    cwd: rootReviewsFeedback,
    input: JSON.stringify({
      red_failure_evidence:
        'ok-1: prior behavior characterized before this reviews-fixture change, meeting the D3 anti-boilerplate floor (>=80 chars).',
      verification_run: 'node -e 0',
    }),
  });
  assert(capped.status === 0, `cells cap setup failed: ${capped.status}: stdout=${capped.stdout} stderr=${capped.stderr}`);
  assert(JSON.parse(capped.stdout).trace.verification_evidence, 'ok-1 should carry recorded verification_evidence for the A10 preflight');
});

await check('reviews.create example runs through the real dispatcher (A10 preflight satisfied by the ok-1 fixture cell)', async () => {
  const scope = {
    id: 'rev-example',
    requested_by: 'user',
    scope_description: 'review the demo3 feature',
    included: [{ type: 'cell', id: 'ok-1' }],
    baseline: 'sha-base',
    head: 'sha-head',
  };
  fs.writeFileSync(path.join(rootReviewsFeedback, 'scope.json'), JSON.stringify(scope), 'utf8');
  const result = await assertExampleOk('reviews.create', { cwd: rootReviewsFeedback });
  assert(JSON.parse(result.stdout).id === 'rev-example', `expected rev-example, got ${result.stdout}`);
});

await check('reviews.list example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('reviews.list', { cwd: rootReviewsFeedback });
  assert(result.stdout.includes('rev-example'), `expected rev-example in list output, got ${result.stdout}`);
});

await check('reviews.show example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('reviews.show', { cwd: rootReviewsFeedback });
  assert(JSON.parse(result.stdout).id === 'rev-example', `expected rev-example, got ${result.stdout}`);
});

await check('reviews.record example runs through the real dispatcher', async () => {
  fs.writeFileSync(path.join(rootReviewsFeedback, 'finding.json'), JSON.stringify({ severity: 'P2', description: 'nit' }), 'utf8');
  const result = await assertExampleOk('reviews.record', { cwd: rootReviewsFeedback });
  assert(JSON.parse(result.stdout).id === 'rev-example', `expected the updated rev-example session, got ${result.stdout}`);
});

await check('reviews.candidate.add example runs through the real dispatcher (nested 3-token verb)', async () => {
  const result = await assertExampleOk('reviews.candidate.add', { cwd: rootReviewsFeedback });
  const entry = JSON.parse(result.stdout);
  assert(entry.feature === 'demo3' && entry.mode === 'standard', `expected the example candidate, got ${result.stdout}`);
});

await check('reviews candidate add auto-fills cells from the feature capped cells when --cells is omitted (GitHub #16)', async () => {
  const rr = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-cand-cells-'));
  try {
    fs.mkdirSync(path.join(rr, '.bee', 'cells'), { recursive: true });
    writeJsonAtomic(path.join(rr, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
    writeJsonAtomic(path.join(rr, '.bee', 'cells', 'revfeat-1.json'), { id: 'revfeat-1', feature: 'revfeat', title: 't', lane: 'small', action: 'a', status: 'capped' });
    writeJsonAtomic(path.join(rr, '.bee', 'cells', 'revfeat-2.json'), { id: 'revfeat-2', feature: 'revfeat', title: 't2', lane: 'small', action: 'a', status: 'open' });
    writeJsonAtomic(path.join(rr, '.bee', 'cells', 'other-1.json'), { id: 'other-1', feature: 'other', title: 't3', lane: 'small', action: 'a', status: 'capped' });
    const res = await runModuleWorker(BEE_MJS, { args: ['reviews', 'candidate', 'add', '--feature', 'revfeat', '--head', 'abc123', '--mode', 'small', '--json'], cwd: rr });
    assert(res.status === 0, `candidate add exit ${res.status}: ${res.stderr}`);
    const entry = JSON.parse(res.stdout);
    assert(
      Array.isArray(entry.cells) && entry.cells.length === 1 && entry.cells[0] === 'revfeat-1',
      `cells should auto-fill to the feature's CAPPED cell only (not open/other-feature), got ${JSON.stringify(entry.cells)}`,
    );
  } finally {
    fs.rmSync(rr, { recursive: true, force: true });
  }
});

await check('reviews.candidates example runs through the real dispatcher (flat 2-token verb, distinct from candidate add)', async () => {
  const result = await assertExampleOk('reviews.candidates', { cwd: rootReviewsFeedback });
  const entries = JSON.parse(result.stdout);
  assert(entries.length === 1 && entries[0].feature === 'demo3', `expected the candidate just added, got ${result.stdout}`);
});

await check('reviews.status example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('reviews.status', { cwd: rootReviewsFeedback });
  const summary = JSON.parse(result.stdout);
  assert(summary.counts.verified === 1, `expected 1 verified candidate, got ${result.stdout}`);
});

await check('feedback.digest example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('feedback.digest', { cwd: rootReviewsFeedback });
  assert(typeof JSON.parse(result.stdout).digest === 'object', `expected a digest object, got ${result.stdout}`);
});

await check('feedback.count example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('feedback.count', { cwd: rootReviewsFeedback });
  assert(typeof JSON.parse(result.stdout).entries === 'number', `expected a numeric entries count, got ${result.stdout}`);
});

await check('feedback.collect example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('feedback.collect', { cwd: rootReviewsFeedback });
  assert(typeof JSON.parse(result.stdout).counts === 'object', `expected a counts object, got ${result.stdout}`);
});

await check('feedback.rank example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('feedback.rank', { cwd: rootReviewsFeedback });
  assert(Array.isArray(JSON.parse(result.stdout)), `expected a ranked cluster array, got ${result.stdout}`);
});

// ─── perf group examples (global perf log; env redirected to the temp repo) ─
// start must run before stop (stop reads the marker start writes). All five run
// against a transcript-less window (fake CLAUDE_CONFIG_DIR) and must still exit 0.
await check('perf.start example writes an open-section marker and exits 0', async () => {
  const result = await assertExampleOk('perf.start');
  const marker = JSON.parse(fs.readFileSync(path.join(root, '.bee', 'cache', 'perf-open.json'), 'utf8'));
  assert(marker.started_at, 'marker records a start time');
  assert(result.status === 0, 'perf start exits 0');
});
await check('perf.stop example closes the section, appends to the global log, clears the marker', async () => {
  const result = await assertExampleOk('perf.stop');
  const rec = JSON.parse(result.stdout);
  assert(rec.schema === 'bee-perf/v1', `section schema tag, got ${result.stdout}`);
  assert(!fs.existsSync(path.join(root, '.bee', 'cache', 'perf-open.json')), 'marker cleared after stop');
  const log = fs.readFileSync(path.join(root, 'perf-global', 'performance.jsonl'), 'utf8').trim();
  assert(log.split('\n').length >= 1, 'section appended to the global log');
});
await check('perf.section one-shot example computes + appends and exits 0', async () => {
  const result = await assertExampleOk('perf.section');
  assert(JSON.parse(result.stdout).schema === 'bee-perf/v1', 'one-shot section logged');
});
await check('perf.log example reads sections back and exits 0', async () => {
  const result = await assertExampleOk('perf.log');
  assert(Array.isArray(JSON.parse(result.stdout)), 'perf log --json returns an array');
});
await check('perf.render example emits Markdown and exits 0', async () => {
  const result = await assertExampleOk('perf.render');
  assert(/bee performance log/.test(result.stdout), 'render emits the report heading');
});
await check('perf.report example reads the store (transcript-less temp env) and exits 0', async () => {
  const result = await assertExampleOk('perf.report');
  const matrix = JSON.parse(result.stdout);
  assert(Array.isArray(matrix.projects), 'perf report --json returns a matrix with a projects array');
});
await check('perf.sync example scans + writes the log (transcript-less temp env) and exits 0', async () => {
  const result = await assertExampleOk('perf.sync');
  const res = JSON.parse(result.stdout);
  assert(typeof res.sessions === 'number', 'perf sync --json reports a session count');
});

// ─── tmp group example (tree-hygiene th-4, CONTEXT D1/D2): --all --dry-run is
// deliberately the registry's own example — it is the one call shape that is
// always safe to run for real against ANY fixture (never deletes anything,
// never refuses for lack of a flag) while still exercising the full
// dispatcher -> lib/scratch.mjs wiring end to end. ─────────────────────────
await check('tmp.sweep example (--all --dry-run) exits 0 and never deletes anything', async () => {
  const result = await assertExampleOk('tmp.sweep');
  const res = JSON.parse(result.stdout);
  assert(res.dry_run === true, `tmp sweep --dry-run example must report dry_run:true, got ${result.stdout}`);
  assert(Array.isArray(res.removed), 'tmp sweep --json reports a removed[] array');
});

// ─── dispatch group example (g22-1, GH #22 P0-3): a read-only "gather" kind
// needs no --cell and no extra fixture state, so it runs safely against the
// shared `root` fixture above (no config.json there -> the seeded default
// claude.generation model "sonnet" resolves, matching state.mjs's
// DEFAULT_MODELS). Full behavioral coverage (codex/claude payload shapes,
// the cli-cell refusal, advisor resolution, the prepare-time dispatch
// record) lives in scripts/test_dispatch_prepare.mjs — this is the
// registry-example-is-a-tested-contract proof for the new group.
await check('dispatch.prepare example runs through the real dispatcher', async () => {
  const result = await assertExampleOk('dispatch.prepare');
  const out = JSON.parse(result.stdout);
  assert(out.tool === 'Agent', `expected tool Agent, got ${result.stdout}`);
  assert(out.payload.subagent_type === 'bee-gather', `expected pinned type bee-gather, got ${result.stdout}`);
  assert(typeof out.dispatch_id === 'string' && out.dispatch_id, `expected a dispatch_id, got ${result.stdout}`);
  assert(out.economics && out.economics.channel === 'claude-agent', `expected channel claude-agent, got ${result.stdout}`);
});

// ─── worktree group examples: a REAL git repo + real `git worktree add`,
// mirroring the fixture pattern scripts/test_worktree_cli.mjs already proved
// end-to-end. A dedicated temp tree (not the shared `root` above, which has
// no .git and is deliberately classified 'ordinary') so register's own
// "must run from inside a linked worktree" requirement is satisfiable. ─────
await check('worktree.new example runs through the real dispatcher against a real ORDINARY checkout, creating and granting a linked worktree in one move (wsr-1, GH #21)', async () => {
  const wtNewTmp = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-worktree-new-'));
  try {
    const git = (cwd, args) => {
      const r = spawnSync('git', args, { cwd, encoding: 'utf8' });
      assert(r.status === 0, `git ${args.join(' ')} (cwd=${cwd}) failed: ${r.stderr}`);
      return r.stdout;
    };

    const wtNewMain = path.join(wtNewTmp, 'main');
    fs.mkdirSync(wtNewMain);
    git(wtNewMain, ['init', '-q', '-b', 'main']);
    git(wtNewMain, ['config', 'user.email', 's@e']);
    git(wtNewMain, ['config', 'user.name', 's']);
    fs.writeFileSync(path.join(wtNewMain, 'f'), 'x');
    git(wtNewMain, ['add', '.']);
    git(wtNewMain, ['commit', '-q', '-m', 'init']);
    fs.mkdirSync(path.join(wtNewMain, '.bee'), { recursive: true });
    writeJsonAtomic(path.join(wtNewMain, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });

    // registry example: 'bee worktree new --feature demo-feature --json'
    const result = await assertExampleOk('worktree.new', { cwd: wtNewMain });
    const created = JSON.parse(result.stdout);
    assert(typeof created.id === 'string' && created.id, `worktree.new example should report a git-verified id, got ${result.stdout}`);
    assert(created.branch === 'wt/demo-feature', `worktree.new example should create branch "wt/demo-feature", got ${JSON.stringify(created)}`);
    assert(fs.existsSync(created.worktreeRoot), `worktree.new example should create ${created.worktreeRoot}`);
    const newStateFile = path.join(created.worktreeRoot, '.bee', 'state.json');
    assert(fs.existsSync(newStateFile), 'worktree.new example should bootstrap .bee/state.json');
    const newState = JSON.parse(fs.readFileSync(newStateFile, 'utf8'));
    assert(
      newState.feature === 'demo-feature' && newState.phase === 'idle',
      `expected a fresh idle demo-feature state, got ${JSON.stringify(newState)}`,
    );
    const grantsFile = path.join(wtNewMain, '.bee', 'runtime', 'worktree-grants.json');
    const grants = JSON.parse(fs.readFileSync(grantsFile, 'utf8'));
    assert(grants[created.id] === true, `worktree.new example should grant the new worktree's id, got ${JSON.stringify(grants)}`);

    // Running the SAME example again from the same ordinary checkout must
    // typed-refuse (the target directory now exists), never crash.
    const repeatResult = await runExample('worktree.new', { cwd: wtNewMain });
    assert(repeatResult.result.status !== 0, 'a second "worktree new --feature demo-feature" from the same checkout must not exit 0');
    assert(
      /WORKTREE_TARGET_EXISTS/.test(repeatResult.result.stdout + repeatResult.result.stderr),
      `expected a typed WORKTREE_TARGET_EXISTS refusal, got stdout=${repeatResult.result.stdout} stderr=${repeatResult.result.stderr}`,
    );
  } finally {
    fs.rmSync(wtNewTmp, { recursive: true, force: true });
  }
});

await check('worktree.merge example (registry refusal-shaped: unknown id) runs through the real dispatcher against a real ORDINARY checkout (wsr-2, GH #21)', async () => {
  const wtMergeTmp = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-worktree-merge-'));
  try {
    const git = (cwd, args) => {
      const r = spawnSync('git', args, { cwd, encoding: 'utf8' });
      assert(r.status === 0, `git ${args.join(' ')} (cwd=${cwd}) failed: ${r.stderr}`);
      return r.stdout;
    };

    const wtMergeMain = path.join(wtMergeTmp, 'main');
    fs.mkdirSync(wtMergeMain);
    git(wtMergeMain, ['init', '-q', '-b', 'main']);
    git(wtMergeMain, ['config', 'user.email', 's@e']);
    git(wtMergeMain, ['config', 'user.name', 's']);
    fs.writeFileSync(path.join(wtMergeMain, 'f'), 'x');
    git(wtMergeMain, ['add', '.']);
    git(wtMergeMain, ['commit', '-q', '-m', 'init']);
    fs.mkdirSync(path.join(wtMergeMain, '.bee'), { recursive: true });
    writeJsonAtomic(path.join(wtMergeMain, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });

    // registry example: 'bee worktree merge --id demo-feature-missing --json'
    // — deliberately refusal-shaped (an unknown/ungranted id): no worktree
    // fixture is needed just to prove the example is runnable through the
    // real dispatcher from a real ORDINARY checkout. The full green-path /
    // MERGE_CONFLICT / MERGE_VERIFY_RED / cleanup surface is proven
    // end-to-end, with real git worktrees, in scripts/test_worktree_cli.mjs
    // (part of the mandatory verify chain) — this check only satisfies the
    // "every registry example is executed" guard below.
    const { result } = await runExample('worktree.merge', { cwd: wtMergeMain });
    assert(result.status !== 0, `expected the unknown-id example to refuse (non-zero exit), got status 0: ${result.stdout}`);
    assert(
      /WORKTREE_MERGE_UNKNOWN_ID/.test(result.stdout + result.stderr),
      `expected a typed WORKTREE_MERGE_UNKNOWN_ID refusal, got stdout=${result.stdout} stderr=${result.stderr}`,
    );
  } finally {
    fs.rmSync(wtMergeTmp, { recursive: true, force: true });
  }
});

await check('worktree.register/list/unregister examples run through the real dispatcher against a real linked git worktree', async () => {
  const wtTmp = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-worktree-'));
  try {
    const git = (cwd, args) => {
      const r = spawnSync('git', args, { cwd, encoding: 'utf8' });
      assert(r.status === 0, `git ${args.join(' ')} (cwd=${cwd}) failed: ${r.stderr}`);
      return r.stdout;
    };

    const wtMain = path.join(wtTmp, 'main');
    fs.mkdirSync(wtMain);
    git(wtMain, ['init', '-q', '-b', 'main']);
    git(wtMain, ['config', 'user.email', 's@e']);
    git(wtMain, ['config', 'user.name', 's']);
    fs.writeFileSync(path.join(wtMain, 'f'), 'x');
    git(wtMain, ['add', '.']);
    git(wtMain, ['commit', '-q', '-m', 'init']);
    fs.mkdirSync(path.join(wtMain, '.bee'), { recursive: true });
    writeJsonAtomic(path.join(wtMain, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });

    const wtLinked = path.join(wtTmp, 'wt');
    git(wtMain, ['worktree', 'add', '-q', '-b', 'wt-example-feature', wtLinked]);

    // registry example: 'bee worktree register --feature demo-feature --json'
    await assertExampleOk('worktree.register', { cwd: wtLinked });
    const worktreeStateFile = path.join(wtLinked, '.bee', 'state.json');
    assert(fs.existsSync(worktreeStateFile), 'worktree.register example should bootstrap .bee/state.json');
    const worktreeState = JSON.parse(fs.readFileSync(worktreeStateFile, 'utf8'));
    assert(worktreeState.feature === 'demo-feature' && worktreeState.phase === 'idle', `expected a fresh idle demo-feature state, got ${JSON.stringify(worktreeState)}`);
    const grantsFile = path.join(wtMain, '.bee', 'runtime', 'worktree-grants.json');
    const grantedIds = Object.keys(JSON.parse(fs.readFileSync(grantsFile, 'utf8')));
    assert(grantedIds.length === 1, `expected exactly one grant after register, got ${JSON.stringify(grantedIds)}`);
    const realId = grantedIds[0];

    // registry example: 'bee worktree list --json'
    const listResult = await assertExampleOk('worktree.list', { cwd: wtLinked });
    const listed = JSON.parse(listResult.stdout);
    assert(listed.grants[realId] === true, `worktree.list example should show the real grant, got ${listResult.stdout}`);

    // registry example: 'bee worktree unregister --id abc123 --json' — a real
    // dispatcher call for an id that was never granted, scoped-removal no-op
    // (never an error): proves the example runs cleanly AND that unregister
    // never touches an unrelated id's grant.
    await assertExampleOk('worktree.unregister', { cwd: wtLinked });
    const afterExampleGrants = JSON.parse(fs.readFileSync(grantsFile, 'utf8'));
    assert(afterExampleGrants[realId] === true, `unregister --id abc123 must not remove the real grant, got ${JSON.stringify(afterExampleGrants)}`);

    // Now exercise the real (no --id) default path directly, proving it
    // resolves the CURRENT worktree's own id and actually removes it.
    const realUnregisterResult = await runModuleWorker(BEE_MJS, { args: ['worktree', 'unregister', '--json'], cwd: wtLinked });
    assert(realUnregisterResult.status === 0, `real unregister (no --id) should exit 0, got status=${realUnregisterResult.status} stderr=${realUnregisterResult.stderr}`);
    const finalGrants = JSON.parse(fs.readFileSync(grantsFile, 'utf8'));
    assert(!(realId in finalGrants), `real unregister (no --id) should remove the current worktree's own grant, got ${JSON.stringify(finalGrants)}`);
  } finally {
    fs.rmSync(wtTmp, { recursive: true, force: true });
  }
});

// herding-dispatch-lock-toggle: herding.enable/disable/status resolve the
// MAIN checkout root via `git rev-parse --git-common-dir` (mirroring
// dispatch-interlock.mjs exactly), so — like the worktree group above — these
// examples need a REAL git repo, not the shared `.bee`-only `root` fixture.
await check('herding.enable/status/disable examples run through the real dispatcher against a real git repo', async () => {
  const herdingTmp = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-herding-'));
  try {
    const git = (cwd, args) => {
      const r = spawnSync('git', args, { cwd, encoding: 'utf8' });
      assert(r.status === 0, `git ${args.join(' ')} (cwd=${cwd}) failed: ${r.stderr}`);
      return r.stdout;
    };

    const herdingMain = path.join(herdingTmp, 'main');
    fs.mkdirSync(herdingMain);
    git(herdingMain, ['init', '-q', '-b', 'main']);
    git(herdingMain, ['config', 'user.email', 's@e']);
    git(herdingMain, ['config', 'user.name', 's']);
    fs.writeFileSync(path.join(herdingMain, 'f'), 'x');
    git(herdingMain, ['add', '.']);
    git(herdingMain, ['commit', '-q', '-m', 'init']);
    fs.mkdirSync(path.join(herdingMain, '.bee'), { recursive: true });
    writeJsonAtomic(path.join(herdingMain, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });

    const marker = path.join(herdingMain, '.bee', 'tmp', 'bee-herding.enable');

    // registry example: 'bee herding status --json'
    const offResult = await assertExampleOk('herding.status', { cwd: herdingMain });
    assert(JSON.parse(offResult.stdout).enabled === false, `expected enabled:false before any enable, got ${offResult.stdout}`);

    // registry example: 'bee herding enable --json'
    await assertExampleOk('herding.enable', { cwd: herdingMain });
    assert(fs.existsSync(marker), 'herding.enable example should create the owner marker');

    const onResult = await assertExampleOk('herding.status', { cwd: herdingMain, exampleIndex: 0 });
    assert(JSON.parse(onResult.stdout).enabled === true, `expected enabled:true after enable, got ${onResult.stdout}`);

    // registry example: 'bee herding disable --json'
    await assertExampleOk('herding.disable', { cwd: herdingMain });
    assert(!fs.existsSync(marker), 'herding.disable example should remove the owner marker');
  } finally {
    fs.rmSync(herdingTmp, { recursive: true, force: true });
  }
});

await check('config.validate example runs through the real dispatcher: clean config exits 0, a malformed/prompt-less/unsafe cli-tier config exits 1 with named problems', async () => {
  // registry example: 'bee config validate --json' — the shared fixture repo
  // (`root`) has no .bee/config.json at all, the common "fresh repo" case
  // this validator must treat as clean, never a problem.
  const cleanResult = await assertExampleOk('config.validate', { cwd: root });
  const cleanParsed = JSON.parse(cleanResult.stdout);
  assert(cleanParsed.ok === true && cleanParsed.problem_count === 0, `expected a clean config to report ok, got ${cleanResult.stdout}`);

  // A second, isolated repo whose config.json carries every kind of models
  // problem this cell exists to catch — proves the real dispatcher path
  // (not just the unit-level validateModelsConfig calls in
  // test_config_validate.mjs) surfaces them and exits non-zero.
  const cfgTmp = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-config-validate-'));
  try {
    fs.mkdirSync(path.join(cfgTmp, '.bee'), { recursive: true });
    writeJsonAtomic(path.join(cfgTmp, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
    writeJsonAtomic(path.join(cfgTmp, '.bee', 'config.json'), {
      models: {
        claude: {
          generation: { kind: 'cli', command: 'some-cli exec --yolo' }, // no promptVia AND an unsafe flag
          review: { command: 'missing-kind-cli' }, // (a) malformed: no kind:'cli'
        },
      },
    });
    const badResult = await runModuleWorker(BEE_MJS, { args: ['config', 'validate', '--json'], cwd: cfgTmp });
    assert(badResult.status === 1, `expected exit 1 on a problem config, got ${badResult.status}: ${badResult.stdout}`);
    const badParsed = JSON.parse(badResult.stdout);
    assert(badParsed.ok === false && badParsed.problem_count >= 3, `expected ok:false with >= 3 problems, got ${badResult.stdout}`);
    const codes = badParsed.problems.map((p) => p.code);
    assert(codes.includes('cli-prompt-transport-missing'), `expected cli-prompt-transport-missing, got ${JSON.stringify(codes)}`);
    assert(codes.includes('cli-unsafe-flag'), `expected cli-unsafe-flag, got ${JSON.stringify(codes)}`);
    assert(codes.includes('cli-malformed'), `expected cli-malformed, got ${JSON.stringify(codes)}`);
  } finally {
    fs.rmSync(cfgTmp, { recursive: true, force: true });
  }
});

await check('config set/get/unset examples round-trip through the real dispatcher (GitHub #15)', async () => {
  const cfgRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-config-getset-'));
  try {
    fs.mkdirSync(path.join(cfgRoot, '.bee'), { recursive: true });
    writeJsonAtomic(path.join(cfgRoot, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
    // set: `--value false` is JSON-coerced to boolean false, not the string "false".
    const setRes = await assertExampleOk('config.set', { cwd: cfgRoot });
    assert(JSON.parse(setRes.stdout).value === false, `set should coerce false to boolean, got ${setRes.stdout}`);
    const onDisk = JSON.parse(fs.readFileSync(path.join(cfgRoot, '.bee', 'config.json'), 'utf8'));
    assert(onDisk.gate_bypass === false, `config.json should carry gate_bypass:false, got ${JSON.stringify(onDisk)}`);
    // get: reads it back.
    const got = JSON.parse((await assertExampleOk('config.get', { cwd: cfgRoot })).stdout);
    assert(got.present === true && got.value === false, `get should read gate_bypass:false, got ${JSON.stringify(got)}`);
    // unset: removes it.
    const unset = JSON.parse((await assertExampleOk('config.unset', { cwd: cfgRoot })).stdout);
    assert(unset.removed === true, `unset should remove the key, got ${JSON.stringify(unset)}`);
    assert(!('gate_bypass' in JSON.parse(fs.readFileSync(path.join(cfgRoot, '.bee', 'config.json'), 'utf8'))), 'gate_bypass should be gone');
  } finally {
    fs.rmSync(cfgRoot, { recursive: true, force: true });
  }
});

await check('config set: nested dot-key, string coercion, refuse-on-invalid, no-clobber of a malformed file (GitHub #15)', async () => {
  const cfgRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-config-edge-'));
  try {
    fs.mkdirSync(path.join(cfgRoot, '.bee'), { recursive: true });
    writeJsonAtomic(path.join(cfgRoot, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
    // nested dot-key -> creates ui.theme = false
    // (was guards.idle_gate before D2/intake-gate-git-exemption forced ALL
    // guards.*/hooks.* keys to the local overlay — see the dedicated "config
    // set/get/unset routes guards.*/hooks.* to the local overlay" test below
    // and packages/bee/tests/test_config_validate.mjs for that
    // namespace's own coverage; this test is about nested dot-key mechanics
    // on the TRACKED file generally, so it now uses a neutral namespace.)
    let r = await runModuleWorker(BEE_MJS, { args: ['config', 'set', '--key', 'ui.theme', '--value', 'false', '--json'], cwd: cfgRoot });
    assert(r.status === 0, `nested set exit ${r.status}: ${r.stderr}`);
    let disk = JSON.parse(fs.readFileSync(path.join(cfgRoot, '.bee', 'config.json'), 'utf8'));
    assert(disk.ui && disk.ui.theme === false, `ui.theme should be false, got ${JSON.stringify(disk)}`);
    // unset prunes the now-empty parent: no stray "ui": {} left behind
    r = await runModuleWorker(BEE_MJS, { args: ['config', 'unset', '--key', 'ui.theme', '--json'], cwd: cfgRoot });
    assert(r.status === 0 && JSON.parse(r.stdout).removed === true, `nested unset exit ${r.status}: ${r.stdout}`);
    disk = JSON.parse(fs.readFileSync(path.join(cfgRoot, '.bee', 'config.json'), 'utf8'));
    assert(!('ui' in disk), `unset should prune the empty ui parent, got ${JSON.stringify(disk)}`);
    // a non-JSON value stays a string
    r = await runModuleWorker(BEE_MJS, { args: ['config', 'set', '--key', 'product_root', '--value', 'repo', '--json'], cwd: cfgRoot });
    assert(r.status === 0 && JSON.parse(r.stdout).value === 'repo', `product_root should be string "repo", got ${r.stdout}`);
    // refuse-on-invalid: an unsafe cli command must be rejected and NOT written
    r = await runModuleWorker(BEE_MJS, { args: ['config', 'set', '--key', 'models.claude.generation', '--value', '{"kind":"cli","command":"x --yolo"}', '--json'], cwd: cfgRoot });
    assert(r.status !== 0, `an unsafe cli set should be refused, got exit ${r.status}: ${r.stdout}`);
    disk = JSON.parse(fs.readFileSync(path.join(cfgRoot, '.bee', 'config.json'), 'utf8'));
    assert(!(disk.models && disk.models.claude), `the refused set must not have been written, got ${JSON.stringify(disk)}`);
    // no-clobber: a malformed config file must be left intact, set refused
    fs.writeFileSync(path.join(cfgRoot, '.bee', 'config.json'), '{ broken', 'utf8');
    r = await runModuleWorker(BEE_MJS, { args: ['config', 'set', '--key', 'product_root', '--value', 'x', '--json'], cwd: cfgRoot });
    assert(r.status !== 0, `set on a malformed config must refuse, got exit ${r.status}`);
    assert(fs.readFileSync(path.join(cfgRoot, '.bee', 'config.json'), 'utf8').includes('broken'), 'the malformed file must be left intact');
  } finally {
    fs.rmSync(cfgRoot, { recursive: true, force: true });
  }
});

await check('config set/get/unset --local redirects to .bee/config.local.json, never the tracked config.json (hardening-8 config overlay)', async () => {
  const cfgRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-config-local-'));
  try {
    fs.mkdirSync(path.join(cfgRoot, '.bee'), { recursive: true });
    writeJsonAtomic(path.join(cfgRoot, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
    // `--local` is a bare boolean flag (FLAG_ALONE_BOOLEANS) — regression
    // guard for the parser bug caught live while writing this fixture: before
    // --local was added to that set, the flag parser consumed the NEXT token
    // (here, --json) as --local's value, so `flags.local === true` was never
    // true and the write silently landed in the tracked file instead.
    let r = await runModuleWorker(BEE_MJS, {
      args: ['config', 'set', '--key', 'dogfood_repos', '--value', '[{"path":"/tmp/x","label":"x"}]', '--local', '--json'],
      cwd: cfgRoot,
    });
    assert(r.status === 0, `config set --local exit ${r.status}: ${r.stderr}`);
    const setParsed = JSON.parse(r.stdout);
    assert(setParsed.local === true, `set result must report local:true, got ${r.stdout}`);
    assert(!fs.existsSync(path.join(cfgRoot, '.bee', 'config.json')), 'a --local set must never create/touch the tracked config.json');
    const overlayOnDisk = JSON.parse(fs.readFileSync(path.join(cfgRoot, '.bee', 'config.local.json'), 'utf8'));
    assert(Array.isArray(overlayOnDisk.dogfood_repos) && overlayOnDisk.dogfood_repos[0].label === 'x', `dogfood_repos must land in config.local.json, got ${JSON.stringify(overlayOnDisk)}`);

    // get --local reads back from the overlay.
    r = await runModuleWorker(BEE_MJS, { args: ['config', 'get', '--key', 'dogfood_repos', '--local', '--json'], cwd: cfgRoot });
    assert(r.status === 0, `config get --local exit ${r.status}: ${r.stderr}`);
    const got = JSON.parse(r.stdout);
    assert(got.present === true && got.local === true, `get --local should read the overlay value, got ${r.stdout}`);

    // omitting --local on a plain set still targets the tracked file exactly as before (D4 zero-flag parity).
    r = await runModuleWorker(BEE_MJS, { args: ['config', 'set', '--key', 'product_root', '--value', 'repo', '--json'], cwd: cfgRoot });
    assert(r.status === 0, `plain set (no --local) exit ${r.status}: ${r.stderr}`);
    const tracked = JSON.parse(fs.readFileSync(path.join(cfgRoot, '.bee', 'config.json'), 'utf8'));
    assert(tracked.product_root === 'repo', `a plain set must land in the tracked config.json, got ${JSON.stringify(tracked)}`);
    assert(!('dogfood_repos' in tracked), 'the tracked file must never gain the overlay-only key');

    // unset --local removes it from the overlay, never the tracked file.
    r = await runModuleWorker(BEE_MJS, { args: ['config', 'unset', '--key', 'dogfood_repos', '--local', '--json'], cwd: cfgRoot });
    assert(r.status === 0, `config unset --local exit ${r.status}: ${r.stderr}`);
    const unsetParsed = JSON.parse(r.stdout);
    assert(unsetParsed.removed === true && unsetParsed.local === true, `unset --local should report removed:true, local:true, got ${r.stdout}`);
    const overlayAfter = JSON.parse(fs.readFileSync(path.join(cfgRoot, '.bee', 'config.local.json'), 'utf8'));
    assert(!('dogfood_repos' in overlayAfter), 'dogfood_repos must be gone from the overlay after unset --local');
  } finally {
    fs.rmSync(cfgRoot, { recursive: true, force: true });
  }
});

await check('state.advisor-ref examples run through the real dispatcher', async () => {
  // makeAdvisorRoot is a hoisted function declaration (defined in the advisor
  // block below); an active feature + a present digest file let the record
  // example succeed, and show round-trips it.
  const dir = makeAdvisorRoot({ mode: 'standard' });
  fs.writeFileSync(path.join(dir, 'consult.txt'), 'example consult digest body');
  await assertExampleOk('state.advisor-ref.record', { cwd: dir });
  const show = await assertExampleOk('state.advisor-ref.show', { cwd: dir });
  assert(JSON.parse(show.stdout).advisor_ref.advisor === 'gpt-5.6-sol', `show example returns the recorded advisor, got ${show.stdout}`);
});

// ─── state.validation-cache (spec #77 P1): the delta-validation evidence
// cache. Both registry examples run here, and the same fixtures prove the three
// properties this cache has to EARN before anyone may lean on it: a row carries
// forward only while the bytes it was proven from are untouched; any of the
// three anchors moving flips it stale with a named reason; and every cache
// defect degrades to full re-validation rather than to a skipped proof.
// makeAdvisorRoot (hoisted, defined in the advisor block below) already builds
// a repo with an active feature, an active decision, and a plan.md — exactly
// the three anchors this cache reuses from AO13.

function writeCacheRows(dir, rows) {
  const p = path.join(dir, 'rows.json');
  fs.writeFileSync(p, JSON.stringify(rows, null, 2), 'utf8');
  return p;
}

function readCacheFile(dir) {
  return JSON.parse(fs.readFileSync(path.join(dir, '.bee', 'validation-cache.json'), 'utf8'));
}

function writeCacheFileRaw(dir, body) {
  fs.writeFileSync(path.join(dir, '.bee', 'validation-cache.json'), body, 'utf8');
}

async function recordRows(dir, rows, slice = 1) {
  writeCacheRows(dir, rows);
  const r = await runBee(['state', 'validation-cache', 'record', '--slice', String(slice), '--rows-file', 'rows.json', '--json'], dir);
  assert(r.status === 0, `recording validation rows should succeed: ${r.stderr}`);
  return JSON.parse(r.stdout);
}

async function cacheCheck(dir, extra = []) {
  const r = await runBee(['state', 'validation-cache', 'check', '--json', ...extra], dir);
  assert(r.status === 0, `validation-cache check is a report, not a gate — it must exit 0: ${r.stderr} ${r.stdout}`);
  return JSON.parse(r.stdout);
}

// A repo with two file-backed rows recorded in slice 1, both fresh.
async function makeCacheRoot() {
  const dir = makeAdvisorRoot({ mode: 'standard' });
  fs.writeFileSync(path.join(dir, 'proof-a.mjs'), 'export const a = 1;\n');
  fs.writeFileSync(path.join(dir, 'proof-b.mjs'), 'export const b = 2;\n');
  await recordRows(dir, [
    { id: 'row-a', kind: 'matrix', claim: 'A holds', verdict: 'PASS', evidence: 'proof-a.mjs:1', sources: [{ path: 'proof-a.mjs' }] },
    { id: 'row-b', kind: 'matrix', claim: 'B holds', verdict: 'PASS', evidence: 'proof-b.mjs:1', sources: [{ path: 'proof-b.mjs' }] },
  ]);
  return dir;
}

await check('state.validation-cache examples run through the real dispatcher; an untouched row carries forward as cached with its slice', async () => {
  const dir = makeAdvisorRoot({ mode: 'standard' });
  fs.writeFileSync(path.join(dir, 'proof-a.mjs'), 'export const a = 1;\n');
  writeCacheRows(dir, [
    { id: 'row-a', kind: 'matrix', claim: 'A holds', verdict: 'PASS', evidence: 'proof-a.mjs:1', sources: [{ path: 'proof-a.mjs' }] },
  ]);
  await assertExampleOk('state.validation-cache.record', { cwd: dir });
  const verdict = JSON.parse((await assertExampleOk('state.validation-cache.check', { cwd: dir })).stdout);
  assert(verdict.degraded === false, `a freshly recorded cache must not be degraded, got ${JSON.stringify(verdict)}`);
  assert(verdict.revalidate === 'delta', `expected a delta re-validation, got ${verdict.revalidate}`);
  assert(verdict.counts.cached === 1 && verdict.counts.stale === 0, `expected 1 cached / 0 stale, got ${JSON.stringify(verdict.counts)}`);
  assert(verdict.cached[0].proven_in_slice === 1, `a carried-forward row must report the slice it was proven in, got ${JSON.stringify(verdict.cached[0])}`);
  assert(verdict.cached[0].evidence === 'proof-a.mjs:1', 'a carried-forward row keeps its original evidence verbatim');
  // The verb hashes the source itself — a caller can never assert a hash it did not earn.
  const stored = readCacheFile(dir).features.advtest.rows[0].sources[0];
  assert(stored.type === 'file' && /^[0-9a-f]{64}$/.test(stored.sha256), `record must stamp a real sha256, got ${JSON.stringify(stored)}`);
});

await check('spec #77 P1: touching ONE cited source file by one line flips exactly that row stale, and names the file', async () => {
  const dir = await makeCacheRoot();
  const before = await cacheCheck(dir);
  assert(before.counts.cached === 2 && before.counts.stale === 0, `baseline should be 2 cached / 0 stale, got ${JSON.stringify(before.counts)}`);
  // The one-line touch the acceptance criteria demand.
  fs.appendFileSync(path.join(dir, 'proof-a.mjs'), 'export const touched = true;\n');
  const after = await cacheCheck(dir);
  assert(after.degraded === false, 'a touched source is a stale row, not a degraded cache');
  assert(after.counts.cached === 1 && after.counts.stale === 1, `one touch must stale exactly one row, got ${JSON.stringify(after.counts)}`);
  assert(after.stale[0].id === 'row-a', `the touched row must be the stale one, got ${JSON.stringify(after.stale)}`);
  assert(after.cached[0].id === 'row-b', `the untouched row must still carry forward, got ${JSON.stringify(after.cached)}`);
  assert(
    after.stale[0].reasons.some((r) => r.includes('proof-a.mjs') && r.includes('sha256')),
    `the stale reason must name the moved source, got ${JSON.stringify(after.stale[0].reasons)}`,
  );
});

await check('spec #77 P1: a deleted source stales its row (absent sentinel never reads as unchanged)', async () => {
  const dir = await makeCacheRoot();
  fs.rmSync(path.join(dir, 'proof-a.mjs'));
  const after = await cacheCheck(dir);
  assert(after.counts.stale === 1 && after.stale[0].id === 'row-a', `deleting a source must stale its row, got ${JSON.stringify(after.counts)}`);
  assert(
    after.stale[0].reasons.some((r) => r.includes('no longer exists')),
    `expected a "no longer exists" reason, got ${JSON.stringify(after.stale[0].reasons)}`,
  );
});

await check('spec #77 P1: a newer active decision id stales EVERY row at once', async () => {
  const dir = await makeCacheRoot();
  fs.appendFileSync(
    path.join(dir, '.bee', 'decisions.jsonl'),
    `${JSON.stringify({ id: 'dec-2', type: 'decide', date: '2026-07-18T00:00:00.000Z', decision: 'later', scope: 'repo' })}\n`,
  );
  const after = await cacheCheck(dir);
  assert(after.counts.cached === 0 && after.counts.stale === 2, `a new decision must stale every row, got ${JSON.stringify(after.counts)}`);
  assert(after.revalidate === 'full', 'no surviving row means this is a full re-validation, and must be reported as one');
  assert(
    after.anchor_drift.some((r) => r.includes('decision')),
    `the drift must name the decision anchor, got ${JSON.stringify(after.anchor_drift)}`,
  );
});

await check('spec #77 P1: a re-shaped plan.md (sha change) stales EVERY row at once', async () => {
  const dir = await makeCacheRoot();
  fs.writeFileSync(path.join(dir, 'docs', 'history', 'advtest', 'plan.md'), '# plan\ncontent\nreshaped\n');
  const after = await cacheCheck(dir);
  assert(after.counts.cached === 0 && after.counts.stale === 2, `a re-shaped plan must stale every row, got ${JSON.stringify(after.counts)}`);
  assert(
    after.anchor_drift.some((r) => r.includes('plan.md')),
    `the drift must name plan.md, got ${JSON.stringify(after.anchor_drift)}`,
  );
});

// ─── Degradation: every one of these must land on full re-validation. This is
// the safety property, so each defect gets its own demonstration.

await check('degradation: a MISSING cache degrades to full re-validation, never to skipped validation', async () => {
  const dir = makeAdvisorRoot({ mode: 'standard' });
  const verdict = await cacheCheck(dir);
  assert(verdict.degraded === true && verdict.revalidate === 'full', `expected degraded/full, got ${JSON.stringify(verdict)}`);
  assert(verdict.counts.cached === 0 && verdict.cached.length === 0, 'a degraded cache must offer zero rows to carry forward');
  assert(/no validation cache recorded/.test(verdict.degrade_reason), `expected an absence reason, got ${verdict.degrade_reason}`);
});

await check('degradation: an UNREADABLE cache degrades to full re-validation', async () => {
  const dir = await makeCacheRoot();
  // A directory where the file belongs: readFileSync throws EISDIR for any uid,
  // unlike a chmod trick that a root-run CI would sail straight through.
  fs.rmSync(path.join(dir, '.bee', 'validation-cache.json'));
  fs.mkdirSync(path.join(dir, '.bee', 'validation-cache.json'));
  const verdict = await cacheCheck(dir);
  assert(verdict.degraded === true && verdict.revalidate === 'full', `expected degraded/full, got ${JSON.stringify(verdict)}`);
  assert(/unreadable/.test(verdict.degrade_reason), `expected an unreadable reason, got ${verdict.degrade_reason}`);
});

await check('degradation: a MALFORMED cache (invalid JSON) degrades to full re-validation', async () => {
  const dir = await makeCacheRoot();
  writeCacheFileRaw(dir, '{ this is not json');
  const verdict = await cacheCheck(dir);
  assert(verdict.degraded === true && verdict.revalidate === 'full', `expected degraded/full, got ${JSON.stringify(verdict)}`);
  assert(/malformed/.test(verdict.degrade_reason), `expected a malformed reason, got ${verdict.degrade_reason}`);
});

await check('degradation: an unknown cache VERSION degrades to full re-validation', async () => {
  const dir = await makeCacheRoot();
  const cache = readCacheFile(dir);
  cache.version = 999;
  writeCacheFileRaw(dir, JSON.stringify(cache));
  const verdict = await cacheCheck(dir);
  assert(verdict.degraded === true && verdict.revalidate === 'full', `expected degraded/full, got ${JSON.stringify(verdict)}`);
  assert(/version/.test(verdict.degrade_reason), `expected a version reason, got ${verdict.degrade_reason}`);
});

await check('degradation: a PARTIALLY-VALID cache (file parses, this feature\'s entry is broken) degrades to full re-validation', async () => {
  const dir = await makeCacheRoot();
  const cache = readCacheFile(dir);
  cache.features.advtest = { slice: 1, anchors: cache.features.advtest.anchors }; // rows array gone
  writeCacheFileRaw(dir, JSON.stringify(cache));
  const brokenRows = await cacheCheck(dir);
  assert(brokenRows.degraded === true && brokenRows.revalidate === 'full', `expected degraded/full, got ${JSON.stringify(brokenRows)}`);
  assert(/no rows array/.test(brokenRows.degrade_reason), `expected a rows-shape reason, got ${brokenRows.degrade_reason}`);

  // Same file, anchors missing instead: still full, never a partial trust.
  const cache2 = readCacheFile(dir);
  cache2.features.advtest = { slice: 1, rows: [] };
  writeCacheFileRaw(dir, JSON.stringify(cache2));
  const noAnchors = await cacheCheck(dir);
  assert(noAnchors.degraded === true && noAnchors.revalidate === 'full', `expected degraded/full, got ${JSON.stringify(noAnchors)}`);
  assert(/anchors/.test(noAnchors.degrade_reason), `expected an anchors reason, got ${noAnchors.degrade_reason}`);
});

await check('degradation: a cache holding no entry for THIS feature degrades to full re-validation', async () => {
  const dir = await makeCacheRoot();
  const cache = readCacheFile(dir);
  cache.features = { 'some-other-feature': cache.features.advtest };
  writeCacheFileRaw(dir, JSON.stringify(cache));
  const verdict = await cacheCheck(dir);
  assert(verdict.degraded === true && verdict.revalidate === 'full', `expected degraded/full, got ${JSON.stringify(verdict)}`);
});

await check('degradation: a row with NO stored hashes is stale — recorded hashless, and hand-written hashless', async () => {
  // Recorded through the real verb with an empty sources list.
  const dir = makeAdvisorRoot({ mode: 'standard' });
  await recordRows(dir, [{ id: 'row-bare', claim: 'no sources at all', verdict: 'PASS', evidence: 'trust me' }]);
  const bare = await cacheCheck(dir);
  assert(bare.counts.cached === 0 && bare.counts.stale === 1, `a sourceless row can never carry forward, got ${JSON.stringify(bare.counts)}`);
  assert(
    bare.stale[0].reasons.some((r) => r.includes('no sources')),
    `expected a no-sources reason, got ${JSON.stringify(bare.stale[0].reasons)}`,
  );

  // Hand-written cache whose file source carries a path but no sha256.
  const dir2 = await makeCacheRoot();
  const cache = readCacheFile(dir2);
  delete cache.features.advtest.rows[0].sources[0].sha256;
  writeCacheFileRaw(dir2, JSON.stringify(cache));
  const hashless = await cacheCheck(dir2);
  assert(hashless.degraded === false, 'a hashless ROW is a stale row, not a degraded file');
  assert(hashless.counts.stale === 1 && hashless.stale[0].id === 'row-a', `expected row-a stale, got ${JSON.stringify(hashless.counts)}`);
  assert(
    hashless.stale[0].reasons.some((r) => r.includes('no stored sha256')),
    `expected a no-stored-hash reason, got ${JSON.stringify(hashless.stale[0].reasons)}`,
  );
});

await check('command evidence: carries forward only against a freshly supplied output hash, and re-proves otherwise', async () => {
  const dir = makeAdvisorRoot({ mode: 'standard' });
  await recordRows(dir, [
    { id: 'row-cmd', claim: 'the command still passes', verdict: 'PASS', evidence: 'exit 0', sources: [{ command: 'node -e "process.exit(0)"', output_sha: 'abc123' }] },
  ]);
  // No --outputs-file: a read-only check cannot re-run the command, so the row
  // re-proves. Failing toward more validation is the whole contract.
  const unsupplied = await cacheCheck(dir);
  assert(unsupplied.counts.stale === 1, `an un-rerun command row must be stale, got ${JSON.stringify(unsupplied.counts)}`);
  assert(
    unsupplied.stale[0].reasons.some((r) => r.includes('not be re-run') || r.includes('not re-run')),
    `expected a not-re-run reason, got ${JSON.stringify(unsupplied.stale[0].reasons)}`,
  );

  // Matching fresh hash supplied: the row carries forward.
  const outputs = path.join(dir, 'outputs.json');
  fs.writeFileSync(outputs, JSON.stringify({ 'node -e "process.exit(0)"': 'abc123' }), 'utf8');
  const matched = await cacheCheck(dir, ['--outputs-file', outputs]);
  assert(matched.counts.cached === 1 && matched.counts.stale === 0, `a matching output hash must carry the row, got ${JSON.stringify(matched.counts)}`);

  // Different output: stale again.
  fs.writeFileSync(outputs, JSON.stringify({ 'node -e "process.exit(0)"': 'deadbeef' }), 'utf8');
  const changed = await cacheCheck(dir, ['--outputs-file', outputs]);
  assert(changed.counts.stale === 1, `a changed output hash must stale the row, got ${JSON.stringify(changed.counts)}`);

  // Unreadable --outputs-file must not be fatal — it just re-proves.
  const broken = await cacheCheck(dir, ['--outputs-file', path.join(dir, 'does-not-exist.json')]);
  assert(broken.counts.stale === 1, `an unreadable outputs file must re-prove, not crash, got ${JSON.stringify(broken.counts)}`);
});

await check('spec #77 P1: no TTL — a cache whose timestamps are ancient stays fresh while its content is unchanged', async () => {
  const dir = await makeCacheRoot();
  const cache = readCacheFile(dir);
  // Backdate every audit timestamp by years. Content-anchored staleness must
  // not notice, because no predicate reads a clock (AO13).
  cache.features.advtest.updated_at = '2001-01-01T00:00:00.000Z';
  for (const row of cache.features.advtest.rows) row.recorded_at = '2001-01-01T00:00:00.000Z';
  writeCacheFileRaw(dir, JSON.stringify(cache));
  const verdict = await cacheCheck(dir);
  assert(verdict.counts.cached === 2 && verdict.counts.stale === 0, `age alone must never stale a row, got ${JSON.stringify(verdict.counts)}`);
});

await check('record replaces a feature row set (a dropped row cannot linger) and leaves other features untouched', async () => {
  const dir = await makeCacheRoot();
  const before = readCacheFile(dir);
  before.features['other-feature'] = { slice: 9, updated_at: 'x', anchors: {}, rows: [] };
  writeCacheFileRaw(dir, JSON.stringify(before));
  await recordRows(dir, [
    { id: 'row-a', claim: 'A holds', verdict: 'PASS', evidence: 'proof-a.mjs:1', sources: [{ path: 'proof-a.mjs' }] },
  ], 2);
  const after = readCacheFile(dir);
  assert(after.features.advtest.rows.length === 1, `re-recording must replace the row set, got ${after.features.advtest.rows.length} rows`);
  assert(after.features.advtest.slice === 2, 'the recorded slice must advance');
  assert(after.features['other-feature'] !== undefined, 'recording one feature must never drop another');
});

await check('validation-cache record refuses a bad --slice and a missing --rows-file without writing', async () => {
  const dir = makeAdvisorRoot({ mode: 'standard' });
  writeCacheRows(dir, []);
  const badSlice = await runBee(['state', 'validation-cache', 'record', '--slice', 'soon', '--rows-file', 'rows.json'], dir);
  assert(badSlice.status !== 0, 'a non-integer slice must be refused');
  const missingFile = await runBee(['state', 'validation-cache', 'record', '--slice', '1', '--rows-file', 'nope.json'], dir);
  assert(missingFile.status !== 0, 'an unreadable rows-file must be refused');
  assert(!fs.existsSync(path.join(dir, '.bee', 'validation-cache.json')), 'a refused record must write nothing');
});

// ─── state.compact-log / state.compact-check (compaction-hardening D3): the
// helper floor's two thin CLI wrappers over lib/compaction.mjs. A dedicated
// isolated fixture repo, never rootState/root, so a fresh compaction.jsonl
// and a fresh anchor-missing predicate are exercised cleanly.

await check('state.compact-log + state.compact-check examples run through the real dispatcher', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-compact-example-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  writeState(dir, {
    ...defaultState(),
    phase: 'swarming',
    feature: 'compact-demo',
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });

  const logResult = await assertExampleOk('state.compact-log', { cwd: dir });
  const record = JSON.parse(logResult.stdout);
  assert(
    record.event === 'precompact' && record.session === 'sess-demo',
    `expected a precompact record for sess-demo, got ${logResult.stdout}`,
  );
  assert(
    record.compact_index === 1 && record.cell_compact_count === 0,
    `expected the first compaction's plain counts (D5), got ${logResult.stdout}`,
  );
  assert(fs.existsSync(path.join(dir, '.bee', 'logs', 'compaction.jsonl')), 'compaction.jsonl should now exist');

  const checkResult = await assertExampleOk('state.compact-check', { cwd: dir });
  const sweep = JSON.parse(checkResult.stdout);
  assert(sweep.session === 'sess-demo', `expected the session echoed back, got ${checkResult.stdout}`);
  assert(typeof sweep.ok === 'boolean' && Array.isArray(sweep.checks), `expected a {ok, checks[]} sweep shape, got ${checkResult.stdout}`);
  const anchorMissingCheck = sweep.checks.find((entry) => entry.name === 'anchor_missing');
  assert(anchorMissingCheck, `expected an "anchor_missing" check in compact-check's output, got ${JSON.stringify(sweep.checks)}`);
  assert(
    anchorMissingCheck.command === ANCHOR_NUDGE_COMMAND,
    `anchor_missing check must carry the exact D10 nudge command, got ${JSON.stringify(anchorMissingCheck)}`,
  );
});

await check('state.compact-check exits 0 even when it reports a mismatch (D13: reports, never blocks)', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-compact-mismatch-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  writeState(dir, {
    ...defaultState(),
    phase: 'swarming',
    feature: 'compact-mismatch-demo',
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });
  // No session record for "no-such-session" exists at all — the session_record
  // check must report false, and the process must still exit 0.
  const result = await runModuleWorker(BEE_MJS, {
    args: ['state', 'compact-check', '--session-id', 'no-such-session', '--json'],
    cwd: dir,
  });
  assert(result.status === 0, `compact-check must exit 0 on a reported mismatch, got status ${result.status}: ${result.stderr}`);
  const sweep = JSON.parse(result.stdout);
  assert(sweep.ok === false, `expected a mismatch to be reported (ok:false), got ${result.stdout}`);
  const sessionCheck = sweep.checks.find((entry) => entry.name === 'session_record');
  assert(sessionCheck && sessionCheck.ok === false, `expected a failed session_record check, got ${JSON.stringify(sweep.checks)}`);
});

await check('state.compact-log refuses (usage error, non-zero exit) on an unknown --event value', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-compact-badevent-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  const result = await runModuleWorker(BEE_MJS, {
    args: ['state', 'compact-log', '--event', 'bogus-event', '--session-id', 'sess-demo', '--json'],
    cwd: dir,
  });
  assert(result.status !== 0, `expected a non-zero exit for an invalid --event, got ${result.status}`);
  assert(!fs.existsSync(path.join(dir, '.bee', 'logs', 'compaction.jsonl')), 'an invalid --event must never write a record');
});

// ─── state.compact-capsule (compaction-hardening D3/D6/D19/D27): the third
// helper-floor wrapper. Same isolated-fixture pattern as the two above — the
// capsule reads state, onboarding, config, HANDOFF.json and the compaction log,
// so it needs a repo of its own rather than the shared rootState fixture.

await check('state.compact-capsule example renders the D6 capsule through the real dispatcher', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-compact-capsule-example-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  writeState(dir, {
    ...defaultState(),
    phase: 'swarming',
    mode: 'standard',
    feature: 'capsule-demo',
    next_action: 'cap the cell',
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });

  const result = await assertExampleOk('state.compact-capsule', { cwd: dir });
  const capsule = result.stdout;
  // D6 item 6: the `- Phase:` label is verbatim, and the capsule orients.
  assert(/^- Phase: swarming \| Mode: standard \| Feature: capsule-demo \| Lane: none$/m.test(capsule), `expected D6 item 6's phase line, got:\n${capsule}`);
  assert(capsule.includes('- Cell: none claimed by this session'), `with no claimed cell the capsule says so, got:\n${capsule}`);
  assert(capsule.includes('- Next action: cap the cell'), `D6 item 9 renders next_action, got:\n${capsule}`);
  // D7: a POINTER to the critical patterns, never the 10-line digest.
  assert(/^- Critical patterns: /m.test(capsule), `D7's pointer is never dropped, got:\n${capsule}`);
  assert(!capsule.includes('### Critical patterns (digest)'), 'D7: the capsule carries a pointer, never the digest');
  // D6's whole point: the startup-only sections stay out.
  for (const section of ['### Project map', '### Recent decisions']) {
    assert(!capsule.includes(section), `"${section}" is startup orientation and must never ride the capsule (D6)`);
  }
  // D19: the hook owns the anchor; the capsule is the preamble replacement only.
  assert(!capsule.includes('INTENT ANCHOR'), `D19: the capsule never renders the anchor, got:\n${capsule}`);
});

await check('state.compact-capsule carries the adoption-refusal reason for a planned-next handoff (D27)', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-compact-capsule-handoff-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  writeState(dir, {
    ...defaultState(),
    phase: 'swarming',
    feature: 'capsule-handoff-demo',
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });
  writeJsonAtomic(path.join(dir, '.bee', 'HANDOFF.json'), {
    kind: 'planned-next',
    phase: 'swarming',
    feature: 'capsule-handoff-demo',
    mode: 'standard',
    cells_in_flight: ['k-1'],
    next_action: 'start k-2',
    writer_session: 'sess-other',
  });

  const result = await runModuleWorker(BEE_MJS, {
    args: ['state', 'compact-capsule', '--session-id', 'sess-demo'],
    cwd: dir,
  });
  assert(result.status === 0, `compact-capsule must exit 0, got ${result.status}: ${result.stderr}`);
  assert(
    result.stdout.includes('### HANDOFF present — present it and WAIT — never auto-resume'),
    `D6 item 4: the wait heading is verbatim, got:\n${result.stdout}`,
  );
  // D27 is a CALL-SITE obligation: the verb must pass handoffOutcome through,
  // or a compacted session silently loses the explanation of the refusal.
  assert(
    /^- Adoption not applied: .+never auto-adopts/m.test(result.stdout),
    `D27: the verb must pass handoffOutcome so the refusal reason renders, got:\n${result.stdout}`,
  );
});

// ─── doctor (codex-native-runtime-v2 cnr2-13, D11): fail-closed runtime
// health report. A dedicated isolated fixture repo per test — doctor reads
// .codex/hooks.json, .claude/settings.json, hooks/*.mjs, and
// .bee/onboarding.json's recorded baseline hash, none of which the shared
// `root`/`root2` fixtures carry in the exact shape these tests need.

const DOCTOR_HOOKS_JSON = {
  hooks: {
    PreToolUse: [
      {
        matcher: 'spawn_agent',
        hooks: [{ type: 'command', command: 'exec node "$r"/hooks/bee-model-guard.mjs --source=repo' }],
      },
    ],
    Stop: [{ hooks: [{ type: 'command', command: 'exec node "$r"/hooks/bee-state-sync.mjs --source=repo' }] }],
  },
};

function buildDoctorFixture({ withHandlerFiles = true } = {}) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-doctor-test-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  fs.mkdirSync(path.join(dir, '.codex'), { recursive: true });
  fs.mkdirSync(path.join(dir, 'hooks'), { recursive: true });
  const hooksJsonPath = path.join(dir, '.codex', 'hooks.json');
  fs.writeFileSync(hooksJsonPath, `${JSON.stringify(DOCTOR_HOOKS_JSON, null, 2)}\n`, 'utf8');
  if (withHandlerFiles) {
    fs.writeFileSync(path.join(dir, 'hooks', 'bee-model-guard.mjs'), '// stub\n', 'utf8');
    fs.writeFileSync(path.join(dir, 'hooks', 'bee-state-sync.mjs'), '// stub\n', 'utf8');
  }
  fs.writeFileSync(path.join(dir, '.codex', 'config.toml'), 'approval_policy = "never"\n', 'utf8');
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
    managed: { repo_hooks: { '.codex/hooks.json': hashFile(hooksJsonPath) } },
    agents_sync: { files: [] },
  });
  // g22-4/D7: bee-render/2 with an empty skills[] — this fixture creates no
  // actual bee-* skill dirs under either root, so the deep audit's expected
  // set is trivially empty and skills_installed stays 'ok'/blocking, exactly
  // like the old shallow v1 check did for every OTHER doctor test in this
  // file that does not care about the skill-inventory audit itself (that
  // audit gets its own dedicated fixture matrix in scripts/test_conformance.mjs
  // scenarios 14/15 — deep-audit pass/missing/stray/drift, and legacy v1
  // warn-not-block — against the real .bee/bin/bee.mjs binary).
  fs.mkdirSync(path.join(dir, '.agents', 'skills'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.agents', 'skills', '.bee-render.json'), { schema: 'bee-render/2', target_runtime: 'codex', skills: [] });
  fs.mkdirSync(path.join(dir, '.claude', 'skills'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.claude', 'skills', '.bee-render.json'), { schema: 'bee-render/2', target_runtime: 'claude', skills: [] });
  writeJsonAtomic(path.join(dir, '.claude', 'settings.json'), {
    permissions: { defaultMode: 'bypassPermissions' },
    hooks: {
      SessionStart: [{ hooks: [{ type: 'command', command: 'node .bee/bin/hooks/bee-session-init.mjs' }] }],
      UserPromptSubmit: [{ hooks: [{ type: 'command', command: 'node .bee/bin/hooks/bee-prompt-context.mjs' }] }],
      PreToolUse: [
        { matcher: 'Edit|Write', hooks: [{ type: 'command', command: 'node .bee/bin/hooks/bee-write-guard.mjs' }] },
        { matcher: 'Agent|Task', hooks: [{ type: 'command', command: 'node .bee/bin/hooks/bee-model-guard.mjs' }] },
      ],
      PostToolUse: [{ hooks: [{ type: 'command', command: 'node .bee/bin/hooks/bee-tools-logger.mjs' }] }],
      Stop: [{ hooks: [{ type: 'command', command: 'node .bee/bin/hooks/bee-state-sync.mjs' }] }],
    },
  });
  fs.mkdirSync(path.join(dir, '.bee', 'bin', 'hooks'), { recursive: true });
  // bee-model-guard.mjs / bee-state-sync.mjs are the SAME two filenames the
  // codex fixture above references (GH #22 P1-1: doctor's dual-location
  // check resolves a codex handler at .bee/bin/hooks/<f> OR hooks/<f>) —
  // when withHandlerFiles is false, both locations must lack them for the
  // codex missing-handler assertion below to still hold; the other four
  // stay so claude's own handlers_resolvable row (a distinct check, not
  // exercised by that assertion) is unaffected.
  const beeBinHandlerFiles = withHandlerFiles
    ? ['bee-session-init.mjs', 'bee-prompt-context.mjs', 'bee-write-guard.mjs', 'bee-model-guard.mjs', 'bee-tools-logger.mjs', 'bee-state-sync.mjs']
    : ['bee-session-init.mjs', 'bee-prompt-context.mjs', 'bee-write-guard.mjs', 'bee-tools-logger.mjs'];
  for (const f of beeBinHandlerFiles) {
    fs.writeFileSync(path.join(dir, '.bee', 'bin', 'hooks', f), '// stub\n', 'utf8');
  }
  return dir;
}

await check('doctor: ok fixture — checkable codex rows pass ok, mechanical-green codex reaches degraded (no attestation), claude reaches overall_status ready', async () => {
  const dir = buildDoctorFixture();
  try {
    const codexResult = await assertExampleOk('doctor', { exampleIndex: 0, cwd: dir });
    const codex = JSON.parse(codexResult.stdout);
    assert(codex.runtime === 'codex', `expected runtime codex, got ${JSON.stringify(codex)}`);
    const byRow = Object.fromEntries(codex.rows.map((r) => [r.row, r]));
    assert(byRow.hooks_file_present.status === 'ok', `hooks_file_present should be ok, got ${JSON.stringify(byRow.hooks_file_present)}`);
    assert(byRow.capability_baseline_match.status === 'ok', `capability_baseline_match should be ok on a matching baseline, got ${JSON.stringify(byRow.capability_baseline_match)}`);
    assert(byRow.hook_handlers_resolvable.status === 'ok', `hook_handlers_resolvable should be ok when every handler file exists, got ${JSON.stringify(byRow.hook_handlers_resolvable)}`);
    // D4 three-state: mechanical rows are all ok, but codex's structurally-
    // unknown trust rows still `degrades` readiness with no attestation
    // recorded — 'degraded', never a bare "ready" from file presence alone,
    // and never 'blocked' either since nothing mechanical failed.
    assert(codex.overall_status === 'degraded', `codex overall_status must be degraded (mechanical green, trust rows unknown, no attestation), got ${codex.overall_status}`);
    assert(codex.reasons.some((r) => r.startsWith('hooks_discovered:')), `reasons must name the degrading trust rows, got ${JSON.stringify(codex.reasons)}`);
    assert(codex.reasons.some((r) => r.startsWith('no_attestation:')), `reasons must name no_attestation, got ${JSON.stringify(codex.reasons)}`);
    assert(codex.attestation && codex.attestation.status === 'invalid' && codex.attestation.reason === 'no_attestation', `attestation summary must report invalid/no_attestation, got ${JSON.stringify(codex.attestation)}`);

    const claudeResult = await assertExampleOk('doctor', { exampleIndex: 1, cwd: dir });
    const claude = JSON.parse(claudeResult.stdout);
    assert(claude.runtime === 'claude', `expected runtime claude, got ${JSON.stringify(claude)}`);
    assert(claude.overall_status === 'ready', `claude should reach ready on a fully-wired fixture with no blocking rows, got ${claude.overall_status}: ${JSON.stringify(claude.rows)}`);
    assert(!('attestation' in claude), `claude has no attestation model and must not carry an attestation field, got ${JSON.stringify(claude)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('doctor attest: a valid attestation over a mechanical-green fixture reaches ready', async () => {
  const dir = buildDoctorFixture();
  try {
    const attestResult = await assertExampleOk('doctor.attest', { cwd: dir });
    const attested = JSON.parse(attestResult.stdout);
    assert(attested.ok === true && attested.attestation && typeof attested.attestation.hooks_file_sha256 === 'string', `doctor attest must record an attestation, got ${attestResult.stdout}`);
    assert(fs.existsSync(path.join(dir, '.bee', 'doctor-attest.json')), 'doctor attest must write .bee/doctor-attest.json');

    const result = await runModuleWorker(BEE_MJS, { args: ['doctor', '--runtime', 'codex', '--json'], cwd: dir });
    assert(result.status === 0, `doctor must not throw after attesting, got exit ${result.status}: ${result.stderr}`);
    const parsed = JSON.parse(result.stdout);
    assert(parsed.overall_status === 'ready', `a valid attestation over a mechanical-green fixture must reach ready, got ${parsed.overall_status}: ${JSON.stringify(parsed.reasons)}`);
    assert(parsed.attestation && parsed.attestation.status === 'valid', `attestation summary must report valid, got ${JSON.stringify(parsed.attestation)}`);
    assert(parsed.reasons.length === 0, `ready must carry no reasons, got ${JSON.stringify(parsed.reasons)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('doctor attest: flipping .codex/hooks.json after attesting goes stale (hash_changed) -> degraded', async () => {
  const dir = buildDoctorFixture();
  try {
    await assertExampleOk('doctor.attest', { cwd: dir });
    // A real post-attestation drift — mutate the live file AFTER attesting.
    // Keeps the same hook commands (so hook_handlers_resolvable/capability_
    // baseline_match — re-baselined below — both stay mechanically ok) and
    // only adds a harmless marker field, isolating the assertion to the
    // attestation's own hash leg rather than the mechanical rows.
    fs.writeFileSync(
      path.join(dir, '.codex', 'hooks.json'),
      `${JSON.stringify({ ...DOCTOR_HOOKS_JSON, _post_attest_marker: true }, null, 2)}\n`,
      'utf8',
    );
    writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), {
      schema_version: '1.0',
      bee_version: '0.1.0',
      managed: { repo_hooks: { '.codex/hooks.json': hashFile(path.join(dir, '.codex', 'hooks.json')) } },
      agents_sync: { files: [] },
    });
    const result = await runModuleWorker(BEE_MJS, { args: ['doctor', '--runtime', 'codex', '--json'], cwd: dir });
    assert(result.status === 0, `doctor must not throw on a stale attestation, got exit ${result.status}: ${result.stderr}`);
    const parsed = JSON.parse(result.stdout);
    assert(parsed.overall_status === 'degraded', `a stale (hash-changed) attestation must degrade, not block or ready, got ${parsed.overall_status}`);
    assert(parsed.attestation && parsed.attestation.status === 'invalid' && parsed.attestation.reason === 'hash_changed', `attestation summary must name hash_changed, got ${JSON.stringify(parsed.attestation)}`);
    assert(parsed.reasons.some((r) => r.startsWith('hash_changed:')), `reasons must name hash_changed, got ${JSON.stringify(parsed.reasons)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('doctor attest: --runtime claude is refused (no attestation model)', async () => {
  const dir = buildDoctorFixture();
  try {
    const result = await runModuleWorker(BEE_MJS, { args: ['doctor', 'attest', '--runtime', 'claude', '--json'], cwd: dir });
    assert(result.status !== 0, `doctor attest --runtime claude must be refused, got exit ${result.status}: ${result.stdout}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('doctor: missing .codex/hooks.json -> blocked (mechanical, not merely degraded)', async () => {
  const dir = buildDoctorFixture();
  try {
    fs.rmSync(path.join(dir, '.codex', 'hooks.json'));
    const result = await runModuleWorker(BEE_MJS, { args: ['doctor', '--runtime', 'codex', '--json'], cwd: dir });
    assert(result.status === 0, `doctor must not throw on a missing hooks file, got exit ${result.status}: ${result.stderr}`);
    const parsed = JSON.parse(result.stdout);
    assert(parsed.overall_status === 'blocked', `a missing mechanical hooks file must block readiness outright, got ${parsed.overall_status}`);
    assert(parsed.reasons.some((r) => r.startsWith('hooks_file_present:')), `reasons must name hooks_file_present, got ${JSON.stringify(parsed.reasons)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('doctor: version-mismatch wording — a live codex --version other than the probed one reports unprobed_version, never the probed conclusions', async () => {
  const dir = buildDoctorFixture();
  const stubDir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-doctor-codex-stub-'));
  try {
    // A tiny fake "codex" binary ahead on PATH that reports a version other
    // than PROBED_CODEX_VERSION ('0.145.0') — proves the wording switches
    // without needing an actually-different codex install on this machine.
    const stubPath = path.join(stubDir, 'codex');
    fs.writeFileSync(stubPath, '#!/bin/sh\necho "codex-cli 9.9.9"\n', { mode: 0o755 });
    const result = await runModuleWorker(BEE_MJS, {
      args: ['doctor', '--runtime', 'codex', '--json'],
      cwd: dir,
      env: { ...process.env, PATH: `${stubDir}${path.delimiter}${process.env.PATH || ''}` },
    });
    assert(result.status === 0, `doctor must not throw on an unprobed codex version, got exit ${result.status}: ${result.stderr}`);
    const parsed = JSON.parse(result.stdout);
    const row = parsed.rows.find((r) => r.row === 'hooks_discovered');
    assert(row.evidence.includes('unprobed_version'), `evidence must carry the unprobed_version token, got ${row.evidence}`);
    assert(!row.evidence.includes('0.145.0 exposes no machine-readable'), `evidence must not assert the probed-version conclusion verbatim, got ${row.evidence}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
    fs.rmSync(stubDir, { recursive: true, force: true });
  }
});

await check('doctor: missing hook handler file -> hook_handlers_resolvable warns and names the missing file', async () => {
  const dir = buildDoctorFixture({ withHandlerFiles: false });
  try {
    const result = await runModuleWorker(BEE_MJS, { args: ['doctor', '--runtime', 'codex', '--json'], cwd: dir });
    assert(result.status === 0, `doctor must not throw, got exit ${result.status}: ${result.stderr}`);
    const parsed = JSON.parse(result.stdout);
    const row = parsed.rows.find((r) => r.row === 'hook_handlers_resolvable');
    assert(row.status === 'warn', `expected hook_handlers_resolvable warn on missing handler files, got ${JSON.stringify(row)}`);
    assert(row.evidence.includes('bee-model-guard.mjs') || row.evidence.includes('bee-state-sync.mjs'), `evidence should name a missing handler, got ${row.evidence}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('doctor: codex binary absent from PATH -> codex_version warns instead of crashing', async () => {
  const dir = buildDoctorFixture();
  try {
    // An empty PATH inside the isolated worker's own env cannot resolve the
    // "codex" binary, regardless of what is actually installed on the
    // machine running this suite — the parent process's PATH is untouched.
    const result = await runModuleWorker(BEE_MJS, {
      args: ['doctor', '--runtime', 'codex', '--json'],
      cwd: dir,
      env: { ...process.env, PATH: '' },
    });
    assert(result.status === 0, `doctor must not throw when codex is absent, got exit ${result.status}: ${result.stderr}`);
    const parsed = JSON.parse(result.stdout);
    const row = parsed.rows.find((r) => r.row === 'codex_version');
    assert(row.status === 'warn', `expected codex_version warn when the binary cannot be found, got ${JSON.stringify(row)}`);
    assert(row.value === null, `codex_version value should be null when unresolved, got ${JSON.stringify(row.value)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('doctor: codex trust/discovery rows are always present, unknown, and degrading (D4 re-class: no longer blocking) — never inferred from file presence', async () => {
  const dir = buildDoctorFixture();
  try {
    const result = await runModuleWorker(BEE_MJS, { args: ['doctor', '--runtime', 'codex', '--json'], cwd: dir });
    const parsed = JSON.parse(result.stdout);
    for (const rowName of ['hooks_discovered', 'hooks_trusted', 'project_trust', 'pending_hook_review']) {
      const row = parsed.rows.find((r) => r.row === rowName);
      assert(row, `row "${rowName}" must always be present on --runtime codex`);
      assert(row.status === 'unknown', `${rowName} must stay unknown, got ${row.status}`);
      // D4 re-class: these rows carry `degrades: true`, never `blocking`
      // anymore — a bare unknown trust state degrades readiness (recoverable
      // via "doctor attest"), it no longer blocks it outright.
      assert(row.degrades === true, `${rowName} must be marked degrades, got ${JSON.stringify(row)}`);
      assert(!row.blocking, `${rowName} must no longer be marked blocking, got ${JSON.stringify(row)}`);
      assert(typeof row.degraded_reason === 'string' && row.degraded_reason.length > 0, `${rowName} must carry a degraded_reason, got ${JSON.stringify(row)}`);
    }
    assert(parsed.overall_status === 'degraded', 'unattested degrading trust rows must degrade (not block, not ready)');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('doctor: custom_agents verdict is version-scoped, never a bare "unsupported"', async () => {
  const dir = buildDoctorFixture();
  try {
    const result = await runModuleWorker(BEE_MJS, { args: ['doctor', '--runtime', 'codex', '--json'], cwd: dir });
    const parsed = JSON.parse(result.stdout);
    const row = parsed.rows.find((r) => r.row === 'custom_agents');
    assert(row.status === 'unsupported', `expected unsupported, got ${JSON.stringify(row)}`);
    assert(row.evidence.includes('0.145.0'), `custom_agents evidence must cite the probed version, got ${row.evidence}`);
    assert(row.evidence.toLowerCase().includes('version-scoped') || row.evidence.toLowerCase().includes('other versions'), `custom_agents evidence must scope the verdict to the probed version, got ${row.evidence}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('doctor: performs zero writes, even with an unwritable cache directory (read-only sandbox)', async () => {
  const dir = buildDoctorFixture();
  try {
    const cacheDir = path.join(dir, '.bee', 'cache');
    fs.mkdirSync(cacheDir, { recursive: true });
    const cacheFile = path.join(cacheDir, 'manifest-hash.json');
    fs.chmodSync(cacheDir, 0o500); // read+execute only: writes/creates inside must fail
    try {
      const result = await runModuleWorker(BEE_MJS, { args: ['doctor', '--runtime', 'codex', '--json'], cwd: dir });
      assert(result.status === 0, `doctor must not crash under an unwritable cache dir, got exit ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
      assert(!fs.existsSync(cacheFile), `doctor must never create ${cacheFile} — it is read-only FOR REAL, not merely best-effort`);
    } finally {
      fs.chmodSync(cacheDir, 0o700);
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('doctor: a mutating command still persists the manifest-hash cache (best-effort, not weakened)', async () => {
  const dir = buildDoctorFixture();
  try {
    const cacheFile = path.join(dir, '.bee', 'cache', 'manifest-hash.json');
    assert(!fs.existsSync(cacheFile), 'precondition: no cache file yet');
    const result = await runModuleWorker(BEE_MJS, { args: ['status', '--json'], cwd: dir });
    assert(result.status === 0, `status must succeed, got ${result.status}: ${result.stderr}`);
    assert(fs.existsSync(cacheFile), 'a non-doctor command must still persist the manifest-hash cache');
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('doctor: --json shape is stable (runtime, overall_status, rows[], reasons[]) for both runtimes', async () => {
  const dir = buildDoctorFixture();
  try {
    for (const runtime of ['codex', 'claude']) {
      const result = await runModuleWorker(BEE_MJS, { args: ['doctor', '--runtime', runtime, '--json'], cwd: dir });
      assert(result.status === 0, `doctor --runtime ${runtime} must exit 0, got ${result.status}: ${result.stderr}`);
      const parsed = JSON.parse(result.stdout);
      assert(parsed.runtime === runtime, `runtime field mismatch, got ${JSON.stringify(parsed)}`);
      assert(['ready', 'degraded', 'blocked'].includes(parsed.overall_status), `overall_status must be ready|degraded|blocked, got ${parsed.overall_status}`);
      assert(Array.isArray(parsed.rows) && parsed.rows.length > 0, `rows must be a non-empty array, got ${JSON.stringify(parsed.rows)}`);
      for (const row of parsed.rows) {
        assert(typeof row.row === 'string' && row.row, `every row needs a name, got ${JSON.stringify(row)}`);
        assert(['ok', 'warn', 'unknown', 'unsupported'].includes(row.status), `row "${row.row}" has an unrecognized status "${row.status}"`);
        assert(typeof row.evidence === 'string' && row.evidence, `row "${row.row}" must carry non-empty evidence`);
      }
      assert(Array.isArray(parsed.reasons), `reasons must be an array, got ${JSON.stringify(parsed.reasons)}`);
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('doctor: an unknown --runtime is refused, never silently defaulted', async () => {
  const dir = buildDoctorFixture();
  try {
    const result = await runModuleWorker(BEE_MJS, { args: ['doctor', '--runtime', 'windows', '--json'], cwd: dir });
    assert(result.status !== 0, `an unrecognized runtime must be refused, got exit ${result.status}: ${result.stdout}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('doctor: hook_sources names the both-present dual-source risk (D5, #54 item 8) — never blocking, active stays unknown, distinguishes claude-hooks.json from hooks.json', async () => {
  // Single-source baseline: buildDoctorFixture() only writes .codex/hooks.json
  // (repo fallback) — no hooks/hooks.json (plugin projection) checked in — so
  // the dual-source sentence must NOT appear yet.
  const singleSourceDir = buildDoctorFixture();
  try {
    const result = await runModuleWorker(BEE_MJS, { args: ['doctor', '--runtime', 'codex', '--json'], cwd: singleSourceDir });
    assert(result.status === 0, `doctor must not throw on the single-source fixture, got exit ${result.status}: ${result.stderr}`);
    const parsed = JSON.parse(result.stdout);
    const row = parsed.rows.find((r) => r.row === 'hook_sources');
    assert(row, `hook_sources row must be present, got ${JSON.stringify(parsed.rows.map((r) => r.row))}`);
    assert(row.status === 'ok', `hook_sources should be ok with .codex/hooks.json present, got ${JSON.stringify(row)}`);
    assert(row.value.configured.plugin_projection_checked_in === false, `plugin projection must read false when hooks/hooks.json is absent, got ${JSON.stringify(row.value)}`);
    assert(row.value.active === 'unknown', `active must stay unknown even in the single-source case, got ${JSON.stringify(row.value)}`);
    assert(!row.evidence.includes('two hook sources exist'), `single-source evidence must not carry the dual-source sentence, got: ${row.evidence}`);
    assert(!row.blocking, `hook_sources must never become a blocking row, got ${JSON.stringify(row)}`);
  } finally {
    fs.rmSync(singleSourceDir, { recursive: true, force: true });
  }

  // Both-present: add the plugin projection (hooks/hooks.json) AND the
  // plugin.json-declared Claude manifest (hooks/claude-hooks.json) alongside
  // the repo fallback (.codex/hooks.json, written by buildDoctorFixture()).
  const bothPresentDir = buildDoctorFixture();
  try {
    fs.mkdirSync(path.join(bothPresentDir, 'packages', 'bee', 'hooks'), { recursive: true });
    fs.writeFileSync(path.join(bothPresentDir, 'packages', 'bee', 'hooks', 'hooks.json'), `${JSON.stringify(DOCTOR_HOOKS_JSON, null, 2)}\n`, 'utf8');
    fs.writeFileSync(path.join(bothPresentDir, 'packages', 'bee', 'hooks', 'claude-hooks.json'), `${JSON.stringify(DOCTOR_HOOKS_JSON, null, 2)}\n`, 'utf8');
    const result = await runModuleWorker(BEE_MJS, { args: ['doctor', '--runtime', 'codex', '--json'], cwd: bothPresentDir });
    assert(result.status === 0, `doctor must not throw on the both-present fixture, got exit ${result.status}: ${result.stderr}`);
    const parsed = JSON.parse(result.stdout);
    const row = parsed.rows.find((r) => r.row === 'hook_sources');
    assert(row, `hook_sources row must be present, got ${JSON.stringify(parsed.rows.map((r) => r.row))}`);
    // Verdict semantics stay conservative — still ok/warn as today, never a
    // new blocking row, and active is never inferred from presence.
    assert(row.status === 'ok', `hook_sources should stay ok (verdict semantics unchanged) with both sources present, got ${JSON.stringify(row)}`);
    assert(!row.blocking, `hook_sources must never become a blocking row even in the both-present state, got ${JSON.stringify(row)}`);
    assert(row.value.active === 'unknown', `active must stay unknown (never inferred from presence) in the both-present state, got ${JSON.stringify(row.value)}`);
    assert(row.value.configured.repo === true, `configured.repo must be true, got ${JSON.stringify(row.value.configured)}`);
    assert(row.value.configured.plugin_projection_checked_in === true, `configured.plugin_projection_checked_in must be true, got ${JSON.stringify(row.value.configured)}`);
    assert(row.value.configured.claude_hooks_manifest_checked_in === true, `configured.claude_hooks_manifest_checked_in must be true, got ${JSON.stringify(row.value.configured)}`);
    assert(row.evidence.includes('two hook sources exist'), `both-present evidence must name the dual-source state, got: ${row.evidence}`);
    assert(row.evidence.includes('hook-source-exclusivity B14'), `both-present evidence must cite the exactly-one-active law (hook-source-exclusivity B14), got: ${row.evidence}`);
    assert(row.evidence.includes('capability matrix row B1'), `both-present evidence must name the current premise (capability matrix row B1), got: ${row.evidence}`);
    assert(row.evidence.includes('re-proved') && row.evidence.includes('probed codex version changes'), `both-present evidence must state the premise must be re-proved when the probed version changes, got: ${row.evidence}`);
    assert(row.evidence.includes('hooks/claude-hooks.json') && row.evidence.includes('plugin.json-declared'), `evidence must distinguish hooks/claude-hooks.json from hooks/hooks.json (#54 item 8), got: ${row.evidence}`);
  } finally {
    fs.rmSync(bothPresentDir, { recursive: true, force: true });
  }
});

// ─── recovery.* (transcript-recovery-2, D1-D6): CLI verbs `recovery scan` /
// `recovery window`, and the fail-open `recovery` status block. Own isolated
// repo (own .bee) + own transcript fixtures written under the SAME fake
// CLAUDE_CONFIG_DIR set at the top of this file (line 111), keyed by this
// repo's own encoded project dir — never touches root/rootState/root2's own
// fixtures or the real ~/.claude/projects. Placed BEFORE the "every registry
// entry had its example executed" coverage check just below (executedNames
// is only populated by assertExampleOk/runExample, which the two recovery
// examples below call) — after that check, recovery.scan/recovery.window
// would read as never-exercised registry entries.

const rootRecovery = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-recovery-test-'));
fs.mkdirSync(path.join(rootRecovery, '.bee'), { recursive: true });
writeJsonAtomic(path.join(rootRecovery, '.bee', 'onboarding.json'), {
  schema_version: '1.0',
  bee_version: '0.1.0',
});

function recoveryTranscriptDir() {
  return path.join(process.env.CLAUDE_CONFIG_DIR, 'projects', encodeProjectDir(rootRecovery));
}

function writeRecoveryTranscript(sessionId, events) {
  const dir = recoveryTranscriptDir();
  fs.mkdirSync(dir, { recursive: true });
  const file = path.join(dir, `${sessionId}.jsonl`);
  fs.writeFileSync(file, `${events.map((e) => JSON.stringify(e)).join('\n')}\n`, 'utf8');
  return file;
}

function writeRecoverySession(sessionId, { started_at, last_heartbeat, lane } = {}) {
  const dir = path.join(rootRecovery, '.bee', 'sessions');
  fs.mkdirSync(dir, { recursive: true });
  const rec = { id: sessionId, started_at, last_heartbeat };
  if (lane) rec.lane = lane;
  writeJsonAtomic(path.join(dir, `${sessionId}.json`), rec);
}

const RECOVERY_STALE_HEARTBEAT = new Date(Date.now() - 1000 * 1000).toISOString(); // 1000s old > the 900s law
// Ends mid-turn (no stop_hook_summary/turn_duration/last-prompt trio) —
// exactly what test_recovery.mjs's own dirtyEndEvents() fixture represents,
// restated here so this CLI-level test carries no import from the lib test.
const RECOVERY_DIRTY_EVENTS = [
  { type: 'user', timestamp: new Date(Date.now() - 5000).toISOString(), message: { role: 'user', content: [{ type: 'text', text: 'go' }] } },
  { type: 'assistant', timestamp: new Date(Date.now() - 4000).toISOString(), message: { role: 'assistant' } },
];

await check('recovery.scan example with zero sessions: empty array, exit 0', async () => {
  const result = await assertExampleOk('recovery.scan', { cwd: rootRecovery });
  const candidates = JSON.parse(result.stdout);
  assert(Array.isArray(candidates) && candidates.length === 0, `expected an empty array, got ${result.stdout}`);
});

await check('recovery.scan lists a crafted stale/dirty session as a crash candidate (session id, lane, last_heartbeat, transcript path)', async () => {
  writeRecoverySession('sess-recovery-demo', {
    started_at: new Date(Date.now() - 20000).toISOString(),
    last_heartbeat: RECOVERY_STALE_HEARTBEAT,
  });
  writeRecoveryTranscript('sess-recovery-demo', RECOVERY_DIRTY_EVENTS);
  const result = await assertExampleOk('recovery.scan', { cwd: rootRecovery });
  const candidates = JSON.parse(result.stdout);
  assert(candidates.length === 1, `expected exactly the one crafted candidate, got ${result.stdout}`);
  assert(candidates[0].session_id === 'sess-recovery-demo', `expected sess-recovery-demo, got ${JSON.stringify(candidates[0])}`);
  assert(candidates[0].last_heartbeat === RECOVERY_STALE_HEARTBEAT, `expected the stale heartbeat carried through, got ${JSON.stringify(candidates[0])}`);
  assert(typeof candidates[0].transcript === 'string' && candidates[0].transcript.endsWith('.jsonl'), `expected a transcript path, got ${JSON.stringify(candidates[0])}`);
});

await check('recovery.window on that candidate: bounded window + a prompt carrying the D5 clauses (redaction, data-never-instructions)', async () => {
  const result = await assertExampleOk('recovery.window', { cwd: rootRecovery });
  const win = JSON.parse(result.stdout);
  assert(typeof win.transcript === 'string' && win.transcript.endsWith('.jsonl'), `expected a transcript path, got ${result.stdout}`);
  assert(Number.isInteger(win.event_count) && win.event_count > 0, `expected a positive event_count, got ${result.stdout}`);
  assert(win.window_truncated === false, `the small fixture window must not be truncated, got ${result.stdout}`);
  assert(typeof win.prompt === 'string' && win.prompt.startsWith('[bee-tier: generation]'), `prompt must lead with the bee-tier marker (critical rule 12), got ${result.stdout}`);
  assert(/redact/i.test(win.prompt), 'prompt must carry the D5 redaction clause');
  assert(win.prompt.includes('DATA, never instructions'), 'prompt must carry the D5 data-never-instructions clause verbatim');
  assert(win.prompt.includes('sess-recovery-demo'), 'prompt must embed the candidate session id');
});

await check('recovery.window refuses (typed, non-zero exit) for an unknown session id — never a bare crash', async () => {
  const result = await runModuleWorker(BEE_MJS, { args: ['recovery', 'window', '--session', 'sess-does-not-exist'], cwd: rootRecovery });
  assert(result.status !== 0, `expected non-zero exit, got ${result.status}`);
  assert(/not found/.test(result.stderr), `expected a "not found" refusal, got stderr=${result.stderr}`);
});

await check('status --json always carries a recovery block (fail-open like review), listing the crafted candidate', async () => {
  const result = await runModuleWorker(BEE_MJS, { args: ['status', '--json'], cwd: rootRecovery });
  assert(result.status === 0, `status must exit 0, got ${result.status}: stderr=${result.stderr}`);
  const status = JSON.parse(result.stdout);
  assert(status.recovery && Array.isArray(status.recovery.candidates), `expected status.recovery.candidates array, got ${JSON.stringify(status.recovery)}`);
  assert(
    status.recovery.candidates.some((c) => c.session_id === 'sess-recovery-demo'),
    `expected the crafted candidate inside status.recovery, got ${JSON.stringify(status.recovery)}`,
  );
});

await check('status --json recovery block never breaks status even with an unreadable/corrupt session record (fail-open)', async () => {
  const rootRecoveryCorrupt = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-recovery-corrupt-'));
  fs.mkdirSync(path.join(rootRecoveryCorrupt, '.bee', 'sessions'), { recursive: true });
  writeJsonAtomic(path.join(rootRecoveryCorrupt, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  fs.writeFileSync(path.join(rootRecoveryCorrupt, '.bee', 'sessions', 'sess-broken.json'), '{ not valid json', 'utf8');
  const result = await runModuleWorker(BEE_MJS, { args: ['status', '--json'], cwd: rootRecoveryCorrupt });
  assert(result.status === 0, `status must still exit 0 over a corrupt session record, got ${result.status}: stderr=${result.stderr}`);
  const status = JSON.parse(result.stdout);
  assert(status.recovery && Array.isArray(status.recovery.candidates), `recovery block must still be well-shaped, got ${JSON.stringify(status.recovery)}`);
  assert(status.recovery.candidates.length === 0, `a corrupt-only session record must never surface as a candidate, got ${JSON.stringify(status.recovery)}`);
});

// ─── knowledge.check (okf-foundation S1): the OKF bundle checker's registry
// example runs against the first fixture repo, which has no docs/knowledge/ —
// an empty bundle is OK by contract (D23), so the example must exit 0 with
// the D13 {okf,profile,counts} shape and zeroed findings.

await check('knowledge.check example: an empty bundle (no docs/knowledge/) exits 0 with the D13 shape', async () => {
  const result = await assertExampleOk('knowledge.check', { cwd: root });
  const report = JSON.parse(result.stdout);
  assert(report.okf && Array.isArray(report.okf.errors) && report.okf.errors.length === 0, `expected okf.errors [], got ${result.stdout}`);
  assert(report.profile && Array.isArray(report.profile.warnings) && report.profile.warnings.length === 0, `expected profile.warnings [], got ${result.stdout}`);
  assert(report.counts && report.counts.concepts === 0 && report.counts.files === 0, `expected zeroed counts, got ${result.stdout}`);
});

// ─── knowledge.index / knowledge.list (okf-foundation S3, cell okf-4): run
// against their OWN fresh fixture repo, never `root` — the knowledge.check
// example above asserts root has zero bundle files, and `knowledge index`
// would create docs/knowledge/index.md there.

const rootKnowledge = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-knowledge-'));
fs.mkdirSync(path.join(rootKnowledge, '.bee'), { recursive: true });
writeJsonAtomic(path.join(rootKnowledge, '.bee', 'onboarding.json'), {
  schema_version: '1.0',
  bee_version: '0.1.0',
});

await check('knowledge.index example: generates the root index on an empty bundle (D21)', async () => {
  const result = await assertExampleOk('knowledge.index', { cwd: rootKnowledge });
  const report = JSON.parse(result.stdout);
  assert(Array.isArray(report.written) && report.written.includes('docs/knowledge/index.md'), `expected written to include docs/knowledge/index.md, got ${result.stdout}`);
  assert(report.count === 1, `an empty bundle must render the root index only, got ${result.stdout}`);
  assert(fs.existsSync(path.join(rootKnowledge, 'docs', 'knowledge', 'index.md')), 'the root index must exist on disk after the example');
});

await check('knowledge.index --check example (examples[1]) passes right after a render, with the {checked,stale,drift} shape', async () => {
  const { entry, result } = await runExample('knowledge.index', { exampleIndex: 1, cwd: rootKnowledge });
  assert(result.status === 0, `${entry.name} example "${entry.examples[1]}" exited ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  const report = JSON.parse(result.stdout);
  assert(report.drift === false && Array.isArray(report.stale) && report.stale.length === 0, `a fresh render must have zero drift, got ${result.stdout}`);
  assert(report.checked === 1, `expected 1 checked index file, got ${result.stdout}`);
});

await check('knowledge.list example: {concepts,count} rows carry path,id,type,lifecycle,title and never content (D15)', async () => {
  const result = await assertExampleOk('knowledge.list', { cwd: rootKnowledge });
  const report = JSON.parse(result.stdout);
  assert(Array.isArray(report.concepts) && report.count === 0, `the empty fixture bundle has zero concepts (index.md is reserved, never a row), got ${result.stdout}`);
});

// ─── knowledge.context (okf-foundation S5, cell okf-7): the registry example
// is `bee knowledge context --work okf-foundation --budget 20000 --json`, so
// the fixture bundle must carry a work item with that bee.id before it runs.
// Seeded HERE (after the list example asserted the bundle was empty) rather
// than against this checkout's real docs/knowledge/ — the isolation law at the
// top of this file holds for every example, read-only ones included.

fs.mkdirSync(path.join(rootKnowledge, 'docs', 'knowledge', 'work', 'okf-foundation'), { recursive: true });
fs.mkdirSync(path.join(rootKnowledge, 'docs', 'knowledge', 'patterns'), { recursive: true });
fs.writeFileSync(
  path.join(rootKnowledge, 'docs', 'knowledge', 'work', 'okf-foundation', 'work-item.md'),
  '---\ntype: bee.work-item\ntitle: Fixture work item for the knowledge.context example\ndescription: Fixture so the registry example resolves inside the isolated repo\nbee:\n  id: okf-foundation\n  lifecycle: active\n  areas: [okf]\n  required_context: [patterns/fixture-critical.md]\n  decisions: [D27]\n---\n\n# Fixture work item\n\nFixture prose.\n',
  'utf8',
);
fs.writeFileSync(
  path.join(rootKnowledge, 'docs', 'knowledge', 'patterns', 'fixture-critical.md'),
  '---\ntype: bee.pattern\ntitle: Fixture critical pattern\ndescription: Always in context\nbee:\n  id: fixture-critical\n  lifecycle: active\n  areas: [okf]\n  critical: true\n---\n\n# Fixture critical pattern\n\nFixture prose.\n',
  'utf8',
);

await check('knowledge.context example: --work okf-foundation --budget 20000 returns a budget-fitting manifest of paths, never content (D27)', async () => {
  const result = await assertExampleOk('knowledge.context', { cwd: rootKnowledge });
  const manifest = JSON.parse(result.stdout);
  assert(manifest.work === 'okf-foundation', `expected the resolved work id, got ${result.stdout}`);
  assert(manifest.estimator === 'bytes/4', `the estimator must be named bytes/4, got ${result.stdout}`);
  assert(manifest.budget === 20000 && manifest.total_est <= 20000, `total_est must fit the budget, got ${result.stdout}`);
  assert(manifest.entries.length === 2 && manifest.entries[0].path === 'docs/knowledge/work/okf-foundation/work-item.md', `the work item must head the manifest, got ${result.stdout}`);
  assert(manifest.entries[1].path === 'docs/knowledge/patterns/fixture-critical.md', `required_context must follow the work item, got ${result.stdout}`);
  for (const entry of manifest.entries) {
    assert(fs.existsSync(path.join(rootKnowledge, entry.path)), `every manifest path must exist on disk: ${entry.path}`);
  }
  assert(!result.stdout.includes('Fixture prose.'), `the manifest must never carry file content, got ${result.stdout}`);
});

// ─── knowledge.context --lane (i54-closeout D3): lane-scaled budget presets ─
// --lane resolves to a numeric --budget BEFORE the generic validate() layer
// runs (bee.mjs's resolveKnowledgeContextLaneBudget), so `budget` stays
// required and a bare call with neither flag keeps failing exactly as before.

await check('knowledge.context --lane standard resolves the standard preset (20000) with no --budget given (i54-closeout D3)', async () => {
  const result = await runModuleWorker(BEE_MJS, {
    args: ['knowledge', 'context', '--work', 'okf-foundation', '--lane', 'standard', '--json'],
    cwd: rootKnowledge,
  });
  assert(result.status === 0, `expected success, got status=${result.status} stdout=${result.stdout} stderr=${result.stderr}`);
  const manifest = JSON.parse(result.stdout);
  assert(manifest.budget === 20000, `--lane standard must resolve to budget 20000, got ${result.stdout}`);
});

await check('knowledge.context: explicit --budget always wins over --lane when both are given (i54-closeout D3)', async () => {
  const result = await runModuleWorker(BEE_MJS, {
    args: ['knowledge', 'context', '--work', 'okf-foundation', '--budget', '5000', '--lane', 'high-risk', '--json'],
    cwd: rootKnowledge,
  });
  assert(result.status === 0, `expected success, got status=${result.status} stdout=${result.stdout} stderr=${result.stderr}`);
  const manifest = JSON.parse(result.stdout);
  assert(manifest.budget === 5000, `explicit --budget (5000) must win over --lane high-risk (30000), got ${result.stdout}`);
});

await check('knowledge.context: neither --budget nor --lane still refuses exactly as before this cell (required, missing)', async () => {
  const result = await runModuleWorker(BEE_MJS, {
    args: ['knowledge', 'context', '--work', 'okf-foundation', '--json'],
    cwd: rootKnowledge,
  });
  assert(result.status !== 0, `a bare call with neither flag must still fail, got status=${result.status} stdout=${result.stdout}`);
  const payload = JSON.parse(result.stdout);
  assert(payload.error && payload.error.field === 'budget' && payload.error.reason === 'required, missing', `expected the byte-identical required-missing refusal on --budget, got ${result.stdout}`);
});

await check('knowledge.context: an unrecognized --lane value is left unfilled, still refusing on missing --budget rather than a silent wrong number', async () => {
  const result = await runModuleWorker(BEE_MJS, {
    args: ['knowledge', 'context', '--work', 'okf-foundation', '--lane', 'bogus', '--json'],
    cwd: rootKnowledge,
  });
  assert(result.status !== 0, `an unrecognized --lane must not silently succeed, got status=${result.status} stdout=${result.stdout}`);
  const payload = JSON.parse(result.stdout);
  assert(payload.error && payload.error.field === 'budget' && payload.error.reason === 'required, missing', `expected the required-missing refusal on --budget, got ${result.stdout}`);
});

// ─── knowledge.promote (okf-foundation S7, cell okf-9): the registry example
// is `bee knowledge promote --work okf-foundation --json`, so it resolves
// against the SAME isolated fixture work item seeded above. One capped cell
// trace is seeded into the fixture repo's own .bee/cells/ store — promote
// READS it (D2 permits reads of the runtime store) and must leave every file
// it touched byte-identical.

fs.mkdirSync(path.join(rootKnowledge, '.bee', 'cells'), { recursive: true });
writeJsonAtomic(path.join(rootKnowledge, '.bee', 'cells', 'fixture-1.json'), {
  id: 'fixture-1',
  feature: 'okf-foundation',
  lane: 'small',
  behavior_change: true,
  status: 'capped',
  title: 'Fixture capped cell for the knowledge.promote example',
  verify: 'node -e "process.exit(0)"',
  trace: {
    outcome: 'fixture outcome recorded at cap time',
    files_changed: ['docs/knowledge/patterns/fixture-critical.md'],
    deviations: ['fixture deviation recorded at cap time'],
    behavior_change: true,
    capped_at: '2026-07-22T08:00:00.000Z',
    verification_evidence: '{"verify_tail":"PASS fixture"}',
    verify_passed: true,
  },
});

await check('knowledge.promote example: --work okf-foundation proposes a delivery draft, area bullets and pitfall candidates — and writes nothing (D38/D2)', async () => {
  const bundleBefore = fs.readFileSync(path.join(rootKnowledge, 'docs', 'knowledge', 'work', 'okf-foundation', 'work-item.md'), 'utf8');
  const cellBefore = fs.readFileSync(path.join(rootKnowledge, '.bee', 'cells', 'fixture-1.json'), 'utf8');
  const result = await assertExampleOk('knowledge.promote', { cwd: rootKnowledge });
  const proposal = JSON.parse(result.stdout);
  assert(proposal.work === 'okf-foundation', `expected the resolved work id, got ${result.stdout}`);
  assert(JSON.stringify(proposal.writes) === JSON.stringify([]), `promote must declare zero writes, got ${result.stdout}`);
  assert(proposal.cells.length === 1 && proposal.cells[0].id === 'fixture-1', `the one capped fixture cell must be mined, got ${JSON.stringify(proposal.cells)}`);
  assert(proposal.delivery.path === 'work/okf-foundation/delivery.md', `the draft must target the work item's delivery sibling, got ${JSON.stringify(proposal.delivery.path)}`);
  assert(proposal.delivery.content.includes('fixture-1'), 'the draft must name the mined cell');
  assert(proposal.area_updates.length === 1 && proposal.area_updates[0].area === 'okf', `the work item's one area must get a section, got ${JSON.stringify(proposal.area_updates)}`);
  assert(proposal.pattern_candidates.length === 1 && proposal.pattern_candidates[0].cell === 'fixture-1', `the deviation must yield one pitfall candidate, got ${JSON.stringify(proposal.pattern_candidates)}`);
  assert(!fs.existsSync(path.join(rootKnowledge, 'docs', 'knowledge', 'work', 'okf-foundation', 'delivery.md')), 'promote must NOT write the delivery draft it proposes');
  assert(fs.readFileSync(path.join(rootKnowledge, 'docs', 'knowledge', 'work', 'okf-foundation', 'work-item.md'), 'utf8') === bundleBefore, 'promote must leave the bundle byte-identical');
  assert(fs.readFileSync(path.join(rootKnowledge, '.bee', 'cells', 'fixture-1.json'), 'utf8') === cellBefore, 'promote must leave the cell trace byte-identical (D2: reads only)');
});

await check('every registry entry had its example executed at least once (nothing silently skipped)', async () => {
  const allNames = new Set(COMMAND_REGISTRY.map((e) => e.name));
  const missing = [...allNames].filter((name) => !executedNames.has(name));
  assert(missing.length === 0, `these registry entries were never exercised: ${missing.join(', ')}`);
  assert(executedNames.size === allNames.size, 'executed-name count should match registry size exactly');
});

// ─── bee.mjs (harness-integration-2): unified dispatcher tests ─────────────
// A SECOND isolated temp repo, kept fully separate from the demo-1 fixture
// chain above so bee.mjs's own mutating calls never collide with it.

const root2 = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-mjs-test-'));
fs.mkdirSync(path.join(root2, '.bee'), { recursive: true });
writeJsonAtomic(path.join(root2, '.bee', 'onboarding.json'), {
  schema_version: '1.0',
  bee_version: '0.1.0',
});
writeState(root2, {
  ...defaultState(),
  phase: 'swarming',
  feature: 'demo2',
  approved_gates: { context: true, shape: true, execution: true, review: false },
});

async function runBee(args, cwd = root2) {
  return await runModuleWorker(BEE_MJS, { args, cwd });
}

// ─── ce-1 (cli-ergonomics D1): batch flag validation end to end ────────────
// One invocation reports every missing/invalid flag, with a runnable
// Example line — both through the generic validate() layer (STDOUT, DA5/D3)
// and through the three handler-owned legacy verbs (STDERR, DB3).

await check('main(): a validate()-layer refusal (cells.tier, no flags) lists every problem plus an Example line, still on STDOUT (DB3 channel unchanged)', async () => {
  const result = await runBee(['cells', 'tier']);
  assert(result.status === 1, `expected exit 1, got ${result.status} (stdout: ${result.stdout}, stderr: ${result.stderr})`);
  assert(/id/.test(result.stdout), `expected "id" named, got: ${result.stdout}`);
  assert(/tier/.test(result.stdout), `expected "tier" named, got: ${result.stdout}`);
  assert(/Example:/.test(result.stdout), `expected an Example: line, got: ${result.stdout}`);
  // stderr still carries the D3 work-visibility timing line ("[bee] cells
  // tier Nms") on every direct invocation — DB3 is about the REFUSAL text
  // itself, which must stay off stderr entirely.
  assert(!/required, missing/.test(result.stderr), `validate-layer refusal text must stay on STDOUT (DB3), not leak onto stderr: ${result.stderr}`);
});

await check('main(): backlog add with zero flags names every missing flag on STDERR (DB3 channel unchanged) plus an Example line', async () => {
  const result = await runBee(['backlog', 'add']);
  assert(result.status === 1, `expected exit 1, got ${result.status} (stdout: ${result.stdout}, stderr: ${result.stderr})`);
  assert(result.stdout === '', `handler-owned refusals stay on STDERR (DB3) — unexpected stdout: ${result.stdout}`);
  for (const flag of ['type', 'title', 'severity', 'layer']) {
    assert(result.stderr.includes(`--${flag}`), `expected --${flag} named in the refusal, got: ${result.stderr}`);
  }
  assert(/Example:/.test(result.stderr), `expected an Example: line, got: ${result.stderr}`);
});

await check('main(): backlog add with an invalid --severity names P1/P2/P3 in one refusal', async () => {
  const result = await runBee(['backlog', 'add', '--type', 'friction', '--title', 'demo', '--severity', 'P9', '--layer', 'state']);
  assert(result.status === 1, `expected exit 1, got ${result.status} (stderr: ${result.stderr})`);
  assert(/P1/.test(result.stderr) && /P2/.test(result.stderr) && /P3/.test(result.stderr), `expected P1/P2/P3 named, got: ${result.stderr}`);
  assert(/Example:/.test(result.stderr), `expected an Example: line, got: ${result.stderr}`);
});

await check('main(): state gate with zero flags names both --name and --approved on STDERR in one refusal', async () => {
  const result = await runBee(['state', 'gate']);
  assert(result.status === 1, `expected exit 1, got ${result.status} (stderr: ${result.stderr})`);
  assert(result.stdout === '', `handler-owned refusals stay on STDERR (DB3) — unexpected stdout: ${result.stdout}`);
  assert(result.stderr.includes('--name'), `expected --name named, got: ${result.stderr}`);
  assert(result.stderr.includes('--approved'), `expected --approved named, got: ${result.stderr}`);
  assert(/Example:/.test(result.stderr), `expected an Example: line, got: ${result.stderr}`);
});

await check('main(): state scribing-run with zero flags names --feature, --areas and --next-action on STDERR in one refusal', async () => {
  const result = await runBee(['state', 'scribing-run']);
  assert(result.status === 1, `expected exit 1, got ${result.status} (stderr: ${result.stderr})`);
  assert(result.stderr.includes('--feature'), `expected --feature named, got: ${result.stderr}`);
  assert(result.stderr.includes('--areas'), `expected --areas named, got: ${result.stderr}`);
  assert(result.stderr.includes('--next-action'), `expected --next-action named, got: ${result.stderr}`);
  assert(/Example:/.test(result.stderr), `expected an Example: line, got: ${result.stderr}`);
});

// ─── C7/C8/C9 (packages-engine-move-3): dispatcher-level unknown-flag
// rejection. The original friction report misread `bee capture add --text x`
// as a silent no-op — it actually exits 1 today via the handler's own
// requireFlag('outcome') check, since --text is simply ignored by validate()
// (validate-args.mjs:90's "unknown-flag rejection is the dispatcher/hook's
// own concern"). The real gap: the refusal never NAMES --text as unknown, so
// an orchestrator skimming only the last line of output misreads a plain
// "outcome missing" message as an unrelated failure. This is the first test
// exercising a truly UNKNOWN (undeclared-in-schema) flag through main() end
// to end — the DB3 tests just above cover MISSING/INVALID values for flags
// the schema already declares, never an undeclared flag name, so this is a
// new row, not a duplicate of any of them (test-economy D5).
await check('main(): capture add --text (an unknown flag) is refused on STDERR naming --text, not just "outcome missing" (C7 dispatcher-level central check)', async () => {
  const result = await runBee(['capture', 'add', '--text', 'x']);
  assert(result.status === 1, `expected exit 1, got ${result.status} (stdout: ${result.stdout}, stderr: ${result.stderr})`);
  assert(result.stdout === '', `unknown-flag refusal stays on STDERR — unexpected stdout: ${result.stdout}`);
  assert(/--text/.test(result.stderr), `expected --text named as the unknown flag, got: ${result.stderr}`);
  assert(/unknown flag/.test(result.stderr), `expected "unknown flag" in the message, got: ${result.stderr}`);
});

// ─── pure-logic unit tests (direct import, no spawn — no side effects since
// bee.mjs guards main() behind a direct-run check) ──────────────────────────

await check('splitCommandTokens separates leading command tokens from the flag section', async () => {
  const { leading, rest } = splitCommandTokens(['cells', 'show', '--id', 'demo-1', '--json']);
  assert(leading.length === 2 && leading[0] === 'cells' && leading[1] === 'show', `leading: ${JSON.stringify(leading)}`);
  assert(rest.length === 3 && rest[0] === '--id', `rest: ${JSON.stringify(rest)}`);
});

await check('resolveCommand special-cases "status" (no subcommand) and dot-joins other groups', async () => {
  assert(resolveCommand([]).commandName === null, 'empty leading -> no command');
  assert(resolveCommand(['status']).commandName === 'status', 'status alone');
  const statusExtra = resolveCommand(['status', 'extra']);
  assert(statusExtra.commandName === 'status' && statusExtra.extra.length === 1, `status extra: ${JSON.stringify(statusExtra)}`);
  const ready = resolveCommand(['cells', 'ready']);
  assert(ready.commandName === 'cells.ready' && ready.extra.length === 0, `cells ready: ${JSON.stringify(ready)}`);
  const bareGroup = resolveCommand(['cells']);
  assert(bareGroup.commandName === 'cells' && bareGroup.extra.length === 0, 'a bare group with no action stays ungrouped (misses the registry -> nearest-match)');
});

await check('parseFlags treats json/stdin/behavior-change/evidence-stdin/active-only as flag-alone booleans', async () => {
  const { flags, json } = parseFlags(['--stdin', '--json']);
  assert(json === true, 'json should be stripped into the json flag');
  assert(flags.stdin === true, 'stdin should be boolean true with no value consumed');
});

await check('parseFlags requires an explicit value for a non-boolean-alone flag, even one the schema types boolean (cells.verify --passed)', async () => {
  const { flags, error } = parseFlags(['--id', 'demo-1', '--command', 'manual check', '--passed', 'true']);
  assert(!error, `unexpected parse error: ${JSON.stringify(error)}`);
  assert(flags.id === 'demo-1' && flags.command === 'manual check' && flags.passed === 'true', `flags: ${JSON.stringify(flags)}`);
});

await check('parseFlags returns a structured error (never throws) for a flag missing its value', async () => {
  const { error } = parseFlags(['--id']);
  assert(error && error.field === 'id' && /requires a value/.test(error.reason), `error: ${JSON.stringify(error)}`);
});

await check('parseFlags returns a structured error for a stray non-flag argument', async () => {
  const { error } = parseFlags(['not-a-flag']);
  assert(error && /unexpected argument/.test(error.reason), `error: ${JSON.stringify(error)}`);
});

await check("parseFlags supports the --name=value form for any flag, taking precedence over the boolean-alone default", async () => {
  const { flags } = parseFlags(['--id=demo-1', '--behavior-change=false']);
  assert(flags.id === 'demo-1', 'id should read from the = form');
  assert(flags['behavior-change'] === 'false', '= form overrides flag-alone boolean handling, matching the original CLIs\' own eq-first parsing order');
});

await check('nearestCommandName suggests the closest real command for a typo', async () => {
  assert(nearestCommandName('cells.lst') === 'cells.list', `got ${nearestCommandName('cells.lst')}`);
  assert(nearestCommandName('staus') === 'status', `got ${nearestCommandName('staus')}`);
});

await check('deprecatedRedirect is null for a live (non-deprecated) registry entry', async () => {
  assert(deprecatedRedirect(entryByName('status')) === null, 'status.deprecated is null -> no redirect');
});

await check('deprecatedRedirect returns a structured redirect naming use_instead for a synthetic deprecated entry, without executing anything', async () => {
  const fakeEntry = { name: 'cells.oldAction', deprecated: { since: '2026-01-01', use_instead: 'cells.newAction' } };
  const redirect = deprecatedRedirect(fakeEntry);
  assert(redirect && redirect.result.ok === false && redirect.result.deprecated === true, `redirect: ${JSON.stringify(redirect)}`);
  assert(redirect.result.use_instead === 'cells.newAction', 'use_instead should name the replacement');
  assert(/use "cells.newAction" instead/.test(redirect.text), `text: ${redirect.text}`);
});

await check('computeManifestHash is deterministic and sensitive to content', async () => {
  const h1 = computeManifestHash();
  const h2 = computeManifestHash();
  assert(h1 === h2, 'the same registry content must hash the same');
  const h3 = computeManifestHash([{ name: 'x' }], '1.0');
  assert(h3 !== h1, 'different registry content must hash differently');
});

// ─── manifestLintWarning (H2, post-advisor-hardening): pure-logic unit tests
// for the advisory release-manifest trap lint (add/update never refuse — see
// the CLI-level end-to-end rows further down for the through-the-dispatcher
// coverage). ─────────────────────────────────────────────────────────────

await check('manifestLintWarning fires on the trap shape: verify mentions release_manifest, files lacks the manifest path', async () => {
  const warning = manifestLintWarning({
    id: 'trap-1',
    verify: 'node scripts/release_manifest.mjs --check',
    files: ['some/other/file.mjs'],
  });
  assert(warning && /trap-1/.test(warning), `expected a warning naming the cell id, got: ${warning}`);
  assert(/release_manifest\.mjs --write/.test(warning), `expected the FIX to name --write, got: ${warning}`);
});

await check('manifestLintWarning is silent when the manifest path is already listed in files', async () => {
  const warning = manifestLintWarning({
    id: 'trap-2',
    verify: 'node scripts/release_manifest.mjs --check',
    files: ['docs/history/codex-harness-hardening/release-manifest.json'],
  });
  assert(warning === null, `expected no warning, got: ${warning}`);
});

await check('manifestLintWarning is silent when verify does not mention release_manifest', async () => {
  const warning = manifestLintWarning({ id: 'trap-3', verify: 'node -e "process.exit(0)"', files: [] });
  assert(warning === null, `expected no warning, got: ${warning}`);
});

await check('manifestLintWarning tolerates malformed cell shapes without throwing', async () => {
  assert(manifestLintWarning(null) === null, 'null cell must not throw');
  assert(manifestLintWarning(undefined) === null, 'undefined cell must not throw');
  assert(manifestLintWarning({}) === null, 'empty object (no verify) must not throw');
  assert(manifestLintWarning({ id: 'trap-4', verify: null, files: [] }) === null, 'non-string verify must not throw');
  assert(
    manifestLintWarning({ id: 'trap-5', verify: 'node scripts/release_manifest.mjs --check' }) !== null,
    'missing files array defaults to [] and still fires — not treated as malformed-silent',
  );
  assert(
    manifestLintWarning({ id: 'trap-6', verify: 'node scripts/release_manifest.mjs --check', files: 'not-an-array' }) !== null,
    'non-array files also defaults to [] and still fires',
  );
});

// ─── regenObligationRefusal (regen-obligation-derived D1/D2): the DERIVED
// regen obligation, which refuses rather than warns. The fixture repo seeds
// its own synthetic guard scripts naming roots bee has never heard of
// ("widgets", "gizmos", ".fixture/runtime") — so a row that goes green here
// can ONLY have gone green by reading the script. Hard-code the roots in the
// guard and every row below reds. ─────────────────────────────────────────

const regenRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-regen-obligation-'));
fs.mkdirSync(path.join(regenRoot, 'scripts'), { recursive: true });
// Synthetic release_manifest.mjs: same shapes the real one uses (a named
// MANIFEST_PATH const, enumerated dirs, individually named files), different
// roots. "widgets" is the seventh-root proof: nothing in bee mentions it.
fs.writeFileSync(
  path.join(regenRoot, 'scripts', 'release_manifest.mjs'),
  [
    'const REPO_ROOT = path.join(__dirname, "..");',
    'const MANIFEST_PATH = path.join(',
    '  REPO_ROOT,',
    '  "docs",',
    '  "fixture",',
    '  "release-manifest.json",',
    ');',
    'const WIDGET_DIR = path.join(REPO_ROOT, "widgets");',
    'const NAMED = [path.join(REPO_ROOT, "gizmos", "one.json")];',
    'const records = [...enumerateTree(path.join(REPO_ROOT, "hooks"), "plugin_hook")];',
    'const skip = path.join(REPO_ROOT, target.path.split("/").join(path.sep));',
    '',
  ].join('\n'),
  'utf8',
);
// Synthetic ledger_parity.mjs: same checkGroup(managed.X, relDir) shape, a
// different base and a group name ("gadgets") the real script does not have.
fs.writeFileSync(
  path.join(regenRoot, 'scripts', 'ledger_parity.mjs'),
  [
    'const abs = path.join(root, ".fixture", "runtime", relDir, name);',
    'checkGroup(managed.widgets, "gadgets");',
    'checkGroup(managed.helpers, "");',
    '',
  ].join('\n'),
  'utf8',
);

const regenCell = (extra) => ({
  id: 'regen-probe',
  feature: 'demo',
  lane: 'small',
  title: 'probe',
  action: 'probe',
  verify: 'node -e "process.exit(0)"',
  files: [],
  ...extra,
});

await check('deriveRegenGuards reads the roots out of the guard scripts — including a root the guard has never heard of, and never the manifest record itself', async () => {
  const guards = deriveRegenGuards(regenRoot);
  const manifest = guards.find((g) => g.key === 'manifest');
  const ledger = guards.find((g) => g.key === 'ledger');
  assert(manifest && ledger, `both guards must be active, got ${JSON.stringify(guards.map((g) => g.key))}`);
  console.log(`      derived manifest roots: ${JSON.stringify(manifest.roots)}`);
  console.log(`      derived manifest requiredFiles: ${JSON.stringify(manifest.requiredFiles)}`);
  console.log(`      derived ledger roots: ${JSON.stringify(ledger.roots)}`);
  assert(manifest.roots.includes('widgets'), `the synthetic seventh root must be derived, got ${JSON.stringify(manifest.roots)}`);
  assert(manifest.roots.includes('gizmos/one.json'), `an individually named file is a root too, got ${JSON.stringify(manifest.roots)}`);
  assert(manifest.roots.includes('hooks'), `an enumerated dir is a root, got ${JSON.stringify(manifest.roots)}`);
  assert(
    !manifest.roots.includes('docs/fixture/release-manifest.json'),
    'the manifest is the RECORD of the hashed set, never a member of it',
  );
  assert(
    JSON.stringify(manifest.requiredFiles) === JSON.stringify(['docs/fixture/release-manifest.json']),
    `the required files entry is the derived MANIFEST_PATH, got ${JSON.stringify(manifest.requiredFiles)}`,
  );
  assert(
    !manifest.roots.some((r) => r.includes('target.path')),
    `an expression-built path.join must contribute nothing, got ${JSON.stringify(manifest.roots)}`,
  );
  assert(
    JSON.stringify(ledger.roots) === JSON.stringify(['.fixture/runtime', '.fixture/runtime/gadgets']),
    `ledger roots come from checkGroup + the base its own checked path uses, got ${JSON.stringify(ledger.roots)}`,
  );
});

await check('the refusal BITES on the manifest ground: a cell touching a hashed root with no release_manifest --check is refused, naming the path, the root, the command and the escape hatch', async () => {
  const refusal = regenObligationRefusal(regenRoot, regenCell({ files: ['widgets/panel.mjs'] }));
  console.log(`      REFUSAL(manifest ground): ${refusal}`);
  assert(refusal !== null, 'a hashed-root touch with no check must be refused, not warned');
  assert(refusal.includes('widgets/panel.mjs'), `must name the offending path, got: ${refusal}`);
  assert(refusal.includes('"widgets"'), `must name the root it hit, got: ${refusal}`);
  assert(refusal.includes('node scripts/release_manifest.mjs --check'), `must name the exact command to add, got: ${refusal}`);
  assert(refusal.includes('docs/fixture/release-manifest.json'), `must name the derived manifest path for files, got: ${refusal}`);
  assert(refusal.includes('regen_obligation_ack'), `must tell the author the escape hatch exists, got: ${refusal}`);
});

await check('the refusal BITES on the manifest ground for the second half too: the check is present but files omits the manifest path', async () => {
  const refusal = regenObligationRefusal(
    regenRoot,
    regenCell({ files: ['gizmos/one.json'], verify: 'node scripts/release_manifest.mjs --check' }),
  );
  console.log(`      REFUSAL(manifest path missing from files): ${refusal}`);
  assert(refusal !== null && /files does not list/.test(refusal), `expected the files half to refuse, got: ${refusal}`);
});

await check('the refusal BITES on the LEDGER ground specifically — a satisfied manifest check never stands in for ledger_parity', async () => {
  const refusal = regenObligationRefusal(
    regenRoot,
    regenCell({
      files: ['.fixture/runtime/gadgets/tool.mjs', 'docs/fixture/release-manifest.json'],
      verify: 'node scripts/release_manifest.mjs --check',
    }),
  );
  console.log(`      REFUSAL(ledger ground): ${refusal}`);
  assert(refusal !== null, 'a ledger-covered touch with no ledger check must be refused');
  assert(refusal.includes('ledger_parity.mjs --check'), `the refusal must be on the ledger ground, got: ${refusal}`);
  assert(refusal.includes('.fixture/runtime/gadgets/tool.mjs'), `must name the offending path, got: ${refusal}`);
});

await check('the refusal stays SILENT on a cell that satisfies the obligation, and on an untouched cell', async () => {
  const satisfied = regenObligationRefusal(
    regenRoot,
    regenCell({
      files: ['widgets/panel.mjs', '.fixture/runtime/gadgets/tool.mjs', 'docs/fixture/release-manifest.json'],
      verify: 'node scripts/release_manifest.mjs --check && node scripts/ledger_parity.mjs --check',
    }),
  );
  console.log(`      SATISFIED cell -> ${satisfied === null ? 'SILENT (no refusal)' : satisfied}`);
  assert(satisfied === null, `a satisfied cell must be silent, got: ${satisfied}`);
  const untouched = regenObligationRefusal(regenRoot, regenCell({ files: ['src/app.ts', 'README.md'] }));
  console.log(`      UNRELATED cell -> ${untouched === null ? 'SILENT (no refusal)' : untouched}`);
  assert(untouched === null, `a cell touching no covered root must be silent, got: ${untouched}`);
});

await check('the refusal stays SILENT on an ACKNOWLEDGED cell — the D1 escape hatch keeps authoring unblocked, as a named act', async () => {
  const acked = regenObligationRefusal(
    regenRoot,
    regenCell({ files: ['widgets/panel.mjs'], regen_obligation_ack: 'docs-only edit; the manifest regen rides fx-9' }),
  );
  console.log(`      ACKNOWLEDGED cell -> ${acked === null ? 'SILENT (no refusal)' : acked}`);
  assert(acked === null, `an acknowledged cell must be silent, got: ${acked}`);
});

await check('a guard script present but shapeless is a BLIND guard — it throws rather than passing silently', async () => {
  const blindRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-regen-blind-'));
  fs.mkdirSync(path.join(blindRoot, 'scripts'), { recursive: true });
  fs.writeFileSync(path.join(blindRoot, 'scripts', 'release_manifest.mjs'), '// no roots here at all\n', 'utf8');
  let message = null;
  try {
    deriveRegenGuards(blindRoot);
  } catch (err) {
    message = err.message;
  }
  console.log(`      BLIND guard -> ${message}`);
  assert(message !== null && /could not derive any covered root/.test(message), `expected a blind-guard refusal, got: ${message}`);
});

await check('a repo with no guard scripts owes nothing — the rule is silent wherever the guard is not installed', async () => {
  const bareRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-regen-bare-'));
  assert(deriveRegenGuards(bareRoot).length === 0, 'no scripts, no active guards');
  assert(
    regenObligationRefusal(bareRoot, regenCell({ files: ['widgets/panel.mjs'] })) === null,
    'a host repo without the guard scripts must never be refused',
  );
});

await check('end-to-end: addCell REFUSES the offending cell and writes nothing, then accepts it once the obligation is met', async () => {
  const offending = regenCell({ id: 'regen-e2e-1', files: ['widgets/panel.mjs'] });
  let threw = null;
  try {
    addCell(regenRoot, offending);
  } catch (err) {
    threw = err.message;
  }
  console.log(`      addCell REFUSED: ${threw}`);
  assert(threw !== null && /REGEN_OBLIGATION/.test(threw), `addCell must refuse, got: ${threw}`);
  assert(!fs.existsSync(path.join(regenRoot, '.bee', 'cells', 'regen-e2e-1.json')), 'a refused add must write nothing');
  const accepted = addCell(
    regenRoot,
    regenCell({
      id: 'regen-e2e-1',
      files: ['widgets/panel.mjs', 'docs/fixture/release-manifest.json'],
      verify: 'node scripts/release_manifest.mjs --check',
    }),
  );
  assert(accepted.id === 'regen-e2e-1', 'the satisfied cell writes normally');
});

await check('end-to-end: updateCell refuses a patch that drags a hashed root into files, and accepts the patch that also carries the check', async () => {
  addCell(regenRoot, regenCell({ id: 'regen-e2e-2', files: ['src/app.ts'] }));
  let threw = null;
  try {
    await updateCell(regenRoot, 'regen-e2e-2', { files: ['widgets/panel.mjs'] });
  } catch (err) {
    threw = err.message;
  }
  console.log(`      updateCell REFUSED: ${threw}`);
  assert(threw !== null && /REGEN_OBLIGATION/.test(threw), `updateCell must refuse the merged shape, got: ${threw}`);
  const untouched = JSON.parse(fs.readFileSync(path.join(regenRoot, '.bee', 'cells', 'regen-e2e-2.json'), 'utf8'));
  assert(JSON.stringify(untouched.files) === JSON.stringify(['src/app.ts']), `a refused patch leaves the cell untouched, got ${JSON.stringify(untouched.files)}`);
  const fixed = await updateCell(regenRoot, 'regen-e2e-2', {
    files: ['widgets/panel.mjs', 'docs/fixture/release-manifest.json'],
    verify: 'node scripts/release_manifest.mjs --check',
  });
  assert(fixed.files.includes('widgets/panel.mjs'), 'the satisfying patch lands');
  const acked = await updateCell(regenRoot, 'regen-e2e-2', { regen_obligation_ack: 'deliberate: regen rides the next cell' });
  assert(acked.regen_obligation_ack === 'deliberate: regen rides the next cell', 'the escape hatch is patchable and recorded on the cell');
});

await check('the escape hatch must carry a REASON — a bare true is refused at authoring', async () => {
  let threw = null;
  try {
    addCell(regenRoot, regenCell({ id: 'regen-ack-bad', regen_obligation_ack: true }));
  } catch (err) {
    threw = err.message;
  }
  assert(threw !== null && /non-empty string/.test(threw), `a boolean ack must be refused, got: ${threw}`);
});

// ─── wave-barrier (parallel-default D2): the ack field already carried any
// reason string; this names "wave-barrier" as the recognized value the
// orchestrator owes at wave close, so a slice's regen-touching cells can
// parallelize instead of false-serializing on shared generated artifacts.
// ─────────────────────────────────────────────────────────────────────────

await check('the refusal names the wave-barrier alternative and the orchestrator\'s wave-close debt (parallel-default D2)', async () => {
  const refusal = regenObligationRefusal(regenRoot, regenCell({ files: ['widgets/panel.mjs'] }));
  console.log(`      REFUSAL(wave-barrier mention): ${refusal}`);
  assert(refusal !== null, 'a hashed-root touch with no check must still be refused');
  assert(refusal.includes('wave-barrier'), `must name the recognized wave-barrier value, got: ${refusal}`);
  assert(
    /orchestrator/.test(refusal) && /wave close/.test(refusal),
    `must name the orchestrator's wave-close debt, got: ${refusal}`,
  );
});

await check('a cell acknowledged with the wave-barrier value is accepted, and the ack is recorded verbatim on the stored cell', async () => {
  const ackReason = 'wave-barrier: shared regen targets deferred to orchestrator at wave close';
  const accepted = addCell(
    regenRoot,
    regenCell({ id: 'regen-e2e-wave-barrier', files: ['widgets/panel.mjs'], regen_obligation_ack: ackReason }),
  );
  assert(accepted.id === 'regen-e2e-wave-barrier', 'the acknowledged cell writes normally');
  assert(
    accepted.regen_obligation_ack === ackReason,
    `the ack must be recorded verbatim on the return value, got: ${JSON.stringify(accepted.regen_obligation_ack)}`,
  );
  const stored = JSON.parse(
    fs.readFileSync(path.join(regenRoot, '.bee', 'cells', 'regen-e2e-wave-barrier.json'), 'utf8'),
  );
  assert(
    stored.regen_obligation_ack === ackReason,
    `the persisted cell must carry the ack verbatim, got: ${JSON.stringify(stored.regen_obligation_ack)}`,
  );
});

await check('a touching cell with neither the manifest check in verify nor a regen_obligation_ack still refuses — wave-barrier is a named act, never a default', async () => {
  let threw = null;
  try {
    addCell(regenRoot, regenCell({ id: 'regen-e2e-no-ack', files: ['widgets/panel.mjs'] }));
  } catch (err) {
    threw = err.message;
  }
  console.log(`      addCell REFUSED (no ack, no check): ${threw}`);
  assert(threw !== null && /REGEN_OBLIGATION/.test(threw), `addCell must refuse without an ack or the required check, got: ${threw}`);
  assert(!fs.existsSync(path.join(regenRoot, '.bee', 'cells', 'regen-e2e-no-ack.json')), 'a refused add must write nothing');
});

await check('measured against THIS checkout\'s real guard scripts (skipped where they are absent)', async () => {
  const realRoot = path.resolve(TEMPLATES_DIR, '..', '..');
  if (!fs.existsSync(path.join(realRoot, 'scripts', 'release_manifest.mjs'))) {
    console.log('      SKIP: this repo does not carry scripts/release_manifest.mjs');
    return;
  }
  const guards = deriveRegenGuards(realRoot);
  const manifest = guards.find((g) => g.key === 'manifest');
  const ledger = guards.find((g) => g.key === 'ledger');
  console.log(`      real manifest roots (${manifest.roots.length}): ${JSON.stringify(manifest.roots)}`);
  console.log(`      real ledger roots (${ledger.roots.length}): ${JSON.stringify(ledger.roots)}`);
  for (const expected of ['skills', 'packages/bee/hooks', '.bee/bin/lib', '.claude-plugin/skills', '.codex-plugin/skills']) {
    assert(manifest.roots.includes(expected), `the real manifest roots must include "${expected}", got ${JSON.stringify(manifest.roots)}`);
  }
  assert(
    manifest.roots.length > 6,
    `the recited "six roots" is itself a copied summary — the script hashes more than that, got ${manifest.roots.length}`,
  );
  assert(ledger.roots.includes('.bee/bin/lib'), `the real ledger must cover .bee/bin/lib, got ${JSON.stringify(ledger.roots)}`);
  // The cell that built this rule must pass it: it touches packages/bee/** and
  // .bee/bin/lib, so it owes both checks and the manifest path.
  const selfCell = {
    id: 'ro-1',
    verify:
      'node packages/bee/tests/test_bee_cli.mjs && node scripts/release_manifest.mjs --check && node scripts/ledger_parity.mjs --check',
    files: [
      'packages/bee/lib/cells.mjs',
      '.bee/bin/lib/cells.mjs',
      'docs/history/codex-harness-hardening/release-manifest.json',
    ],
  };
  assert(regenObligationRefusal(realRoot, selfCell) === null, 'this rule must pass its own cell');
  assert(
    regenObligationRefusal(realRoot, { ...selfCell, verify: 'node scripts/release_manifest.mjs --check' }) !== null,
    'and must refuse that same cell once the ledger check is dropped',
  );
});

// ─── judgeStandardWarning (D3, self-correcting-loop): pure-logic unit tests
// for the advisory judge-standard sufficiency matrix (F4) — add/update never
// refuse, see the CLI-level end-to-end rows further down for the through-the-
// dispatcher coverage, mirroring manifestLintWarning's own H2 layout above.

await check('judgeStandardWarning is silent for an unclassified cell — no change_class, no behavior_change:true (D3: no matrix check at all)', async () => {
  assert(judgeStandardWarning({ id: 'jsw-1', verify: 'node -e 0' }) === null, 'unclassified cell must never warn');
  assert(judgeStandardWarning({ id: 'jsw-2', verify: 'node -e 0', behavior_change: false }) === null, 'behavior_change:false stays unclassified');
});

await check('judgeStandardWarning fires per class when the verify string is missing that class\'s named minimum (formatting/bugfix/api/security/migration)', async () => {
  const cases = [
    ['formatting', { id: 'jsw-fmt', change_class: 'formatting', verify: 'node -e 0' }],
    ['bugfix', { id: 'jsw-bug', change_class: 'bugfix', verify: 'node -e 0' }],
    ['api', { id: 'jsw-api', change_class: 'api', verify: 'node -e 0' }],
    ['security', { id: 'jsw-sec', change_class: 'security', verify: 'node -e 0' }],
    ['migration', { id: 'jsw-mig', change_class: 'migration', verify: 'node -e 0' }],
  ];
  for (const [cls, cell] of cases) {
    const warning = judgeStandardWarning(cell);
    assert(warning && warning.includes('JUDGE_STANDARD_INSUFFICIENT'), `expected a JUDGE_STANDARD_INSUFFICIENT warning for class "${cls}", got: ${warning}`);
    assert(warning.includes(cell.id), `expected the warning to name the cell id for class "${cls}", got: ${warning}`);
    assert(warning.includes(cls), `expected the warning to name the class "${cls}", got: ${warning}`);
  }
});

await check('judgeStandardWarning stays silent per class once verify names that class\'s minimum', async () => {
  assert(judgeStandardWarning({ id: 'jsw-fmt-ok', change_class: 'formatting', verify: 'npm run lint && npm run typecheck' }) === null, 'formatting: lint/typecheck present');
  assert(judgeStandardWarning({ id: 'jsw-bug-ok', change_class: 'bugfix', verify: 'node tests/test_foo.mjs' }) === null, 'bugfix: a test path named');
  assert(judgeStandardWarning({ id: 'jsw-api-ok', change_class: 'api', verify: 'node tests/test_contract.mjs' }) === null, 'api: a contract test named');
  assert(judgeStandardWarning({ id: 'jsw-sec-ok', change_class: 'security', verify: 'node tests/test_negative_path.mjs' }) === null, 'security: a negative-path test named');
  assert(judgeStandardWarning({ id: 'jsw-mig-ok', change_class: 'migration', verify: 'node migrate.mjs forward && node migrate.mjs rollback' }) === null, 'migration: forward + rollback both named');
});

await check('judgeStandardWarning fires for a behavior-class cell with no pre-attached red_failure_evidence, and is silent once one is present', async () => {
  const warning = judgeStandardWarning({ id: 'jsw-behavior-1', behavior_change: true, verify: 'node -e 0' });
  assert(warning && warning.includes('jsw-behavior-1') && warning.includes('behavior'), `expected a behavior-class warning, got: ${warning}`);
  const silent = judgeStandardWarning({
    id: 'jsw-behavior-2',
    behavior_change: true,
    verify: 'node -e 0',
    verification_evidence: { red_failure_evidence: 'a pre-attached characterization of the prior behavior' },
  });
  assert(silent === null, 'a cell already carrying red_failure_evidence at authoring time must not warn');
});

await check('judgeStandardWarning tolerates malformed cell shapes without throwing', async () => {
  assert(judgeStandardWarning(null) === null, 'null cell must not throw');
  assert(judgeStandardWarning(undefined) === null, 'undefined cell must not throw');
  assert(judgeStandardWarning({}) === null, 'empty object (unclassified) must not throw');
  assert(judgeStandardWarning({ id: 'jsw-bad', change_class: 'behavior', verify: null }) !== null, 'non-string verify must not throw, and behavior still warns without evidence');
});

// ─── end-to-end: --help / --help --json (D3 tool-schema manifest) ─────────

await check('bee --help --json parses as valid JSON and lists every existing subcommand', async () => {
  const result = await runBee(['--help', '--json']);
  assert(result.status === 0, `exit ${result.status}: ${result.stderr}`);
  const manifest = JSON.parse(result.stdout);
  assert(manifest.schema_version === SCHEMA_VERSION, `schema_version: ${manifest.schema_version}`);
  const names = new Set(manifest.commands.map((c) => c.name));
  for (const entry of COMMAND_REGISTRY) {
    assert(names.has(entry.name), `--help --json is missing "${entry.name}"`);
  }
  assert(manifest.commands.every((c) => !('helper' in c)), 'the public manifest must never leak the internal `helper` dispatch field');
});

await check('bee --help renders non-empty prose naming known commands', async () => {
  const result = await runBee(['--help']);
  assert(result.status === 0, `exit ${result.status}: ${result.stderr}`);
  assert(result.stdout.includes('bee cells ready'), `expected "bee cells ready" invoke text, got: ${result.stdout}`);
});

// ─── group/command-scoped --help (GH #23) ──────────────────────────────────

await check('bee state --help --json exits 0 and lists only state.* commands, including state.set', async () => {
  const result = await runBee(['state', '--help', '--json']);
  assert(result.status === 0, `exit ${result.status}: ${result.stderr}`);
  const manifest = JSON.parse(result.stdout);
  assert(manifest.schema_version === SCHEMA_VERSION, `schema_version: ${manifest.schema_version}`);
  const names = manifest.commands.map((c) => c.name);
  assert(names.includes('state.set'), `expected "state.set" among scoped commands, got: ${names.join(', ')}`);
  assert(names.every((n) => n.startsWith('state.')), `expected only state.* commands, got: ${names.join(', ')}`);
});

await check('bee cells --help (text) exits 0 and names only cells.* invokes', async () => {
  const result = await runBee(['cells', '--help']);
  assert(result.status === 0, `exit ${result.status}: ${result.stderr}`);
  assert(result.stdout.includes('bee cells ready'), `expected "bee cells ready" invoke text, got: ${result.stdout}`);
  assert(!result.stdout.includes('bee state set'), `scoped "cells --help" leaked an unrelated command: ${result.stdout}`);
});

await check('bee state handoff --help --json scopes to state.handoff.* only', async () => {
  const result = await runBee(['state', 'handoff', '--help', '--json']);
  assert(result.status === 0, `exit ${result.status}: ${result.stderr}`);
  const manifest = JSON.parse(result.stdout);
  const names = manifest.commands.map((c) => c.name);
  assert(names.length > 0, 'expected at least one state.handoff.* command');
  assert(names.every((n) => n.startsWith('state.handoff.')), `expected only state.handoff.* commands, got: ${names.join(', ')}`);
  assert(names.includes('state.handoff.show'), `expected "state.handoff.show" among scoped commands, got: ${names.join(', ')}`);
});

await check('bee bogusgroup --help still errors exactly like an unrecognized command (unknown group unaffected)', async () => {
  const result = await runBee(['bogusgroup', '--help']);
  assert(result.status === 1, `expected exit 1, got ${result.status}: stdout=${result.stdout}`);
  // No GROUP_USAGE_FALLBACKS entry for "bogusgroup" -> falls through to the
  // generic nearest-match suggestion path, which emits via emit() (stdout),
  // not emitError() (stderr) — unchanged from today's non-help behavior.
  assert(result.stdout.includes('Unknown command "bogusgroup"'), `expected the unchanged unknown-command message, got: ${result.stdout}`);
});

// ─── demo-2 fixture chain, driven entirely through the bee.mjs dispatcher ──

await check('bee cells add creates the demo-2 fixture cell used by the rest of this dispatcher chain', async () => {
  const cellFixture = {
    id: 'demo-2',
    feature: 'demo2',
    title: 'Demo cell for bee.mjs dispatcher test',
    lane: 'small',
    action: 'Exercise every cells.* command through the bee.mjs dispatcher.',
    verify: 'node -e "process.exit(0)"',
  };
  fs.writeFileSync(path.join(root2, 'cell-demo-2.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
  const result = await runBee(['cells', 'add', '--file', 'cell-demo-2.json', '--json']);
  assert(result.status === 0, `exit ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  assert(fs.existsSync(path.join(root2, '.bee', 'cells', 'demo-2.json')), 'demo-2 cell file should now exist');
});

await check('bee cells list --json includes demo-2', async () => {
  const result = await runBee(['cells', 'list', '--json']);
  assert(result.status === 0, `exit ${result.status}`);
  const cells = JSON.parse(result.stdout);
  assert(cells.some((c) => c.id === 'demo-2'), `expected demo-2 in list, got ${result.stdout}`);
});

await check('bee cells ready --json lists demo-2 (open, no deps)', async () => {
  const result = await runBee(['cells', 'ready', '--json']);
  assert(result.status === 0, `exit ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  assert(JSON.parse(result.stdout).some((c) => c.id === 'demo-2'), 'demo-2 should be ready (open, no deps)');
});

await check('bee cells show --id demo-2 --json returns the cell', async () => {
  const result = await runBee(['cells', 'show', '--id', 'demo-2', '--json']);
  assert(JSON.parse(result.stdout).id === 'demo-2', `expected demo-2, got ${result.stdout}`);
});

await check('bee cells update patches an allowed field on the open demo-2 fixture, through the dispatcher', async () => {
  const patch = { title: 'Demo cell for bee.mjs dispatcher test (updated)' };
  fs.writeFileSync(path.join(root2, 'cell-demo-2-update.json'), JSON.stringify(patch, null, 2), 'utf8');
  const result = await runBee(['cells', 'update', '--id', 'demo-2', '--file', 'cell-demo-2-update.json', '--json']);
  assert(result.status === 0, `exit ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  assert(JSON.parse(result.stdout).title === patch.title, `expected patched title, got ${result.stdout}`);
});

await check('bee cells update refuses a frozen key (status)', async () => {
  const patch = { status: 'capped' };
  fs.writeFileSync(path.join(root2, 'cell-demo-2-frozen.json'), JSON.stringify(patch, null, 2), 'utf8');
  const result = await runBee(['cells', 'update', '--id', 'demo-2', '--file', 'cell-demo-2-frozen.json']);
  assert(result.status === 1, `expected exit 1, got ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  assert(/status/.test(result.stderr), `expected the frozen field named in stderr, got: ${result.stderr}`);
});

// ─── H2 manifest-lint, through the dispatcher: `cells add`/`cells update`
// warn (stderr, both --json and text) on the trap shape but never refuse the
// write or change the exit code — a separate fixture cell from demo-2 so this
// block never disturbs demo-2's own claim/verify/cap lifecycle below. ──────

await check('bee cells add fires the manifest lint WARNING on the trap shape and still succeeds', async () => {
  const cellFixture = {
    id: 'demo-2-lint-trap',
    feature: 'demo2',
    title: 'H2 lint fixture — trap shape',
    lane: 'small',
    action: 'H2 lint fixture only, never claimed/executed.',
    verify: 'node scripts/release_manifest.mjs --check',
  };
  fs.writeFileSync(path.join(root2, 'cell-lint-trap.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
  const result = await runBee(['cells', 'add', '--file', 'cell-lint-trap.json', '--json']);
  assert(result.status === 0, `the write must always succeed: exit ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  assert(/WARNING/.test(result.stderr) && /demo-2-lint-trap/.test(result.stderr), `expected a WARNING naming the cell in stderr, got: ${result.stderr}`);
  assert(/release-manifest\.json/.test(result.stderr), `expected the missing manifest path named, got: ${result.stderr}`);
});

await check('bee cells add stays silent when the manifest path is already listed in files', async () => {
  const cellFixture = {
    id: 'demo-2-lint-listed',
    feature: 'demo2',
    title: 'H2 lint fixture — manifest already listed',
    lane: 'small',
    action: 'H2 lint fixture only, never claimed/executed.',
    verify: 'node scripts/release_manifest.mjs --check',
    files: ['docs/history/codex-harness-hardening/release-manifest.json'],
  };
  fs.writeFileSync(path.join(root2, 'cell-lint-listed.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
  const result = await runBee(['cells', 'add', '--file', 'cell-lint-listed.json', '--json']);
  assert(result.status === 0, `exit ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  assert(!/WARNING/.test(result.stderr), `expected no WARNING, got stderr=${result.stderr}`);
});

await check('bee cells add stays silent when verify does not mention release_manifest', async () => {
  const cellFixture = {
    id: 'demo-2-lint-unrelated',
    feature: 'demo2',
    title: 'H2 lint fixture — unrelated verify',
    lane: 'small',
    action: 'H2 lint fixture only, never claimed/executed.',
    verify: 'node -e "process.exit(0)"',
  };
  fs.writeFileSync(path.join(root2, 'cell-lint-unrelated.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
  const result = await runBee(['cells', 'add', '--file', 'cell-lint-unrelated.json', '--json']);
  assert(result.status === 0, `exit ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  assert(!/WARNING/.test(result.stderr), `expected no WARNING, got stderr=${result.stderr}`);
});

await check('bee cells update fires the manifest lint WARNING when a patch leaves the MERGED cell in the trap shape, and still succeeds', async () => {
  // demo-2-lint-unrelated was added above with a verify that does not mention
  // release_manifest; patching `verify` alone (files stays absent/[]) must
  // lint the MERGED result, not the raw one-field patch.
  const patch = { verify: 'node scripts/release_manifest.mjs --check' };
  fs.writeFileSync(path.join(root2, 'cell-lint-update-trap.json'), JSON.stringify(patch, null, 2), 'utf8');
  const result = await runBee(['cells', 'update', '--id', 'demo-2-lint-unrelated', '--file', 'cell-lint-update-trap.json', '--json']);
  assert(result.status === 0, `the write must always succeed: exit ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  assert(/WARNING/.test(result.stderr) && /demo-2-lint-unrelated/.test(result.stderr), `expected a WARNING naming the cell in stderr, got: ${result.stderr}`);
});

await check('bee cells update stays silent when the patched cell keeps the manifest path in files', async () => {
  // demo-2-lint-listed already carries the manifest path in files; patching
  // an unrelated field must keep the merged cell out of the trap shape.
  const patch = { title: 'H2 lint fixture — manifest already listed (updated)' };
  fs.writeFileSync(path.join(root2, 'cell-lint-update-listed.json'), JSON.stringify(patch, null, 2), 'utf8');
  const result = await runBee(['cells', 'update', '--id', 'demo-2-lint-listed', '--file', 'cell-lint-update-listed.json', '--json']);
  assert(result.status === 0, `exit ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  assert(!/WARNING/.test(result.stderr), `expected no WARNING, got stderr=${result.stderr}`);
});

// ─── D3 judge-standard matrix, through the dispatcher: `cells add`/`cells
// update` warn (stderr, JUDGE_STANDARD_INSUFFICIENT) on an under-specified
// change_class shape but never refuse the write (F4); `cells cap` warns when
// a behavior-class cap rides the deliberate_exceptions door (F5). Separate
// fixture cells from demo-2 so this block never disturbs demo-2's own
// claim/verify/cap lifecycle below (H2 layout precedent). ─────────────────

await check('bee cells add fires JUDGE_STANDARD_INSUFFICIENT on an under-specified api-class cell and still succeeds', async () => {
  const cellFixture = {
    id: 'demo-2-jsw-api',
    feature: 'demo2',
    title: 'D3 matrix fixture — api class, no contract/integration test named',
    lane: 'small',
    action: 'D3 matrix fixture only, never claimed/executed.',
    verify: 'node -e "process.exit(0)"',
    change_class: 'api',
  };
  fs.writeFileSync(path.join(root2, 'cell-jsw-api.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
  const result = await runBee(['cells', 'add', '--file', 'cell-jsw-api.json', '--json']);
  assert(result.status === 0, `the write must always succeed: exit ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  assert(
    /JUDGE_STANDARD_INSUFFICIENT/.test(result.stderr) && /demo-2-jsw-api/.test(result.stderr),
    `expected a JUDGE_STANDARD_INSUFFICIENT warning naming the cell, got: ${result.stderr}`,
  );
});

await check('bee cells add stays silent on the matrix when the verify already names the class minimum', async () => {
  const cellFixture = {
    id: 'demo-2-jsw-api-ok',
    feature: 'demo2',
    title: 'D3 matrix fixture — api class, contract test named',
    lane: 'small',
    action: 'D3 matrix fixture only, never claimed/executed.',
    verify: 'node tests/test_contract.mjs',
    change_class: 'api',
  };
  fs.writeFileSync(path.join(root2, 'cell-jsw-api-ok.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
  const result = await runBee(['cells', 'add', '--file', 'cell-jsw-api-ok.json', '--json']);
  assert(result.status === 0, `exit ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  assert(!/JUDGE_STANDARD_INSUFFICIENT/.test(result.stderr), `expected no JUDGE_STANDARD_INSUFFICIENT warning, got stderr=${result.stderr}`);
});

await check('bee cells add stays silent on the matrix for an unclassified cell (no change_class, no behavior_change:true)', async () => {
  const cellFixture = {
    id: 'demo-2-jsw-unclassified',
    feature: 'demo2',
    title: 'D3 matrix fixture — unclassified',
    lane: 'small',
    action: 'D3 matrix fixture only, never claimed/executed.',
    verify: 'node -e "process.exit(0)"',
  };
  fs.writeFileSync(path.join(root2, 'cell-jsw-unclassified.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
  const result = await runBee(['cells', 'add', '--file', 'cell-jsw-unclassified.json', '--json']);
  assert(result.status === 0, `exit ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  assert(!/JUDGE_STANDARD_INSUFFICIENT/.test(result.stderr), `expected no warning for an unclassified cell, got stderr=${result.stderr}`);
});

await check('bee cells update fires JUDGE_STANDARD_INSUFFICIENT when a patch leaves the MERGED cell under-specified, and still succeeds', async () => {
  const patch = { change_class: 'security' };
  fs.writeFileSync(path.join(root2, 'cell-jsw-update.json'), JSON.stringify(patch, null, 2), 'utf8');
  const result = await runBee(['cells', 'update', '--id', 'demo-2-jsw-unclassified', '--file', 'cell-jsw-update.json', '--json']);
  assert(result.status === 0, `the write must always succeed: exit ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  assert(
    /JUDGE_STANDARD_INSUFFICIENT/.test(result.stderr) && /demo-2-jsw-unclassified/.test(result.stderr),
    `expected the warning naming the cell, got: ${result.stderr}`,
  );
});

await check('bee cells cap fires JUDGE_STANDARD_INSUFFICIENT (F5) when a behavior-class cap rides deliberate_exceptions, but still succeeds', async () => {
  const cellFixture = {
    id: 'demo-2-jsw-exception',
    feature: 'demo2',
    title: 'D3 F5 fixture — behavior class riding deliberate_exceptions',
    lane: 'small',
    action: 'D3 F5 fixture only.',
    verify: 'node -e "process.exit(0)"',
    change_class: 'behavior',
  };
  fs.writeFileSync(path.join(root2, 'cell-jsw-exception.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
  const added = await runBee(['cells', 'add', '--file', 'cell-jsw-exception.json', '--json']);
  assert(added.status === 0, `add setup failed: ${added.status}: stdout=${added.stdout} stderr=${added.stderr}`);
  const claimed = await runBee(['cells', 'claim', '--id', 'demo-2-jsw-exception', '--worker', 'worker-jsw', '--json']);
  assert(claimed.status === 0, `claim setup failed: ${claimed.status}: stdout=${claimed.stdout} stderr=${claimed.stderr}`);
  const verified = await runBee([
    'cells', 'verify', '--id', 'demo-2-jsw-exception', '--command', 'node -e 0', '--output', 'ok', '--passed', 'true', '--json',
  ]);
  assert(verified.status === 0, `verify setup failed: ${verified.status}: stdout=${verified.stdout} stderr=${verified.stderr}`);

  const capped = await runModuleWorker(BEE_MJS, {
    args: ['cells', 'cap', '--id', 'demo-2-jsw-exception', '--outcome', 'done', '--files', 'a.js', '--evidence-stdin', '--json'],
    cwd: root2,
    input: JSON.stringify({ deliberate_exceptions: ['brand-new surface, no prior behavior to characterize'] }),
  });
  assert(capped.status === 0, `cap must succeed: exit ${capped.status}: stdout=${capped.stdout} stderr=${capped.stderr}`);
  assert(
    /JUDGE_STANDARD_INSUFFICIENT/.test(capped.stderr) && /demo-2-jsw-exception/.test(capped.stderr) && /deliberate_exceptions/.test(capped.stderr),
    `expected the F5 advisory naming the cell and the exception door, got stderr=${capped.stderr}`,
  );
});

await check('bee cells cap stays silent on the F5 advisory for a green-row behavior-class cap (sufficient, unique red_failure_evidence)', async () => {
  const cellFixture = {
    id: 'demo-2-jsw-green',
    feature: 'demo2',
    title: 'D3 F5 fixture — behavior class, sufficient evidence',
    lane: 'small',
    action: 'D3 F5 fixture only.',
    verify: 'node -e "process.exit(0)"',
    change_class: 'behavior',
  };
  fs.writeFileSync(path.join(root2, 'cell-jsw-green.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
  const added = await runBee(['cells', 'add', '--file', 'cell-jsw-green.json', '--json']);
  assert(added.status === 0, `add setup failed: ${added.status}: stdout=${added.stdout} stderr=${added.stderr}`);
  const claimed = await runBee(['cells', 'claim', '--id', 'demo-2-jsw-green', '--worker', 'worker-jsw', '--json']);
  assert(claimed.status === 0, `claim setup failed: ${claimed.status}: stdout=${claimed.stdout} stderr=${claimed.stderr}`);
  const verified = await runBee([
    'cells', 'verify', '--id', 'demo-2-jsw-green', '--command', 'node -e 0', '--output', 'ok', '--passed', 'true', '--json',
  ]);
  assert(verified.status === 0, `verify setup failed: ${verified.status}: stdout=${verified.stdout} stderr=${verified.stderr}`);

  const capped = await runModuleWorker(BEE_MJS, {
    args: ['cells', 'cap', '--id', 'demo-2-jsw-green', '--outcome', 'done', '--files', 'a.js', '--evidence-stdin', '--json'],
    cwd: root2,
    input: JSON.stringify({
      red_failure_evidence:
        'demo-2-jsw-green: a genuinely unique characterization of the prior failing behavior before this change, clearing the D3 floor.',
    }),
  });
  assert(capped.status === 0, `cap must succeed: exit ${capped.status}: stdout=${capped.stdout} stderr=${capped.stderr}`);
  assert(!/JUDGE_STANDARD_INSUFFICIENT/.test(capped.stderr), `expected no F5 advisory on a green-row cap, got stderr=${capped.stderr}`);
});

await check('bee cells claim --id demo-2 --worker claims it', async () => {
  const result = await runBee(['cells', 'claim', '--id', 'demo-2', '--worker', 'worker-test', '--json']);
  assert(JSON.parse(result.stdout).status === 'claimed', `expected claimed, got ${result.stdout}`);
});

// D1 (msh-2): `cells claim --id` is re-backed by the same O_EXCL claim file
// claim-next uses — a second claim on the SAME cell must refuse loudly
// (typed CLAIMED, non-zero exit) instead of silently double-claiming.
await check('bee cells claim --id twice on the same cell: the second call refuses with a typed CLAIMED error, non-zero exit, cell untouched by the loser', async () => {
  addCell(root2, {
    id: 'claim-race-cli-1',
    feature: 'demo2',
    title: 'CLI claim-race fixture',
    lane: 'small',
    action: 'Exercise the double-claim refusal.',
    verify: 'node -e "process.exit(0)"',
  });
  const first = await runBee(['cells', 'claim', '--id', 'claim-race-cli-1', '--worker', 'worker-first', '--session-id', 'sess-cli-first', '--json']);
  assert(first.status === 0 && JSON.parse(first.stdout).status === 'claimed', `first claim should succeed, got status=${first.status} stdout=${first.stdout}`);

  const second = await runBee(['cells', 'claim', '--id', 'claim-race-cli-1', '--worker', 'worker-second', '--session-id', 'sess-cli-second']);
  assert(second.status !== 0, `second claim on the same cell must exit non-zero, got ${second.status}`);
  assert(/CLAIMED/.test(second.stderr), `expected a typed CLAIMED refusal on stderr, got ${second.stderr}`);
  assert(/sess-cli-first/.test(second.stderr), `refusal should name the actual owner, got ${second.stderr}`);
});

// D3: --session-id is optional on `cells claim --id` — a call with neither
// flag nor CLAUDE_CODE_SESSION_ID env still claims cleanly (sessionless).
await check('bee cells claim --id with no --session-id and no CLAUDE_CODE_SESSION_ID env still claims cleanly (single-session flow unaffected)', async () => {
  addCell(root2, {
    id: 'claim-sessionless-cli-1',
    feature: 'demo2',
    title: 'CLI sessionless-claim fixture',
    lane: 'small',
    action: 'Exercise the sessionless claim path.',
    verify: 'node -e "process.exit(0)"',
  });
  const { CLAUDE_CODE_SESSION_ID: _drop, ...envNoSession } = process.env;
  const result = await runModuleWorker(BEE_MJS, {
    args: ['cells', 'claim', '--id', 'claim-sessionless-cli-1', '--worker', 'worker-sessionless', '--json'],
    cwd: root2,
    env: envNoSession,
  });
  assert(result.status === 0, `sessionless claim should succeed, got ${result.status}: ${result.stderr}`);
  assert(JSON.parse(result.stdout).status === 'claimed', `expected claimed, got ${result.stdout}`);
});

// D3: claim-next's --session-id keeps working exactly as before; it now also
// resolves from CLAUDE_CODE_SESSION_ID, and a call with neither is refused
// by the handler (not silently treated as sessionless — claim-next's own
// cross-session selection genuinely needs a session id).
await check('bee cells claim-next: --session-id omitted resolves from CLAUDE_CODE_SESSION_ID env; omitted with no env at all is refused with a clear message', async () => {
  addCell(root2, {
    id: 'claim-next-env-1',
    feature: 'demo2',
    title: 'CLI claim-next env-fallback fixture',
    lane: 'small',
    action: 'Exercise the claim-next session-id env fallback.',
    verify: 'node -e "process.exit(0)"',
  });
  const withEnv = await runModuleWorker(BEE_MJS, {
    args: ['cells', 'claim-next', '--worker', 'worker-env', '--json'],
    cwd: root2,
    env: { ...process.env, CLAUDE_CODE_SESSION_ID: 'sess-from-env-cli' },
  });
  assert(withEnv.status === 0, `claim-next with only the env session id should succeed, got ${withEnv.status}: ${withEnv.stderr}`);
  const parsed = JSON.parse(withEnv.stdout);
  assert(parsed.ok === true && parsed.cell.id === 'claim-next-env-1', `expected claim-next-env-1 claimed, got ${withEnv.stdout}`);

  addCell(root2, {
    id: 'claim-next-noenv-1',
    feature: 'demo2',
    title: 'CLI claim-next no-session fixture',
    lane: 'small',
    action: 'Exercise the claim-next refusal with no session source at all.',
    verify: 'node -e "process.exit(0)"',
  });
  const { CLAUDE_CODE_SESSION_ID: _drop2, ...envNoSession2 } = process.env;
  const withoutEnv = await runModuleWorker(BEE_MJS, {
    args: ['cells', 'claim-next', '--worker', 'worker-noenv'], // no --json: refusal lands on stderr as plain text
    cwd: root2,
    env: envNoSession2,
  });
  assert(withoutEnv.status !== 0, 'claim-next with neither --session-id nor env must refuse');
  assert(/session-id|CLAUDE_CODE_SESSION_ID/.test(withoutEnv.stderr), `refusal should name the missing session source, got ${withoutEnv.stderr}`);
});

// hardening-1-7-10 D5/1710-10: a solo native Codex session has a real
// .bee/sessions/<id>.json record (written by the session-init hook) but no
// CLAUDE_CODE_SESSION_ID/BEE_SESSION_ID env var identifying it — before this
// cell, handleCellsClaimNext resolved its session id WITHOUT `root`, so
// claims.mjs's durable single-live-session fallback never got a chance to
// fire and this exact scenario refused every time. Own isolated roots (not
// root2) so the session records this test writes never leak into any other
// check in this file.
await check('bee cells claim-next: sessionless call with exactly ONE fresh live session record adopts it (no --session-id, no env)', async () => {
  const soloRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-claimnext-solo-'));
  fs.mkdirSync(path.join(soloRoot, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(soloRoot, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  writeState(soloRoot, {
    ...defaultState(),
    phase: 'swarming',
    feature: 'claimnext-solo',
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });
  addCell(soloRoot, {
    id: 'claim-next-solo-1',
    feature: 'claimnext-solo',
    title: 'CLI claim-next solo-session adoption fixture',
    lane: 'small',
    action: 'Exercise the claim-next single-live-session adoption fallback.',
    verify: 'node -e "process.exit(0)"',
  });
  createSession(soloRoot, { id: 'solo-native-codex-session' });

  const { CLAUDE_CODE_SESSION_ID: _dropSolo, BEE_SESSION_ID: _dropSoloBee, ...envNoSessionSolo } = process.env;
  const result = await runModuleWorker(BEE_MJS, {
    args: ['cells', 'claim-next', '--worker', 'worker-solo-adopt', '--json'],
    cwd: soloRoot,
    env: envNoSessionSolo,
  });
  assert(result.status === 0, `sessionless claim-next with one live session should adopt and succeed, got ${result.status}: ${result.stderr}`);
  const parsed = JSON.parse(result.stdout);
  assert(parsed.ok === true && parsed.cell.id === 'claim-next-solo-1', `expected claim-next-solo-1 claimed, got ${result.stdout}`);
  assert(
    parsed.cell.trace.claim_session === 'solo-native-codex-session',
    `expected the claim to be adopted under the sole live session, got ${result.stdout}`,
  );
});

await check('bee cells claim-next: sessionless call with TWO fresh live session records still refuses (real ambiguity, unchanged)', async () => {
  const twoRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-claimnext-two-'));
  fs.mkdirSync(path.join(twoRoot, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(twoRoot, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  writeState(twoRoot, {
    ...defaultState(),
    phase: 'swarming',
    feature: 'claimnext-two',
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });
  addCell(twoRoot, {
    id: 'claim-next-two-1',
    feature: 'claimnext-two',
    title: 'CLI claim-next two-live-session fixture',
    lane: 'small',
    action: 'Exercise the claim-next refusal when adoption is genuinely ambiguous.',
    verify: 'node -e "process.exit(0)"',
  });
  createSession(twoRoot, { id: 'live-session-a' });
  createSession(twoRoot, { id: 'live-session-b' });

  const { CLAUDE_CODE_SESSION_ID: _dropTwo, BEE_SESSION_ID: _dropTwoBee, ...envNoSessionTwo } = process.env;
  const result = await runModuleWorker(BEE_MJS, {
    args: ['cells', 'claim-next', '--worker', 'worker-two-refuse'], // no --json: refusal lands on stderr as plain text
    cwd: twoRoot,
    env: envNoSessionTwo,
  });
  assert(result.status !== 0, 'claim-next with two fresh live sessions and no explicit identity must still refuse');
  assert(/session-id|CLAUDE_CODE_SESSION_ID/.test(result.stderr), `refusal should name the missing session source, got ${result.stderr}`);
});

// Same fallback, threaded through the reservations path (reservations.mjs's
// reserve()) — the cell's second call site. A solo live session adopts and
// the reservation row carries its session id; two live sessions still
// refuses SESSION_REQUIRED, unchanged.
await check('bee reservations reserve: sessionless call with exactly ONE fresh live session record adopts it', async () => {
  const soloRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-reserve-solo-'));
  fs.mkdirSync(path.join(soloRoot, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(soloRoot, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  writeState(soloRoot, {
    ...defaultState(),
    phase: 'swarming',
    feature: 'reserve-solo',
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });
  addCell(soloRoot, {
    id: 'reserve-solo-1',
    feature: 'reserve-solo',
    title: 'CLI reserve solo-session adoption fixture',
    lane: 'small',
    action: 'Exercise the reservations reserve single-live-session adoption fallback.',
    verify: 'node -e "process.exit(0)"',
  });
  createSession(soloRoot, { id: 'solo-native-codex-session-2' });

  const { CLAUDE_CODE_SESSION_ID: _dropRes, BEE_SESSION_ID: _dropResBee, ...envNoSessionRes } = process.env;
  const result = await runModuleWorker(BEE_MJS, {
    args: ['reservations', 'reserve', '--agent', 'solo-reserve-agent', '--cell', 'reserve-solo-1', '--path', 'src/solo-adopt-test.js', '--json'],
    cwd: soloRoot,
    env: envNoSessionRes,
  });
  assert(result.status === 0, `sessionless reserve with one live session should adopt and succeed, got ${result.status}: ${result.stderr}`);
  const parsed = JSON.parse(result.stdout);
  assert(parsed.ok === true, `expected reserve ok:true, got ${result.stdout}`);
  assert(
    parsed.reservation.session === 'solo-native-codex-session-2',
    `expected the reservation to be adopted under the sole live session, got ${result.stdout}`,
  );
});

await check('bee reservations reserve: sessionless call with TWO fresh live session records still refuses SESSION_REQUIRED', async () => {
  const twoRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-reserve-two-'));
  fs.mkdirSync(path.join(twoRoot, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(twoRoot, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  writeState(twoRoot, {
    ...defaultState(),
    phase: 'swarming',
    feature: 'reserve-two',
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });
  addCell(twoRoot, {
    id: 'reserve-two-1',
    feature: 'reserve-two',
    title: 'CLI reserve two-live-session fixture',
    lane: 'small',
    action: 'Exercise the reservations reserve refusal when adoption is genuinely ambiguous.',
    verify: 'node -e "process.exit(0)"',
  });
  createSession(twoRoot, { id: 'live-reserve-session-a' });
  createSession(twoRoot, { id: 'live-reserve-session-b' });

  const { CLAUDE_CODE_SESSION_ID: _dropTwoRes, BEE_SESSION_ID: _dropTwoResBee, ...envNoSessionTwoRes } = process.env;
  const result = await runModuleWorker(BEE_MJS, {
    args: ['reservations', 'reserve', '--agent', 'two-reserve-agent', '--cell', 'reserve-two-1', '--path', 'src/two-refuse-test.js', '--json'],
    cwd: twoRoot,
    env: envNoSessionTwoRes,
  });
  assert(result.status === 1, `expected exit 1 on SESSION_REQUIRED, got ${result.status}: ${result.stdout} ${result.stderr}`);
  const parsed = JSON.parse(result.stdout);
  assert(parsed.ok === false && parsed.code === 'SESSION_REQUIRED', `expected typed SESSION_REQUIRED refusal, got ${result.stdout}`);
});

await check('bee cells verify --passed true (explicit "true" argument, not a bare flag) records a passing verify', async () => {
  const result = await runBee([
    'cells', 'verify', '--id', 'demo-2', '--command', 'manual check', '--output', '0 failing', '--passed', 'true', '--json',
  ]);
  assert(result.status === 0, `exit ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  assert(JSON.parse(result.stdout).trace.verify_passed === true, `expected verify_passed true, got ${result.stdout}`);
});

// D1: --signature threads from bee.mjs's CLI flag through recordVerify into
// the trace.attempts ledger — the worker-suppliable override, end to end
// through the dispatcher (not just the direct lib call already covered above).
await check('bee cells verify --signature overrides the mechanical normalizer through the dispatcher, and a --passed false verify without --signature appends a ledger entry', async () => {
  addCell(root2, {
    id: 'ledger-cli-1',
    feature: 'demo2',
    title: 'CLI ledger fixture',
    lane: 'small',
    action: 'Exercise the --signature flag through the dispatcher.',
    verify: 'node -e "process.exit(0)"',
  });
  const failed = await runBee([
    'cells', 'verify', '--id', 'ledger-cli-1', '--command', 'npm test', '--output', 'FAIL from dispatcher', '--passed', 'false', '--signature', 'cli-custom-sig', '--json',
  ]);
  assert(failed.status === 0, `exit ${failed.status}: stdout=${failed.stdout} stderr=${failed.stderr}`);
  const afterFail = JSON.parse(failed.stdout);
  assert(afterFail.trace.attempts.length === 1, `expected 1 ledger entry, got ${JSON.stringify(afterFail.trace.attempts)}`);
  assert(afterFail.trace.attempts[0].failure_signature === 'cli-custom-sig', `expected the CLI --signature to win, got ${afterFail.trace.attempts[0].failure_signature}`);

  const passed = await runBee([
    'cells', 'verify', '--id', 'ledger-cli-1', '--command', 'npm test', '--output', 'ok', '--passed', 'true', '--json',
  ]);
  const afterPass = JSON.parse(passed.stdout);
  assert(afterPass.trace.attempts.length === 2, `expected 2 ledger entries after the passing verify, got ${afterPass.trace.attempts.length}`);
  assert(afterPass.trace.attempts[1].verdict === 'pass' && afterPass.trace.attempts[1].failure_signature === null, 'the passing entry carries no failure_signature');
});

await check('bee cells cap --id demo-2 caps the cell', async () => {
  const result = await runBee(['cells', 'cap', '--id', 'demo-2', '--outcome', 'dispatcher test cap', '--files', 'cell-demo-2.json', '--json']);
  assert(JSON.parse(result.stdout).status === 'capped', `expected capped, got ${result.stdout}`);
});

// ─── test-economy D1: the `cells cap` handler's computeDiffStats — real git
// end-to-end (the `root`/`root2` fixtures above are deliberately never
// `git init`-ed, so every cap example run against them ALREADY proves the
// no-git fail-open path — see the no-git-specific assertion below for the
// warning-log half of that same claim). These rows use a DEDICATED repo
// with a real `git init` so the tracked/untracked detection, the
// refactor/formatting new-test-file refusal (D1), and the 5-mirror dedupe
// are exercised against real `git diff --numstat` / `git status
// --porcelain` output, not a synthetic diff_stats object. ─────────────────

function gitOk(cwd, args) {
  const r = spawnSync('git', args, { cwd, encoding: 'utf8' });
  assert(r.status === 0, `git ${args.join(' ')} (cwd=${cwd}) failed: ${r.stderr}`);
  return r.stdout;
}

function makeDiffStatsRepo() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-diffstats-'));
  gitOk(dir, ['init', '-q', '-b', 'main']);
  gitOk(dir, ['config', 'user.email', 's@e']);
  gitOk(dir, ['config', 'user.name', 's']);
  fs.mkdirSync(path.join(dir, 'tests'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'src.js'), 'module.exports = 1;\n');
  fs.mkdirSync(path.join(dir, '.bee', 'bin', 'lib'), { recursive: true });
  fs.writeFileSync(path.join(dir, '.bee', 'bin', 'lib', 'mirror.mjs'), '// mirror placeholder\n');
  gitOk(dir, ['add', '.']);
  gitOk(dir, ['commit', '-q', '-m', 'init']);
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  writeState(dir, {
    ...defaultState(),
    phase: 'swarming',
    feature: 'diffstats',
    approved_gates: { context: true, shape: true, execution: true, review: false },
  });
  return dir;
}

async function runDiffStatsRepo(dir, args) {
  return await runModuleWorker(BEE_MJS, { args, cwd: dir });
}

await check('cells cap (D1 real git): a refactor-class cell caps clean when the diff touches only an existing tracked source file (no new test file)', async () => {
  const dir = makeDiffStatsRepo();
  try {
    fs.writeFileSync(path.join(dir, 'src.js'), 'module.exports = 2; // changed\n');
    const cellFixture = {
      id: 'ds-refactor-green',
      feature: 'diffstats',
      title: 'D1 diff_stats fixture — refactor, tracked-only diff',
      lane: 'small',
      action: 'D1 diff_stats fixture only.',
      verify: 'node -e "process.exit(0)"',
      change_class: 'refactor',
    };
    fs.writeFileSync(path.join(dir, 'cell.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
    const added = await runDiffStatsRepo(dir, ['cells', 'add', '--file', 'cell.json', '--json']);
    assert(added.status === 0, `add failed: ${added.stdout}${added.stderr}`);
    const claimed = await runDiffStatsRepo(dir, ['cells', 'claim', '--id', 'ds-refactor-green', '--worker', 'w', '--json']);
    assert(claimed.status === 0, `claim failed: ${claimed.stdout}${claimed.stderr}`);
    const verified = await runDiffStatsRepo(dir, ['cells', 'verify', '--id', 'ds-refactor-green', '--command', 'node -e 0', '--output', 'ok', '--passed', 'true', '--json']);
    assert(verified.status === 0, `verify failed: ${verified.stdout}${verified.stderr}`);
    const capped = await runDiffStatsRepo(dir, ['cells', 'cap', '--id', 'ds-refactor-green', '--outcome', 'done', '--files', 'src.js', '--json']);
    assert(capped.status === 0, `expected a clean cap over a tracked-only diff, got exit ${capped.status}: stdout=${capped.stdout} stderr=${capped.stderr}`);
    assert(JSON.parse(capped.stdout).status === 'capped', `expected capped, got ${capped.stdout}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('cells cap (D1 real git): a refactor-class cell is REFUSED when its diff adds a real untracked test file, naming "refactor" — new_suite_reason does not override', async () => {
  const dir = makeDiffStatsRepo();
  try {
    fs.writeFileSync(path.join(dir, 'tests', 'test_new_thing.mjs'), 'console.log("new suite");\n'.repeat(5));
    const cellFixture = {
      id: 'ds-refactor-newtest',
      feature: 'diffstats',
      title: 'D1 diff_stats fixture — refactor, new untracked test file',
      lane: 'small',
      action: 'D1 diff_stats fixture only.',
      verify: 'node -e "process.exit(0)"',
      change_class: 'refactor',
    };
    fs.writeFileSync(path.join(dir, 'cell.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
    const added = await runDiffStatsRepo(dir, ['cells', 'add', '--file', 'cell.json', '--json']);
    assert(added.status === 0, `add failed: ${added.stdout}${added.stderr}`);
    const claimed = await runDiffStatsRepo(dir, ['cells', 'claim', '--id', 'ds-refactor-newtest', '--worker', 'w', '--json']);
    assert(claimed.status === 0, `claim failed: ${claimed.stdout}${claimed.stderr}`);
    const verified = await runDiffStatsRepo(dir, ['cells', 'verify', '--id', 'ds-refactor-newtest', '--command', 'node -e 0', '--output', 'ok', '--passed', 'true', '--json']);
    assert(verified.status === 0, `verify failed: ${verified.stdout}${verified.stderr}`);
    const capped = await runModuleWorker(BEE_MJS, {
      args: ['cells', 'cap', '--id', 'ds-refactor-newtest', '--outcome', 'done', '--files', 'tests/test_new_thing.mjs', '--evidence-stdin', '--json'],
      cwd: dir,
      input: JSON.stringify({ new_suite_reason: 'trying to override the refactor ban with a stated reason' }),
    });
    assert(capped.status !== 0, `expected the cap to be refused, got exit 0: stdout=${capped.stdout}`);
    assert(/refactor/.test(capped.stdout + capped.stderr), `expected the refusal to name "refactor", got stdout=${capped.stdout} stderr=${capped.stderr}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('cells cap (D1 real git, mirror dedupe): a new untracked test-shaped file under a mirror prefix (.bee/bin/) is excluded from diff_stats — a refactor-class cap over it still succeeds', async () => {
  const dir = makeDiffStatsRepo();
  try {
    fs.writeFileSync(path.join(dir, '.bee', 'bin', 'test_mirror_thing.mjs'), 'console.log("mirror-shaped, must be excluded");\n'.repeat(5));
    const cellFixture = {
      id: 'ds-refactor-mirror',
      feature: 'diffstats',
      title: 'D1 diff_stats fixture — refactor, new file under a mirror prefix',
      lane: 'small',
      action: 'D1 diff_stats fixture only.',
      verify: 'node -e "process.exit(0)"',
      change_class: 'refactor',
    };
    fs.writeFileSync(path.join(dir, 'cell.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
    const added = await runDiffStatsRepo(dir, ['cells', 'add', '--file', 'cell.json', '--json']);
    assert(added.status === 0, `add failed: ${added.stdout}${added.stderr}`);
    const claimed = await runDiffStatsRepo(dir, ['cells', 'claim', '--id', 'ds-refactor-mirror', '--worker', 'w', '--json']);
    assert(claimed.status === 0, `claim failed: ${claimed.stdout}${claimed.stderr}`);
    const verified = await runDiffStatsRepo(dir, ['cells', 'verify', '--id', 'ds-refactor-mirror', '--command', 'node -e 0', '--output', 'ok', '--passed', 'true', '--json']);
    assert(verified.status === 0, `verify failed: ${verified.stdout}${verified.stderr}`);
    const capped = await runDiffStatsRepo(dir, ['cells', 'cap', '--id', 'ds-refactor-mirror', '--outcome', 'done', '--files', '.bee/bin/test_mirror_thing.mjs', '--json']);
    assert(capped.status === 0, `a mirror-prefixed new test file must be dedupe-excluded, so this cap must succeed — got exit ${capped.status}: stdout=${capped.stdout} stderr=${capped.stderr}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

await check('cells cap (D1 fail-open, no-git): a cap over a repo with no .git logs a computeDiffStats warning to .bee/logs/hooks.jsonl and still caps clean', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-diffstats-nogit-'));
  try {
    fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
    writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
    writeState(dir, {
      ...defaultState(),
      phase: 'swarming',
      feature: 'diffstats-nogit',
      approved_gates: { context: true, shape: true, execution: true, review: false },
    });
    const cellFixture = {
      id: 'ds-nogit-1',
      feature: 'diffstats-nogit',
      title: 'D1 fail-open fixture — no .git at all',
      lane: 'small',
      action: 'D1 fail-open fixture only.',
      verify: 'node -e "process.exit(0)"',
    };
    fs.writeFileSync(path.join(dir, 'cell.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
    const added = await runDiffStatsRepo(dir, ['cells', 'add', '--file', 'cell.json', '--json']);
    assert(added.status === 0, `add failed: ${added.stdout}${added.stderr}`);
    const claimed = await runDiffStatsRepo(dir, ['cells', 'claim', '--id', 'ds-nogit-1', '--worker', 'w', '--json']);
    assert(claimed.status === 0, `claim failed: ${claimed.stdout}${claimed.stderr}`);
    const verified = await runDiffStatsRepo(dir, ['cells', 'verify', '--id', 'ds-nogit-1', '--command', 'node -e 0', '--output', 'ok', '--passed', 'true', '--json']);
    assert(verified.status === 0, `verify failed: ${verified.stdout}${verified.stderr}`);

    const hooksLog = path.join(dir, '.bee', 'logs', 'hooks.jsonl');
    assert(!fs.existsSync(hooksLog), 'precondition: no hooks.jsonl warning yet');

    const capped = await runDiffStatsRepo(dir, ['cells', 'cap', '--id', 'ds-nogit-1', '--outcome', 'done', '--files', 'src.js', '--json']);
    assert(capped.status === 0, `a no-git repo must still cap cleanly (fail-open), got exit ${capped.status}: stdout=${capped.stdout} stderr=${capped.stderr}`);
    assert(JSON.parse(capped.stdout).status === 'capped', `expected capped, got ${capped.stdout}`);

    assert(fs.existsSync(hooksLog), 'expected computeDiffStats to append a warning line to .bee/logs/hooks.jsonl on git failure');
    const lines = fs.readFileSync(hooksLog, 'utf8').trim().split('\n').filter(Boolean).map((l) => JSON.parse(l));
    const warning = lines.find((l) => l.hook === 'cells-cap-diff-stats');
    assert(warning && warning.event === 'warning', `expected a cells-cap-diff-stats warning entry, got ${JSON.stringify(lines)}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// ─── test-economy D3: the D3 new_suite_reason + ratio-ceiling checks driven
// by the SAME real-git computeDiffStats as the D1 rows above (not a
// synthetic diff_stats object) — a behavior-class cell that adds a real
// untracked test file and only lightly touches an existing tracked source
// file, so the ratio genuinely exceeds the standard-lane ceiling.

await check('cells cap (D3 real git): a behavior-class cell adding a new test file with a high test/source ratio is refused for the missing new_suite_reason first, then for the missing ratio_waiver, and caps once both are supplied', async () => {
  const dir = makeDiffStatsRepo();
  try {
    // small tracked source churn (a couple of changed lines)...
    fs.writeFileSync(path.join(dir, 'src.js'), 'module.exports = 2; // changed\n// a second changed line\n');
    // ...next to a much larger new untracked test file, so the ratio (test
    // lines added / source lines changed) genuinely clears the standard-lane
    // ceiling of 4.
    fs.writeFileSync(path.join(dir, 'tests', 'test_new_behavior.mjs'), 'console.log("new behavior suite");\n'.repeat(30));
    const cellFixture = {
      id: 'ds-behavior-ratio',
      feature: 'diffstats',
      title: 'D3 diff_stats fixture — behavior, new test file + high ratio',
      lane: 'standard',
      action: 'D3 diff_stats fixture only.',
      verify: 'node -e "process.exit(0)"',
      change_class: 'behavior',
      must_haves: { truths: ['ds-behavior-ratio: D3 real-git fixture'] },
    };
    fs.writeFileSync(path.join(dir, 'cell.json'), JSON.stringify(cellFixture, null, 2), 'utf8');
    const added = await runDiffStatsRepo(dir, ['cells', 'add', '--file', 'cell.json', '--json']);
    assert(added.status === 0, `add failed: ${added.stdout}${added.stderr}`);
    const claimed = await runDiffStatsRepo(dir, ['cells', 'claim', '--id', 'ds-behavior-ratio', '--worker', 'w', '--json']);
    assert(claimed.status === 0, `claim failed: ${claimed.stdout}${claimed.stderr}`);
    const verified = await runDiffStatsRepo(dir, ['cells', 'verify', '--id', 'ds-behavior-ratio', '--command', 'node -e 0', '--output', 'ok', '--passed', 'true', '--json']);
    assert(verified.status === 0, `verify failed: ${verified.stdout}${verified.stderr}`);

    const capArgs = ['cells', 'cap', '--id', 'ds-behavior-ratio', '--outcome', 'done', '--files', 'src.js,tests/test_new_behavior.mjs'];

    // 1. no evidence at all -> refused for the missing new_suite_reason (D3 checks new_suite_reason before the ratio).
    const noEvidence = await runModuleWorker(BEE_MJS, { args: [...capArgs, '--json'], cwd: dir });
    assert(noEvidence.status !== 0, `expected the cap to be refused with no evidence, got exit 0: stdout=${noEvidence.stdout}`);
    assert(/new_suite_reason/.test(noEvidence.stdout + noEvidence.stderr), `expected the refusal to name new_suite_reason, got stdout=${noEvidence.stdout} stderr=${noEvidence.stderr}`);

    // 2. new_suite_reason supplied, but no ratio_waiver -> refused for the ratio ceiling.
    const reasonOnly = await runModuleWorker(BEE_MJS, {
      args: [...capArgs, '--evidence-stdin', '--json'],
      cwd: dir,
      input: JSON.stringify({ new_suite_reason: 'this behavior needed its own dedicated test suite file' }),
    });
    assert(reasonOnly.status !== 0, `expected the cap to still be refused without ratio_waiver, got exit 0: stdout=${reasonOnly.stdout}`);
    assert(/ratio_waiver/.test(reasonOnly.stdout + reasonOnly.stderr), `expected the refusal to name ratio_waiver, got stdout=${reasonOnly.stdout} stderr=${reasonOnly.stderr}`);

    // 3. both fields supplied -> caps clean.
    const bothSupplied = await runModuleWorker(BEE_MJS, {
      args: [...capArgs, '--evidence-stdin', '--json'],
      cwd: dir,
      input: JSON.stringify({
        new_suite_reason: 'this behavior needed its own dedicated test suite file',
        ratio_waiver: 'the new suite legitimately dwarfs the small source tweak it covers',
      }),
    });
    assert(bothSupplied.status === 0, `expected the cap to succeed once both fields are supplied, got exit ${bothSupplied.status}: stdout=${bothSupplied.stdout} stderr=${bothSupplied.stderr}`);
    assert(JSON.parse(bothSupplied.stdout).status === 'capped', `expected capped, got ${bothSupplied.stdout}`);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

// D-GHF-C (GH #27.5): `cells cap --override-judge` end to end through the
// real dispatcher — refused without the flag when the latest judge-recorded
// verdict is NEEDS_REVISION, capped with an audited trace.judge_overrides
// entry when the flag is supplied.
await check('bee cells cap refuses a NEEDS_REVISION-judged cell without --override-judge, and --override-judge caps it with an audited trace.judge_overrides entry', async () => {
  addCell(root2, {
    id: 'judge-cli-1',
    feature: 'demo2',
    title: 'CLI judge-override fixture',
    lane: 'small',
    action: 'Exercise the --override-judge flag through the dispatcher.',
    verify: 'node -e "process.exit(0)"',
  });
  const claimed = await runBee(['cells', 'claim', '--id', 'judge-cli-1', '--worker', 'worker-judge-cli', '--json']);
  assert(claimed.status === 0, `cells claim setup failed: ${claimed.status}: stdout=${claimed.stdout} stderr=${claimed.stderr}`);
  const verified = await runBee(['cells', 'verify', '--id', 'judge-cli-1', '--command', 'node -e 0', '--output', 'ok', '--passed', 'true', '--json']);
  assert(verified.status === 0, `cells verify setup failed: ${verified.status}: stdout=${verified.stdout} stderr=${verified.stderr}`);

  const verdictPath = path.join(root2, 'verdict-judge-cli-1.json');
  fs.writeFileSync(
    verdictPath,
    JSON.stringify({
      schema: 'judge-verdict/1',
      verdict: 'NEEDS_REVISION',
      checks: [{ id: 'must_haves', status: 'FAIL', evidence: 'diff missed a CONTEXT truth' }],
      failure_signature: 'missed-truth',
      fixability: 'automatic',
      confidence: 'high',
    }),
    'utf8',
  );
  const recorded = await runBee(['cells', 'judge-record', '--id', 'judge-cli-1', '--file', 'verdict-judge-cli-1.json', '--json']);
  assert(recorded.status === 0, `cells judge-record setup failed: ${recorded.status}: stdout=${recorded.stdout} stderr=${recorded.stderr}`);

  const blocked = await runBee(['cells', 'cap', '--id', 'judge-cli-1', '--outcome', 'done', '--files', 'a.js', '--json']);
  assert(blocked.status !== 0, `cap without --override-judge must be refused, got status ${blocked.status}: stdout=${blocked.stdout}`);
  assert(/JUDGE_REWORK_REQUIRED|NEEDS_REVISION/.test(blocked.stdout), `refusal must name the judge block (emitError writes JSON to stdout under --json), got stdout=${blocked.stdout}`);

  const overridden = await runBee(['cells', 'cap', '--id', 'judge-cli-1', '--outcome', 'done', '--files', 'a.js', '--override-judge', 'accepted risk via CLI', '--json']);
  assert(overridden.status === 0, `cap with --override-judge must succeed, got status ${overridden.status}: stdout=${overridden.stdout} stderr=${overridden.stderr}`);
  const overriddenCell = JSON.parse(overridden.stdout);
  assert(overriddenCell.status === 'capped', `expected capped, got ${overridden.stdout}`);
  const overrides = overriddenCell.trace.judge_overrides;
  assert(Array.isArray(overrides) && overrides.length === 1 && overrides[0].reason === 'accepted risk via CLI', `expected one audited judge_overrides entry, got ${JSON.stringify(overrides)}`);
});

await check('bee cells judge --id demo-2 reports no frozen-judge hits', async () => {
  const result = await runBee(['cells', 'judge', '--id', 'demo-2', '--json']);
  assert(JSON.parse(result.stdout).hits.length === 0, `expected no hits, got ${result.stdout}`);
});

await check('bee cells tier --id demo-2 --tier generation sets the tier', async () => {
  const result = await runBee(['cells', 'tier', '--id', 'demo-2', '--tier', 'generation', '--json']);
  assert(JSON.parse(result.stdout).tier === 'generation', `expected generation, got ${result.stdout}`);
});

// D2 + GH #27.4 (D-GHF-C): `cells reset-budget` end to end through the real
// dispatcher — the audited door that reopens a budget-exhausted or
// repeated-failure cell. resetCellBudget now refuses on a healthy cell, so
// budget-cli-1 is exhausted (3 claim/verify/unclaim cycles, same pattern as
// budget-cli-2 below) before the reset itself is exercised. Full exhaustion/
// refusal coverage lives at the lib level (test_lib.mjs); this proves the
// CLI wiring (registry + handler + dispatch table) threads
// --id/--reason/--operator into resetCellBudget correctly.
await check('bee cells reset-budget --id --reason --operator runs through the dispatcher: appends a budget_resets entry, and the reason/actor round-trip verbatim (D-GHF-C)', async () => {
  addCell(root2, {
    id: 'budget-cli-1',
    feature: 'demo2',
    title: 'CLI budget-reset fixture',
    lane: 'small',
    action: 'Exercise cells reset-budget through the dispatcher.',
    verify: 'node -e "process.exit(0)"',
  });
  for (let i = 0; i < 3; i += 1) {
    const claimed = await runBee(['cells', 'claim', '--id', 'budget-cli-1', '--worker', 'w', '--session-id', `sess-cli-reset-${i}`, '--json']);
    assert(claimed.status === 0, `claim #${i + 1} should succeed: ${claimed.stderr}`);
    await runBee(['cells', 'verify', '--id', 'budget-cli-1', '--command', 'node -e ok', '--output', 'ok', '--passed', 'true', '--session-id', `sess-cli-reset-${i}`, '--json']);
    await runBee(['cells', 'unclaim', '--id', 'budget-cli-1', '--session-id', `sess-cli-reset-${i}`, '--json']);
  }
  const blocked = await runBee(['cells', 'claim', '--id', 'budget-cli-1', '--worker', 'w', '--session-id', 'sess-cli-reset-3']);
  assert(blocked.status !== 0, 'precondition: the door should be exhausted before reset');

  const result = await runBee(['cells', 'reset-budget', '--id', 'budget-cli-1', '--reason', 'dispatcher smoke test', '--operator', 'cli-operator-1', '--json']);
  assert(result.status === 0, `exit ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  const cell = JSON.parse(result.stdout);
  assert(Array.isArray(cell.trace.budget_resets) && cell.trace.budget_resets.length === 1, `expected one budget_resets entry, got ${JSON.stringify(cell.trace.budget_resets)}`);
  assert(cell.trace.budget_resets[0].reason === 'dispatcher smoke test', `reason should round-trip verbatim, got ${JSON.stringify(cell.trace.budget_resets[0])}`);
  assert(cell.trace.budget_resets[0].by_actor === 'cli-operator-1', `--operator should round-trip verbatim as by_actor, got ${JSON.stringify(cell.trace.budget_resets[0])}`);
});

await check('bee cells reset-budget --id X refuses without --reason', async () => {
  const result = await runBee(['cells', 'reset-budget', '--id', 'budget-cli-1']);
  assert(result.status !== 0, 'reset-budget without --reason must refuse');
});

await check('bee cells reset-budget --id X --reason refuses without an actor (no --operator, no BEE_AGENT_NAME)', async () => {
  addCell(root2, {
    id: 'budget-cli-1b',
    feature: 'demo2',
    title: 'CLI budget-reset no-actor fixture',
    lane: 'small',
    action: 'Exercise cells reset-budget through the dispatcher without an actor.',
    verify: 'node -e "process.exit(0)"',
  });
  for (let i = 0; i < 3; i += 1) {
    await runBee(['cells', 'claim', '--id', 'budget-cli-1b', '--worker', 'w', '--session-id', `sess-cli-noactor-${i}`, '--json']);
    await runBee(['cells', 'verify', '--id', 'budget-cli-1b', '--command', 'node -e ok', '--output', 'ok', '--passed', 'true', '--session-id', `sess-cli-noactor-${i}`, '--json']);
    await runBee(['cells', 'unclaim', '--id', 'budget-cli-1b', '--session-id', `sess-cli-noactor-${i}`, '--json']);
  }
  // Explicit env with BEE_AGENT_NAME stripped — this refusal must not
  // depend on whatever happens to be set in the host shell running the
  // suite itself.
  const strippedEnv = { ...process.env };
  delete strippedEnv.BEE_AGENT_NAME;
  const result = await runModuleWorker(BEE_MJS, {
    args: ['cells', 'reset-budget', '--id', 'budget-cli-1b', '--reason', 'no actor supplied'],
    cwd: root2,
    env: strippedEnv,
  });
  assert(result.status !== 0, 'reset-budget without an actor must refuse');
  assert(/operator|BEE_AGENT_NAME/.test(result.stderr), `refusal should name --operator or BEE_AGENT_NAME, got stderr=${result.stderr}`);
});

await check('bee cells claim --id refuses with typed CELL_BUDGET_EXHAUSTED once the default max_claims budget is spent, through the real dispatcher (D2)', async () => {
  addCell(root2, {
    id: 'budget-cli-2',
    feature: 'demo2',
    title: 'CLI budget-exhaustion fixture',
    lane: 'small',
    action: 'Exercise the claim-door budget refusal through the dispatcher.',
    verify: 'node -e "process.exit(0)"',
  });
  for (let i = 0; i < 3; i += 1) {
    const claimed = await runBee(['cells', 'claim', '--id', 'budget-cli-2', '--worker', 'w', '--session-id', `sess-cli-budget-${i}`, '--json']);
    assert(claimed.status === 0, `claim #${i + 1} should succeed: ${claimed.stderr}`);
    await runBee(['cells', 'verify', '--id', 'budget-cli-2', '--command', 'node -e ok', '--output', 'ok', '--passed', 'true', '--session-id', `sess-cli-budget-${i}`, '--json']);
    await runBee(['cells', 'unclaim', '--id', 'budget-cli-2', '--session-id', `sess-cli-budget-${i}`, '--json']);
  }
  // No --json here (matches the CLAIMED-refusal precedent above): the CLI's
  // own error() helper writes plain text to stderr only in the non-JSON
  // branch — with --json the same error object is written to STDOUT instead
  // (bee.mjs line ~3939), so a JSON-flagged refusal must be read from stdout.
  const fourth = await runBee(['cells', 'claim', '--id', 'budget-cli-2', '--worker', 'w', '--session-id', 'sess-cli-budget-3']);
  assert(fourth.status !== 0, 'the 4th claim must refuse');
  assert(/CELL_BUDGET_EXHAUSTED/.test(fourth.stderr), `refusal should name CELL_BUDGET_EXHAUSTED, got ${fourth.stderr}`);

  const reset = await runBee(['cells', 'reset-budget', '--id', 'budget-cli-2', '--reason', 'CLI test: reopening after exhaustion', '--operator', 'cli-operator-2', '--json']);
  assert(reset.status === 0, `reset-budget should succeed: ${reset.stderr}`);
  const reopened = await runBee(['cells', 'claim', '--id', 'budget-cli-2', '--worker', 'w', '--session-id', 'sess-cli-budget-4', '--json']);
  assert(reopened.status === 0, `claim after reset should succeed: ${reopened.stderr}`);
});

await check('bee cells block --id demo-2 --reason blocks the cell', async () => {
  const result = await runBee(['cells', 'block', '--id', 'demo-2', '--reason', 'dispatcher test block', '--json']);
  assert(JSON.parse(result.stdout).status === 'blocked', `expected blocked, got ${result.stdout}`);
});

await check('bee cells drop --id demo-2 --reason drops the cell', async () => {
  const result = await runBee(['cells', 'drop', '--id', 'demo-2', '--reason', 'dispatcher test drop', '--json']);
  assert(JSON.parse(result.stdout).status === 'dropped', `expected dropped, got ${result.stdout}`);
});

// ─── reservations, through the dispatcher ──────────────────────────────────

await check('bee reservations reserve/list/release/sweep round-trip through the dispatcher', async () => {
  const reserveResult = await runBee(['reservations', 'reserve', '--agent', 'worker-test', '--cell', 'demo-2', '--path', 'src/dispatcher-test.js', '--json']);
  assert(JSON.parse(reserveResult.stdout).ok === true, `reserve failed: ${reserveResult.stdout}`);

  const listResult = await runBee(['reservations', 'list', '--active-only', '--json']);
  assert(listResult.stdout.includes('worker-test'), `expected worker-test in list, got ${listResult.stdout}`);

  const releaseResult = await runBee(['reservations', 'release', '--agent', 'worker-test', '--json']);
  assert(JSON.parse(releaseResult.stdout).released >= 1, `expected at least 1 released, got ${releaseResult.stdout}`);

  const sweepResult = await runBee(['reservations', 'sweep', '--json']);
  assert(typeof JSON.parse(sweepResult.stdout).released === 'number', `expected a released count, got ${sweepResult.stdout}`);
});

await check('bee reservations reserve returns a CONFLICT (exit 1) when another agent already holds an overlapping path', async () => {
  const first = await runBee(['reservations', 'reserve', '--agent', 'agent-a', '--cell', 'demo-2', '--path', 'src/conflict-test.js', '--json']);
  assert(JSON.parse(first.stdout).ok === true, `first reserve should succeed: ${first.stdout}`);
  const second = await runBee(['reservations', 'reserve', '--agent', 'agent-b', '--cell', 'demo-2', '--path', 'src/conflict-test.js', '--json']);
  assert(second.status === 1, `expected exit 1 on conflict, got ${second.status}`);
  assert(JSON.parse(second.stdout).ok === false, `expected ok:false on conflict, got ${second.stdout}`);
});

// multisession-native-16, advisor consult slice 3 condition B (BINDING,
// biggest-risk cell): the atomic `findForeignHolds + <lease write> +
// insertHold` reserve seam (bee.mjs's handleReservationsReserve, ~1529-1616,
// under worktree-holds.mjs's withHoldsLock) must keep behaving byte-for-byte
// after reserve() moved onto the lease-store shim — bee.mjs's own reserve
// handler was NOT touched by this cell, so this proves the shim slots into
// that unchanged seam correctly rather than merely asserting it wasn't
// edited.
await check(
  "bee reservations reserve (condition B): a successful reserve still double-writes into the shared cross-worktree ledger (a foreign checkout can see it); a path already held by a foreign checkout still denies FOREIGN_HOLD — both through the msn-16 lease-store shim",
  async () => {
    // root2 is an ORDINARY checkout (.bee/onboarding.json present, no .git —
    // see resolveRoots) so resolveHoldTopology resolves holder='main' and the
    // FULL atomic withHoldsLock(findForeignHolds -> reserve -> insertHold)
    // seam already runs on every reserve() call through this dispatcher, not
    // only inside a real linked-worktree checkout.
    const reserved = await runBee(['reservations', 'reserve', '--agent', 'xwh-b-agent', '--cell', 'xwh-b-cell', '--path', 'src/xwh-b/mine.ts', '--json']);
    assert(JSON.parse(reserved.stdout).ok === true, `reserve should succeed: ${reserved.stdout}`);

    const mirrored = findForeignHolds(root2, 'some-other-worktree', ['src/xwh-b/mine.ts']);
    assert(
      mirrored.length === 1 && mirrored[0].holder === 'main',
      `the successful reserve must double-write a 'main'-holder mirror any foreign checkout can see, got ${JSON.stringify(mirrored)}`,
    );

    // A DIFFERENT path, already held by a simulated foreign checkout (a real
    // linked worktree's own prior reserve would mirror exactly this way): a
    // new reserve for that same path must still be denied, never silently
    // succeed just because the local lease store itself has no record of it.
    await mirrorHold(root2, { path: 'src/xwh-b/foreign-held.ts', holder: 'worktree-other', feature: 'demo2', cell: 'other-cell' });
    const denied = await runBee(['reservations', 'reserve', '--agent', 'xwh-b-agent2', '--cell', 'xwh-b-cell2', '--path', 'src/xwh-b/foreign-held.ts', '--json']);
    assert(denied.status === 1, `expected exit 1 on a foreign cross-worktree hold, got ${denied.status}`);
    const deniedParsed = JSON.parse(denied.stdout);
    assert(deniedParsed.ok === false && deniedParsed.code === 'FOREIGN_HOLD', `expected a typed FOREIGN_HOLD denial, got ${denied.stdout}`);
  },
);

// ─── gfb-1 (GH #87 bug 1): hold release scopes by {holder, cell, session} ──
// handleReservationsRelease's xwh-2 affected-cells derivation used to map
// active local reservations to bare cell ids and call
// releaseHolds(mainRoot, { holder, cell }) per cell — never passing session,
// even though a cell id is not unique to one session (two different agents,
// each in their own session, can both hold a reservation on the SAME cell
// id) and releaseHolds/insertHold both already support/mirror a session
// filter. One session's release therefore cleared a DIFFERENT session's
// still-active mirrored hold on that cell.

await check(
  "bee reservations release (gfb-1, GH #87): releasing one session's reservation on a cell never clears a DIFFERENT session's mirrored hold on the same cell id",
  async () => {
    const reserveA = await runBee([
      'reservations', 'reserve', '--agent', 'gfb1-agent-a', '--cell', 'gfb1-shared-cell',
      '--path', 'src/gfb1/a.ts', '--session', 'gfb1-session-a', '--json',
    ]);
    assert(JSON.parse(reserveA.stdout).ok === true, `agent-a reserve should succeed: ${reserveA.stdout}`);

    const reserveB = await runBee([
      'reservations', 'reserve', '--agent', 'gfb1-agent-b', '--cell', 'gfb1-shared-cell',
      '--path', 'src/gfb1/b.ts', '--session', 'gfb1-session-b', '--json',
    ]);
    assert(JSON.parse(reserveB.stdout).ok === true, `agent-b reserve should succeed: ${reserveB.stdout}`);

    const before = findForeignHolds(root2, 'gfb1-checker', ['src/gfb1/a.ts', 'src/gfb1/b.ts']);
    assert(before.length === 2, `expected both mirrored holds present before release, got ${JSON.stringify(before)}`);

    const releaseA = await runBee(['reservations', 'release', '--agent', 'gfb1-agent-a', '--cell', 'gfb1-shared-cell', '--json']);
    assert(JSON.parse(releaseA.stdout).released >= 1, `agent-a release should succeed: ${releaseA.stdout}`);

    const after = findForeignHolds(root2, 'gfb1-checker', ['src/gfb1/a.ts', 'src/gfb1/b.ts']);
    assert(
      after.length === 1 && after[0].session === 'gfb1-session-b' && after[0].path === 'src/gfb1/b.ts',
      `expected only session B's hold on src/gfb1/b.ts to survive untouched after releasing session A's reservation, got ${JSON.stringify(after)}`,
    );
  },
);

await check(
  'bee reservations release (gfb-1): a sessionless (legacy) reservation row still falls back to today\'s exact cell-only hold scoping',
  async () => {
    // runModuleWorker inherits process.env by default (same hazard documented
    // above at the lanes fixture, ~line 837): when THIS test process is
    // itself running inside a bee session, BEE_SESSION_ID/CLAUDE_CODE_SESSION_ID
    // would reach these spawned reserve calls and win over the "no session"
    // fallback in resolveSessionId's own precedence chain, turning what
    // should be sessionless rows into session-bearing ones. Strip both so the
    // fixture's rows resolve to the sessionless sentinel, exactly as a
    // genuinely solo/legacy caller would.
    const strippedEnv = { ...process.env };
    delete strippedEnv.BEE_SESSION_ID;
    delete strippedEnv.CLAUDE_CODE_SESSION_ID;
    const noSession = (args) => runModuleWorker(BEE_MJS, { args, cwd: root2, env: strippedEnv });

    const reserveC = await noSession([
      'reservations', 'reserve', '--agent', 'gfb1-agent-c', '--cell', 'gfb1-sessionless-cell',
      '--path', 'src/gfb1/c.ts', '--json',
    ]);
    assert(JSON.parse(reserveC.stdout).ok === true, `agent-c reserve should succeed: ${reserveC.stdout}`);

    const reserveD = await noSession([
      'reservations', 'reserve', '--agent', 'gfb1-agent-d', '--cell', 'gfb1-sessionless-cell',
      '--path', 'src/gfb1/d.ts', '--json',
    ]);
    assert(JSON.parse(reserveD.stdout).ok === true, `agent-d reserve should succeed: ${reserveD.stdout}`);

    const before = findForeignHolds(root2, 'gfb1-checker2', ['src/gfb1/c.ts', 'src/gfb1/d.ts']);
    assert(before.length === 2, `expected both sessionless mirrored holds present before release, got ${JSON.stringify(before)}`);

    const releaseC = await noSession(['reservations', 'release', '--agent', 'gfb1-agent-c', '--cell', 'gfb1-sessionless-cell', '--json']);
    assert(JSON.parse(releaseC.stdout).released >= 1, `agent-c release should succeed: ${releaseC.stdout}`);

    const after = findForeignHolds(root2, 'gfb1-checker2', ['src/gfb1/c.ts', 'src/gfb1/d.ts']);
    assert(
      after.length === 0,
      `sessionless rows must keep today's exact cell-only scoping (byte-identical to pre-fix behavior) — expected both holds cleared, got ${JSON.stringify(after)}`,
    );
  },
);

// ─── decisions, through the dispatcher ─────────────────────────────────────

await check('bee decisions log/active/search round-trip through the dispatcher', async () => {
  const logResult = await runBee(['decisions', 'log', '--decision', 'Use the unified bee.mjs dispatcher', '--rationale', 'Single discoverable CLI surface', '--json']);
  assert(typeof JSON.parse(logResult.stdout).id === 'string', `log failed: ${logResult.stdout}`);

  const activeResult = await runBee(['decisions', 'active', '--recent', '5', '--json']);
  assert(JSON.parse(activeResult.stdout).decisions.length >= 1, `expected at least 1 active decision, got ${activeResult.stdout}`);

  const searchResult = await runBee(['decisions', 'search', '--text', 'dispatcher', '--json']);
  assert(JSON.parse(searchResult.stdout).decisions.length >= 1, `expected the logged decision to match, got ${searchResult.stdout}`);
});

// ─── malformed input / unknown command (never a bare not-found or a stack trace) ─

await check('a call missing a required parameter returns a structured {ok:false,error} shape, never a stack trace', async () => {
  const result = await runBee(['cells', 'show', '--json']);
  assert(result.status === 1, `expected exit 1, got ${result.status}`);
  const parsed = JSON.parse(result.stdout);
  assert(parsed.ok === false && parsed.error && parsed.error.field === 'id', `expected structured id-missing error, got ${result.stdout}`);
  assert(!result.stdout.includes('at Object.'), 'a stack trace must never reach stdout');
});

await check('an unrecognized command returns a nearest-match suggestion, not a bare not-found', async () => {
  // Retargeted off "cells lst" (dispatcher-unify du-4): now that "cells" is
  // one of the 8 GROUP_USAGE_FALLBACKS groups (DB3 — the dispatcher must
  // reproduce the group's legacy "Use: ..." text for ANY unrecognized
  // cells.* command, not just a bare group), that probe now
  // legitimately hits the group fallback instead of the generic nearest-
  // match path — a deliberate, cell-mandated behavior change, not a
  // weakening. A single unregistered top-level token ("staus", a typo of
  // "status", the one dot-free registry entry) has no group of its own to
  // fall back to, so it still exercises the exact same generic
  // nearestCommandName suggestion path end-to-end.
  const result = await runBee(['staus', '--json']);
  assert(result.status === 1, `expected exit 1, got ${result.status}`);
  const parsed = JSON.parse(result.stdout);
  assert(parsed.ok === false && parsed.suggestion === 'status', `expected suggestion "status", got ${result.stdout}`);
});

await check('a call shaped like a bee.mjs invocation with an unregistered command is denied with a structured error, never executed', async () => {
  const result = await runBee(['not', 'a-real-command', '--json']);
  assert(result.status === 1, `expected exit 1, got ${result.status}`);
  assert(JSON.parse(result.stdout).ok === false, `expected ok:false, got ${result.stdout}`);
});

// ─── manifest content-hash drift ───────────────────────────────────────────

await check('a registry content change surfaces manifest_changed on stderr, never reshaping stdout (P1 fix, review-phase-1.md)', async () => {
  // Baseline call: persists the real hash to .bee/manifest-hash.json.
  const baseline = await runBee(['status', '--json']);
  assert(baseline.status === 0, `baseline exit ${baseline.status}`);
  const baselineBody = JSON.parse(baseline.stdout);
  assert(!('manifest_changed' in baselineBody), 'steady state must never carry manifest_changed on stdout (byte-parity requirement)');

  // Simulate drift by corrupting the persisted hash directly — this cell
  // never edits the real command-registry.mjs (out of its file scope).
  const hashFile = path.join(root2, '.bee', 'cache', 'manifest-hash.json');
  writeJsonAtomic(hashFile, { hash: 'deadbeef', checked_at: new Date().toISOString() });

  const drifted = await runBee(['status', '--json']);
  const driftedBody = JSON.parse(drifted.stdout);
  // stdout's top-level shape is IDENTICAL to the baseline's — same keys, no
  // manifest_changed / manifest_changed_hint / result nesting — a consumer
  // parsing stdout never has to special-case a drift call.
  assert(
    JSON.stringify(Object.keys(driftedBody).sort()) === JSON.stringify(Object.keys(baselineBody).sort()),
    `drifted stdout shape must match steady-state shape; baseline keys=${Object.keys(baselineBody)}, drifted keys=${Object.keys(driftedBody)}`,
  );
  assert(driftedBody.phase === 'swarming', 'the underlying result must be the same bare shape as steady state, not nested under .result');
  assert(drifted.stderr.includes('manifest_changed: true'), `expected the drift hint on stderr, got: ${drifted.stderr}`);

  // The drifted call re-persists the real hash, so the very next call is steady again (no stderr hint).
  const settled = await runBee(['status', '--json']);
  assert(!settled.stderr.includes('manifest_changed'), 'the hash should self-heal to steady state after one drift report');
});

// ─── honest runtime drift (codex-harness-hardening 1c) ───────────────────────
// bee status must compare LIVE .bee/bin managed bytes against the per-file
// sha256 the onboarding ledger recorded — content drift is drift even at the
// same bee_version (PROJ-08), and an absent ledger degrades fail-open.

function sha256(buf) {
  return crypto.createHash('sha256').update(buf).digest('hex');
}

function buildDriftFixture() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-drift-test-'));
  const libDir = path.join(dir, '.bee', 'bin', 'lib');
  fs.mkdirSync(libDir, { recursive: true });
  const libBody = 'export const SAMPLE = 1;\n';
  const helperBody = '// vendored dispatcher\n';
  fs.writeFileSync(path.join(libDir, 'sample.mjs'), libBody);
  fs.writeFileSync(path.join(dir, '.bee', 'bin', 'bee.mjs'), helperBody);
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: BEE_VERSION, // version matches so any drift is CONTENT drift
    managed: {
      lib: { 'sample.mjs': sha256(Buffer.from(libBody)) },
      helpers: { 'bee.mjs': sha256(Buffer.from(helperBody)) },
    },
  });
  writeState(dir, defaultState());
  return dir;
}

async function statusOnboarding(dir) {
  const r = await runBee(['status', '--json'], dir);
  assert(r.status === 0, `status must render (exit 0), got ${r.status}: ${r.stderr}`);
  return JSON.parse(r.stdout).onboarding;
}

await check('drift: an intact runtime (live hashes == recorded managed map) reads drift:false, no drift_detail', async () => {
  const dir = buildDriftFixture();
  const ob = await statusOnboarding(dir);
  assert(ob.drift === false, `expected drift:false on an intact runtime, got ${JSON.stringify(ob)}`);
  assert(ob.drift_detail === undefined, `intact runtime must carry no drift_detail, got ${JSON.stringify(ob.drift_detail)}`);
});

await check('drift: a content-edited managed lib file reads drift:true and names it, even at the same bee_version (PROJ-08)', async () => {
  const dir = buildDriftFixture();
  fs.writeFileSync(path.join(dir, '.bee', 'bin', 'lib', 'sample.mjs'), 'export const SAMPLE = 999;\n');
  const ob = await statusOnboarding(dir);
  assert(ob.drift === true, `expected drift:true after a content edit, got ${JSON.stringify(ob)}`);
  assert(typeof ob.drift === 'boolean', 'drift must stay a boolean (public contract)');
  assert(
    Array.isArray(ob.drift_detail) && ob.drift_detail.includes('.bee/bin/lib/sample.mjs'),
    `drift_detail must name the exact drifted path, got ${JSON.stringify(ob.drift_detail)}`,
  );
});

await check('drift: a content-edited managed HELPER (bee.mjs, not lib) reads drift:true and names it (review P1)', async () => {
  const dir = buildDriftFixture();
  fs.writeFileSync(path.join(dir, '.bee', 'bin', 'bee.mjs'), '// tampered dispatcher\n');
  const ob = await statusOnboarding(dir);
  assert(ob.drift === true, `expected drift:true after a helper edit, got ${JSON.stringify(ob)}`);
  assert(
    Array.isArray(ob.drift_detail) && ob.drift_detail.some((d) => d.includes('bee.mjs') && !d.includes('lib/')),
    `drift_detail must name .bee/bin/bee.mjs (no lib/ prefix), got ${JSON.stringify(ob.drift_detail)}`,
  );
});

await check('drift: a legacy ledger (no managed map) with a mismatched bee_version reads drift:true (version-only signal is live)', async () => {
  const dir = buildDriftFixture();
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.0.1' });
  const ob = await statusOnboarding(dir);
  assert(ob.drift === true, `legacy ledger with a mismatched version must read drift:true, got ${JSON.stringify(ob)}`);
  assert(ob.drift_detail === undefined, 'version-only drift carries no drift_detail');
});

await check('drift: a corrupt (non-JSON) onboarding.json degrades fail-open — status renders exit 0, never throws', async () => {
  const dir = buildDriftFixture();
  fs.writeFileSync(path.join(dir, '.bee', 'onboarding.json'), '{ broken not json');
  const r = await runBee(['status', '--json'], dir);
  assert(r.status === 0, `status must still render on a corrupt ledger, got exit ${r.status}: ${r.stderr}`);
  const ob = JSON.parse(r.stdout).onboarding;
  assert(ob.drift === false, `corrupt ledger must degrade to drift:false, got ${JSON.stringify(ob)}`);
});

await check('drift: a missing managed file reads drift:true (file-set drift)', async () => {
  const dir = buildDriftFixture();
  fs.rmSync(path.join(dir, '.bee', 'bin', 'lib', 'sample.mjs'));
  const ob = await statusOnboarding(dir);
  assert(ob.drift === true, `expected drift:true for a missing managed file, got ${JSON.stringify(ob)}`);
  assert(ob.drift_detail.some((d) => d.includes('sample.mjs') && d.includes('missing')), `expected a "(missing)" detail, got ${JSON.stringify(ob.drift_detail)}`);
});

await check('drift: an extra .mjs in the managed lib dir reads drift:true (file-set drift)', async () => {
  const dir = buildDriftFixture();
  fs.writeFileSync(path.join(dir, '.bee', 'bin', 'lib', 'rogue.mjs'), 'export const X = 1;\n');
  const ob = await statusOnboarding(dir);
  assert(ob.drift === true, `expected drift:true for an extra managed lib file, got ${JSON.stringify(ob)}`);
  assert(ob.drift_detail.some((d) => d.includes('rogue.mjs') && d.includes('extra')), `expected an "(extra)" detail, got ${JSON.stringify(ob.drift_detail)}`);
});

await check('drift: an absent/legacy managed map degrades fail-open — status renders, drift falls back to version-only, never throws (sentinel)', async () => {
  const dir = buildDriftFixture();
  // Legacy ledger: no managed map, version matches the running constant.
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: BEE_VERSION });
  const ob = await statusOnboarding(dir); // must not throw
  assert(ob.drift === false, `legacy ledger with matching version must degrade to drift:false, got ${JSON.stringify(ob)}`);
  assert(ob.drift_detail === undefined, 'legacy fail-open path carries no drift_detail');
});

// ─── source identity in status (SRC-01 / DIST-04) ────────────────────────────

await check('status: surfaces a report-only source field classifying the repo bee-hive (project_projection for a host projection)', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-src-status-'));
  fs.mkdirSync(path.join(dir, '.claude', 'skills', 'bee-hive', 'scripts'), { recursive: true });
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: BEE_VERSION });
  writeState(dir, defaultState());
  const r = await runBee(['status', '--json'], dir);
  assert(r.status === 0, `status must render: ${r.stderr}`);
  const j = JSON.parse(r.stdout);
  assert(j.source && j.source.kind === 'project_projection', `expected source.kind project_projection, got ${JSON.stringify(j.source)}`);
  assert(typeof j.onboarding.drift === 'boolean', 'existing onboarding.drift field must remain (additive change)');
});

// ─── state advisor-ref + Gate 3 precondition (ao-4-1 / AO3 / AO13) ───────────

function readStateFile(dir) {
  return JSON.parse(fs.readFileSync(path.join(dir, '.bee', 'state.json'), 'utf8'));
}

// Build an isolated repo with a state record, an active decision, and a plan.md
// so the advisor_ref staleness anchors have something real to bind to.
function makeAdvisorRoot({ mode = 'high-risk', feature = 'advtest', phase = 'swarming', decisionId = 'dec-1', planBody = '# plan\ncontent\n' } = {}) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-advisor-ref-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  writeState(dir, {
    ...defaultState(),
    phase,
    feature,
    mode,
    approved_gates: { context: true, shape: true, execution: false, review: false },
  });
  if (decisionId) {
    fs.writeFileSync(
      path.join(dir, '.bee', 'decisions.jsonl'),
      `${JSON.stringify({ id: decisionId, type: 'decide', date: '2026-07-17T00:00:00.000Z', decision: 'seed', scope: 'repo' })}\n`,
    );
  }
  if (planBody != null) {
    fs.mkdirSync(path.join(dir, 'docs', 'history', feature), { recursive: true });
    fs.writeFileSync(path.join(dir, 'docs', 'history', feature, 'plan.md'), planBody);
  }
  return dir;
}

function writeDigest(dir, body) {
  const p = path.join(dir, 'consult-digest.txt');
  fs.writeFileSync(p, body);
  return p;
}

// A fresh recorded ref that leaves the record non-stale (records + returns dir).
async function recordFreshRef(dir, { advisor = 'gpt-5.6-sol', body = 'DIGEST-BODY' } = {}) {
  const digest = writeDigest(dir, body);
  const r = await runBee(['state', 'advisor-ref', 'record', '--advisor', advisor, '--digest-file', digest, '--json'], dir);
  assert(r.status === 0, `recording a fresh advisor_ref should succeed: ${r.stderr}`);
  return r;
}

await check('advisor-ref record refuses when no feature is active (idle repo), zero write', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-advisor-noref-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), { schema_version: '1.0', bee_version: '0.1.0' });
  writeState(dir, defaultState()); // phase idle, feature null
  const digest = writeDigest(dir, 'x');
  const r = await runBee(['state', 'advisor-ref', 'record', '--advisor', 'a', '--digest-file', digest], dir);
  assert(r.status !== 0, `expected non-zero exit, got ${r.status}`);
  assert(/no active feature/.test(r.stderr), `expected a no-active-feature refusal, got stderr=${r.stderr}`);
  assert(readStateFile(dir).advisor_ref === undefined, 'a refused record must not write advisor_ref');
});

await check('advisor-ref record stamps consulted_at + verb-computed anchors + digest_head (anchors never caller-supplied)', async () => {
  const dir = makeAdvisorRoot({});
  const digest = writeDigest(dir, 'D'.repeat(600));
  const r = await runBee(['state', 'advisor-ref', 'record', '--advisor', 'gpt-5.6-sol', '--digest-file', digest, '--json'], dir);
  assert(r.status === 0, `record should succeed: ${r.stderr}`);
  const ref = readStateFile(dir).advisor_ref;
  assert(ref && typeof ref === 'object', 'advisor_ref must be written');
  assert(typeof ref.consulted_at === 'string' && ref.consulted_at.length > 0, 'consulted_at stamped');
  assert(ref.feature === 'advtest', `anchor feature should be the record's feature, got ${ref.feature}`);
  assert(ref.newest_decision_id === 'dec-1', `newest_decision_id anchor should be the active decision, got ${ref.newest_decision_id}`);
  assert(/^[0-9a-f]{64}$/.test(ref.plan_sha256), `plan_sha256 should be a real hash, got ${ref.plan_sha256}`);
  assert(ref.advisor === 'gpt-5.6-sol', `advisor identity round-trips, got ${ref.advisor}`);
  assert(ref.digest_head === 'D'.repeat(500), 'digest_head is the first 500 chars of the digest');
  // The record verb exposes no anchor flags — anchors are computed, not passed.
  const entry = COMMAND_REGISTRY.find((e) => e.name === 'state.advisor-ref.record');
  const props = Object.keys(entry.parameters.properties);
  assert(!props.includes('feature') && !props.includes('newest-decision-id') && !props.includes('plan-sha256'), `record must not accept anchor flags, got ${props.join(',')}`);
});

await check('advisor-ref show round-trips a recorded ref and reports it non-stale', async () => {
  const dir = makeAdvisorRoot({});
  await recordFreshRef(dir);
  const r = await runBee(['state', 'advisor-ref', 'show', '--json'], dir);
  assert(r.status === 0, `show should succeed: ${r.stderr}`);
  const out = JSON.parse(r.stdout);
  assert(out.advisor_ref.advisor === 'gpt-5.6-sol', `show returns the recorded advisor, got ${JSON.stringify(out)}`);
  assert(out.stale === false, `a fresh ref must read non-stale, got ${JSON.stringify(out)}`);
});

await check('Gate 3: high-risk execution approval THROWS without an advisor_ref, naming AO3/AO13, zero write', async () => {
  const dir = makeAdvisorRoot({});
  const r = await runBee(['state', 'gate', '--name', 'execution', '--approved', 'true'], dir);
  assert(r.status !== 0, `expected non-zero exit, got ${r.status}`);
  assert(/AO3\/AO13/.test(r.stderr) && /missing or stale/.test(r.stderr), `expected the AO3/AO13 refusal, got stderr=${r.stderr}`);
  assert(/advisor-ref record/.test(r.stderr), `refusal must spell the FIX consult flow, got stderr=${r.stderr}`);
  assert(readStateFile(dir).approved_gates.execution === false, 'a refused execution approval must not flip the gate');
});

await check('Gate 3: high-risk execution approval PASSES with a fresh advisor_ref', async () => {
  const dir = makeAdvisorRoot({});
  await recordFreshRef(dir);
  const r = await runBee(['state', 'gate', '--name', 'execution', '--approved', 'true', '--json'], dir);
  assert(r.status === 0, `fresh ref should let execution approve: ${r.stderr}`);
  assert(JSON.parse(r.stdout).approved_gates.execution === true, 'execution gate approved with a fresh ref');
});

await check('AO13 staleness (1/4): a feature change alone flips the ref stale', async () => {
  const dir = makeAdvisorRoot({});
  await recordFreshRef(dir);
  // Change the record's feature to one whose plan.md has IDENTICAL bytes, so
  // ONLY the feature anchor differs (decision + plan hash unchanged).
  const st = readStateFile(dir);
  fs.mkdirSync(path.join(dir, 'docs', 'history', 'advtest2'), { recursive: true });
  fs.writeFileSync(path.join(dir, 'docs', 'history', 'advtest2', 'plan.md'), '# plan\ncontent\n');
  st.feature = 'advtest2';
  writeJsonAtomic(path.join(dir, '.bee', 'state.json'), st);
  const show = JSON.parse((await runBee(['state', 'advisor-ref', 'show', '--json'], dir)).stdout);
  assert(show.stale === true, `feature change must flip stale, got ${JSON.stringify(show)}`);
  assert(show.reasons.length === 1 && /feature changed/.test(show.reasons[0]), `only the feature reason should fire, got ${JSON.stringify(show.reasons)}`);
  const gate = await runBee(['state', 'gate', '--name', 'execution', '--approved', 'true'], dir);
  assert(gate.status !== 0 && /feature changed/.test(gate.stderr), `gate must refuse on feature change, got stderr=${gate.stderr}`);
});

await check('AO13 staleness (2/4): a newly logged decision alone flips the ref stale', async () => {
  const dir = makeAdvisorRoot({});
  await recordFreshRef(dir);
  fs.appendFileSync(
    path.join(dir, '.bee', 'decisions.jsonl'),
    `${JSON.stringify({ id: 'dec-2', type: 'decide', date: '2026-07-17T01:00:00.000Z', decision: 'later', scope: 'repo' })}\n`,
  );
  const show = JSON.parse((await runBee(['state', 'advisor-ref', 'show', '--json'], dir)).stdout);
  assert(show.stale === true, `a new decision must flip stale, got ${JSON.stringify(show)}`);
  assert(show.reasons.length === 1 && /new decision was logged/.test(show.reasons[0]), `only the decision reason should fire, got ${JSON.stringify(show.reasons)}`);
});

await check('AO13 staleness (3/4): a plan.md edit alone flips the ref stale', async () => {
  const dir = makeAdvisorRoot({});
  await recordFreshRef(dir);
  fs.writeFileSync(path.join(dir, 'docs', 'history', 'advtest', 'plan.md'), '# plan\nEDITED content\n');
  const show = JSON.parse((await runBee(['state', 'advisor-ref', 'show', '--json'], dir)).stdout);
  assert(show.stale === true, `a plan edit must flip stale, got ${JSON.stringify(show)}`);
  assert(show.reasons.length === 1 && /plan\.md changed/.test(show.reasons[0]), `only the plan reason should fire, got ${JSON.stringify(show.reasons)}`);
});

await check('AO13 staleness (4/4): a ref predating an execution-gate revocation flips stale', async () => {
  const dir = makeAdvisorRoot({});
  await recordFreshRef(dir);
  // Revoke execution (approved=false stamps gate_revoked_at.execution = now,
  // strictly after the consult) — the ref now predates the revocation.
  const revoke = await runBee(['state', 'gate', '--name', 'execution', '--approved', 'false', '--json'], dir);
  assert(revoke.status === 0, `revoking execution should succeed: ${revoke.stderr}`);
  assert(typeof JSON.parse(revoke.stdout).gate_revoked_at.execution === 'string', 'execution revocation must be stamped');
  const show = JSON.parse((await runBee(['state', 'advisor-ref', 'show', '--json'], dir)).stdout);
  assert(show.stale === true, `a ref older than the revocation must be stale, got ${JSON.stringify(show)}`);
  assert(show.reasons.length === 1 && /predates the most recent execution-gate revocation/.test(show.reasons[0]), `only the revocation reason should fire, got ${JSON.stringify(show.reasons)}`);
  const gate = await runBee(['state', 'gate', '--name', 'execution', '--approved', 'true'], dir);
  assert(gate.status !== 0 && /predates the most recent execution-gate revocation/.test(gate.stderr), `gate must refuse a revocation-stale ref, got stderr=${gate.stderr}`);
});

await check('non-high-risk mode: execution approval never requires an advisor_ref', async () => {
  const dir = makeAdvisorRoot({ mode: 'standard' });
  const r = await runBee(['state', 'gate', '--name', 'execution', '--approved', 'true', '--json'], dir);
  assert(r.status === 0, `standard mode must approve execution with no ref: ${r.stderr}`);
  assert(JSON.parse(r.stdout).approved_gates.execution === true, 'standard-mode execution approved');
});

await check('other gates on high-risk are untouched: context approval needs no advisor_ref', async () => {
  const dir = makeAdvisorRoot({});
  const r = await runBee(['state', 'gate', '--name', 'context', '--approved', 'true', '--json'], dir);
  assert(r.status === 0, `context gate must approve with no ref on high-risk: ${r.stderr}`);
  assert(JSON.parse(r.stdout).approved_gates.context === true, 'context gate approved');
  assert(readStateFile(dir).advisor_ref === undefined, 'context approval writes no advisor_ref');
});

await check('malformed advisor_ref reads as missing — the gate verb refuses cleanly, never crashes', async () => {
  const dir = makeAdvisorRoot({});
  const st = readStateFile(dir);
  st.advisor_ref = 'not-an-object'; // hand-corrupted fixture
  writeJsonAtomic(path.join(dir, '.bee', 'state.json'), st);
  const gate = await runBee(['state', 'gate', '--name', 'execution', '--approved', 'true'], dir);
  assert(gate.status !== 0, `a corrupt ref must refuse execution, got ${gate.status}`);
  assert(/missing or stale/.test(gate.stderr), `corrupt ref reads as missing, got stderr=${gate.stderr}`);
  assert(!/TypeError|Cannot read|is not a function/.test(gate.stderr), `must not crash on a corrupt ref, got stderr=${gate.stderr}`);
  const show = await runBee(['state', 'advisor-ref', 'show', '--json'], dir);
  assert(show.status === 0 && JSON.parse(show.stdout) === null, `show reads a corrupt ref as missing, got ${show.stdout}`);
});

// ─── work-visibility D3 (decision 4439bd7e): CLI self-timing [wv-1] ────────
// Every direct `bee.mjs` run wall-times itself: a fail-open JSON line to
// .bee/logs/timings.jsonl plus exactly one stderr summary line. stdout stays
// byte-identical for every verb — timing is stderr/log-only, never stdout.

function readLastTimingLine(dir) {
  const raw = fs.readFileSync(path.join(dir, '.bee', 'logs', 'timings.jsonl'), 'utf8');
  const lines = raw.trim().split('\n');
  return JSON.parse(lines[lines.length - 1]);
}

await check('timing (a): a --json verb keeps stdout pure JSON with no "[bee]" text; stderr carries the one summary line', async () => {
  const result = await runBee(['status', '--json']);
  assert(result.status === 0, `status --json should succeed: ${result.stderr}`);
  JSON.parse(result.stdout); // must parse cleanly as JSON — no timing text mixed in
  assert(!/\[bee\]/.test(result.stdout), `stdout must never carry the timing line, got: ${result.stdout}`);
  assert(/\[bee\] \S+ \d+ms/.test(result.stderr), `stderr must carry the "[bee] <cmd> <ms>ms" summary line, got: ${result.stderr}`);
});

await check('timing (b): an unknown command still logs a timing line, recorded as ok:false', async () => {
  const result = await runBee(['definitely-not-a-real-command']);
  assert(result.status !== 0, `an unknown command must fail, got status ${result.status}`);
  const last = readLastTimingLine(root2);
  assert(last.ok === false, `an unknown/failing command must log ok:false, got ${JSON.stringify(last)}`);
});

await check('timing (c): the timings.jsonl line JSON-parses with ts/cmd/ms/ok fields', async () => {
  const result = await runBee(['status', '--json']);
  assert(result.status === 0, `status --json should succeed: ${result.stderr}`);
  const last = readLastTimingLine(root2);
  assert(typeof last.ts === 'string' && !Number.isNaN(Date.parse(last.ts)), `ts must be a parseable ISO string, got ${JSON.stringify(last)}`);
  assert(typeof last.cmd === 'string' && last.cmd.length > 0, `cmd must be a non-empty string, got ${JSON.stringify(last)}`);
  assert(typeof last.ms === 'number' && last.ms >= 0, `ms must be a non-negative number, got ${JSON.stringify(last)}`);
  assert(typeof last.ok === 'boolean' && last.ok === true, `a successful status call must log ok:true, got ${JSON.stringify(last)}`);
});

await check('timing (d): an unwritable/blocked logs path never breaks the command — stdout stays normal, exit stays 0', async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-cli-timing-failopen-'));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  writeState(dir, { ...defaultState(), phase: 'idle' });
  // A plain FILE sitting where the logs directory should be makes the
  // fail-open mkdirSync/appendFileSync throw regardless of process
  // privilege (unlike chmod, which a root-run test would ignore).
  fs.writeFileSync(path.join(dir, '.bee', 'logs'), 'not a directory');
  const result = await runModuleWorker(BEE_MJS, { args: ['status', '--json'], cwd: dir });
  assert(result.status === 0, `command must still succeed despite a blocked logs path: ${result.stderr}`);
  JSON.parse(result.stdout); // stdout must still be normal, parseable JSON
  assert(fs.statSync(path.join(dir, '.bee', 'logs')).isFile(), 'the blocking file must be left untouched (fail-open never replaces it)');
});

// ─── summary ────────────────────────────────────────────────────────────────

console.log(`\n${passed} passed, ${failed} failed`);
process.exitCode = failed > 0 ? 1 : 0;
