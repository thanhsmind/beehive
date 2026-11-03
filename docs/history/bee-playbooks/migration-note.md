# Migration note — `content` is the ninth route class

**Feature:** bee-playbooks · **Decision:** D5 (`da83bb92`) · **Cell:** bpb-2
**Change:** `ROUTE_CLASS_VALUES` grew from eight values to nine. The new value
is `content`, appended last
(`packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:356-357`).

This is a public-contract change, the same kind as `perf` in
`docs/history/pstack-adoption/migration-note.md`, and it carries the same
note. The note is written because the precedent says so, not because a break
was found — the finding below is that nothing breaks, and that finding is the
note's content.

## Home of this note

This repository has no dedicated home for contract-change notes: there is no
`CHANGELOG`, no `docs/migrations/`, and no upgrade guide. The precedent put the
note inside the feature's own history folder
(`docs/history/pstack-adoption/migration-note.md`). So this note lives at
`docs/history/bee-playbooks/migration-note.md`.

## What a host repo pinning an older bee sees

**Nothing refuses, and nothing is lost.** An older bee that meets a record
carrying `class: content` prints the value and never validates it.

The reason is where validation runs. `bee route` has two arms, and only one of
them checks the vocabulary:

- `--show` is a **pure read**. The comment stating this is at
  `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:337` —
  "``--show`` is a pure read through resolveMutationTarget." It resolves the
  record and emits it. It runs no enum check at all.
- `--set` is the **only** validating arm. The class check lives in the `--set`
  flag validation at `workflows.rs:578-579`: a value absent from
  `ROUTE_CLASS_VALUES` becomes an `--class "<v>" (must be one of …)` entry in
  the typed refusal.

So the degradation is one-directional and silent-safe:

| What the older bee does | Result |
|---|---|
| `bee route --show` on a record holding `class: content` | prints `class=content`; no validation, no refusal, exit 0 |
| the session preamble's `Route: class=… \| lane=…` line | renders `class=content` verbatim |
| `bee route --set --class content …` on the older binary | refused, typed, naming the eight values it knows |

Only the third row needs a human action, and it is the ordinary one: upgrade the
pinned bee before **writing** a `content` route. Reading an existing one needs
no upgrade.

## Why `content` was safe to add at all

A lane record's `mode` field usually carries a workflow CLASS, and two readers
(`verbs/drivers/close.rs:393-403`, `uat.rs:139-171`) fall back to reading `mode`
as a LANE only when the value it holds is itself a lane value. `content` is
absent from `ROUTE_LANE_VALUES` (`workflows.rs:358-359`), so a `mode: content`
record can never be misread as a lane. `docs` and `spike` already sit in both
vocabularies; `content` adds no new collision. The safety argument's code home
is the comment at `workflows.rs:360-368`, written for `perf`; it applies to
`content` word for word, and it was left unedited because this repository's
comment guard refuses any changed comment line. The test
`route_set_accepts_content_as_the_ninth_class`
(`packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs`) pins it: `--lane
content` must refuse.

## Rollback

Reverting the enum to eight values is safe on its own terms, but any record
already written with `class: content` then fails the next `--set` that rewrites
it. Rewrite such records to `docs` before reverting, and delete the
`### content` section of `skills/bee-planning/references/planning-reference.md`
in the same change, or `class_playbook_parity` goes red.
