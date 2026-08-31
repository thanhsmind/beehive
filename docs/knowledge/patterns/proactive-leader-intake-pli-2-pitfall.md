---
type: bee.pattern
title: A section title that names a retired mechanism should be renamed the moment the mechanism retires
description: A live skill section still headlined by a dispatch shape no code path reaches anymore misleads every reader who opens it
timestamp: 2026-08-30
bee:
  id: proactive-leader-intake-pli-2-pitfall
  lifecycle: draft
  areas: [advisor-protocol, doctrine-layer]
  sources: [.bee/cells/pli-2.json]
  polarity: pitfall
---

# A section title that names a retired mechanism should be renamed the moment the mechanism retires

## What happened

Cell pli-2 retired the "review wave" (a merged-reviewer dispatch) in favor
of the plan-step hat wave. The section heading still read "Review wave"
even though no live skill file cited that dispatch anymore, so the cell
renamed it to "Plan check — the hat wave." The old prompt block written for
the merged-reviewer dispatch was replaced with the two mandates as prose
plus the synthesis shape, since the five dimensions and cold-pickup flags
it carried live only there — a fact the plan had gotten wrong.

## The lesson

When a cell retires the mechanism a section describes, sweep that section's
own heading and any embedded prompt text in the same change — a title or
prompt block left pointing at the retired shape outlives the mechanism and
misleads the next reader, even after every other reference is updated.

## Status

Candidate only. Naming the pattern, generalizing it beyond this cell, and
moving `bee.lifecycle` to `active` are a human or agent decision.
