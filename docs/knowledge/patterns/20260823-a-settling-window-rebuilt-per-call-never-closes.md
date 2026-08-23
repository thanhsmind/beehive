---
type: bee.pattern
title: A settling window rebuilt inside each call never closes when the caller polls in bursts shorter than the window
description: "A 'screen is quiet' window of six seconds was rebuilt from zero on every agent_wait call while the caller polled in 200 ms bursts, so the window could never fill and the ready gate burned its full 60 s ceiling on a pane that had been idle from the start. State that must accumulate across calls lives on the long-lived object (per pane), not in the call frame. A unit test per call shipped green; only a poller-shaped test shows the hang."
tags: [bee-herding, tmux, polling, state, timing]
timestamp: 2026-08-23
bee:
  id: pattern-20260823-a-settling-window-rebuilt-per-call-never-closes
  lifecycle: active
  sources: ["tmux-ready-wait cell trw-1 (the stability window moved from the agent_wait call frame onto the per-pane transport state; dropped on a failed read, a pane close, and a new job into the pane)"]
  polarity: pitfall
  critical: false
---

# A settling window rebuilt per call never closes

A readiness check that wants "N identical reads over at least T seconds"
has to remember reads between calls. Built inside the call, it starts from
zero each time; a caller whose per-call timeout is shorter than T can poll
forever and never see the window fill. Nothing errors — the pane is idle,
every call returns "not yet", and the outer ceiling is what finally fires.

Put accumulating state on the object that outlives the calls (the transport,
keyed by pane), and name the events that reset it: a failed read, a closed
pane, a new worker started into the pane. Then test the way the caller
actually calls — a loop of short waits — not one long wait. The per-call
unit test was green throughout; it could not see the hang.
