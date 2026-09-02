# Gate 2 — merged shape + execution (auto, bypass full)

**Feature:** verification-in-the-flow · **Date:** 2026-09-02 · **Actor:** auto (`gate_bypass: full`)
**Plan:** `docs/history/verification-in-the-flow/plan.md` (sha256 `8d0e336c…`)
**Advisor consult:** hat wave, 5 seats — digest at `reports/hat-wave-digest.md`, synthesis `1fb5dc3e`
**Plan conflicts:** derived at `plan_rev` 0 — zero candidates

## What will be built

Three slices, split by artifact kind so the serial-skill constraint is paid once.

- **Slice 1 (current) — Rust only.** `bee onboard` stops gating its verification notice on "this repo declares no test command" and starts branching on whether `.bee/verify/verify-app/SKILL.md` exists: absent → the generate notice naming `bee-verifying`, present → a new upkeep notice naming `bee-verify-upkeep`. Six states are covered red-first, including the two legacy-`commands.verify` rows the wave found missing.
- **Slice 2 — every text surface in one serial pass, then one regen.** The two verify skills take the constant name and path and lose the composition sections; `AGENTS.block.md` gains the read-first mention and the fourth proof case; shaping, planning and swarming gain their load points.
- **Slice 3 — the dogfood.** `verify-bee` moves into the source shape, the old twin is removed in the same commit, and one mapped feature is driven for a `green:live` proof.

## Why this size

High-risk on 4 flags. The two that carry it are real: D6 removes proof a shipped door is documented to run, and D1 changes a path contract every host inherits. The wave then removed the third — the renderer rewrite — by retiring D2, and added two the draft had missed: doctrine regen drift is invisible to every existing net, and the whole "no migration needed" argument expires at the next release tag.

## Cost if the shape is wrong

Slice 1 alone is reversible — it is Rust behind unit tests with no host consumers, because `bee-verifying` has never shipped in a release (claim 9). Slice 2 is the expensive one to get wrong: it edits six copies of each skill file through the regen chain, and the risk map's new HIGH row exists because nothing today would catch a body edit that was never regenerated. Slice 3 is a move plus a deletion, recoverable from git.

## Recommendation taken

<!-- bee:not-a-deferral: this sentence states what was NOT deferred; it is the absence of a deferral, not one -->
Approve. Every locked decision has a work item and a proof row; the one open question was answered by the user at this gate; the claims table's blocker and three drifted anchors are fixed; and the wave's ten remaining findings were folded in as settled planning decisions rather than deferred.
<!-- /bee:not-a-deferral -->

## Carried forward, not resolved by this gate

- **The release constraint.** No release tag between now and slice 2's merge, or the plan grows a migration cell and re-gates. No test can enforce this.
- **D4 is an unproven theory** with a named falsifier: after two mapped features, check whether plan risk maps actually cite feature-file gotchas. If not, its shaping tier is reverted.
