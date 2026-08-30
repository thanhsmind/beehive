# proactive-leader-intake — plan

Route: class=feature, lane=standard, mode=high-risk, flags
[public-contracts, multi-domain], ~5 product files. Docs/skill-law
change only — no Rust edit expected (the advisor-ref record and the
hat seats already exist in code; the law changes are prose).

Named deviation: the herding gather transport is unavailable in this
session (no herdr pane), so planning's reads ran inline, scoped.

## What ships (per CONTEXT.md D1–D7)

The hat wave moves to the plan step and absorbs the internal consults.
Law lands in the procedure home + the two consuming skills + AGENTS,
then `bee dev regen` refreshes vendored copies.

Store-id map the cells cite: D1/D2 → a52c854d (supersedes 8fb1e0da),
D3 → 423e1664, D4 → b34fdea9, D5 → 98ac20a1, D6 → f73d6c49; procedure
home decision 07328333.

<!-- bee:not-a-deferral: this paragraph RESOLVES the handed-down questions; the answers shipped in cell pli-1 -->
Plan-step law details the review asked planning to set (CONTEXT
"Deferred To Planning" — answered here, they land in Cell A's text):

- Budget: one wave, wall-clock ceiling 10 minutes; a seat that misses
  the ceiling is dropped and NAMED in the synthesis record — partial
  return synthesizes what came back, never blocks the gate on a
  missing seat.
- Quorum: no hard quorum — the wave runs with whatever seats resolve;
  all-fall-through (zero diversity) and dropped seats are named in the
  record; `bee doctor`'s hat advisory stays the config nag.
- Idempotence carrier: the recorded advisor-ref IS the once-per-feature
  mark — a live (non-stale) advisor_ref means the wave ran; a resumed
  or compacted session never re-runs on a live ref; a stale ref after
  a material plan change permits one re-run.
- Gate bypass full/total: the wave still runs (internal consult, not a
  gate); its questions are RECORDED as plan Open Questions exactly
  like headless D6 — the recommended option proceeds, nothing new
  stops, the always-stop information-question law at exploring/Gate 1
  is untouched.

## Slice 1 (the whole feature) — 3 cells, strict order A → B → C

C runs last: it carries the one `bee dev regen`, so vendored targets
render once, after A and B settle the source (regen-obligation
barrier).

Source rule for every cell: edit `skills/**` and
`packages/bee/AGENTS.block.md` only — NEVER `.claude/skills/**` or the
AGENTS.md BEE block directly; regen rewrites those.

### Cell A — procedure home (skills/bee-hive/references/gates-and-delegation.md)

Rewrite the "Hat wave" section (lines 179–230; "Judgment contract"
opens at 231) to the new law, and update the two plan-checker/consult
mentions at lines 56 and 133 in the same file:

- Firing point (D1): after shaping locks the spec, at the plan step —
  the leader opens the wave proactively to build the plan; synthesized
  answers feed plan.md through the leader (synthesis stays
  decide-altitude; Lock still never takes hat output directly).
- Threshold (D2): once per feature, never per message; clear/tiny asks
  skip the wave — the ceremony-capture line stays.
- Seats (D3): default 3 (hat-facts-gaps, hat-alternatives,
  hat-user-impact); all 5 on high-risk.
- Absorption (D4): the planning review wave (plan-checker) and the
  high-risk advisor gate consult RUN AS the hat wave; the wave's
  synthesis is recorded via `bee state advisor-ref record`, so the
  existing high-risk gate precondition is satisfied unchanged — no
  code edit. TIMING LAW (feasibility blocker fix): the ref is
  recorded AFTER plan.md reaches its gate-ready bytes and after the
  last pre-gate decision log — the staleness anchors are plan.md's
  sha256 and the newest decision id
  (verbs/state_group/advisor_ref.rs:166-180); a ref recorded at wave
  time goes stale by construction. bee-reviewing/Gate 3 untouched
  (explicit line).
- Instruments at plan altitude (explicit column update): facts-gaps
  keeps the 5-Layer rubric + Truth Table over the drafted plan;
  alternatives runs the SMALLER PATH question AT PLAN ALTITUDE by
  citing bee-planning's inline mandate (one home, no second copy);
  user-impact runs gray-area probes over the planned behavior. The
  spec-altitude instrument wording stays only in the pre-Lock window.
- Mandate ownership (absorbed plan-checker, blocker fix): MANDATE 1
  Structure (BLOCKER/WARNING, the five structure dimensions) rides
  hat-facts-gaps' plan-step question; MANDATE 2 cold-pickup
  (CRITICAL/MINOR) stays with the leader at cell drafting — the same
  self-check the tiny/small lanes already use. Vocabularies never
  merge.
- Pre-Lock spec-critique window stays discretionary (D5) — keep the
  existing window text, retitled so the two windows read apart.
- Headless (D6): new bullet inside the Hat wave section — unattended
  runs record the wave's questions in the plan's Open Questions
  (approach.md "Questions still open" carrier), never block, never
  self-answer.
- Communication (D7): new bullet — one plain state line while the wave
  runs, one leader voice out, findings filtered against the request
  text.
- "No checker verb" paragraph (lines 227-229) updates: the plan-step
  wave's check is Gate 2 (the human at the shape/execution gate); the
  pre-Lock wave's check stays Lock.

### Cell B — planning consumes the wave (skills/bee-planning/SKILL.md + references/planning-reference.md)

- SKILL.md "Shape"/"Gate": standard/high-risk plan review = the hat
  wave, by pointer to the procedure home; scaling per D2/D3 (standard
  ≤5 files no-dispatch inline self-check stays, per D2's threshold).
- planning-reference.md "Review wave": rewrite to the hat-wave form —
  same two vocabularies (Structure BLOCKER/WARNING; Cells
  CRITICAL/MINOR) carried by the synthesis, one blocker pass rule kept.
- SMALLER PATH one-home (CONTEXT discretion): the inline check stays
  the home at every lane; the hat-alternatives seat's prompt cites it
  instead of duplicating it — one line in the reference.

### Cell C — pointers, AGENTS block, knowledge, regen (ordered)

Strict order inside the cell:

1. skills/bee-shaping/SKILL.md:71-73 — split into two pointers:
   pre-Lock critique wave (discretionary, D5) and the plan-step wave
   (D1).
2. Old-law pointer sweep, all four sites:
   skills/bee-hive/references/routing-and-contracts.md:11 and
   :139-140 ("merged reviewer" / "persona panel"),
   skills/bee-hive/references/go-mode.md:24, and
   skills/bee-swarming/references/swarming-reference.md:362 ("review
   role consumed by bee-planning's merged reviewer") — update to the
   new law's pointer, naming the dispatch-kind change (plan check now
   rides `--kind advisor` hat seats, not the review tier).
3. packages/bee/AGENTS.block.md — update the delegation/deep-contracts
   lines naming the advisor consult and hats (pointer language only).
   Never edit AGENTS.md's BEE block directly.
4. Knowledge homes — concept EDITS, not stubs:
   docs/knowledge/areas/advisor-protocol/triggers.md:39 (B3 pre-gate
   consult trigger → the hat wave's synthesis, recorded post-plan per
   Cell A's timing law) and
   docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md:176
   (seat/tier law: plan check moves from the review tier to the
   advisor-kind hat seats). Plus one capture stub for the settlement.
5. `bee dev regen` — re-renders AGENTS.md from the block and refreshes
   vendored `.claude/skills/**`; confirm AGENTS.md carries the edit.
6. `bee knowledge index` (files added/renamed under docs/knowledge)
   then `bee knowledge index --check` green.

## Proof (docs change type)

- `bee dev regen` — clean/idempotent run — regen chain is the parity
  net for vendored copies and AGENTS.md.
- `rg` pointer checks: exactly ONE procedure home for the hat wave;
  shaping/planning/AGENTS/routing/go-mode carry pointers, no second
  copy. Hand-checked — no automated pointer gate exists (the old
  test_skill_pointers.mjs is gone); recorded as such on the cap.
- `bee knowledge check` (NON-strict; 73 pre-existing profile warnings)
  green, plus `bee knowledge index --check` green.
- Declared cargo suite deliberately not run locally — scope reason:
  prose/docs-only diff, zero Rust source touched; the suite rides CI
  on every push, unchanged.
- Known cosmetic debt, deliberately out of scope: three Rust doc
  comments say "five hats" (models.rs:664,:695; doctor.rs:268) —
  comments only, no assertion pins the count; noted for capture.

## SMALLER PATH check

Could one cell do it? The three files' edits are disjoint and each has
its own proof surface; one mega-cell loses the one-commit-per-cell
trail on a doctrine change future reviewers read. Could we skip
planning-reference? No — it is the plan-checker's home; leaving it
contradicts D4. Shape holds: 3 cells, no code, no new artifact types.

## Rollback

Prose-only + regen: `git revert` of the three cell commits restores
the old law; decisions store carries the supersession trail
(8fb1e0da superseded by the plan-step refinement). No data, no
migration, no external surface.
