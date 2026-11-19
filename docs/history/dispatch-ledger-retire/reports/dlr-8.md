# dlr-8 — check the transport-check doc and the herding status message

Checked by the leader after three herded checker runs ended `timed_out_idle`.
Commit under check: 0d31c78f0 (dlr-4). The dlr-4 cap report was not read.

| Truth | Result | Evidence |
|-------|--------|----------|
| bee-swarming states the transport check | held | `skills/bee-swarming/SKILL.md:71-74`: "A herding payload carries `transport_ready` and `transport_reason`: read them before you spawn. The payload's `fallback` model applies only when it says `transport_ready: false`" |
| herding status says it does not answer availability and names HERDR_ENV and HERDR_PANE_ID | held | `STATUS_SCOPE_NOTE` in `packages/bee-rs/crates/bee/src/herding.rs`: "reports the calling session's own pane, not worker availability — herdr reads HERDR_ENV and HERDR_PANE_ID, tmux reads TMUX and TMUX_PANE" |
| with the message line reverted the test fails, restored it passes | held | see runs |

## Runs

1. Mutation: replaced `herdr reads HERDR_ENV and HERDR_PANE_ID` with `herdr reads its env and HERDR_PANE_ID`, then
   `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- status_scope_note`
   → `test herding::tests::status_scope_note_says_own_pane_not_availability_and_names_both_transports_variables ... FAILED`, `test result: FAILED. 0 passed; 1 failed`
2. Restored with `git restore`, then
   `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herding::status status_scope_note`
   → `test result: ok. 7 passed; 0 failed`
3. `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity`
   → `test result: ok. 10 passed; 0 failed`
