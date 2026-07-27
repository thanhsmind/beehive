// adapter_encoding_oracle.mjs — file-based node driver used by
// crates/queen-bee/tests/hook_conformance.rs (rust-port-7) to prove the
// ported `queen_bee::adapter` output-encoding functions
// (isAdvisoryEvent/encodeAdvisory/encodeBlock/emitHookOutput) match the REAL
// `.bee/bin/hooks/adapter.mjs` — never a reimplementation guess. Same
// pattern as `crates/bee-core/tests/support/mjs_oracle.mjs` (rust-port-5)
// and `lock_driver.mjs` (rust-port-3): a tracked FILE, never `node -e`.
//
// This driver imports the LIVE repo `.bee/bin/hooks/adapter.mjs` directly
// (not a seeded temp-root copy) because it exercises pure functions with no
// root/store dependency — there is no "rig root" to seed or verify sha256
// against for this fixture class.
//
// `.bee/bin/hooks/adapter.mjs` is imported, never edited (mjs frozen per D1).

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));

function findRepoRoot(startDir) {
  let dir = startDir;
  for (let i = 0; i < 12; i++) {
    if (fs.existsSync(path.join(dir, '.bee', 'bin', 'hooks', 'adapter.mjs'))) return dir;
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  throw new Error(`adapter_encoding_oracle: could not locate .bee/bin/hooks/adapter.mjs above ${startDir}`);
}

const repoRoot = findRepoRoot(here);
const adapterPath = path.join(repoRoot, '.bee', 'bin', 'hooks', 'adapter.mjs');
const { encodeAdvisory, encodeBlock, isAdvisoryEvent, emitHookOutput } = await import(`file://${adapterPath}`);

const [, , op, ...rest] = process.argv;

switch (op) {
  case 'is-advisory': {
    process.stdout.write(`${JSON.stringify({ result: isAdvisoryEvent(rest[0]) })}\n`);
    break;
  }
  case 'encode-advisory': {
    process.stdout.write(`${encodeAdvisory(rest[0])}\n`);
    break;
  }
  case 'encode-block': {
    process.stdout.write(`${encodeBlock(rest[0])}\n`);
    break;
  }
  case 'emit': {
    // rest[0] = event, rest[1] = text
    const ctx = { event: rest[0] };
    const chunks = [];
    const originalWrite = process.stdout.write.bind(process.stdout);
    process.stdout.write = (chunk) => {
      chunks.push(chunk);
      return true;
    };
    emitHookOutput(ctx, rest[1], {});
    process.stdout.write = originalWrite;
    process.stdout.write(`${JSON.stringify({ out: chunks.join('') })}\n`);
    break;
  }
  default: {
    console.error(`adapter_encoding_oracle.mjs: unknown op ${op}`);
    process.exit(2);
  }
}
