# pcx-t — red/green proof for plan-conflicts-and-briefs

Checked against CONTEXT.md D1–D5 and the commit diffs, without the implementers' cap reports.

Verify command, run for every row below:

```
PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- plan_conflicts brief backlog
```

Baseline at HEAD 4f75c6190: `test result: ok. 134 passed; 0 failed; 7 ignored; 0 measured; 5371 filtered out; finished in 0.57s`

Each scratch revert was undone with `git checkout HEAD -- <file>`, and the tree was clean at the end.

## pcx-1 (D1): derive without cells reads plan.md and CONTEXT.md

Scratch revert in `plan_conflicts.rs` `derive_candidates_reported`: `let terms = plan_terms(&cells);` with no `document_terms` fallback.

- Red: `test result: FAILED. 133 passed; 1 failed; 7 ignored; 0 measured; 5371 filtered out; finished in 0.28s`. `plan_conflicts_derive_without_cells_draws_terms_from_plan_and_context` panicked at tests.rs:3292 with `left: []`.
- Green after restore: `test result: ok. 134 passed; 0 failed; 7 ignored; 0 measured; 5371 filtered out; finished in 0.23s`

## pcx-2 (D2): one verdict over a comma list of ids

Scratch revert in `apply_conflict_verdict`: single-id semantics with the signature kept. Only `ids[0]` is used, the function returns after the first match, and an unknown id is refused only when it is the first.

- Red: `test result: FAILED. 132 passed; 2 failed; 7 ignored; 0 measured; 5371 filtered out; finished in 0.23s`. Both `plan_conflicts_batch_of_ids_records_one_verdict_per_id` (tests.rs:3093) and `plan_conflicts_unknown_id_in_a_batch_refuses_the_whole_batch` (tests.rs:3113, `one unknown id must refuse the batch: ()`) failed.
- Green after restore: `test result: ok. 134 passed; 0 failed; 7 ignored; 0 measured; 5371 filtered out; finished in 0.22s`

Gap: I also tried a second scratch revert that only changes the CLI parse in `run_plan_conflicts_verdict` to `let ids = vec![id.as_str()]`, which drops the comma split. The suite stayed green (`test result: ok. 134 passed; 0 failed; ...`). No test drives `bee state plan-conflicts verdict --id a,b` end to end, so the comma split itself is unguarded.

## pcx-4 (D4): LaneBrief sub-questions and refusal text

Scratch revert: `git show ff52a6ab8 -- brief_lint.rs | git apply -R`.

- Red: `test result: FAILED. 131 passed; 3 failed; 7 ignored; 0 measured; 5371 filtered out; finished in 0.22s`. Three tests failed:
  - `brief_bulleted_sub_question_under_question_is_carried_and_a_bulleted_statement_is_refused` (tests.rs:9335, `a bullet ending in ? is a sub-question`). This covers D4a.
  - `brief_leaning_refusal_names_how_to_rewrite_a_format_constraint` (tests.rs:9355). This covers D4b.
  - `brief_section_refusal_names_the_closed_set_and_why_it_is_closed` (tests.rs:9368, `the refusal says the set is closed`). This covers D4c.
- Green after restore: `test result: ok. 134 passed; 0 failed; 7 ignored; 0 measured; 5371 filtered out; finished in 0.22s`

## pcx-5 (D5): uncheckable-CoS warning on pbi add and propose

Scratch revert in `backlog.rs`: removed the `cos_warning` call in `run_propose` and in `run_pbi_add`.

- Red: `test result: FAILED. 133 passed; 1 failed; 7 ignored; 0 measured; 5371 filtered out; finished in 0.22s`. `a_vague_cos_warns_on_add_and_propose_and_the_row_is_still_added` panicked at backlog.rs:3165.
- Green after restore: `test result: ok. 134 passed; 0 failed; 7 ignored; 0 measured; 5371 filtered out; finished in 0.22s`

`a_cos_naming_a_command_or_a_path_prints_no_warning` stays green under this revert. That is expected, because it is the negative case.

## registry_dispatch

`cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_dispatch`: `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.35s`. This includes `the_blind_check_registry_example_is_refused_as_an_unlogged_dispatch ... ok`.

## Findings outside the truths

- pcx-3 does not follow locked D3. D3 says the second `blind check` example is removed from the registry and a test asserts every registry example exits 0. What shipped instead keeps the example (`bee blind check --dossier docs/history/slp-blind-lanes/blind/example-run.md --json`), now the only one. The new test asserts that this example exits non-zero with `dossier_dispatch_unlogged`. A leader should either rule this an accepted deviation or reopen pcx-3.
- D5 asks for "the same checkable-CoS test that dispatch uses". The pcx-5 commit body says the rule existed only as dispatch prose, so `cos_names_something_checkable` in backlog.rs is new and dispatch does not share it.
- Nothing tests the D2 CLI comma split (see pcx-2 above).
