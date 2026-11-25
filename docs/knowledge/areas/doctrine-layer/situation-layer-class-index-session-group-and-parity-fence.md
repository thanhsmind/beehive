---
type: bee.area
title: "Doctrine Layer — situation layer, class index grammar, session group and the parity fence"
description: "How a leader routes to a situation under a route class: the `## Situations` line grammar a class index must keep, the one reader and its compiled-in fallback, the orient situation lines, the plan check citation rule, the session group that binds no class, the figure-it-out fallback and the fence that keeps index and files in step."
tags: [doctrine-layer, routing, situations, playbooks, parity-fence, pstack]
timestamp: 2026-11-15
bee:
  id: doctrine-layer-situation-layer-class-index-session-group-and-parity-fence
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/per-task-route-tester-pairing-and-proof-floors.md]
  decisions: ["pstack-playbook-port D1-D5 (docs/history/pstack-playbook-port/CONTEXT.md, 2026-11-15)"]
  sources: [docs/history/pstack-playbook-port/CONTEXT.md, docs/history/pstack-playbook-port/plan.md, docs/history/pstack-playbook-port/port-notes.md, docs/history/pstack-playbook-port/delivery.md]
  authoritative_for: "doctrine-layer: situations under a route class, the session group and the fallback skill"
  anchors: [packages/bee-rs/crates/bee/src/situations.rs, packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs, packages/bee-rs/crates/bee/src/verbs/plan_group.rs, packages/bee-rs/crates/bee/tests/class_playbook_parity.rs, packages/bee/LEADER.md, skills/bee-planning/playbooks/session/index.md, skills/bee-planning/playbooks/bugfix.md, skills/bee-figure-it-out/SKILL.md, skills/bee-architect/SKILL.md, skills/bee-arena/SKILL.md, skills/bee-interrogate/SKILL.md, skills/bee-show-me-your-work/SKILL.md, skills/bee-fanout/SKILL.md, skills/bee-tdd/SKILL.md, skills/bee-reflect/SKILL.md, skills/bee-recall/SKILL.md, skills/bee-blast-radius/SKILL.md, skills/bee-comment-review/SKILL.md, skills/bee-principle-foundational-thinking/SKILL.md, skills/bee-principle-redesign-from-first-principles/SKILL.md, skills/bee-principle-outcome-oriented-execution/SKILL.md, skills/bee-principle-laziness-protocol/SKILL.md, skills/bee-principle-minimize-reader-load/SKILL.md, skills/bee-principle-model-the-domain/SKILL.md, skills/bee-principle-prove-it-works/SKILL.md, skills/bee-principle-sequence-verifiable-units/SKILL.md, skills/bee-principle-make-operations-idempotent/SKILL.md, skills/bee-principle-guard-the-context-window/SKILL.md, skills/bee-principle-never-block-on-the-human/SKILL.md, skills/bee-principle-exhaust-the-design-space/SKILL.md, skills/bee-principle-experience-first/SKILL.md]
  verified_at: 2026-11-25
  verified_commit: 63d270af0980f908cbd5a43847aa4b14253b5530
---

# Doctrine Layer — situation layer, class index grammar, session group and the parity fence

Read when: `bee orient --class` prints `situation:` lines or none, `bee plan check` says plan.md cites no situation of a class, the parity test names a situation file or index line, or you add a situation, a session procedure or a pstack-derived skill or principle.

## Purpose

Nine route classes were too coarse: a leader who knew the task was a bugfix still had to guess which procedure inside it fit. The pstack playbook set (23 playbooks) comes in as situations under the classes, so the router picks a situation, not only a class (D1). The class stays the key for principles and proof floors; the situation carries the steps. Where a pstack playbook overlapped a bee class playbook, the two merged into one file per situation (D3).

## Behaviors & Operations

**Layout.** A class index is `skills/bee-planning/playbooks/<class>.md`; its situations are `playbooks/<class>/<name>.md`. The index lists them under `## Situations`, one line each: a backticked lowercase name, an em dash, trigger phrases in English and Vietnamese, an em dash, `delivers:` and the deliverable. Triggers sit in the index so the leader matches a task without opening every file.

**The one reader.** `situations.rs` parses that section and nothing else. A line off the grammar is skipped, never an error, so a typo hides a situation instead of breaking routing; the fence below is what catches it. The reader takes the repo class index when it can read it and answers from the compiled-in copy of that one class index when the file cannot be read (`class_situations` in `situations.rs`). A repo index that reads but has no `## Situations` section yields no situations; it never falls back. The compiled-in copy is the index text itself, so it moves only on a rebuild; a host that vendors newer playbooks wins through the repo file.

**Orient.** `bee orient --class <c>` adds a `situations` array to its JSON, each row carrying its resolved `path`, and one `situation: <path> — <triggers>` text line each, ahead of the principles. A class with no lines has one playbook file. One resolver takes, for each playbook or situation file, the first of `skills/`, `.claude/skills/`, `.agents/skills/` and `.opencode/skills/` where that file exists, so `--json` and text agree (orient-host-playbook D1); picking the first `bee-planning` directory instead printed missing files when a host copy held only some of them. When no root has the file the path values name the skill and say to load it with `skill_get` and open the file under its `local.path`. The text ends with one `next:` line: open the situation whose triggers match and build the todo list from its numbered steps, or the playbook file when the class has no situations.

**Plan check.** When the routed class lists situations, plan.md must cite one `playbooks/<class>/<name>.md`; the bare class index no longer passes and the finding lists the paths that would. A class with none keeps the index rule. With no class known, any `playbooks/<class>/<name>.md` counts. A cited name that is not listed does not count.

**The session group.** Pick up, pause, run until done, orchestrate, babysit, ship, clean worktrees, the two autopilot runs (full queue and stack) and open a PR belong to a session, not a feature route. They live in `playbooks/session/` with their own `index.md` of the same grammar, bind no class, and are not read by `situations.rs`. Nothing routes a leader there: LEADER's route step names only the nine classes and `bee orient --class <c>`, so a leader opens `skills/bee-planning/playbooks/session/index.md` by name. Those that land work merge on green with `bee worktree merge`; the owner's `uat` acceptance at `bee close` stays the owner's (D4).

**The fallback.** `bee-figure-it-out` is the answer when no situation fits or the work is large and cross-cutting. Every class index carries a line naming it under its situations, just above the proof-floor line, so the fallback is in front of the leader at the moment a situation fails to match.

**Skills and principles the situations name (D5).** Eleven skills bee had no counterpart for: `bee-architect`, `bee-arena`, `bee-interrogate`, `bee-show-me-your-work`, `bee-fanout` (pstack swarm; renamed to avoid `bee-swarming`), `bee-tdd`, `bee-reflect`, `bee-recall`, `bee-blast-radius`, `bee-comment-review` and `bee-figure-it-out`. Thirteen principles: foundational-thinking, redesign-from-first-principles, outcome-oriented-execution, laziness-protocol, minimize-reader-load, model-the-domain, prove-it-works, sequence-verifiable-units, make-operations-idempotent, guard-the-context-window, never-block-on-the-human, exhaust-the-design-space and experience-first. Each principle's classes and one-line rule are rows in the router concept's `## Principle homes`; find them there, not here. Where bee already covered a pstack skill or principle, the pstack improvements merged into the bee one rather than a second copy.

## Business Rules

- The index is the only place triggers and deliverables live; the situation file is the only place its steps live; LEADER's trigger index points to the skill and never restates either.
- Ported text swaps every pstack tool for its bee counterpart or drops the step and names it in `port-notes.md`; no file under `skills/` names a pstack-only tool (D2).
- The pstack MIT notice ships as `LICENSE.pstack` beside the playbooks.

## The parity fence

`class_playbook_parity.rs` holds, in both directions: every route class has exactly one index file and every index names a real class; every `playbooks/<class>/` file is listed in its index and every listed name has a file; a malformed situation line fails; a subdirectory that is neither a class nor `session` fails; `session/` obeys the same two-way rule against its own index. A clean sandbox without `session/` passes. Without the fence a one-sided change would drop a situation from routing silently, because the reader skips what it cannot parse.

## Rejected options

- A situation field on the route record with a write-time file check (rejected in `docs/history/pstack-playbook-port/plan.md` § Approach after the hat wave): it refuses in every host where the playbook tree is absent and grows orient's argv patterns; the plan took no new flag and no record change.
- Docs-only selection: the plan check could then hold only the class menu, so the steps stop being enforced.
- One route class per pstack playbook: breaks the proof-floor table, the principle `classes:` filter and two parity fences for 23 values.
- Situations as sections inside the class file: the router could not name a file and the plan check could not hold one.
- Failing orient or plan check on a malformed index line (a design choice of the reader, not recorded as decided): the reader skips it and the fence reports it, so a typo breaks CI, not routing.

## Pointers

- Per-task route, tester pairing, proof floors and the `bee orient --class` door: `areas/doctrine-layer/per-task-route-tester-pairing-and-proof-floors.md`.
- Principle homes and the duplication boundary: `areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`.
- Per-playbook tool swaps and dropped steps: `docs/history/pstack-playbook-port/port-notes.md`.
- What shipped per slice: `docs/history/pstack-playbook-port/delivery.md`.
