# Herding run hardening — Context

**Feature slug:** herding-run-hardening
**Date:** 2026-11-19
**Backlog items:** p-2858afbb, p-5614f047, p-7cc2ebe1, p-6b237338
**Shaping session:** complete under gate bypass full (owner said go on the 2026-11-19 backlog sweep)
**Scope:** Standard
**Domain types:** RUN

## Feature Boundary

Four herding fixes in `packages/bee-rs/crates/bee/src/herding/` and its docs:
pointer delivery, wiring-site tests, one home for the worker cap, and a bounded
`bee herding status --json`.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `bee herding run` resolves the dispatch pane id once at run start and passes it to every spawn; no spawn re-reads the focused pane (`herdr.pane_current()`, run.rs near 2699). A test with a fake herdr whose focus moves proves later spawns split off the pinned pane. | p-2858afbb (1). |
| D2 | `start_with_retry` (run.rs near 1452-1470) clears the pane input line before each retry, so a retry after `agent_pane_busy` never types keystrokes twice. A test proves the clear happens before the second start. | p-2858afbb (2). |
| D3 | Each piggery-borrow wiring site gets a test that goes red when its call is removed: the `kill_tree` call in job_verbs.rs Alive arm (near 580), the `check_limits` call in run.rs `run()` (near 6243), the pi belt-contract branch in doctor.rs (near 1056; keep the existing test if it already drives the branch and say so). The claim-probe race in prepare.rs (near 4311) is closed or recorded as a decision with its reason. | p-5614f047. |
| D4 | `herding.limits.concurrency` (code default in herding/wave.rs `default_concurrency`) is the one home of the worker cap. Prose in `skills/bee-herding/README.md` (lines near 90 and 134) and `skills/bee-swarming/references/swarming-reference.md` cites the key, never a literal number. A test shows a fresh run at the default cap is refused with reason `limits.concurrency`. | p-7cc2ebe1. |
| D5 | `bee herding status --json` with no `--job` prints a bounded summary by default (one row per job, no pane scrollback) and spills the full form to a file under `.bee/tmp/output/` named in the output; a flag prints the full form inline. A test asserts the default output stays under a fixed byte cap with many jobs. | p-6b237338; audit `.bee/tmp/token-audit/reports/H-herding-help.md` finding 13 (454 KB inline). |

## Read set

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/herding/run.rs` | D1, D2, D3. |
| `packages/bee-rs/crates/bee/src/herding/job_verbs.rs` | D3. |
| `packages/bee-rs/crates/bee/src/herding/wave.rs` | D4. |
| `packages/bee-rs/crates/bee/src/herding.rs` | D5. |
| `packages/bee-rs/crates/bee/src/doctor.rs` | D3. |

## Not In Scope

- Changing the default cap value.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
