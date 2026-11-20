# stw-2 — Prove the shared target warning red then green

**Outcome:** six tests, every name containing `shared_target`, pin contract C1
(6fd09277) on the stw-1 commit `aaa06810e`.

## Tests

`packages/bee-rs/crates/bee/src/verbs/test_runner.rs`, `mod tests`, real paths in a tempdir:

- `shared_target_outside_the_checkout_with_a_live_worker_elsewhere_warns` — the line names the dir, the other checkout, and says a red may be false.
- `shared_target_unset_gives_no_warning` — `None` for an unset value.
- `shared_target_blank_gives_no_warning` — `None` for a blank value.
- `shared_target_inside_the_checkout_gives_no_warning` — `None` for a target inside the checkout.
- `shared_target_with_live_workers_only_in_this_checkout_gives_no_warning` — `None` when every live worker is in this checkout.

`packages/bee-rs/crates/bee/src/herding/wave_ledger.rs`, `mod tests`, fixture `.bee/wave-ledger.jsonl` written through `append_wave`:

- `shared_target_live_worker_worktrees_lists_only_unresolved_workers_of_fresh_waves` — a fresh wave with one unresolved and one resolved worker, plus a wave older than the stale window with an unresolved worker; only the fresh unresolved worker's worktree is listed.

## Red run

The production hunks of `aaa06810e` were reverse-applied
(`git show aaa06810e -- packages/bee-rs/crates | git apply -R`), which put
`herding.rs`, `wave_ledger.rs` and `test_runner.rs` back to the stw-1 parent
with the new tests kept. Verify:

```
PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- shared_target
error[E0425]: cannot find function `live_worker_worktrees` in this scope
error[E0425]: cannot find function `shared_target_warning` in this scope   (x5)
error: could not compile `bee` (bin "bee" test) due to 6 previous errors; 7 warnings emitted
```

Restored with `git apply` of the same patch; both files compared byte-equal
to the pre-revert copies, and `git status` showed only the two test-edited files.

## Green run

```
PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- shared_target
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 5557 filtered out; finished in 0.00s
```

Deviations: none. Production code untouched.

## Full proof

```
cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings && cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml
test result: ok. 5534 passed; 0 failed; 29 ignored; 0 measured; 0 filtered out; finished in 14.44s
exit 0, no test binary reported a failure
```
