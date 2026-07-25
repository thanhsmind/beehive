#!/usr/bin/env node
// test_config_validate.mjs — `bee config set/unset/get` local-overlay routing
// for guards.*/hooks.* keys (D2, docs/history/intake-gate-git-exemption/
// CONTEXT.md). This is the defect that let a temporary safety lift
// (`guards.idle_gate: false`) reach a teammate's checkout in incident
// a7d2069 (corrected in 63a41e0): the escape hatch wrote into the TRACKED,
// git-committed .bee/config.json instead of the already-existing gitignored
// overlay (.bee/config.local.json, hardening-8). Same PASS/FAIL/exit-1
// contract as every other suite here — see scripts/lib/test-fixture.mjs.
// Standalone mkdtemp fixture roots per row — never the real repo.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { runModuleWorker } from '../../../scripts/lib/run-module-worker.mjs';
import { check, assert, printSummaryAndExit } from '../../../scripts/lib/test-fixture.mjs';
import { readConfig } from '../lib/state.mjs';
import { readJson, writeJsonAtomic } from '../lib/fsutil.mjs';

function beeModulePath() {
  return fileURLToPath(new URL('../bee.mjs', import.meta.url));
}

function makeConfigRepo(prefix) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(dir, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  return dir;
}

function runBeeConfig(cwd, args) {
  return runModuleWorker(beeModulePath(), { args: ['config', ...args, '--json'], cwd });
}

function trackedConfigPath(dir) {
  return path.join(dir, '.bee', 'config.json');
}

function localConfigFilePath(dir) {
  return path.join(dir, '.bee', 'config.local.json');
}

function withRepo(prefix, fn) {
  const dir = makeConfigRepo(prefix);
  return Promise.resolve()
    .then(() => fn(dir))
    .finally(() => fs.rmSync(dir, { recursive: true, force: true }));
}

// ─── guards.*/hooks.* set/unset routes to the local overlay only ───────────

await check('config set --key guards.* --value ... (no --local) writes ONLY .bee/config.local.json; leaves an absent .bee/config.json absent', async () => {
  await withRepo('bee-config-guard-fresh-', async (dir) => {
    assert(!fs.existsSync(trackedConfigPath(dir)), 'precondition: no tracked config.json yet');
    const result = await runBeeConfig(dir, ['set', '--key', 'guards.idle_gate', '--value', 'false']);
    assert(result.status === 0, `set should succeed, got ${result.status}: ${result.stderr}`);
    assert(!fs.existsSync(trackedConfigPath(dir)), 'guards.* set must never CREATE the tracked config.json');
    assert(fs.existsSync(localConfigFilePath(dir)), 'guards.* set must land in .bee/config.local.json');
    const overlay = readJson(localConfigFilePath(dir), null);
    assert(overlay && overlay.guards && overlay.guards.idle_gate === false, `overlay should carry guards.idle_gate=false, got ${JSON.stringify(overlay)}`);
  });
});

await check('config set --key guards.* --value ... leaves a PRE-EXISTING tracked config.json byte-identical', async () => {
  await withRepo('bee-config-guard-preexisting-', async (dir) => {
    writeJsonAtomic(trackedConfigPath(dir), { product_root: 'app', commands: { verify: 'npm test' } });
    const before = fs.readFileSync(trackedConfigPath(dir));
    const result = await runBeeConfig(dir, ['set', '--key', 'guards.idle_gate', '--value', 'false']);
    assert(result.status === 0, `set should succeed, got ${result.status}: ${result.stderr}`);
    const after = fs.readFileSync(trackedConfigPath(dir));
    assert(Buffer.compare(before, after) === 0, 'tracked .bee/config.json must be byte-identical after a guards.* set');
    const overlay = readJson(localConfigFilePath(dir), null);
    assert(overlay && overlay.guards && overlay.guards.idle_gate === false, `overlay should carry guards.idle_gate=false, got ${JSON.stringify(overlay)}`);
  });
});

await check('a guards.* value set without --local is honored on read via readConfig() (overlay precedence unchanged)', async () => {
  await withRepo('bee-config-guard-readback-', async (dir) => {
    const result = await runBeeConfig(dir, ['set', '--key', 'guards.idle_gate', '--value', 'false']);
    assert(result.status === 0, `set should succeed, got ${result.status}: ${result.stderr}`);
    const config = readConfig(dir);
    assert(config.guards.idle_gate === false, `readConfig() must honor the overlay value, got ${JSON.stringify(config.guards)}`);
  });
});

await check('config unset --key guards.* (no --local) removes it from the overlay only, tracked config.json untouched', async () => {
  await withRepo('bee-config-guard-unset-', async (dir) => {
    writeJsonAtomic(trackedConfigPath(dir), { product_root: 'app' });
    const before = fs.readFileSync(trackedConfigPath(dir));
    await runBeeConfig(dir, ['set', '--key', 'guards.idle_gate', '--value', 'false']);
    const unsetResult = await runBeeConfig(dir, ['unset', '--key', 'guards.idle_gate']);
    assert(unsetResult.status === 0, `unset should succeed, got ${unsetResult.status}: ${unsetResult.stderr}`);
    const overlay = readJson(localConfigFilePath(dir), {});
    assert(overlay.guards === undefined, `guards key should be pruned from the overlay, got ${JSON.stringify(overlay)}`);
    const after = fs.readFileSync(trackedConfigPath(dir));
    assert(Buffer.compare(before, after) === 0, 'tracked .bee/config.json must stay byte-identical after a guards.* unset');
  });
});

await check('hooks.* keys route the same as guards.* (local overlay only, never tracked)', async () => {
  await withRepo('bee-config-hooks-', async (dir) => {
    writeJsonAtomic(trackedConfigPath(dir), { product_root: 'app' });
    const before = fs.readFileSync(trackedConfigPath(dir));
    const result = await runBeeConfig(dir, ['set', '--key', 'hooks.bee_write_guard', '--value', 'false']);
    assert(result.status === 0, `set should succeed, got ${result.status}: ${result.stderr}`);
    const after = fs.readFileSync(trackedConfigPath(dir));
    assert(Buffer.compare(before, after) === 0, 'tracked .bee/config.json must be byte-identical after a hooks.* set');
    const overlay = readJson(localConfigFilePath(dir), null);
    assert(overlay && overlay.hooks && overlay.hooks.bee_write_guard === false, `overlay should carry hooks.bee_write_guard=false, got ${JSON.stringify(overlay)}`);
  });
});

// ─── a pre-existing tracked guards./hooks. value: honored + warned, never migrated ─

await check('a guards.* value already sitting in the TRACKED config.json is still honored on read AND produces a warning; never auto-migrated', async () => {
  await withRepo('bee-config-guard-legacy-', async (dir) => {
    writeJsonAtomic(trackedConfigPath(dir), { guards: { idle_gate: true } });
    const before = fs.readFileSync(trackedConfigPath(dir));
    // Honored on read via the real runtime merge (readConfig()), untouched by this cell.
    assert(readConfig(dir).guards.idle_gate === true, 'a pre-existing tracked guards.* value must still be honored on read');
    // `config get` (no --local) must surface the same effective value...
    const getResult = await runBeeConfig(dir, ['get', '--key', 'guards.idle_gate']);
    assert(getResult.status === 0, `get should succeed, got ${getResult.status}: ${getResult.stderr}`);
    const parsed = JSON.parse(getResult.stdout);
    assert(parsed.value === true, `config get must honor the legacy tracked value, got ${JSON.stringify(parsed)}`);
    // ...and it must warn that the value is stuck in the tracked file.
    assert(typeof parsed.warning === 'string' && parsed.warning.length > 0, `config get must produce a one-line warning for a legacy tracked guards.* key, got ${JSON.stringify(parsed)}`);
    assert(/config\.local\.json|TRACKED/i.test(parsed.warning), `warning should point at the local overlay / tracked file, got "${parsed.warning}"`);
    // Never auto-migrated or auto-edited: the tracked file is byte-identical after the read.
    const after = fs.readFileSync(trackedConfigPath(dir));
    assert(Buffer.compare(before, after) === 0, 'reading a legacy tracked guards.* key must never rewrite .bee/config.json');
  });
});

await check('config unset --key guards.* never auto-edits a pre-existing TRACKED value (only the overlay is ever touched)', async () => {
  await withRepo('bee-config-guard-legacy-unset-', async (dir) => {
    writeJsonAtomic(trackedConfigPath(dir), { guards: { idle_gate: true } });
    const before = fs.readFileSync(trackedConfigPath(dir));
    const result = await runBeeConfig(dir, ['unset', '--key', 'guards.idle_gate']);
    assert(result.status === 0, `unset should succeed, got ${result.status}: ${result.stderr}`);
    const after = fs.readFileSync(trackedConfigPath(dir));
    assert(Buffer.compare(before, after) === 0, 'unset of a guards.* key must never touch the tracked config.json, even to remove it');
    // Legacy tracked value is still live (nothing removed it — that's the point).
    assert(readConfig(dir).guards.idle_gate === true, 'the legacy tracked value keeps working; unset only ever targets the overlay');
  });
});

// ─── non-guard/hook namespaces are unaffected — current tracked-file home ──

await check('a non-guard/hook key (e.g. commands.verify) still writes to the tracked config.json as before', async () => {
  await withRepo('bee-config-nonguard-', async (dir) => {
    const result = await runBeeConfig(dir, ['set', '--key', 'commands.verify', '--value', 'npm test', '--string']);
    assert(result.status === 0, `set should succeed, got ${result.status}: ${result.stderr}`);
    const tracked = readJson(trackedConfigPath(dir), null);
    assert(tracked && tracked.commands && tracked.commands.verify === 'npm test', `commands.verify should land in the tracked config, got ${JSON.stringify(tracked)}`);
    const overlay = readJson(localConfigFilePath(dir), null);
    assert(!overlay || !overlay.commands, `commands.* must not spill into the overlay, got ${JSON.stringify(overlay)}`);
  });
});

printSummaryAndExit();
