---
type: bee.area
title: "Compiled runtime: per-command performance budgets and the host-real fixture floors"
description: "Speed is a gated contract, not an aspiration: per-command budgets measured spawn-inclusive over a store pinned to real sizes, both cache states reported, the reference figure recorded beside every result. Includes the status supersession to an interim 70 ms, its measured cause, and the mandatory follow-up that tightens it."
tags: [rust-runtime, performance, budgets, benchmark, fixtures]
timestamp: 2026-07-26
bee:
  id: area-rust-runtime-performance-budgets
  lifecycle: active
  areas: [rust-runtime]
  required_context: []
  decisions: [e119fc8b, a7d7b3d5]
  sources: [docs/history/rust-port/CONTEXT.md, docs/history/rust-port/reports/rust-port-15.md, docs/history/rust-port/reports/rust-port-19.md]
  authoritative_for: "rust-runtime: performance budgets and the host-real fixture floors"
---

## Purpose

The port exists for speed, so speed is a gated contract rather than an aspiration. Each command carries its own budget, measured the way an operator actually experiences it — full process lifetime, on a store the size of a real one.

## Entry Points & Triggers

- The benchmark gate runs as a command's verify step and as part of the port's own proof runs. It generates a fixture, measures each gated command, and fails when any gated series exceeds its budget.
- It refuses to measure at all if the generated fixture falls below any pinned floor.

## Data Dictionary

- **Spawn-inclusive measurement** — wall time across the whole process lifetime, startup included. The only figure that matches what a caller waits for.
- **Host-real fixture** — a generated store whose sizes are pinned to what this repository actually carries, so a green result cannot be bought by shrinking the input.
- **Cold series** — measured with the review derivation cache absent before every iteration.
- **Warm series** — measured with that cache present. The gated series.
- **Baseline** — the same command run on the reference runtime, over the same fixture, recorded beside the result for scale.
- **Budget source** — the decision that set a budget, carried inline in the report next to the number.

## Behaviors & Operations

**Measuring.** For each gated command the gate takes at least fifty runs and reports the median, the 95th percentile, the 99th and the maximum. The 95th percentile is what the budget is checked against. The reference runtime's figure for the same command on the same fixture is recorded alongside, never gated.

**Reporting cache state.** Where a command is affected by the review derivation cache, both the cold and the warm series are reported unconditionally, and the report states which one the gate was taken against. Reporting only the favourable series is not a result.

**Failing.** Exceeding a budget fails the gate, and the failure is the honest end of the run: the response is either a real reduction in work or an explicitly recorded budget change. Shrinking the fixture and widening a tolerance are both forbidden — the fixture floors exist precisely to make the first one impossible without saying so out loud.

## Business Rules

- **R1** — Budgets are per command, each with its own constant. A shared budget loose enough for the slowest command silently retires the gate on the fastest one (decision e119fc8b).
- **R2** — The startup-floor command holds the original target: 5 ms at the 95th percentile, spawn-inclusive. It measures what a process costs before doing any work, and it must stay tight for every other number to mean anything.
- **R3** — Status assembly carries a budget of 30 ms at the 95th percentile (CI smoke 90 ms), superseding both the original 5 ms target and the 70 ms interim guard that preceded the read-deduplication work (decisions e119fc8b then 58ad0b5c). The number is derived, not chosen: it is the measured warm 95th percentile rounded up to the next 5 ms multiple plus one 5 ms step of headroom, because the gate is a strict less-than and a budget equal to the measurement would fail on the run that set it.
- **R4** — The repeated-read elimination that R3's budget follows is done: each store is now read once per invocation, which took the measured warm 95th percentile from roughly 52 ms to roughly 24.5 ms. The 20 ms target named alongside the interim budget was **not** reached and is superseded rather than left standing (decision 58ad0b5c). Status never returns to the 5 ms target.
- **R5** — A command proven unable to meet its budget on the host-real fixture gets an explicitly recorded per-command budget, never a smaller fixture (D5).
- **R6** — The scheduled-runner variant of each budget is three times the developer figure, absorbing runner variance without changing what is measured (D5).

## Edge Cases Settled

- **Fixture floors are refusals, not warnings.** Generation fails below any of: the decisions journal at 700 KB, the reservation store at 600 KB, the backlog journal at 250 KB, 250 work-item records, 50 commits of history, 60 review candidates, and a 300 KB session transcript. The last three exist because ancestry derivation and crash-candidate detection cost nothing on a store with no history.
- **The measured cause of the status budget change is recorded, not inferred.** Of a warm run, roughly 40 ms is in-process and almost all of that inside the readers, with well under a millisecond of process envelope: the same large journal parsed four times, the work-item directory scanned six times, the transcript roots walked twice. Perfect elimination of the repeats floors around 13 ms in-process.
- **The review derivation cache is not what dominates.** Cold and warm series differ by less than measurement noise on the current fixture, so the cache's contribution is real but small at this candidate count; the earlier expectation of a large gap does not hold and is recorded as such rather than quietly kept.
- **The dedup's own accounting is instrumented, not asserted.** Counters live inside the shared read primitives and reader functions, and a baseline recorded what the un-deduplicated code actually read per fixture before anything was changed, so the one-read-per-store claim is a measured transition rather than a statement written afterwards. The counters ship in release; their cost measures at roughly 1.65 ns per increment, about a millionth of a status invocation.

## Open Gaps

- Two blocks now dominate what remains: the crash-recovery block at roughly 54% of a warm run and the work-item directory listing at roughly 24%. Both sit outside the read-deduplication's scope — the reads are already single — so further reduction means doing less work, not reading fewer times.
- Budgets exist today for the startup floor and status only. Every further command the port assembles needs its own budget entry when it lands.

## Pointers (implementation)

- Gate and fixture generator: `crates/queen-bench` (`--check`, `--generate`, `--self-test`).
- Budget constants: separate per-command constants with separate flags; the status gate carries its budget source string inline in the emitted report.
- Decisions: `docs/history/rust-port/CONTEXT.md` D5; supersession `e119fc8b`; cache addendum `a7d7b3d5`.
