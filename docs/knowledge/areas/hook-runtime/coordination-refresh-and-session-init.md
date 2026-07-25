---
type: bee.area
title: Hook Runtime — durable state a checkpoint maintains as a side job
description: "The opportunistic heartbeat-and-lease refresh two checkpoints attempt without ever blocking on it, the try-once/skip-on-busy discipline every checkpoint uses against the coordination lock, and the transcript path session-init stores so later readers stop recomputing it."
timestamp: 2026-07-22
bee:
  id: hook-runtime-coordination-refresh-and-session-init
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: ["multi-session-hardening D2/D5 with Δ3-amendment (docs/history/multi-session-hardening/CONTEXT.md; audit 12f54e88, locked 17a624dc)", "codex-loop-p0 clp-1 (session-routed prompt reminder — a lane-bound session sees its own phase; repair recorded by scribing-integrity D5, 2026-07-24)"]
  sources: ["multi-session-hardening cell msh-5 (throttled heartbeat-and-lease renewal wired into the per-prompt and post-task-update checkpoints, try-once/skip-on-busy against the coordination lock; trace in .bee/cells/, report docs/history/multi-session-hardening/reports/msh-5.md, 2026-07-19)", hardening-1-7-10 cells 1710-1..1710-11 (2026-07-21 — session-init persists the runtime-provided transcript path into the session record so crash recovery prefers it over layout math), "docs/specs/hook-runtime.md#B20", "docs/specs/hook-runtime.md#B21", "docs/specs/hook-runtime.md#R19", "docs/specs/hook-runtime.md#R21", "docs/specs/hook-runtime.md#E8", "docs/specs/hook-runtime.md#P17"]
  authoritative_for: "hook-runtime: opportunistic coordination refresh and session-init state persistence"
---

# Hook Runtime — durable state a checkpoint maintains as a side job

Some checkpoints do a second, quieter job alongside the one they exist for:
keeping a live session's claims and holds from expiring, and recording where
this session's transcript actually is. Both are strictly secondary — each sits
in its own failure boundary, neither may delay or change the checkpoint's real
outcome, and the refresh would rather skip a cycle than wait on a lock a
checkpoint is never allowed to wait on.

## Behaviors & Operations

**B20 — The per-prompt and post-task-update checkpoints opportunistically
refresh coordination state, never blocking on it.** Trigger: the per-prompt
checkpoint and the post-task-update checkpoint fire, as they already do for
their own primary job (context injection, state/cell-count refresh). What
happens: each also attempts the acting session's own heartbeat-and-lease
refresh (workflow-state B24) — its own automatic, throttled renewal of the
session's heartbeat plus every claim and hold it owns. That attempt is wrapped
in its own failure boundary, separate from the checkpoint's primary job: a
failure inside the refresh is logged as a visible gap and never blocks,
delays, or changes the checkpoint's own outcome (B1 unchanged). The refresh's
own writes never wait on a busy shared coordination store: they try once and
skip silently on contention, the same discipline every lifecycle checkpoint
uses against the coordination lock (workflow-state B21) — a checkpoint never
waits on that lock, only a command-line verb does. What each actor observes: a
session that stays genuinely active keeps its claims and holds fresh without
any extra step; a checkpoint that loses the lock race simply skips one
opportunistic refresh, and the primary reminder or state-refresh work it
exists for still runs and is still reported exactly as before
(multi-session-hardening D5). The per-prompt reminder is session-routed
(codex-loop-p0, cell clp-1): it threads the session's own identifier, so a
lane-bound session sees its OWN lane's phase — never the default record's
`idle`, which used to push a working session back toward exploring it had
already passed; the same fix removed the false review-gate prompt and the
spurious idle→exploring entry that the mis-routed reminder produced.

**B21 — Session-init persists its own runtime-provided transcript path,
instead of leaving every later reader to recompute it.** Trigger: the
session-init checkpoint runs at session start. What happens: whenever the
active runtime hands the checkpoint a transcript path as part of its own
session-start payload, that path is written into the session's own durable
record at the same moment, rather than only being used transiently for the
checkpoint's own purposes. What each actor observes: any later consumer that
needs to find this session's transcript — most notably crash recovery
(workflow-state B33) — can read the stored path directly instead of deriving
it from the runtime's usual on-disk layout convention. This makes a
non-Claude, Codex-shaped transcript layout first-class rather than a
best-effort guess: recovery no longer has to already know, or successfully
infer, where a second runtime's transcripts live in order to find one that
was actually reported at session start. A runtime that hands no transcript
path leaves the field absent, and lookup falls back to layout math exactly as
it did before this behavior existed (hardening-1-7-10).

## Business Rules

- R19 — A lifecycle checkpoint's own opportunistic coordination-state refresh
  never waits on the coordination lock and never lets a failure inside it
  change the checkpoint's primary outcome; only a command-line verb's
  read-modify-write waits for the lock (B20; multi-session-hardening D2/D5,
  Δ3-amended).

- R21 — Session-init persists a runtime-provided transcript path into the
  session record the moment it is available; any later reader (crash
  recovery above all) prefers that stored path over recomputing a layout,
  making a second runtime's transcript layout first-class rather than a
  best-effort guess (B21; hardening-1-7-10).

## Edge Cases Settled

- Concurrent hook invocations can no longer corrupt a state write through
  temp-file collision: atomic writes use a unique per-invocation temp name
  (write-then-rename contract unchanged), proven by a parallel regression test
  (18 concurrent OS processes, zero corrupt reads). Logical last-writer-wins
  between full read-modify-write cycles is now CLOSED for every
  read-modify-write verb that runs its body under the coordination store lock
  (workflow-state B21/B23) — a command-line verb's read-check-write serializes
  through the lock and waits for it; only a lifecycle checkpoint's own
  opportunistic refresh still uses the try-once, skip-on-busy discipline
  (B20), which was always allowed to skip a cycle rather than corrupt one. A
  future full revision/compare-and-swap layer remains a deferred concern for
  if cross-process contention ever outgrows the lock, not a currently open
  gap (codex-native-runtime-v2, cnr2-5, superseded by
  multi-session-hardening D2/D6).

## Pointers (implementation)

- Opportunistic coordination refresh (B20, R19): `heartbeatTouch` (session
  heartbeat + `renewClaimTTL`) in `skills/bee-hive/templates/lib/claims.mjs`,
  `renewHoldsBySession` in `skills/bee-hive/templates/lib/reservations.mjs`,
  composed at the checkpoint call site (not imported by `claims.mjs` itself)
  in `hooks/bee-prompt-context.mjs` (UserPromptSubmit) and
  `hooks/bee-state-sync.mjs` (PostToolUse/Stop), each inside its own
  try/catch separate from the checkpoint's primary job; both hook copies
  mirrored under `.bee/bin/hooks/`. Coordination lock primitive:
  `withStoreLock` (`options.maxAttempts`/`retryDelayMs` power the checkpoint's
  try-once mode) in `skills/bee-hive/templates/lib/lock.mjs`. Suite:
  `scripts/tests/test_heartbeat_touch.mjs` (throttle no-op, real-hook-driven
  refresh, touch-throw fail-open, `LOCK_BUSY` silent skip, renewal-vs-adopt
  gate skip). Evidence: `.bee/cells/msh-5.json`,
  `docs/history/multi-session-hardening/reports/msh-5.md`.
