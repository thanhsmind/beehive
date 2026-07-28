---
name: bee-planning
description: >-
  Research the work, pick the smallest honest mode, and shape an executable plan. Use when exploring has locked CONTEXT.md, or a clear-scope task needs a mode decision and work shape before validation.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies: []
---

# planning

Waggle dance: turns locked `CONTEXT.md` into mode, lane-scaled shape, and (post-approval) current-slice cells. `.bee/onboarding.json` missing/stale → stop, invoke `bee-hive`. Rules stated bare — decision IDs: `references/provenance.md`.

## Hard Gates

- `CONTEXT.md` is truth — locked decisions cited (`per D2`), never reinterpreted, never scope-reduced.
- **Stop at Gate 2** — no cell creation, no prep artifacts before shape approval.
- **`plan.md` frozen at Gate 2:** once `approved_gates.shape` is set, content sections are immutable — only an approval stamp may follow, never a content edit.
- Cells for the **current slice only** — future-slice cells prohibited.
- Handoff only to `bee-swarming` — `bee-validating` is deleted (D1); every lane hands off here once its gate is approved.

## 1. Mode Gate — intake classification (mechanical, first — per D8)

Cheap intake classification runs before the lane-scaled bootstrap: classify from the request text + at most 2 targeted reads — tiny work must not pay full context reads before it knows it's tiny (critical-patterns digest stays mandatory every lane; D8 rescopes only *additional* reads).

Count risk flags — do not vibe it:

> auth · authorization · data model · audit/security · external systems · public contracts · cross-platform · changes behavior an existing test asserts (a covered contract must change) · the change requires weakening, deleting, or replacing existing proof · multi-domain

A covered bugfix keeping tests green + adding one scores **0** on the last two.

**Lane file caps count product files only** — never `.bee/**`, `docs/**`, plans/reports, or generated projections.

| Lane | Trigger |
|---|---|
| `docs` | all touched files are knowledge, not runtime → exit: one line, write, format-check, capture — no plan.md/cells/gates |
| `tiny` | 0–1 flags, ≤2 product files, one direct task |
| `small` | 0–1 flags, ≤3 product files, no gray areas |
| `standard` | 2–3 flags, or story-sized behavior |
| `high-risk` | 4+ flags or any hard-gate flag (auth, authz, data loss, audit/security, external provider, validation removal) |
| `spike` | one yes/no proof decides whether the plan is real — opt-in by change class, see below |

Re-runs upward on new evidence; de-escalation needs cited evidence. Record lands: `tiny` → cell `action`; `small` → logged scoping decision; `standard`/`high-risk` → `plan.md`. Greenfield: one init cell first — `references/planning-reference.md` ("Greenfield init lane").

**Spike lane is opt-in by change class, never a default (D8).** Route to `spike` only when the change class is `migration`, `security`, or reaches an external system with a side effect, **or** it uses an API/library/technique with no in-repo precedent — mirrors R55's existing narrowing of red-first proof by change class. Every other change classifies on the flag count above and builds directly: no spike, no feasibility matrix, no delta rule (both deleted outright, no replacement — D6).

## 2. Bootstrap, Discovery, Synthesis (lane-scaled)

Bootstrap scales to the lane: `tiny` = ≤2 reads only; `small` = bounded (`CONTEXT.md` if any + 3 recent decisions); `standard`/`high-risk` = full ordered sweep (area truth, `CONTEXT.md`, patterns, decisions, learnings grep, scout, re-lane only if exploring skipped). Discovery picks the lowest level removing real uncertainty (L0 skip/cite → L3 deep dive); L2+ invokes `bee-xia`, merged into the approach, never standalone below L2. Synthesis is chosen path + rejected alternatives + risk map + files/order + open questions — `## Approach` in `plan.md` by default, standalone at high-risk/L2+; `tiny`/`small` carry it in the cell/scoping decision. Mechanics: `references/planning-reference.md` ("Lane-scaled bootstrap in full", "Discovery in full", "Artifact fan-out"); `bee-hive/references/routing-and-contracts.md` ("Re-lane checkpoint").

## 3. Shape (STOP at Gate 2)

| Lane | Shape |
|---|---|
| `tiny` | request + one cell — no plan.md, the cell *is* the micro-plan |
| `small` | scoping synthesis + 1–3 cells; plan.md is opt-in — never written by default |
| `standard`/`high-risk` | one `docs/history/<feature>/plan.md`, phase plan or epic map — `references/planning-reference.md` ("Artifact: plan.md", "Phase plan vs epic map") |

`implement-plan.md` via `bee-briefing`: high-risk always, standard on-demand, small on request, tiny/spike none.

**SMALLER PATH check (per D1) — the sole reality-gate survivor, every lane.** The moment the shape is drafted (`plan.md` written for standard/high-risk, or the tiny/small cell(s) drafted below): one inline question, one line of file/command evidence, never a report — *is there a cheaper shape than this one that still honors every locked `CONTEXT.md` decision?* It is the only reality-gate item that saves money instead of spending it; every other reality-gate check (MODE FIT / REPO FIT / ASSUMPTIONS / PROOF SURFACE) and the feasibility matrix + delta rule are deleted outright, no replacement (D6). FAIL — a cheaper shape exists and still satisfies every locked decision — redraft before presenting any gate; never persist-then-preview. PASS carries straight into the review wave below, then the gate.

**Review wave (per D5) — dispatched when the shape is drafted, standard/high-risk.** At the same moment as the SMALLER PATH check, dispatch the merged reviewer: one `bee-review`-class run covering **Structure** (requirement/decision coverage, cell completeness, dependency correctness, key links, scope sanity — each BLOCKER/WARNING) and **Cells, cold pickup** (could a worker with zero session history implement each cell from `CONTEXT.md` + `plan.md` + the cell alone — each CRITICAL/MINOR). This is review value, not feasibility — repo critical pattern: "Pre-code gates filter spec defects; only diff review catches implementation defects." Findings are held — never surfaced, never block anything — until the Gate 2 presentation below; the dispatch runs *while* the SMALLER PATH check and any remaining prep happen on the session model, so the stage costs `max(reviewer, planning)`, never the sum. Lane scaling: `standard` ≤5 product files with no hard-gate flag runs both mandates inline on the session model instead (no dispatch); `standard` >5 files or any hard-gate flag, and every `high-risk` slice, dispatches the wave (`high-risk` scales it to a persona panel). `tiny`/`small` never dispatch — cold pickup is self-checked at the point the cell(s) are drafted (Tiny/small merged gate, below). One shot, then at most one blocker-scoped pass; a BLOCKER still open after pass 2 escalates to the user with both positions. Every CRITICAL cell flag is fixed before Gate 2; MINOR flags ship with a recorded note. Full mechanics, dispatch prompt, persona panel: `references/planning-reference.md` ("Review Wave in full").

**Gate 2** (standard/high-risk; small only if plan.md exists): bypass check first — read the active `gate_bypass_level`; `full`/`total` lift the high-risk floor and cover every lane, auto-approving (stamp + audit line, straight to §4); else plain-language layer + verbatim "Work shape is ready. Approve before current-work preparation?", then stop. Bypass: `bee-hive/references/routing-and-contracts.md` ("Gate bypass mode", "Gate Presentation Contract"); stamp/audit steps: `references/planning-reference.md` ("Gate 2 bypass mechanics").

**Tiny/small merged gate:** draft the cell(s) + the SMALLER PATH check (per D1 — the sole reality-gate survivor; one line of file/command evidence, never a report) FIRST, previewed in the gate message — never persist-then-preview. One question covers both approval gates; `cells add` only after approval. Bypass covers tiny/small: `gate_bypass_level` check still runs, FAIL always surfaces, PASS auto-approves both with one audit line. Protocol: `references/planning-reference.md` ("Tiny/small merged gate").

## 4. Prep (after Gate 2 approval only)

Never rewrite `plan.md` — frozen; prep only creates cells, current slice only, one batched `cells add --stdin` call — `references/planning-reference.md` ("Cell quality rules", "Example cell JSON").

**Walking skeleton first.** Any user-visible surface (UI/API/CLI) → slice 1 is the thinnest end-to-end runnable path, one happy path, real behavior however thin, no stubs; each slice's done-report owes one artifact proving it runs. Full rule: `bee-hive/references/routing-and-contracts.md` ("Ship visibility").

**One trailing test cell per slice.** Any slice with ≥1 code-touching `behavior`/`api` cell (instruction/knowledge text owes no test) emits exactly **one** `change_class: 'test'` cell, last, `deps` naming every implementation cell. Its `action` is the slice's **net behavior** — happy path, edges, errors, never per-cell internals. `bugfix`/`high-risk` stay per-cell red-first, never batched. Rule in full: `references/planning-reference.md` ("Slice-tail test batching in full").

Verify is scoped, never the full chain: `references/planning-reference.md` ("Verify scoping"). Hand off: every lane → phase `swarming` — the merged gate lives entirely inside phase `planning`; `validating` is no longer a phase-enum value (D1/D3).

## Scope-Reduction Prohibition

If the shape cannot fit the budget or context, **never** quietly shrink a locked decision or drop a must-have. Answer `SPLIT RECOMMENDED`: propose slice boundaries, each honoring every locked decision it touches, and let the user choose. Cheaper research alternatives are *noted* beside the honored decision — swapping in needs the user to supersede the D-ID.

## Headless

Run intake, bootstrap, discovery, synthesis without questions. Standard/high-risk: write `plan.md`, stop — Gate 2 never self-approved. Tiny/small: draft-cell preview + reality check, stop before persisting — merged gate never self-approved. Ambiguities → `Outstanding Questions`.

## Red Flags

skipping critical-patterns, decisions, or `CONTEXT.md` · full-bootstrapping before the mode gate picks the lane · a mode chosen without counting flags · counting `.bee/**`/`docs/**`/projections against a lane cap · phases defaulted-to, unproven · editing `plan.md` after Gate 2 · a `plan.md` for tiny, or by default for small · persisting cells before merged-gate approval · cells/prep before Gate 2 · future-slice or pseudo-cells · vague exit states, missing deps, an unrunnable `verify` · swapping a locked decision for a "better" find · shrinking scope instead of SPLIT RECOMMENDED

Violating the letter of the rules is violating the spirit of the rules.

## Reference Map

| File | When to load |
|---|---|
| `references/planning-reference.md` | Templates, fan-out table, cell quality, full bootstrap/discovery/gate/verify protocols |
| `references/edge-dimensions.md` | The 12 edge-case test-matrix dimensions |
| `references/provenance.md` | Decision IDs + rationale for every body rule |

Plan shaped, current-slice cells prepared. Invoke bee-swarming.
