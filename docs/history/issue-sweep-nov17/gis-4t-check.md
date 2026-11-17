# gis-4t: independent check of gis-4 (issue 160)

Cell under test: gis-4, commit dd92cc8a3. Parent commit: 514adb6f8. Decision: 8ed6f4ae (CONTEXT D5).

## Truths

| Truth | Result | Fix site | Tests |
| --- | --- | --- | --- |
| A cell whose verify contains the separator caps with proof `<verify> — green:unit — <scope>` | PASS | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:177`, `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:332` | `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:12036` (gis-4), `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:12098` (gis-4t edge) |
| A proof whose command differs from verify is still refused | PASS | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:343` | `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:12054` (gis-4), `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:12072` (gis-4t edge) |

## Red on the parent commit

Run in a detached worktree at 514adb6f8, with this commit's `tests.rs` copied in. One older test calls the two-argument `parse_report_flag(&report, "")` that gis-4 introduced; that one call was reverted to the parent's one-argument form so the module compiles.

`cargo test --release --bin bee -- gis4` on the parent: `2 passed; 2 failed`.

- `gis4_verify_containing_the_separator_caps_with_the_whole_verify_as_command` FAILED: `result segment is "--nocapture"`. This is the reported symptom: the parser splits at the separator inside the verify.
- `gis4t_separator_verify_with_a_red_result_is_refused_as_red_and_writes_nothing` FAILED: the parent refuses with `result segment is "--nocapture"`, never reaching the red check.
- `gis4_verify_containing_the_separator_still_refuses_a_different_command` passed.
- `gis4t_separator_verify_refuses_a_near_miss_and_a_plain_different_command` passed.

Truth 2 is a no-regression truth. The parent already refused a different command, so no test of it can be red on the parent. Both truth-2 tests pass on the parent and on the fix. An earlier draft of the gis-4t test asserted the verify-mismatch message for a separator-bearing near miss. It was red on the parent and also red on the fix (see Notes), so it was rewritten.

## Symptom reproduced with the real binary

Scratch repo with `.bee/config.json` `{"commands":{"test":"none"}}` and one claimed cell `sep` whose verify is `cargo test -p bee — --nocapture`. Command:
`bee cells cap --id sep --report '{"outcome":"o","commit":"c","files":[],"tests":"cargo test -p bee — --nocapture — green:unit — touched tests.rs","deviations":[]}'`

Parent binary (built at 514adb6f8):

```
cells finish: --report key "tests" result segment is "--nocapture" — a cap records HOW the change was shown to work, ...
```

The cell stays `claimed`.

Fixed binary (worktree HEAD):

```
Capped sep at 2026-11-17T08:46:35.654Z (tests: undeclared).
```

The cell is `capped` with `verify_command` `cargo test -p bee — --nocapture`, `verify_output` `green:unit`, `verification_evidence` `touched tests.rs`.

## Edge cases added by gis-4t

- `gis4t_separator_verify_with_a_red_result_is_refused_as_red_and_writes_nothing`: `<verify> — red — <scope>` reaches the red-is-fix-first refusal (`result segment is "red"`), and the cell file is byte-identical afterwards.
- `gis4t_separator_verify_refuses_a_near_miss_and_a_plain_different_command`: two near misses are refused: `cargo test -p bee — --exact — …` and `cargo test -p bee — --nocapture --exact — …`, where the second has the verify as a prefix but no separator directly after it. A plain `cargo test -p other — green:unit — …` is refused with `does not match approved cell verify command`. The cell file is unchanged.

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- verbs::cells`

Result on the worktree: `test result: ok. 530 passed; 0 failed; 10 ignored; 0 measured; 4875 filtered out; finished in 5.16s`.

The four gis-4 tests ran by name and passed:

- `verbs::cells::tests::gis4_verify_containing_the_separator_caps_with_the_whole_verify_as_command` (gis-4)
- `verbs::cells::tests::gis4_verify_containing_the_separator_still_refuses_a_different_command` (gis-4)
- `verbs::cells::tests::gis4t_separator_verify_refuses_a_near_miss_and_a_plain_different_command` (gis-4t)
- `verbs::cells::tests::gis4t_separator_verify_with_a_red_result_is_refused_as_red_and_writes_nothing` (gis-4t)

## Notes

- Misleading refusal text, not a truth failure. A different command that itself contains the separator, such as `cargo test -p bee — --exact — green:unit — …` against verify `cargo test -p bee — --nocapture`, falls back to the first-separator split. It is refused as `result segment is "--exact"`, not as a verify mismatch. The cap is still refused. The message points at the result segment instead of the command.
- The gis-4 test for truth 2 accepts either `refused` or `result segment` in the message. Its separator-bearing proof hits the result-segment path described above, so it never reaches the mismatch check at handlers_close.rs:343. The gis-4t test covers that check directly.
