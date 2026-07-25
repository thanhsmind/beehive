# packages-engine-move-2 — report

**[DONE]**

SKILL.md and `routing-and-contracts.md` onboarding steps now name the
`packages/bee/scripts` entrypoint from the bee source root. Swept the
remaining stale engine-path refs in `docs/02-architecture.md` (tree
diagram + D9 renderer reference), `docs/07-contracts.md`,
`docs/specs/onboarding.md`, `docs/specs/reading-map.md` (path corrections
inside existing lines only). Ran the full regen chain
(`render_plugin_skill_trees`, self-onboard `--apply`, `release_manifest
--write`, `impact_registry --write`); projection-shrink proof confirmed —
`onboard_bee.mjs --json` returns `up_to_date`, and both `.claude/skills`
and `.agents/skills` projections of `bee-hive` carry no `scripts/`
directory. `docs/knowledge` left untouched — scribing owns the 12-concept
sweep at feature close (C10).

Files touched: see `.bee/cells/packages-engine-move-2.json`
(`trace.files_changed`) for the complete list — SKILL.md +
routing-and-contracts.md, the 5 docs files, and the 4 regenerated/synced
projections (`.claude`, `.agents`, `.claude-plugin`, `.codex-plugin`).

Commit: `ba73e70` — one commit, cell id in the message. Also folded in
cell 1's untracked report file (`packages-engine-move-1.md`), which was
feature history left uncommitted by the prior cell.

Full trace and verification evidence: `.bee/cells/packages-engine-move-2.json`.

No Advisor Consults on this claim.
