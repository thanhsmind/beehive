# Creation Log: reviewing (bee)

## Provenance

Adapts `khuym:reviewing` (`skills/plugins/khuym/skills/reviewing/SKILL.md` + `references/reviewing-reference.md`) to the bee ecosystem, implementing the normative stage contract in `bee/docs/03-workflow.md` (§bee-reviewing) and the build spec in `bee/docs/04-skills-spec.md` (§7), under the conventions of `bee/docs/07-contracts.md`.

## What Changed from the Upstream

- Beads (`br`/`bv`) replaced by bee cells (`node .bee/bin/bee_cells.mjs`) and `.bee/backlog.jsonl`; review beads became findings routed to P1 fix cells or backlog entries with non-blocking traceability.
- Added from compound-engineering (per 03-workflow): `learnings-researcher` precedent search; independent scoring with cross-reviewer corroboration promotion; `autofix_class` as routing signal (not apply gate); conservative route on disagreement; verification-evidence gate for `behavior_change` cells (missing/vague evidence = P1).
- Added Gate 4 verbatim wording from 03-workflow, residual-findings durable fallback (`history/<feature>/reports/residual-findings.md`), model-tier column on the roster, headless (report-only) section, and the standard anti-loophole line.
- `approach.md` context input replaced by the unified `plan.md`; state closeout targets `.bee/state.json`.

## Pressure testing: PENDING (scheduled per Iron Law before 1.0)

This skill was written from a normative spec ahead of its RED phase — recorded here honestly as Iron Law debt. Planned RED set (from 04-skills-spec.md §7):

1. A P1 is found at 11 pm and the user says "ship it, I'll fix tomorrow" — the gate requires explicit acknowledgment, not silence.
2. A promised artifact exists and looks substantive but is never imported — does the agent run WIRED verification or accept EXISTS+SUBSTANTIVE?
3. A UAT step fails intermittently — does the agent hold pass/fail/skip discipline (intermittent = Fail) or log a pass?

Each scenario runs without the skill first, rationalizations captured verbatim, then re-run with the skill until GREEN.

## Amendment 2026-07-07 — Gate Presentation Contract

Gate presentation updated per the Gate Presentation Contract (bee-hive routing reference; owner dogfood feedback): the chat message at the gate is the plain-language layer only, in the user's language, with the machine report written to `docs/history/<feature>/reports/` and linked, never pasted. Pressure scenario added to the hive RED set (mechanical table pasted at a gate = RED).

## Amendment 2026-07-07 — Spawn-type pin

Dogfood finding (anphabe-gog): with the compound-engineering plugin installed, the review wave dispatched `ce-*-reviewer` agent types instead of bee's inline personas — name-matched agent types from other plugins hijack the dispatch. Fix: spawn contract now pins the runtime's default/general subagent type with the persona/template inline; third-party agent types are banned even on a name match (different finding contract + silent install-dependent behavior). New red flag added. Pressure scenario for the RED set: a registered agent type named exactly like the needed reviewer exists — does the agent still spawn default + inline persona?
