# cfl-1 — BEE_CHECK_ONLY name filter in the shared check() helper

Status: DONE (capped --feature-verify-pending)

Added a `BEE_CHECK_ONLY` name-pattern filter to `check()` in
`scripts/lib/test-fixture.mjs`: non-matching checks run no body and are
counted/printed as `SKIP`, not `PASS`; matching is case-insensitive
substring by default, `/re/[flags]` is a regex. `printSummaryAndExit`
reports the skip count and filter value when active, and exits non-zero
with a typed message if zero checks matched. Unfiltered runs are
byte-identical to before this cell. Exported `checkOnlyPredicate` for the
22 suites with a local `check()` to adopt later (not converted here).

Files touched:
- scripts/lib/test-fixture.mjs

Full trace/evidence: `.bee/cells/cfl-1.json`
