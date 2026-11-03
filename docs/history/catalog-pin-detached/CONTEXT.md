# Catalog pin for --detached — Context

**Feature slug:** catalog-pin-detached
**Date:** 2026-09-16
**Scope:** Tiny (fix-first)

## What was asked

The user asked for a release. `scripts/release.sh 2.41.0` ran the full suite
before tagging and stopped with nothing changed.

## What was found

`catalog::tests::distinct_flag_vocabulary_is_pinned_so_growth_is_a_decision`
failed: `left: 208, right: 207`. `harness-worktree-relocation` (cell hwr-2)
added `bee worktree merge --detached`, a new flag name, and did not bump the pin.
The cell's scoped proof never ran this test. No existing flag name means "skip
the caller-session relocation check" (`no-lane` picks the default record,
`skip-uat` skips one door, `force` overrides an ownership guard), so the growth
is kept and recorded as decision `cf50327a`.

## What will be done

Bump `PINNED_FLAG_COUNT` to 208 with a comment entry naming `--detached` and
why. One file, no behavior change.

## Locked Decisions

| ID | Store ID | Decision |
|----|----------|----------|
| D1 | `cf50327a-104e-4e44-97b7-d52fd850494b` | Grow the pinned CLI flag vocabulary 207 -> 208 for `bee worktree merge --detached`. |
