# i54-closeout-2 — worker report

Status: [DONE]

Outcome: `run_verify.mjs`'s `runOne()` gets a per-suite wall-clock timeout
(`BEE_VERIFY_SUITE_TIMEOUT_MS`, default 300000ms; `0`/`none` disables) that
kills the suite's WHOLE child process group on expiry — not just the direct
child, so a grandchild the suite spawned can't outlive it either — and reports
a distinct `TIMEOUT` status, never conflated with an ordinary `FAIL`. A
heartbeat (`BEE_VERIFY_HEARTBEAT_MS`, default 30000ms) prints to stderr,
naming whichever suites are still in flight, via an injectable
tracker/interval so a long run never reads as frozen. Hermetic env scrub
(`childEnv()`) and non-timeout pass/fail semantics are untouched (D2).

Files touched:
- scripts/run_verify.mjs
- scripts/test_verify_timeout.mjs (new; auto-discovered, no manual
  registration per PLAN-CHECK W2)

Verify: `node scripts/test_verify_timeout.mjs && node scripts/run_verify.mjs
--only test_verify_timeout` — 16 passed, 0 failed; scoped run picks up the new
suite via discovery. Red-first genuinely proven: same test run against the
pre-change `run_verify.mjs` (via `git stash`) failed all 16 cases. Regression
check on the one pre-existing `runOne()` consumer,
`scripts/test_run_verify_skip_marker.mjs`, stayed green (7/7) — the opts-based
signature change is backward compatible.

Full trace and verification evidence: `.bee/cells/i54-closeout-2.json`.
