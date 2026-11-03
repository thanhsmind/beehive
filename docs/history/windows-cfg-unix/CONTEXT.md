# Windows cfg gate for the backlog fail-open test — Context

**Feature slug:** windows-cfg-unix
**Date:** 2026-09-22
**Scope:** Tiny (fix-first)

## What was asked

Nothing was asked directly. The Windows CI job on `main` has compiled red
since the `mistake-fix-at` merge; the red surfaced while watching CI after
the `no-code-comments` push and again beside the 2.45.0 release.

## What was found

GitHub run 35715737480 (`verify-windows`) fails to compile the `bee` test
binary: `error[E0433]: cannot find unix in os` at
`crates/bee/src/verbs/drivers/close.rs:5283` and two `E0599` errors on
`Permissions::from_mode` at lines 5290 and 5294. The test
`an_unwritable_backlog_warns_and_the_close_still_passes` (added by cell
mfa-4, commit e82490e22) imports `std::os::unix::fs::PermissionsExt` with
no `#[cfg(unix)]`, unlike its neighbours at `close.rs:4242` and
`close.rs:4516`. The cell's scoped proof and the Linux CI job never compile
for Windows, so the cap read green.

## What will be done

Add `#[cfg(unix)]` above `#[test]` on that one function. One attribute, no
behavior change; the test still runs on Linux and macOS.

## Locked Decisions

| ID | Store ID | Decision |
|----|----------|----------|
| D1 | `0f2e5d4e-e11f-42aa-ae01-b3689a2a2fe4` | A test that imports `std::os::unix` carries `#[cfg(unix)]`; the backlog fail-open test gets the gate its neighbours already have. |
