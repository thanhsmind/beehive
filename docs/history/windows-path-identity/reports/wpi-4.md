# wpi-4 — what the Windows run actually said

Run `30226423174`, workflow `Windows`, head `d41dfd40` (the pushed fix). Conclusion: **failure**. `install-ps1-syntax` passed; both `portable-suites` shards failed.

This is the acceptance evidence the three preceding cells could not produce from a POSIX machine. It confirms one claim, refutes another, and exposes two things nobody had seen.

## Confirmed fixed

**`packages/bee/tests/test_cli_cells.mjs` no longer appears in the failed list.** It failed on 2026-07-24, -25 and -26; it passes on this run. wpi-2's site-1 correction — comparing against the forward-slash form the version-control tool actually emits, instead of a runtime-built join — is proven on the platform it was written for.

## Refuted

**The gitdir identity fix did not clear the merge-queue chain.** `test_msn_invariants.mjs` still fails invariant 12 with the same signature as before:

```
FAIL LOUD: underlying suite "scripts/tests/test_worktree_merge_queue.mjs" did not pass as a whole (exit 1)
Error: waitForFile: timed out after 20000ms waiting for C:\Users\RUNNER~1\AppData\Local\Temp\bee-wt-merge-mq-.../serialize/started-a.marker
```

The diagnosis held that a linked worktree failing to resolve was what stopped the merge from reaching its verify child. wpi-1 fixed that resolution and proved it locally, and the chain still times out. So either the resolution was not the (only) cause, or a second failure sits between the fixed resolution and the marker write. The honest statement is that this cause is **not yet diagnosed**, and the earlier confidence was misplaced.

## Newly exposed

**1. Short-name aliasing defeats the comparison when the path does not exist.**

```
FAIL  bee herding status --json on a repo with no marker reports {enabled:false, marker, main_root}
      expected marker C:\Users\RUNNER~1\AppData\Local\Temp\...\.bee\tmp\bee-herding.enable
      got      marker C:\Users\runneradmin\AppData\Local\Temp\...\.bee\tmp\bee-herding.enable
```

Both spellings name the same file. `canonicalPathsEqual` resolves aliasing through filesystem identity — but only when both paths exist. Here the marker file is deliberately absent (that is the case under test), so the comparison falls back to string equality, and a string comparison cannot fold `RUNNER~1` into `runneradmin`.

The fix direction is to canonicalize the **nearest existing ancestor** and rejoin the non-existent remainder, so a path that does not exist yet still gets its existing prefix resolved. The module already computes a nearest-existing-ancestor for its case probe; the comparison does not use it.

**2. wpi-2's own red-first guard encodes a single-platform assumption — the exact bug it was written to fix.**

```
FAIL  RED-FIRST REGRESSION GUARD, separator ... rejected too by canonicalPathsEqual with
      NO platformPath injected (the POSIX-ambient control — a bare backslash is DATA on
      this platform ...)
```

That control asserts the ambient comparison **rejects** a win32-shaped rendering. On POSIX it does, correctly. On Windows the ambient comparison **accepts** it — also correctly, because there a backslash really is a separator. The assertion is true only on the platform it was written on.

This is worth stating plainly: the guard was added to prove a fix for platform-assuming assertions, and is itself a platform-assuming assertion. It needs to branch on the real platform, or assert only under injection.

## What this run does not say

The four standing exclusions were not removed and were not exercised. Nothing here supports or refutes their causes. `packages/bee/tests/test_worktree_store.mjs` also fails, as it did before this work — it is on the same earlier backlog row and is untouched by these cells.

## Scorecard

| Suite | Before | This run |
|---|---|---|
| `test_cli_cells.mjs` | fail | **pass** |
| `test_herding_cli.mjs` | fail | fail — short-name aliasing on a non-existent path, plus the guard's own platform assumption |
| `test_msn_invariants.mjs` | fail | fail — same merge-queue timeout; cause not yet diagnosed |
| `test_worktree_store.mjs` | fail | fail — pre-existing, untouched |
