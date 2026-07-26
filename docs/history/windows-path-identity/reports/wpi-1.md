# wpi-1 — canonical path-identity comparison at the worktree gitdir pointer check

**Status:** [DONE]

**Outcome:** Added `packages/bee/lib/path-identity.mjs` — a canonical path-identity
comparison (`canonicalPathsEqual`) that detects case-fold behaviour per-volume
(never from `process.platform`), treats a zero inode/device as unusable and
falls back to a normalized-string comparison, and caches the volume probe per
root. Wired it into the ONE real comparison site,
`packages/bee/lib/worktree-store.mjs:1110` inside `resolveWorktreeById`
(`:491`'s `readWorktreeGitVerifiedId` left untouched — it returns git's
registry key, not a comparison). Gave `resolveWorktreeById` an injectable
`pathsEqual` option (default `canonicalPathsEqual`) and threaded it through
`mergeFeatureWorktreeStage`/`mergeFeatureWorktree`'s existing options bag, so
the new test drives the real function through its one real call site, not a
copy in isolation. Regenerated and verified the release manifest.

**Files touched:**
- `packages/bee/lib/path-identity.mjs` (new)
- `packages/bee/lib/worktree-store.mjs`
- `scripts/tests/test_path_identity.mjs` (new)
- `docs/history/codex-harness-hardening/release-manifest.json` (regenerated)

Full trace/evidence: `.bee/cells/wpi-1.json`.
