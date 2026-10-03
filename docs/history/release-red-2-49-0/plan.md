# Plan: release-red-2-49-0

## Summary
The 2.49.0 release suite went red on one test: five prose spans in two
closed, frozen plans look like refused `bee` command spellings. They join the
test's pinned list of historical exceptions, as earlier closed plans did.

Mode: `tiny` — 0 risk flags. One file, one test.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | the test keeps a pinned list for frozen history | read | packages/bee-rs/crates/bee/src/hooks/cli_shape.rs:1152 | const KNOWN_HISTORICAL_EXCEPTIONS: [&str; 5] = [ |
| 2 | the release suite failed on that test | ran | bash scripts/release.sh 2.49.0 | the declared test suite went RED — nothing tagged, nothing pushed |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| rr49-1 | Pin the five closed-plan spans as historical exceptions | cli_shape.rs | — | the shipped-spelling test is green | cli_shape tests |

```json
[
  {
    "id": "rr49-1",
    "feature": "release-red-2-49-0",
    "lane": "tiny",
    "role": "code",
    "change_class": "test",
    "title": "Pin the five closed-plan spans as historical exceptions",
    "deps": [],
    "decisions": ["D1", "ddc902c1-4f05-41ec-a1f8-9b605d70b236", "0bbb7726-ace1-4dac-80da-df78e17c168d"],
    "files": ["packages/bee-rs/crates/bee/src/hooks/cli_shape.rs"],
    "read_first": ["packages/bee-rs/crates/bee/src/hooks/cli_shape.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1: grow KNOWN_HISTORICAL_EXCEPTIONS in packages/bee-rs/crates/bee/src/hooks/cli_shape.rs from 5 to 10 entries, adding the five spans the release suite printed, byte for byte. Write no code comments; the why lives in decision D1.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --bin bee cli_shape",
    "must_haves": {
      "truths": ["no_shipped_command_spelling_is_refused_by_the_widened_guard passes", "every pinned exception is still refused"]
    }
  }
]
```
