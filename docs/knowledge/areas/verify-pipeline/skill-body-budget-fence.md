---
type: bee.area
title: Verify Pipeline — skill body byte budget and provenance fence
description: "The blocking gate that caps every skill instruction body at a recorded byte budget (a ratchet that only shrinks) and forbids provenance citations in migrated bodies — why bodies are metered in bytes not lines, and why migrated status is an explicit list, never inferred from size."
tags: [verify-pipeline, guards, instruction-surfaces]
timestamp: 2026-07-28
bee:
  id: verify-pipeline-skill-body-budget-fence
  lifecycle: active
  areas: [verify-pipeline]
  required_context: [areas/verify-pipeline/skill-reference-pointer-integrity.md]
  decisions: [skill-token-diet D1, skill-token-diet D2, skill-token-diet D6, skill-token-diet D7, skill-token-diet D8, skill-token-diet validation D1, skill-token-diet validation D2]
---

## Purpose

Skill instruction bodies are injected whole on every invoke and never unloaded, so every byte in a body is a permanent per-session tax. This gate makes that tax finite and one-directional: each body has a recorded byte budget that may only shrink, migrated bodies may not exceed 8,192 bytes, and a migrated body may not carry provenance citations inline. It exists because bodies grew monotonically — every lesson learned became permanent prose — and because line counts pass human review while byte counts triple (dense 142-character law-lines cost what a file three times longer should).

## Entry Points & Triggers

- Runs in the verify chain as a selftest + live pair, beside the other instruction-surface fences: the selftest proves the gate bites on fixtures before the live run judges the repo.
- Runs inside every skill-migration change's own verification.
- A baseline maintenance mode seeds a new skill's budget at its current size and lowers a trimmed one; it refuses to raise any entry.

## Data Dictionary

- **Body**: a skill's instruction document, loaded whole on invoke. Metered in bytes (not lines — the dense-line failure mode is exactly what line budgets miss).
- **Budget**: the per-skill recorded byte ceiling. Ratchet semantics: only lowered, never raised.
- **Migrated**: a skill listed in the baseline's explicit migrated list — meaning its body follows the thin-body doctrine (trigger + lane scaling, one flow table, load-bearing invariants, reference routing table; all else lives in references). Membership is an explicit editorial act, never inferred from size: an unmigrated skill can be small and still legitimately carry citations.
- **Grandfathered**: an unmigrated skill whose budget exceeds 8,192 bytes; it must carry a justification note ("pending migration"). Growth is blocked immediately; the note dies when the skill migrates.
- **Provenance citation**: an inline decision-ID/plan-name/hardening-label reference. Migrated bodies state rules bare; the rule-to-decision map lives in that skill's provenance reference, one hop away.

## Behaviors & Operations

- Any body exceeding its recorded budget fails the chain, naming skill, size, budget, and delta — "pay for new text by removing text."
- A skill with no recorded budget fails loud (new skills are seeded deliberately); a missing baseline file fails loud, never silently.
- A budget above 8,192 with no justification note fails.
- A provenance citation in a migrated body fails; the same citation in an unmigrated body passes.
- The selftest proves each of these bites on fixtures; a live run over the repo must report zero findings.

## Actors & Access

Agents editing instruction text, and the verify chain itself. The baseline file is edited only through the maintenance mode or an explicitly-scoped migration change.

## Business Rules

- **Bytes, not lines.** The metered unit is what the platform actually loads.
- **Ratchet, one-in-one-out.** Adding a rule to a body means paying for it by trimming that same body; over budget means trim first. New learnings land in the knowledge bundle or the owning skill's references by default; a body edit is reserved for load-bearing invariants (the regrowth law, stated in the skill-authoring and self-improvement disciplines).
- **Migrated is a list, not an inference.** Judged by explicit membership; size-based inference was proven wrong in validation (a small unmigrated skill carried 11 legitimate citations and would have turned the chain red).
- **Narrow supersession.** The blocking behavior of this one gate (budget + provenance checks) supersedes the older "instruction text gets a lint, not a suite; nothing blocks" law for these checks only; anchor-integrity and ordered-list checks remain advisory in the lint.

## Edge Cases Settled

- Body exactly at 8,192: passes; one byte over: fails with delta 1.
- Baseline maintenance run twice: second run reports nothing to lower; a grown entry is never raised by it (the one-time drift correction at fence introduction was a deliberate direct baseline edit, recorded as a validation decision).
- Budget exactly 8,192 with the skill in the migrated list: provenance check applies.

## Open Gaps

- Nine skills remain grandfathered pending their thin-body migration (tracked as the wave-2 backlog item); until then their budgets block growth but exceed the 8,192 target.
- The provenance pattern is deliberately tunable — mechanical, not perfect; false-negative citations in exotic formats are accepted.

## Pointers (implementation)

- Fence: `scripts/skill_budget_fence.mjs` (selftest + live modes, `--update-baseline`).
- Baseline: `scripts/skill-body-budget.json` (budgets, migrated[], notes).
- Chain registration: `scripts/run_verify.mjs` EXTRA_SUITES.
- Advisory sibling: `scripts/skill_lint.mjs` (anchor integrity, ordered lists).
- Regrowth law text: `skills/bee-writing-skills/SKILL.md` (pressure-test checklist), `skills/bee-evolving/SKILL.md` (learning placement).
- Feature history: `docs/history/skill-token-diet/` (CONTEXT D1-D8, plan, per-cell reports).
