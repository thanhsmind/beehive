# gis-8t: independent check of gis-8 (issue 159)

Checked commit 23652196b against decision 882c75e1 (D9). Parent commit: c42ebce60.

## Truths

| Truth | Result | Anchor |
| --- | --- | --- |
| A job with a terminal result frees its concurrency slot | PASS | `packages/bee-rs/crates/bee/src/herding/wave.rs:1104` (done, blocked, failed are terminal), `wave.rs:1127` (finished ids leave the live set) |
| A running job still counts toward the limit | PASS | `packages/bee-rs/crates/bee/src/herding/wave.rs:1098` (a job with no result file is skipped, so it stays live) |

## Symptom reproduction

The tests below were copied onto the parent commit c42ebce60 and run there:

- `a_job_with_a_terminal_result_frees_its_slot_but_a_running_job_still_counts` (gis-8, `wave.rs:3056`): parent `left: Live(4) right: Live(1)`, FAILED. Now ok.
- `a_question_result_keeps_its_slot_and_a_finished_pane_job_frees_it` (new edge case, `wave.rs:3074`): parent `left: Live(2) right: Live(1)`, FAILED. Now ok. It covers a tmux job keyed by `pane_id` whose latest round (2) is done after a question in round 1, and a paseo job whose only result is `question`, which still counts.

On the parent every finished job held its slot, which was the reported symptom. With the fix, finished jobs leave the count and running jobs stay in it.

## Verify

`cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herding::wave`:
`test result: ok. 95 passed; 0 failed; 0 ignored; 0 measured; 5314 filtered out`. Both test names above appear as `ok` in the run.
