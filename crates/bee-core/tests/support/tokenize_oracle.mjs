// tokenize_oracle.mjs — file-based node driver used by
// crates/bee-core/tests/guard_support.rs (rust-port-8) to prove
// bee_core::tokenize::tokenize_command matches BOTH frozen mjs copies of
// the algorithm: `.bee/bin/hooks/tokenize-command.mjs`'s `tokenizeCommand`
// AND `.bee/bin/lib/guards.mjs`'s `tokenize` (the documented source of
// truth — the two are hand-synced duplicates, and drift between them is
// exactly what this oracle is built to surface, not paper over). A
// tracked FILE deliberately, never `node -e` (the internals-reach guard
// denies inline evals importing bin/lib; file-based imports are the paved
// path — see `.bee/bin/lib/guards.mjs`'s `checkBinLibImportBashCommand`).
//
// Reads a JSON array of command strings from stdin, runs BOTH tokenizers
// over the whole corpus in one process (avoids one node spawn per case),
// and writes `{tokenize_command: string[][], guards_tokenize: string[][]}`
// to stdout.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));

function findRepoRoot(startDir) {
  let dir = startDir;
  for (let i = 0; i < 12; i++) {
    if (fs.existsSync(path.join(dir, '.bee', 'bin', 'hooks', 'tokenize-command.mjs'))) return dir;
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  throw new Error(`tokenize_oracle: could not locate .bee/bin/hooks/tokenize-command.mjs above ${startDir}`);
}

const repoRoot = findRepoRoot(here);
const tokenizeCommandPath = path.join(repoRoot, '.bee', 'bin', 'hooks', 'tokenize-command.mjs');
const guardsPath = path.join(repoRoot, '.bee', 'bin', 'lib', 'guards.mjs');

const { tokenizeCommand } = await import(`file://${tokenizeCommandPath}`);
const { tokenize } = await import(`file://${guardsPath}`);

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

async function main() {
  const raw = await readStdin();
  const corpus = JSON.parse(raw);
  if (!Array.isArray(corpus)) {
    console.error('tokenize_oracle: stdin must be a JSON array of command strings');
    process.exit(2);
    return;
  }
  const result = {
    tokenize_command: corpus.map((cmd) => tokenizeCommand(cmd)),
    guards_tokenize: corpus.map((cmd) => tokenize(cmd)),
  };
  process.stdout.write(`${JSON.stringify(result)}\n`);
}

main().catch((err) => {
  console.error(err.stack || String(err));
  process.exit(1);
});
