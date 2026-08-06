# exec-speed — learnings (captured 2026-08-06)

Twelve cells landed 2026-07-30 under decision `44242df8`: five changed runtime
behavior (`es-1`..`es-5`, group A), six changed doctrine (`es-6`..`es-11`, group
B), one refreshed managed hashes (`es-12`). The five behavior-changing cells owed
a knowledge sync that never ran. This file is that repair — and it closes the
debt with **no area-spec merge**, on purpose.

## Why no spec merge

Every group-A change targeted the runtime bee ran on at the time. That runtime
was retired; `js-parity-cleanup` removed its last structural traces on
2026-08-04. Verified against the source on 2026-08-06:

| Cell | What shipped | Status today |
|---|---|---|
| es-1 | Guarded compile cache + lazy-import split at the CLI entry point | **Gone** — no entry point of that kind exists in the source tree |
| es-2 | Compile-cache variable for verify suite children | **Gone** — the current test runner sets no environment for its children |
| es-3 | Hook fast paths: a lite enable check and a read-tool short circuit | **Partial** — a read-tool branch exists, but it *checks* (secret paths, read size) rather than allowing; no separate lite predicate ever existed in the port |
| es-4 | Multi-path reservation batch in one call | **Gone** — the reservation verb takes exactly one path per call |
| es-5 | Skip the cells scan and projection rebuild when the store is unchanged | **Gone** — the state-sync hook rebuilds unconditionally, which the bundle already documents as intended |

Writing any of this into an area spec would put behavior in the specs that no
code performs. A spec merge here would have been the wrong kind of diligence.

## What survived

Group B — the doctrine half — cost nothing to carry across the port and is still
in force: a tiny lane may execute inline, worker dispatch inlines the cell and
state instead of making the worker re-read them, the semantic judge runs once per
slice rather than once per cell, ticks are composited for small work, and a
per-cell report file is written only for a blocked, handed-off, or consult-bearing
cell. Those six decisions are the durable output of this feature.

## What generalised

One pattern:
[An optimization inherits the lifetime of what it optimizes](../../knowledge/patterns/20260806-an-optimization-inherits-the-lifetime-of-what-it-optimizes.md).
The measurements were honest and the execution was clean; the question nobody
asked was how long the substrate would exist. The feature's own record shows the
overlap with the in-flight port being weighed nowhere.

## What else the record shows, and does not

- **The numbers, for the record:** hook paths about 77ms → 58ms; a repeated
  knowledge check 216ms → 101ms; an unchanged-store sync about 179ms → 91ms;
  small verbs roughly unchanged in wall time. Baseline: about 100ms of process
  startup per invocation, 10–15 invocations per cell.
- **`es-4` was capped over two NEEDS_REVISION judge verdicts**, both overridden
  by the orchestrator; the second override is recorded as hand-verified "outside
  harness" at the user's request to wrap up. The judge's second finding — that a
  batch pre-check skipped the filter that keeps an advisory record from
  hard-denying — was a real contract concern, and the code it concerned no longer
  exists. Recorded here so the override is not silently inherited as precedent.
- **No commit shas anywhere:** the checkout carried no git, recorded as a named
  constraint in CONTEXT.md; caps recorded files and outcome only. `feature_verify`
  is still `pending` on all five traces, and the planned close-verify comparison
  against the baseline log is not visible in the cell records.

## Debt this repair leaves behind

- Nothing in the runtime. The remaining question is whether any group-B doctrine
  decision deserves an area spec of its own; the doctrine layer already carries
  lane and worker discipline, and this file does not assume the answer.
