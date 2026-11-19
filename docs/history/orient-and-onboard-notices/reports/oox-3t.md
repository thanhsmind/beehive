# oox-3t — check the knowledge notice wiring tests

Commit under check: 05d53537f (oox-3). Checked without reading the oox-3 cap
report. Source was restored with `git restore` after each scratch edit; no
source change is committed.

## Runs

Every run is `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- onboard notice knowledge::`.

| # | Scratch edit | Result |
|---|--------------|--------|
| 0 | none (baseline) | `test result: ok. 562 passed; 0 failed; 2 ignored` |
| 1 | `for n in notices::knowledge_notices(&repo_root) {` in onboard/mod.rs replaced by `for n in Vec::<String>::new() {` | `onboard::tests::onboard_notice_collection_offers_knowledge_setup_when_the_bundle_is_absent ... FAILED`, panicked at `crates/bee/src/onboard/tests.rs:1814:5` with the notice array lacking the knowledge setup offer; `test result: FAILED. 561 passed; 1 failed` |
| 2 | run 1 restored with `git restore` | `test result: ok. 562 passed; 0 failed` |
| 3 | `lines.push(upkeep_line(upkeep_codes.as_deref()));` deleted from `run_report` in verbs/knowledge/report.rs | `verbs::knowledge::tests::report_upkeep_wiring_tests::knowledge_report_prints_exactly_one_upkeep_line ... FAILED`, panicked at `crates/bee/src/verbs/knowledge/tests.rs:4346:9`, `left: 0` `right: 1`; `test result: FAILED. 561 passed; 1 failed` |
| 4 | run 3 restored with `git restore` | `test result: ok. 562 passed; 0 failed` |

## Verdict

Both wiring sites are now pinned. The gap oox-t recorded (its run 4: the onboard
call dropped with every test green) is closed, and the `upkeep_line` push in
`run_report` is closed too. Each new test fails by name under its scratch edit
and passes once the file is restored.
