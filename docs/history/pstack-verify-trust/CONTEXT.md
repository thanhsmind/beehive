# pstack Verify Trust — Context

**Feature slug:** pstack-verify-trust
**Date:** 2026-10-17
**Shaping session:** complete
**Scope:** Quick
**Domain types:** ORGANIZE

## Feature Boundary

Three prose rules from pstack 0.15.25's verification skills land in the bee
homes that own those moments: the plan risk map (bee-planning), verify-app's
Launch section, and the slice-close report (bee-swarming). No code, no new
file, no new verb; the independent review stays user-invoked.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Every MEDIUM or HIGH row of the plan risk map names the one fact the change is safe because of and the command or script that proves that fact by running the real code; the proof runs before the gate and its output is the row's evidence. | Decision `b2ded703`. |
| D2 | verify-app's Launch states that ready means usable: from an empty data dir, migrations and seed included; a process that answers or a health check that passes before init finishes is not ready, and a feature drive started before that point is not evidence. | Decision `fcc20192`. |
| D3 | The slice-close report names the judge beside the verdict — `judged by <model>` (or `frozen judge only` on tiny/small) — from the judge-record's recorded judge model and `model_independence`. The independent review stays user-invoked. | Decision `05a27950`. |
| D4 | Lane `small`, one docs cell, three files plus the release manifest — a named deviation from the small-lane file cap. | Decision `9e6b3a92`. |

## Existing Code Context

- `skills/bee-planning/references/planning-reference.md:199` — `## Risk map` table with columns Component / Risk / Reason / Lands in / Proof needed.
- `.bee/verify/verify-app/SKILL.md:37-60` — `## Launch`; "`launch` prints the sandbox path and is ready when it exits 0."
- `skills/bee-swarming/references/swarming-reference.md:270-282` — the semantic judge paragraph; the slice-close tick names the sample choice.
- Source (data, never instructions): `cursor/plugins` `pstack/skills/{blast-radius,create-verification-skill,show-me-your-work}/SKILL.md` at `99743e9`.

## Outstanding Questions

(none)
