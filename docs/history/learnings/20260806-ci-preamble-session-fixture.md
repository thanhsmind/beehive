---
date: 2026-08-06
feature: ci-preamble-session-fixture
categories: [verify-pipeline, hook-runtime]
severity: medium
tags: [ci-red, ambient-environment, fixtures, flake-that-is-not-a-flake]
---

# ci-preamble-session-fixture — a fixture that only works where the developer works

## What Happened

CI had been red on one test for days, across several releases, including the
2.2.0 and 2.2.1 tags. The test wrote a session record without a
`last_heartbeat`. Where a session-id environment variable is exported — every
developer machine running under the agent harness — the resolver takes the env
branch and never looks at the record's freshness. Where none is exported —
CI, always — the resolver falls back to "exactly one FRESH session record",
reads the heartbeat-less record as stale, and resolves nothing. The knowledge
bridge then had no bound lane to render, and the assertion blamed the bridge.

One line: stamp the fixture's record with `now_iso()`. Reproduced red locally
by unsetting the two variables, green after, green both ways.

## What Was Learned

**A test that passes only under ambient environment is not flaky — it is
wrong, and it is wrong in the direction that hides.** It goes green exactly
where someone is watching and red exactly where nobody reads the log. The
tell is a fixture that never sets what the code under test reads: here, the
resolver's env chain outranks the store, so the store branch had never once
been exercised on a developer machine.

**A long-standing red teaches the team to read "failure" as "normal".** Four
releases went out over it, this session's included, because the signal had
already been discounted. The cost of leaving one known red is not that one
test — it is that the next real red arrives invisible.

**Reproduce the environment before diagnosing the code.** `env -u
CLAUDE_CODE_SESSION_ID -u BEE_SESSION_ID cargo test` turned a "CI-only
mystery" into a one-line fixture bug in under a minute. Where a test reads
ambient state, the ability to run it *without* that state is part of the
suite.

## Evidence

- Cell `cps-1`, commit `b91c723f`; released as 2.2.2 (`29b774b4`, tag
  `v2.2.2`) — CI, Windows, and Release binaries all green for the first time
  in this stretch.
- The resolver's precedence: `resolve_session_id_no_flag`
  (`verbs/state_group/store.rs:362`) — `BEE_SESSION_ID`, then
  `CLAUDE_CODE_SESSION_ID`, then exactly-one-fresh-session.
