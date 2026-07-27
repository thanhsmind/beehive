// status_readers_a_oracle.mjs — file-based node driver used by
// crates/bee-core/tests/status_readers_a.rs (rust-port-13) to prove
// bee_core's status-reader port matches the REAL `.bee/bin/lib/backlog.mjs`
// (readBacklogCounts), `.bee/bin/lib/capture.mjs` (captureQueue),
// `.bee/bin/lib/decisions.mjs` (activeDecisions), and `.bee/bin/lib/cells.mjs`
// (archivedTotals, readyCells, scribingDebt, globalScribingDebt, tierMix,
// ceilingScarcityWarning) — never a reimplementation guess (same discipline
// as `tests/support/mjs_oracle.mjs`, rust-port-5). A tracked FILE
// deliberately, never `node -e` (the internals-reach guard denies inline
// evals importing bin/lib).
//
// Every path this script touches is supplied by the caller (the Rust test
// harness, via argv) and MUST be a per-test temp path outside the repo's
// live `.bee/` store — this driver never defaults to or infers the repo's
// own store.
//
// usage: status_readers_a_oracle.mjs <op> <root> [<json-args-on-stdin>]

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));

function findRepoRoot(startDir) {
  let dir = startDir;
  for (let i = 0; i < 12; i++) {
    if (fs.existsSync(path.join(dir, '.bee', 'bin', 'lib', 'cells.mjs'))) return dir;
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  throw new Error(`status_readers_a_oracle: could not locate .bee/bin/lib/cells.mjs above ${startDir}`);
}

const repoRoot = findRepoRoot(here);
const lib = (name) => `file://${path.join(repoRoot, '.bee', 'bin', 'lib', name)}`;

const { readBacklogCounts } = await import(lib('backlog.mjs'));
const { captureQueue } = await import(lib('capture.mjs'));
const { activeDecisions } = await import(lib('decisions.mjs'));
const {
  archivedTotals,
  readyCells,
  scribingDebt,
  globalScribingDebt,
  tierMix,
  ceilingScarcityWarning,
} = await import(lib('cells.mjs'));

function readStdin() {
  return new Promise((resolve, reject) => {
    let data = '';
    process.stdin.setEncoding('utf8');
    process.stdin.on('data', (chunk) => {
      data += chunk;
    });
    process.stdin.on('end', () => resolve(data));
    process.stdin.on('error', reject);
  });
}

async function readArgs() {
  const raw = (await readStdin()).trim();
  if (!raw) return {};
  return JSON.parse(raw);
}

async function main() {
  const [, , op, root] = process.argv;
  if (!op || !root) {
    console.error('usage: status_readers_a_oracle.mjs <op> <root> [<json-args-on-stdin>]');
    process.exit(2);
    return;
  }

  let result;
  switch (op) {
    case 'backlog-counts':
      result = readBacklogCounts(root);
      break;
    case 'capture-queue':
      result = captureQueue(root);
      break;
    case 'active-decisions': {
      const args = await readArgs();
      result = activeDecisions(root, {
        recent: args.recent === undefined ? null : args.recent,
        all: Boolean(args.all),
      });
      break;
    }
    case 'archived-totals':
      result = archivedTotals(root);
      break;
    case 'ready-cells': {
      const args = await readArgs();
      result = readyCells(root, args.feature === undefined ? null : args.feature);
      break;
    }
    case 'scribing-debt': {
      const args = await readArgs();
      const opts = {};
      if (args.feature !== undefined) opts.feature = args.feature;
      if (args.sinceTs !== undefined) opts.sinceTs = args.sinceTs;
      result = scribingDebt(root, opts);
      break;
    }
    case 'global-scribing-debt':
      result = globalScribingDebt(root);
      break;
    case 'tier-mix': {
      const args = await readArgs();
      result = tierMix(root, { feature: args.feature === undefined ? null : args.feature });
      break;
    }
    case 'ceiling-scarcity':
      result = ceilingScarcityWarning(root);
      break;
    default:
      console.error(`status_readers_a_oracle.mjs: unknown op ${op}`);
      process.exit(2);
      return;
  }
  process.stdout.write(`${JSON.stringify(result === undefined ? null : result)}\n`);
}

main().catch((err) => {
  console.error(err.stack || String(err));
  process.exit(1);
});
