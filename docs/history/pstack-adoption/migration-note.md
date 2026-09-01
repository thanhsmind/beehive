# Migration note — `perf` is the eighth route class

**Feature:** pstack-adoption · **Decision:** D2 (`1593e365`) · **Cell:** psa-3
**Change:** `ROUTE_CLASS_VALUES` grew from seven values to eight. The new value
is `perf`, appended last
(`packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:287-288`).

D2 calls this a public-contract change and requires this note. The note is
written because D2 says so, not because a break was found — the finding below
is that nothing breaks, and that finding is the note's content.

## Home of this note

This repository has no dedicated home for contract-change notes: there is no
`CHANGELOG`, no `docs/migrations/`, and no upgrade guide. The nearest existing
practice is a migration note carried inside the feature's own history folder
(`docs/history/multisession-native/plan.md:64` treats migration notes as a
release-cell item of the feature that caused them). So this note lives at
`docs/history/pstack-adoption/migration-note.md`.

## What a host repo pinning an older bee sees

**Nothing refuses, and nothing is lost.** An older bee that meets a record
carrying `class: perf` prints the value and never validates it.

The reason is where validation runs. `bee route` has two arms, and only one of
them checks the vocabulary:

- `--show` is a **pure read**. The comment stating this is at
  `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:268` —
  "``--show`` is a pure read through resolveMutationTarget." It resolves the
  record and emits it. It runs no enum check at all.
- `--set` is the **only** validating arm. The class check lives in the `--set`
  flag validation at `workflows.rs:509-512`: a value absent from
  `ROUTE_CLASS_VALUES` becomes an `--class "<v>" (must be one of …)` entry in
  the typed refusal.

So the degradation is one-directional and silent-safe:

| What the older bee does | Result |
|---|---|
| `bee route --show` on a record holding `class: perf` | prints `class=perf`; no validation, no refusal, exit 0 |
| the session preamble's `Route: class=… \| lane=…` line | renders `class=perf` verbatim |
| `bee route --set --class perf …` on the older binary | refused, typed, naming the seven values it knows |

Only the third row needs a human action, and it is the ordinary one: upgrade the
pinned bee before **writing** a `perf` route. Reading an existing one needs no
upgrade.

## Why `perf` was safe to add at all

A lane record's `mode` field usually carries a workflow CLASS, and two readers
(`verbs/drivers/close.rs:393-403`, `uat.rs:139-171`) fall back to reading `mode`
as a LANE only when the value it holds is itself a lane value. `perf` is absent
from `ROUTE_LANE_VALUES` (`workflows.rs:289-290`), so a `mode: perf` record can
never be misread as a lane. `docs` and `spike` already sit in both vocabularies;
`perf` adds no new collision. The safety argument is single-homed in the comment
at `workflows.rs:291-299`, and the fence
`packages/bee-rs/crates/bee/tests/route_class_parity.rs` holds it: a future class
value that collides with a lane name goes red there.

## Rollback

Reverting the enum to seven values is safe on its own terms, but any record
already written with `class: perf` then fails the next `--set` that rewrites it.
Rewrite such records to `bugfix` or `feature` before reverting.
