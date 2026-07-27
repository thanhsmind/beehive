// status_readers_b2_oracle.mjs — file-based node driver used by
// crates/bee-core/tests/status_readers_b2.rs (rust-port-20) to prove
// bee_core's status-readers-B2 panel matches the REAL, FROZEN
// `.bee/bin/lib/state.mjs` / `.bee/bin/lib/source-identity.mjs` — never
// this port author's reading of the mjs source. Same discipline and same
// shape as `tests/support/status_readers_b1_oracle.mjs` (rust-port-14) and
// `tests/support/status_readers_a_oracle.mjs` (rust-port-13).
//
// A tracked FILE deliberately, never `node -e`: the internals-reach guard
// denies inline evals that import bin/lib.
//
// Two oracle DISCIPLINES in this one driver, per this cell's own
// oracle-split instruction:
//   - importable units (isKnownPhase, validateModelsConfig,
//     validateAgentFilesDrift, hasStaleAdvisorKey, readOnboarding,
//     readHandoff, bypassLevel, classifySource) get FINE-GRAINED ops that
//     call the real exported function directly;
//   - the seven bee.mjs-PRIVATE helpers (buildRecoveryBlock,
//     buildContentionSummary, buildLaneRows/buildLaneSummary,
//     computeRuntimeDrift, findRepoHive, ungrantedWorktreeNotice) have NO
//     importable home — they are oracled through the WHOLE-COMMAND driver,
//     the `status` op below, which shells out to the real
//     `node .bee/bin/bee.mjs status --json` with cwd at the given root and
//     returns the parsed JSON verbatim for the Rust side to diff
//     sub-object by sub-object. Never a hand-invented reimplementation of
//     those seven functions in this file.
//
// Every path this script touches is supplied by the caller (the Rust test
// harness, via argv/stdin) and MUST be a per-test temp path outside the
// repo's live `.bee/` store — this driver never defaults to or infers the
// repo's own store.
//
// usage: status_readers_b2_oracle.mjs <op> [root] < stdin-json

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';

const here = path.dirname(fileURLToPath(import.meta.url));

function findRepoRoot(startDir) {
  let dir = startDir;
  for (let i = 0; i < 12; i++) {
    if (fs.existsSync(path.join(dir, '.bee', 'bin', 'lib', 'state.mjs'))) return dir;
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  throw new Error(`status_readers_b2_oracle: could not locate .bee/bin/lib/state.mjs above ${startDir}`);
}

const repoRoot = findRepoRoot(here);
const lib = (name) => `file://${path.join(repoRoot, '.bee', 'bin', 'lib', name)}`;

const {
  isKnownPhase,
  validateModelsConfig,
  validateAgentFilesDrift,
  hasStaleAdvisorKey,
  readOnboarding,
  readHandoff,
  bypassLevel,
} = await import(lib('state.mjs'));
const { classifySource } = await import(lib('source-identity.mjs'));
// Reference-identity reservation fixture (this cell's must-have #4): the
// REAL `listReservations` is the oracle for both the "all" and
// "activeOnly" calls `bee.mjs`'s inline `expiredUnreleased` computation
// makes (bee.mjs:739-746) — never a hand-reimplementation of that filter.
const { listReservations } = await import(lib('reservations.mjs'));

function readStdin() {
  try {
    const text = fs.readFileSync(0, 'utf8');
    return text.trim() ? JSON.parse(text) : {};
  } catch {
    return {};
  }
}

async function main() {
  const [, , op, rootArg] = process.argv;
  if (!op) {
    console.error('usage: status_readers_b2_oracle.mjs <op> [root] < stdin-json');
    process.exit(2);
    return;
  }

  let result;
  switch (op) {
    case 'is-known-phase': {
      const { phase } = readStdin();
      result = isKnownPhase(phase);
      break;
    }
    case 'validate-models-config': {
      const { present, config } = readStdin();
      result = validateModelsConfig(present ? config : undefined);
      break;
    }
    case 'validate-agent-files-drift': {
      const { present, rawConfig } = readStdin();
      result = validateAgentFilesDrift(rootArg, present ? rawConfig : undefined);
      break;
    }
    case 'has-stale-advisor-key':
      result = hasStaleAdvisorKey(rootArg);
      break;
    case 'read-onboarding':
      result = readOnboarding(rootArg);
      break;
    case 'read-handoff':
      result = readHandoff(rootArg);
      break;
    case 'bypass-level':
      result = bypassLevel(rootArg);
      break;
    case 'classify-source': {
      const { hiveDir, homeDir } = readStdin();
      result = classifySource({ hiveDir: hiveDir ?? undefined, homeDir: homeDir ?? undefined });
      break;
    }
    case 'list-reservations': {
      const { activeOnly, now } = readStdin();
      result = listReservations(rootArg, { activeOnly: Boolean(activeOnly), now });
      break;
    }
    // WHOLE-COMMAND oracle (this cell's seven bee.mjs-private helpers):
    // the REAL CLI, run as a subprocess with cwd at `rootArg`, never a
    // hand-invented reimplementation of buildRecoveryBlock/
    // buildContentionSummary/buildLaneRows/buildLaneSummary/
    // computeRuntimeDrift/findRepoHive/ungrantedWorktreeNotice.
    case 'status': {
      const { lanesFull } = readStdin();
      const beeMjs = path.join(repoRoot, '.bee', 'bin', 'bee.mjs');
      const args = [beeMjs, 'status', '--json'];
      if (lanesFull) args.push('--lanes-full');
      const stdout = execFileSync('node', args, {
        cwd: rootArg,
        encoding: 'utf8',
      });
      result = JSON.parse(stdout);
      break;
    }
    default:
      console.error(`status_readers_b2_oracle.mjs: unknown op ${op}`);
      process.exit(2);
      return;
  }
  process.stdout.write(`${JSON.stringify(result === undefined ? null : result)}\n`);
}

main().catch((err) => {
  console.error(err.stack || String(err));
  process.exit(1);
});
