# ppb-4t — the situation fence goes red for each break

Cell ppb-4t checks the fence ppb-4 added (`2bddce22e`) against the real
tree, not a sandbox. The fence is the test
`every_situation_file_matches_its_index` in
`packages/bee-rs/crates/bee/tests/class_playbook_parity.rs`. It covers
decisions `c64c5827` and `913115b3`.

## Method

Each break was a temporary edit to `skills/bee-planning/playbooks/` on the
worktree at `2bddce22e`. After each one I ran the fence alone:

    cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee \
      --test class_playbook_parity -- every_situation_file_matches_its_index

Then I reverted the edit before making the next one. After the third
revert, `git diff --quiet HEAD -- skills` passed, so the tree is
byte-identical. Then the whole test file ran green.

The baseline before any break was 9 passed, 0 failed.

## Verdict

All three breaks go red, each with one finding that names the break. The
clean tree is green again. Absolute paths are shortened to `<wt>`.

| Break | Temporary edit | Red | Finding |
|---|---|---|---|
| 1. unlisted situation file | add `bugfix/stray-situation.md` | yes | `bugfix: <wt>/skills/bee-planning/playbooks/bugfix/stray-situation.md is not listed in <wt>/skills/bee-planning/playbooks/bugfix.md, so no router can reach it` |
| 2. listed name, file missing | delete `bugfix/bug-fix.md` (restored from a copy) | yes | `bugfix: <wt>/skills/bee-planning/playbooks/bugfix.md lists `bug-fix` but <wt>/skills/bee-planning/playbooks/bugfix/bug-fix.md does not exist` |
| 3. stray subdirectory | `mkdir stray-dir` | yes | `` stray-dir/: <wt>/skills/bee-planning/playbooks/stray-dir is neither a route class in `ROUTE_CLASS_VALUES` (packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs) nor `session` `` |

## Break 1 — full fence output

```text
running 1 test
test every_situation_file_matches_its_index ... FAILED

failures:

---- every_situation_file_matches_its_index stdout ----

thread 'every_situation_file_matches_its_index' (<tid>) panicked at crates/bee/tests/class_playbook_parity.rs:347:5:
the situation playbooks disagree with their indexes:
  bugfix: <wt>/skills/bee-planning/playbooks/bugfix/stray-situation.md is not listed in <wt>/skills/bee-planning/playbooks/bugfix.md, so no router can reach it

A situation file no index lists is never routed to, and a listed name with no file sends the router to nothing. FIX: add the line - `<name>` — <triggers, English and Vietnamese> — delivers: <deliverable> under `## Situations` in the class index (or session/index.md), write the missing file, or delete the stray one.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    every_situation_file_matches_its_index

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p bee --test class_playbook_parity`
```

## Break 2 — full fence output

```text
running 1 test
test every_situation_file_matches_its_index ... FAILED

failures:

---- every_situation_file_matches_its_index stdout ----

thread 'every_situation_file_matches_its_index' (<tid>) panicked at crates/bee/tests/class_playbook_parity.rs:347:5:
the situation playbooks disagree with their indexes:
  bugfix: <wt>/skills/bee-planning/playbooks/bugfix.md lists `bug-fix` but <wt>/skills/bee-planning/playbooks/bugfix/bug-fix.md does not exist

A situation file no index lists is never routed to, and a listed name with no file sends the router to nothing. FIX: add the line - `<name>` — <triggers, English and Vietnamese> — delivers: <deliverable> under `## Situations` in the class index (or session/index.md), write the missing file, or delete the stray one.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    every_situation_file_matches_its_index

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p bee --test class_playbook_parity`
```

## Break 3 — full fence output

```text
running 1 test
test every_situation_file_matches_its_index ... FAILED

failures:

---- every_situation_file_matches_its_index stdout ----

thread 'every_situation_file_matches_its_index' (<tid>) panicked at crates/bee/tests/class_playbook_parity.rs:347:5:
the situation playbooks disagree with their indexes:
  stray-dir/: <wt>/skills/bee-planning/playbooks/stray-dir is neither a route class in `ROUTE_CLASS_VALUES` (packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs) nor `session`

A situation file no index lists is never routed to, and a listed name with no file sends the router to nothing. FIX: add the line - `<name>` — <triggers, English and Vietnamese> — delivers: <deliverable> under `## Situations` in the class index (or session/index.md), write the missing file, or delete the stray one.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    every_situation_file_matches_its_index

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p bee --test class_playbook_parity`
```

## Clean tree after the reverts

```text
running 9 tests
test every_playbook_names_a_real_route_class ... ok
test every_route_class_has_exactly_one_playbook ... ok
test every_situation_file_matches_its_index ... ok
test a_listed_name_with_no_file_fails_the_fence ... ok
test a_clean_sandbox_without_session_passes ... ok
test a_subdirectory_that_is_neither_a_class_nor_session_fails_the_fence ... ok
test a_malformed_situation_line_fails_the_fence ... ok
test an_unlisted_situation_file_fails_the_fence ... ok
test session_files_and_session_index_agree_both_ways ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

## Notes

- Break 3 used an empty directory. The fence still saw it, because it
  lists subdirectories and does not need a `.md` file inside them.
- The fence only reports. Each break produced exactly one finding, so no
  break set off a second, unrelated finding.
- `.bee/onboarding.json` and `packages/bee-rs/crates/bee/.bee/logs/timings.jsonl`
  were already modified before this cell started. I did not touch them and
  they are not part of this commit.
