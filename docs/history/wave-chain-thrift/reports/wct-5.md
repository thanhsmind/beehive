# wct-5 — Prove the three contracts and check the docs

Verify (green, fresh):

```
PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- wave_judge_record conflict_verdict_carry read_only_shell_rules
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 5582 filtered out
```

Only test code changed, inside the three named test modules. No production code was edited; after each red run the production files were restored and the working tree diff compared byte-identical to the saved copy.

## Tests added

| Contract | Test | Where |
|---|---|---|
| wave-judge-record | `wave_judge_record_records_one_verdict_per_dep_block_of_a_test_cell` | verbs/drivers/wave_run.rs:567 |
| wave-judge-record | `wave_judge_record_ignores_a_block_for_a_cell_that_is_not_a_dep` | verbs/drivers/wave_run.rs:583 |
| wave-judge-record | `wave_judge_record_records_nothing_for_an_invalid_block_and_shows_it_on_the_row` | verbs/drivers/wave_run.rs:598 |
| wave-judge-record | `wave_judge_record_never_reads_the_report_of_a_code_cell` | verbs/drivers/wave_run.rs:613 |
| conflict-verdict-carry | `conflict_verdict_carry_keeps_verdicts_the_new_cell_does_not_touch_and_opens_its_own` | verbs/state_group/tests.rs:3189 |
| conflict-verdict-carry | `conflict_verdict_carry_reopens_the_candidates_of_a_cell_whose_packet_changed` | verbs/state_group/tests.rs:3214 |
| read-only-shell-rules | `read_only_shell_rules_list_every_allowed_command_and_the_refused_forms` | herding/mailbox.rs:1661 |
| read-only-shell-rules | `read_only_shell_rules_are_absent_from_a_brief_that_is_not_read_only` | herding/mailbox.rs:1690 |
| read-only-shell-rules | `read_only_shell_rules_ack_step_needs_no_command_substitution` | herding/mailbox.rs:1698 |

The conflict tests drive the real `state plan-conflicts derive` and `verdict` verbs through `try_native` with the shared cwd lock, and read the stored `conflict_review`.

## Red proof at each code cell's parent

Method: reverse-apply the production hunks of the code cell commit onto the working tree (equivalent to the parent's production file with the new tests kept), run the filter, restore.

### wave-judge-record — wave_run.rs without commit 9a7bdddd3 (wct-1)

All four tests fail:

```
test ...wave_judge_record_records_nothing_for_an_invalid_block_and_shows_it_on_the_row ... FAILED
test ...wave_judge_record_never_reads_the_report_of_a_code_cell ... FAILED
test ...wave_judge_record_ignores_a_block_for_a_cell_that_is_not_a_dep ... FAILED
test ...wave_judge_record_records_one_verdict_per_dep_block_of_a_test_cell ... FAILED
wave_run.rs:507  left: Null  right: Array [String("a-1"), String("a-2")]
wave_run.rs:523  left: Null  right: Array [String("a-1")]
wave_run.rs:536  left: Null  right: Array []
wave_run.rs:551  left: Null  right: Array []
test result: FAILED. 0 passed; 4 failed
```

Reason: the row has no `judged` field before wct-1.

### conflict-verdict-carry — plan_conflicts.rs without commit b575d3a8d (wct-2)

```
test ...conflict_verdict_carry_keeps_verdicts_the_new_cell_does_not_touch_and_opens_its_own ... FAILED
tests.rs:3204  left: Null  right: String("compatible")
test ...conflict_verdict_carry_reopens_the_candidates_of_a_cell_whose_packet_changed ... ok
test result: FAILED. 1 passed; 1 failed
```

The keep-verdicts test is the red one. The reopen test passes at the parent too: the old derive reopened every candidate, so it is a regression guard for the new carry, not a red proof of its own.

### read-only-shell-rules — mailbox.rs, run.rs and worker_guard.rs without commit 1ce1a1806 (wct-3)

Compile error, the tests cannot build against the parent:

```
error[E0603]: constant `READ_ONLY_ALLOWED` is private
    --> crates/bee/src/herding/mailbox.rs:1636:52
error[E0609]: no field `read_only` on type `herding::mailbox::BriefSpec<'_>`
    --> crates/bee/src/herding/mailbox.rs:1626:14
error: could not compile `bee` (bin "bee" test) due to 2 previous errors
```

## wct-4 parity (commit 8e8f5b440)

| must_have | Shown at | Verdict |
|---|---|---|
| bee-swarming step 2 names `bee dispatch wave --rolling --run` the default loop; no leader call between code cells, testers, judge record, caps; leader reviews diffs and runs `leader-check` once per wave | skills/bee-swarming/SKILL.md:62 (sentence runs on to the leader-check line) | met |
| waves-and-occupancy states judge recording and the `capped` and `judged` row fields, citing D1 | docs/knowledge/areas/bee-herding/waves-and-occupancy.md:303 (heading names D1), :309 (`capped`, `judged`); frontmatter decisions end with `wave-chain-thrift D1` | met |
| gates.md states the verdict carry, `cell_digests`, `carried_from`, citing D2 | docs/knowledge/areas/workflow-state/gates.md:445; frontmatter decisions end with `wave-chain-thrift D2` | met |
| worker-outward-guard states the read-only shell-rules block and its one source, citing D3 | docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md:105 (heading names D3), :109 (block and `read_only_commands` source), :140 (pointer to `shell_rules_block`) | met |
| rendered skill trees regenerated and committed | `default loop is` sentence present in .agents/skills/bee-swarming/SKILL.md:60, .claude-plugin/skills/bee-swarming/SKILL.md:58, .claude/skills/bee-swarming/SKILL.md:58, .codex-plugin/skills/bee-swarming/SKILL.md:58, .opencode/skills/bee-swarming/SKILL.md:58; each tree's `.bee-render.json` and docs/history/codex-harness-hardening/release-manifest.json changed in the same commit | met |

No gap found. I did not run `bee dev release-manifest --check` or `bee knowledge check`; the cell verify excludes them and the brief limits me to the cell verify.

## Judge verdicts (from the diffs)

```json wct-1
{
  "schema": "judge-verdict/1",
  "verdict": "PASS",
  "checks": [
    {"id": "reads-only-test-cells", "status": "PASS", "evidence": "wave_run.rs:96 returns early unless role is test; wave_judge_record_never_reads_the_report_of_a_code_cell passes and fails nothing at HEAD"},
    {"id": "dep-filter", "status": "PASS", "evidence": "wave_run.rs:120 skips a block whose id is not in the test cell's deps; wave_judge_record_ignores_a_block_for_a_cell_that_is_not_a_dep shows x-9 gets no entry"},
    {"id": "invalid-block-on-row", "status": "PASS", "evidence": "wave_run.rs:128-143 pushes the error text to judge_errors and wave_run.rs:84 puts it on the row; wave_judge_record_records_nothing_for_an_invalid_block_and_shows_it_on_the_row passes"},
    {"id": "row-fields", "status": "PASS", "evidence": "wave_run.rs:81-82 add capped and judged to every row; red run at the parent shows judged Null"}
  ],
  "fixability": "automatic",
  "confidence": "high"
}
```

```json wct-2
{
  "schema": "judge-verdict/1",
  "verdict": "PASS",
  "checks": [
    {"id": "digest-stored", "status": "PASS", "evidence": "plan_conflicts.rs:433-449 hash title, files, affects_skills, affects_specs per cell; plan_conflicts.rs:632-635 store them under cell_digests"},
    {"id": "carry-skips-touched", "status": "PASS", "evidence": "plan_conflicts.rs:644-645 derives from changed cells alone and carry_verdicts skips those ids at :466; conflict_verdict_carry_keeps_verdicts_the_new_cell_does_not_touch_and_opens_its_own passes and fails at the parent"},
    {"id": "carried-from-marker", "status": "PASS", "evidence": "plan_conflicts.rs:478 sets carried_from to the previous plan_rev; the test asserts carried_from == 3 on both carried rows and absent on the new candidate"},
    {"id": "changed-packet-reopens", "status": "PASS", "evidence": "conflict_verdict_carry_reopens_the_candidates_of_a_cell_whose_packet_changed passes after koh-demo files change"},
    {"id": "first-derive-carries-nothing", "status": "PASS", "evidence": "plan_conflicts.rs:632 filters previous to a review that holds cell_digests, so the first derive takes the None arm and carries 0"}
  ],
  "fixability": "automatic",
  "confidence": "medium"
}
```

```json wct-3
{
  "schema": "judge-verdict/1",
  "verdict": "PASS",
  "checks": [
    {"id": "single-source-list", "status": "PASS", "evidence": "worker_guard.rs:12-16 builds read_only_commands from READ_ONLY_ALLOWED and mailbox.rs:378 joins it; read_only_shell_rules_list_every_allowed_command_and_the_refused_forms checks all 23 commands"},
    {"id": "refused-forms-named", "status": "PASS", "evidence": "mailbox.rs:377-393 lists substitution, redirects, sh -c and eval, sort, fd --exec, pipes; the same test asserts each line"},
    {"id": "ack-needs-no-substitution", "status": "PASS", "evidence": "mailbox.rs:369-373 tells the worker to run date alone and paste it; read_only_shell_rules_ack_step_needs_no_command_substitution asserts no $( before the Shell rules block"},
    {"id": "non-read-only-unchanged", "status": "PASS", "evidence": "mailbox.rs:369-376 returns the original clause and an empty block when read_only is false; read_only_shell_rules_are_absent_from_a_brief_that_is_not_read_only passes and run.rs passes opts.read_only at all six render sites"}
  ],
  "fixability": "automatic",
  "confidence": "high"
}
```

wct-2 confidence is medium because the carry decision rests on a derive over the changed cells alone: a candidate that clears the score threshold only with the whole plan's terms is not seen as touched, and the 50-candidate cap applies to each derive separately. Neither case is covered by a test.
