// lock_driver.mjs — file-based node driver used by bee-core's Rust lock
// conformance tests (crates/bee-core/tests/lock_interop.rs, rust-port-3) to
// contend the SAME lock files through the REAL `.bee/bin/lib/lock.mjs` —
// never a reimplementation guess. This is a tracked FILE deliberately, never
// invoked as `node -e` (the internals-reach guard denies inline evals
// importing bin/lib). Same pattern as tests/support/mjs_oracle.mjs
// (rust-port-5).
//
// The path to lock.mjs arrives as argv (the Rust test resolves it by walking
// ancestors from CARGO_MANIFEST_DIR); every store root this script touches
// is supplied by the caller and MUST be a per-test temp root outside the
// repo's live `.bee/` store — a defensive check below refuses any root that
// looks like a bee checkout.
//
// `.bee/bin/lib/lock.mjs` is imported, never edited (mjs frozen per D1).
//
// Ops (one JSON line per step on stdout):
//   path <lockMjs> <root> <name>
//       print {"lockPath": lockFilePath(root, name)}
//   acquire-once <lockMjs> <root> <name>
//       acquireStoreLockOnceSync; on success release immediately;
//       print {"acquired": bool, "pid": ..., "holder": ...}
//   acquire-abandon <lockMjs> <root> <name>
//       acquire and exit WITHOUT release (crashed-holder fixture);
//       print {"acquired": bool, "pid": ..., "holder": ...}
//   hold <lockMjs> <root> <name>
//       acquire, print {"acquired": true, "pid": ...}, stay ALIVE holding
//       the lock until one line arrives on stdin, then release and print
//       {"released": true}. Denied: print {"acquired": false, "holder"},
//       exit 3.

import fs from 'node:fs';
import path from 'node:path';

const [, , op, lockMjsPath, root, name] = process.argv;

if (!op || !lockMjsPath || !root || !name) {
  console.error('usage: lock_driver.mjs <path|acquire-once|acquire-abandon|hold> <lock.mjs path> <root> <name>');
  process.exit(2);
}

// Never operate on a live bee checkout's own store: a real repo root has
// .bee/bin/lib/lock.mjs; per-test temp roots never do.
if (fs.existsSync(path.join(root, '.bee', 'bin', 'lib', 'lock.mjs'))) {
  console.error(`lock_driver.mjs: refusing to operate on what looks like a live bee checkout: ${root}`);
  process.exit(2);
}

const { acquireStoreLockOnceSync, lockFilePath } = await import(`file://${path.resolve(lockMjsPath)}`);

function emit(obj) {
  process.stdout.write(`${JSON.stringify(obj)}\n`);
}

function waitForStdinLine() {
  return new Promise((resolve) => {
    process.stdin.setEncoding('utf8');
    let buffered = '';
    const onData = (chunk) => {
      buffered += chunk;
      if (buffered.includes('\n')) {
        process.stdin.off('data', onData);
        resolve(buffered);
      }
    };
    process.stdin.on('data', onData);
    process.stdin.on('end', () => resolve(buffered));
  });
}

switch (op) {
  case 'path': {
    emit({ lockPath: lockFilePath(root, name) });
    break;
  }
  case 'acquire-once': {
    const r = acquireStoreLockOnceSync(root, name);
    if (r.acquired) {
      emit({ acquired: true, pid: process.pid, holder: null });
      r.release();
    } else {
      emit({ acquired: false, pid: process.pid, holder: r.holder ?? null });
    }
    break;
  }
  case 'acquire-abandon': {
    const r = acquireStoreLockOnceSync(root, name);
    emit({ acquired: r.acquired, pid: process.pid, holder: r.acquired ? null : (r.holder ?? null) });
    // Deliberately NO release: the exiting process leaves the lock file
    // behind, exactly the crashed-holder shape the stale-takeover path
    // exists for.
    break;
  }
  case 'hold': {
    const r = acquireStoreLockOnceSync(root, name);
    if (!r.acquired) {
      emit({ acquired: false, pid: process.pid, holder: r.holder ?? null });
      process.exit(3);
    }
    emit({ acquired: true, pid: process.pid });
    await waitForStdinLine();
    r.release();
    emit({ released: true });
    // The flowing stdin listener keeps the event loop alive — exit
    // explicitly so the parent's wait() never hangs on this child.
    process.exit(0);
  }
  default: {
    console.error(`lock_driver.mjs: unknown op ${op}`);
    process.exit(2);
  }
}
