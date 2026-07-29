# dch-4 — Reconcile the portable-paths scan set with what exists on disk

**Status:** `[DONE]`

## Outcome

`scripts/tests/test_portable_paths.mjs` built its scan set from `git ls-files -z`
(the index) only, no existence filter — so an untracked-but-real file with an
illegal Windows character sailed through green (the quiet direction of the
defect, since this suite's loop is string-only and never touches the
filesystem, so it never crashed the way `test_doctrine_parity.mjs` did).

Fix: added the existence filter (`test_doctrine_parity.mjs:136`'s pattern) and
unioned the index listing with `git status --porcelain`, parsed the same way
as `statusPorcelainFiles()` at `scripts/run_verify.mjs:868-885` (skip the
2-char status code, resolve a rename's destination, unquote) — no second
git-listing approach invented. The illegal-char/reserved-name/trailing-dot
assertions themselves are untouched.

## Proof (real cases, no probe script authored)

- **Before (current code, captured pre-fix):** created a real untracked file
  `scripts/tests/tmp-dch4-proof:bad.txt` (illegal `:`). Ran the unmodified
  script: `PASS portable-paths: 2792 tracked paths are Windows-safe`, exit 0
  — silently missed it.
- **After (same fixture still present):** `FAIL portable-paths: 1 tracked
  path(s) cannot be checked out on Windows — scripts/tests/tmp-dch4-proof:bad.txt
  -> illegal character on Windows: :`, exit 1 — now caught. Fixture then
  removed.
- **Indexed-but-deleted-from-disk (isolated nested repo under `.bee/tmp/`,
  never touching the shared index):** committed a file named
  `reserved:name.txt` (illegal `:`), deleted it from disk, ran the fixed
  script from that repo: `PASS portable-paths: 0 paths are Windows-safe`,
  exit 0 — correctly excluded (would have failed on the illegal char if
  wrongly included). Proof directory removed after.

## Verify

`node scripts/tests/test_portable_paths.mjs` -> `PASS portable-paths: 2800
paths are Windows-safe`, exit 0. Recorded via `cells verify`.

## Files + commit

- `scripts/tests/test_portable_paths.mjs`
- Commit `5ddee1a854462833c2f776f7d078ec7216e0e508` — "fix(dch-4): reconcile
  portable-paths scan set with disk reality" (landed via temp-index
  compare-and-swap per the live-sibling-worker guard; shared index synced for
  this one path only, no other worker's staged files touched)

Full trace: `.bee/cells/dch-4.json`
