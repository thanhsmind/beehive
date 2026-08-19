# Config Sample Herding — Context

**Feature slug:** config-sample-herding
**Date:** 2026-08-20
**Shaping session:** complete (brief — small lane)
**Scope:** Quick

## Feature Boundary

The two config samples document the `herding` key (agent_command/control_command, `bee herding run`), and `bee onboard` seeds `.bee/config-sample.json` into the host repo so a release user sees a full commented sample without visiting the bee repo. Ends before any behavior change to config *reading*.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Both samples gain a `herding` block: `agent_command` (token 0 = herdr kind, passed through, no bee-side allow-list) and `control_command`, each with a `_doc` note naming `bee herding run` / `--continue` and the cell-only boundary | The key existed undocumented; herding-executor made it the executor switch |
| D2 | No key is removed from the samples without citing its retirement in docs/config-reference.md "Removed keys" — the samples were reader-validated by the config-sample feature; only additions are owed today | The ask was "remove unused" — audit found none unread |
| D3 | `bee onboard` seeds `.bee/config-sample.json` create-if-missing, content embedded from the repo's own sample at compile time (include_str), so binary and sample cannot drift | Release ships the binary; the binary carries the sample |

## Existing Code Context

- `.bee/config-sample.json`, `.bee/config-sample-cli-executors.json` — the two samples; no `herding` key today
- `packages/bee-rs/crates/bee/src/onboard/plan.rs:391-401` — create_runtime_file list (create-if-missing)
- `packages/bee-rs/crates/bee/src/onboard/apply.rs:260-272` — per-file content match arms
- `skills/bee-herding/references/operational-invariants.md` — canonical herding.agent_command doc (hx-5); samples point there

## Outstanding Questions

- (none)
