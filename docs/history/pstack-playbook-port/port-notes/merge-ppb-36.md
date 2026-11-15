# Port notes — pstack verification skills merged into bee-verifying and bee-verify-upkeep, cell ppb-36

Source: pstack skills `create-verification-skill` and `maintain-verification-skill`
(`SKILL.md`, and `references/feature-map-example/` for the first), MIT, copyright
2026 Lauren Tan. `skills/bee-verifying/` and `skills/bee-verify-upkeep/` each got
`LICENSE.pstack`, a verbatim copy of the pstack `LICENSE`. This file is the only
place a pstack-only tool is named.

Rule applied (D5): the bee skill stays bee's. Every bee rule is kept, including the
constant `.bee/verify/verify-app/` source path, the `bash <path>` helper rule, the
fast and full drive contract, the exit-code contract, the `commands.test` rule,
"When a run is due", the Entry points step and the Record the run step, none of
which pstack has. Both descriptions are unchanged, under the 300 budget.

## What came in

- Nothing in either SKILL.md. Each pstack sentence was checked against its bee
  skill, and every rule, step and check is already there.

## Left out because bee already has it

- create-verification-skill: the intro, the five interview questions and the
  broken-checkout rule, the six generated sections (Launch, Doctor, Drive,
  Evidence, Cleanup, Helpers), the feature-map seed with its four H2s, the
  prove-it-once step and the maintenance-loop pointer. All are in
  `bee-verifying` sections 1 to 4 and 6.
- `references/feature-map-example/`: `bee-verifying` already carries the same
  three files, byte for byte.
- maintain-verification-skill: the three outcomes, the edit scope, the product-code
  ban, index hygiene, the source wave, reconcile, the live pass with its three
  invariants, triage, ship or stop and the scratch run notes. All are in
  `bee-verify-upkeep`.

## Left out because bee differs

- pstack's "Locate the target" step, which asks the user to pick among several
  candidate skills: bee fixes the target at `.bee/verify/verify-app/`, so there is
  one candidate. Step 0 already stops and routes to `bee-verifying` when it is
  missing.
- pstack's `.cursor/skills/verify-<app>/` paths, branch and PR wording, and
  `/create-verification-skill` and `/maintain-verification-skill` slash names: bee
  uses the source path, a worktree and one cell, `bee worktree merge` and the
  skill names `bee-verifying` and `bee-verify-upkeep`.
- pstack's helpers line ("executable, invocation shown"): bee's longer rule adds
  the `bash` prefix and the reason for it.
- pstack's frontmatter (`disable-model-invocation` text, descriptions): bee routes
  by its own descriptions.

## Tool swaps

- None. Neither pstack text names a pstack-only tool beyond the `.cursor` path and
  the PR, which map to the bee wording above.

## Not ported

- No pstack step was dropped for lack of a bee counterpart.
- No bee skill or principle was missing for this merge.

## Regen note

`bee dev regen` also re-renders unrelated `.claude/skills/` entries in this
checkout, as in ppb-30 to ppb-35; those stay uncommitted. Only the
`bee-verifying` and `bee-verify-upkeep` directories in each rendered tree, the
render manifests, the onboarding hash and the release manifest are committed.
