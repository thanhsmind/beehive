# registry-drift-derivation — CONTEXT

## Asked
Backlog p-e8b98793 (review rev-backlog-fixes-20260816 P2, twice corroborated),
user said do it (2026-08-16): the ghd-1 drift test hardcodes `KNOWN_FLAGS` as a
const copy of `set_gate.rs:555` `keys_known`, so a NEW handler flag shipping
unlisted in the help payload stays green — the exact p-62f0566d defect can recur.

## Found
- `tests/registry_contracts.rs:169-179`: const list, checks payload presence for
  `state.gate` and `gate`; catches deletion, not addition.
- Precedent for source-derived contracts: `tests/opencode_plugin_contracts.rs:163-165`
  does `include_str!` of a source file and parses it.
- Known asymmetry: `owner` is declared in the payload but thrown by
  `run_gate_body` (set_gate.rs:612) — a strict payload⊆handler assertion would
  fail on it today; that asymmetry is out of scope here.

## Will do (locked)
- D1: The drift test derives the flag list by `include_str!` of
  `../src/verbs/state_group/set_gate.rs` and parsing the `keys_known` array
  literal — no more hand copy. Every parsed flag must appear as a parameter of
  BOTH `state.gate` and `gate` payload entries.
- D2: The parse is defensive: if the array literal cannot be found or parses to
  fewer than 5 names, the test fails loudly (a silent empty parse must not pass).
- D3: Deletion coverage stays (missing payload entry or missing flag still fails).
  The reverse direction (payload-declared but handler-rejected, e.g. `owner`)
  stays out of scope.

## Open questions
None.
