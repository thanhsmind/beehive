#!/usr/bin/env node
// test_ship_visibility.mjs — durable behavior suite for ship_visibility
// config surfacing (cell sv-2, feature ship-visibility-config, spec #81 P1).
// sv-1 (packages/bee/lib/state.mjs shipVisibility/SHIP_VISIBILITY_VALUES,
// packages/bee/bee.mjs status --json ship_visibility field,
// packages/bee/lib/inject.mjs buildSessionPreamble's draft-pr preamble line)
// proved its behavior at cap time with throwaway .bee/config.local.json
// overlay probes against the LIVE repo, reverted after each probe. This
// suite is the durable, hermetic replacement it deferred: every case below
// builds its own disposable temp-dir fixture repo (mirrors
// scripts/tests/test_state_compounding_gate.mjs's makeStateRepo and
// scripts/tests/test_compact_capsule.mjs's direct-import-of-buildSessionPreamble
// conventions) and never reads or writes this checkout's own .bee/config.json.
//
// Four behaviors (cell sv-2 must_haves, all four asserted green):
//   1. absent ship_visibility key -> status --json ship_visibility "off",
//      no preamble line, no stderr warning.
//   2. "draft-pr" -> status --json ship_visibility "draft-pr" + the exact
//      one-line preamble addition.
//   3. an unrecognized value -> status --json ship_visibility "off" + a
//      one-line stderr warning naming the bad value.
//   4. explicit "off" -> same silent shape as (1): no preamble line, no
//      stderr warning.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { check, assert, printSummaryAndExit } from '../lib/test-fixture.mjs';
import { runModuleWorker } from '../lib/run-module-worker.mjs';
import { writeJsonAtomic } from '../../packages/bee/lib/fsutil.mjs';
import { buildSessionPreamble } from '../../packages/bee/lib/inject.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..', '..');
const BEE_MJS = path.join(REPO_ROOT, 'packages', 'bee', 'bee.mjs');

const DRAFT_PR_PREAMBLE_LINE =
  '- Ship visibility: draft-pr — first cap opens a draft PR, every cap pushes (routing-and-contracts "Ship visibility")';

// Disposable per-case fixture: a bare .bee/onboarding.json marker is enough
// for resolveRoots (state.mjs) to treat the temp dir as its own repo root —
// same minimal shape scripts/tests/test_state_compounding_gate.mjs's
// makeStateRepo uses. Config is written (or deliberately omitted) per case
// below via setConfig; the live checkout's own .bee/config.json is never
// touched by this file.
function makeFixture(prefix) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(root, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(root, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  return root;
}

function setConfig(root, config) {
  writeJsonAtomic(path.join(root, '.bee', 'config.json'), config);
}

async function runStatusJson(root) {
  const result = await runModuleWorker(BEE_MJS, { args: ['status', '--json'], cwd: root });
  assert(result.status === 0, `status --json exited ${result.status} :: ${result.stderr}`);
  return { payload: JSON.parse(result.stdout), stderr: result.stderr };
}

await check(
  'absent ship_visibility key: status --json "off", no preamble line, no stderr warning',
  async () => {
    const root = makeFixture('bee-ship-vis-absent-');
    try {
      // No .bee/config.json at all — the absent-key case, distinct from the
      // explicit "off" case exercised separately below.
      const { payload, stderr } = await runStatusJson(root);
      assert(
        payload.ship_visibility === 'off',
        `expected "off", got ${JSON.stringify(payload.ship_visibility)}`,
      );
      assert(!/ship_visibility/i.test(stderr), `no config key present must never warn on stderr, got: ${stderr}`);
      const preamble = buildSessionPreamble(root);
      assert(!preamble.includes(DRAFT_PR_PREAMBLE_LINE), 'no config key present must render no Ship visibility preamble line');
      assert(!/Ship visibility/.test(preamble), 'no "Ship visibility" text at all when off');
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
);

await check(
  '"draft-pr": status --json surfaces "draft-pr" and the preamble carries the exact one-line addition',
  async () => {
    const root = makeFixture('bee-ship-vis-draftpr-');
    try {
      setConfig(root, { ship_visibility: 'draft-pr' });
      const { payload, stderr } = await runStatusJson(root);
      assert(
        payload.ship_visibility === 'draft-pr',
        `expected "draft-pr", got ${JSON.stringify(payload.ship_visibility)}`,
      );
      assert(!/unrecognized ship_visibility/i.test(stderr), `a recognized value must never warn, got: ${stderr}`);
      const preamble = buildSessionPreamble(root);
      assert(
        preamble.includes(DRAFT_PR_PREAMBLE_LINE),
        `preamble must carry the exact draft-pr line, got:\n${preamble}`,
      );
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
);

await check(
  'junk value: status --json normalizes to "off" and stderr carries a one-line warning naming the bad value',
  async () => {
    const root = makeFixture('bee-ship-vis-junk-');
    try {
      setConfig(root, { ship_visibility: 'launch-the-rocket' });
      const { payload, stderr } = await runStatusJson(root);
      assert(
        payload.ship_visibility === 'off',
        `unrecognized value must normalize to "off", got ${JSON.stringify(payload.ship_visibility)}`,
      );
      assert(
        /unrecognized ship_visibility "launch-the-rocket"/.test(stderr),
        `stderr must name the bad value, got: ${stderr}`,
      );
      assert(/Allowed: off, draft-pr/.test(stderr), `stderr must list the allowed values, got: ${stderr}`);
      const preamble = buildSessionPreamble(root);
      assert(!/Ship visibility/.test(preamble), 'a normalized-to-off value must render no preamble line, same as off');
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
);

await check(
  'explicit "off": status --json "off", no preamble line, no stderr warning — same silent shape as the absent-key case',
  async () => {
    const root = makeFixture('bee-ship-vis-explicit-off-');
    try {
      setConfig(root, { ship_visibility: 'off' });
      const { payload, stderr } = await runStatusJson(root);
      assert(
        payload.ship_visibility === 'off',
        `expected "off", got ${JSON.stringify(payload.ship_visibility)}`,
      );
      assert(!/ship_visibility/i.test(stderr), `explicit off must never warn on stderr, got: ${stderr}`);
      const preamble = buildSessionPreamble(root);
      assert(!/Ship visibility/.test(preamble), 'explicit off must render no Ship visibility preamble line');
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
);

printSummaryAndExit();
