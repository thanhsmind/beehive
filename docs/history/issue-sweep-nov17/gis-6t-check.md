# gis-6t — independent check of gis-6 (issue #162, D7 c9826c53)

Checked commit 10c4109d9 against its parent 5933d6721.

## Symptom reproduction

The sandbox was a `git init` with `.bee/onboarding.json` set to `{}` and one
`.bee/decisions.jsonl` record stamped `model-role-split`. I ran
`bee decisions reattribute --id aaaaaaaa --to prompt-work-record --dry-run`
on both builds.

| Build | Dry-run output | Store sha256 (first 16) before → after |
|-------|----------------|-----------------------------------------|
| parent 5933d6721 | `corrected 1 — aaaaaaaa : model-role-split -> prompt-work-record` | a8bd26d01abe938d → 6a33958f5bea3bc6 (rewritten) |
| fixed (this branch) | `would correct 1 — aaaaaaaa : model-role-split -> prompt-work-record` | a8bd26d01abe938d → a8bd26d01abe938d (unchanged) |

On the fixed build, the real run that follows prints
`corrected 1 — aaaaaaaa : model-role-split -> prompt-work-record`, and the
store then holds `"feature":"prompt-work-record"`. The symptom is gone.

## Truths

1. **A named reattribute dry run leaves the decision store byte-identical. PASS.**
   - Fix: `packages/bee-rs/crates/bee/src/verbs/decisions/verbs_write.rs:473` now passes `dry_run` to the named arm. `verbs_write.rs:578` skips the write when it is set.
   - Tests:
     - `packages/bee-rs/crates/bee/src/verbs/decisions/tests.rs:2096` was written by gis-6.
     - `tests.rs:2126` is my edge case. It covers an unstamped record, a short id prefix, a shared store with a non-JSON line and a sibling record, and two repeated dry runs. It checks that the store is byte-identical, that no `.jsonl.reattribute.tmp` is left behind, and that the output reads `(none) -> prompt-work-record`.
2. **The named real run still corrects the record. PASS.**
   - Code path: `verbs_write.rs:578`, the write branch when `dry_run` is false.
   - Tests:
     - `tests.rs:2096` checks the real run after a dry run.
     - `tests.rs:2126` checks that the real run corrects only the first record and leaves the non-JSON line and the sibling line byte-for-byte.

## Fails on parent

- **Unit tests:** I copied both tests onto the parent and they do not compile: `error[E0061]: this function takes 4 arguments but 5 arguments were supplied`, 10 errors. The parent's `do_reattribute_named` has no dry-run parameter at all, which is the defect itself.
- **Behavior:** the CLI reproduction above shows the parent rewriting the store during a dry run.

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- verbs::decisions`

`test result: ok. 109 passed; 0 failed; 0 ignored; 0 measured; 5297 filtered out`

These new tests appear in the run, each `... ok`:

- `verbs::decisions::tests::a_named_reattribution_dry_run_writes_nothing_and_the_real_run_still_corrects`
- `verbs::decisions::tests::a_named_dry_run_on_an_unstamped_record_in_a_shared_store_leaves_every_byte_and_no_temp_file`
