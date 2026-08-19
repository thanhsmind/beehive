# Flaky-hazard investigation: `status_does_not_depend_on_the_pane_having_been_focused`

## Report

`packages/bee-rs/crates/fleet/tests/herdr_backend.rs::status_does_not_depend_on_the_pane_having_been_focused`
failed once during a full-workspace run: `left: Unverifiable, right: Finished`.
Passed 3/3 in isolation and on 3 subsequent full runs after that.

## What was checked in the code (no repro run needed)

Read `Stub` (`herdr_backend.rs:32-170`) and `HerdrBackend::run_herdr`/`child_path`
(`packages/bee-rs/crates/fleet/src/backend/herdr.rs:190-242`) looking for the
shared-state-between-parallel-tests shape the task named as the likely cause:

- Every `Stub::new()` gets its own scratch dir keyed on
  `std::process::id()` + a process-wide `AtomicU64` (`STUB_SEQ`) — two
  stub-backed tests running in the same test binary at the same time never
  share a `state`/`log`/`spill` directory.
- `run_herdr` sets the child's `PATH` via `Command::env` (per-spawn), never
  `std::env::set_var` (process-global). Grepped the whole workspace for
  `set_var`/`remove_var`: several exist (`bee` crate tests, `GIT_CEILING_DIRECTORIES`,
  `BEE_SESSION_ID`, etc.) but none touch `PATH`, and none are in the `fleet`
  crate — they run in a different test binary (separate process), so they
  cannot race this file's tests either way.
- No `static`/`OnceLock`/`lazy_static` in `fleet/src/` feeds `status()` —
  the only process-wide state is `STUB_SEQ` itself, which is race-safe by
  construction (atomic fetch-add, used only for naming).
- `status()` is asserted (in this very test) to make exactly one `herdr`
  call, so there is no second in-test call it could race against.

Conclusion from reading alone: no shared-state or ordering hazard exists
between the stub-backed tests in this file. `HerdrCallError` folds spawn
failure, non-zero exit, and unparseable stdout all into `Unverifiable`
(fail-closed, D7) — so the single observed failure is consistent with *any*
transient hiccup spawning the stub's `/bin/sh` child, not with a logic bug
in the stub or in `HerdrBackend`.

## Reproduction attempts

1. Built `fleet`'s `herdr_backend` test binary in `--release` once, then ran
   it 120 times (15 rounds × 8 concurrent copies, each with
   `--test-threads=32`) while a background loop forked ~200k short-lived
   `/bin/true` processes across 3 workers to add fork/exec pressure on a
   16-core / 31 GiB box. 0 failures, this test included.
2. Ran the exact reported repro command,
   `cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
   (the full workspace, matching `bee status`'s recorded `test:` command),
   3 additional times. This test passed all 3 times.

Total: 120 isolated-binary runs + 3 isolation runs (before this session,
per the task report) + 3 full-workspace runs here = well over 120 clean
passes, 0 repros.

**Side finding, out of scope**: the 3 full-workspace runs each surfaced
`herding::control_loop::tests::quick_exit_helper` (in the `bee` crate,
`crates/bee/src/herding/control_loop.rs:794`) failing consistently — a
different test, different crate, not stub-backed, not touched by this
task. Flagging it since it's a real red in this tree, but it is unrelated
to the hazard this task was scoped to and no production code was changed
to investigate or fix it.

## Cause

**Not confirmed by reproduction.** Ruled out, by reading the code, the
"shared state between parallel stub-backed tests" shape the task
hypothesized as most likely — `Stub`'s per-call unique temp dirs and
`run_herdr`'s per-spawn (not global) `PATH` make that mechanism structurally
impossible here.

Best-supported remaining theory, unconfirmed: the workspace has 225
`tests/*.rs` integration-test files, each its own binary; a genuine
full-workspace `cargo test` run launches far more concurrent subprocesses
than my 8-way stress rig did. A rare transient OS-level hiccup spawning the
stub's `sh` child (fork/exec contention, not a code race) would, by design,
surface as exactly `Unverifiable` rather than a wrong-but-plausible status —
matching the one observed failure. This is a hypothesis about environment
resource pressure, not a demonstrated defect, and I could not trigger it.

## Does a fix generalize?

If the resource-pressure theory above is correct, the mechanism is
"spawning `/bin/sh` can transiently fail under a big-enough concurrent
process fan-out" — that is a property of `run_herdr`'s one spawn call site,
shared by every stub-backed test in this file (all of them go through
`Stub` → `run_herdr`), not something specific to
`status_does_not_depend_on_the_pane_having_been_focused`. So a fix (a retry
around the spawn, say) would belong at the `run_herdr` call site and would
apply to the whole file uniformly — never to this one test alone. No such
change was made here: the cause was not confirmed, and the task scope was
investigation only, not production code changes.
