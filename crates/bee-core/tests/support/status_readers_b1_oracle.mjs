// status_readers_b1_oracle.mjs — file-based node driver used by
// crates/bee-core/tests/status_readers_b1.rs (rust-port-14) to prove
// bee_core::reviews matches the REAL, FROZEN `.bee/bin/lib/reviews.mjs`
// (listCandidates :362, listReviews :117, deriveCandidateStatus :483) —
// never this port author's reading of the mjs source. Same discipline and
// same shape as `tests/support/status_readers_a_oracle.mjs` (rust-port-13)
// and `tests/support/mjs_oracle.mjs` (rust-port-5).
//
// A tracked FILE deliberately, never `node -e`: the internals-reach guard
// denies inline evals that import bin/lib.
//
// This driver is the fine-grained oracle (per-candidate derivation). The
// COARSE oracle — the aggregated `review` block — is not reimplemented here;
// the Rust harness gets that from a real `node .bee/bin/bee.mjs status
// --json` run with cwd at the fixture, so `buildReviewBlock` (bee.mjs:408)
// is executed as-is rather than mirrored. Anything this file computed itself
// would be a second implementation, which is exactly what an oracle must not
// be.
//
// Every path this script touches is supplied by the caller (the Rust test
// harness, via argv) and MUST be a per-test temp path outside the repo's
// live `.bee/` store — this driver never defaults to or infers the repo's
// own store.
//
// usage: status_readers_b1_oracle.mjs <op> <root>

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));

function findRepoRoot(startDir) {
  let dir = startDir;
  for (let i = 0; i < 12; i++) {
    if (fs.existsSync(path.join(dir, '.bee', 'bin', 'lib', 'reviews.mjs'))) return dir;
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  throw new Error(`status_readers_b1_oracle: could not locate .bee/bin/lib/reviews.mjs above ${startDir}`);
}

const repoRoot = findRepoRoot(here);
const lib = (name) => `file://${path.join(repoRoot, '.bee', 'bin', 'lib', name)}`;

const { listCandidates, listReviews, deriveCandidateStatus } = await import(lib('reviews.mjs'));

async function main() {
  const [, , op, root] = process.argv;
  if (!op || !root) {
    console.error('usage: status_readers_b1_oracle.mjs <op> <root>');
    process.exit(2);
    return;
  }

  let result;
  switch (op) {
    case 'list-candidates':
      result = listCandidates(root);
      break;
    case 'list-reviews':
      result = listReviews(root);
      break;
    // The whole derivation pass, shaped exactly like bee.mjs's candidate
    // loop (bee.mjs:415-424): sessions fetched ONCE, one pass-local gitMemo
    // shared by every call. Returns one derived record per candidate, in
    // ledger order, so the Rust side can diff element-by-element and see
    // WHICH candidate disagrees rather than only that a count is off.
    case 'derive-all': {
      const candidates = listCandidates(root);
      const sessions = listReviews(root);
      const gitMemo = new Map();
      result = candidates.map((candidate) => deriveCandidateStatus(root, candidate, { sessions, gitMemo }));
      break;
    }
    default:
      console.error(`status_readers_b1_oracle.mjs: unknown op ${op}`);
      process.exit(2);
      return;
  }
  process.stdout.write(`${JSON.stringify(result === undefined ? null : result)}\n`);
}

main().catch((err) => {
  console.error(err.stack || String(err));
  process.exit(1);
});
