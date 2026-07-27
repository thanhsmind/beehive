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

## Rework round

A goal-check judge (opus) returned `NEEDS_REVISION` on the first cap: the
first draft's `normalizeSeparators` folded every literal backslash to
`path.sep` unconditionally, on the false premise that a backslash is illegal
in a POSIX filename — it isn't, and the fold ran before resolve/stat, so a
real directory named `a\b` compared EQUAL to the genuinely different `a/b`
(a new wrong-pointer-accept, not a pre-existing quirk: the reverse pointer
was already backslash-folded, the other side was not).

Fix: separator handling is no longer hand-rolled. `canonicalPathsEqual` now
resolves both sides through an injectable `platformPath` (default the
ambient `node:path`) and relies entirely on the platform's own `resolve` —
`path.win32.resolve` already normalizes `/` to `\` on its own, and
`path.posix.resolve` correctly leaves a literal backslash as data. Also
addressed the judge's three advisory residuals: the case-fold decision now
requires BOTH sides to agree (not just side A); the volume-behaviour cache
is keyed by device number (volume), not by the ancestor's path string; and
the probe prefers a read-only basename-case-flip check before ever falling
back to the write-based marker probe (real gitdir/worktree directories
almost always have letters to flip, so the merge path itself is essentially
never a write in practice). Test-suite hygiene fixed: sibling worktree
directories are now removed alongside `mainRoot` in every fixture.

`scripts/tests/test_path_identity.mjs` grew from 9 to 13 cases: a POSIX
regression guard for the exact blocker, a Windows-shaped fold test via
`path.win32` (Node's own real win32 path semantics), a both-sides-fold
guard, a device-keyed cache test, and a strengthened read-only-probe proof.
Fresh red-first reproduction recorded in the cap trace: reintroducing the
exact pre-rework comparison turns 13/13 green into 11 passed / 2 failed —
exactly the two tests targeting the blocker and residual #1.

Capped with an audited `--override-judge` (the prior `NEEDS_REVISION`
verdict is now stale, superseded by this fix); a fresh goal-check judge pass
is expected on this cap.

Full trace/evidence: `.bee/cells/wpi-1.json`.
