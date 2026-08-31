---
type: bee.pattern
title: A door's refusal form follows its sibling refusals, not the plan's prose template
description: Mixing a multi-line refusal template into a door whose other refusals are single typed lines reads worse than matching the sibling
timestamp: 2026-08-29
bee:
  id: slp-advisor-nudge-an-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/archive/slp-advisor-nudge/an-3.json]
  polarity: pitfall
---

# A door's refusal form follows its sibling refusals, not the plan's prose template

## What happened

Cell an-3's plan specified a three-line headline/remedy/next refusal shape
for `[WORKTREE_MERGE_ADVISOR_NUDGE_DEBT]`. Every other refusal inside that
same merge-door function is one typed line naming its own remedy
(`bee decisions log --tags advisor-nudge`). The cell shipped the one-line
form instead — mixing two refusal shapes in one door reads worse than
matching the existing sibling, even though the plan asked for the longer
form.

## The lesson

When a plan's prescribed shape for a new refusal disagrees with the shape
every sibling refusal in the same door already uses, match the door, not
the plan — and record the deviation rather than silently picking one.

## Status

Candidate only. Naming the pattern, generalizing it beyond this cell, and
moving `bee.lifecycle` to `active` are a human or agent decision.
