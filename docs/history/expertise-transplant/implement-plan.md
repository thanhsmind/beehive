# Expertise Transplant — Implement Plan

**Feature:** expertise-transplant · **Lane:** standard (docs-only, 8 files) · **Date:** 2026-08-11
**Source of decisions:** docs/history/expertise-transplant/CONTEXT.md (D1–D11)

## What ships

Eight craft gaps from the mattpocock engineering skill set land in bee's
expertise and skill files, rewritten in bee's voice. Gap analysis (three gather
digests, 2026-08-11) governs scope: craft bee already holds is not duplicated (D1).

## Cells — one wave, all parallel (no file overlap)

| Cell | Files | Delivers |
|------|-------|----------|
| et-1 | `.bee/expertise/merges.md` (new), `INDEX.md` | Merge/rebase conflict-resolution craft (D2) — the one area with zero coverage anywhere |
| et-2 | `.bee/expertise/debugging.md` | Feedback-loop-first section: loop ladder, no-loop-no-hypothesis gate, repro-rate for flaky, debug-tag cleanup (D3) |
| et-3 | `.bee/expertise/architecture.md`, `tests.md` | Deletion test, adapter-count seam rule, dependency taxonomy, replace-don't-layer (D4); independent-oracle rule (D6) |
| et-4 | `.bee/expertise/planning.md` | Design-it-twice swarm move (D5); spike craft for the spike lane (D7) |
| et-5 | `skills/bee-hive/references/routing-and-contracts.md`, `skills/bee-shaping/SKILL.md` | Phase-boundary decision tree (D8); Qualify verify-the-claim + concept-dedup (D9) |

All five cells carry `regen_obligation_ack: wave-barrier` — the orchestrator
runs the full regen chain (`bee dev render-skill-trees` → `bee onboard
--repo-root . --apply` → `bee dev release-manifest --write`) once at wave
close, in the wave-close commit, then `bee dev release-manifest --check`.

## Technical design

Additive sections only; each target file's existing voice (routing-table row +
trigger → move → concrete example) is the format contract (D10). Skill-source
edits follow bee-writing-skills discipline; vendored `.claude/skills/` trees
are never hand-edited (D11). No runtime code changes; `commands.test` (cargo
test) guards against accidental breakage and each cell caps through
`bee cells finish`.

## Verification

- Per cell: commands.test green; section in file voice; no duplication of existing craft.
- Wave close: regen chain run, `bee dev release-manifest --check` green, one commit per cell plus wave-close regen commit.

## Rollback

Pure docs — revert the commits; no data, schema, or runtime surface involved.

## Smaller path considered

Single mega-cell rejected (8 files, one reviewer bite too large); dropping et-5
rejected (D8/D9 are the two highest-leverage behavior hooks). Splitting et-3/et-4
further rejected — each is one file-pair with one theme.
