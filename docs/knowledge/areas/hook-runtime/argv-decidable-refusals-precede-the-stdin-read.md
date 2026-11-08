---
type: bee.area
title: Hook Runtime — argv-decidable refusals run before the blocking stdin read
description: "Why a hook's dispatch order refuses what argv alone can decide before it ever reads stdin: read_stdin_once has no timeout by design, so nothing argv-decidable may sit behind it."
timestamp: 2026-08-30
bee:
  id: hook-runtime-argv-decidable-refusals-precede-stdin-read
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: ["herding-activity-hook D1 (09f16084): under BEE_HERDING_WORKER=1 every hook but activity exits 0 before reading stdin"]
  sources: [packages/bee-rs/crates/bee/src/hooks/mod.rs — try_native dispatch order]
  authoritative_for: "hook-runtime: the fixed order between argv-decidable refusals and the stdin read"
  anchors: [packages/bee-rs/crates/bee/src/hooks/mod.rs]
  verified_at: 2026-08-31
  verified_commit: 9ac7eead709bd5b918cdb7ef3db662e7f0f1fca3
---

# Hook Runtime — argv-decidable refusals run before the blocking stdin read

`try_native`'s dispatch order is fixed on purpose: any refusal decidable from
argv alone — an unknown hook name, for instance — runs **before** the
blocking stdin read. `read_stdin_once` has no timeout by design (a hook's
stdin payload can legitimately be large or slow to arrive), so nothing that
could be decided without reading stdin may be placed behind that read —
doing so would make an argv-decidable refusal (fast, cheap, no dependency on
the caller ever writing to stdin) wait on a blocking call that can hang
indefinitely on a caller that never sends one.

## Business Rules

- An argv-decidable refusal is checked and, if it fires, returned before any
  stdin read is attempted. A check that needs stdin content runs only after
  argv-only checks are exhausted.

## Pointers (implementation)

- `try_native` and `read_stdin_once`:
  `packages/bee-rs/crates/bee/src/hooks/mod.rs`.
