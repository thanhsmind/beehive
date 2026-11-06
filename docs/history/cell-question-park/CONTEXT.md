# Cell question park — Context

**Feature slug:** cell-question-park
**Date:** 2026-11-06
**Scope:** Standard (cell state, new verbs, close and merge refusal)
**Domain types:** CALL, RUN

## What was asked

The user asked on 2026-11-06 to build part B of the forgent adoption work
("Làm A, xong rồi tới B"). This feature is item 2 of
`docs/history/research/forgent-xia.md`: a per-item park on a human question.

## Why

Today one open product question ends the leader's turn and stalls the whole
lane. `waiting_on` is one mark per session, and the cell state `blocked`
already means "cannot proceed" (a swept claim, `bee cells block`, a blocker
dissent), with its own release paths. Nothing in dispatch reads `waiting_on`; the
stall is doctrine. Forgent parks only the item that has the question, and
sibling items keep running.

## Locked Decisions

The decisions are agent named guesses under `gate_bypass=full`; the user was
not asked. Cite them; never reinterpret them.

| ID | Decision | Store id |
|---|---|---|
| D1 | A worker with a product question it cannot answer parks only its own cell: the cell takes a parked (awaiting-human) state that carries the question text; it leaves the ready set, its dependents wait, and every sibling cell stays schedulable. | b7b5b3c7 |
| D2 | An answer to a parked cell is given through one bee verb; it logs the answer as a decision cited by the cell and returns the cell to open, so the next wave or claim-next picks it up with the answer in its brief. | 8ed6f4a2 |
| D3 | The leader sets the session waiting_on question mark only when no ready cell remains in the lane; while ready cells remain it keeps working and lists open parked questions in its reply and in the supervisor what-needs-you view. | e2e0e77e |
| D4 | bee close and bee worktree merge refuse a feature that still has a parked cell, naming the cell and its question. | 2fe6f2a3 |

## Rejected

- A lane-wide `waiting_on` mark on any question: it is the stall this
  feature removes (D1).

## Out of scope

- A per-cell attempt limit (forgent-xia item 6).
- The content check on owner questions (forgent-xia item 5).
