# lct-5 — Prove the three contracts and check the concept sync

Verify (cell): `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- cap_one_refusal auto_settle_answers worker_full_suite` — 8 passed, 0 failed.
Also run: `cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings` — clean. The full suite was not run (the cell says never run it).

## Tests added (test modules only, no production code)

| Contract | Test | Where |
|---|---|---|
| cap-one-refusal (fab7d838) | `cap_one_refusal_names_the_mistakes_principles_and_sync_fixes_together_and_writes_nothing` | verbs/cells/tests.rs:10577 |
| cap-one-refusal | `cap_one_refusal_does_not_log_the_judge_override_when_a_later_check_refuses` | verbs/cells/tests.rs:10605 |
| auto-settle-answers (42961013) | `auto_settle_answers_cap_a_principles_arm_cell_the_sync_door_would_refuse` | herding/run.rs:12734 |
| auto-settle-answers | `auto_settle_answers_missing_are_refused_with_a_finish_command_naming_both_flags` | herding/run.rs:12771 |
| worker-full-suite-guard (6e253441) | `worker_full_suite_refuses_the_declared_suite_bee_test_and_env_prefixed_suite` | hooks/worker_guard.rs:761 |
| worker-full-suite-guard | `worker_full_suite_allows_a_filtered_test_and_a_cell_with_a_full_suite_reason` | hooks/worker_guard.rs:780 |
| worker-full-suite-guard | `worker_full_suite_does_not_refuse_a_job_with_no_cell_id` | hooks/worker_guard.rs:794 |
| worker-full-suite-guard | `worker_full_suite_exits_two_through_the_hook_only_for_a_herded_job` | hooks/worker_guard.rs:803 |

What each shows:
- Cap refusal: a cell in the principles arm (dispatch.jsonl) that touches owned code without its skill, with an incomplete `--mistake`, refuses once with `needs 3 answer(s) before it caps:` naming the mistake fix, `--principle-used`/`--no-principle-used` and `SYNC_DOOR`/`--sync-ack`; the cell file is byte-identical after. With `--no-mistakes`, `--no-principle-used` and `--sync-ack` it caps. A NEEDS_REVISION cell with `--override-judge` whose sync door refuses writes no decision row; once the sync ack is given the row and `trace.judge_overrides` appear.
- Auto-settle: a done result carrying `principles_used` and `sync_ack` is capped by `try_cap_finished_cell` in the principles arm on a cell the sync door refuses. Without them it returns `Refused` naming `--principle-used` and `--sync-ack`, and `settle_command` builds a `bee cells finish --id .. --from-job ..` line carrying both flags (and neither when the refusal names none).
- Worker guard: with a declared suite `cargo test --release`, `cargo test --release`, `bee test`, `.bee/bin/bee test`, `RUST_LOG=debug cargo test --release` and `cd packages && cargo test --release` are refused with the cell verify named; `cargo test --release wf_filter` is allowed; a cell with `full_suite_reason` is allowed the suite; a job with no cell id is not refused; through `run()` the herded env gives exit 2 for the suite, not for a filtered test, and not outside a herd.

## Red proofs (production file at the code cell's parent, then restored)

lct-1 — `git show 59dd3851e^:.../verbs/cells/handlers_close.rs`, verify filter `cap_one_refusal`: both tests FAILED.
```
cap_one_refusal_names_the_mistakes_principles_and_sync_fixes_together_and_writes_nothing ... FAILED
  capCell: cell "co-1" refused — a recorded mistake is incomplete: a reflection has three required parts, ...
cap_one_refusal_does_not_log_the_judge_override_when_a_later_check_refuses ... FAILED
  refused cap must not log the override: {"id":"fcd70ef2-...","type":"decide",... "judge override by unknown — judge is wrong here" ...}
test result: FAILED. 0 passed; 2 failed
```

lct-2 — `git show 8377921d6^:.../herding/mailbox.rs`, filter `auto_settle_answers`: compile error.
```
error[E0560]: struct `herding::mailbox::MailboxResult` has no field named `sync_ack`
    --> crates/bee/src/herding/run.rs:3496:9
```
Run.rs holds the tests, so it cannot be put at its parent without dropping them; the parent behaviour was also reproduced by setting the auto-settle cap flag back to `sync_ack: None` (the parent value):
```
auto_settle_answers_cap_a_principles_arm_cell_the_sync_door_would_refuse ... FAILED
  left: Refused("capCell: cell \"cell-ans-1\" needs 1 answer(s) before it caps:\n- capCell: SYNC_DOOR — ...")
 right: Capped
test result: FAILED. 1 passed; 1 failed
```
The second test (`..._missing_are_refused_with_a_finish_command...`) needs `PostflightCapResult::Refused` and `settle_command`, both added by lct-2, so at the parent it is a compile error too.

lct-3 — `git show dbf053c50^:.../verbs/cells/validate.rs`, filter `worker_full_suite`: compile error.
```
error[E0425]: cannot find function `full_suite_hit` in module `crate::verbs::cells`
   --> crates/bee/src/hooks/worker_guard.rs:283:38
error: could not compile `bee` (bin "bee" test) due to 1 previous error
```
worker_guard.rs also holds its tests, so the guard gate was neutralised instead (`let herded_job: Option<String> = None;`, the parent behaviour):
```
worker_full_suite_exits_two_through_the_hook_only_for_a_herded_job ... FAILED
  left: "ExitCode(unix_exit_status(0))"
 right: "ExitCode(unix_exit_status(2))"
test result: FAILED. 3 passed; 1 failed
```
The three direct `evaluate_full_suite` tests call the helper that lct-3 added, so they have no parent to run against. Every edit was restored; `git diff --stat` after shows only added lines in the three test files.

## lct-4 parity (commit a8c192825)

| must_have | Shown at | Verdict |
|---|---|---|
| cells-completion-judge-and-archive.md states the one combined refusal and the judge override write order, citing D1 | docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md:876-882 (section "One combined refusal and the judge override write order (leader-call-thrift D1)"), frontmatter :11 | matches handlers_close.rs:903 (refuse) and :926 (log_decision after) |
| the-run-verb-and-worker-outcomes.md states auto-settle passes principles_used and sync_ack and prints the full finish command, citing D2 | docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md:876, frontmatter :11 | matches herding/run.rs:6144 and :6216 |
| hook-runtime-the-worker-outward-guard.md states the full-suite refusal for herded cell jobs and the full_suite_reason escape, citing D3 | docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md:90-96 (section "Full-suite refusal for herded cell jobs"), Pointers :131, frontmatter :11 | matches worker_guard.rs:79 and validate.rs:189 |

No gap found. The cell's verify `./.bee/bin/bee knowledge check` was not run here (a worker runs no bee command).

## Judge verdicts

```json lct-1
{
  "schema": "judge-verdict/1",
  "verdict": "PASS",
  "checks": [
    {"id": "one-refusal-lists-every-problem", "status": "PASS", "evidence": "handlers_close.rs:251-270 collect_refusal and combined_cap_refusal; :903 refuses once with 'needs N answer(s)'; test cap_one_refusal_names_the_mistakes_principles_and_sync_fixes_together_and_writes_nothing shows all three fixes in one text"},
    {"id": "state-refusals-keep-order-and-refuse-alone", "status": "PASS", "evidence": "handlers_close.rs:509 NEEDS_REVISION refusal still returns early before :903; the diff touches only the answer checks (mistakes :406, principles, trailer :450, floor, arm :482-487, sync :883)"},
    {"id": "judge-override-logged-after-every-check", "status": "PASS", "evidence": "handlers_close.rs:926 log_decision now sits after the :903 problems gate; test cap_one_refusal_does_not_log_the_judge_override_when_a_later_check_refuses failed at the parent (decision row written) and passes at HEAD"},
    {"id": "existing-single-refusal-tests-updated", "status": "PASS", "evidence": "verbs/cells/tests.rs one-commit-per-cell assertion now expects 'needs ' and '\\n- one commit per cell'; the cell verify passed in lct-1"}
  ],
  "fixability": "automatic",
  "confidence": "high"
}
```

```json lct-2
{
  "schema": "judge-verdict/1",
  "verdict": "PASS",
  "checks": [
    {"id": "sync-ack-and-principles-reach-the-cap", "status": "PASS", "evidence": "herding/run.rs:6216 passes r.sync_ack; from_job carries the result's principles_used; test auto_settle_answers_cap_a_principles_arm_cell_the_sync_door_would_refuse caps with both and fails with sync_ack: None"},
    {"id": "refusal-printed-with-full-finish-command", "status": "PASS", "evidence": "herding/run.rs:6141 Refused variant and :6144 settle_command add --principle-used for a refusal naming it and --sync-ack for one naming it; test auto_settle_answers_missing_are_refused_with_a_finish_command_naming_both_flags asserts both flags and none on a plain refusal. settle_command emits --principle-used only, never --no-principle-used, which matches the 'or' in the truth"},
    {"id": "brief-and-parser-name-the-two-fields", "status": "PASS", "evidence": "herding/mailbox.rs:476-478 'Cap answers' lines; MailboxResult.sync_ack at :563 parsed at :865-884 with blank filtered"},
    {"id": "existing-tests-updated", "status": "PASS", "evidence": "run.rs existing MailboxResult literals gained sync_ack: None; the cell verify passed in lct-2"}
  ],
  "fixability": "automatic",
  "confidence": "high"
}
```

```json lct-3
{
  "schema": "judge-verdict/1",
  "verdict": "PASS",
  "checks": [
    {"id": "guard-refuses-full-suite-for-herded-cell-job", "status": "PASS", "evidence": "worker_guard.rs:269-290 herded_job and evaluate_full_suite, wired at :79 with exit code 2; test worker_full_suite_refuses_the_declared_suite_bee_test_and_env_prefixed_suite covers the declared suite, bee test, .bee/bin/bee test, an env prefix and a cd-prefixed segment"},
    {"id": "no-refusal-for-filter-reason-or-no-cell-id", "status": "PASS", "evidence": "validate.rs:189-195 returns None for a non-blank full_suite_reason and when the command is not a full-suite segment; worker_guard.rs:278 returns None without a cell id; tests worker_full_suite_allows_a_filtered_test_and_a_cell_with_a_full_suite_reason and worker_full_suite_does_not_refuse_a_job_with_no_cell_id"},
    {"id": "refusal-text-names-suite-once-per-feature-and-verify", "status": "PASS", "evidence": "worker_guard.rs:285-288 text: starts the full suite, runs once per feature through bee test, FIX names the cell verify; asserted in the refusal test"},
    {"id": "one-shared-segment-test", "status": "PASS", "evidence": "validate.rs:183 command_runs_full_suite and :189 full_suite_hit; full_suite_verify_problem at :205 and the guard at worker_guard.rs:283 both call full_suite_hit"}
  ],
  "fixability": "automatic",
  "confidence": "high"
}
```

## Notes

- The brief's "mistakes answer" check: the cap never refuses a silent mistakes answer, only an incomplete one (`read_mistakes_answer`, handlers_close.rs:1169), so the test uses an incomplete `--mistake` for that slot.
- The brief's Proof command lists a full `cargo test`; the cell action says never run the full suite, so only the cell verify and clippy were run.
- principles_used: bee-principle-red-before-green — ran each contract test against the parent before trusting it; bee-principle-test-behavior-not-structure — asserted refusal text, cell file bytes and exit codes, not the internal collector.
