# knowledge-upkeep-nov26b — brief

Lane: docs. Class: docs.

## Asked

Run the knowledge upkeep pass (`bee-knowledge-upkeep`) on the bundle.

## Found

`bee upkeep due --kind knowledge` reported a run due: 3 suspect concepts
loaded 5 or more times. No concept had a wrong report and none graded orphan.
The pass picked the five suspect concepts loaded most:
`areas/onboarding/hook-vendoring-import-closure-completeness-for-repo-local-gu.md`,
`areas/workflow-state/prompt-work-record.md`,
`areas/decision-memory/owner-brief-contract.md`,
`areas/hook-runtime/token-economy-of-injected-context.md`,
`areas/onboarding/host-project-artifacts.md`.

The gather door could not start a read-only reader: every configured role
runs on a Paseo agent with the antigravity provider, and `bee herding run
--read-only` refuses that provider. The leader read each concept against its
own sources and anchors in the session instead.

One held: `owner-brief-contract.md` (labels, word range, enums, forbidden
tokens, bullets, folding and the doctor note match the validator). Four
drifted.

## Will write

Revise in place, inside `docs/knowledge/` only:

- `hook-vendoring-import-closure-completeness-for-repo-local-gu.md`: the
  Node hook set left with R6; onboarding vendors no `.mjs` hook, the legacy
  name list lives in `onboard/templates.rs`, and an apply removes stale
  vendored hooks. Anchors move off `main.rs`.
- `prompt-work-record.md`: a secret-shaped prompt is stored whole as
  `[redacted]`; the title cap, the text cap, the fresh record after `open`;
  `bee work set --status` guards the value, not the transition.
- `token-economy-of-injected-context.md`: the handoff mark history reads
  300000, then 200000, then 120000; the LEADER.md size limit has no test and
  the file is past it.
- `host-project-artifacts.md`: the hub publish skips a skill the hub already
  serves with the same files, and hub mode publishes the hub-served skills.

Add Read-when lines to three concepts. Each picked concept gets
`bee knowledge confirm`.

## Not in scope

Product code, splitting the two concepts past the 300-line limit, and the
bundle-wide strict errors this pass did not touch.
