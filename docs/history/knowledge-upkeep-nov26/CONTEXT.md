# knowledge-upkeep-nov26 — brief

Lane: docs. Class: docs.

## Asked

Run the knowledge upkeep pass (`bee-knowledge-upkeep`) on the bundle.

## Found

`bee upkeep due --kind knowledge` reported a run due: 8 suspect concepts
loaded 5 or more times. No concept had a wrong report and none graded orphan.
The pass picked the five suspect concepts loaded most:
`areas/workflow-state/outcome-scorecard.md`,
`areas/onboarding/skillstation-hub-skills-and-aliases.md`,
`areas/advisor-protocol/blind-lanes-and-the-convergence-dossier.md`,
`areas/verify-pipeline/verify-pipeline-skill-trigger-cases.md`,
`areas/okf-profile/freshness-and-document-homes.md`.

The gather door could not start a read-only reader: every configured role
runs on a Paseo agent with the antigravity provider, and `bee herding run
--read-only` refuses that provider. The leader read each concept against its
own sources and anchors in the session instead.

Two held: `outcome-scorecard.md` (the `run.rs` change is the settle hint,
not the job usage stamp) and `verify-pipeline-skill-trigger-cases.md` (the
fence, budget and eval code did not change; the fixture gained cases for new
skills). Three drifted.

## Will write

Revise in place, inside `docs/knowledge/` only:

- `skillstation-hub-skills-and-aliases.md`: the alias falls back to
  `skill_get` for a relative file, a non-active status, or a missing or
  failed CLI, and names `skillstation connect -g` when neither works.
- `blind-lanes-and-the-convergence-dossier.md`: add the Read-when line; each
  lane has its own model seat (lane-1 to lane-3); the framing-limit backlog
  item is parked; the hat wave builds the plan at the plan step and a
  pre-Lock window critiques a big spec.
- `freshness-and-document-homes.md`: the read-set gate reads CONTEXT.md and
  resolves read-set paths under the feature's granted worktree when one
  exists, else the repo root.

Each picked concept gets `bee knowledge confirm`.

## Not in scope

Product code and the owner-only team config.
