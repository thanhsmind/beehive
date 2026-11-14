# Per-task router and peer quality — Context

**Feature slug:** per-task-router
**Date:** 2026-11-14
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN

## Feature Boundary

bee routes each task the leader receives to its own class playbook and
principles instead of a stored lane route, and every code cell is built by
one worker and verified by a second, separate tester worker that carries the
class principles and a minimum proof kind; it ends at the leader and worker
contracts, the preamble, `bee dispatch prepare` and the cap.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The worker that writes the code and the worker that tests it are always two separate dispatched workers, never one. A code cell never certifies itself. Peers own their quality: implementer and tester each carry the class principles and a minimum proof kind. Store id `676a73b1`. | Today the code writer picks its own narrowest proof and owns tests inside its cell, so a too-narrow proof still reads green. |
| D2 | The implementer writes the code first. A separate tester worker then verifies it independently and adds the tests. The tester never shares a context with the implementer. Store id `c5c00e89`. | Owner chose implementer-first over tester-first. |
| D3 | The two-worker rule covers every code-touching cell in every lane, `tiny` included. A docs-only cell needs no tester; a second worker runs its parity and pointer checks. Store id `95e310df`. | No lane exemption. |
| D4 | Routing runs per task, like poteto-mode: the leader classifies the task in front of it and loads that class's playbook and principles. The session preamble stops printing the Route line and the routed principles from a record the session did not bind; a new unbound session gets no principles until its task is classified. Store id `70c3387f`. | `active_record` (`hooks/session_preamble/budget.rs:440`) falls back to the default state record for an unbound session, so a fresh session got `windows-test-build`'s bugfix principles on a research task. |
| D5 | Five accepted changes. (1) The leader's session todo list opens with the steps of the task's class playbook; a skipped step stays, with `skip:` and its reason. (2) `.bee/LEADER.md` gains a short trigger index from a situation to the skill that owns it, pointing, never restating. (3) `bee dispatch prepare` attaches a Principles block for the cell's route class to every worker prompt, names and one-line rules only, with a holdout share. (4) A worker's cap names each principle it applied and the decision it changed. (5) Each class playbook names a minimum proof kind; the cap refuses a proof line of a weaker kind. Store id `eb8842b0`. | — |

### Agent's Discretion

How the per-task classification is computed and surfaced (a verb, a flag on
`bee orient`, or a prompt hook), how a red tester result returns to the
implementer, the holdout share, and the proof-kind vocabulary are planning
choices, bounded by D1 to D5.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| implementer | the dispatched worker that writes a cell's code |
| tester | the separate dispatched worker that verifies the implementer's result and adds its tests, with no shared context |
| per-task route | the class chosen for the task the leader holds now, not the class stored on a lane record |
| proof kind | the kind of evidence a class requires at cap, for example red-then-green output for a bugfix |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md | suspect | Principle homes, the `classes:` mapping, and the duplication boundary the trigger index must respect |
| docs/knowledge/areas/advisor-protocol/consult-loop.md | suspect | How a worker on a red test consults; the tester loop sits beside it |
| docs/knowledge/patterns/20260713-promote-an-order-to-the-always-loaded-layer.md | pattern | The trigger index enters the always-loaded layer; its transport must ride along |
| docs/knowledge/patterns/20260821-a-door-placed-before-an-older-one-inherits-every-case-the-older-one-answered.md | pattern | A per-task route placed before the lane route inherits every case the lane route answered |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs` | `active_record` fallback and the Route and principles lines |
| `packages/bee-rs/crates/bee/src/principles.rs` | `principle_lines`, the class-to-principles filter |
| `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs` | `bee orient` principles output |
| `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` | worker prompt assembly, Expertise and Learned context blocks |
| `packages/bee/prompts/worker-cell.md` | the worker prompt template |
| `skills/bee-planning/playbooks/` | the nine class playbooks |
| `skills/bee-swarming/SKILL.md` | the worker contract and proof scoping |
| `skills/bee-planning/SKILL.md` | "The writer owns tests TDD-style" — conflicts with D1 |

## Specific Ideas And References

- `docs/history/research/poteto-mode-router.md` — the poteto-mode comparison
  this feature comes from.

## Existing Code Context

### Reusable Assets

- `principles.rs` `principle_lines` — already filters principles by class;
  the worker Principles block can reuse it.
- The Learned context holdout in `prepare.rs` — the pattern for D5 (3).

## Outstanding Questions

- Which recorded decision homes "the writer owns tests TDD-style as part of
  each cell"? Planning must supersede it to land D1.
- The slice-tail consolidated test cell (spec #80/#85): does the tester
  worker replace it or sit beside it?
- A fresh read of pstack newer than commit `b9ddc83` is blocked by the
  cross-project guard until the owner adds `/mnt/data/Projects/refs/pstack`
  to `guards.cross_project_allow`.
