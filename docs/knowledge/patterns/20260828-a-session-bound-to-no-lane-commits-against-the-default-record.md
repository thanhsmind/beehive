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
  sources: [".bee/cells/archive/slp-supervisor-heartbeat/sup-5.json", ".bee/cells/archive/slp-dissent-stop-and-ask/sd-4.json", "slp-followup-gaps cell sfg-1 (commit 9809d34e, 2026-08-29 — the claim-derived acting record and the session-binding remedy line)", "decision edd92ac9 (slp-followup-gaps D1/D2)"]
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
or restore the record, re-run the verification green, and commit.

**Narrowed 2026-08-29** (cell sfg-1, decision `edd92ac9`): the gate now does
resolve the worker's own lane record first in this path. A session that has a
record of its own and no lane binding derives its acting record from its OWN
live claim — the claim names a cell, the cell names a feature, and that
feature's lane record answers — before the control-root default record is read
at all. The ordinary shape of this pitfall, one unbound worker holding one
claim, no longer refuses.

Three shapes stay live, because in each of them the derivation gives no answer
and the default record still answers:

- the session holds claims on two or more different features — ambiguous by
  design, so nothing is derived;
- the session holds no claim at all — there is nothing to derive from;
- the claimed feature's lane record is missing, corrupt, or names a different
  feature — the derived path stays silent rather than refusing.

In those three, still read a mid-unit idle refusal in a multi-lane session as
this conflict, not as a broken guard. The refusal itself now helps: when the
default record answered for a session carrying no lane, it names binding the
session, or claiming its cell, as the remedy beside the usual routing advice.
