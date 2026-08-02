# Implement-Plan Reference

Load when shaping's Brief move renders an implement plan. The template, the section→source map, and the writing guide live here. The template is present tense, projection-ruled, D-ID-cited, drop-empty, with no fabricated validation.

Path: `docs/history/<feature>/implement-plan.md`. One file per feature, chosen at first render, updated in place forever. Never `-v2`/`-new`/dated.

## Section → source map

Every section projects from a named source; source silent → Open Question, never a guess.

| Section | Source | Rule |
|---|---|---|
| Review Status | `.bee/state.json` gates | mirror gate state, never assert early |
| Goal / Success | `CONTEXT.md` decisions | user outcome, cite D-IDs |
| Current State | shaping scout + `approach.md` | what's inspected, how it behaves now |
| Scope | `CONTEXT.md` + `plan.md` Out of scope | deferred stays deferred |
| Approach + alternatives | `approach.md` or `plan.md`'s `## Approach` | as written, no substitutes |
| **Technical Design** | authored from `approach.md` + cells | narrative as implied; beyond → Open Question |
| Affected Files | `approach.md`, then cells after prep | cells authoritative post-prep |
| Implementation Steps | `plan.md` shape, then cells | project titles/deps after prep |
| Validation Plan | test-result records (`.bee/logs/test-results.json`) | describe + link evidence, never assert unrun |
| Risks & Mitigation | `approach.md` risk map | as written |
| **Rollback Plan** | authored | how *this* work reverts; undecided → Open Question |
| Open Questions | `approach.md` + uncovered gaps | honest home for every gap/guess |

The two **authored** sections are written from judgment — reading what the artifacts imply, never designing anew:

- **Technical Design** — the flow the approach implies: components, data shape, API/UI/security surface. A choice the artifacts don't contain is a proposal, not a rendering — it goes to Open Questions and flows back through planning, never smuggled in.
- **Rollback Plan** — how *this* change is undone (revert commits / disable a flag / reverse a migration). Undecided → "OPEN QUESTION: …", never a plausible procedure nobody agreed to. `high-risk` must resolve it before the gate.

## Full template (`standard` / `high-risk`)

Drop any section with no real content (except where the lane makes it mandatory). Do NOT keep an empty header with "N/A" — a dropped section reads as "not relevant"; an "N/A" header reads as ceremony.

````markdown
---
artifact_contract: bee-implement-plan/v1
feature: <feature-slug>
lane: small | standard | high-risk
status: Draft | Ready for Review | Approved | Needs Revision
updated: YYYY-MM-DD
sources: [CONTEXT.md, approach.md, plan.md, <report files>]
decisions: [<D-IDs cited below>]
---

# Implementation Plan: <Feature>

> Human-layer projection of the truth artifacts. Truth lives in CONTEXT.md
> (decisions), plan.md + cells (work), and test-result records (evidence).
> Feedback on this document flows back to those artifacts, then this re-renders.

## 1. Goal

<The user-facing outcome, in the user's terms. One short paragraph.>

**Success looks like**
- <criterion — from a locked decision, cite D-ID>

## 2. Current State

<What exploring/planning inspected and how the area behaves today. Facts only —
what the code does now, not what it will do.>

## 3. Scope

**In scope**
- <what changes — from CONTEXT.md decisions / plan.md, cite D-IDs>

**Out of scope**
- <explicitly not solved; deferred ideas stay deferred>

## 4. Proposed Approach

<The chosen path from approach.md, in plain language. Cite D-IDs.>

**Why this approach** — <the recorded reason>
**Alternatives considered** — <from approach.md's rejected list; one line each>

## 5. Technical Design   (AUTHORED — see the section→source map above)

<A readable narrative of the flow the approach produces: components touched,
data shape, API/UI/security surface THE APPROACH ALREADY IMPLIES. A design
choice the artifacts do not contain is a proposal → put it in Open Questions,
do not state it here as the plan. Use a small flow sketch when it helps:>

```text
<actor/trigger> -> <component> -> <component> -> <observable result>
```

Subsections, included only when the work touches them:
- **Data model** — new/changed stored elements; migration needed?; backward compatibility
- **API / contract** — endpoint/payload/error changes
- **UI / UX** — screens, states (empty/loading/error)
- **Security / Permissions** — auth, authorization, sensitive data, abuse limits
  *(mandatory in the `high-risk` lane)*

## 6. Affected Files

<Projected from cell `files` after prep (authoritative); from approach.md before prep.>

| Action | File / Component | Purpose |
|--------|------------------|---------|
| Modify | `<path>` | <why> |
| Create | `<path>` | <why> |

## 7. Implementation Steps

<Projected from the cells (titles + deps) after prep; from plan.md shape before.
Phase headers only if plan.md is phase-shaped — never invent phases.>

- [ ] <cell title> (`<cell-id>`)

## 8. Validation Plan

<Describe what WILL be checked. Link evidence from the test-result record
(`.bee/logs/test-results.json`) once it exists. NEVER state a result that has not run.>

**Automated** — `<the declared test command (commands.test)>` → expected: <target outcome>
**Manual** — [ ] <check for SEE/CALL/RUN decisions>
**Evidence** — <link to docs/history/<feature>/reports/… once the declared tests run; "pending" before>

## 9. Risks & Mitigation

<From approach.md's risk map.>

| Risk | Impact | Mitigation |
|------|--------|------------|
| <risk> | High/Med/Low | <mitigation> |

## 10. Rollback Plan   (AUTHORED — see the section→source map above)

<How THIS change is undone: revert the cells' commits / disable a flag / reverse
a migration. If genuinely undecided: "Not yet decided — OPEN QUESTION: …".
Mandatory real content in the `high-risk` lane.>

## 11. Open Questions

<The honest home for every gap the sources did not cover and every guess.
If none: "No blocking open questions. Ready for review.">
````

## Writing guide (bee-specific; deduped against what the chain already enforces)

1. **Consolidate, do not originate.** Render from the truth artifacts; author only Technical Design and Rollback. Anything you cannot source is an Open Question.
2. **Only real references.** Name only files/APIs/tables that exist or are marked "to be created" in a cell. No invented paths.
3. **Separate facts from assumptions.** No plausibility language ("should work"). A guess is labeled a guess and lives in Open Questions.
4. **Present tense, projected.** "The sender retries on 5xx" — describe the planned system as the artifacts define it. Do not narrate history ("we changed…").
5. **Never claim validation ran** unless a test-result record (`.bee/logs/test-results.json`) exists; the Validation Plan links evidence, it does not assert green.
6. **Status mirrors the gate.** Never `Approved` before the gate fires; flip to `Needs Revision` when a source changes after approval.

**Not restated here because bee enforces them elsewhere** (do not re-add): "do not modify files before approval" (the write-guard denies source writes before the merged Gate 2 approves execution), "inspect the codebase first" (shaping/planning own that), "always include validation steps" (`bee finish` runs the declared suite at every cap).

## Rendering procedure (concise)

```text
1. Read plan.md frontmatter → lane. tiny/spike → no brief, stop.
2. small → mini-brief (mini-brief-template.md). standard/high-risk → full template.
3. Fill each section from its source (the map above). Source silent → Open Question.
4. Author Technical Design + Rollback from what the artifacts imply.
5. Drop empty sections (respect lane-mandatory ones).
6. Set frontmatter status from .bee/state.json gates.
7. Return; the calling skill links the brief at its gate.
```
