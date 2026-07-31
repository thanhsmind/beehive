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

Waggle dance: turns locked `CONTEXT.md` into mode, lane-scaled shape, and (post-approval) current-slice cells. `.bee/onboarding.json` missing/stale → stop, invoke `bee-hive`.

## Hard Gates

- `CONTEXT.md` is truth — locked decisions cited, never reinterpreted, never scope-reduced.
- **Stop at Gate 2** — no cell creation, no prep artifacts before shape approval.
- **`plan.md` frozen at Gate 2:** once `approved_gates.shape` is set, content sections are immutable — only an approval stamp may follow, never a content edit.
- Cells for the **current slice only** — future-slice cells prohibited.
- Handoff only to `bee-swarming` — every lane, once its gate is approved.

## 1. Mode Gate — intake classification

Cheap intake classification runs first: classify from the request text + at most 2 targeted reads — tiny work must not pay full context reads before it knows it's tiny (the critical-patterns digest stays mandatory; only *additional* reads scale down).

Count risk flags — do not vibe it:

> auth · authorization · data model · audit/security · external systems · public contracts · cross-platform · changes behavior an existing test asserts · weakening, deleting, or replacing existing proof · multi-domain

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

**Spike lane is opt-in by change class, never a default.** Route to `spike` only for `migration`, `security`, an external side effect, or no in-repo precedent. Everything else classifies on the flag count above and builds directly — no spike.

## 2. Bootstrap, Discovery, Synthesis (lane-scaled)

Bootstrap scales to the lane: `tiny` = ≤2 reads only; `small` = bounded (`CONTEXT.md` if any + 3 recent decisions); `standard`/`high-risk` = full ordered sweep (area truth, `CONTEXT.md`, patterns, decisions, learnings, scout; re-lane only if exploring skipped). Discovery picks the lowest level removing real uncertainty (L0 skip/cite → L3 deep dive); L2+ invokes `bee-xia`, merged into the approach. Synthesis is chosen path + rejected alternatives + risk map + files/order + open questions — `## Approach` in `plan.md` by default, standalone at high-risk/L2+; `tiny`/`small` carry it in the cell/scoping decision. Mechanics: `references/planning-reference.md` ("Lane-scaled bootstrap in full", "Discovery in full", "Artifact fan-out"); `bee-hive/references/routing-and-contracts.md` ("Re-lane checkpoint").

## 3. Shape (STOP at Gate 2)

| Lane | Shape |
|---|---|
| `tiny` | request + one cell — no plan.md, the cell *is* the micro-plan |
| `small` | scoping synthesis + 1–3 cells; plan.md is opt-in — never written by default |
| `standard`/`high-risk` | one `docs/history/<feature>/plan.md`, phase plan or epic map — `references/planning-reference.md` ("Artifact: plan.md", "Phase plan vs epic map") |

`implement-plan.md` via `bee-briefing`: high-risk always, standard on-demand, small on request, tiny/spike none.

**SMALLER PATH check — every lane.** Once the shape is drafted (`plan.md`, or the tiny/small cell(s) below): one inline question, one line of file/command evidence, never a report — *is there a cheaper shape than this one that still honors every locked `CONTEXT.md` decision?* The check saves money, not spends it. FAIL → redraft before presenting any gate, never persist-then-preview. PASS → straight into the review wave below, then the gate.

**Review wave — dispatched when the shape is drafted, standard/high-risk.** Same moment as SMALLER PATH: dispatch the merged reviewer, one `bee-review`-class run covering **Structure** (requirement/decision coverage, cell completeness, dependency correctness, key links, scope sanity — BLOCKER/WARNING) and **Cold pickup** (could a zero-history worker implement each cell from `CONTEXT.md` + `plan.md` alone — CRITICAL/MINOR). Spec defects only; findings held until Gate 2, running *while* remaining prep happens, so cost is `max(reviewer, planning)`, never the sum. Scaling: `standard` ≤5 files, no hard-gate flag → inline; >5 files or a hard-gate flag, and every `high-risk` → dispatch (persona panel). `tiny`/`small` skip — cold pickup self-checks at the merged gate below. One shot, one blocker-scoped pass; a BLOCKER surviving both escalates with both positions. CRITICAL fixed before Gate 2; MINOR ships noted. Full: `references/planning-reference.md` ("Review Wave in full").

**Gate 2** (standard/high-risk; small only if plan.md exists): read the active `gate_bypass_level` first; `full`/`total` lift the high-risk floor, auto-approving every lane (stamp + audit line, straight to §4); else plain-language layer + verbatim "Work shape is ready. Approve before current-work preparation?", then stop. Bypass: `bee-hive/references/routing-and-contracts.md` ("Gate bypass mode", "Gate Presentation Contract"); stamp/audit: `references/planning-reference.md` ("Gate 2 bypass mechanics").

**Tiny/small merged gate:** draft the cell(s) + the SMALLER PATH check FIRST, previewed in the gate message — never persist-then-preview. One question covers both approval gates; `cells add` only after approval. Bypass: `gate_bypass_level` still runs, FAIL always surfaces, PASS auto-approves both with one audit line. Protocol: `references/planning-reference.md` ("Tiny/small merged gate").

## 4. Prep (after Gate 2 approval only)

Never rewrite `plan.md` — frozen; prep only creates cells, current slice only, one batched `cells add --stdin` call — `references/planning-reference.md` ("Cell quality rules", "Example cell JSON").

**Walking skeleton first.** Any user-visible surface (UI/API/CLI) → slice 1 is the thinnest end-to-end runnable path, one happy path, real behavior, no stubs; each slice's done-report owes one artifact proving it runs. Full rule: `bee-hive/references/routing-and-contracts.md` ("Ship visibility").

**One trailing test cell per slice — coverage judgment first, authoring second.** Any slice with ≥1 code-touching `behavior`/`api` cell (instruction/knowledge text owes no test) emits exactly **one** `change_class: 'test'` cell, last, `deps` naming every implementation cell: a code-touching slice with no test cell is a planning defect. Its **first mandated step is a coverage judgment, not authoring** — cite the nearest existing tests by `file:line` and state whether they already cover the slice's acceptance criteria. Covered → the cell caps by running those tests green and recording "already covered, no new rows". Partly covered → it authors **only** the uncovered gap. For a `tiny`/`small` slice whose net behavior is not a public contract and carries no hard-gate flag, verified transcripts recorded on the implementation cells satisfy the coverage judgment too: the cell caps by re-running the cited transcript commands green and recording "proven by transcript" with pointers — new rows only where a transcript cannot prove the criterion. **A test cell that authors no test is not a defect**; authoring rows that duplicate existing coverage is the waste this rule exists to stop. Where rows are genuinely owed, the shape at `standard` and below is the triad — happy path, edge cases, error paths — at its smallest demonstrating size; `references/edge-dimensions.md` applies only at `high-risk`/hard-gate. Red-first cells stay per-cell, never batched — scoped as the proof-tier rules compute it, not as "`bugfix`/`high-risk`": `security`/`migration` every lane, `bugfix`/`behavior`/`api` at `high-risk`, with `bugfix` below `high-risk` keeping repro-first at `targeted-green`; at `high-risk`, `refactor`/`formatting` are still `suite-green` and `test` still `targeted-green`, so the lane alone does not buy red-first. Full: `references/planning-reference.md` ("Slice-tail test batching in full").

**Test-cell debt has no lane exemption.** The test-cell debt check is keyed on the **feature**, not the slice, and its two kinds differ in what they require: *missing* needs capped code-touching `behavior`/`api` cells to exist (an unrecorded file list counts as code-touching) with no `test` cell at all — dropped counts as none; *not-green* fires from the offending `test` cell **alone**, no capped-behavior requirement — still open/claimed/blocked, capped with a failing verify, or capped `trace.proof: "unrecorded"`. Those three states are the whole predicate, **not** "capped green with recorded proof": a `test` cell capped on `--feature-verify-pending` with no failing verify recorded on it clears this door and is caught by the feature-verify debt check instead. No `gate_bypass` level lifts either kind. Plan one trailing test cell even on a `high-risk` feature whose cells each carry per-cell red-first proof, and let it cap green; that cell's coverage judgment is usually the whole of its work.

Verify is scoped, never the full chain: `references/planning-reference.md` ("Verify scoping"). Hand off, every lane: `node .bee/bin/bee.mjs state set --owner planning --phase swarming --next-action "Invoke bee-swarming."` — the merged gate lives entirely inside phase `planning`.

## Scope-Reduction Prohibition

If the shape cannot fit the budget or context, **never** quietly shrink a locked decision or drop a must-have. Answer `SPLIT RECOMMENDED`: propose slice boundaries honoring every locked decision, let the user choose. Cheaper alternatives are *noted* beside the honored decision — swapping in needs the user to supersede the D-ID.

## Headless

Run intake, bootstrap, discovery, synthesis without questions. Standard/high-risk: write `plan.md`, stop — Gate 2 never self-approved. Tiny/small: draft-cell preview + reality check, stop before persisting. Ambiguities → `Outstanding Questions`.

## Red Flags

skipping critical-patterns, decisions, or `CONTEXT.md` · full-bootstrapping before the mode gate picks the lane · a mode chosen without counting flags · counting `.bee/**`/`docs/**`/projections against a lane cap · phases defaulted-to, unproven · editing `plan.md` after Gate 2 · a `plan.md` for tiny, or by default for small · persisting cells before merged-gate approval · cells/prep before Gate 2 · future-slice or pseudo-cells · vague exit states, missing deps, an unrunnable `verify` · swapping a locked decision for a "better" find · shrinking scope instead of SPLIT RECOMMENDED

When a rule's letter stops serving its purpose here, say so out loud and
deviate with a recorded reason — boundary rules (gates, state, secrets) hold
as written; silent deviation is the defect (bee-hive routing reference,
"Judgment contract").

## Reference Map

| File | When to load |
|---|---|
| `references/planning-reference.md` | Templates, fan-out, cell quality, full bootstrap/discovery/gate/verify protocols |
| `references/edge-dimensions.md` | The 12 edge-case test-matrix dimensions — `high-risk`/hard-gate only; `standard` and below use the triad |

Plan shaped, current-slice cells prepared. Invoke bee-swarming.
