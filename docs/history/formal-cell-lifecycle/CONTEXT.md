# formal-cell-lifecycle — Context

**Feature slug:** formal-cell-lifecycle
**Date:** 2026-11-14
**Shaping session:** complete
**Scope:** Quick
**Domain types:** RUN

## Feature Boundary

The owner asked to run pstack's `create-formal-verification-skill` on
the bee cell lifecycle. This feature adds a TLA+ model of how claims and
the cell-closing verbs race, and a project-local verify skill under
`.bee/verify/formal-cell-lifecycle/` that reruns it. It changes no
product code: what the model finds is reported, not fixed.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | The verified target is the claim protocol of one cell: `cells claim` and the five cell-closing verbs (`cap`, `block`, `unclaim`, `reopen`, `drop`) racing each other and the expired-claim sweep, across three sessions. | Cells are claimed and closed by several sessions at once; tests do not reach the step orders between the claim file, the cell lock and the late claim clear. |
| D2 | The tool is TLA+ checked by TLC 1.7.4 on Java Temurin 25, pinned through mise, with the `TLA2TOOLS` and `JAVA` overrides. | The question is whether steps can happen in a bad order, which a model checker answers over every order. |
| D3 | Two properties: a cell the store calls claimed by a session always has that session's claim file (unless a sweep is parking it), and a guarded close (`cap`, `block`, `unclaim`, `reopen`) never succeeds against another session's claim that was never swept or let go stale. | The two promises of claims-and-ownership B23 and R39 that the race can break. |
| D4 | A finding stays a finding: the model describes the current code, TLC's counterexample is reported in the code's words, and no product code changes in this feature. | pstack create-formal-verification-skill, "Findings stay findings". |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/workflow-state/claims-and-ownership.md | suspect | States B23, R36 and R39, the promises the model checks. |
| docs/knowledge/areas/workflow-state/overview.md | unverified | Names the verbs and the claim store. |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/cells/claims.rs` | Claim file creation, the claim gate, the owner-blind best-effort clear. |
| `packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs` | `claim_cell_cross_session_ex`: claim file first, then the cell lock. |
| `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` | `cap_cell_from_flags` and `mutate_cell`: close under the lock, clear after it. |
| `packages/bee-rs/crates/bee/src/verbs/cells/handlers_select.rs` | `sweep_expired_claims` and `sweep_reset_cell`. |
| `packages/bee-rs/crates/bee/src/verbs/cells/trace.rs` | `check_claim_ownership`: no live claim file means any caller passes. |

## Canonical References

- `.bee/verify/formal-cell-lifecycle/SKILL.md` — the verify skill this feature delivers.
