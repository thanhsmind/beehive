---
type: bee.area
title: "Hook Runtime — the catalog of record, its two projections, and checkpoint activation"
description: "One logical definition of every checkpoint, rendered deterministically into one projection per assistant runtime with every difference named, and the separate question of whether a project's checkpoints are enabled, rooted, and trusted enough to run at all."
timestamp: 2026-07-22
bee:
  id: hook-runtime-catalog-projections-and-activation
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: ["codex-hook-state-parity D1-D3, D8-D13", "codex-runtime-parity D1, D2", d91a8398-2d63-426b-a133-341568453200]
  sources: ["codex-hook-state-parity cells 2, 3, 5 (paired Codex subagent audit, package authority, exclusive hook-source arbitration, and fresh-host handler delivery; capped traces and reports, 2026-07-16)", "codex-runtime-parity Safety foundation — cells codex-parity-2, 2b, 3, 4 (traces in .bee/cells/), reports in docs/history/codex-runtime-parity/reports/", "codex-native-runtime-v2 cnr2-2 (state-sync trigger extended at the generator sources to both runtimes' plan tools)", "docs/specs/hook-runtime.md#B5", "docs/specs/hook-runtime.md#B6", "docs/specs/hook-runtime.md#R1", "docs/specs/hook-runtime.md#R6", "docs/specs/hook-runtime.md#E4", "docs/specs/hook-runtime.md#E5", "docs/specs/hook-runtime.md#E7", "docs/specs/hook-runtime.md#P3"]
  authoritative_for: "hook-runtime: the catalog of record, projection parity, and checkpoint activation"
---

# Hook Runtime — the catalog of record, its two projections, and checkpoint activation

Which checkpoints exist is one question; whether they fire in a given project is
another. The first is answered by a single catalog of record rendered into one
projection per runtime, where every difference between the projections is a
named export rather than drift. The second is answered by the project's own
configuration and by the human owner's trust in each command definition.

## Entry Points & Triggers

- Which checkpoints are active comes from one **catalog of record** rendered
  into projections. Each supported runtime consumes only its own projection;
  the projections differ only by an explicitly named allowed list. The
  directional differences: both runtimes carry a pre-spawn dispatch guard —
  Claude on its dispatch tools, Codex on its native spawn call, judging only
  the envelope shape actually observed on the probed runtime version and
  passing every unobserved shape through open — while Codex alone has
  child-start and child-stop lifecycle audits (codex-native-runtime-v2,
  cnr2-8).

## Data Dictionary

| Element | Meaning |
|---|---|
| catalog of record | The single logical definition of every checkpoint: event, matcher, handler. Both runtime projections are rendered from it deterministically — rendering again must reproduce both byte-for-byte. |
| projection | The runtime-specific checkpoint list a given assistant actually loads. One per runtime, checked in, never hand-divergent. |
| allowed difference | A named, exported exception explaining why one projection carries a checkpoint the other lacks. Any un-named difference between projections is a defect. |
| reviewed definition | The exact command definition the owner has inspected and trusted. A new or changed non-managed definition does not run until it is reviewed again. |

## Behaviors & Operations

**B5 — Two projections, one truth.** Changing the catalog of record and
re-rendering updates both projections in the same change; the parity check in
the installation suite compares the assistant-facing settings against the
correct projection for that runtime and fails on any un-allowed drift. Each
difference is declared by runtime, event, and handler, and each projection is
proved independently.

**B6 — Project checkpoints are active, rooted, and reviewed.** Project
checkpoints are enabled unless an active configuration explicitly disables
them. A checkpoint command starts with the session's working directory, which
may be below the project root, so a project-local command first resolves the
project root and then launches its handler. A new or changed non-managed
definition is listed for review and skipped until the human owner trusts that
exact definition. Afterwards, a fresh lifecycle event uses the reviewed
definition; until then the assistant continues without that checkpoint and the
owner sees the pending-review warning.

## Business Rules

- R1 — One catalog of record; projections are rendered, never hand-edited;
  all three directional differences must be exported by name
  (codex-runtime-parity D1; codex-hook-state-parity D1-D3).

- R6 — Project checkpoints are enabled by default, resolve project-local
  handlers from the project root even when a session starts below it, and any
  changed non-managed definition requires fresh human review before execution
  (decision d91a8398-2d63-426b-a133-341568453200).

## Edge Cases Settled

- Explicitly disabling checkpoints produces no project lifecycle execution;
  the absence of an opt-in flag does not disable them.

- Editing a reviewed command definition makes only the changed definition
  pending review; automation never rewrites or bypasses the owner's trust
  record.

- The state-sync trigger matches the plan/task tools of BOTH runtimes as a
  superset — Codex's native plan tool (`update_plan`) alongside the legacy
  Claude names — extended at the generator sources (catalog + both host
  renderers), never by hand-editing a rendered manifest; behavior proven by a
  contract row driving a real `update_plan` payload (codex-native-runtime-v2,
  cnr2-2).

## Pointers (implementation)

- Catalog + renderer: `packages/bee/hooks/catalog.mjs` (exports `ALLOWED_DIFFERENCES`,
  `TARGETS`, `REPO_TRANSPORT_UNAVAILABLE_DIAGNOSTIC`); `renderProjection`/
  `renderProjectionText` take an explicit `target` (`plugin` default, `repo`)
  so both rendering targets share one function, never forked logic.
  Projections: `packages/bee/hooks/hooks.json` (Codex, plugin target), `packages/bee/hooks/claude-hooks.json`
  (Claude, plugin target; `.claude-plugin/plugin.json` points here).
