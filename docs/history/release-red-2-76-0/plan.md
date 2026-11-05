# Plan: release-red-2-76-0

Lane: small · class: bugfix · flags: proof-weakening · product files: 3

## Summary

The 2.76.0 release reached its test gate and stopped: the declared suite
went red on three tests in `-p bee --bin bee`, all from the
worker-status-digest merge. One cell turns them green without a product
behavior change: raise the flag-count pin for the locked `--step` flag,
pin one gate-approved history line as a known exception, and re-cut the
native cell prompt golden to the new template.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | The three tests fail on the worktree base. | ran | `cargo test --release -p bee --bin bee -- distinct_flag_vocabulary no_shipped_command_spelling native_cell_prompt_render_is_byte_identical` | `test result: FAILED. 0 passed; 3 failed` |
| 2 | The flag count grew by one, from the new notify flag. | read | `packages/bee-rs/crates/bee/src/herding/notify.rs:161` | `"--step" => step = it.next(),` |
| 3 | The pin constant lives in the catalog test. | read | `packages/bee-rs/crates/bee/src/catalog.rs:823` | `const PINNED_FLAG_COUNT: usize = 235;` |
| 4 | worker-status-digest D2 locks the flag name. | read | `docs/history/worker-status-digest/CONTEXT.md:19-22` | `takes \`--step\` and \`--job\`` |
| 5 | History lines are pinned, not rewritten. | read | `packages/bee-rs/crates/bee/src/hooks/cli_shape.rs:1194` | `const KNOWN_HISTORICAL_EXCEPTIONS: [&str; 19] = [` |
| 6 | The refused span is a cell truth string in that plan. | read | `docs/history/worker-status-digest/plan.md:104` | `"bee herding notify --kind progress writes progress.jsonl only` |
| 7 | The golden lacks the progress-note line the template now renders. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs:10555` | `const GOLDEN_NATIVE_CELL_PROMPT: &str = r#"Nickname (reservation identity): w-golden` |

## Cells — current slice preview

```json
[
  {
    "id": "rr76-1",
    "feature": "release-red-2-76-0",
    "title": "Turn the three 2.76.0 release-gate reds green without changing product behavior",
    "lane": "small",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["56d30fdd-b714-4cca-9562-cfd46f83557e", "b0d85440-6544-4e89-a789-b07c51258c12", "6a7ba50e-a605-4de8-ab29-a340e771e3a1"],
    "files": [
      "packages/bee-rs/crates/bee/src/catalog.rs",
      "packages/bee-rs/crates/bee/src/hooks/cli_shape.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "docs/history/release-red-2-76-0/CONTEXT.md",
      "docs/history/release-red-2-76-0/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "judge_obligation_ack": "The only hooks/ change is one entry in KNOWN_HISTORICAL_EXCEPTIONS inside the cfg(test) documented_invocations module of cli_shape.rs: test-only code, the guard's runtime refusal path is untouched.",
    "action": "RED BASE: three tests fail. Reproduce them first. (1) catalog.rs: set PINNED_FLAG_COUNT from 235 to 236 (D1, 56d30fdd). (2) hooks/cli_shape.rs: add one entry to KNOWN_HISTORICAL_EXCEPTIONS and bump the array length 19 to 20; the string is exactly the candidate the failing test prints for docs/history/worker-status-digest/plan.md (D2, b0d85440). (3) verbs/drivers/tests.rs: in GOLDEN_NATIVE_CELL_PROMPT add the line the render now emits after the 'Load the bee-swarming skill' contract line, byte for byte as the failing test's left side shows (D3, 6a7ba50e). Write no code comment. Change no product code, no worker-cell.md, no docs/history/worker-status-digest file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml",
    "must_haves": {
      "truths": [
        "the distinct flag vocabulary test passes at 236",
        "the widened CLI-shape spelling test passes on the repo docs",
        "the native cell prompt golden test passes",
        "the full declared suite is green with no filter"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/catalog.rs", "substantive": "PINNED_FLAG_COUNT is 236"},
        {"path": "packages/bee-rs/crates/bee/src/hooks/cli_shape.rs", "substantive": "one new KNOWN_HISTORICAL_EXCEPTIONS entry for the wsdg-1 truth span"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "substantive": "golden carries the progress-note contract line"}
      ],
      "key_links": ["GOLDEN_NATIVE_CELL_PROMPT matches the render of packages/bee/prompts/worker-cell.md"],
      "prohibitions": [
        "No edit to docs/history/worker-status-digest/",
        "No edit to packages/bee/prompts/worker-cell.md or herding/notify.rs",
        "No code comment"
      ]
    },
    "trace": {
      "worker": null, "outcome": null, "files_changed": [],
      "deviations": [], "friction": null, "capped_at": null,
      "behavior_change": false
    }
  }
]
```

## Proof and close

The cell's `verify` is the full declared suite with no filter. Then the
release runs again in the Paseo pane `release 2.76.0`; its test gate
runs the same suite.
