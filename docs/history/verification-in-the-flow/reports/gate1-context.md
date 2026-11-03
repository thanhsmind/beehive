# Gate 1 — Context approval (auto, bypass full)

**Feature:** verification-in-the-flow · **Date:** 2026-09-02 · **Actor:** auto (`gate_bypass: full`)

## What was decided

Seven decisions, all settled with the user in an interactive session before Lock
ran. Lock originated nothing.

- D1 `d0e3c3a0` — fixed skill name `verify-app`, not `bee-` prefixed.
- D2 `28140420` — source root flattens to `.bee/verify/`.
- D3 `65592f3f` — `bee onboard` two-way branch on `.bee/verify/SKILL.md`.
- D4 `c93a6948` — feature map becomes read-first, two tiers.
- D5 `036e8a79` — drive rides the cap proof line as `green:live`.
- D6 `2a8eac15` — no composition into `commands.test`; supersedes `d79baa77`.
- D7 `2effbe54` — bee regenerates its own `verify-bee` into this shape.

## Evidence behind the two load-bearing refusals

- **`bee-` prefix refused.** `packages/bee-rs/crates/bee/src/onboard/skills.rs:954`,
  test `foreign_bee_skill_in_target_is_removed_but_non_bee_is_untouchable`: a
  `bee-legacy` directory present in a target skill home and absent from bee's
  source yields `remove_skill`; a `my-own-skill` directory is untouched. The
  ownership axiom is stated at `plan.rs:359`.
- **`commands.test` composition reversed.** `docs/history/verification-ships-to-hosts/CONTEXT.md`
  § "Consequence recorded against D2" already records that composition partly
  reverses `ci-owned-verify` D5/D6, that the agent raised it before the pick, and
  that the user chose composition anyway. D6 reverses that pick with the same
  reasoning now accepted.

## Recommendation taken

<!-- bee:not-a-deferral: a gate report records what was presented at that moment; both questions it names were answered at the plan step and CONTEXT.md now marks them so -->
Approve. The decisions are the user's own words from this session, every one
carries its rationale, and the two reversals each cite verifiable evidence
rather than preference. Two questions are deferred to planning and neither
blocks shaping.
<!-- /bee:not-a-deferral -->

## Known limit recorded, not resolved

<!-- bee:not-a-deferral: the condition is registered as trigger `a-host-repo-needs-a-second-verification-s__9f4f90f0`; this line is the gate-time record of it -->
A host repo with two separate products cannot carry two `verify-app`. Recorded
in CONTEXT.md § Known Constraints as a later decision, not a blocker.
<!-- /bee:not-a-deferral -->
