---
type: bee.pattern
title: A test name that states an old invariant is itself stale once the behavior changes
description: Moving a side effect earlier in a flow can invalidate the premise a passing test's own name asserts, not just its assertions
timestamp: 2026-08-30
bee:
  id: usage-in-bee-store-uibs-move-usage-record-pitfall
  lifecycle: draft
  sources: [.bee/cells/uibs-move-usage-record.json]
  polarity: pitfall
---

# A test name that states an old invariant is itself stale once the behavior changes

## What happened

Cell uibs-move-usage-record made a green close always write
`.bee/usage/<feature>.json` before the close finishes. An existing test,
`clean_store_green_close_reports_reason_clean`, asserted `.bee` is clean at
that point — which is no longer true, since the record write itself now
touches `.bee`. The cell renamed it to
`green_close_commits_the_usage_record_it_just_wrote` and moved the
"reason clean" assertion one step later, rather than patching the old
assertion in place under its old, now-false name. A sibling test asserting
"only dirty .bee paths get committed" needed widening to expect the new
usage path alongside the existing config path, for the same reason.

## The lesson

When a behavior change moves or adds a side effect, check every test whose
own NAME encodes the premise the change is invalidating — a test can keep
passing with a stale name if its assertions get patched without renaming
what the name promises, which leaves the suite lying about what it proves.

## Status

Candidate only. Naming the pattern, generalizing it beyond this cell, and
moving `bee.lifecycle` to `active` are a human or agent decision.
