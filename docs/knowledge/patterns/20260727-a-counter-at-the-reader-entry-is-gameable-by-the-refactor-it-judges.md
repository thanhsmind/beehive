---
type: bee.pattern
title: "A counter placed at today's reader is gameable by the refactor it was built to judge"
description: "Read counters built to measure a deduplication sat beside one store's call sites rather than inside the shared primitive; injecting two real reads at the level the refactor would hoist to left the count unchanged and the test green. The dedup would have reported the target number while twice as many reads happened."
tags: [instrumentation, proof-discipline, refactor, counters, falsification]
timestamp: 2026-07-27
bee:
  id: pattern-20260727-a-counter-at-the-reader-entry-is-gameable
  lifecycle: active
  areas: [rust-runtime]
  required_context: []
  decisions: []
  sources: ["rust-port (cell rust-port-22, goal-check round 1 — the judge proved it by injected reads in a scratch copy)", docs/history/rust-port/reports/rust-port-22.md]
---

## The pattern

An instrument built to judge a refactor must survive that refactor. A counter placed at the function that reads a store today is bypassed the moment the refactor moves the read somewhere else — and it keeps reporting a number, which is worse than reporting nothing.

The instance: a cell built read counters so the next cell's "each store is read once" claim would be measurable. The counters for three stores sat inside the shared reader functions; the fourth sat beside the call sites inside the module that owned that store. A reviewer injected two real reads of that store at the level the next refactor was going to hoist loads to, and the baseline test printed the unchanged count and passed. Three extra real reads, invisible. The refactor being judged would have hoisted the load to exactly that level.

The failure is quiet in a specific way: the dedup would have reported "one read per store" while two reads happened — the target number reached by counting one of them and missing the other.

## What to do

- **Place counters at the lowest shared primitive, keyed by what is being read** — the file-read and directory-scan layer — so any path that touches the store increments, including a path written after the instrument. A counter at a reader's entry only measures the readers that exist today.
- **Ask the survival question explicitly before trusting the instrument**: if the change under test moves the work, does this counter still see it? Answer it per counter, not for the instrument as a whole. In the instance, four of five counters survived and one did not, and only naming them one at a time surfaced which.
- **Prove it by injection, not by argument.** Add a real extra read at the level the refactor will use and require the count to move. Keep that as a permanent test — it is the guard that the instrument still measures what its name says after the next change.
- Where keying is by path or name, probe the other direction too: a neighbouring store read through the same primitive must not increment. An instrument that over-counts is as wrong as one that under-counts, and both produce confident numbers.

Related: [[20260726-a-comparison-blind-to-the-contracts-own-dimension]] is the same failure one layer up — there the comparison could not see the contract's dimension; here the counter cannot see the code's new shape.
