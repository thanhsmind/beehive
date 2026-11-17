# gis-7t check: skip the bee scratch area in the shared nested checkout scan (issue 158)

Checked: cell gis-7, commit c42ebce60, decision D8 (93c9c272). The tester did not use the gis-7 worker's context.

## Reproduction

The parent commit is c42ebce60^. Its only non-test change is the skip at `packages/bee-rs/crates/bee/src/nested_checkout.rs:183`. On the parent scan code, with a second live session and a git checkout at `.bee/tmp/job/repo`, `has_any_shared_nested_checkout` returns `true`. That is the predicate behind the issue 158 refusal from `worktree new` and `feature start`. On the fixed code the same fixture returns `false`.

## Truths

| Truth | Result | Anchor |
|---|---|---|
| a nested git checkout under .bee/tmp is not reported | PASS | skip at `nested_checkout.rs:183`; gis-7 test `nested_checkout.rs:339` (assert at :347, `.bee/tmp/job/repo`); tester edge test `nested_checkout.rs:366` (assert at :374, `.bee/tmp` is itself a checkout) |
| a nested checkout elsewhere in the repo is still reported | PASS | `nested_checkout.rs:353` (`src/tmp/repo`), `:361` (`.bee/other/repo`), tester edge test `:380` (`.bee/tmp-old/repo`, a name that only starts with tmp) and `:388` (`pkg/.bee/tmp/repo`, a `.bee/tmp` below the root); existing `nested_checkout_with_a_second_live_session_is_flagged` at `:267` |

## Red on the parent commit

The tester removed only the three-line skip at `nested_checkout.rs:183`, which gives the parent's scan code. Both test functions stayed in place. Result: `8 passed; 2 failed`.

- `the_bee_scratch_area_is_skipped_but_other_bee_dirs_are_not` FAILED at `nested_checkout.rs:342`: "a clone under .bee/tmp is never reported"
- `only_the_root_scratch_area_is_skipped_not_look_alikes` FAILED at `nested_checkout.rs:369`: "a .bee/tmp that is itself a git checkout is skipped"

The skip was then restored, byte for byte.

Truth 2 guards against regressions. The parent commit already reported these checkouts, so its assertions cannot be red on the parent alone. They are red there only as part of their test functions, where the `.bee/tmp` assertion fails first. The tester edge test shows that the skip matches the exact path `<root>/.bee/tmp`. A prefix look-alike (`.bee/tmp-old`) and a nested project's own `.bee/tmp` are still reported.

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- nested_checkout`

Result: `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 5396 filtered out`. The run includes `nested_checkout::tests::the_bee_scratch_area_is_skipped_but_other_bee_dirs_are_not ... ok` and `nested_checkout::tests::only_the_root_scratch_area_is_skipped_not_look_alikes ... ok`.

## Not checked

The tester did not repeat the end-to-end CLI repro (`bee worktree new` with a live second session), because the brief allows no bee command except `bee herding notify`. The check is made at the scan predicate those verbs call.
