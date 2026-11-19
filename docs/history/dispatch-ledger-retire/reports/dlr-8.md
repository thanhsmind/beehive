# dlr-8: check the transport-check doc and the herding status message

Checker for dlr-4 (commit 0d31c78f0) against CONTEXT.md D4. dlr-4's cap report was not read.

Outcome: partial. Truths 1, 2 and 4 hold. Truth 3 does not: with the
message line removed, the herding status tests still pass.

## Truth 1: the swarming doc states the transport check

`skills/bee-swarming/SKILL.md:71-74`, step 3 of the Delegation contract:

> A herding payload carries `transport_ready` and
> `transport_reason`: read them before you spawn. The payload's `fallback`
> model applies only when it says `transport_ready: false`; `bee herding
> status` reports the calling session's own pane, never worker availability.

This matches D4: a dispatch reads both fields, and the fallback model
applies only on `transport_ready: false`. Holds.

## Truth 2: herding status says it does not answer availability and names the variables

`packages/bee-rs/crates/bee/src/herding.rs:1599`:

> const STATUS_SCOPE_NOTE: &str = "herding: this line reports the calling session's own pane, not worker availability — herdr reads HERDR_ENV and HERDR_PANE_ID, tmux reads TMUX and TMUX_PANE; a dispatch carries its own transport_ready and transport_reason";

It is printed at `herding.rs:1591`, in the plain (non `--json`) branch of
`status_with_panes_and_transport_and_paseo`, right after the
`herding: enabled=… transport=…` line:

> println!("{STATUS_SCOPE_NOTE}");

That branch prints the note every time, whether or not a job is
built. The `--json` branch does not include it. Holds.

## Truth 3: mutation, line removed, then restored

The worktree already had the mutation when this round started: `git diff` showed the
`println!("{STATUS_SCOPE_NOTE}");` line deleted at `herding.rs:1591` and
nothing else changed. That is the exact line the cell's action says to
remove, so I ran the mutation against it as it was.

The verify filter `herding::status` matches 6 tests, all of them in
`herding::status_digest::tests`. None of them covers the message. The tests
for status itself live under `herding::tests::status*`, so I also ran that
filter (19 tests).

Under the mutation:

```
cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herding::status
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 5513 filtered out
cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herding::tests::status
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 5500 filtered out
```

Expected a failure and got none. The mutation survives. The only test
about the message is
`status_scope_note_says_own_pane_not_availability_and_names_both_transports_variables`
(`herding.rs:2123-2127`). It checks the `STATUS_SCOPE_NOTE` constant's
text and never checks that status prints it. So deleting the `println!`
breaks the behavior D4 asks for, and nothing catches it.

After `git checkout HEAD -- packages/bee-rs/crates/bee/src/herding.rs`:

```
cargo test ... --bin bee -- herding::tests::status
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 5500 filtered out
cargo test ... --test pointer_integrity
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Truth 3 does not hold. Making it hold requires a test that captures the plain
`herding status` output and looks for the note, or a change that sends the
printed lines through a function a test can call. Either one edits `herding.rs`,
and this cell is not allowed to edit the files it checks. That work is left
for a new cell.

## Verify filter

`herding::status` is not a zero-match filter: it matches 6 `status_digest`
tests. But it misses every test for the status output. A filter that
covers the status tests is `herding::tests::status` (19 tests).

## Edits

This cell edited no source. The only file written is this report.
