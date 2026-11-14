---
artifact_contract: bee-research/v1
topic: poteto-mode-router
depth: standard
date: 2026-11-14
---

## Bottom Line

- Recommendation (ladder rung): adapt-upstream. Keep bee's four layers. Move
  the routing step from "once per feature" to "once per task".
- Why this is the lightest credible path: bee already has all four layers of
  poteto-mode. It has a route class, 9 class playbooks, 24 `bee-principle-*`
  skills, and the rule that a reply names each principle it applied. What
  differs is WHEN the router runs. poteto-mode routes every prompt. bee routes
  a feature once, at shaping or planning, and reads the playbook only inside
  bee-planning.
- Why the next-best rung lost: reuse does not fix the stale route. This
  session proves it: the preamble named the `bugfix` principles of feature
  `windows-test-build` for a research task. build is not needed: no new
  store, no new layer.
- Confidence: 75%. The bee side is read this session. The pstack side is the
  user's summary plus the pinned reads in `pstack-xia.md` and
  `pstack-playbooks-gap-check.md` (commit `b9ddc83`). The cross-project guard
  blocked a fresh read of `/mnt/data/Projects/refs/pstack/pstack`.
- Suggested next step: bee-shaping for feature `per-task-router`.

## Repo Snapshot

- bee: Rust CLI (`packages/bee-rs`), doctrine in `.bee/LEADER.md`,
  `AGENTS.md`, `skills/bee-*`.
- Route record: `bee route --set --class --lane --flags --files`, one per
  lane record.
- Playbooks: `skills/bee-planning/playbooks/<class>.md`, 9 files.
- Principles: 24 `bee-principle-*` skills, mapped to classes in
  `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`
  § "Principle homes".

## Question & Assumptions

- What was asked: study poteto-mode's router (goal, playbook, skill,
  principle) and find how to bring it into the beehive leader and peers.
- Settles it: a layer-by-layer map with EXISTS / NEW / CONFLICT and the
  smallest change that closes the real gaps.
- Assumption to confirm: the user's summary (24 principles, guide pages 02
  and 08) matches a pstack commit newer than `b9ddc83`.

## Findings

### Local

| poteto-mode layer | bee today | Verdict | Anchor |
|---|---|---|---|
| Restate the goal first | Goal playback only in shaping interview craft | EXISTS, narrow | `pstack-0-15-port.md:71` |
| Principles index, "when to apply" | `bee orient` names principles for the route class | EXISTS | `router-triage-...md` § Principle homes |
| Name principle and decision in reply | rule `agents-name-applied-principles` | EXISTS | `.bee/LEADER.md` § Judgment and deviation |
| Match task to one playbook | Route class picks a playbook, at planning | EXISTS, wrong time | `planning-reference.md:229-265` |
| Router runs per prompt | Route runs per feature; a later task inherits it | NEW | orient output this session: `class=bugfix` for a research task |
| Playbook steps become the first todo items, skips kept with reason | Plan cites the playbook by path; skip stays visible with reason; todo list is a projection of cells | CONFLICT, small | `planning-reference.md:249-257`, `routing-and-contracts.md:209-211` |
| Non-negotiables: "see X, run skill Y" | Each skill's own description; no short trigger index in the always-loaded block | NEW, small | `.bee/LEADER.md` (no trigger table) |
| Autonomy: reversible acts, irreversible asks | `LEADER.md` § "Act. Don't ask." plus gates | EXISTS, stronger | `LEADER.md` |
| Model by role, fresh subagent per job | `bee dispatch prepare` picks the role and model | EXISTS, stronger | `.bee/LEADER.md` § Work in parallel |
| Worker re-reads the mode | Worker never reads LEADER.md; the brief carries the Expertise and Learned context blocks | EXISTS, different form | `bee dispatch prepare --help` |
| Claim labels: measured / inferred / guessed | `[verified]` / `[unverified]` | EXISTS | rule `agents-verify-before-agreeing` |
| Opening a PR at the end | draft PR on first cap, `uat` door is acceptance | Deliberate non-match | `pstack-playbooks-gap-check.md` row 22 |

### Upstream

- poteto-mode routes each prompt: restate goal, read principle index, match
  one playbook (fallback `figure-it-out`), copy its steps into the todo list.
  Source: the user's summary; layout matches the pinned read in
  `pstack-xia.md`.
- Earlier ports already took the playbooks (23 rows, `pstack-playbooks-gap-check.md`),
  the principles (`pstack-principles`, `expertise-principles`) and the craft
  surface (`pstack-craft-study.md`).

### Inference

- The gain the user wants is real but narrow. bee's layers are not missing;
  bee runs its router at the wrong grain. A leader turn for research, a
  close, or a release gets the principles and playbook of whatever feature
  the lane holds.
- Three changes close it:
  1. Per-task route. `bee orient --task "<goal>"` (or a turn-start hook)
     classifies THIS task, not the lane. It prints the class playbook path
     and that class's principles. The lane route stays for feature work.
  2. Playbook steps as the session's first todo items. This fits the
     projection rule if the todo items are a projection of the playbook
     file, not new work units. Skipped steps stay with `skip: <reason>`.
  3. A short trigger index in `.bee/LEADER.md`: about 8 lines of "see X,
     load skill Y" (how-question to bee-how, why-question to bee-why, port
     request to bee-researching, UI to bee-design, prose to bee-unslop).
- `figure-it-out` has no bee match. A task that fits no class today falls to
  `feature`. A fallback class that writes its own step list is a candidate,
  not a need yet.

### Peers (execution workers)

| poteto-mode | bee worker today | Verdict | Anchor |
|---|---|---|---|
| Subagent re-reads the whole mode before work | Worker loads `bee-swarming` "Execute" only, never LEADER.md, by design (cold pickup) | Deliberate non-match | `.claude/agents/bee-worker.md`, `AGENTS.md` |
| Subagent sees the principles index | Principles reach the leader only: `principle_lines` has two callers, the preamble and `bee orient` | NEW | `principles.rs:113`, callers `session_preamble/budget.rs:679`, `status_full/orient.rs:637` |
| Playbook step drives the subagent | Playbook steps stay out of the worker prompt on purpose | Deliberate non-match | `skills/bee-planning/playbooks/bugfix.md` step 2 |
| Skill per step | Optional `--expertise` list, picked by hand by the leader, "never auto-derived" | EXISTS, manual | `skills/bee-swarming/SKILL.md:65` |
| Knowledge before work | Learned context block, auto, with a 10% holdout | EXISTS, stronger | `packages/bee/prompts/worker-cell.md` |
| Reply names principles used | Cap records proof and mistakes, no principle line | NEW | `bee cells cap` |

Inference: the worker gap is that principles never reach it unless the
leader adds them by hand. `bee dispatch prepare --kind cell` can call
`principle_lines` for the cell's route class and render a Principles block,
names and one-line rules only, with the same holdout pattern as Learned
context so the gain can be measured. The cap can take
`--principle <slug>:<decision>` so the worker names what it applied.

## Risks, Unknowns, Follow-Ups

- Risk: a per-task classifier that guesses wrong is worse than a stale one
  that is easy to see. Keep it a suggestion the leader confirms in the
  Triage line, not a gate.
- Risk: a trigger index in LEADER.md copies skill descriptions. It must
  point, never restate (duplication boundary).
- Unknown: pstack content newer than `b9ddc83`. A re-diff needs
  `/mnt/data/Projects/refs/pstack` in `guards.cross_project_allow`.
- Open questions for shaping: lane route vs task route when both exist;
  whether the Triage line names the class too.

## Source Pack

- Local: `.bee/LEADER.md`, `LEADER.md`,
  `skills/bee-planning/references/planning-reference.md`,
  `skills/bee-planning/playbooks/bugfix.md`,
  `skills/bee-hive/references/routing-and-contracts.md:209-211`,
  `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`,
  `docs/history/research/pstack-playbooks-gap-check.md`,
  `docs/history/research/pstack-xia.md`,
  `docs/history/research/pstack-0-15-port.md`, `bee orient` output.
- Upstream: the user's poteto-mode summary (this conversation). Fresh read
  blocked by the cross-project guard.
