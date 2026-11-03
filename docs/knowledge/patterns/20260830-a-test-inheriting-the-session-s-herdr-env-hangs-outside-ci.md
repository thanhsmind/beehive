---
type: bee.pattern
title: A test that inherits the session's HERDR env can block on a real socket outside CI
description: hooks::tests::without_the_marker_the_same_invocation_reaches_dispatch inherited the session's HERDR_ENV/HERDR_SOCKET_PATH and blocked reading the real herdr socket, hanging a release test run for 45 minutes; CI has no HERDR env so it passes there
tags: [tests, herding, ci, environment-leak]
timestamp: 2026-08-30
bee:
  id: pattern-20260830-herdr-env-leak-hangs-local-test
  lifecycle: active
  areas: [bee-herding]
  sources: ["release test run, 2026-08-30 — hung 45 minutes on hooks::tests::without_the_marker_the_same_invocation_reaches_dispatch"]
  polarity: pitfall
  evidence: observed
---

# A test that inherits HERDR_ENV can hang outside CI

`hooks::tests::without_the_marker_the_same_invocation_reaches_dispatch`
inherited the session's `HERDR_ENV`/`HERDR_SOCKET_PATH` environment
variables and blocked reading the real herdr socket instead of exercising
its own isolated fixture. A release test run hung 45 minutes on this one
test. CI never reproduces it — CI has no `HERDR_*` env, so the test passes
there — which is exactly what let it ship unnoticed until a local release
run inherited a live session's environment.

## Fix direction (not yet implemented)

The test (or the suite harness / `release.sh`) must scrub `HERDR_*` env
before running, so a test's isolation does not depend on the ambient shell
happening to be clean. Filed as a defect observation only.
