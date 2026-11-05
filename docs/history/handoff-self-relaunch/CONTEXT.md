# handoff-self-relaunch — CONTEXT

## Ask

User, 2026-11-05: a deploy session in another repo stopped at the context
handoff mark and told the user to open a new Claude Code session. The user
wants the leader to report to waggledance and let it open the fresh leader,
with no question back to the user. The user picked the bee doctrine fix.

## Locked decisions

- **D1** (9a231232, c9439edb) — At the handoff mark the leader writes the
  handoff, hands the remaining work to a fresh leader through the host's
  leader door (waggledance: `waggledance_dispatch` with `leader: true`), and
  releases its session. It never tells the user to open a new session and
  never spawns the leader itself. The handoff is shown to the user, and the
  session waits, only when a gate or decision the user owns is pending, or
  when the host has no leader door.
- **D2** (scope) — This feature changes the doctrine text, its skill and
  handbook homes, and the prompt-context hook line. The verb
  `bee state handoff dispatch` named in 9a231232 is split to the backlog:
  bee calling waggledance from the CLI needs its own shape.

## Why the old rule said pause

The pause rule protects one thing: a fresh session must not start from a
lossy summary, and in-flight claims must be triaged. D1 keeps both — the
handoff record and the resume triage stay as they are. Only who starts the
next session changes.
