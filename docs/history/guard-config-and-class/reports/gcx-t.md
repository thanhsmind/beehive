# gcx-t: proof of the three guard changes

Contract: docs/history/guard-config-and-class/CONTEXT.md (D1, D2, D3). I checked the code and tests directly and did not read the implementers' cap reports.

Verify command, the same for every run:

    PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- write_guard::

Method: for each change, I reverse-applied that commit's hunks to `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs` (`git show <sha> -- checks.rs | git apply -R`) and kept the tests. Then I ran the verify and restored the file with `git checkout HEAD -- checks.rs`. I edited no source.

## Baseline (HEAD 0b39f59e9)

    test result: ok. 368 passed; 0 failed; 0 ignored; 0 measured; 5143 filtered out; finished in 0.29s

## D1 / gcx-1 (1575152d8): cli slots read from team and models

Reverted:

    test hooks::write_guard::tests::worker_outward_reads_cli_slots_from_team_and_models_together ... FAILED
    test result: FAILED. 367 passed; 1 failed; 0 ignored; 0 measured; 5143 filtered out; finished in 0.25s

Restored: green (see the final run below).

## D2 / gcx-2 (6359303db): research class refuses a source edit

Reverted:

    test hooks::write_guard::tests::research_class_refuses_a_source_edit_naming_the_class_and_remedy ... FAILED
    test result: FAILED. 367 passed; 1 failed; 0 ignored; 0 measured; 5143 filtered out; finished in 0.23s

`research_class_still_allows_docs_and_bee_writes` and `a_writing_class_still_allows_a_source_edit` stay green when the change is reverted. That is correct: both are allow arms that held before the change, and they guard against the new refusal reaching too far.

## D3 / gcx-3 (0b39f59e9): leader with the worktree grant may start an agent

Reverted:

    test hooks::write_guard::tests::worker_outward_lets_the_grant_holder_without_a_claim_start_an_agent ... FAILED
    test hooks::write_guard::tests::worker_outward_still_refuses_a_grant_holder_with_a_claimed_cell ... FAILED
    test result: FAILED. 366 passed; 2 failed; 0 ignored; 0 measured; 5143 filtered out; finished in 0.15s

The claimed-cell test fails at tests.rs:2125. That line is the leader-allowed assertion, not the refusal, so a full revert cannot prove the claimed-worker refusal.

### Claimed worker is refused (truth 2)

To prove it, I applied a scratch mutation: `leader_holds_worktree_grant` returns `Ok(true)` instead of checking `session_claimed_features`, so any grant holder passes even with a claim.

    test hooks::write_guard::tests::worker_outward_still_refuses_a_grant_holder_with_a_claimed_cell ... FAILED
    panicked at crates/bee/src/hooks/write_guard/tests.rs:2122:9
    test result: FAILED. 367 passed; 1 failed; 0 ignored; 0 measured; 5143 filtered out; finished in 0.10s

tests.rs:2122 is `assert_eq!(e.code, 2, ...)` for the worker session that holds `cell-1`. After gcx-3, a worker session with a claimed cell is refused an outward start in the worktree, and the test goes red if the claim check is removed.

## Final green (checks.rs restored from HEAD)

    test hooks::write_guard::tests::a_writing_class_still_allows_a_source_edit ... ok
    test hooks::write_guard::tests::research_class_refuses_a_source_edit_naming_the_class_and_remedy ... ok
    test hooks::write_guard::tests::research_class_still_allows_docs_and_bee_writes ... ok
    test hooks::write_guard::tests::worker_outward_lets_the_grant_holder_without_a_claim_start_an_agent ... ok
    test hooks::write_guard::tests::worker_outward_reads_cli_slots_from_team_and_models_together ... ok
    test hooks::write_guard::tests::worker_outward_still_refuses_a_session_without_the_grant ... ok
    test hooks::write_guard::tests::worker_outward_still_refuses_a_grant_holder_with_a_claimed_cell ... ok
    test result: ok. 368 passed; 0 failed; 0 ignored; 0 measured; 5143 filtered out; finished in 0.29s

## Not checked

- `worker_outward_still_refuses_a_session_without_the_grant` was not mutation-tested. The refusal it covers existed before gcx-3, and the revert leaves it green as expected.
- I did not run the brief's full proof command (clippy plus the whole workspace test suite). This cell changes only this report.
