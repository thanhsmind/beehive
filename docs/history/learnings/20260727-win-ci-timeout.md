# win-ci-timeout — learnings (2026-07-27)

Feature: `win-ci-timeout` (tiny, 1 cell: win-1). Windows portable-suites CI job
was red with exit 124 at exactly 300000ms on spawn-heavy git suites while the
Node-22 job on the same commit ran green.

## What settled

- **Platform slowness is not suite logic.** A TIMEOUT at exactly the configured
  ceiling on one platform, green elsewhere on the same commit, is a
  capacity/pacing signal — fix the ceiling for that platform, never the suite.
- **win-ci-timeout D1** (decision e2373374): the Windows portable-suites job
  sets `BEE_VERIFY_SUITE_TIMEOUT_MS=600000`; the 300s default stays everywhere
  else; job-level `timeout-minutes: 30` still bounds the whole run.

## Knowledge sync

Merged into `docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md`
(hung-suite timeout section) — scribing run stamped 2026-07-27T17:18:33.585Z.
