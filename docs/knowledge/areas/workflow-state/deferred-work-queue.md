---
type: bee.area
title: "Workflow State — the deferred-work queue, its claim exclusivity, and the scribe/promote materialization"
description: "The one claimable queue behind deferred capture, scribing, review, and promote-proposal work — an event-sourced JSONL store with add/list/claim/release/complete, claim exclusivity proven by a real multi-process race, and a dual-condition reclaim rule — and how the two derived deferred-work scans (scribing debt across six scan copies, unapplied promote proposals) now materialize real queue records at the moment their debt is incurred, reconciled against the legacy scan through one shared clearing rule so no copy can disagree or double-report."
timestamp: 2026-08-14
bee:
  id: workflow-state-deferred-work-queue
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md, areas/workflow-state/capture-queue-and-the-blocker-threshold.md, areas/workflow-state/holds-and-the-coordination-lock.md]
  decisions: ["traceable-runs D5 (docs/history/traceable-runs/CONTEXT.md, 2026-08-14 — deferred capture, scribing, review, and promote-proposal work all become records in ONE claimable queue, each carrying feature, cells, areas, files, and a reason, plus a claim/lease, so a parallel agent absent when the item was queued can still execute it)"]
  sources: ["traceable-runs cell trun-8 (trace .bee/cells/trun-8.json, capped 2026-08-14 — new deferred_queue.rs verb module: add/list/claim/release/complete, claim exclusivity proven by a real multi-process race in tests/concurrency.rs plus a negative control)", "traceable-runs cell trun-9 (trace .bee/cells/trun-9.json, capped 2026-08-14 after two rounds of judge rework — scribe/promote materialization from the two derived scans, reconciled across all six scribing-debt scan copies through state_group::ledger::deferred_debt_cleared and the shared fold scribe_queue_cells)", "docs/history/traceable-runs/plan.md (S4, the reusable primitives: lock::acquire_store_lock, resolve_session_id, backlog.rs's append-then-fold shape, cells/claims.rs's O_EXCL claim protocol and dual-condition stale sweep)"]
  authoritative_for: "workflow-state: the deferred-work queue (.bee/deferred-queue.jsonl), its claim protocol, and the scribe/promote scan-to-record materialization"
---

# Workflow State — the deferred-work queue, its claim exclusivity, and the scribe/promote materialization

Before this, only two of the four kinds of deferred work were real stores: the
capture queue (`.bee/capture-queue.jsonl`) and review candidates
(`.bee/review-candidates.jsonl`). Scribing debt and unapplied promote
proposals were derived scans — recomputed on every call from capped-cell
traces and file mtimes — which meant they had nothing to claim and no payload:
a parallel agent absent when the debt was incurred had no record it could pick
up. This concept covers the new queue those two scans now materialize into,
and the rule that keeps the queue and the legacy scan from ever disagreeing.

## Behaviors & Operations

**The queue holds one record shape across all four kinds, event-sourced like
`backlog.rs` (D5, trun-8).** Trigger: `bee deferred-queue add`. What happens:
a record carries `kind` (`capture`/`scribe`/`review`/`promote`), `feature`,
`cells`, `areas`, `files`, and a `reason` in the queuer's own words — enough
to act on with no session memory, per D5's acceptance bar. The store is
`.bee/deferred-queue.jsonl`, folded last-event-wins per id
(`add`/`claim`/`release`/`complete`), mirroring `backlog.rs`'s `fold_pbis`
fold: an unparseable line is skipped, and a duplicate `add` for the same id
is ignored (first wins). This cell built the queue and its verbs only —
`.bee/capture-queue.jsonl` and `.bee/review-candidates.jsonl` are explicitly
**not** migrated or absorbed; CONTEXT.md's open question about whether the
unified queue eventually absorbs them stays undecided.

**Claiming one item is exclusive, proven by a real multi-process race, not an
in-process one (D5, trun-8).** Trigger: two concurrent `bee deferred-queue
claim` calls against the same item. What happens: the claim path follows
`backlog.rs`'s append-then-fold critical section — a pre-lock fold as a cheap
deterministic-refusal probe, `lock::acquire_store_lock_once` (the same
O_EXCL-backed store lock every other mutating store in this crate contends
on), a RE-fold under the lock as the actual race check, the append, then
release — so only one process's re-fold can ever see the item as claimable
and win the append. The proof is real OS child processes (never in-process
async), an exact-count assertion that exactly one wins, and a NEGATIVE
CONTROL that must itself produce a violation when exclusivity is removed —
the same discipline `test_state_projection_race.mjs` set for the workflow
projection lock (`workflow-records-and-projections.md`), so the suite cannot
pass vacuously.

**Reclaiming an already-claimed item needs BOTH conditions, never lease
expiry alone (D5, trun-8, re-deriving `cells/claims.rs`'s pattern as a
pattern, not as importable code).** Trigger: a claim attempt against an item
someone else already holds. What happens: the item is reclaimable only when
its lease has expired AND the claiming owner's own session heartbeat is
stale — a live owner mid-lease-renewal is never raced out from under itself.
Owner identity comes from `resolve_session_id`'s existing precedence chain
(flag → `BEE_SESSION_ID` → `CLAUDE_CODE_SESSION_ID` → single-live-session
adoption → none) — the exact chain `cells claim` already uses, reused
directly rather than re-derived.

**Scribing debt materializes a real `scribe` record at the moment it is
incurred, with a lazy fallback rather than a cap-time hook (D5, trun-9 —
deviation from the brief, recorded, unchanged across all three cap
rounds).** Trigger: anything that calls any of the six scribing-debt scan
functions (see the next behavior for the full list). What happens: the
brief asked for enqueue-on-cap, inside `cells/handlers_close.rs::run_cap` —
outside trun-9's declared files, so the record is instead lazily
materialized the next time `drivers::close::scribing_debt` runs, carrying
the feature, cell ids, and files — but not `areas`, which the record always
ships as an empty list (also a recorded deviation; both are Open Gaps
below, judged not blocking). What each actor observes: the SAME debt is
visible either way — a queue record for a parallel agent to claim, once
`drivers::close::scribing_debt` has run since the cell capped — but a debt
that caps and is never followed by a call into that function before the
process exits leaves no record until the next such call, not until the cap
itself.

**Promote-proposal enqueue is synchronous, at the moment `bee close` writes
the proposal file (D5, trun-9).** Trigger: `bee close`'s soft promote door
(`gates.md` R80) writing `docs/history/<feature>/promote-proposals.md`.
What happens: a `promote` record enqueues in the same call, carrying the
feature and the proposal path — `bee close`'s own write is in scope for
trun-9, unlike the cap-time trigger above.

**All SIX scribing-debt scan copies across four files reconcile through one
shared function — not three, and not five, the two counts two earlier judge
rounds each mistook for the true total (D5, trun-9).** Trigger: any of
`drivers::close::scribing_debt`, `hooks/session_preamble/store.rs::
scribing_debt` and `::global_scribing_debt`, `hooks/chain_nudge.rs::
scribing_debt`, or `verbs/status_full/cells.rs::scribing_debt` and `::
global_scribing_debt` — plus the separate promote-proposal path,
`status_full::mod::unapplied_promote_proposals` — deciding whether a given
piece of debt still counts. What happens: every one of the six scribing-debt
scans builds its own `legacy_cleared` exactly the way it always did, then
ORs it with `queue_completed` via `state_group::ledger::deferred_debt_cleared
(legacy_cleared, queue_completed) -> legacy_cleared || queue_completed` —
neither signal outranks the other. All six read the SAME queue-completed set
through one shared fold, `state_group::ledger::scribe_queue_cells`, which
wraps `deferred_queue::items_for(root, "scribe", feature)` so no caller
re-implements the read. A repo that predates the queue clears debt exactly
as it always did (a scribing/compounding stamp alone still satisfies it,
zero queue involvement); a repo using the queue clears debt by completing
the matching record. Double-reporting is prevented the same way on every
caller: each builds its own `queued_*` set from the same fold before
scanning, so a cell a queue record already names is never independently
re-materialized — the same debt is never both "found by the scan" and
"sitting open in the queue" in one report. This cell capped three times
before all six agreed: the first cap wired only `drivers::close.rs` and the
promote path, so completing a scribe record cleared `bee close`'s door but
left the session preamble and the mid-session nudge still reporting the
same debt as open; the first rework round wired those two, closing that
gap but leaving `verbs/status_full/cells.rs` — what `bee status --json` and
`bee orient`'s routing blocker actually read — still an independent copy,
so the debt went quiet everywhere except the one surface agents read on
every routing turn; the second rework round wired that copy too. Each round
found the gap the previous round's own count had missed.

## Business Rules

- R120 — A queued item's payload (`kind`, `feature`, `cells`, `areas`,
  `files`, `reason`) is complete enough to execute with no session memory —
  the D5 acceptance bar (trun-8).
- R121 — A claim is exclusive: two concurrent processes claiming the same
  item produce exactly one winner, proven by a real multi-process race with
  a negative control that must itself fail without exclusivity (trun-8).
- R122 — An already-claimed item is reclaimable only when its lease has
  expired AND the claiming owner's session heartbeat is stale — lease
  expiry alone is never sufficient (trun-8, re-deriving `cells/claims.rs`'s
  dual-condition rule).
- R123 — Scribing debt and unapplied-promote-proposal debt are each decided
  by exactly one function, `deferred_debt_cleared`, read by all SIX
  scribing-debt scan copies (`drivers::close.rs`; `hooks/
  session_preamble/store.rs` ×2; `hooks/chain_nudge.rs`; `verbs/
  status_full/cells.rs` ×2) and by the promote-proposal scan — an OR of
  "legacy stamp covers it" and "queue record completed" — so no answer can
  disagree with any other, and a debt already named by a queue record is
  never independently re-surfaced by any scan (trun-9, reached only after
  two rework rounds each found a scan copy the previous count had missed).
- R124 — `.bee/capture-queue.jsonl` and `.bee/review-candidates.jsonl` are
  untouched by this queue: no migration, no absorption, in either trun-8 or
  trun-9 (D5 scope boundary).

## Edge Cases Settled

- A debt that predates the queue entirely — no `scribe`/`promote` record
  exists for it — still surfaces through the legacy scan exactly as before;
  the queue is additive, never a replacement the legacy scan must pass
  through first.
- A `scribe` record is not created at the instant a `behavior_change` cell
  caps; it is created the next time anything scans for scribing debt. A
  session that caps such a cell and exits before any scan runs leaves the
  debt visible only through the legacy scan until the next scan
  materializes the record.

## Open Gaps

- **The materialized `scribe` record always ships an empty `areas`
  list.** `drivers/close.rs` passes `&[]` to `deferred_queue::enqueue` on
  both the scribe and promote paths — D5 asked the record to carry
  `areas`, and it does not. Judged not a blocker across all three cap
  rounds: the record still carries feature, cell ids, the union of those
  cells' declared files, and a reason, which is enough for an agent absent
  when the debt was queued to act on with no session memory (D5's actual
  bar); areas are also separately derivable in-repo
  (`verbs/knowledge/promote.rs::latest_scribing_areas`), and `areas` is
  display-only in the queue today — nothing branches on it. Not fixed;
  worth a follow-up cell if a future consumer needs it populated.
- **A `scribe` record materializes lazily on the next scan, not at cap
  time.** The brief asked for enqueue-on-cap inside
  `cells/handlers_close.rs::run_cap`, outside trun-9's declared files; the
  trigger instead lives inside `drivers::close::scribing_debt` itself. A
  behavior_change cell that caps and is never followed by a call into that
  function before the session exits is invisible to a parallel claimer
  until the next such call — a real dent in D5's "claimable the moment
  debt is incurred" bar. Not fixed; recorded as a deviation across all
  three cap rounds and explicitly not blocked on by the judge.
- **`bee --help --json`'s generated registry payload does not list `state
  gate`'s new `--actor`/`--bypass-level`/`--reason` flags** (see
  `workflow-records-and-projections.md` Pointers) — unrelated to this
  queue directly, but the same documentation-generation gap class: a
  hand-guessed CLI surface can drift from `run_gate_body`'s own enforced
  allowlist. Already filed in the backlog; not re-filed here.

## Pointers (implementation)

- Store and verbs (R120-R122): `packages/bee-rs/crates/bee/src/verbs/deferred_queue.rs`
  — `KINDS`, `QUEUE_LOCK_NAME`, the event-sourced `fold`, and
  `run_add`/`run_list`/`run_claim`/`run_release`/`run_complete`, dispatched
  from `deferred_queue::try_native` on argv `["deferred-queue", <verb>,
  ...]`. Fold/lifecycle/dual-condition-stale unit tests live inline in the
  same file (the `capture.rs`/`backlog.rs` convention); the multi-process
  claim race and its negative control live in
  `packages/bee-rs/crates/bee/tests/concurrency.rs`, the crate's
  established home for real-OS-process race proofs. Evidence: trace
  `.bee/cells/trun-8.json`.
- Reconciliation (R123): `deferred_debt_cleared` and the shared fold
  `scribe_queue_cells`, both in
  `packages/bee-rs/crates/bee/src/verbs/state_group/ledger.rs`. All six
  scribing-debt scan copies call through it: `scribing_debt` in
  `verbs/drivers/close.rs`; `scribing_debt` and `global_scribing_debt` in
  `hooks/session_preamble/store.rs`; `scribing_debt` in
  `hooks/chain_nudge.rs`; `scribing_debt` and `global_scribing_debt` in
  `verbs/status_full/cells.rs`. The promote-proposal path,
  `unapplied_promote_proposals` in `verbs/status_full/mod.rs`, reconciles
  the same way. Evidence: trace `.bee/cells/trun-9.json` — three cap
  rounds; the first wired only `drivers/close.rs` and the promote path,
  the second wired `hooks/session_preamble/store.rs` and
  `hooks/chain_nudge.rs`, the third wired `verbs/status_full/cells.rs`.
- Recorded deviations, not fixed (Open Gaps above): the materialized
  `scribe` record's empty `areas` field, and its lazy, next-scan
  materialization instead of a cap-time hook — both in
  `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs`.
- `bee deferred-queue` is not yet listed in the generated CLI help registry
  (`bee --help --all` / `--json`) — the verb dispatches and is fully
  functional (`deferred_queue::try_native`), the gap is documentation
  surfacing only.
