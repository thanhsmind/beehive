
# claim-clear-identity — Context

**Feature slug:** claim-clear-identity
**Date:** 2026-11-14
**Shaping session:** complete
**Scope:** Quick
**Domain types:** RUN

## Feature Boundary

The formal-cell-lifecycle model found that a claim-clearing transition
removes the claim file after it lets the cell lock go, whoever owns that
file. A newer claim can be deleted that way, and the cell then reads as
claimed with no claim file: the ownership check lets any session through
and the sweep never frees the cell. This feature makes the clear remove
only the exact claim it saw under the lock, and moves the model and its
verify skill to the fixed code. Nothing else about the closing verbs
changes.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Every claim-clearing transition (cap, block, unclaim, reopen, drop, park, dissent, a NEEDS_REVISION judge reopen) reads the claim's `acquired_at` under the `cells:<id>` lock. After the lock it removes the claim file only if the file still carries that `acquired_at`. A claim absent under the lock clears nothing. Contract `claim-clear-identity`. | `acquired_at` is written once at claim creation and is not changed by renew or adoption, so it names one exact claim. An owner name is not enough: the same session can claim again in between. |
| D2 | The clear stays best-effort: a failure to read, gate or remove is swallowed, as today. | Scope is which file is removed, not the failure posture. |
| D3 | The model's `code` behavior becomes the exact-claim clear, its old owner-blind clear becomes a broken variant that must still fail rule 1, and the verify skill moves its sync commit to the fix. The plain check must then exit 0. | The model must describe the code it covers, and each rule must still be shown able to fail. |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/workflow-state/claims-and-ownership.md | suspect | R36 and the known-gap note this feature closes. |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/cells/claims.rs` | `release_claim_file_best_effort`; `acquired_at` written once in `claim_cell_file`. |
| `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` | `mutate_cell` and `cap_cell_from_flags`: the lock and the late clear. |
| `packages/bee-rs/crates/bee/src/verbs/cells/handlers_meta.rs` | The judge-record reopen's late clear. |
| `.bee/verify/formal-cell-lifecycle/CellLifecycle.tla` | The model's clear action. |
