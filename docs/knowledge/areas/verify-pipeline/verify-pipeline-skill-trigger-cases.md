---
type: bee.area
title: Verify Pipeline — skill trigger cases
description: "The gate that gives every routable skill a set of user briefs that must open it and near misses that must not, judged without a model inside the declared suite, plus the opt-in eval that asks a real agent the same briefs."
tags: [verify-pipeline, skills, guards]
timestamp: 2026-09-22
bee:
  id: verify-pipeline-verify-pipeline-skill-trigger-cases
  lifecycle: active
  areas: [verify-pipeline]
  decisions: ["skill-trigger-cases D2+D3+D4 (39b02fdb): one fixture of briefs with expect and near cases, and a brief never names its skill", "skill-trigger-cases D5+D6+D7 (516b603e): a model-free fence runs in the declared suite and an opt-in eval asks a real agent", "skill-description-trim D1-D3 (docs/history/skill-description-trim/CONTEXT.md): every routable description fits 300 characters and the eval must not fall below its baseline"]
  anchors: [skills/, packages/bee-rs/crates/bee/tests/skill_triggers.rs, packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json, skills/bee-writing-skills/SKILL.md]
  verified_at: 2026-09-23
  verified_commit: 00330b47416a8a64323e15cd6c1d286e3029bcee
---

## Purpose

A skill's description is the text an agent reads to decide whether to open it. Nothing tested that text: a description could drift until it opened on the wrong asks or missed the right ones, and every check stayed green. This gate makes the routing surface testable — each routable skill carries a set of user briefs that must open it and look-alike briefs that must not — so a description change is judged by cases, and a new skill cannot ship without them.

Adapted from the Seatworks plugin's skill-trigger tests (`docs/history/research/seatworks-xia.md`, item A3), which established that the description line is the tested surface.

## Entry Points & Triggers

- The declared test suite runs the fence on every push, as one integration test among the others.
- A skill author adds cases when creating or renaming a skill; the writing-skills checklist names the fixture and the rule.
- A maintainer runs the paid eval by hand, with an agent command in one environment variable, to measure how often a real agent routes each brief right. Unset, the eval is skipped and says so.

## Data Dictionary

| Term | Meaning |
|---|---|
| routable skill | A bee skill whose description is how an agent finds it: every `bee-*` skill except the principle skills, which the router selects from class and flags (skill-trigger-cases D1) |
| case | One user brief, the skills it must open (`expect`, empty for none), and at most one skill it must not open (`near`) (skill-trigger-cases D3) |
| brief | The request in the user's own words, English or Vietnamese; it never contains the slug or slash form of a skill it expects or nears (skill-trigger-cases D4) |
| near miss | A brief that sounds like a skill and must not open it; its `expect` says what opens instead |
| fixture | The one file holding every case, kept beside the tests and never shipped to hosts (skill-trigger-cases D2) |

## Behaviors & Operations

**The fence** (skill-trigger-cases D5) reads every routable skill's description in both spellings the front matter allows and the fixture, then fails by name when: a routable skill has fewer than three cases expecting it or fewer than two nearing it; a case names a skill that does not exist; a brief names a skill it expects or nears; two cases share a brief. It runs with no model, inside the declared suite.

**The eval** (skill-trigger-cases D6) is opt-in and never part of the suite or CI. It shows the agent the routable skills as `name: description` lines and one brief, asks for a JSON list of skills to open, runs each brief three times, and scores a run right when no near skill appears, an empty-expect case returns nothing, and otherwise at least one expected skill appears. A case passes at two of three; the eval prints one line per skill and a total, and fails only on a case that scored zero of three.

**The budget fence** (skill-description-trim D1) fails when a routable skill's folded description runs past 300 characters, counted in characters. One message names every offender with its length. It runs with no model, inside the declared suite. Every description loads into every session, so the budget caps a cost paid on every turn.

**The checklist row** (skill-trigger-cases D7) tells a skill author to add at least three opening briefs and two near misses for a new or renamed skill, and that the suite refuses a skill with none.

## Business Rules

- Cases live outside `skills/`, so host packaging — which copies the skill directories — never ships them (skill-trigger-cases D2).
- Principle skills are out of scope: their trigger is class-based routing, not description matching (skill-trigger-cases D1; decision 11221f5b).
- A description is one purpose clause, the trigger words its fixture briefs need, and a short "Not for" only where a near case needs it. `bee-capturing` keeps its self-trigger line and `bee-reviewing` keeps its never-automatic boundary (skill-description-trim D2).
- A description cut is judged by the paid eval against the score before the cut. The cut must not lower the total, and no brief may score zero (skill-description-trim D3).
- Near misses are real confusions between sibling skills (how/why/teach, researching/wayfinding, verifying/verify-upkeep, shaping/planning, capturing/evolving, herding/swarming/herdr, unslop/technical-writing), never random sentences.

## Edge Cases Settled

- A description written as a folded block and one written as a quoted line are both read; the fence joins continuation lines with one space.
- A brief that names its own skill is refused even when it would route correctly: it tests string matching, not routing (D4).
- The eval command is a shell string; an agent that prints prose around its JSON still scores, because the first JSON object in its output is what is read.

## Open Gaps

- Principle skills have no cases; a fixture keyed on class and flags, judged against the router's output, is filed as backlog.
- Descriptions the eval shows as confused are not rewritten here; that is a follow-up per skill once the eval has data, filed as backlog.
- The fence proves coverage and shape, never routing quality; only the paid eval measures that.
- With haiku, four briefs that expect no skill score zero both before and after the description cut: the bee release, "the slice is finished — merge it", the subsystem spec document, and the AGENTS.md convention. Haiku names some skill for each of them, sometimes one that does not exist. The total moves by about three briefs from run to run, so one run cannot judge a small change (skill-description-trim D3).
- The eval calls the agent once per brief and run, in series, and prints nothing until the end: about an hour per run. A parallel runner does the same work in about four minutes.

## Pointers (implementation)

- `packages/bee-rs/crates/bee/tests/skill_triggers.rs` — the fence (`skill_triggers_fixture_is_complete`), the budget fence (`skill_descriptions_fit_the_budget`, const `DESCRIPTION_BUDGET`) and the ignored eval (`skill_triggers_real_agent_eval`, env `BEE_SKILL_TRIGGER_EVAL`).
- `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json` — the cases (104 at first landing, 20 skills).
- `skills/bee-writing-skills/SKILL.md` — the checklist row, which names the 300-character budget.
- `docs/history/skill-trigger-cases/CONTEXT.md` — decisions skill-trigger-cases D1-D7; cells skt-1, skt-2 (capped 2026-09-23, merged at 350c126).
