---
name: bee-planning
description: >-
  Shape approved-scope work into an executable plan — classify the lane, research just enough, draft the smallest honest shape, gate it, and prepare current-slice cells. Use when shaping has locked CONTEXT.md, or a clear-scope task needs a lane and a work shape before execution. Not for locking product decisions (bee-shaping) or executing approved cells (bee-swarming).
metadata:
  version: '0.3'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: unavailable
      reason: Route, gate, and cell records are written through the vendored bee binary. The binary is vendored into the repo by onboarding; no Node runtime is involved.
---

# Planning — shape the work

Inputs: locked decisions in `docs/history/<feature>/CONTEXT.md`.
Craft: `.bee/expertise/planning.md`.

## Route

Classify before reading deeply — the request text plus at most two targeted
reads. Count risk flags, never vibe it:

> auth · authorization · data model · audit/security · external systems ·
> public contracts · cross-platform · changes behavior an existing test
> asserts · weakening, deleting, or replacing existing proof · multi-domain

A covered bugfix keeping tests green and adding one scores 0 on the last two.
File caps count product files only — never `.bee/**`, `docs/**`, or plans.

| Lane | Trigger |
|---|---|
| `docs` | all touched files are knowledge, not runtime → write, format-check, capture — no plan, cells, or gates |
| `tiny` | 0–1 flags, ≤2 product files, one direct task |
| `small` | 0–1 flags, ≤3 product files, no gray areas |
| `standard` | 2–3 flags, or story-sized behavior |
| `high-risk` | 4+ flags or any hard-gate flag (auth, authz, data loss, audit/security, external provider, validation removal) |
| `spike` | one yes/no proof decides whether the plan is real — only for migration, security, an external side effect, or no in-repo precedent |

Record: `bee route --set --class <c> --lane <l> --flags <f> --files <n>`;
re-route upward on new evidence any time, de-escalate only on cited evidence.
A code-touching route's next action includes creating the feature worktree
and opening the session there (worktree-first — AGENTS.md; `docs` and a solo
`tiny` stay in main). Code truth changes only in the feature worktree, the
user tests at staging once the slice is ready, and main receives the branch
only after its uat gate.

## Research

Reading scales with the lane: tiny keeps its two targeted reads; small adds
`CONTEXT.md` and recent decisions; standard/high-risk add area truth, critical
patterns, decisions, and prior learnings — precedent beats research. Remove
uncertainty at the lowest cost: cite a pattern or verify one fact; unfamiliar
territory or competing approaches dispatch `bee-researching`, findings merged in.

## Shape

Draft the smallest honest shape — the smallest that still covers what the
work endangers (decomposition, walking skeleton, dependency thinking:
`.bee/expertise/planning.md`). Every locked decision lands in it, cited. By lane:

| Lane | Shape |
|---|---|
| `tiny` | one cell — the cell is the micro-plan |
| `small` | logged scoping synthesis + 1–3 cells; `plan.md` only on request |
| `standard`/`high-risk` | `docs/history/<feature>/plan.md` — `references/planning-reference.md` ("Artifact: plan.md", "Phase plan vs epic map") |

Once drafted, the SMALLER PATH check — every lane, one inline question, one
line of evidence: *is there a cheaper shape that still honors every locked
decision?* FAIL → redraft. Standard/high-risk add the review wave before the gate.

## Gate

Standard/high-risk: present the shape in plain language — what will be built,
why this size, cost if the shape is wrong — link the plan, then ask verbatim:
"Work shape is ready. Approve before current-work preparation?" and stop. On
approval, `bee gate --name shape --approved true`; `plan.md` freezes —
a stamp may follow, a content edit may not.

Tiny/small merge shape and execution: draft the cell(s), preview them in the
gate message — never persist-then-preview — then ask: "Work shape + execution:
I'm about to do [X] via [Y], verified by [Z]. Approve?" One yes records both
(`bee gate --merge --approved true`); cells persist only after it —
full protocol: `references/planning-reference.md` ("Tiny/small merged gate").

## Prep

After approval only: current-slice cells, walked through pre-flight and a
clean `--dry-run` before the one batched `bee cells add --stdin` call —
later slices keep one-line headlines, not cells
(`references/planning-reference.md` ("Pre-flight before cells add", "Cell
quality rules")). A user-visible surface makes slice 1 a walking skeleton:
end-to-end, real behavior, no stubs.

The writer owns tests TDD-style as part of each cell — coverage judgment
first: cite existing tests by file and case, author only the gap
(`.bee/expertise/tests.md`). The agent owns test scope: pick the proof
each cap's change type needs (code → related tests green; docs →
parity/pointer checks; behavior → judge verdict), run it, and record it
as the cap's proof line. `bee close` and `bee worktree merge` check that
recorded proof; CI runs the full declared command on every push
(`references/planning-reference.md` ("Test scoping")). Then
`bee state set --owner planning --phase swarming --next-action "Invoke bee-swarming."`

## Scope integrity

When the shape cannot fit the budget, never quietly shrink a locked decision
or drop a must-have. Answer SPLIT RECOMMENDED — slice boundaries honoring
every decision, the user choosing what waits; a cheaper swap needs a supersede.

## Headless

`bee-hive` ("Headless") governs; planning's run drafts the shape and stops
at the gate — never self-approved.

## Hard rules

- Locked decisions are cited, never reinterpreted (AGENTS.md), never scope-reduced.
- No cells, no prep artifacts, before the gate is approved.
- Current slice only — a future-slice cell does not exist yet.
- Scope gaps go back to `bee-shaping`; the finished shape only to `bee-swarming`.

## References

| File | When to load |
|---|---|
| `references/planning-reference.md` | plan.md/approach.md templates, cell quality rules + example JSON, merged-gate protocol, review wave, verify scoping, greenfield init lane |
| `references/edge-dimensions.md` | 12 edge-case dimensions — high-risk/hard-gate test matrix only; standard and below use the triad |
| `.bee/expertise/planning.md`, `.bee/expertise/tests.md` | Decomposition, walking skeleton, smaller path, cold pickup; coverage judgment, case selection, red-first |
| `.bee/expertise/INDEX.md` | The work is domain-shaped — data and migrations, a contract callers depend on, a trust boundary, a rollout, a speed budget, a surface people use: route from the index for the ordering and reversibility constraints before slicing |
