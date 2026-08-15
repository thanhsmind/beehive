# gate-help-drift — CONTEXT

## Asked
Backlog p-62f0566d: `bee --help --json` / `bee state gate --help --json` do not list
`--actor`, `--bypass-level`, `--reason` on `state gate` / `gate`, though the binary
accepts them (trun-2 shipped them outside the payload's file scope).

## Found
- Source of truth for help: `packages/bee-rs/crates/bee/src/generated/registry_payload.json`,
  compiled in via `include_str!` (`registry.rs:13`).
- Generator `scripts/export_registry_payload.mjs` no longer exists in this repo
  (tests/registry_contracts.rs:157 still names it). Payload is now hand-maintained.
- Flags live in `verbs/state_group/set_gate.rs` (known-flags list at :555; parsing
  and validation at :645-668). `state.gate` and flow alias `gate` both understate.

## Will do (locked)
- D1: Hand-edit the payload: add `actor`, `bypass-level`, `reason` parameter entries
  (with the enum/requirement facts from set_gate.rs) to both `state.gate` and `gate`.
- D2: Add a drift test pinning that every flag in set_gate.rs's known-flags list
  appears in the payload's parameters for both commands, so a new flag cannot ship
  unlisted again.
- Out of scope: restoring the generator script.

## Open questions
None.
