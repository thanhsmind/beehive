---
type: bee.pattern
title: Never poll scratchpad files to wait for your own background subagents
description: Never poll scratchpad files to wait for your own background subagents
tags: [failure, swarming, review, background-agents, tokens, polling]
timestamp: 2026-07-11
bee:
  id: pattern-20260711-never-poll-scratchpad-files-to-wait-for-your
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT22", "original feature: session-observation (anphabe-gogl review run)", docs/history/learnings/20260711-subagent-poll-waste.md]
  polarity: pitfall
  critical: false
---

# Never poll scratchpad files to wait for your own background subagents

A review orchestrator spawned its 6 reviewers via a self-written `run-wave.sh` (prompt files
+ headless CLI processes writing `out_*.md`) instead of the Agent tool — shell processes are
invisible to the harness, so it then had to poll the files with an `ls` + `wc -c` loop
repeating six ~110-char absolute paths per iteration (~300–400 tokens each, all 0 bytes).
The Agent tool already provides everything the script rebuilt: parallel dispatch, isolated
context, and completion re-invoking the orchestrator with the final message as the report
(swarming-reference collection contract). Dispatch subagents only through the Agent tool;
never poll for agents you dispatched; polling is only for external state the harness cannot
see (CI, deploys), and even then emit ONE compact line (a count), never per-file paths.

**Full entry:** docs/history/learnings/20260711-subagent-poll-waste.md
