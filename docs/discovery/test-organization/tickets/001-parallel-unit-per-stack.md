---
type: research
status: closed
claimed-by: none
blocked-by: none
---

## Question

For TS/JS (Jest, Vitest), Python (pytest + xdist), Go, and Rust
(libtest, cargo-nextest): what unit does the runner schedule in
parallel, what are the 2026 defaults for the governing flags, and —
stated plainly per stack — does adding more test FILES increase
parallelism, yes or no?

This is the fact LAW 1 (parallel-first, D-588eecb5) has to be built
on. Without it the convention would tell agents to split files for
speed in stacks where that buys nothing.

## Answer

Findings: ../research/001-findings.md

There is no single answer, and that is itself the decisive result:
every stack schedules a different unit, so "split files to go
parallel" is true in two stacks, conditional in one, and false in one.
The convention must be two-layered — universal laws over a per-stack
mapping table.

- **Jest / Vitest** — the FILE is the unit. More files is a straight
  win. Jest `maxWorkers` defaults to cores-1; Vitest `fileParallelism`
  defaults to true with `pool: 'forks'`. In both, tests *inside* one
  file run sequentially unless marked concurrent.
- **pytest** — single process until `pytest-xdist`. With `-n auto` the
  unit depends on `--dist`: `load` (default) schedules individual
  tests, `loadfile` schedules whole files, `loadgroup` schedules
  `@pytest.mark.xdist_group` groups. No first-party parallel runner
  exists.
- **Go** — PACKAGES run concurrently by default (`-p`, default
  GOMAXPROCS) whether or not any test calls `t.Parallel()`. Inside a
  package only `t.Parallel()`-marked tests overlap, and only after the
  serial ones finish.
- **Rust** — the surprise. Under `cargo test`, multiple test binaries
  run **serially**, one target after another; the Cargo Book states it
  outright. Parallelism exists only in the thread pool inside a single
  binary. Splitting a `tests.rs` into submodules buys nothing, and
  splitting into more *integration-test files* makes it worse: more
  serialized binary phases. Only `cargo-nextest` changes the model, by
  giving each test its own process.

Unpinned: the exact literal default of libtest's `--test-threads` is
not restated on the official Cargo/Rust Book pages. Consistently
reported as logical CPU count, matching nextest's explicit `num-cpus`.
