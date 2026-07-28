---
name: bee-briefing
description: >-
  Render one human-readable implementation plan per feature so the human and the agent review and agree on the same document before code is touched. Use when planning has shaped work that needs Gate 2/3 approval, when a feature's implement plan must be (re)generated, or when the terse per-feature artifacts need consolidating into one reviewable doc. Do NOT use to originate decisions, scope, or approach — those come from exploring/planning.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: degraded
      reason: Reads cell traces and gate/status state via the vendored .bee/bin helpers.
---

# Briefing (the beekeeper's brief)

One artifact per feature: `docs/history/<feature>/implement-plan.md`. Consolidates truth artifacts (`CONTEXT.md`, `approach.md`, `plan.md`, cells, verify output); authors only Technical Design and Rollback Plan. Never originates a decision, scope, approach, or cell — inventing content to fill a section is the one failure this skill exists to prevent. Rules stated bare — decision IDs: `references/provenance.md`.

If `.bee/onboarding.json` is missing or stale, stop and invoke `bee-hive`.

## Lane forms (invoked conditionally, never automatic)

`bee-planning` calls briefing only where the fan-out table earns a brief; below high-risk the caller may skip it; briefing NOOPs when not called.

| Lane | Brief |
|---|---|
| `tiny` / `spike` | none — the Gate chat layer + `plan.md`'s direct note is the record |
| `small` | none by default; the ~15-line mini-brief only when `plan.md` exists and the user asks for a consolidated doc |
| `standard` | on-demand full template (empty sections dropped, never "N/A") when asked or the slice spans multiple domains |
| `high-risk` | mandatory full template; Rollback + Security/Permissions sections mandatory with real content |

## Modes

| Mode | Trigger | Does |
|---|---|---|
| render (chain) | `bee-planning` before Gate 2 | build `implement-plan.md`; `status: Ready for Review`; Gate 2 links it |
| refresh (chain) | after Gate 2 prep, on cell changes only (plan.md frozen post-approval) | re-project changed sections in place, never a second file |
| walkthrough (chain) | Gate 4 passed, standard/high-risk | write `walkthrough.md`; set plan `status: Shipped` |
| on-demand | user asks | render/refresh/walkthrough as above, any phase |

## 1. Section → Source Map

Every section projects from a named source; source silent → Open Question, never a guess.

| Section | Source | Rule |
|---|---|---|
| Review Status | `.bee/state.json` gates | mirror gate state, never assert early |
| Goal / Success | `CONTEXT.md` decisions | user outcome, cite D-IDs |
| Current State | exploring scout + `approach.md` | what's inspected, how it behaves now |
| Scope | `CONTEXT.md` + `plan.md` Out of scope | deferred stays deferred |
| Approach + alternatives | `approach.md` or `plan.md`'s `## Approach` | as written, no substitutes |
| **Technical Design** | authored from `approach.md` + cells | narrative as implied; beyond → Open Question |
| Affected Files | `approach.md`, then cells after prep | cells authoritative post-prep |
| Implementation Steps | `plan.md` shape, then cells | project titles/deps after prep |
| Validation Plan | cell `verify` + feature-verify record | describe + link evidence, never assert unrun |
| Risks & Mitigation | `approach.md` risk map | as written |
| **Rollback Plan** | authored | how *this* work reverts; undecided → Open Question |
| Open Questions | `approach.md` + uncovered gaps | honest home for every gap/guess |

Full template, writing guide, rendering procedure:
`references/implement-plan-template.md` ("Full template (`standard` / `high-risk`)"), ("Writing guide (bee-specific; deduped against what the chain already enforces)"), ("Rendering procedure (concise)").

Delegation: the projection walk and walkthrough reconstruction dispatch as
generation-tier I/O workers; the two authored sections (§2) stay on the session model.

## 2. The Two Authored Sections

Only two sections briefing writes from judgment — reading what the artifacts imply, never designing anew:

- **Technical Design** — the flow the approach implies: components, data shape, API/UI/security surface. A choice the artifacts don't contain is a proposal, not a rendering — Open Questions, flows back through `bee-planning`, never smuggled in.
- **Rollback Plan** — how *this* change is undone (revert commits / disable a flag / reverse a migration). Undecided → "OPEN QUESTION: …", never a plausible procedure nobody agreed to. `high-risk` must resolve it before Gate 2.

## Projection & Lifecycle

- The brief is a **projection**, never the sole change site; truth stays in `CONTEXT.md`/`plan.md`/cells/reports.
- Feedback flows to the truth artifacts first — `plan.md` revised, a locked decision superseded via `node .bee/bin/bee.mjs decisions supersede`, `CONTEXT.md` updated — THEN the brief re-renders. Hand-editing the brief alone is forbidden.
- `status` mirrors the gates: `Draft` → `Ready for Review` → `Approved` → `Needs Revision` → `Shipped` post-walkthrough.
- `plan.md` freezes at Gate 2, so drift fires on **cell changes only**: cells change after approval → `Needs Revision`, re-render before the next gate.

`bee-planning` presents gates, not briefing — the brief is what its Gate 2 message **links**. Chat stays plain-language; the brief is the durable review object; never paste the whole brief into gate chat.

## Walkthrough (post-Gate-4)

`standard`/`high-risk` only, after Gate 4; `tiny`/`spike`/`small` skip it (cap trace + commit are the record). Reconstructs from **execution reality** — capped cells' outcome/`files_changed`/deviations/`verify` output, review findings, UAT — never from the plan; where they differ, reality wins and the difference is named. Sections + quiz offer:
`references/walkthrough-template.md` ("Template"), ("Quiz (optional, P10 / decision 0020)"). Then set `status: Shipped`.

## Headless

Render/refresh mechanically; set `status` from gate state; drop empty sections; never self-approve a gate. Anything needing invented content, or feedback needing a decision superseded, goes to `Outstanding Questions` in the structured report — never guessed in.

## Red Flags

inventing content because "blank looks unprofessional" — the honest home for a gap is Open Questions · a Technical Design or Rollback holding a decision the artifacts never made · editing `implement-plan.md` directly for gate feedback, leaving `CONTEXT.md`/`plan.md` stale · hand-editing the brief as the sole change site · an auto-brief for a `tiny`/`small`/single-slice `standard` fix nobody asked to consolidate · a full 12-section brief for a small fix · `N/A` placeholder sections · a Validation Plan stating results before anything ran · the whole brief pasted into a gate chat message · a `-v2`/`-new`/dated implement-plan file, or a fresh brief without checking for the existing one · `status: Approved` before the gate passed, or stale after a source changed · a walkthrough summarizing `implement-plan.md` in past tense instead of reconstructing from cell traces/review/UAT · a walkthrough claiming "verified end-to-end" with a skipped UAT, or omitting deferred findings · secrets or PII anywhere in the brief.

Violating the letter of these rules is violating the spirit of these rules.

## Handoff

- **render / refresh** (Gate 2/3): plan rendered (`<lane>`, `status: <status>`), linked for the gate. Return to the calling skill.
- **walkthrough** (post-Gate-4): walkthrough written, implement plan `status: Shipped`. Invoke bee-scribing skill.

| Reference | When to Load |
|---|---|
| `references/implement-plan-template.md` | full template, writing guide, rendering procedure |
| `references/mini-brief-template.md` | the `small`-lane ~15-line form |
| `references/walkthrough-template.md` | post-Gate-4 sections, quiz protocol, reconstruct rules |
| `references/provenance.md` | decision IDs + rationale for every body rule |
