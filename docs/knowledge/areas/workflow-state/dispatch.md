---
type: bee.area
title: Workflow State — the unified command entry point and its catalog
description: "One entry point owning the single implementation of all nine verb groups, publishing a machine-readable catalog of every command it accepts, validating a request before dispatching it, and signalling a changed discovery surface without disturbing any command's ordinary output."
timestamp: 2026-07-22
bee:
  id: workflow-state-dispatch
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: [30606de4-5fae-4c9d-9e3f-8f47a494f8a3 (one unified command entry point publishing a machine-readable catalog), bbc6bcea (shim-retire D1 — the legacy per-group forwarders are deleted; bee.mjs is the sole shipped CLI), 8ef2bae6 (cli-ergonomics D1 — exhaustive refusal, every problem + a runnable example in one message)]
  sources: ["harness-integration-adopt cells hia-1 and hia-2 (traces and reports, 2026-07-12)", "dispatcher-unify cells du-1..du-6 (traces and reports, 2026-07-12, flushed capture stubs b6a2233c/9e68432b)", "docs/specs/workflow-state.md#B8", "docs/specs/workflow-state.md#R12", "docs/specs/workflow-state.md#R13", "docs/specs/workflow-state.md#E10", "docs/specs/workflow-state.md#E11", "docs/specs/workflow-state.md#P6", "docs/specs/workflow-state.md#P10"]
  authoritative_for: "workflow-state: unified command discovery, validation, and dispatch"
---

# Workflow State — the unified command entry point and its catalog

An automated assistant can only call what it can discover. This concept owns the
one surface that makes the workflow discoverable: a single entry point that owns
the implementation, a catalog that describes exactly the commands that surface
really accepts, and a validation step that refuses a malformed request before any
record changes.

## Behaviors & Operations

**B8 — Unified command discovery and dispatch.** Every workflow operation — all
nine verb groups — is available both through its specialized entry point and
through one unified entry point, and the unified side owns the single
implementation: each specialized entry point is a thin forwarder whose output
is byte-identical to the unified path, and a new verb is added exactly once
(one catalog entry plus one handler), never re-implemented in a forwarder.
The unified entry
point publishes the complete command catalog in human-readable and
machine-readable forms. It validates required parameters and their value shapes
before dispatch, then invokes the same underlying operation as the specialized
entry point; it does not run one command-line program from another. For the same
valid request, observers receive the same result and exit outcome through either
surface. This includes revising an open or blocked work cell's allowed plan
fields. An unknown command is refused with the nearest known command when one is
available. A malformed request is refused with the command, field, and reason,
without executing the operation — and the refusal is exhaustive (cli-ergonomics
D1, 8ef2bae6): every missing and invalid parameter is named in the one refusal,
alongside a runnable example taken from the catalog entry, so a caller never
discovers problems one retry at a time. The structured error keeps the first
problem in its legacy fields (existing consumers unchanged) and carries the
full list additively. Legacy verbs that deliberately own their own checks
(DB3) gained the same all-at-once behavior inside the handler layer, on their
original error channel. After a catalog change, observers receive a
separate diagnostic signal while the requested command's normal output keeps its
stable shape.

## Business Rules

- R12 — The unified entry point serves all nine command groups from one
  implementation; the specialized entry points are thin forwarders with
  byte-identical output, and a new verb is added once — one catalog entry plus
  one handler, never a second implementation in a forwarder (decision
  30606de4-5fae-4c9d-9e3f-8f47a494f8a3; dispatcher-unify decision 2026-07-12).
- R13 — The published command catalog and executable dispatch surface describe
  the same command set. Every published example is exercised against the real
  operation, so a documented but unusable command is a verification failure
  (decision 30606de4-5fae-4c9d-9e3f-8f47a494f8a3).

## Edge Cases Settled

- A catalog fingerprint change never appears inside the requested command's
  ordinary result. Consumers that parse normal output therefore remain stable
  while diagnostics can still report that discovery metadata changed.
- A missing required parameter, a value with the wrong shape, or an unknown
  command is rejected before any workflow record changes.

## Pointers (implementation)

- Unified dispatcher and catalog: `packages/bee/bee.mjs`,
  `packages/bee/lib/command-registry.mjs`, and
  `packages/bee/lib/validate-args.mjs`, mirrored under `.bee/bin/`.
  Evidence: `.bee/cells/hia-1.json`, `.bee/cells/hia-2.json`, and
  `docs/history/harness-integration-adopt/reports/`.
- Unified dispatcher (all nine groups): `packages/bee/bee.mjs` owns
  registry + handlers; dispatcher-unify (`.bee/cells/du-{1..6}.json`,
  `docs/history/dispatcher-unify/`) first made every legacy per-group script a
  2-line forwarder with byte-identical output, then shim-retire (D1, decision
  bbc6bcea; `.bee/cells/shim-retire-{1..6}.json`) deleted those forwarders
  outright — `bee.mjs` is now the sole shipped CLI, no forwarders remain.
