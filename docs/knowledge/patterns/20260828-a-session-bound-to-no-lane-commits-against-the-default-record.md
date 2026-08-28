---
type: bee.pattern
title: A session bound to no lane commits against the default record
description: A session bound to no lane commits against the default record
tags: [lanes, commit-gate, multi-session, guards]
timestamp: 2026-08-28
bee:
  id: pattern-20260828-a-session-bound-to-no-lane-commits-against-the-default-record
  lifecycle: active
  areas: [hook-runtime, workflow-state]
  sources: [".bee/cells/archive/slp-supervisor-heartbeat/sup-5.json", ".bee/cells/archive/slp-dissent-stop-and-ask/sd-4.json"]
  polarity: pitfall
  critical: false
  evidence: present
---

# A Session Bound to No Lane Commits Against the Default Record

When a worker's session is bound to no lane, or a concurrent lane resets the
shared default record, the commit-time gate reads that default record instead
of the worker's own — and the default says idle, so a legitimate mid-unit
commit is refused.

This hit two features in two days: once because the session was never bound to
a lane, once because a sibling lane left the default record at idle while the
worker's own lane said otherwise.

The worker's correct move, taken both times, is to stop and report — never to
work around the guard. The orchestrator's correct move is to bind the session
or restore the record, re-run the verification green, and commit. Until the
gate resolves the worker's own lane record first in this path, read a mid-unit
idle refusal in a multi-lane session as this conflict, not as a broken guard.
