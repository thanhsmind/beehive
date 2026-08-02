# 11 — Implement-Plan Adoption (Antigravity-style briefing)

- **Status:** proposed (owner review pending)
- **Date:** 2026-07-08
- **Source:** owner request + `docs/sample-implement-plan.md` (Antigravity Implementation Plan template)
- **Problem named by the owner:** bee's step-by-step communication is short and hard to understand. The chain's artifacts (CONTEXT.md, approach.md, plan.md, cells) are agent-optimized and scattered; the chat plain-language layer is ephemeral. There is no single durable, human-readable document that agent and human both anchor to and agree on before code is touched.

## 1. What Antigravity's Implementation Plan actually adds

The Antigravity model: agent presents **intent, scope, risks, and verification** in one reviewable document *before* touching code; human comments; agent revises; only then implementation — followed by a Walkthrough with evidence.

Mapping the 12-section template against what bee already produces:

| Template section | Where bee has it today | Grade |
|---|---|---|
| 0 Review Status (Draft/Approved lifecycle) | `state.json` gates — machine-only, invisible in any doc | **gap** |
| 1 Goal / success criteria | CONTEXT.md boundary + locked decisions | covered, terse |
| 2 Current State / Findings | exploring scout + planning discovery (merged into approach.md) | covered, terse |
| 3 Scope / Out of scope | CONTEXT.md decisions + plan.md `Out of scope` | covered, split |
| 4 Proposed Approach + alternatives | approach.md (recommended path, rejected alternatives) | covered |
| 5 Technical Design (flow, data, API, UI, security) | fragments in approach.md; mostly implicit in cells | **gap** (no narrative) |
| 6 Affected Files / Components | approach.md `Files and order` + cell `files` | covered, scattered |
| 7 Implementation Steps | plan.md shape + cells | covered, agent-shaped |
| 8 Validation Plan | cell `verify` + (at proposal time) the standalone validating stage's reality gate & feasibility matrix — since deleted; the reality gate's SMALLER PATH survivor and the review wave now live in `bee-planning` (validation-diet D1/D5/D6) | covered, scattered across reports |
| 9 Risks & Mitigation | approach.md risk map | covered |
| 10 Rollback Plan | **nowhere in bee** | **gap** |
| 11 Open Questions | approach.md `Questions for planning's reality check` (formerly "for validating") | covered |
| (post-impl) Walkthrough | worker reports + review findings — machine layer only | **gap** (no human artifact) |

Diagnosis: **the information mostly exists; the human-grade consolidation does not.** Four genuine gaps: (a) one readable document instead of 3–4 terse files, (b) a Technical Design narrative, (c) a Rollback plan, (d) a visible Review Status lifecycle. A fifth, deferrable: the post-implementation Walkthrough.

Worth noting: Antigravity enforces "no code before approval" by agent discipline; bee already enforces it mechanically (write-guard denies source writes before Gate 2's execution approval). The brief adds the *legibility* Antigravity has; bee keeps the *enforcement* Antigravity lacks.

## 2. Design decision: a separate rendering skill, not a bigger plan.md

Owner constraint, adopted verbatim: document generation is its own skill; the chain (harness) only *calls* it.

**Proposed skill: the beekeeper's brief** (shipped as a standalone briefing skill; today `bee-shaping`'s "Brief" section). In the hive metaphor: `bee-planning` is the waggle dance (bee-to-bee communication, precise and terse); the Brief step translates the dance for the beekeeper. It writes and maintains **one artifact per feature**:

```
docs/history/<feature>/implement-plan.md
```

Key property: **briefing is a consolidator, not a second planner.** It renders the brief *from* the truth artifacts (CONTEXT.md, approach.md, plan.md, cells, validation reports) and authors only the two genuinely new sections (Technical Design narrative, Rollback). It never invents content the chain has not produced — no double planning, no second opinion drift.

### Why not extend an existing skill

- **bee-planning:** already the heaviest skill; its outputs are agent-shaped *by design* (cells are executable prompts). Mixing a human-audience artifact into it blurs both audiences and makes the renderer un-callable on demand ("regenerate the plan doc").
- **The scribe (today `bee-capturing`):** owns durable, tech-agnostic state (`docs/specs/`). The brief is per-feature, tech-*specific*, and lives in history. Opposite rules (scribing quarantines technology; the brief centers on it).
- **Gate Presentation Contract alone:** fixes the chat layer but chat evaporates; the reports it links are machine-layer by contract. The gap is precisely the durable middle layer.

Per decision 0002, adding a skill requires a decision record naming the uncovered gap — this document is the analysis; a `docs/decisions/0008-*.md` record accompanies the build. Per the Iron Law, the skill is built through the skill-writing discipline (now [handbook/writing-skills.md](handbook/writing-skills.md)) with a failing pressure test first.

## 3. Truth model: the brief is a projection *and* an agreement record

Extends the existing **Projection Rule (D12)**:

- **Truth stays where it is:** CONTEXT.md (decisions), plan.md + cells (executable work), planning's reality-check evidence (SMALLER PATH + review wave). No section of the brief overrides them.
- **The brief is the human-layer projection** of those artifacts — regenerated whenever a source changes.
- **Approval happens on the brief.** Gate 2 — now also the execution approval, the old standalone execution gate folded into it (validation-diet D2) — links `implement-plan.md` as *the* review object. Human feedback on the brief flows back into the truth artifacts (planning revises plan.md / CONTEXT.md decisions get superseded), then the brief re-renders. The brief itself is never hand-edited as the sole change site.
- **Review Status is real state:** frontmatter mirrors `state.json` gates (`Draft → Ready for Review → Approved → Needs Revision`). Approved at Gate 2 covers shape; the Validation Plan section is patched with actual reality-check evidence before the same gate's execution component is approved.
- **Drift guard (mechanical, later):** frontmatter records source-artifact hashes; when a source changes after approval, status flips to `Needs Revision` and `bee_status` warns. (v1 can start with the rule stated in the skill; the hash check is a follow-up helper change, not a blocker.)

This resolves the dual requirement — "tài liệu agent bám vào" *and* "human đọc được cùng đồng ý": the agent anchors to the truth artifacts as before, the human anchors to the brief, and the projection + status machinery keeps the two provably in sync. Worker isolation is unchanged (workers still receive cell + CONTEXT.md only — the brief adds no dispatch context).

## 4. Lane scaling (ceremony must not regress)

"Lanes scale ceremony, never memory" applies to the brief exactly as to everything else:

| Lane | Brief form |
|---|---|
| `tiny` | none — the Gate chat layer suffices; plan.md's direct note is the record |
| `spike` | none — the spike question *is* the brief |
| `small` | **mini-brief** (~15 lines): Goal · Scope in/out · Affected files · Validation · one-line Risk · one-line Rollback |
| `standard` | full template, **empty sections dropped** (no `N/A` placeholder rot) |
| `high-risk` | full template; Rollback and Security/Permissions sections mandatory |

Anti-bloat rules carried from CONTEXT.md discipline: concrete language, no placeholders, sections that do not apply are deleted rather than templated. A tiny fix wearing a 12-section brief is the same red flag as a tiny fix wearing epic ceremony.

## 5. Chain integration (the "harness only calls it" contract)

Touch points, all one-to-two-line edits:

1. **`bee-planning` §5 (Shape):** after writing plan.md (requirements-only) and before presenting Gate 2 — *"Invoke the Brief step to render `implement-plan.md`; the Gate 2 message links the brief as the review document."*
2. **`bee-planning` §6 (Prep):** after cells are created, briefing refreshes the projected sections (Affected Files, Implementation Steps) from the cells, and, once `bee-planning`'s own reality check (SMALLER PATH) and review wave run, patches the Validation Plan section with their evidence links before Gate 2's execution component is approved (the standalone validating stage is deleted — validation-diet D1 — so there is no separate handoff for this step).
3. **`bee-hive` routing:** one row — "(re)generate the implement plan for a feature" → the Brief step (today in `bee-shaping`), on demand any phase.
4. **`routing-and-contracts.md`:** skill-catalog row + chaining-contract row (`briefing | reads: CONTEXT.md, approach.md, plan.md, cells, planning's reality-check evidence | writes: docs/history/<feature>/implement-plan.md`).
5. **Gate Presentation Contract:** unchanged in structure; the brief becomes the canonical linked document for Gate 2 (reports/ stays the machine layer for reality-gate tables etc.).

No new hooks (cap untouched), no new helper CLI in v1, no write-guard change (`docs/` is already writable in every phase). Codex parity is automatic — the skill is markdown + the same file conventions.

## 6. Skill shape (for the build, after approval)

```
skills/bee-shaping/                         # the Brief step's home today (originally built as a standalone briefing skill)
  SKILL.md                                  # modes: render (Gate 2), refresh (post-prep, post-reality-check), on-demand
  references/implement-plan-template.md     # full template — adapted from docs/sample-implement-plan.md
  references/mini-brief-template.md         # small-lane form
```

(The pressure-test creation log lives with the briefing skill's records under `docs/decisions/skills/`.)

Core rules for SKILL.md (distilled from the sample's agent guide, deduplicated against what bee already enforces):

- Consolidate from truth artifacts; author only Technical Design and Rollback; **never** state anything the chain has not produced — missing info goes to Open Questions, not guesses.
- Only name files/APIs/tables that exist or are explicitly marked "to be created" (cells already carry this).
- Separate facts from assumptions; no plausibility language ("should work") — same evidence bar as the rest of the hive.
- Never claim the reality check ran unless its evidence exists; the Validation Plan section links evidence, it does not assert it.
- Status lifecycle mirrors gates; a source change after approval flips `Needs Revision`.

Dropped from the sample as redundant with bee: "do not modify files before approval" (write-guard), "inspect the codebase first" (exploring/planning contracts), "always include validation steps" (cells refuse to exist without `verify`).

## 7. Built: the Walkthrough artifact

Antigravity's third artifact (post-implementation summary + how-to-test) is implemented as the Brief step's **walkthrough mode** (today in `bee-shaping`), invoked by `bee-reviewing`'s Gate 3 when a user-invoked review session covers a `standard`/`high-risk` feature — Gate 3 no longer runs automatically at feature close (review-on-demand, decision 565e68d0); it fires only inside that session. It writes `docs/history/<feature>/walkthrough.md` — what shipped, how it was verified (real recorded evidence), how to test it, deviations from plan, and known limitations — **reconstructed from the execution records** (capped cell traces, review findings, UAT), never from the plan, and sets the implement plan `status: Shipped`. It was initially scoped as phase 2 but built the same session on owner request; its three walkthrough-specific RED scenarios are recorded in the briefing skill's creation log under `docs/decisions/skills/` (all passed at the Fable/Opus tier — plan-narration-vs-reality, over-claimed verification, hidden findings).

## 8. Risks of this adoption itself

| Risk | Mitigation |
|---|---|
| Brief drifts from plan.md/cells after approval | Projection rule + status flip; hash guard as follow-up |
| Ceremony regression in small lanes | lane table above; tiny/spike produce no brief at all |
| Placeholder/template rot | drop-empty-sections rule; concrete-language rule inherited from CONTEXT.md |
| Brief becomes a second planning surface (agent "thinks in the brief") | consolidator contract — render *from* artifacts; the two authored sections have no execution authority |
| 13th skill sprawl concern | decision-gate record required (0002); briefing is a workflow-stage skill (communication layer), not a domain skill |

## 9. Proposed next steps

1. Owner approves/amends this design (esp. the skill name and the lane table).
2. Decision record `0008-briefing-skill.md` (gap: human-grade agreement doc; why not planning/scribing).
3. Build the briefing skill through the skill-writing discipline (failing pressure test first: e.g., a standard-lane feature whose Gate 2 message currently forces the human to open three files).
4. Wire the five chain touch points (§5); absorb `docs/sample-implement-plan.md` into `references/implement-plan-template.md` and retire the sample.
5. Dogfood on the next standard-lane feature; walkthrough mode afterwards.
