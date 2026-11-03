# Plan: pi-tests-unix-stubs

## Summary

Eight pi belt contract tests are red on Windows CI. Each one writes a stub
`bee` or `paseo` as a POSIX sh script, and Windows cannot run a sh script
(`spawn EFTYPE`, `ENOENT`). The belt itself finds `bee.exe` on Windows, so the
product is not at fault. The repo already gates sh-stub tests to unix
(`pi_plugin_contracts.rs`, `write_stub_bee`). This plan does the same for the
eight tests; Linux CI keeps running them, and a Windows stub harness is filed
as a backlog row.

Mode: `small` — 1 risk flag: cross-platform
Why this is the least workflow that protects the work: two test files, no
assertion changes; the tests never passed on Windows, so no Windows proof is lost.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The repo gates sh stubs to unix already | read | packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:1564 | #[cfg(unix)] |
| 2 | The heartbeat tests write sh stubs | read | packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs:385 | #!/bin/sh |
| 3 | The belt finds the binary by a list of names | read | .pi/extensions/bee-guard/locate.ts:63 | for (const name of BINARY_NAMES) { |
| 4 | Windows cannot spawn the sh stub | ran | gh run view 37583701230 --log-failed | spawn EFTYPE |

## Cells (current slice)

```json
[
  {
    "id": "pts-1",
    "feature": "pi-tests-unix-stubs",
    "title": "Compile the sh-stub pi contract tests only on unix",
    "lane": "small",
    "role": "test",
    "deps": [],
    "decisions": [
      "9875b230-997a-479f-b007-8c568d7fc85a"
    ],
    "files": [
      "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs",
      "packages/bee-rs/crates/bee/tests/pi_worker_guard_contracts.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs",
      "packages/bee-rs/crates/bee/tests/pi_worker_guard_contracts.rs",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add #[cfg(unix)] to the eight tests whose assertions need a POSIX sh stub bee or paseo to run, following the precedent at pi_plugin_contracts.rs:1564 (write_stub_bee is #[cfg(unix)]): in pi_paseo_heartbeat_contracts.rs test_leader_session_start_calls_stub_paseo_once, test_heartbeat_input_with_news_returns_transform_news, test_quiet_heartbeat_settle_with_block_verdict_injects_no_nudge, test_leader_shutdown_deletes_heartbeat_and_restart_creates_one, test_leader_children_lose_the_agent_id_while_belt_spawns_keep_it, test_worker_keeps_its_id_guard_and_hidden_leader_tools; in pi_worker_guard_contracts.rs test_belt_blocks_paseo_worker_shell_call_when_stub_denies, test_belt_allows_paseo_worker_shell_call_when_stub_allows. If a helper becomes unused on Windows after this, gate it the same way so the Windows build has no dead-code warning turned error. Do not change what any test asserts. Check every other test in the two files: one that also needs a sh stub to execute (not just to exist) gets the same gate; one that passed on Windows CI stays as it is. No new code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_paseo_heartbeat_contracts --test pi_worker_guard_contracts",
    "must_haves": {
      "truths": [
        "The eight sh-stub tests are compiled only on unix",
        "Every test in the two files still runs on Linux and passes",
        "No test assertion changes"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs",
          "substantive": "#[cfg(unix)] on six tests"
        },
        {
          "path": "packages/bee-rs/crates/bee/tests/pi_worker_guard_contracts.rs",
          "substantive": "#[cfg(unix)] on two tests"
        }
      ],
      "key_links": [
        "Windows CI verify-windows no longer runs these eight tests"
      ],
      "prohibitions": [
        "No new code comments",
        "No assertion changes"
      ]
    },
    "behavior_change": false
  }
]
```
