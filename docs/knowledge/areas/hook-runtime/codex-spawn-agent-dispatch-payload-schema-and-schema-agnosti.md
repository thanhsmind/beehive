---
type: bee.area
title: Hook Runtime — the codex spawn_agent dispatch payload schema and schema-agnostic guard evaluation
description: "The live-probed codex spawn_agent tool schema the dispatch helper emits against, and how the pre-spawn guard judges every spawn_agent payload by tool name and marker regardless of which payload shape carries it."
timestamp: 2026-07-24
bee:
  id: hook-runtime-codex-spawn-agent-dispatch-payload-schema
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md, areas/hook-runtime/native-spawn-and-transport-classification.md]
  decisions: [i54-closeout D1 (dispatch schema converges on live-probed truth), 103a5608 (i54-closeout scope lock)]
  sources: ["i54-closeout cell i54-closeout-1 (helper + guard + doc converge on the live-probed codex 0.145.0 spawn_agent schema; round-trip tests both directions; trace in .bee/cells/, 2026-07-24)", "docs/history/i54-closeout/reports/validation-canary.md (live probe evidence: tool-schema self-inspection, override-rejection probe)", docs/history/i54-closeout/CONTEXT.md D1]
  authoritative_for: "hook-runtime: codex spawn_agent dispatch payload schema and schema-agnostic guard evaluation by tool name and marker"
---

# Hook Runtime — The Codex spawn_agent Dispatch Payload Schema and Schema-Agnostic Guard Evaluation

A dispatch helper and a pre-spawn guard must agree on what a real spawn call looks
like, or one of them is teaching or judging a shape the runtime never actually
sends. This concept owns the live-probed shape of the codex `spawn_agent` tool call
and the guard rule that stays correct even when that shape changes.

## Data Dictionary

| Element | Meaning |
|---|---|
| codex spawn_agent schema | Live-probed on Codex 0.154.0: required fields `task_name`, `message`; optional fields `fork_turns` (STRING: "none", "all", or positive integer string, e.g. "1", not integer), `model`, and `reasoning_effort`. There is no `agent_type` field and no `sandbox` field in this callable schema. |
| everyday dispatch payload | The shape `dispatch prepare` emits for supported cell dispatch: `{task_name, message, fork_turns: "none", model?, reasoning_effort?}`. Native cell dispatch on installed Codex 0.154.0 is refused at preparation time due to opaque hook messages (`native_hook_input_opaque`), directing to configured herding or CLI routes. Configured herding and CLI routes stay explicit; for model-shaped or prompt-budget non-cell roles (gather, reviewer, advisor, including Model, Native and null/budget resolutions), it emits an explicit read-only CLI sandbox command (`codex exec --sandbox read-only --ephemeral -`) because native spawn lacks a sandbox field. |
| legacy payload shape | The pre-0.145.0 shape, `{agent_type: "worker", message}` — no longer emitted by the helper, but a shape the guard still handles, since an older client build or a stale caller could still send it. |

## Behaviors & Operations

**On Codex 0.154.0, native cell dispatch is refused due to opaque message delivery before schema examples apply.**
Native input arrives with an opaque message body hiding the role marker; `evaluate_codex_spawn` in `model-guard` denies unmarked spawns as transport `codex-spawn-unmarked` (exit 2), while `dispatch prepare` classifies the client version as `native_hook_input_opaque` and refuses native cell dispatch with clear diagnostics pointing to configured CLI/herding transports.

**The pre-spawn guard evaluates callable spawn_agent payloads by tool name, the
anchored marker in `message`, and declared model/effort settings.**
Both the doc-canonical `{task_name, message, fork_turns, model, reasoning_effort}`
shape and the legacy `{agent_type, message}` shape are judged. An anchored
`[bee-tier: ...]` marker in `message` selects the configured role. The guard
verifies that requested `model` and `reasoning_effort` match configured role
settings, denies overrides on full-history forks and escalated roles, and denies
read-only native requests as unenforceable.

**The dispatch helper emits configured settings for cell dispatches and CLI
sandboxes for model-shaped or prompt-budget non-cell roles.**
For cell execution, `dispatch prepare` resolves the role and attaches configured
`model` and `reasoning_effort` with `fork_turns: "none"`. Configured herding and CLI
routes stay explicit. For model-shaped or prompt-budget non-cell roles (gather,
reviewer, advisor, including Model, Native and null/budget resolutions), `dispatch prepare`
emits `codex exec --sandbox read-only --ephemeral -` to guarantee filesystem read-only enforcement.

## Business Rules

- `task_name` and `message` are required; `agent_type` and `sandbox` do not
  exist in the callable schema (i54-closeout D1, codex-parity D1).
- `fork_turns` is a STRING ("none", "all", or positive integer string), not an integer.
- `model` and `reasoning_effort` are supported in the callable schema; `dispatch
  prepare` attaches configured values for supported cell dispatches.
- Configured herding and CLI routes stay explicit.
- Model-shaped or prompt-budget non-cell roles must use the CLI read-only sandbox transport
  because native spawn provides no filesystem sandbox boundary. Explicit unknown
  `--role` is refused, never granted fallback.
- Full-history forks (`fork_turns: "all"` or omitted) and escalated roles cannot
  carry model or reasoning_effort overrides.

## Edge Cases Settled

- A payload constructed directly in the doc-canonical shape, and a payload
  actually emitted by the dispatch helper, both round-trip through the guard to
  the identical verdict — the exact untested direction a doc/helper/guard
  three-way mismatch had left unproven is now covered by explicit round-trip
  tests in both directions.
- A payload built in the legacy `{agent_type, message}` shape is judged
  identically to the doc-canonical shape by the same marker check; the guard's
  widening is additive only — no shape that used to deny now allows, and no
  shape that used to allow now denies.

## Pointers (implementation)

- Emit: `codex_spawn_payload` and codex branch in
  `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`.
- Judge: `evaluate_codex_spawn` in
  `packages/bee-rs/crates/bee/src/hooks/model_guard.rs`.
- Doc: the Spawn row in `skills/bee-swarming/references/swarming-reference.md`.
- Historical Node implementations (historical evidence):
  `packages/bee/lib/dispatch-prepare.mjs`, `packages/bee/lib/dispatch-guard.mjs`,
  `scripts/tests/test_dispatch_prepare.mjs`, and `hooks/test_model_guard.mjs`.
- Evidence: `.bee/cells/i54-closeout-1.json`,
  `docs/history/i54-closeout/reports/validation-canary.md`,
  `docs/history/i54-closeout/reports/i54-closeout-1.md`.

