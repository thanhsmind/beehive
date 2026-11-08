---
type: bee.delivery
title: stage-gate-phase-parity — delivery
description: "Delivery record proposed by bee knowledge promote for work item stage-gate-phase-parity: 1 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-09-19
bee:
  id: stage-gate-phase-parity-delivery
  lifecycle: active
  required_context: [docs/history/stage-gate-phase-parity/plan.md]
  sources: [docs/history/stage-gate-phase-parity/plan.md, .bee/cells/archive/stage-gate-phase-parity/sgpp-1.json]
---

# stage-gate-phase-parity — Delivery

## What shipped

- **sgpp-1** — Pi stage tool gate follows the write guard's phase groups; grooming keeps edit/write, unrecognized phases narrow (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **sgpp-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --bins hooks::stage_tools` — 8 passed / 0 failed, red first; the whole hooks module also green at 626/626, and the rebuilt binary re-driven live against sandbox stores one run per phase

## Deviations

- **sgpp-1** — Ran the tiny cell inline in the main checkout rather than a feature worktree: solo session (bee status: Active workers: 1), one product file, which is the recorded tiny-fix exemption in AGENTS.md.
- **sgpp-1** — Kept the tiny lane past the JUDGE_OBLIGATION guard on hooks/ by recording judge_obligation_ack, but did NOT skip the independent read it protects: dispatched a bee-review worker, which found two P2 findings; both are fixed in this cell and the plan was revised to rev 1.
- **sgpp-1** — The approved cell verify named `--lib`, which this crate has no target for (`error: no library targets found in package bee`). Corrected the cell's verify to `--bins` via cells update before capping, rather than capping against a command that cannot run.

## Provenance

Proposed by `bee knowledge promote --work stage-gate-phase-parity` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/stage-gate-phase-parity/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

## What the independent read changed

The cell shipped twice. The first rule was written after reading one predicate
in the write guard and inferring the rest; a dispatched reviewer opened the
sibling predicates and showed the inference was false. Two findings landed, both
fixed inside this cell:

- A phase the guard does not recognize was being handed write tools, although
  the guard refuses every write under such a phase.
- The terminal states are not simply "writes allowed" — the allowance is
  path-shaped. The plan's rationale, its decision record and the source comment
  all stated otherwise and were corrected; the plan carries the correction as
  Revision 1.

The lesson is recorded as
[The predicate you found first is not the rule](../../patterns/20260919-the-predicate-you-found-first-is-not-the-rule-read-its-siblings-before-you-derive-from-it.md).
The shipped behavior is recorded as B41 in
[governed paths and the intake gate](../../areas/hook-runtime/governed-paths-and-the-intake-gate.md).

## Known gap at delivery

The source is fixed; the vendored binary the checkpoints actually execute still
carries the old rule until the next release build.
