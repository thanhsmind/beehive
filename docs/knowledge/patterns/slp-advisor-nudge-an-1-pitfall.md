---
type: bee.pattern
title: Widening a shared vocabulary trips the guard that pins the prompt to the verb
description: Widening a closed set like KNOWN_SIGNALS in the verb requires the same-change prompt update the guard is built to demand
timestamp: 2026-08-29
bee:
  id: slp-advisor-nudge-an-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/archive/slp-advisor-nudge/an-1.json]
  polarity: pitfall
---

# Widening a shared vocabulary trips the guard that pins the prompt to the verb

## What happened

Cell an-1 added two new poor-work signals to the advisor-nudge mailbox kind.
Widening the verb's KNOWN_SIGNALS set alone left the supervisor prompt out of
sync — `control_loop.rs`'s `the_shipped_prompt_pins_the_record_verbs_own_closed_sets`
guard exists exactly to refuse that gap, so the cell went red until the
prompt learned the two new signal names in the same change. Editing the
prompt then required `bee dev regen`, which re-renders every skill
projection (.claude/.opencode/.agents/.claude-plugin/.codex-plugin) and the
release manifest — both parity tests went red until regen ran.

## The lesson

A closed-set field with a matching prompt/vocabulary guard is not touched
in isolation: widen the set and update the prompt in the same cell, then run
the regen chain before capping — otherwise the guard (or the parity check)
catches it late instead of the plan catching it early.

## Status

Candidate only. Naming the pattern, generalizing it beyond this cell, and
moving `bee.lifecycle` to `active` are a human or agent decision.
