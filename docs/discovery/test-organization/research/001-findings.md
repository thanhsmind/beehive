# 001 — parallel scheduling unit per stack

Answers ticket `../tickets/001-parallel-unit-per-stack.md`. Web
research, 2026-08-18, official docs preferred.

## Jest

Parallel unit is the test **file**. `maxWorkers` defaults to CPU cores
minus one for a single run, half of cores in watch mode. Files run
concurrently across worker processes; tests *within* one file run
sequentially. `--runInBand` / `-i` puts everything on the main thread.
Source: https://jestjs.io/docs/cli

## Vitest

Parallel unit is also the test **file**. `fileParallelism` defaults to
`true`; setting it `false` pins `maxWorkers` to 1. `pool` defaults to
`forks` — each file in its own child process. Tests inside one file run
sequentially unless marked `test.concurrent` / `describe.concurrent`.
Sources: https://vitest.dev/config/fileparallelism ,
https://vitest.dev/guide/parallelism

## pytest

Bare pytest is single-process. `pytest-xdist -n auto` spawns roughly
one worker per CPU. The scheduling unit depends on `--dist`:

- `load` (default once `-n` is given) — the individual **test**.
- `loadfile` — the **file**; the whole file runs on one worker.
- `loadscope` — module for functions, class for methods.
- `loadgroup` — the `@pytest.mark.xdist_group(name=...)` group.

No first-party parallel runner exists as of 2026; xdist remains the de
facto standard. Source:
https://pytest-xdist.readthedocs.io/en/stable/distribution.html

## Go

Two independent layers. `-p` is package-level and defaults to
GOMAXPROCS — packages build and run concurrently **whether or not any
test calls `t.Parallel()`**. `-parallel` also defaults to GOMAXPROCS
but applies only to `t.Parallel()`-marked tests inside one binary, and
only after that package's serial tests finish. Sources:
https://pkg.go.dev/cmd/go , go help testflag

## Rust

The decisive finding. Under `cargo test`, if a package has multiple
test targets, **each target compiles to its own executable and the
executables run serially** — Cargo Book, cargo-test. Concurrency exists
only in the thread pool inside a single binary, sized by
`--test-threads`.

Consequences for this convention:

- Splitting one `tests.rs` into submodules inside the same target:
  **zero** parallelism change. Same binary, same pool.
- Splitting into more `tests/*.rs` integration files: **worse** — each
  new file is another serialized binary phase.
- `cargo-nextest` changes the model entirely: one process per test,
  default concurrency `num-cpus`, with `test-groups` and
  `threads-required` to rate-limit or serialize tests that touch
  shared global state.

Sources: https://doc.rust-lang.org/cargo/commands/cargo-test.html ,
https://doc.rust-lang.org/book/ch11-02-running-tests.html ,
https://nexte.st/docs/running/ , https://nexte.st/book/test-groups.html

## Summary

| Stack | Parallel unit | More test files = more parallel? | Config line |
|---|---|---|---|
| Jest | test file (worker process) | yes | `maxWorkers` (default cores-1) |
| Vitest | test file (fork) | yes | `fileParallelism: true`, `pool: 'forks'` |
| pytest | test, or file/group under `loadfile`/`loadgroup` | depends on `--dist` | `pytest -n auto --dist loadfile` |
| Go | package (`-p`), plus `t.Parallel()` tests within | packages yes; test funcs only with `t.Parallel()` | `go test -p N -parallel N` |
| Rust `cargo test` | test binary runs **serially**; threads within one binary | **no** — more files is worse | `-- --test-threads=N` |
| Rust nextest | one process per test | yes | `cargo nextest run` |

## Not confirmed

The exact literal default for libtest's `--test-threads` is not
restated on the official Cargo Book or Rust Book pages fetched.
Consistently reported as logical CPU count across secondary sources,
and matching nextest's explicitly documented `num-cpus` default.
