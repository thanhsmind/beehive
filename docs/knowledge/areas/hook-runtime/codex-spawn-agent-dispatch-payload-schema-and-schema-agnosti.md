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
| codex spawn_agent schema | Live-probed on codex-cli 0.145.0: required fields `task_name`, `message`; optional fields `fork_turns`, `model`, `reasoning_effort`. There is no `agent_type` field in this schema — a payload or doc that still names one is teaching a retired shape. |
| everyday dispatch payload | The shape the dispatch helper's ordinary codex-branch emit produces on every routine dispatch: `{task_name, message, fork_turns: "none"}` — the doc-canonical shape, with no override fields attached. |
| legacy payload shape | The pre-0.145.0 shape, `{agent_type: "worker", message}` — no longer emitted by the helper, but a shape the guard must still judge identically to the doc-canonical one, since an older client build or a stale caller could still send it. |

## Behaviors & Operations

**The pre-spawn guard evaluates every spawn_agent payload by tool name and the
anchored marker in `message`, never by which optional or legacy field the
payload happens to carry.** Both the doc-canonical `{task_name, message,
fork_turns}` shape and the legacy `{agent_type, message}` shape receive a real
allow/deny verdict from the identical marker check: an anchored `[bee-tier: ...]`
marker in `message` allows, an unmarked `message` denies — in both shapes, with
no difference in outcome. A payload carrying no `message` at all is the only
shape that produces no verdict (the guard's own no-opinion branch). This closes
the gap a doc/helper/guard mismatch had left open: before this, the guard matched
on `agent_type` alone, so a schema shift away from that field would have made
every future dispatch silently stop being judged at all, rather than being denied
or allowed on its actual content.

**The dispatch helper's ordinary emit is the doc-canonical shape, and the doc
teaches the same shape the helper emits.** The three-way mismatch this feature
closes (a doc saying one shape, a helper emitting another, and a guard judging a
third) is resolved by converging all three on the live-probed schema: the
helper's codex-branch emit, the swarming reference's documented Spawn row, and
the guard's judged shape are now the same `{task_name, message, fork_turns}`
form.

## Business Rules

- `task_name` is required and `agent_type` does not exist in the probed
  0.145.0 schema; a helper, guard clause, or doc still emitting or teaching
  `{agent_type, message}` as the primary shape is stale (i54-closeout D1).
- `model` and `reasoning_effort` exist in the schema, but the ordinary emit
  path never attaches them regardless of shape — whether an attached override
  is itself judged, versus merely passed through unread, is owned by
  [`native-spawn-and-transport-classification.md`](native-spawn-and-transport-classification.md),
  never decided here.

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

- Emit: the codex branch of `dispatch prepare`, `skills/bee-hive/templates/lib/dispatch-prepare.mjs`.
- Judge: `evaluateCodexSpawn` in `skills/bee-hive/templates/lib/dispatch-guard.mjs`
  (mirrored in `.bee/bin/lib/dispatch-guard.mjs`).
- Doc: the Spawn row in `skills/bee-swarming/references/swarming-reference.md`.
- Suites: `scripts/tests/test_dispatch_prepare.mjs` (doc-canonical/legacy round-trip
  rows), `hooks/test_model_guard.mjs` (rows 47-49, 58-59).
- Evidence: `.bee/cells/i54-closeout-1.json`,
  `docs/history/i54-closeout/reports/validation-canary.md`,
  `docs/history/i54-closeout/reports/i54-closeout-1.md`.

