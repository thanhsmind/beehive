---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: Make the four release-gate test flakes deterministic

## Summary

The release script's test gate cannot go green on this machine: one test
fails every time when tests run on one thread, and three Pi plugin tests fail
under parallel load. This makes all four wait for the real condition instead
of a fixed time or a fixed line shape.

Mode: `small` — 0 risk flags (tests only).
Why this is the least workflow that protects the work: two test files, no
product code, each failure reproduced.

## Requirements

- D1 (`91105ed2`): the Pi harness gains `await_notification` and
  `await_order_entry` steps; the three Pi tests wait on them.
- D2 (`91105ed2`): the job-id test finds `JOB_ID:` anywhere in a line.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | the job-id test needs JOB_ID: at line start | read | packages/bee-rs/crates/bee/src/herding/run.rs:11234 | if let Some(rest) = line.strip_prefix("JOB_ID:") { |
| 2 | single-threaded libtest prints the name first on the same line | ran | BEE_TEST_PIHP_UNIQUE_JOB_ID_CHILD=1 RUST_TEST_THREADS=1 bee-<hash> herding::run::tests::pihp_unique_job_id_child_process_helper --nocapture | test herding::run::tests::pihp_unique_job_id_child_process_helper ... JOB_ID:job- |
| 3 | the forged-marker case waits a fixed 50 ms | read | packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:6044 | sleep_step(50), |
| 4 | the two exit tests wait a fixed 300 ms | read | packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:7375 | sleep_step(300), |
| 5 | the harness already has a polling await step to copy | read | packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:901 | case "await_switches": { |

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"rtf-1 changes two test files."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests are the change; the leader runs both modes."},
    {"stage":"documentation-and-capture","classification":"not-applicable","role":"docs","reason":"The leader queues the capture stub."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"The release is its own lane."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Review is user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"Small lane: no consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Small lane: no hat wave."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Small lane: no hat wave."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Small lane: no hat wave."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Small lane: no hat wave."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Small lane: no hat wave."}
  ]
}
```

## Cells (current slice)

```json
[
  {
    "id": "rtf-1",
    "feature": "release-test-flakes",
    "title": "Wait for the real condition in the four release-gate flaky tests",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "D2", "91105ed2-ab78-44b9-968d-95c8b5798a70"],
    "files": [
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "read_first": [
      "docs/history/release-test-flakes/plan.md",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Tests only; no product code. (1) Per D2: in herding/run.rs test `pihp_unique_job_id_concurrent_cli_processes_cannot_share_default_id`, the loop `if let Some(rest) = line.strip_prefix(\"JOB_ID:\")` misses the id when tests run single-threaded, because libtest prints `test <name> ... ` on the same line before the child's `JOB_ID:<id>`. Find `JOB_ID:` anywhere in the line (e.g. `line.find(\"JOB_ID:\")` and take the rest after it, trimmed). (2) Per D1: in tests/pi_plugin_contracts.rs the node harness has polling steps `await_messages` and `await_switches` (each loops `while (<array>.length < want && Date.now() < deadline) await sleep(25)` and pushes a step result). Add two steps the same way: `await_notification` with `contains` (poll until some entry of `notifications` has a `message` that includes the string) and `await_order_entry` with `entry` (poll until `orderLog` includes that exact entry); both take `timeout_ms` (default 10000) and push a result step like `{notification_seen: bool}` / `{order_entry_seen: bool}`. Add Rust helpers next to `fn sleep_step` (e.g. `fn await_notification_step(contains: &str) -> Value` and `fn await_order_entry_step(entry: &str) -> Value`) that pass `\"timeout_ms\": positive_wait_ms()`. Then: in `deferred_transition_rejects_forged_marker` case 1 replace `sleep_step(50)` with `await_notification_step(\"deferred transition intent failed authenticity validation\")`; in `deferred_exit_ordering_settled_fork_switch_replacement_merge` replace `sleep_step(300)` with `await_order_entry_step(\"withSession_end\")`; in `exit_worktree_marker_relocates_and_no_merge_command_runs` replace `sleep_step(300)` with `await_notification_step(\"worktree was kept\")`. Keep every assertion unchanged. Update the doc line above run_harness that lists the timer-facing step kinds only if it is a doc comment you must keep true; add no new comments. Proof: run the verify (it runs both modes); run the Pi target three times in parallel mode to show it no longer flakes. No code comments.",
    "verify": "RUST_TEST_THREADS=1 PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee pihp_unique_job_id && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && RUST_TEST_THREADS=1 PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts",
    "must_haves": {
      "truths": [
        "the job-id test passes with RUST_TEST_THREADS=1 and in parallel",
        "the three Pi tests wait for their condition instead of a fixed sleep",
        "pi_plugin_contracts passes in parallel and single-threaded"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", "substantive": "await_notification and await_order_entry harness steps and their use"}
      ],
      "key_links": ["the new steps poll the same notifications and orderLog arrays the assertions read"],
      "prohibitions": ["No assertion is weakened or removed", "No product code change", "No code comments"]
    }
  }
]
```
