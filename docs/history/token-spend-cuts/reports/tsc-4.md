# tsc-4 — the three contracts red then green

Cell tsc-4 tests contracts C1 handoff-mark-directive (56a0eec4), C2
knowledge-pack-budget (ce42c69f) and C3 judge-small-diff (3e94e127)
against the commits for tsc-1 (5a9dae308), tsc-2 (c4e633e93) and tsc-3
(e45b6f5cc). It edits no production code.

Verify command, used for every run below:

```
PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- handoff_mark knowledge_pack judge_small_diff
```

## Tests added

| Contract | Test | Module |
|---|---|---|
| C1 | `handoff_mark_absent_key_at_exactly_300000_tokens_adds_the_directive_line` | `hooks/prompt_context.rs` tests |
| C1 | `handoff_mark_absent_key_at_299999_tokens_adds_no_line` | `hooks/prompt_context.rs` tests |
| C1 | `handoff_mark_key_zero_adds_no_line_at_any_size` (0, 299999, 300000, 1000000 tokens) | `hooks/prompt_context.rs` tests |
| C1 | `handoff_mark_a_green_merge_next_action_names_the_fresh_leader_session` | `verbs/worktree/tests.rs` |
| C2 | `knowledge_pack_lane_budget_holds_the_five_d4_values` | `verbs/drivers/tests.rs` |
| C2 | `knowledge_pack_empty_manifest_with_an_index_on_disk_yields_no_learned_context_line` | `verbs/drivers/tests.rs` |
| C3 | `judge_small_diff_standard_lane_skips_a_cell_with_29_changed_lines` | `verbs/cells/tests.rs` |
| C3 | `judge_small_diff_standard_lane_counts_a_cell_with_30_changed_lines` | `verbs/cells/tests.rs` |
| C3 | `judge_small_diff_standard_lane_counts_a_cell_with_no_changed_lines` | `verbs/cells/tests.rs` |
| C3 | `judge_small_diff_high_risk_counts_a_cell_with_5_changed_lines` | `verbs/cells/tests.rs` |
| C3 | `judge_small_diff_a_real_cap_over_a_git_commit_stores_changed_lines` (real git repo, second commit turns `x` into three lines, numstat 3+1, so `trace.changed_lines` is 4) | `verbs/cells/tests.rs` |

Transcript token counts are input 10000 + cache_read + cache_creation
50000, so cache_read 240000 gives exactly 300000 and 239999 gives 299999.
The C3 tests reuse `capped_behavior_change_cell`, `write_lane_record_routed`
and `write_cell_fixture`. The cap test reuses `commit_history_repo`,
`commit_with_message` and `cap_flags_frd`.

The filter also matches three tests that already existed:
`transcript_past_handoff_mark_carries_warning_line`,
`transcript_below_handoff_mark_adds_no_line` and
`feature_outcomes_pull_rate_counts_a_knowledge_pack_call_as_a_pull`.
That is why the runs below show 14 tests.

## Red runs

For each run I set the contract's production files to their parent-commit
versions, ran the verify, then restored them from HEAD. After each restore,
`git diff --stat -- packages/bee-rs` showed only the four test files
(144 insertions).

### C1 at 5a9dae308^ (`phases.rs` and the production half of `prompt_context.rs`)

The guard refuses `git checkout` while two workers share the checkout, so
I restored files with `git show HEAD:<path> > <path>`. `prompt_context.rs`
holds its own tests module, and a whole-file `git show 5a9dae308^:` would
have removed the new tests too. So the red state reverts only the code
above `mod tests {`: the two `300_000` defaults go back to `150_000`, and
the directive text goes back to the old text. The bytes match the parent
exactly.

```
test hooks::prompt_context::tests::handoff_mark_absent_key_at_exactly_300000_tokens_adds_the_directive_line ... FAILED
test hooks::prompt_context::tests::handoff_mark_absent_key_at_299999_tokens_adds_no_line ... FAILED
test hooks::prompt_context::tests::transcript_past_handoff_mark_carries_warning_line ... FAILED
test verbs::worktree::tests::handoff_mark_a_green_merge_next_action_names_the_fresh_leader_session ... FAILED
---- ...handoff_mark_absent_key_at_exactly_300000_tokens_adds_the_directive_line stdout ----
  left: Some("context 300k tokens (mark 150k): keep working; hand off only on the user's word or at feature close")
 right: Some("context 300k tokens (mark 300k): hand off at the next clean stop (a capped cell or a merged worktree), then continue in a fresh leader session")
---- ...handoff_mark_absent_key_at_299999_tokens_adds_no_line stdout ----
  left: Some("context 299k tokens (mark 150k): keep working; hand off only on the user's word or at feature close")
 right: None
---- ...handoff_mark_a_green_merge_next_action_names_the_fresh_leader_session stdout ----
panicked at crates/bee/src/verbs/worktree/tests.rs:5649:9:
Merged into main at 7090edc; capture what settled, then bee close --feature demo.
test result: FAILED. 10 passed; 4 failed; 0 ignored; 0 measured; 5560 filtered out; finished in 0.05s
```

`handoff_mark_key_zero_adds_no_line_at_any_size` stays green at the parent,
as it should. D1 keeps the rule that 0 turns the mark off.

### C2 at c4e633e93^ (`verbs/drivers/prepare.rs`)

```
test verbs::drivers::tests::knowledge_pack_lane_budget_holds_the_five_d4_values ... FAILED
test verbs::drivers::tests::knowledge_pack_empty_manifest_with_an_index_on_disk_yields_no_learned_context_line ... FAILED
---- ...knowledge_pack_lane_budget_holds_the_five_d4_values stdout ----
  left: [8000.0, 12000.0, 20000.0, 30000.0, 20000.0]
 right: [4000.0, 6000.0, 10000.0, 15000.0, 10000.0]
---- ...knowledge_pack_empty_manifest_with_an_index_on_disk_yields_no_learned_context_line stdout ----
["- docs/knowledge/index.md — Knowledge bundle index (see \"Critical patterns\")"]
test result: FAILED. 12 passed; 2 failed; 0 ignored; 0 measured; 5560 filtered out; finished in 0.05s
```

### C3 at e45b6f5cc^ (`verbs/cells/handlers_close.rs`, `verbs/drivers/close.rs`)

The parent's `judge_debt` takes two arguments, so the run fails to compile:

```
error[E0061]: this function takes 2 arguments but 3 arguments were supplied
    --> crates/bee/src/verbs/cells/tests.rs:3092:9
3092 |         crate::verbs::drivers::judge_debt(root, "demo", lane).unwrap().count
     |                                                         ---- unexpected argument #3 of type `&str`
 655 | pub(crate) fn judge_debt(root: &Path, feature: &str) -> D<DebtSummary> {
error: could not compile `bee` (bin "bee" test) due to 4 previous errors; 7 warnings emitted
```

## Green run (HEAD production code, after every restore)

```
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 5560 filtered out; finished in 0.05s
```

`cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings`:
`Finished dev profile`, no warnings.

## Judge verdicts

Each verdict below comes from reading the cell's commit diff against its
must_haves.

tsc-1 (5a9dae308): both arms of the default read 300000, and 0 still
returns early. The line text matches exactly. The green-merge next action
ends with the required sentence, and the uat variant is untouched. The
tests that pinned the old text and the old default were updated.

```json tsc-1
{"schema": "judge-verdict/1", "verdict": "PASS", "checks": [
  {"id": "default-300000-absent-or-non-number-and-0-off", "result": "PASS"},
  {"id": "directive-line-text", "result": "PASS"},
  {"id": "green-merge-next-action-ends-with-fresh-leader-session", "result": "PASS"},
  {"id": "old-pins-updated-and-passing", "result": "PASS"}
]}
```

tsc-2 (c4e633e93): `lane_budget` returns 4000, 6000, 10000 and 15000,
with 10000 in both default arms. The `index.md` fallback block is gone.
`lane_budget_scales_and_defaults` and the old fallback test were updated.

```json tsc-2
{"schema": "judge-verdict/1", "verdict": "PASS", "checks": [
  {"id": "lane-budget-five-values", "result": "PASS"},
  {"id": "no-index-fallback-on-empty-manifest", "result": "PASS"},
  {"id": "old-pins-updated-and-passing", "result": "PASS"}
]}
```

tsc-3 (e45b6f5cc): `commit_changed_lines` sums numstat added plus removed
for `<commit>^..<commit>`. Binary rows (`-`) count 0. It is called even
when the commit is empty or `none`, but git then exits 128 (`bad
revision '^'`, `ambiguous argument 'none^'`, probed by hand), so nothing
is stored. A root commit also exits 128 and stores nothing. `judge_debt`
takes the lane and skips only when the lane is `standard` and
`changed_lines` is below 30. Both call sites (`close.rs:1667`,
`close.rs:2865`) pass the route lane, and no other caller exists.

```json tsc-3
{"schema": "judge-verdict/1", "verdict": "PASS", "checks": [
  {"id": "cap-stores-changed-lines-from-first-parent-numstat", "result": "PASS"},
  {"id": "nothing-stored-for-none-absent-or-uncountable-commit", "result": "PASS"},
  {"id": "standard-skips-below-30-counts-30-and-missing", "result": "PASS"},
  {"id": "high-risk-counts-every-unjudged-cell", "result": "PASS"},
  {"id": "door-call-sites-pass-route-lane", "result": "PASS"}
]}
```

Notes that do not change a verdict: tsc-3 only works because git refuses
the `none` and empty commits. Skipping them explicitly, the way the
`patch_id` branch does, would make that clear. Its upstream summary also
lists commit c4e633e93, which is tsc-2's commit.
