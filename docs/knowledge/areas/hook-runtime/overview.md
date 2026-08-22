---
type: bee.area
title: "Hook Runtime — purpose, lifecycle checkpoints, and the fail-open frame"
description: "What the lifecycle checkpoints around an assistant guarantee, who observes them, and the two properties that hold for every checkpoint without exception: hostile input never ends a turn, and the guardrails are a safety net rather than the authority."
timestamp: 2026-08-05
bee:
  id: hook-runtime-overview
  lifecycle: active
  areas: [hook-runtime]
  required_context: []
  decisions: ["codex-runtime-parity D1, D2", "c2c46488 (a closed feature's approvals never license the next write; the guard's silence is never permission)"]
  sources: ["codex-runtime-parity Safety foundation — cells codex-parity-2, 2b, 3, 4 (traces in .bee/cells/), reports in docs/history/codex-runtime-parity/reports/", "docs/specs/hook-runtime.md#B1", "docs/specs/hook-runtime.md#R2", "docs/specs/hook-runtime.md#R13", "knowledge-loop cell kl-4 (session-start critical-pattern digest ranked by relevance; trace .bee/cells/kl-4.json, commit d74ca11c, 2026-08-05) — cross-referenced, owned by okf-profile/context-and-promote.md"]
  authoritative_for: "hook-runtime: purpose, actors, and the cross-cutting checkpoint frame"
  owns.code: ["packages/bee-rs/crates/bee/src/hooks/*"]
  owns.skills: []
  owns.tests: [packages/bee-rs/crates/bee/tests/hook_contracts.rs, packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs, packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs, packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs]
---

# Hook Runtime — purpose, lifecycle checkpoints, and the fail-open frame

Every other concept in this area describes one checkpoint or one guard. This one
describes the frame all of them sit inside: what the checkpoints are for, who
sees them, what a checkpoint does when its input is hostile, and why none of it
is ever allowed to be mistaken for the authority.

## Purpose

While an AI assistant works inside a bee-managed project, a set of lifecycle
checkpoints runs around it: session start context, per-prompt reminders, write
protection, dispatch auditing, state refresh, worker nudges, and close-time
hygiene. This area describes what those checkpoints guarantee, on which
assistant runtime, and what happens when input is hostile or a path is
unsupported. The guardrails are a safety net, not a security boundary — the
durable project instructions and the shared helper checks remain the final
belt for anything a checkpoint cannot see.

## Entry Points & Triggers

- A supported assistant runtime (two are supported today) fires a checkpoint at
  each lifecycle event: session start, user prompt submitted, before a tool
  runs, after tracked task updates, before context compaction, when a child
  agent starts, when a child agent stops, and when the session stops.
- **The session-start checkpoint's critical-pattern digest** — how it ranks the
  bundle's critical patterns by relevance to the bound feature, which files it
  reads to do so, and how its header names its own ranking mode versus a
  recency fallback — is owned by
  [`okf-profile/context-and-promote.md`](../okf-profile/context-and-promote.md)
  (B8, knowledge-loop D3), the same concept that owns the shared anchor the
  ranking resolves against. Kept there rather than duplicated here, since that
  concept already documents the digest's other rules (silence on no bundle,
  the byte-identical no-migration branch).

## Data Dictionary

| Element | Meaning |
|---|---|
| fail-open | On unreadable/hostile input or an internal crash, the checkpoint permits the action and logs the gap visibly. It never silently swallows the event. |
| fail-closed (deny) | The checkpoint blocks the action with a corrective message telling the actor how to proceed correctly. |
| coverage gap | A named event/path the runtime cannot intercept, logged visibly at runtime and listed here — never claimed as protected. |

## Actors & Access

- **The assistant** (either runtime) — subject of every checkpoint; observes
  context injections, denials with corrective messages, and advisories.
- **The human owner** — sees deny messages surfaced by the assistant and the
  visible gap log; approves anything the guardrails escalate (privacy reads,
  gates).
- **Workers (child agents)** — same write rules as the main assistant;
  additionally matched by registered identity for nudges.

## Behaviors & Operations

**B1 — Hostile-input immunity (every checkpoint).** Whatever arrives on a
checkpoint's input — empty, garbage bytes, null, a list where an object was
expected, a wrong-typed working directory, or a multi-megabyte payload — the
checkpoint normalizes it before touching any field. It never crashes the
assistant's turn. The decision it would have made is never *flipped* by an
internal failure: a crash in logging or loading support code ends in fail-open
with a visible log entry, not in a new allow or a new deny. Every actor
observes either the normal outcome or a logged fail-open — never a stack trace
ending the turn.

## Business Rules

- R2 — A checkpoint failure never flips an allow/deny decision; fail-open is
  visible, never silent (codex-runtime-parity D2).

- R13 — The guardrails are a safety net, not the authority. The workflow's
  written law is what governs the assistant; a checkpoint exists to catch what
  the assistant forgets, and its silence is never permission. An assistant must
  never treat "the guard did not stop me" as approval, because that promotes
  the guard's coverage into the protocol and turns every gap in the guard into
  a gap in the workflow — which is exactly how R12's gap was found in real use
  (decision c2c46488).

## Open Gaps

- Native (non-shell) file reads and the incomplete unified-shell path on the
  second runtime cannot be intercepted — governed by the durable instructions
  and helper checks; logged as coverage gaps at runtime.

(The other named gaps live with the concept whose topic they belong to:
[`delivery-targets-and-the-fallback-command.md`](delivery-targets-and-the-fallback-command.md)
carries the dogfood-boundary and non-POSIX-shell gaps,
[`child-agent-attribution-and-audit.md`](child-agent-attribution-and-audit.md) the
child-identity gap, [`native-spawn-and-transport-classification.md`](native-spawn-and-transport-classification.md)
the override-field route-check, [`governed-paths-and-the-intake-gate.md`](governed-paths-and-the-intake-gate.md)
the invisible-experiment tradeoff, and
[`health-checks-and-proof-surfaces.md`](health-checks-and-proof-surfaces.md) the outstanding
live package proof.)
