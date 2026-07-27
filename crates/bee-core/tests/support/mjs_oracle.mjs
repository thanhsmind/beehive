// mjs_oracle.mjs — file-based node driver used by bee-core's Rust
// integration tests (crates/bee-core/tests/fsutil_oracle.rs, rust-port-5) to
// prove the Rust fsutil port matches the REAL `.bee/bin/lib/fsutil.mjs`
// reader/writer — never a reimplementation guess (advisor note 5). This is
// a tracked FILE deliberately, never invoked as `node -e` (the
// internals-reach guard denies inline evals importing bin/lib).
//
// Every path this script touches is supplied by the caller (the Rust test
// harness, via argv) and MUST be a per-test temp path outside the repo's
// live `.bee/` store — this driver never defaults to or infers the repo's
// own store, and never writes anywhere the caller didn't name explicitly.
//
// `.bee/bin/lib/fsutil.mjs` is imported, never edited (mjs frozen per D1).

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));

// Walk up from this file's own directory to find the repo root (the
// directory containing `.bee/bin/lib/fsutil.mjs`), so the Rust side never
// has to hardcode a relative-path depth that would break if this file
// moves. Mirrors the CARGO_MANIFEST_DIR-anchored walk rust-port-3's lock
// interop driver uses for lock.mjs.
function findRepoRoot(startDir) {
  let dir = startDir;
  for (let i = 0; i < 12; i++) {
    if (fs.existsSync(path.join(dir, '.bee', 'bin', 'lib', 'fsutil.mjs'))) return dir;
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  throw new Error(`mjs_oracle: could not locate .bee/bin/lib/fsutil.mjs above ${startDir}`);
}

const repoRoot = findRepoRoot(here);
const fsutilPath = path.join(repoRoot, '.bee', 'bin', 'lib', 'fsutil.mjs');
const { writeJsonAtomic, appendJsonl, readJson, readJsonl } = await import(
  `file://${fsutilPath}`
);

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
  const [, , op, targetPath] = process.argv;
  if (!op || !targetPath) {
    console.error('usage: mjs_oracle.mjs <write-json|append-jsonl|read-json|read-jsonl> <path>');
    process.exit(2);
    return;
  }

  switch (op) {
    case 'write-json': {
      const payload = JSON.parse(await readStdin());
      writeJsonAtomic(targetPath, payload);
      process.stdout.write('ok\n');
      break;
    }
    case 'append-jsonl': {
      const payload = JSON.parse(await readStdin());
      appendJsonl(targetPath, payload);
      process.stdout.write('ok\n');
      break;
    }
    case 'read-json': {
      const result = readJson(targetPath, null);
      process.stdout.write(`${JSON.stringify(result)}\n`);
      break;
    }
    case 'read-jsonl': {
      const result = readJsonl(targetPath);
      process.stdout.write(`${JSON.stringify(result)}\n`);
      break;
    }
    default: {
      console.error(`mjs_oracle.mjs: unknown op ${op}`);
      process.exit(2);
    }
  }
}

main().catch((err) => {
  console.error(err.stack || String(err));
  process.exit(1);
});
