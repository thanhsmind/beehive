---
mode: tiny
---

# Plan: CI green fixes

## Summary

Two disjoint one-file fixes: the Windows split lock treats a delete-pending
lock as busy, and the flaky Pi fixture polls for its switch instead of
sleeping 50 ms.

Mode: `tiny` — 0 risk flags: none
Why this is the least workflow that protects the work: two files, no shared
lines, each with its own scoped test; the Windows and Linux CI runs after the
push are the deterministic net.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The Windows failure is an ERROR_ACCESS_DENIED from create_new | ran | GitHub run 35718068205, `gh run view --log-failed` | `acquire must not error: "herding split lock: C:\\Users\\RUNNER~1\\…\\herding-pane-split.lock: Access is denied. (os error 5)"` |
| 2 | try_acquire maps only AlreadyExists to busy | read | `packages/bee-rs/crates/bee/src/herding/split_lock.rs` fn try_acquire | `Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),` then `Err(e) => Err(format!("herding split lock: {}: {e}", lock_path.display())),` |
| 3 | The store lock already treats PermissionDenied as Windows-transient | read | `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:1736` | `std::io::ErrorKind::PermissionDenied \| std::io::ErrorKind::WouldBlock` |
| 4 | The Linux failure is the switch count after a 50 ms sleep, four runs in a row | ran | GitHub runs 35718068277, 35716164227, 35568900584, 35474190730 | `assertion left == right failed: deferred command must execute switchSession / left: 0 / right: 1` at `pi_plugin_contracts.rs:5222:5` |
| 5 | The harness runs the command from setTimeout 0 and already has a polling await step | read | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:803` and `:848-851` | `setTimeout(async () => {` … `}, 0);`; `case "await_messages": … while (messages.length < want && Date.now() < deadline) await sleep(25);` |

## Cells (current slice)

```json
[
  {
    "id": "cgf-1",
    "feature": "ci-green-fixes",
    "title": "Read a Windows delete-pending lock as busy in the split lock",
    "lane": "tiny",
    "role": "code",
    "deps": [],
    "decisions": ["cf294aef-36e1-4f09-be18-fbae00cdc964"],
    "files": ["packages/bee-rs/crates/bee/src/herding/split_lock.rs"],
    "read_first": ["packages/bee-rs/crates/bee/src/herding/split_lock.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In split_lock::try_acquire add a cfg(windows) match arm: Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => Ok(false), placed after the AlreadyExists arm (per D1). No comment in code; the why is D1. Nothing else changes.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding::split_lock",
    "must_haves": {"truths": ["try_acquire returns Ok(false) on a Windows PermissionDenied and the split_lock tests stay green on Linux"], "artifacts": [], "key_links": [], "prohibitions": ["Do not change unix behavior", "Do not widen the arm beyond PermissionDenied"]},
    "behavior_change": true
  },
  {
    "id": "cgf-2",
    "feature": "ci-green-fixes",
    "title": "Poll for the deferred switch in the Pi marker fixture",
    "lane": "tiny",
    "role": "test",
    "deps": [],
    "decisions": ["4792a92e-2064-48ee-aae3-4f0bf371ac16"],
    "files": ["packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"],
    "read_first": ["packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In the embedded harness JS add case \"await_switches\" beside await_messages: poll switches.length until call.count or call.timeout_ms, push step({switches_seen}). Add fn await_switches_step(count) -> Value using positive_wait_ms() beside await_injections_in_vain. In shell_tool_result_captures_marker_and_agent_settled_submits_private_command replace sleep_step(50) with await_switches_step(1) (per D2). Other fixtures unchanged.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts shell_tool_result_captures_marker",
    "must_haves": {"truths": ["the harness has an await_switches step and the marker fixture uses it with the positive-wait window"], "artifacts": [], "key_links": [], "prohibitions": ["Do not change the fixture's assertions", "Do not touch other fixtures"]},
    "behavior_change": false
  }
]
```
