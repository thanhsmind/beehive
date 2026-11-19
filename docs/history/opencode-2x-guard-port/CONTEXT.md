# Port the bee guard belt to the opencode 2.x plugin API — Context

**Feature slug:** opencode-2x-guard-port
**Date:** 2026-11-19
**Backlog items:** p-627c4c0f
**Shaping session:** complete under gate bypass full (owner said go on the 2026-11-19 backlog sweep)
**Scope:** Standard
**Domain types:** RUN | CALL

## Feature Boundary

opencode 2.x loads the bee guard plugin, and the guard blocks writes and shell
calls by the same belt rules as on 1.x. CI proves it on opencode 2.x. The 1.x
belt keeps working.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | The bee opencode plugin exports the 2.x plugin shape alongside the 1.x shape, from one rule source, so the belt rules have one home. | p-627c4c0f; docs/briefs/p-627c4c0f.md states 2.x does not load the guard today. |
| D2 | The 2.x tool-id registry already derived in `tests/opencode_plugin_contracts.rs` (near 2082, 2181-2185) is the map the 2.x guard uses; every 2.x write- or read-capable tool is mapped or named as a gap. | Reuse the shipped derivation. |
| D3 | CI gains one job that installs opencode 2.x and runs the guard contract proof against it; the 1.x job stays. | Proof on the real runtime. |

## Read set

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs` | D2, proof. |
| `.github/workflows/ci.yml` | D3. |
| `docs/briefs/p-627c4c0f.md` | The owner brief. |

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
