#!/usr/bin/env node
// decisions_driver.mjs — file-based node driver for bee-core's decisions
// WRITE conformance tests (crates/bee-core/tests/decisions_lock_conformance.rs,
// rpl-4). It drives the REAL frozen `.bee/bin/lib/decisions.mjs` and
// `.bee/bin/lib/capture.mjs` — never a reimplementation guess — so the Rust
// port is compared against the oracle itself.
//
// A tracked FILE deliberately, never `node -e` (the internals-reach guard
// denies inline evals importing bin/lib). Same pattern as
// tests/support/lock_driver.mjs (rust-port-3) and tests/support/mjs_oracle.mjs
// (rust-port-5).
//
// The module path arrives as argv (the Rust test resolves it by walking
// ancestors from CARGO_MANIFEST_DIR); every store root this script touches is
// supplied by the caller and MUST be a per-test temp root outside the repo's
// live `.bee/` store — the defensive check below refuses any root that looks
// like a bee checkout.
//
// Ops (one JSON line per step on stdout, unless noted):
//   log-many <decisions.mjs> <root> <label> <count> <goFile>
//       Spin until <goFile> exists, then append <count> decide events.
//       Prints "ID <uuid>" per SUCCESSFUL append (one per line), then a final
//       JSON line {"logged": n, "busy": n, "failed": null|string}.
//       A typed DECISIONS_LOCK_BUSY refusal is counted, never fatal: a loudly
//       refused append is not a lost update, and the Rust side asserts on the
//       ids actually reported here.
//   log-once <decisions.mjs> <root> <text>
//       One logDecision attempt. Prints {"ok":true,"id":…} or
//       {"ok":false,"code":…,"message":…}.
//   archive-until <decisions.mjs> <root> <before> <goFile> <stopFile>
//       Spin until <goFile>, then loop archiveDecisions(root, {before})
//       until <stopFile> appears. This is the REWRITER the decisions lock
//       exists for: it reads the whole journal, prunes it, and renames a
//       replacement over it, so an append that is not serialized against it
//       is silently clobbered. Prints {"archived": n, "nothing": n,
//       "busy": n, "failed": null|string}.
//   sweep <decisions.mjs> <root> <id> <short8>
//       Prints sweepDecisionCitations(root, {id, short8}) as JSON.
//   capture-stub <capture.mjs> <root> <outcome> <didsJson> <filesJson> <source>
//       Calls addCaptureStub with real ARRAYS (the shape
//       handleDecisionsSupersede always passes, and the only shape rpl-3's
//       Rust port did not cover). Prints the appended stub as JSON.

import fs from 'node:fs';
import path from 'node:path';

const [, , op, modulePath, root, ...rest] = process.argv;

if (!op || !modulePath || !root) {
  console.error('usage: decisions_driver.mjs <log-many|log-once|sweep|capture-stub> <module path> <root> ...');
  process.exit(2);
}

// Never operate on a live bee checkout's own store: a real repo root has
// .bee/bin/lib/decisions.mjs; per-test temp roots never do.
if (fs.existsSync(path.join(root, '.bee', 'bin', 'lib', 'decisions.mjs'))) {
  console.error(`decisions_driver.mjs: refusing to operate on what looks like a live bee checkout: ${root}`);
  process.exit(2);
}

const mod = await import(`file://${path.resolve(modulePath)}`);

function emit(obj) {
  process.stdout.write(`${JSON.stringify(obj)}\n`);
}

function spinUntil(goFile) {
  // Same busy-wait barrier as packages/bee/tests/race_decisions_child.mjs:
  // a sleep would blur the start edge the race depends on.
  while (!fs.existsSync(goFile)) {
    /* spin */
  }
}

switch (op) {
  case 'log-many': {
    const [label, countRaw, goFile] = rest;
    const count = Number.parseInt(countRaw, 10);
    spinUntil(goFile);
    let logged = 0;
    let busy = 0;
    let failed = null;
    for (let i = 0; i < count; i += 1) {
      try {
        const event = mod.logDecision(root, {
          decision: `${label} decide ${i}`,
          rationale: 'rpl-4 mixed-runtime lock conformance fixture',
          scope: 'rpl4-race',
        });
        process.stdout.write(`ID ${event.id}\n`);
        logged += 1;
      } catch (error) {
        if (error && error.code === 'DECISIONS_LOCK_BUSY') {
          busy += 1;
          continue;
        }
        failed = error instanceof Error ? error.message : String(error);
        break;
      }
    }
    emit({ logged, busy, failed });
    process.exit(failed ? 1 : 0);
  }
  case 'archive-until': {
    const [before, goFile, stopFile] = rest;
    spinUntil(goFile);
    let archived = 0;
    let nothing = 0;
    let busy = 0;
    let failed = null;
    while (!fs.existsSync(stopFile)) {
      try {
        const result = mod.archiveDecisions(root, { before });
        archived += result.archived.length;
      } catch (error) {
        const code = error && error.code;
        if (code === 'DECISIONS_ARCHIVE_NOTHING_QUALIFIES') {
          nothing += 1;
        } else if (code === 'DECISIONS_LOCK_BUSY') {
          busy += 1;
        } else {
          failed = error instanceof Error ? error.stack : String(error);
          break;
        }
      }
      // A real archive verb is a human/scheduled action, never a tight loop.
      // Pausing between attempts (OUTSIDE the lock) gives the loggers real
      // windows to acquire it — without this the archiver alone can
      // monopolize the lock past a logger's whole bounded-retry budget,
      // which is a fixture artifact rather than a defect. Same reasoning and
      // the same 15 ms as packages/bee/tests/race_decisions_child.mjs.
      Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 15);
    }
    emit({ archived, nothing, busy, failed });
    process.exit(failed ? 1 : 0);
  }
  case 'log-once': {
    const [text] = rest;
    try {
      const event = mod.logDecision(root, { decision: text, rationale: 'rpl-4 contention probe' });
      emit({ ok: true, id: event.id });
    } catch (error) {
      emit({
        ok: false,
        code: (error && error.code) ?? null,
        message: error instanceof Error ? error.message : String(error),
      });
    }
    break;
  }
  case 'sweep': {
    const [id, short8] = rest;
    emit(mod.sweepDecisionCitations(root, { id, short8 }));
    break;
  }
  case 'capture-stub': {
    const [outcome, didsJson, filesJson, source] = rest;
    const stub = mod.addCaptureStub(root, {
      outcome,
      dids: JSON.parse(didsJson),
      files: JSON.parse(filesJson),
      source,
    });
    emit(stub);
    break;
  }
  default: {
    console.error(`decisions_driver.mjs: unknown op ${op}`);
    process.exit(2);
  }
}
