---
type: bee.pattern
title: A listing verb aimed at one item can answer for its whole container
description: "tmux's list-panes -t <pane> lists every pane in that pane's WINDOW, not the one pane named; a caller that trusts 'one row back' silently reads a sibling's row. Any query-by-target API can have this shape — a target that is also a member of a container may select the container. Filter the answer by the target's own id, never by position."
tags: [bee-herding, tmux, transport, query, external-tool]
timestamp: 2026-08-23
bee:
  id: pattern-20260823-a-listing-verb-aimed-at-one-item-answers-for-its-container
  lifecycle: active
  sources: ["tmux-herding-transport cell tht-3 (list-panes -t <pane> answered with every pane of the window; the transport now asks for the #{pane_id} field and matches its own pane id in the rows, never the first row)"]
  polarity: pitfall
  critical: false
---

# A listing verb aimed at one item can answer for its container

The spec half is already a herding rule: a pane read on tmux asks the tool
for the pane id field and matches its own id in the rows
(`areas/bee-herding/the-run-verb-and-worker-outcomes.md`, Transport). The
general rule is the part worth keeping.

Some listing verbs take a target and answer for the container the target
belongs to: "list panes for this pane" lists the window; "list members
for this member" may list the group. A caller that reads the first row
back, or counts on exactly one row, silently takes a sibling's state as its
own — the same shape as the prompt-echo receipt trap, one layer down: the
answer looks right exactly when it is wrong.

When a query names one item, filter the result by that item's own identity
before reading anything from it, and make the test hand back a multi-row
answer with the wanted row not first.
