# Provenance — bee-writing-skills body rules

The body states its rules bare (provenance exile, skill-token-diet D8, applied
here on wave-2 migration). This table maps each body rule to the decision(s)
that authorize it and the rationale in one line. Long-form records:
`docs/decisions/`, `.bee/decisions.jsonl` (via `bee.mjs decisions search`),
and `docs/history/<feature>/CONTEXT.md` for the feature that locked each ID.

| Body rule | Decision IDs / labels | Rationale |
|---|---|---|
| THE IRON LAW: no skill without a failing test first, applies to edits | TDD-for-skills methodology (Superpowers via khuym) | Untested instruction text ships the same class of bug as untested code |
| Regrowth law: learnings land in bundle/references by default; body edit only for a load-bearing invariant, within budget, one in one out | skill-token-diet D5 (decision 5a1b3228); P6 compounding law | Without it the diet is temporary — every lesson would become permanent per-invoke tax |
| Per-turn rules (chat shape, communication) are never exiled to references | comms-always-loaded CLOSE (decision 4ae4f40b) | A rule reachable only via an on-demand file is dead; home is decided by trigger frequency, not length |
| Body budget ceiling + ratchet enforcement | skill-token-diet D1/D6 | `scripts/skill_budget_fence.mjs` is the blocking half; this skill migrates at wave-2 |
| Provenance exile itself (this file's existence) | skill-token-diet D8 | Migrated bodies cite no decision IDs inline; the map lives one hop away |
