# hrh-6t — Check the flag vocabulary pin (hrh-6, commit 30104b7fe)

**Verdict: GREEN on all three truths.** I did not read hrh-6's cap report. I edited no source. The scratch edit to `catalog.rs` was reverted with `git checkout HEAD -- packages/bee-rs/crates/bee/src/catalog.rs`, and `git diff --stat` on that file was empty afterwards.

The constant at HEAD is `const PINNED_FLAG_COUNT: usize = 258;` (`packages/bee-rs/crates/bee/src/catalog.rs:823`), checked by `catalog::tests::distinct_flag_vocabulary_is_pinned_so_growth_is_a_decision`.

All three runs used the cell verify:

    PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- catalog::tests

## Run 1: HEAD, pin at 258. PASS

    test catalog::tests::distinct_flag_vocabulary_is_pinned_so_growth_is_a_decision ... ok
    test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 5500 filtered out; finished in 0.01s

## Run 2: scratch edit, pin set back to 257. FAILS AS EXPECTED, naming 258

    test catalog::tests::distinct_flag_vocabulary_is_pinned_so_growth_is_a_decision ... FAILED
    thread 'catalog::tests::distinct_flag_vocabulary_is_pinned_so_growth_is_a_decision' (4126871) panicked at crates/bee/src/catalog.rs:828:9:
    assertion `left == right` failed: distinct flag-name count changed from 257 to 258. A new spelling for an existing concept makes a caller's knowledge of one verb useless on the next command, so growing this vocabulary is a decision to take on purpose: first check whether an existing flag name already means what the new one means and reuse it; only if none does, bump PINNED_FLAG_COUNT above and say why in the commit.
      left: 258
      right: 257
    test result: FAILED. 18 passed; 1 failed; 0 ignored; 0 measured; 5500 filtered out; finished in 0.05s

The real distinct count is 258 (`left`), so the one new flag hrh-6 added (`--full` on `bee herding status`) is the only growth.

## Run 3: restored from HEAD. PASS

    test catalog::tests::distinct_flag_vocabulary_is_pinned_so_growth_is_a_decision ... ok
    test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 5500 filtered out; finished in 0.00s

## Not run

- The full proof command (clippy plus the whole test suite). This cell changes only this report, so the cell verify is the narrowest proof.
