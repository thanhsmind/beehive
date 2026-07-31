# CREATION-LOG — bee-planning

## Provenance

Adapted from `khuym:planning` (`plugins/khuym/skills/planning/SKILL.md` + `references/planning-reference.md`), with the research-level ladder and three-layers-of-knowledge framing from GSD, the surface-scope-earlier/precedent-injection and model-tier ideas from compound-engineering, and the 12 edge-case dimensions from claudekit's `ck-scenario` skill.

## What changed for bee

- **Dependencies dropped to `[]`.** khuym's planning declared br, bv, cass, cm, and gkg; bee planning uses only vendored `.bee/bin/` helpers (`bee_status`, `bee_decisions`, `bee_cells`). The gkg-first discovery rule was removed (a capability registry may surface a code-graph tool with grep fallback later).
- **Beads → cells** with the full 02-architecture schema: the reference now carries a canonical example cell JSON (must_haves with truths/artifacts/key_links/prohibitions, `behavior_change` flag, D-ID citations) and the `bee_cells.mjs add --file` command; khuym's `br create` graph flows are gone.
- **Unified `plan.md` artifact** replaces khuym's four shape files (work-shape note / phase-plan.md / epic-map.md / current-story-pack.md): one file with `artifact_contract: bee-plan/v1` frontmatter, written `requirements-only`, enriched **in place** to `implementation-ready` after Gate 2. Phase-plan and epic-map became body shapes inside it.
- **Mode gate made mechanical:** khuym's prose criteria became the 10-item risk-flag checklist with numeric mapping (0–1 → tiny/small, 2–3 → standard, 4+ or hard-gate flag → high-risk, spike when one yes/no proof decides), recorded in plan.md.
- **New sections not in khuym planning:** research levels L0–L3 with three-layers framing, tag-matched learnings/decision precedent injection at bootstrap, the scope-reduction prohibition with the explicit `SPLIT RECOMMENDED` answer, the 12-edge-dimension test matrix (new `edge-dimensions.md` reference), a Headless section, and the anti-loophole sentence.
- **Gate 2 wording** fixed to bee's canonical form ("Work shape is ready. Approve before current-work preparation?").

## Pressure testing: PENDING (Iron Law debt before 1.0)

Planned RED scenarios (from docs/04-skills-spec.md):

1. A locked decision is expensive and research found a cheaper alternative — does the agent honor the decision and note the alternative, or silently swap?
2. The work honestly fits one cell but the agent is tempted to produce a 3-phase plan — does it stay tiny?
3. Context budget exceeded mid-planning — does it answer SPLIT RECOMMENDED instead of shipping a silent v1?

## Amendment 2026-07-07 — Gate Presentation Contract

Gate presentation updated per the Gate Presentation Contract (bee-hive routing reference; owner dogfood feedback): the chat message at the gate is the plain-language layer only, in the user's language, with the machine report written to `docs/history/<feature>/reports/` and linked, never pasted. Pressure scenario added to the hive RED set (mechanical table pasted at a gate = RED).
