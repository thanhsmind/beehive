# Context mark advisory — Context

**Feature slug:** context-mark-advisory
**Date:** 2026-11-06
**Scope:** Small (one hook line, one rule rewritten in every home)
**Domain types:** CALL, RUN

## What was asked

The owner asked on 2026-11-06: "Toi cũng muốn bỏ vụ chạm mốc bộ nhớ này
đi, nên chạy đến khi xong việc người dùng sẽ tự nói cần chuyển hay không,
không nên đang làm rồi ngắt". The context mark must not interrupt work.

The same feature carries the backlog row from commit 14b99bbe9: a handoff
dispatch passes `follow_up_of` with the current run, so the successor
joins the same waggledance run family.

## Why

Leaders churned every 6-7 owner turns at the 150k mark. A handoff opened
a new run family root while the old leader stayed alive, and the same ask
ran in two leaders. The audit of 2026-11-06 found false "done" reports
after these splits.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The context mark is no longer a stop. The agent runs until the user's work is done; the user says when to switch. The hook line and rule agents-context-handoff-65 become advisory; the handoff record stays a tool for feature close or the user's word. (owner) | 8921875b |
| D2 | The hook keeps `handoff_context_tokens` (default 150000, 0 means no line). At or past the mark it adds one advisory line: `context Nk tokens (mark Tk): keep working; hand off only on the user's word or at feature close`. | 48741dcf |
| D3 | Rule agents-context-handoff-65: a session never interrupts work at a context mark; it finishes the task. A handoff is written only when the user asks for a switch or at feature close; then it goes through `waggledance_dispatch` with `leader: true` and `follow_up_of` naming the current run. Never a self-spawned Paseo agent, never "open a new session". | a23d677e |

## Rejected

- Default `handoff_context_tokens` to 0: it hides the size signal the user
  needs to decide a switch (D2).

## Out of scope

- The worker `[HANDOFF]` status token: a dispatched worker's own window
  is a hard limit, not the session mark this ask removes.
- The waggledance handoff flag beyond `follow_up_of`: its name is not
  final; it lands in a follow-up when waggledance run
  run-cddd9fc69d1bcc6c merges.
