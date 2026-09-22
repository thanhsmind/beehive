# CI green fixes: Windows split lock and the Pi switch wait — Context

**Feature slug:** ci-green-fixes
**Date:** 2026-09-22
**Scope:** Tiny (fix-first, two disjoint cells)

## What was asked

The user asked "fix lỗi Windows". After the two test-only fixes
(`windows-cfg-unix`, `windows-json-path`) the Windows job reached the
suite and one test still fails; the Linux CI job on the same commit fails
on a different, long-flaky test.

## What was found

- Windows, run 35718068205: `herding::split_lock::tests::concurrent_acquires_are_mutually_exclusive`
  panics at `split_lock.rs:421` because `acquire` returned
  `Err("herding split lock: …herding-pane-split.lock: Access is denied. (os error 5)")`.
  `try_acquire` (`split_lock.rs`) maps only `AlreadyExists` to busy; on
  Windows a just-deleted lock file is delete-pending for a moment and
  `create_new` on it fails with `ERROR_ACCESS_DENIED`. The store lock this
  module copies already treats `PermissionDenied` as Windows-transient
  (`hooks/prompt_context.rs:1736`).
- Linux, run 35718068277 (and 35716164227, 35568900584, 35474190730 before
  it): `shell_tool_result_captures_marker_and_agent_settled_submits_private_command`
  fails `deferred command must execute switchSession` (left 0, right 1) at
  `tests/pi_plugin_contracts.rs:5222`. The fixture waits `sleep_step(50)`
  while the harness runs the command from `setTimeout(…, 0)` through an
  async handler; the harness already has a polling `await_messages` step and
  a `positive_wait_ms()` window meant for exactly this.

## What will be done

1. `split_lock::try_acquire`: under `cfg(windows)`, a `PermissionDenied`
   from `create_new` returns `Ok(false)` (busy) so the caller retries.
2. `pi_plugin_contracts.rs`: add an `await_switches` harness step mirroring
   `await_messages`, and use it in the one flaky fixture with
   `positive_wait_ms()`.

## Locked Decisions

| ID | Store ID | Decision |
|----|----------|----------|
| D1 | `cf294aef-36e1-4f09-be18-fbae00cdc964` | Windows `PermissionDenied` on `create_new` reads as busy in the split lock. |
| D2 | `4792a92e-2064-48ee-aae3-4f0bf371ac16` | The Pi harness gains `await_switches`; the flaky fixture polls for its switch. |

## Out of Scope

- The other `sleep_step(50)` fixtures in the same file that have not failed
  in CI; filed to the backlog for a follow-up sweep.
