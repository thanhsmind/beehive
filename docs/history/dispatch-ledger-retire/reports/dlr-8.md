# dlr-8 — Check the transport-check doc and the herding status message

Checker for dlr-4 (commit 0d31c78f0) and dlr-5. I checked CONTEXT.md D4 against the code at HEAD 3819638f4 without reading dlr-4's cap report. I edited no source.

## D4, as locked

> The remaining work is doc: the bee-swarming Delegation contract states that a dispatch reads `transport_ready` and `transport_reason`, and that the fallback model applies only when the payload says `transport_ready: false`. `bee herding status` without a built job says it does not answer availability and names `HERDR_ENV` and `HERDR_PANE_ID`.

## Check 1 — skills/bee-swarming/SKILL.md — PASS

`skills/bee-swarming/SKILL.md:71-74`:

> A herding payload carries `transport_ready` and
> `transport_reason`: read them before you spawn. The payload's `fallback`
> model applies only when it says `transport_ready: false`; `bee herding
> status` reports the calling session's own pane, never worker availability.

The doc says a dispatch reads both fields, and that the fallback applies only on `transport_ready: false`.

## Check 2 — the herding status message — PASS

`packages/bee-rs/crates/bee/src/herding.rs:1600`:

> const STATUS_SCOPE_NOTE: &str = "herding: this line reports the calling session's own pane, not worker availability — herdr reads HERDR_ENV and HERDR_PANE_ID, tmux reads TMUX and TMUX_PANE; a dispatch carries its own transport_ready and transport_reason";

`herding.rs:1591` prints it in the plain branch (`println!("{STATUS_SCOPE_NOTE}");`), right after the `herding: enabled=… transport=…` line. Both lines print whether or not `--job` is given, so the no-job case is covered. The message says it does not answer worker availability and names `HERDR_ENV` and `HERDR_PANE_ID`.

## Check 3 — verify — PASS, after fixing the filter

As written, `-- herding::status` matched 6 tests, all in `herding::status_digest::tests`. The dlr-4 test is `herding::tests::status_scope_note_says_own_pane_not_availability_and_names_both_transports_variables`, and `herding::status` is not a substring of that path, so the verify as written never ran it. Output of the filter as written:

> test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 5513 filtered out

The corrected filter is `-- herding::status herding::tests::status_scope_note`. Final run on the restored HEAD:

> test herding::tests::status_scope_note_says_own_pane_not_availability_and_names_both_transports_variables ... ok
> test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 5512 filtered out; finished in 0.01s
> test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s   (pointer_integrity)

## Check 4 — mutation of the message line — PASS

I replaced `herding.rs:1600` in a scratch edit with `const STATUS_SCOPE_NOTE: &str = "herding: status";` and ran the test:

> test herding::tests::status_scope_note_says_own_pane_not_availability_and_names_both_transports_variables ... FAILED
> panicked at crates/bee/src/herding.rs:2126:13:
> own pane
> test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5518 filtered out

Then I restored the file with `git checkout HEAD -- packages/bee-rs/crates/bee/src/herding.rs`, re-ran, and the test passed (Check 3 output). The tree was clean afterwards.

## Gap found — the printing is not under test

As a second scratch mutation, I deleted only the `println!("{STATUS_SCOPE_NOTE}");` line (`herding.rs:1591`) and ran every `herding::` test:

> test result: ok. 845 passed; 0 failed; 0 ignored; 0 measured; 4674 filtered out; finished in 28.16s

The dlr-4 test checks the text of the constant, not that `bee herding status` prints it. If someone drops the print, no test fails. I restored from HEAD. Fixing this means adding an output-level test, which is outside this cell's scope ("editing any checked file"), so it goes in the report as a finding.

## dlr-5

dlr-5's own report says the parity checks passed with no edits. `pointer_integrity` is green here (10 passed), which agrees with it.
