---
type: bee.area
title: Decision Memory — the unified backlog store (event-sourced PBI records)
description: "How a product backlog item lives as an append-only event record in the same stream as machine friction/grooming events, how its current state is derived by folding those events, and why docs/backlog.md is a generated view no session ever hand-edits."
timestamp: 2026-07-23
bee:
  id: decision-memory-backlog-store
  lifecycle: active
  areas: [decision-memory]
  required_context: [areas/decision-memory/overview.md]
  decisions: ["backlog-unification D1-D6 v2 (c023ee98, supersedes v1 d2b8b94f)", "D8 (scribing, decision 0007 pattern)", D9, D11a, D11b, D13]
  sources: [docs/history/backlog-unification/CONTEXT.md, .bee/bin/lib/backlog.mjs, "cells bu-1, bu-2, bu-3, bu-4 (backlog-unification, capped, verified; traces in `.bee/cells/`)"]
  authoritative_for: "decision-memory: the unified backlog store (event-sourced PBI records)"
---

# Decision Memory — The Unified Backlog Store (Event-Sourced PBI Records)

## Purpose

A hand-edited product-backlog table under concurrent sessions produces exactly the failure a
single append-only log exists to prevent: two sessions each compute "the next free id" from a
stale snapshot and both commit it, producing a duplicate row nothing catches until a human
notices. The store this concept describes closes that gap by making the product backlog just
another event kind in the store that already never had this problem — `.bee/backlog.jsonl`, the
same append-only stream that already carries machine friction and grooming events. One store,
one owner, generated ids that never collide however many sessions run at once.

## Data Dictionary

- **PBI event** — `{ts, kind:"pbi", event:"add"|"status"|"amend", id, title?, cos?, status?,
  feature?}`, appended to `.bee/backlog.jsonl` alongside (never mixed with) friction/grooming
  events; `collectFeedback`/the feedback digest skip `kind:"pbi"` lines explicitly.
- **Id** — a legacy row's `P<n>` carried across verbatim by the one-time migration, or a
  newly-generated `p-<8hex>` from crypto randomness (`pbi add`). No allocator ever reads-then
  -increments; a duplicate id at add time is refused defensively by the fold.
- **The fold** — current PBI state, derived fresh from every `kind:"pbi"` event for an id,
  last-event-wins per field. This is the token-cheap read (`backlog pbi list --json`) — never a
  `docs/backlog.md` parse.
- **Status enum** (`PBI_STATUSES`) — `proposed | in-flight | parked | done | declined`, five
  values. Every count and every consumer includes all five.
- **The generated view** — `docs/backlog.md`, rendered from the fold by `backlog render
  --write`; `proposed`/`in-flight`/`parked` render as full rows, `done`/`declined` collapse to
  one-line links so the view stays short forever. Deterministic: no generation timestamp, byte
  -identical for identical fold contents. `backlog render --check` reports drift without writing.

## Behaviors & Operations

- **`backlog pbi add --title --cos [--status] [--id]`** — appends an `add` event and prints the
  created id. `--status` defaults to `proposed`. `--id` exists only for migration, to preserve a
  legacy id; an agent never supplies it in ordinary use.
- **`backlog pbi status --id --to [--feature]`** — appends a `status` event flipping the PBI to
  `--to`. `--feature` optionally stamps the feature slug in the *same* event — the
  exploring D11a flip needs status and slug written together, not as two writes a crash could
  split. Refuses an unknown `--id` or an out-of-enum `--to`.
- **`backlog pbi amend --id [--title] [--cos]`** — appends an `amend` event updating title and/or
  cos. Status and feature never move through amend — flipping those is `pbi status`'s job alone,
  so a reader never has to guess which verb moved which field.
- **`backlog pbi list [--status] [--json]`** — the fold, id-sorted, optionally filtered to one
  status. This is the surface every consumer (grooming's drift audit, herding's dispatch
  candidate build, the session preamble's PBI count) reads instead of the generated file.
- **`backlog render --write|--check`** — regenerates `docs/backlog.md` from the current fold.
  `backlog rank --write` is retired: a second writer of the same view was a double-truth trap,
  so the deterministic render is the view's only writer now.
- **Write-guard enforcement** — `docs/backlog.md` is an exact-path entry in `DIRECT_EDIT_DENY`
  (same early-return branch as `.bee/state.json`); a direct `Write`/`Edit` of the file is refused,
  naming the owning verbs. The rest of `docs/` is unaffected — this is one exact key, not a
  prefix rule.

## Actors & Access

- **bee-exploring** — owns the `proposed → in-flight` flip (D11a): status and feature slug
  together, in one `pbi status` call, the moment a feature opens against a matching PBI (or
  after a fresh `pbi add` when the request never had one).
- **bee-scribing** — owns two moves: the unprompted-capture append (D8, "ghi vào backlog" is a
  detection failure, not a required prompt) via `pbi add`, and the CoS-gated close-flip to
  `done` (D11b) — every clause of the PBI's `cos` must have cited delivered evidence before the
  flip; a partial delivery gets a `Delivered:`/`Remaining:` annotation via `pbi amend` instead of
  a silent full flip, with the remainder split into a new `pbi add` row when it ships
  independently.
- **bee-qualifying** — owns the `parked` flip (D13), written in the same move as the park brief
  `bee-context-locking` writes into the feature's `CONTEXT.md`.
- **bee-grooming** — audits the fold (`pbi list --json`) for three drift patterns: an
  `in-flight` PBI with no matching active feature, a `done` feature with no PBI at all, and
  duplicate PBIs describing one story — each files as a tiny, prose-ruled fix cell, never a hook
  (D7).
- **bee-herding's dispatch role** — reads a PBI's `feature` field directly from the fold to
  find its slug (D1(a) readiness), and its `status` field for the `in-flight` gate (D1(b)) — both
  reads the fold, never `docs/backlog.md`, because the fold is the current-truth surface and the
  rendered file only mirrors it after the next `render --write`.

## Business Rules

- **ONE store.** PBIs are event-sourced records in the same stream friction/grooming events
  already use; there is never a second backlog store, file, or per-item-file layout. In bee's
  own repo the stream holds bee-improvement PBIs; deployed into a host project, the identical
  stream holds that project's PBIs — the concept does not change shape between the two.
- **Ids never collide under concurrency.** Crypto-random `p-<8hex>` generation replaces
  read-then-increment entirely; the fold refuses a duplicate id defensively as a second line of
  defense, not the primary one.
- **The CLI verbs are the only writers.** `docs/backlog.md` is generated, never a source of
  truth and never hand-edited; every status flip is prose-ruled to one of exploring/scribing
  /qualifying (D7) — no hook enforces the flip itself, only the write-guard enforces that the
  generated file cannot be edited directly.
- **The status enum is five values, always.** `proposed | in-flight | parked | done | declined`;
  a consumer that only counts three (the pre-migration vocabulary) undercounts live data.
- **No validation coupling.** A cell's optional `pbi` field naming a PBI id is never a cap
  blocker when stale or missing — that is a grooming find (D9), not an execution gate.

## Edge Cases Settled

- **Legacy ids are first-class forever.** All 76 pre-migration rows kept their `P<n>` id
  verbatim through the one-time migration; cells' optional `pbi` field keeps pointing at them
  unchanged.
- **Decorated legacy statuses were normalized, never dropped.** Three rows (`P8`, `P13`, `P32`)
  carried decoration text alongside their status in the old table; migration normalized each to
  the closed enum and preserved the decoration inside the item's `cos` — nothing was silently
  discarded.
- **A cell's own `pbi` reference is advisory, not authoritative.** It exists to help a reader
  find the originating PBI; its staleness is a grooming signal, never something that blocks a
  cell from capping.

## Pointers (implementation)

- Store and fold: `foldPbis`, `pbiAdd`, `pbiStatus`, `pbiAmend`, `listPbis`, `PBI_STATUSES` in
  `.bee/bin/lib/backlog.mjs` (mirrored at `packages/bee/lib/backlog.mjs`).
- Generated view: `renderBacklogMd` in the same file; `backlog render --write|--check` in
  `.bee/bin/bee.mjs`.
- Write-guard: `DIRECT_EDIT_DENY` in `.bee/bin/lib/guards.mjs` (mirrored at
  `packages/bee/lib/guards.mjs`).
- One-time migration and its invariant check: `scripts/migrate_backlog_pbis.mjs`,
  `scripts/backlog_uniqueness.mjs`.
- Verb prose and merge rules: `skills/bee-scribing/references/scribing-reference.md`'s
  "Product Backlog" section.
