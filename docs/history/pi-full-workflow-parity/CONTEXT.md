# Pi full workflow parity — context

**Feature slug:** pi-full-workflow-parity
**Date:** 2026-09-13
**Shaping session:** complete
**Scope:** Deep
**Domain types:** RUN

## Feature boundary

Make Bee complete the same lifecycle on Pi that it completes on Claude Code. Every difference must follow from a missing Pi capability and have a tested named exclusion.

## Locked decisions

These decisions are fixed. Planning must cite them and must not reinterpret them.

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Keep `d59befe3` as the parity definition. The CLI, the Pi plugin, and generated host helpers must enforce the same lifecycle state as Claude Code. | A fix in only one path leaves another path open. |
| D2 | Target `@earendil-works/pi-coding-agent` only. Do not add an OMP target. | Decision `5d87f14e` fixes the supported runtime. |
| D3 | Keep Pi worker dispatch herding-only. Do not add a native Pi worker transport. | Decision `9f5c6d17` keeps one dispatch path. |
| D4 | Add the workflow-tail acceptance from `0510d81d`. A Pi sandbox must dismiss a pause handoff and close its workflow without an active `HANDOFF.json` projection. | A closed workflow currently leaves an open pause handoff and blocks `bee orient`. |
| D5 | Preserve the Pi belt's failure policies. Blocking tool checks fail closed. Advisory lifecycle checks fail open and report their failure. | The current parity tests and hook-runtime specification depend on this split. |
| D6 | Treat semantic parity as the target. Pi event names can differ from Claude event names, but each Claude lifecycle obligation needs an equivalent Pi path or a tested named exclusion. | Pi and Claude expose different host events. |

### Agent's discretion

Planning can choose the state command shape and the internal close transaction. The command must preserve planned-next claim safety and mailbox history.

## Existing code context

### Reusable assets

- `.pi/extensions/bee-guard.ts` maps Pi events to the shared Rust hook commands.
- `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` derives Pi coverage and drives an onboarded sandbox.
- `packages/bee-rs/crates/bee/src/verbs/workflow_store/handoff.rs` owns mailbox handoff state.
- `packages/bee-rs/crates/bee/src/verbs/workflow_store/projections.rs` rebuilds `.bee/HANDOFF.json`.
- `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs` closes workflow records.

### Established patterns

- The Pi belt derives checkpoint coverage from the Claude manifest and names unsupported host capabilities.
- Workflow records are authoritative. `.bee/HANDOFF.json` is a compatibility projection.
- A planned-next handoff clears only through guarded claim adoption.

### Integration points

- Add the user command to the state command catalog and dispatcher.
- Keep workflow close, handoff mailbox state, and the legacy projection consistent in one operation.
- Extend the Pi lifecycle sandbox through handoff dismissal and workflow close.

## Canonical references

- `docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md`
- `docs/knowledge/areas/workflow-state/handoff.md`
- `docs/knowledge/areas/workflow-state/workflow-records-and-projections.md`
- `docs/history/pi-harness-workflow-parity/CONTEXT.md`
- `.bee/verify/verify-app/features/README.md`

## Outstanding questions

### Deferred to planning

- Which command name gives a pause handoff an explicit dismissal without allowing a planned-next claim to disappear?
- Should workflow close mark its open handoff records cleared, ignore closed workflows during projection rebuild, or apply both protections?
- Which existing Pi sandbox test is the smallest reliable place to prove the full workflow tail?

## Handoff note

`CONTEXT.md` is the source of truth. Planning must keep the existing parity decisions and resolve only the technical questions above.
