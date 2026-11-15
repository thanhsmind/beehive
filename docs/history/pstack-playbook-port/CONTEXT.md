# pstack Playbook Port — Context

**Feature slug:** pstack-playbook-port
**Date:** 2026-11-15
**Shaping session:** complete
**Scope:** Deep
**Domain types:** ORGANIZE, RUN

## Feature Boundary

bee gains the full pstack poteto-mode playbook set (23 playbooks plus the
figure-it-out fallback) as situation playbooks under its nine route classes,
and the router picks a situation, not only a class; the feature ends when a
leader can route a task to one situation, see that situation's steps and
principles, and have the plan check hold the citation.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | All 23 pstack poteto-mode playbooks come into bee as situation playbooks under the nine route classes, plus a figure-it-out style fallback for a task no playbook fits. The pstack method text is the base, not a short bee summary. Store id `d480d467`; supersedes `da83bb92` and `82331837`. | Owner judged the set high quality. The 2026-09-24 gap-check judged content coverage, never selection. |
| D2 | Ported text keeps the pstack method and swaps each pstack tool (Cursor `/loop`, Graphite, bugbot, `poteto-agent`, arena, architect, how, why) for its bee counterpart (`bee dispatch prepare`, `bee worktree merge`, herding, `bee-how`, `bee-why`). A step with no bee counterpart is dropped and named in the port notes. Store id `063a21e4`. | Playbooks vendor into every host repo; a step naming an absent tool breaks silently (decision `8fd45bbb`). |
| D3 | Where a pstack playbook overlaps a bee class playbook (bug-fix/bugfix, feature, refactoring/refactor, perf-issue/perf, prototype/spike, investigation/research), the two merge into one file per situation: the pstack method is the body, and bee's enforced lines (proof floor, routed principles, statechart and formal-verify steps) stay in it. Store id `5a1fc656`. | One home per situation. |
| D4 | Ported playbooks that land work (shipping, autopilot-full, autopilot-stack, babysit, autonomous-run, orchestrate) merge on their own: on green they run `bee worktree merge` without asking, as `uat_stop = "close"` already does. The `uat` acceptance at `bee close` stays the owner's. Store id `b749c98d`. | uat is never auto-approved at any bypass level. |
| D5 | Every pstack skill and principle the ported playbooks or the poteto-mode router cite, and that bee has no counterpart for, comes into bee too: the skills architect, arena, interrogate, show-me-your-work, swarm (parallel fan-out), tdd, reflect, recall, blast-radius, the no-comments review and figure-it-out; the comment-sicko agent; and each pstack principle no bee principle covers (at least foundational-thinking, guard-the-context-window, laziness-protocol, minimize-reader-load, model-the-domain, prove-it-works, redesign-from-first-principles). A skill or principle bee already covers (how, why, teach, unslop, technical-writing, the verification skills, the principles already ported) stays bee's, and any pstack improvement merges into it the way D3 merges playbooks. Store id `3ac421ec`. | Owner: bring everything, related skills included. A ported step that cites a skill the host lacks is a broken step. |

### Agent's Discretion

- How a situation is recorded and selected (a field beside `class` on the route
  record, a `bee orient` flag, file layout under `skills/bee-planning/playbooks/`),
  bounded by D1–D3 and by the class parity fence staying two-directional.
- Which class each classless pstack playbook (session pickup, pause safely,
  autonomous run, orchestrate, babysit, shipping, worktree cleanup, opening a PR,
  authoring a skill, eval, multi-phase plan) sits under, or whether they form a
  session group outside the nine classes, bounded by D1.
- Trigger phrases per situation, in English and Vietnamese.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Situation | One named procedure under a route class: trigger phrases, numbered steps, deliverable, proof line. A class holds one or more. |
| Fallback | The figure-it-out procedure: the leader designs a playbook (falsifiable done predicate, phases, decision trail) when no situation fits or the work is large and cross-cutting. |
| Port notes | The record, per ported playbook, of which pstack tool was swapped for which bee counterpart and which step was dropped. |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/doctrine-layer/per-task-route-tester-pairing-and-proof-floors.md | unverified | Per-task routing, the Principles block in worker prompts and the per-class proof floor the situations must keep |
| docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md | suspect | The `## Principle homes` index, the one source of routed principles |
| docs/knowledge/areas/workflow-state/gates.md | suspect | Phase-owned routing and feature start |

The pack for this slug is in the knowledge holdout arm (empty); rows come from
`bee knowledge search --text "class playbook route class situation"`.

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs` | `class_packet` maps a class to `skills/bee-planning/playbooks/<class>.md` and its principles |
| `packages/bee-rs/crates/bee/src/verbs/plan_group.rs` | `playbook_findings`: `bee plan check` requires the plan to cite the class playbook |
| `packages/bee-rs/crates/bee/src/principles.rs` | Principle selection by route class |
| `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` | `cell_principles`: the Principles block attached to a worker prompt |
| `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs` | `ROUTE_CLASS_VALUES`, the nine classes |
| `packages/bee-rs/crates/bee/tests/class_playbook_parity.rs` | One playbook file per class, both directions |
| `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` | Cap principle answer and proof floor |

## Specific Ideas And References

- Source, MIT, copyright 2026 Lauren Tan: `/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/` (`SKILL.md` § Playbooks, `playbooks/*.md`) and `../figure-it-out/SKILL.md`, commit `b9ddc83`. The MIT notice travels with the ported text.
- `docs/history/research/pstack-playbooks-gap-check.md` — the earlier per-playbook verdict table; its bee-side anchors are where the overlapping playbooks merge (D3).
- `docs/history/research/pstack-craft-study.md` § L2 — the method elements bee lacked.

## Existing Code Context

### Reusable Assets

- `skills/bee-planning/playbooks/*.md` — the nine class playbooks D3 merges into.
- `bee orient --class` — the existing per-task selection door.

### Integration Points

- `bee dev release-manifest` walks `skills/**`, so new playbook files vendor to hosts with no manifest change (decision `8a5f5a72`).
- `.bee/LEADER.md` § Route each task and § Trigger index — where a leader learns to pick a situation.

<!-- bee:not-a-deferral: a record or description of work already done, never a promise to act -->
## Deferred Ideas
<!-- /bee:not-a-deferral -->

- pstack skills the playbooks do not cite stay out: setup-pstack (bee team config already picks models per role), typescript-best-practices (one language), make-bot-ui, automate-me, bro — none is related to the playbook set.

## Outstanding Questions

### Resolve Before Planning

None. The principle question resolved as D5.

<!-- bee:not-a-deferral: a record or description of work already done, never a promise to act -->
### Deferred To Planning
<!-- /bee:not-a-deferral -->

- [ ] The selection shape — route record field, `bee orient` output, plan-check citation, parity fence over situations.
- [ ] Whether principles filter per situation or stay per class.
- [ ] Which pstack principle maps to which existing bee principle (fix-root-causes, type-system-discipline, sequence-verifiable-units and the rest), and so which ones are new.
- [ ] A bee name for pstack `swarm` that does not collide with `bee-swarming`.

## Handoff Note

<!-- bee:not-a-deferral: a record or description of work already done, never a promise to act -->
CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, the read set, code context, canonical references, and deferred-to-planning questions.
<!-- /bee:not-a-deferral -->
