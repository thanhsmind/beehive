# Preview id clash — Context

**Feature slug:** preview-id-clash
**Date:** 2026-11-19
**Scope:** Small
**Domain types:** RUN

## What was asked

The owner asked to work the fix-at debt. The top open group (8 findings):
plan.md named cells psd-1 and psd-2, ids already held by archived
pi-stage-dispatch cells, and the clash surfaced only at `cells add` after
Gate 2. `writeCell` refuses an archived id, but only after the gate, so a
rename costs an unapprove, a re-preview and a re-approve.

## What will be done

`bee gate --preview` refuses a packet whose cell id is already held by a live
or archived cell of a different feature, before it writes anything.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | Gate preview refuses, writing nothing, when a packet cell id is held by a live or archived cell of a different feature; the refusal names the id, the holding feature, live or archived, and says to rename the id in plan.md. Same-feature ids pass. | e03c4bf8 |
| C1 | Test contract preview-id-clash: archived and live clashes each refuse and name the id and holding feature, gate_preview stays unwritten; free and same-feature ids preview as before. | fd9a503f |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/workflow-state/gates.md | suspect | the gate preview step and its packet |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs` | `run_gate_preview_body`, the preview door |
| `packages/bee-rs/crates/bee/src/verbs/cells/util.rs` | `resolve_cell_file`, the live-then-archive id lookup |
| `packages/bee-rs/crates/bee/tests/gate_preview.rs` | the preview integration tests |

## Out of scope

The `change_class` half of the related paseo-pi finding.
