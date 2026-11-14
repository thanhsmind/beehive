# skill-description-trim — locked decisions

The owner asked whether flow skills, which the router calls by name, need
descriptions as long as they were. Every description loads into every
session: 58 skills cost about 4,400 tokens before any work starts.

## Decisions

- **D1.** Every routable skill description (`skills/bee-*` except
  `bee-principle-*`) is at most 300 characters. The test
  `skill_descriptions_fit_the_budget` in
  `packages/bee-rs/crates/bee/tests/skill_triggers.rs` enforces it.
- **D2.** A description is one purpose clause, then the trigger words that
  the fixture briefs need, then a short "Not for" only where a near case
  needs it. `bee-capturing` keeps its self-trigger line. `bee-reviewing`
  keeps its never-automatic boundary.
- **D3.** The paid routing eval must not score below its pre-change
  baseline, and no brief that the baseline passes may score zero. Measured
  with haiku, three runs per brief: the baseline scored 111 and 114 of 119,
  the trimmed descriptions 114, 113 and 110. Four briefs that expect no
  skill score zero on both sides; they stay open in the trigger-cases
  concept.
- **D4.** The work runs as a small lane. It touches about 24 skill files, but
  the edit is one front-matter field, and one eval measures the whole effect.

## Rejected

- `disable-model-invocation` on router-called skills: the model must still
  call them through the Skill tool.
- One-line descriptions for every skill: the fixture briefs show that users
  name flow skills in their own words too.

## Read set

| Knowledge | Why |
|---|---|
| `docs/knowledge/areas/verify-pipeline/verify-pipeline-skill-trigger-cases.md` | The fixture, the fence and the eval this budget joins |

| Code | Why |
|---|---|
| `packages/bee-rs/crates/bee/tests/skill_triggers.rs` | Home of the budget fence |
| `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json` | The briefs each trimmed description must still route |
