# Pi parity review fixes — Context

**Feature slug:** pi-parity-review-fixes
**Date:** 2026-09-13
**Shaping session:** complete
**Scope:** Deep
**Domain types:** CALL | RUN | ORGANIZE

## Feature boundary

Resolve all four findings from review `pi-harness-current-parity-review-20260913` so Pi can complete the Bee workflow with the same lifecycle obligations as Claude Code.

## Locked decisions

These decisions are fixed. Planning must implement them exactly.

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | The Pi session preamble and any compacted replacement must publish `dispatch prepare --runtime pi`. The proof must execute the command extracted from the injected preamble. | A test that writes `--runtime pi` itself does not prove the agent receives the correct command. |
| D2 | Pi dispatch stays herding-only. The repair must not add an Agent tool or a second worker transport. | Decision `9f5c6d17` keeps herding as the only Pi worker path. |
| D3 | Workflow close and mailbox writes must use one lock order that prevents a planned-next handoff from appearing after close preflight. A closed workflow must reject a late handoff write. | A planned-next claim is live authority and cannot become hidden behind a closed workflow. |
| D4 | Map documented Pi lifecycle events to Bee activity state. Use `tool_execution_start` for the pre-tool event and `ui_prompt_start` or `ui_prompt_end` for the user-wait span. Keep blocking checks fail closed and advisory checks fail open. | Pi 0.84.4 provides these events, so their current named exclusions are false. |
| D5 | `bee doctor --runtime pi` must return a fail-closed health report for the installed Pi extension, the shared `.agents/skills` tree, binary freshness, and herding transport readiness. Any fact Pi cannot expose must appear as an explicit unknown row, not an unsupported command. | Claude has a runtime health command. Pi needs an equivalent operational check. |
| D6 | Update the existing Pi tests, configuration reference, and knowledge owners. Do not add a second parity document or a second event catalog. | One fact must have one source. |
| D7 | Target only `@earendil-works/pi-coding-agent` 0.84.x. OMP remains outside this feature. | Decision `5d87f14e` fixes the supported host. |
| D8 | Resolve every P1 and P2 finding. Do not reduce scope to make the suite pass. | User decision `7da86fdb` requires the complete repair. |

### Agent's discretion

Planning can choose the runtime field shape, the activity payload mapping, the lock implementation, and the doctor row structure. The implementation must preserve public command compatibility and existing Claude behavior.

## Existing code context

### Reusable assets

- `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs` builds the shared session preamble.
- `.pi/extensions/bee-guard.ts` translates Pi events to shared Bee hooks.
- `packages/bee-rs/crates/bee/src/verbs/workflow_store/handoff.rs` owns mailbox records and handoff locks.
- `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs` owns workflow close.
- `packages/bee-rs/crates/bee/src/doctor.rs` builds Claude and Codex health reports.

### Established patterns

- Pi uses `.agents/skills` through Pi's Agent Skills discovery.
- Pi worker dispatch resolves only to herding commands.
- Mailbox history is append-only. Status changes clear authority without deleting records.
- The Pi belt has separate fail-closed blocking handlers and fail-open advisory handlers.

### Integration points

- Pass the runtime from the Pi extension into session preamble generation.
- Coordinate workflow and mailbox state through shared lock ownership.
- Extend Pi contract tests and the installed sandbox path.
- Extend doctor argument parsing, rows, and tests for Pi.

## Canonical references

- `.bee/reviews/pi-harness-current-parity-review-20260913.json`
- `docs/history/pi-support/CONTEXT.md`
- `docs/history/pi-beehive/CONTEXT.md`
- `docs/history/pi-full-workflow-parity/CONTEXT.md`
- `docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md`
- `docs/knowledge/areas/workflow-state/handoff.md`
- `docs/knowledge/areas/workflow-state/workflow-records-and-projections.md`

## Acceptance

- A Pi session receives a preamble whose dispatch command names runtime `pi`.
- Executing that extracted command returns a herding payload from `team.pi`.
- Claude still receives runtime `claude` in the same preamble section.
- A concurrent workflow close cannot hide or discard planned-next authority.
- A handoff writer that resolved before closure rechecks state and refuses after closure.
- Pi activity records pre-tool work and user-wait spans through documented events.
- `bee doctor --runtime pi --json` returns a structured health verdict.
- The installed Pi sandbox completes the workflow using the injected dispatch command.
- The full Rust test suite passes.

## Handoff note

`CONTEXT.md` is the source of truth. Planning must cite D1 through D8 and preserve the accepted Pi capability differences.
