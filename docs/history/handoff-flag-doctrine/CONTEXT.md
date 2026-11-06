# Handoff flag doctrine — Context

**Feature slug:** handoff-flag-doctrine
**Date:** 2026-11-06
**Scope:** Small (one phrase in every home of rule agents-context-handoff-65)
**Domain types:** CALL

## What was asked

The backlog row of commit 14b99bbe9: the handoff dispatch must pass
`follow_up_of` plus the new waggledance handoff flag. context-mark-advisory
added `follow_up_of`; this feature adds the flag now that waggledance 0.5.3
(leader-lifecycle) merged it as `handoff: true`.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | A handoff dispatch passes `handoff: true` beside `leader: true` and `follow_up_of` naming the current run; the daemon starts the successor at once and retires the old leader of the family. | 3eddff18 |

## Out of scope

- Any change to when a handoff happens: docs/history/context-mark-advisory D1 and D3 hold.
