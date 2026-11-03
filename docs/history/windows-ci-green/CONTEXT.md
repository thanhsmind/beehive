# Windows CI green — Context

**Feature slug:** windows-ci-green
**Date:** 2026-09-19
**Shaping session:** complete
**Scope:** Quick
**Domain types:** RUN

## Feature Boundary

The Windows workflow on `main` is red on the same two tests, and has been since at
least Release 2.41.2. This makes it green. It ends at the two tests: no product
behavior changes, because the one cause already known is in test code, and the
other is not known yet and is found from evidence rather than guessed.

## Why now

`.github/workflows/windows.yml` states that *"win32 is bee's primary platform"*, and
the workflow runs on every push. A workflow that is always red trains people to
ignore it, which is exactly how a new failure slips past. It has already hidden one
question: whether a 74-commit push added failures. Answering that needed a manual
before/after comparison of failing-test lists.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `enter_and_merge_and_new_carry_session_runtime_and_instruction_with_injected_caller` builds its expected paths with the product's own `canonical_path_str`, at both sites (the enter path and the merge path). | The product canonicalizes with `dunce::canonicalize` (`verbs/worktree/handlers.rs:38-42`), which turns the Windows 8.3 short name `RUNNER~1` into the long name `runneradmin`. The test did not canonicalize its expectation. The same test file already uses `canonical_path_str(...)` for expected paths at `:5896`, `:5897` and `:5998` — this test is the one that missed the idiom. The product is correct; the fault is where the test builds the expected value, not where the assertion fails. |
| D2 | `test_release_dirt_script_filters_only_bee_paths` reports the exit code, stdout and stderr when the script fails, at both of its script runs. | Its failure message on Windows was literally `release-dirt.sh failed: ` with an empty stderr. It hides its own exit code and stdout, so the cause cannot be read from the log. A test that hides its own cause is a defect whatever that cause turns out to be. |
| D3 | The cause of the release-dirt failure is found from Windows CI evidence, not guessed. No change to `scripts/release-dirt.sh` lands until that evidence names it. | The script is used by the release path. Two hypotheses have already been refuted by checking: `bash` does resolve to a working shell on the runner (the test at `tests.rs:11219` and five in `statusline_contract.rs` call `Command::new("bash")` and pass), and `.gitattributes` pins `* text=auto eol=lf`, so CRLF is ruled out. Changing a release script on a third guess would be building on an untested assumption. |
| D4 | No change reaches `main` until the Windows workflow is green on this branch. | The workflow runs on every branch push, so the branch can be proven first. |

## Existing Code Context

- `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs:38` — `canonical_path_str`, the product's canonicalizer.
- `packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs:6494` and `:6515` — the two expectation sites that skip it.
- `packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs:11281` and `:11296` — the two opaque failure messages.
- `scripts/release-dirt.sh` — the script under test; reads `git status --porcelain -z` through process substitution.

<!-- bee:not-a-deferral: The cause of the second failure is found inside this feature, from the branch's own Windows CI run. It is an investigation step with a named evidence source, not work postponed past the close. -->
## Outstanding Questions

- [ ] Why does `release-dirt.sh` exit non-zero on Windows with an empty stderr? — answered by the branch's Windows run once D2 lands.
<!-- /bee:not-a-deferral -->

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
