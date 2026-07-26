# multisession-native-24

[DONE] — legacy `.bee/HANDOFF.json` writer set reclassified and pinned by a
red-first-proven grep-audit test (`rebuildHandoffProjection` sole projection
writer; `writeHandoff`/`adoptHandoff` retained one more release as the dated
C1 no-workflow-records fallback, per advisor condition E); msn-21's dead
lane-cache branches confirmed already retired in its own cell (nothing left
to delete); `docs/knowledge/areas/workflow-state/` entry point rewritten to
present the workflow record as THE state model; `AGENTS.md` handoff wording
updated to name the mailbox+projection model, within its 20 KiB budget.

Files touched: `skills/bee-hive/templates/lib/state.mjs`,
`skills/bee-hive/templates/lib/state-projection.mjs`,
`skills/bee-hive/templates/tests/test_state.mjs`,
`skills/bee-hive/templates/AGENTS.block.md`, `AGENTS.md`,
`docs/knowledge/areas/workflow-state/overview.md`,
`docs/knowledge/areas/workflow-state/index.md` (generated, regenerated via
`bee knowledge index`), `docs/history/codex-harness-hardening/release-manifest.json`,
`.bee/onboarding.json`, plus the mirrored plugin skill trees
(`.agents/`, `.claude/`, `.claude-plugin/`, `.codex-plugin/`).

Full trace/evidence: `.bee/cells/multisession-native-24.json`.
