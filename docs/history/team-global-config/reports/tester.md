# team-global-config — independent tester pass (tgc-9)

Tester: tgc-9, sharing no context with the implementers of tgc-1, tgc-2,
tgc-3, tgc-4, tgc-7 and tgc-8. Each contract point below was checked
against CONTEXT.md D1-D14 and C1-C3, not against the implementer's own
claims. "Red by" names a temporary break in production code, applied,
run, and reverted with `git checkout` before the commit. The break left
at least the named test red.

## Per-cell verdicts

| Cell | Contract point | Proving test | Red by | Gap and what was added |
|------|----------------|--------------|--------|------------------------|
| tgc-1 | C1 precedence global < tracked | `config_layers_tracked_beats_global`, `config_global_explain_a_key_in_two_layers_names_the_higher_one` | — (read) | none |
| tgc-1 | C1 precedence tracked < team | `config_layers_team_beats_tracked`, `config_tester_both_panes_allow_the_equal_view_move_and_refuse_a_team_json_slot_change` | B6: `Layer::ORDER` with Tracked and Team swapped | none |
| tgc-1 | C1 precedence team < local | `config_layers_local_beats_team` | — (read) | none |
| tgc-1 | C1 / D4 forbidden global keys dropped | `config_layers_global_forbidden_keys_are_dropped`, `config_layers_global_hooks_false_keeps_write_guard_enabled`, `config_global_forbidden_keys_never_reach_the_merged_view`, `config_tester_hooks_off_in_team_json_or_global_never_disables_the_write_guard` | B3: Global `restrict` made a no-op | The unit tests check the merged map. Added an integration test: a global `hooks.write-guard: false` must not switch off the write guard of the built binary, in a hooked pane or a herded one. |
| tgc-1 | C1 / D1 team.json foreign keys ignored | `config_layers_team_file_foreign_keys_are_ignored`, `config_tester_hooks_off_in_team_json_or_global_never_disables_the_write_guard` | B2: Team `restrict` made a no-op | Added the binary-level proof that a `hooks` key in team.json does not reach `hook_enabled`, and that a foreign key there reads as unset through `bee config get`. |
| tgc-1 | C1 / D5 corrupt layer read as empty | `config_layers_corrupt_layer_reads_as_empty_for_both_policies`, `config_global_corrupt_file_is_named_and_read_as_empty` | B7: corrupt layer merged as a marker object | none |
| tgc-1 | C1 `models` and `team` fold once after the merge | `config_layers_models_and_team_across_layers_fold_once_after_merge` | — (read) | none |
| tgc-1 | D3 global dir resolution | `config_tester_global_dir_falls_back_to_xdg_then_home` | B1: XDG branch of `global_config_dir` reads a different variable | Gap. Every existing test sets `BEEHIVE_CONFIG_DIR` or uses the test-only thread-local, so the `XDG_CONFIG_HOME` and `~/.config` fallbacks and the "empty counts as unset" rule had no test. Added one. |
| tgc-2 | herding readers use the layered view; bootstrap copies team.json | `config_layers_reader_team_json_matches_config_json_for_herding`, `config_layers_reader_global_only_herding_value_reaches_the_readers`, `config_layers_reader_corrupt_config_reads_as_defaults`, `config_layers_reader_worktree_bootstrap_copies_team_json`, `split_team_and_drop_global_keep_team_show_and_the_agent_command_identical` | B2 / B6 (shared reader) | none. Remaining direct `config.json` reads in production were checked one by one: `doctor.rs` `team_config_advisory`, `close.rs` `close_commit_bookkeeping_offending_file` and the `onboard/*` reads are FILE reads (a legacy-key check, the file that names a key, the onboard text merge). D5 allows those. |
| tgc-3 | C2 governed team.json change refused | `team_guard_refuses_edit_and_write_that_change_or_remove_a_team_slot`, `team_guard_refuses_a_bash_write_to_team_json`, `team_guard_refuses_apply_patch_on_team_json`, `team_guard_refuses_invalid_json_written_to_team_json`, `config_tester_both_panes_allow_the_equal_view_move_and_refuse_a_team_json_slot_change` | B4: guard merged view drops the Team layer | none (the binary-level test was added for D14 below) |
| tgc-3 | C2 equal-view move allowed | `team_guard_allows_moving_the_team_table_when_team_json_is_written_first`, `config_tester_both_panes_allow_the_equal_view_move_and_refuse_a_team_json_slot_change` | B4 | none |
| tgc-3 | C2 / D13 global layer in the comparison (no shadowing slot) | `team_guard_refuses_a_project_slot_that_shadows_a_global_slot` | B8: guard comparison built with no global dir | none |
| tgc-3 | C2 / D12 global-file write refused | `team_guard_refuses_every_agent_write_to_the_global_config_path`, `team_guard_refuses_bash_that_assigns_the_config_dir_variables`, `config_tester_both_panes_refuse_a_write_to_the_global_file_the_env_names` | B5 (herded side) | The unit tests resolve the global dir through the test-only thread-local. Added a binary-level test where `BEEHIVE_CONFIG_DIR` names the file. **Defect found, not fixed:** see below. |
| tgc-4 | C3 `explain` names each source layer | `config_global_explain_names_each_of_the_five_sources`, `config_global_explain_a_key_in_two_layers_names_the_higher_one`, `explain_names_the_layer_the_effective_value_comes_from` | B6 | none |
| tgc-4 | C3 `validate`: parse, team.json foreign key, duplicate, forbidden global key, global-supplied keys | `config_global_validate_reports_each_problem_kind_with_file_and_key`, `config_global_validate_lists_the_keys_the_global_layer_supplies`, `config_global_corrupt_file_is_named_and_read_as_empty` | B2, B3 | none |
| tgc-4 | C3 / D7 `validate` judges team problems on the merged view | `config_tester_validate_judges_team_problems_on_the_merged_view` | B9: `validate` judges the tracked layer alone | Gap. No test failed when `validate` stopped reading the global and team layers for team problems. Added one: a malformed slot supplied by the global file is a problem, and team.json completing that slot clears it. |
| tgc-4 | C3 `--global` reads only the global file; set/unset refuse it; `--global --local` refused | `config_global_get_reads_only_the_global_file`, `config_global_set_and_unset_refuse_the_global_flag`, `config_global_with_local_refuses`, `config_global_governed_keys_still_refuse_with_a_global_layer_present` | — (read) | none |
| tgc-4 | C3 / D8 / D11 split-team moves, is idempotent, refuses a conflict, `--drop-global` | `split_team_moves_the_team_table_and_keeps_the_merged_view`, `split_team_is_idempotent`, `split_team_refuses_a_conflicting_value_with_zero_change`, `split_team_finishes_from_the_in_between_state`, `split_team_drop_global_removes_equal_leaves_and_keeps_differing_ones`, `split_team_without_drop_global_never_compares_with_global`, `split_team_says_to_commit_both_files_together` | — (read) | none |
| tgc-8 | D14 herded pane refuses global-file writes | `herded_team_guard_refuses_a_write_to_the_global_config_file`, `herded_team_guard_refuses_bash_that_assigns_the_config_dir`, `config_tester_both_panes_refuse_a_write_to_the_global_file_the_env_names` | B5: `run_herded` drops `global_config_write_denial` | The unit tests call `emit_for` in-process. Added the binary-level proof with `BEE_HERDING_WORKER=1` through `bee hook write-guard`. |
| tgc-8 | D14 herded pane refuses a governed team.json change and allows the equal-view move | `herded_team_guard_refuses_an_edit_that_changes_a_team_slot`, `herded_team_guard_refuses_a_bash_write_to_team_json`, `config_tester_both_panes_allow_the_equal_view_move_and_refuse_a_team_json_slot_change` | B4 | Gap. No herded test proved the equal-view move is allowed, so a herded pane that refused every team.json write would have passed. Added one. |
| tgc-8 | D14 every other guard stays open | `herded_team_guard_passes_a_plain_source_write` | — (read) | none |
| tgc-7 | every path and D-number cited resolves | checked by script over `docs/config-reference.md`, `delivery.md` and the five concepts (`thin-peer-boot.md`, `governed-paths-and-the-intake-gate.md`, `write-guard-request-shapes.md`, `command-surface.md`, `host-project-artifacts.md`) | — | All paths that tgc-7 added resolve (`config_layers.rs`, `hooks/write_guard/worker_tree.rs`, `write_guard/store.rs` `read_config`, `catalog.rs`, the four `.bee/` layer files). Every team-global-config D-number cited (D1-D14) exists in CONTEXT.md. Each anchor in the five concepts exists. Three anchors are directories (`plans/`, `.pi/extensions/bee-guard/`, `scripts/`), against the files-only anchor rule; tgc-7 did not add them. |

## Defect found (reported, not fixed)

D12 says the guard refuses "a Bash command that names its path". The Bash
arm of `global_config_write_denial` (`hooks/write_guard/guards.rs`)
matches the command text against the literal absolute path only. A
command that names the global file through `~`, `$HOME` or
`$XDG_CONFIG_HOME` passes in a herded pane:

```
HOME=/tmp/tgc9probe/home, BEEHIVE_CONFIG_DIR and XDG_CONFIG_HOME unset, BEE_HERDING_WORKER=1
bee hook write-guard  <- Bash "echo {} > ~/.config/beehive/config.json"              exit=0
bee hook write-guard  <- Bash "echo {} > $HOME/.config/beehive/config.json"          exit=0
bee hook write-guard  <- Bash "echo {} > /tmp/tgc9probe/home/.config/beehive/config.json"  exit=2 (refused)
XDG_CONFIG_HOME=/tmp/tgc9probe/xdg, BEE_HERDING_WORKER=1
bee hook write-guard  <- Bash "echo {} > $XDG_CONFIG_HOME/beehive/config.json"       exit=0
```

In a hooked (non-herded) pane, the outside-worktree containment guard
refuses the `~` form instead, so the gap shows only where D14 leaves the
other guards open. Owner of the fix: a follow-up cell on
`global_config_write_denial`, for example by expanding `~`, `$HOME` and
`$XDG_CONFIG_HOME` before the match. No test for this was committed,
because a red test would fail the cell's verify.

## Added tests (tests/config_verbs.rs)

- `config_tester_global_dir_falls_back_to_xdg_then_home`
- `config_tester_both_panes_refuse_a_write_to_the_global_file_the_env_names`
- `config_tester_both_panes_allow_the_equal_view_move_and_refuse_a_team_json_slot_change`
- `config_tester_hooks_off_in_team_json_or_global_never_disables_the_write_guard`
- `config_tester_validate_judges_team_problems_on_the_merged_view`

Each one runs the built `bee` binary with `BEEHIVE_CONFIG_DIR`, or
`XDG_CONFIG_HOME` and `HOME`, set to a temp dir.

## Break log

| Break | Red tests |
|-------|-----------|
| B1 XDG branch reads another variable | `config_tester_global_dir_falls_back_to_xdg_then_home` |
| B2 Team restrict no-op | `config_global_validate_reports_each_problem_kind_with_file_and_key`, `config_tester_hooks_off_…`, `config_layers_team_file_foreign_keys_are_ignored`, `config_global_check_layers_names_file_and_key_for_each_problem_kind` |
| B3 Global restrict no-op | `config_global_forbidden_keys_never_reach_the_merged_view`, `config_tester_hooks_off_…`, `config_layers_global_hooks_false_keeps_write_guard_enabled`, `config_layers_global_forbidden_keys_are_dropped`, `config_global_check_layers_lists_only_keys_no_project_file_sets` |
| B4 guard merged view drops Team | `config_tester_both_panes_allow_…`, `config_tester_hooks_off_…`, `herded_team_guard_refuses_an_edit_that_changes_a_team_slot`, `team_guard_refuses_edit_and_write_that_change_or_remove_a_team_slot`, `team_guard_refuses_a_project_slot_that_shadows_a_global_slot`, `team_guard_allows_moving_the_team_table_when_team_json_is_written_first`, `team_guard_refuses_invalid_json_written_to_team_json` |
| B5 herded pane drops the global write refusal | `config_tester_both_panes_refuse_…`, `config_tester_hooks_off_…`, `herded_team_guard_refuses_a_write_to_the_global_config_file`, `herded_team_guard_refuses_bash_that_assigns_the_config_dir` |
| B6 Tracked and Team order swapped | `config_tester_both_panes_allow_…`, `config_layers_team_beats_tracked` |
| B7 corrupt layer merged as an object | `config_layers_corrupt_layer_reads_as_empty_for_both_policies` |
| B8 guard comparison without the global dir | `team_guard_refuses_a_project_slot_that_shadows_a_global_slot` |
| B9 validate judges team problems on tracked alone | `config_tester_validate_judges_team_problems_on_the_merged_view` |
