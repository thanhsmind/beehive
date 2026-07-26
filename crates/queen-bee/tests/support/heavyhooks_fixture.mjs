// heavyhooks_fixture.mjs — file-based node driver used by
// crates/queen-bee/tests/heavyhooks_conformance.rs (rust-port-17) to seed
// authentic session/claim/lease/hold fixtures through the REAL
// `.bee/bin/lib/claims.mjs` and `.bee/bin/lib/lease-store.mjs` (session
// creation, path-lease acquisition) and `.bee/bin/lib/worktree-holds.mjs`
// (cross-worktree hold mirroring) — never a hand-guessed file shape for
// anything the real library already knows how to produce (hash naming for
// leases in particular). This is a tracked FILE deliberately, never invoked
// as `node -e` (the internals-reach guard denies inline evals importing
// bin/lib). Same pattern as tests/support/lock_driver.mjs and
// tests/support/projection_oracle.mjs (bee-core).
//
// Every root this script touches is supplied by the caller and MUST be a
// per-test temp root outside the repo's live `.bee/` store. Unlike
// lock_driver.mjs/projection_oracle.mjs (which operate on a BARE store with
// no `bin/lib` copy at all, so "a `bin/lib` file already exists" is a safe
// live-checkout signal there), this rig's seeded fixture roots legitimately
// carry a FULL `.bee/bin/lib` copy (the mjs hooks under test dynamically
// import their own lib modules relative to `root`) — so the discriminator
// here is a `.git` directory instead: every `tempfile::tempdir()` fixture
// root is git-less, while the repo's own live checkout always has one.
//
// Ops (one JSON line on stdout):
//   session <libDir> <root> <sessionId> <heartbeatAgoSeconds>
//       creates the session via claims.createSession, then patches
//       last_heartbeat to now - heartbeatAgoSeconds*1000 (staleness
//       control); print {file}.
//   claim <libDir> <root> <cellId> <sessionId> <claimedAgoSeconds>
//       writes claims.claimPath(root, cellId) directly (documented claim
//       shape: {cell, session, ttl_seconds, claimed_at}); print {file}.
//   lease <libDir> <root> <sessionId> <pathId> <ttlSeconds>
//       acquires a real path lease via lease-store.mjs's acquireLeases;
//       print {file} (the resolved on-disk lease file this cell's own
//       resource-hash naming produced — the Rust test reads this path
//       directly afterward, never re-deriving the hash itself).
//   hold <libDir> <root> <holder> <sessionId> <path> <ttlSeconds>
//       mirrors a cross-worktree hold via worktree-holds.mjs's mirrorHold;
//       print {file} (the ledger path).

import fs from 'node:fs';
import path from 'node:path';

const [, , op, libDir, root, ...rest] = process.argv;

if (!op || !libDir || !root) {
  console.error('usage: heavyhooks_fixture.mjs <session|claim|lease|hold> <lib dir> <root> [args...]');
  process.exit(2);
}

if (fs.existsSync(path.join(root, '.git'))) {
  console.error(`heavyhooks_fixture.mjs: refusing to operate on what looks like a live git checkout: ${root}`);
  process.exit(2);
}

function emit(obj) {
  process.stdout.write(`${JSON.stringify(obj)}\n`);
}

function libUrl(name) {
  return `file://${path.resolve(path.join(libDir, name))}`;
}

switch (op) {
  case 'session': {
    const [sessionId, agoStr] = rest;
    const { createSession, sessionPath } = await import(libUrl('claims.mjs'));
    const now = Date.now();
    const created = createSession(root, { id: sessionId, now });
    if (!created.ok) {
      console.error(`heavyhooks_fixture.mjs: createSession failed: ${JSON.stringify(created)}`);
      process.exit(1);
    }
    const file = sessionPath(root, sessionId);
    const ago = Number(agoStr);
    if (Number.isFinite(ago)) {
      const record = JSON.parse(fs.readFileSync(file, 'utf8'));
      record.last_heartbeat = new Date(now - ago * 1000).toISOString();
      fs.writeFileSync(file, `${JSON.stringify(record, null, 2)}\n`);
    }
    emit({ file });
    break;
  }
  case 'claim': {
    const [cellId, sessionId, agoStr] = rest;
    const { claimPath } = await import(libUrl('claims.mjs'));
    const now = Date.now();
    const ago = Number(agoStr) || 0;
    const file = claimPath(root, cellId);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    const record = {
      cell: cellId,
      session: sessionId,
      ttl_seconds: 3600,
      claimed_at: new Date(now - ago * 1000).toISOString(),
    };
    fs.writeFileSync(file, `${JSON.stringify(record, null, 2)}\n`);
    emit({ file });
    break;
  }
  case 'lease': {
    const [sessionId, pathId, ttlStr] = rest;
    const { acquireLeases, leasePathsDir } = await import(libUrl('lease-store.mjs'));
    const ttl = Number(ttlStr);
    const [record] = acquireLeases(root, [
      {
        type: 'path',
        id: pathId,
        mode: 'write',
        workflow_id: 'fixture-cell',
        session_id: sessionId,
        workspace_id: 'agent:fixture-worker',
        epoch: 1,
        ttl,
      },
    ]);
    // Same hashing the module itself just used to name the file — found by
    // listing the paths dir rather than re-deriving the hash a second time
    // in this script (the resource key is the only thing needed to find
    // it back: exactly one file was just created).
    const dir = leasePathsDir(root);
    const file = fs
      .readdirSync(dir)
      .map((name) => path.join(dir, name))
      .find((candidate) => {
        try {
          return JSON.parse(fs.readFileSync(candidate, 'utf8')).resource === record.resource;
        } catch {
          return false;
        }
      });
    emit({ file });
    break;
  }
  case 'hold': {
    const [holder, sessionId, holdPath, ttlStr] = rest;
    const { mirrorHold } = await import(libUrl('worktree-holds.mjs'));
    await mirrorHold(root, { path: holdPath, holder, session: sessionId, ttl: Number(ttlStr) });
    // worktree-holds.mjs's own `holdsLedgerPath` is a private (unexported)
    // helper — its module header documents the store location verbatim:
    // "<mainRoot>/.bee/runtime/cross-worktree-holds.json".
    emit({ file: path.join(root, '.bee', 'runtime', 'cross-worktree-holds.json') });
    break;
  }
  default: {
    console.error(`heavyhooks_fixture.mjs: unknown op ${op}`);
    process.exit(2);
  }
}
