# Plan: Paseo occupancy test uses an explicit pane set

## Summary

CI run 37499114379 is red on `main`: the test
`occupancy_counts_labelled_non_archived_paseo_agents` passed `None` for the
live-pane override, so it read the machine's real herdr or tmux panes. On a
desktop with panes it is green; on CI it reads `Fallback(1)` instead of
`Live(1)`. The fix gives the test an explicit empty pane set, as its sibling
test already does.

Mode: `tiny` — 0 risk flags: none
Why this is the least workflow that protects the work: one test call in one
file, no product code.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The test passes no pane override | read | `packages/bee-rs/crates/bee/src/herding/wave.rs:2584` | `let (exit, occ, _) = occupancy_with_panes_and_paseo(&["--main-root", root_str], None, Some(&fake));` |
| 2 | No override means the real pane list is read | read | `packages/bee-rs/crates/bee/src/herding/wave.rs:1049` | `None => live_pane_ids(&main_root),` |
| 3 | The sibling test already passes an explicit set | read | `packages/bee-rs/crates/bee/src/herding/wave.rs:2620` | `Some(Some(live_panes)),` |
| 4 | CI reads Fallback where the test expects Live | ran | `gh run view 37499114379 --log-failed` | `Fallback(1)` |

## Cells (current slice)

```json
[
  {
    "id": "otpf-1",
    "feature": "occupancy-test-pane-fixture",
    "title": "Give the paseo occupancy test an explicit empty pane set",
    "lane": "tiny",
    "role": "test",
    "deps": [],
    "decisions": [
      "2d6e4255-7f37-4995-a28f-00b9dd0c9933"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/wave.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/herding/wave.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In herding::wave::tests::occupancy_counts_labelled_non_archived_paseo_agents, pass Some(Some(HashSet::new())) as the live-pane override instead of None, so the test never reads the machine's real panes. The Paseo fake then supplies the one live agent and occupancy is Live(1) on every machine. No product code changes.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee herding::wave::tests::occupancy",
    "must_haves": {
      "truths": [
        "occupancy_counts_labelled_non_archived_paseo_agents passes an explicit pane set and asserts Live(1)",
        "every herding::wave occupancy test is green"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/herding/wave.rs",
          "substantive": "test passes Some(Some(HashSet::new()))"
        }
      ],
      "key_links": [
        "the Paseo fake adds its live agent to the explicit pane set"
      ],
      "prohibitions": [
        "No product code change",
        "No new code comments"
      ]
    },
    "behavior_change": false
  }
]
```

## Proof

- `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee herding::wave::tests::occupancy` green, and the next CI run on `main` green.
