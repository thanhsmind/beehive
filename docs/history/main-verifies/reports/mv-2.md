# mv-2 — Doctrine flip on the instruction layer (D4)

**Status:** [DONE]

## Outcome

Rewrote the worker/orchestrator instruction surfaces for the new verify
doctrine (D4): bee-executing's loop is now read -> reserve -> implement ->
commit -> cap -> release -> report, defaulting to `cells cap
--feature-verify-pending`; bee-swarming drops the routine per-`[DONE]`
verify re-run and the wave-close impacted run; the ONE feature verify at
final-slice close is documented once, in full, in
`swarming-reference.md` ("Feature verify at close, in full") and pointed
to from every other surface. Bugfix authoring law (main-produced repro
red, worker fixes not re-proves) is stated at the authoring surface.
routing-and-contracts.md's lane table, Chaining Contract, Verify Ladder,
and Ship-visibility ticks realigned; provenance rows added to
bee-swarming's provenance.md.

## Verify

`node scripts/skill_budget_fence.mjs && node scripts/skill_lint.mjs && node scripts/okf_instructions_fence.mjs` -> all green (recorded as this cell's verify).
`node packages/bee/tests/test_misc.mjs` -> 116 passed, 1 failed — the failure is the pre-existing wave-barrier vendored-mirror parity check (mv-1's `packages/bee/bee.mjs`/`lib/cells.mjs` edits not yet mirrored to `.bee/bin`, regen deferred to the orchestrator per `regen_obligation_ack`). Every census check (including the ones this cell's prose edits could have broken) passed. Verify recorded on the amended, barrier-excluding command, mirroring the st-1 precedent (`.bee/cells/st-1.json`).

Both SKILL.md bodies net-shrank: bee-executing 9227/10225 bytes, bee-swarming 8172/8187 bytes.

## Files + commit

skills/bee-executing/SKILL.md, skills/bee-executing/references/worker-details.md, skills/bee-swarming/SKILL.md, skills/bee-swarming/references/swarming-reference.md, skills/bee-hive/references/routing-and-contracts.md, skills/bee-swarming/references/provenance.md.
Commit: d174954d.

## Deviations

- Did not create `skills/bee-executing/references/provenance.md` (listed in the cell's declared files): bee-executing is not a migrated skill, so provenance exile (D8) does not apply — no inline-citation problem to fix, nothing to place there.

Full trace/evidence: `.bee/cells/mv-2.json`.
