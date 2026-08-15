# default-pipeline-liveness — CONTEXT

## What was asked

Several agents run at once, each on its own feature. The user reported that a
session can claim a cell belonging to a DIFFERENT feature — someone else's
in-flight work — and asked for the hole to be closed in code rather than by a
convention agents must remember.

## What was found

`bee cells claim-next` deliberately falls back to other pipelines when the
acting session's own lane has nothing ready. That work-stealing is the intended
design, and it is guarded: a candidate lane is skipped when its files intersect
another session's active reservation, when its execution gate is unapproved,
and — GH#20 — when the lane is bound to another session with a fresh heartbeat.

The GH#20 liveness guard protects LANE records only. The DEFAULT pipeline
(`.bee/state.json`, the feature an ordinary `bee state start-feature` sets) is
pushed into the same fallback pool with no liveness check at all. The ordering
in `run_claim_next` makes this plain: the default pipeline is appended to
`pipelines` first, and the `live_owned` list is not built until afterward, then
consulted only inside the lane loop.

So a session bound to a lane, finding its own lane empty, will pool and claim
cells out of whatever feature the default pipeline currently holds — even while
another live session is actively working it.

### Why the other combinations are already safe

- Two unbound sessions cannot hold two different features at once: the default
  pipeline stores exactly one feature, and `start-feature` refuses outright
  while a prior feature is unfinished (observed live this session:
  `startFeature: refused — current phase is "swarming", not idle`). Running
  several features in parallel therefore REQUIRES lane records, which is what
  makes the unguarded default-pipeline arm reachable rather than theoretical.
- Two sessions that both use `--as-lane` and both `session bind` are covered by
  GH#20 as written.

The reachable hole is exactly the mixed case: one session on a lane, another on
the default pipeline.

## Decisions

- D1 — The default pipeline earns the same liveness protection lanes already
  have. A session record that is alive (fresh heartbeat) and NOT bound to any
  lane is, by definition, working the default pipeline; while one exists, the
  default pipeline is not poolable by anyone else.
- D2 — The acting session never blocks itself. An unbound acting session
  already resolves the default pipeline as its OWN pipeline, and the existing
  `own_feature` comparison keeps it from being pushed as a fallback at all, so
  the new check must not introduce a second, contradictory self-exclusion.
- D3 — Same failure discipline as the surrounding code: the session-record read
  already propagates its error, and this change reuses that one read rather
  than adding a second walk with different semantics.
- D4 — Proof is a real multi-process race, not a single-process simulation.
  `packages/bee-rs/crates/bee/tests/concurrency.rs` already races 8 OS
  processes per scenario; this fix is proven there, in the suite built for
  exactly this class of claim.

## Out of scope

Two adjacent findings from the same audit, recorded so they are not lost and
deliberately not fixed here:

- `claim-next` picks exactly one candidate and, on losing the claim race, fails
  the whole invocation instead of falling through to the next ready candidate.
  Ownership is never double-granted — the O_EXCL claim file guarantees one
  winner — so this costs a retry, not correctness.
- Reservations remain voluntary: nothing forces a worker to reserve its cell's
  files before writing, so the file-overlap protections only bite when an agent
  chose to reserve. Closing that is the `guards.write_policy: shared-disjoint`
  switch, a separate decision the user has not taken.
