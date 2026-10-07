# Plan: worker-guard-stub-stdin

## Summary

`test_belt_allows_paseo_worker_shell_call_when_stub_allows` fails on Linux CI
now and then with "did not return a verdict (no output)". The stub `bee`
exits without reading stdin; when it exits before the belt writes its JSON,
`execFileSync` throws `EPIPE` and the belt blocks. Reproduced 1 in 120 runs
under 8 parallel loops. The stable stub in `pi_plugin_contracts.rs` drains
stdin first; this plan makes the worker-guard stubs do the same.

Mode: `tiny` — 0 risk flags
Why this is the least workflow that protects the work: one test file, no assertion changes.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The belt sends the payload on stdin | read | .pi/extensions/bee-guard/hooks.ts:48 | input: JSON.stringify(payload), |
| 2 | The stable stub drains stdin | read | packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:1582 | cat > |
| 3 | The flake reproduces locally under load | ran | bash stress.sh (8 loops x 15 runs) | did not return a verdict (no output) |

## Cells (current slice)

```json
[
  {
    "id": "wgs-1",
    "feature": "worker-guard-stub-stdin",
    "title": "Drain stdin in the worker-guard test stubs",
    "lane": "tiny",
    "role": "test",
    "deps": [],
    "decisions": [
      "49c907f8-e331-4099-a1d0-afc0709d3ff2"
    ],
    "files": [
      "packages/bee-rs/crates/bee/tests/pi_worker_guard_contracts.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/tests/pi_worker_guard_contracts.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In pi_worker_guard_contracts.rs, make every sh stub bee that the belt calls with JSON on stdin read all of stdin first (cat > /dev/null) before it answers, as write_stub_bee in pi_plugin_contracts.rs does (cat > last_stdin.json). This covers the allow stub and the deny stub at least. Do not change what any test asserts. No new code comments. Proof: the cell verify, plus the stress loop (8 parallel loops x 15 runs of each worker-guard stub test) with zero failures.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_worker_guard_contracts",
    "must_haves": {
      "truths": [
        "Every worker-guard stub bee drains stdin before it answers",
        "The allow test passes 120 of 120 runs under 8 parallel loops"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/tests/pi_worker_guard_contracts.rs",
          "substantive": "cat > /dev/null in the stub hook branches"
        }
      ],
      "key_links": [
        "Linux CI green for test_belt_allows_paseo_worker_shell_call_when_stub_allows"
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
