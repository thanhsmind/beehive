# multisession-native-2

[DONE] — split `mergeFeatureWorktree` into P1 (locked stage) / P2 (unlocked verify) / P3 (re-locked fence + commit) so the `worktree-admin` lock is never held across the verify subprocess; P3 re-checks HEAD, MERGE_HEAD, staged-tree identity (`git write-tree`), and grant before ever committing, aborting untouched on any drift.

Files touched: `skills/bee-hive/templates/lib/worktree-store.mjs` (canonical), `.bee/bin/lib/worktree-store.mjs` + plugin mirrors (vendored), `scripts/test_worktree_cli.mjs`, `docs/history/codex-harness-hardening/release-manifest.json`, `scripts/impact-registry.json`, render/onboarding sidecars.

Commit: `b8fc926`.

Full trace/evidence: `.bee/cells/multisession-native-2.json`.
