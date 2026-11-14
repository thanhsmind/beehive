# drop-ownership — Context

**Feature slug:** drop-ownership
**Date:** 2026-11-14
**Shaping session:** complete
**Scope:** Quick
**Domain types:** RUN

## Feature Boundary

`bee cells drop` today changes any cell to `dropped` with no ownership
check, so one session can throw away another session's claimed work.
The formal-cell-lifecycle model showed drop as the only closing verb
without the check. This feature makes drop check ownership the same way
cap, block, unclaim and reopen do, and moves the model and its verify
skill to match. It does not fix the separate late-claim-clear race.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `bee cells drop` takes `--session-id` and `--force-ownership`. It refuses with the same ownership refusal as the other closing verbs when a live claim names a different session. An expired, absent, sessionless or matching claim proceeds unchanged. With `--force-ownership` it proceeds and records the audited ownership override in `trace.ownership_overrides`. | The owner picked option B: drop discards another session's work, which claims-and-ownership B23 protects. |
| D2 | Every other drop behavior stays: the reason is required, the status becomes `dropped`, `trace.dropped_reason` is written, an archived cell is refused, and the claim file is cleared after the write. | Scope is the ownership check only. |
| D3 | The formal-cell-lifecycle model adds drop to the guarded closing verbs, and its verify skill moves its sync commit to the merged change. | The model must describe the code it covers. |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/workflow-state/claims-and-ownership.md | suspect | B23 and R39: the ownership rule drop now joins. |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` | `run_drop` and the `unclaim_cell` pattern it copies. |
| `packages/bee-rs/crates/bee/src/verbs/cells/trace.rs` | `guard_claim_ownership`. |
| `.bee/verify/formal-cell-lifecycle/CellLifecycle.tla` | The model's guarded verb set. |
