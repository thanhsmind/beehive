---
artifact_contract: bee-research/v1
topic: ak-brainstorm-distill
depth: standard
date: 2026-08-18
---

## Bottom Line

- Recommendation (ladder rung): reuse — bee already owns ~90% of
  ak-brainstorm's substance; distill at most two one-line rules into
  existing docs, no new skill, no protocol change.
- Why this is the lightest credible path: every load-bearing idea in
  ak-brainstorm maps to a shipped bee mechanism (matrix below); the two
  genuine nuggets are single sentences, not structures.
- Why the next-best rung lost: adapt-upstream (porting the 4-field
  contract as a new artifact) would duplicate CONTEXT.md/the tiny brief
  and add a second decision record — the exact drift bee's single-writer
  Lock exists to prevent.
- Confidence (0–100%): 85
- Suggested next step: none (discussion first; a tiny docs cell if the
  user wants the two nuggets merged)

## Source Manifest (xia)

| Field | Value |
|---|---|
| Repo or path | /home/thanhsmind/projects/AI/ak/.claude/skills/ak-brainstorm |
| Ref | HEAD |
| Resolved commit SHA | a70c7cdb |
| Narrowed scope | SKILL.md (124 lines, v2.3.0) — the skill's only file |

## Question & Assumptions

- What was asked: xia (distill) ak-brainstorm — strengths, weaknesses,
  what bee already has, what is worth bringing back.
- What success appears to mean: a discussion-ready verdict, building
  nothing.
- Assumptions still needing confirmation: none material.

## Findings

### Local (bee side)

Dependency matrix — source component → bee equivalent:

| Source component | Bee equivalent | Verdict | Label |
|---|---|---|---|
| 4-field contract (outcome, constraints, non-goals, acceptance) | CONTEXT.md sections (Feature Boundary, Locked Decisions, Out of scope) + cell `verify`/must-haves; tiny/docs short brief (asked/found/will-do) | EXISTS | Local |
| "Accepted contract exists → reuse, don't re-ask" | Lock consumes map D-IDs (D8); settled rules cited, never re-asked | EXISTS | Local |
| Proportional behavior (concrete request → no interview) | bee-shaping triage table (clear / partially clear / vague) | EXISTS | Local |
| Direct answers need no design loop | "A question is a question" (AGENTS.md); docs lane | EXISTS | Local |
| Bug routing: no fixes from the symptom; diagnose, then compare cause-aligned options | Qualify's reproduce-before-verdict; `.bee/expertise/debugging.md`; knowledge search on symptom | PARTIAL — repro-first exists; "options only after diagnosis" is implicit, never stated | Local |
| Option exploration: ≤3 options, recommend smallest, resolve disagreement first | Interview craft (numbered questions each carrying a recommended answer); SMALLER PATH check | EXISTS | Local |
| YAGNI/KISS/DRY ordering | Smallest honest shape + SMALLER PATH | EXISTS (unnamed) | Local |
| "No report merely to satisfy the gate" | AGENTS.md: never author an artifact whose only purpose is proof | EXISTS | Local |
| Mermaid authoritative-flow diagram inside the skill | No bee skill carries a routing diagram (area specs do) | NEW — but collides with bee's prompt-diet direction | Local |

Cross-cutting sweep: ak-brainstorm is wired into 9 sibling skills
(ak-fix, ak-plan, ak-cook, ak-debug routing tables) and a `brainstormer`
agent; the wiring is handoff-only — no hidden middleware. bee's
equivalent wiring is bee-hive's route table, already in place. [Local]

### Upstream

- ak-xia's own guard ("do not invoke ak:brainstorm from inside xia —
  phase ownership") mirrors bee's one-skill-owns-the-phase rule; nothing
  to import. [Upstream]

### Docs

- Not applicable — single-file local source; no external docs consulted.

### Inference

- The two nuggets worth keeping are sentences, not mechanisms:
  1. **"Compare cause-aligned solutions only after diagnosis"** — one
     line for `.bee/expertise/debugging.md`; blocks symptom-driven fix
     menus.
  2. **Acceptance evidence named up front**: the tiny/small brief could
     state the intended proof line before work starts (bee records proof
     at cap; naming it in the brief is the up-front half). One line in
     the shaping brief rule, if wanted.
- The mermaid-flow idea reads well but fights bee's prompt-diet lanes;
  bee already mandates diagrams where they pay (flow-shaped area specs).

## Strengths / Weaknesses (distill)

**Strengths (hay):** proportionality is stated crisply; the 4-field
contract is a good minimal shape for repos with no bee; bug path refuses
symptom-driven fixes; the flow diagram makes routing legible to a cold
reader.

**Weaknesses (dở):** no decision persistence (contract lives in chat;
"durable summary only when needed"), no D-IDs, no supersession — the
drift bee's decision log exists to kill; no fog path (assumes a nameable
outcome; bee-wayfinding owns that ground); "autonomous execution may
continue" without a recorded gate — weaker human control than bee's
gates; no evidence labels; one monolithic skill mixing triage, bug
craft, and option craft.

## Risks, Unknowns, Follow-Ups

- None blocking. Open question for the user: merge the two one-line
  nuggets (debugging expertise + brief rule), or take nothing?

## Source Pack

- Local files read: skills/bee-wayfinding/SKILL.md,
  skills/bee-shaping/SKILL.md, skills/bee-planning/SKILL.md (all read
  earlier this session), AGENTS.md
- Upstream: /home/thanhsmind/projects/AI/ak/.claude/skills/ak-brainstorm/SKILL.md
  @ a70c7cdb; sibling wiring via rg over .claude/skills
- Docs pages checked: none (not applicable)
