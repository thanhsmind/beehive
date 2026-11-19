# tch-5t — the bee crate folder stays clean after the tests

Contract: no-crate-bee-dir. Target: tch-5, commit bc3bb8bbd.

## Finding: the tch-5 commit holds only the log deletion

`git show --stat bc3bb8bbd` lists one file: `packages/bee-rs/crates/bee/.bee/logs/timings.jsonl` (56 deletions).
The test changes the commit message describes are not in the commit. They are an uncommitted edit to
`packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs`: the `TempCwd` guard, the temp root in
`test_deploy_authorization_cli_routing`, and the new test
`dispatch_authorize_timings_land_under_the_temp_root_not_the_crate_dir`.
At HEAD alone the routing test still writes timings into the crate folder.
Every result below ran against that uncommitted working-tree edit.

## Verify

`cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- dispatch::authorize timings`

    test verbs::drivers::tests::dispatch_authorize_timings_land_under_the_temp_root_not_the_crate_dir ... ok
    test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 5489 filtered out; finished in 0.00s

After it ran, `packages/bee-rs/crates/bee/.bee` did not exist, and `git status` showed only the uncommitted tests.rs edit.
The `dispatch::authorize` filter matches no test. Only the `timings` filter selects the new test, and the
routing test `test_deploy_authorization_cli_routing` does not run under this verify.

## Mutation 1: send the new test back to the crate folder

Replaced `TempCwd::enter(&root)` with `TempCwd::enter(Path::new(env!("CARGO_MANIFEST_DIR")))`:

    test verbs::drivers::tests::dispatch_authorize_timings_land_under_the_temp_root_not_the_crate_dir ... FAILED
    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5504 filtered out; finished in 0.00s

It created `crates/bee/.bee/logs/timings.jsonl`. I restored the file from a byte copy, not from HEAD, because
HEAD does not hold the edit, and removed the stray folder:

    test verbs::drivers::tests::dispatch_authorize_timings_land_under_the_temp_root_not_the_crate_dir ... ok
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5504 filtered out; finished in 0.00s

## Mutation 2: take the temp root out of the routing test

    test verbs::drivers::tests::test_deploy_authorization_cli_routing ... ok
    test result: ok. 1 passed; 0 failed; ...
    ?? packages/bee-rs/crates/bee/.bee/logs/timings.jsonl

With its temp root removed, the routing test still passes and dirties the crate folder again. No test catches it.
The new test protects only its own call. Restored:

    test verbs::drivers::tests::test_deploy_authorization_cli_routing ... ok
    test verbs::drivers::tests::dispatch_authorize_timings_land_under_the_temp_root_not_the_crate_dir ... ok
    test result: ok. 2 passed; 0 failed; ...

`git status` after the restore showed only ` M .../drivers/tests.rs`, and `cmp` matched it to the pre-mutation copy.

## Verdict

The working tree meets the contract. Commit bc3bb8bbd does not: tch-5 needs to commit its tests.rs edit.
