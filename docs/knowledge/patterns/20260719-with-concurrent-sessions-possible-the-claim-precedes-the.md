---
type: bee.pattern
title: "With concurrent sessions possible, the claim precedes the spawn — and session ids are self-derived, never handed down"
description: "The orchestrator claims a cell atomically before spawning its worker; session ids are read from the worker’s own runtime, never handed down"
tags: [process, multi-session, claims, atomicity, heartbeat, dispatch]
timestamp: 2026-07-19
bee:
  id: pattern-20260719-with-concurrent-sessions-possible-the-claim-precedes-the
  lifecycle: active
  areas: [worktree-parallelism]
  sources: ["docs/history/learnings/critical-patterns.md#PAT38", "original feature: codex-native-transport"]
  polarity: pitfall
  critical: true
---

# With concurrent sessions possible, the claim precedes the spawn — and session ids are self-derived, never handed down

Three near-misses in one feature traced to claims arriving AFTER dispatch: two sessions built
cnt-1/cnt-2 in parallel (~2 worker runs discarded), and a second session's worker tried to claim
a cell the first session's worker already held. The one cell claimed atomically BEFORE its worker
was spawned (`cells claim-next --session-id ...` from the orchestrator, worker told to validate
— never `cells claim`) absorbed the concurrent attempt with zero duplicated work. **Two rules
until multi-session-hardening lands (backlog, 2×P1):** (1) the orchestrator wins the cell first,
then spawns — a worker-side `cells claim` is a non-atomic read-modify-write and its refusal is
the safety net, not the mechanism; (2) any session id attached to reservations/holds is read from
the worker's own runtime env at reserve time — an orchestrator-handed id in the prompt denied a
worker's own write as a "cross-session" conflict this feature. Corollary: a live session reads as
stale after 15 min (heartbeat refreshes only at session start), so liveness signals are advisory
— check for commits/holds before treating a lane owner as dead.
