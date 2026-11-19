# hrh-3t — Mutation-check the hrh-3 wiring tests (hrh-3, commit b03c0d63)

**Verdict: GREEN.** Both wiring tests go red when their call is removed and pass again once the file is restored from HEAD. hrh-3's cap report was not read.

Verify command, run unchanged for every step:

    PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herding::

## Baseline on HEAD 0d9d9a36c

    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5510 filtered out; finished in 0.00s
    test result: ok. 851 passed; 0 failed; 0 ignored; 0 measured; 4660 filtered out; finished in 39.48s

## Mutation 1 — remove the kill_tree call in the cancel Alive arm

`packages/bee-rs/crates/bee/src/herding/job_verbs.rs:580`, the whole line deleted:

    -                    kill_tree(tree, pid, TREE_KILL_GRACE);

Result (exit 101):

    test herding::job_verbs::tests::cancel_on_an_alive_pane_process_asks_the_tree_to_kill_it ... FAILED
    panicked at crates/bee/src/herding/job_verbs.rs:1538:9:
    cancel must term the pane pid: []
    test result: FAILED. 850 passed; 1 failed; 0 ignored; 0 measured; 4660 filtered out; finished in 36.51s

Restored with `git checkout HEAD -- packages/bee-rs/crates/bee/src/herding/job_verbs.rs` (exit 0):

    test herding::job_verbs::tests::cancel_on_an_alive_pane_process_asks_the_tree_to_kill_it ... ok
    test result: ok. 851 passed; 0 failed; 0 ignored; 0 measured; 4660 filtered out; finished in 38.62s

## Mutation 2 — remove the check_limits call in run()

`packages/bee-rs/crates/bee/src/herding/run.rs:6261-6272`, the whole `if let Some(refusal) = super::wave::check_limits(...) { ... return ExitCode::FAILURE; }` block deleted. `export_depth` stays, so the build still compiles.

Result (exit 101):

    test herding::run::tests::run_refuses_at_the_depth_limit_before_any_side_effect ... FAILED
    panicked at crates/bee/src/herding/run.rs:7969:9:
    assertion `left == right` failed: a depth limit of 0 must refuse a fresh run
      left: ExitCode(unix_exit_status(0))
     right: ExitCode(unix_exit_status(1))
    test result: FAILED. 850 passed; 1 failed; 0 ignored; 0 measured; 4660 filtered out; finished in 22.32s

Restored with `git checkout HEAD -- packages/bee-rs/crates/bee/src/herding/run.rs` (exit 0):

    test herding::run::tests::run_refuses_at_the_depth_limit_before_any_side_effect ... ok
    test result: ok. 851 passed; 0 failed; 0 ignored; 0 measured; 4660 filtered out; finished in 20.51s

## Notes

- Each mutation failed exactly one test, the one hrh-3 added, so no older test covered either call.
- The kill_tree test fails on its first assertion (the Term signal); the Kill assertion did not run in the mutated build.
- Mutation 2 removed the whole refusal block, not only the call expression, since the call result is used only there.
- Out of scope and not checked: the doctor.rs pi belt branch and the prepare.rs claim-probe race (D3's other two parts); hrh-3 says it filed them as a follow-up PBI.
- No source was edited in the commit; `git status` was clean after each restore.
