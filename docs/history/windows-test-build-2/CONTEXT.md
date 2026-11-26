<!-- bee:no-read-set: bugfix from the Windows CI log of run 45723393331 and a local cargo check --target x86_64-pc-windows-gnu reproduction -->

# windows-test-build-2 — context

The Windows CI job on tag v2.123.0 (run 45723393331, job verify-windows)
fails to compile the bee tests again, the same class of break that
`windows-test-build` fixed on 2026-11-12. Two new test fixtures write a
`#!/bin/sh` fake `waggledance`, mark it executable with
`std::os::unix::fs::PermissionsExt` and `Permissions::from_mode`, and
join `PATH` with `:`:

- `packages/bee-rs/crates/bee/tests/handoff_self_drive.rs:82-83`
- `packages/bee-rs/crates/bee/src/verbs/feature_group.rs:1019-1020`
  (`FakeWaggledance::install` in the unit tests)

A build error hides every other Windows failure (pattern
20260821-a-platform-job-red-on-a-build-error).

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | Gate the sh-fake test code with `#[cfg(unix)]`: `#![cfg(unix)]` on `tests/handoff_self_drive.rs`, and in `src/verbs/feature_group.rs` the `FakeWaggledance` helper, `FAKE_WAGGLEDANCE`, `FakeDispatch` and every test that uses them. Do not port them. | 4f4447ee |
| C1 | The verify-windows CI job compiles the test targets and its real Windows test result is read; runtime failures it then shows are fixed in this feature before merge. | 3487e31a |
