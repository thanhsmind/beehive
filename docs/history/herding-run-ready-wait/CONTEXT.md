# Herding Run Ready Wait — Context

**Feature slug:** herding-run-ready-wait
**Date:** 2026-08-20
**Shaping session:** complete (fix-first brief, tiny lane)
**Scope:** Quick

## Feature Boundary

`bee herding run` waits for the started agent to report ready before sending the round-1 brief, because a prompt sent during boot lands in the banner and is lost (live smoke smoke-agy-2: brief printed above the agy banner; agent idle forever after). One file.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | After a successful `agent start`, execute_new polls `herdr agent list` until the job's agent reports ready-for-input (status idle + interactive_ready true when present), bounded by a fixed ready-wait ceiling (default 60s, matching agent start's own timeout); only then sends the brief. Ready-wait exhaustion is a typed spawn failure that keeps the pane | Prompt-vs-boot race observed live; start's own success signal fired before the agent accepted input |
