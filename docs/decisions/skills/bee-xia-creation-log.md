# Creation Log: bee-xia

## Source Material

- khuym `skills/xia` (SKILL.md, references/xia-protocol.md, references/research-brief-template.md) — the proven original: research-first feature discovery, evidence labels, anti-reinvention ladder.
- Decision [0005](../../docs/decisions/0005-research-protocol.md) — owner-approved addition via the 0002 decision gate; workflow gap: standalone research with no feature underway (routing previously sent "research task" to `bee-planning`, which requires feature context and gates).

## Extraction Decisions

**Kept from khuym xia (wholesale):** four-step order (stack ledger → local reuse → upstream → docs), evidence labels `Local/Upstream/Docs/Inference`, recommendation ladder with stated-rejection rule, version-awareness ("local behavior beats docs"), ask-only-when-it-matters criteria, research-brief structure, red flags.

**Changed for bee:**
- Hard MCP dependencies (Exa, DeepWiki) → capability-registry entries (`web-docs-search`, `upstream-pattern-research`) with documented fallbacks; absence degrades honestly (claims become `Inference` → proof obligations for `bee-validating`), never blocks.
- khuym's "no code before the brief" Hard Gate → dropped as redundant: bee's Gate 3 + write-guard hook already enforce it mechanically. Replaced with "research only" scope gate.
- Dual output contract: in-chain runs merge into `approach.md` (unified-artifact rule, CE lesson — no second canonical doc per feature); standalone runs write `docs/history/research/<topic-slug>.md`.
- Depth `Quick/Standard/Deep` explicitly mapped to planning's L1/L2/L3.
- Locked-decision guard added (D-ID contradictions noted, never auto-swapped) — scope-reduction prohibition inherited from planning.
- Spike boundary added: "writing code to try it" mid-research is a red flag routing to `bee-validating`.

**Rejected:** separate brief file for in-chain runs; `openai.yaml` agent config (bee spawns via runtime-default agents); treating DeepWiki gaps as skip permission.

## Iron Law Status

**RED/GREEN/REFACTOR: not yet run in bee form.** Inherits pressure-testing lineage from khuym's xia (see its CREATION-LOG and references/pressure-scenarios.md), but bee-form scenarios (in-chain merge, capability fallbacks, D-ID guard) are untested. Same pre-1.0 debt class as the v0.1 skills — to be cleared via `bee-writing-skills` during dogfooding.
