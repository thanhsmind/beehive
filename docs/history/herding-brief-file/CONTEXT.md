# Herding Brief File — Context

**Feature slug:** herding-brief-file
**Date:** 2026-08-20
**Shaping session:** complete (fix-first brief, tiny lane)
**Scope:** Quick

## Feature Boundary

The rendered brief is written to `brief-N.txt` inside the job mailbox and the agent receives a ONE-LINE pointer prompt ("Read the file <abs path> and follow its instructions exactly."), because a multi-line prompt is silently dropped by at least one agent kind (agy: live smokes 4/5 lost the brief with the agent idle and ready; a single-line prompt landed instantly — pong test — and the pointer form ran the task to a green result). Applies to the fresh spawn AND --continue rounds.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | render_brief output is persisted as `<mailbox>/brief-N.txt` (atomic write) before any prompt; the prompt sent via agent prompt is the fixed one-line pointer naming that absolute path | Multi-line injected prompts are unreliable per kind; a file + pointer is encoding-proof and matches the mailbox philosophy |
