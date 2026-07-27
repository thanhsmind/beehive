// projection_oracle.mjs — file-based node driver used by bee-core's Rust
// projection-parity tests (crates/bee-core/tests/projection_parity.rs,
// rust-port-16) to run the REAL `.bee/bin/lib/state-projection.mjs`
// rebuild verbs on a fixture store — never a reimplementation guess. This
// is a tracked FILE deliberately, never invoked as `node -e` (the
// internals-reach guard denies inline evals importing bin/lib). Same
// pattern as tests/support/mjs_oracle.mjs (rust-port-5) and
// tests/support/lock_driver.mjs (rust-port-3).
//
// Every store root this script touches is supplied by the caller (the Rust
// test harness, via argv) and MUST be a per-test temp root outside the
// repo's live `.bee/` store — a defensive check below refuses any root that
// looks like a bee checkout, same guard `lock_driver.mjs` uses.
//
// `.bee/bin/lib/state-projection.mjs` and `.bee/bin/lib/workflow-store.mjs`
// are imported, never edited (mjs frozen per D1).
//
// Ops (one JSON line on stdout):
//   rebuild-state <stateProjMjs> <root> [overridesJson]
//       rebuildStateProjection(root, overrides); print {authoritative, source}
//   rebuild-lane <stateProjMjs> <root> <feature>
//       rebuildLaneProjection(root, feature); print {authoritative, source}
//   rebuild-handoff <stateProjMjs> <root>
//       rebuildHandoffProjection(root); print {authoritative, source}
//   rebuild-all <stateProjMjs> <root>
//       rebuildAllProjections(root); print {state:{authoritative,source},
//         handoff:{authoritative,source}, lanes:[{authoritative,source}, ...]}

import fs from 'node:fs';
import path from 'node:path';

const [, , op, stateProjPath, root, ...rest] = process.argv;

if (!op || !stateProjPath || !root) {
  console.error(
    'usage: projection_oracle.mjs <rebuild-state|rebuild-lane|rebuild-handoff|rebuild-all> <state-projection.mjs path> <root> [args...]',
  );
  process.exit(2);
}

// Never operate on a live bee checkout's own store: a real repo root has
// .bee/bin/lib/state-projection.mjs; per-test temp roots never do.
if (fs.existsSync(path.join(root, '.bee', 'bin', 'lib', 'state-projection.mjs'))) {
  console.error(`projection_oracle.mjs: refusing to operate on what looks like a live bee checkout: ${root}`);
  process.exit(2);
}

const { rebuildStateProjection, rebuildLaneProjection, rebuildHandoffProjection, rebuildAllProjections } = await import(
  `file://${path.resolve(stateProjPath)}`
);

function emit(obj) {
  process.stdout.write(`${JSON.stringify(obj)}\n`);
}

function toProjResult(r) {
  return { authoritative: r.authoritative, source: r.source ?? null };
}

switch (op) {
  case 'rebuild-state': {
    const overrides = rest[0] ? JSON.parse(rest[0]) : {};
    const r = rebuildStateProjection(root, overrides);
    emit(toProjResult(r));
    break;
  }
  case 'rebuild-lane': {
    const feature = rest[0];
    if (!feature) {
      console.error('rebuild-lane requires a feature argument');
      process.exit(2);
    }
    const r = rebuildLaneProjection(root, feature);
    emit(toProjResult(r));
    break;
  }
  case 'rebuild-handoff': {
    const r = rebuildHandoffProjection(root);
    emit(toProjResult(r));
    break;
  }
  case 'rebuild-all': {
    const r = rebuildAllProjections(root);
    emit({
      state: toProjResult(r.state),
      handoff: toProjResult(r.handoff),
      lanes: r.lanes.map(toProjResult),
    });
    break;
  }
  default: {
    console.error(`projection_oracle.mjs: unknown op ${op}`);
    process.exit(2);
  }
}
