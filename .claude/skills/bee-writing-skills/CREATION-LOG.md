# Creation Log: bee-writing-skills

## Provenance

Adapted from khuym's `writing-khuym-skills` (itself the TDD-for-skills methodology from obra/superpowers, backed by Meincke et al. 2025, N=28,000 persuasion research). Changes for bee:

- Paths and validation commands retargeted to the bee plugin repo (`node --check` on scripts, manual frontmatter/link verification — bee ships no Python validator).
- Ecosystem naming: skill names carry the `bee-` prefix directly (compound-engineering pattern) so a plain copy into ~/.claude/skills or ~/.codex/skills stays namespaced.
- Dependency-metadata examples use bee's `nodejs-runtime`/helper conventions instead of beads/gkg.
- References carried over: `pressure-test-template.md` (7 pressure types + scenario template), `creation-log-template.md` (this file's own template).

The Iron Law, RED/GREEN/REFACTOR mapping, description trap, persuasion-principles table, meta-testing technique, rationalization table, and red flags are kept substantively verbatim — they are the pressure-tested core of the upstream skill.

## Pressure testing: PENDING (Iron Law debt before 1.0)

This v0.1 adaptation inherits its bulletproofing from the khuym/superpowers lineage but has NOT yet been pressure-tested in bee form. Scheduled RED scenarios (from docs/04-skills-spec.md):

1. **"It's just a small addition"** — agent edits an existing bee skill under time pressure and argues a one-section change doesn't need a baseline test.
2. **"Academic questions passed"** — agent quizzes a subagent about the skill content and claims that substitutes for application-scenario testing.
3. **"I already know what agents will do"** — agent writes the skill first, promising to test after, citing prior experience with similar skills.

Per the Iron Law, any edit to this skill (including this adaptation) must run these scenarios before 1.0. Until then, treat this skill as provisionally trusted on upstream evidence.
