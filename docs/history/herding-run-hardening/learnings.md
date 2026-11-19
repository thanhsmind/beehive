# Herding run hardening — learnings (PBI p-2858afbb, D1 and D2)

- The caller pane is read once in `run()` and travels as `Options.dispatch_pane`, and as `--dispatch-pane` to the detached runner. Pinning only inside `execute_new` would not have covered the detached child, which is a separate process.
- A new `Options` field breaks every struct literal in `herding/tests.rs` and the `run.rs` test helpers, so a cell that adds one lists `tests.rs` in its files up front.
- `start_with_retry` takes a `clear_input` callback, so the retry loop stays testable with no transport; the `ctrl+u` key name is the one herdr's own docs use for `ctrl+c`.
- Tester cells need a logged `contract:` decision before they can be claimed, and a red-first floor means the cap proof result is `green:red-first`.
- Recorded as a bee decision: `contract:run-pane-pin-and-start-retry`.
- Out of this run: D3, D4 and D5 of CONTEXT.md belong to the other PBIs of the feature, so the routing door stays open for them.
