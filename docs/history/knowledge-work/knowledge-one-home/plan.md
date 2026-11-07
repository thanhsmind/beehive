---
type: bee.plan
title: knowledge-one-home — Plan
description: "The four-phase standard-lane plan: frontmatter schema and check codes, ownership maps, the refusing cap door, the plan-time conflict gate, and the migration of the twelve inventoried rules."
tags: [knowledge, plan, standard]
timestamp: 2026-08-22
bee:
  id: knowledge-one-home-plan
  lifecycle: active
  areas: [okf-profile, workflow-state, decision-memory]
  required_context: [work/knowledge-one-home/work-item.md]
  decisions: [D1, D2, D3, D4, D5]
  sources: [docs/history/knowledge-one-home/plan.md, docs/history/knowledge-one-home/CONTEXT.md]
  lane: standard
  review_status: Shipped
---

# knowledge-one-home — Plan

## Mode gate

Standard lane, three risk flags: a covered contract change (two new
refusals alter behaviour that existing tests already assert), public
contracts (the frontmatter schema and the CLI output), and multi-domain
reach (knowledge, cells, gate, decisions, skills). Two new refusing doors
in bee's own control plane need a frozen shape and a review wave;
anything smaller would let the doors drift the way the rules they guard
had already drifted.

## Approach

Four demoable phases, each carrying exactly one door, in an order where
no phase can start before the data it grades exists. Phase 1 makes the
data exist and gradeable. Phase 2 puts the cap door on it. Phase 3 puts
the gate door on it. Phase 4 migrates the twelve inventoried rules and
the stale help text through both doors, which is their first real use.

Values planning fixed inside the agent's discretion the locked decisions
granted:

- Four flat keys under the `bee:` block — `owns.code`, `owns.skills`,
  `owns.tests` on an area's overview concept only, and `applied_at` on any
  concept that homes a rule. Flat, because the parser accepts no nested
  map other than `bee:` itself; all four join the fixed emit order so the
  parse-emit round trip stays byte-stable.
- An explicit marker pair in the home body carries the rule id, and a copy
  is any line elsewhere carrying the pointer form. No prose heuristic: an
  unmarked block is never guessed at.
- Six new check codes, graded as profile **errors**, not warnings — a
  backstop that never blocks is not a backstop.
- Cap-time diff source: the commit's own numstat union the declared file
  list, falling back to the declared list alone when no commit resolves.
- Conflict verdicts stored on the workflow record beside the plan
  revision, so a revision bump makes the recorded review stale by itself
  and needs no separate reset step.

## Slices

| Phase | What changes | Demo | Cells |
|---|---|---|---|
| 1 Schema and map | The four keys parse, emit, and grade; rule markers and references grade; six new check codes; ownership maps for all fifteen areas; the first three inventoried rules homed | `knowledge check` reports the new codes on the live bundle; a deliberately duplicated id is flagged | koh-1, koh-2, koh-3, koh-4 |
| 2 Cap door | Prediction fields on every cell; `update_obligations` on `decisions log`; the refusing sync door at cap and finish with its recorded escape | A cap that touched owned code without the owned skill is refused naming the skill; the same cap with a reason is capped and the reason recorded | koh-5, koh-6, koh-7 |
| 3 Gate door | Conflict candidates derived at plan time, one verdict each, and the merged gate's precondition | The gate is refused with the unverdicted ids named; after verdicts it approves; after a plan-revision bump it refuses again | koh-8, koh-9 |
| 4 Migration | The remaining inventoried rules homed with ids and outbound lists, every copy reduced to one line plus a pointer, the retired help-text claim removed, the capture stub's skill answer required | The bundle check stays clean while twelve rules move | koh-10, koh-11, koh-12 |

## Risk map

| Component | Risk | Proof taken |
|---|---|---|
| Frontmatter parser and profile table | Low | Knowledge suite green; bundle check clean |
| Ownership map content for fifteen areas | Medium — a wrong glob is a false refusal | Zero missing-map and zero dangling-path findings; no false refusal on live caps |
| Cap door | High — every cap passes through it | Cells suite green with new refuse, escape, and no-commit cases; existing cap cases unchanged |
| Gate precondition | Medium | Gate suite green with absent, stale, unverdicted, and acknowledged-conflict cases |
| Migration of twelve rules | Medium — prose churn | Bundle check clean; every copy resolves to a homed id |

## Rejected Alternatives

- **Warn-only at cap** — rejected by the locked decision in its own words.
  Soft reminders are the failure this work exists to end.
- **A separate ownership document** — two homes for ownership, which is
  the defect being repaired.
- **A prose heuristic for recognising a rule block** — it re-creates the
  guessing the whole work removes.
- **Generating the help payload** — no generator exists in this repo; the
  payload stays hand-edited with its drift test as the net.
