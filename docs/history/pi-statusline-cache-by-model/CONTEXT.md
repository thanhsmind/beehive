# Pi Statusline Cache By Model — Context

**Feature slug:** pi-statusline-cache-by-model
**Date:** 2026-09-17
**Shaping session:** complete
**Scope:** Standard
**Domain types:** SEE, READ

## Feature Boundary

Add one Pi footer segment that shows new and cached token totals for each model used on the active session branch.

## Locked Decisions

These decisions are fixed. Planning must cite them and must not change their meaning.

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Group usage by provider and model. Render each model as `<model> <new> new/<cached> cached`. | The user must see which model produced each total. Decision `72f4a8b2-b1f4-467a-bfe0-56cf22da2565`. |
| D2 | `new` equals input, output, and cache-write tokens. `cached` equals cache-read tokens. | This matches the existing bee status-display accounting contract. |
| D3 | Count assistant messages on the active session branch. Do not assign model-less nested tool usage to a model. | A tool result can contain usage but does not identify its model. |
| D4 | Keep Pi's default footer and add this data as an extension status. Hide the status when no model usage exists. | The new fact must not replace Pi's existing status information. |

### Agent's Discretion

Planning can select lifecycle events, formatting helpers, and test fixtures. The implementation must update after a completed model turn and after session restore.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| active session branch | The entries returned by Pi's `SessionManager.getBranch()` for the current leaf. |
| new tokens | Input tokens, output tokens, and cache-write tokens. |
| cached tokens | Cache-read tokens. |

## Existing Code Context

### Reusable Assets

- `.pi/extensions/bee-guard.ts` — the project-local Pi extension and the correct owner for a persistent footer status.
- `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` — executes the real Pi extension under Node with a stub Pi API.
- `packages/bee-rs/crates/bee/src/devtools/statusline.rs` — established token formatting and per-model aggregation behavior for the Claude status display.

### Established Patterns

- Use `ctx.ui.setStatus()` to extend Pi's default footer.
- Read usage from assistant messages on `ctx.sessionManager.getBranch()`.
- Keep display-only failures advisory. They must not stop a session.

### Integration Points

- `.pi/extensions/bee-guard.ts` — calculate and publish the status.
- `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` — prove aggregation, model grouping, refresh, and the empty state.
- `docs/history/codex-harness-hardening/release-manifest.json` — refresh the shipped Pi extension fingerprint through the existing regeneration command.

## Canonical References

- `docs/knowledge/areas/onboarding/status-display-vendoring.md` — status-display behavior and accounting terms.
- Pi `docs/extensions.md` — `session_start`, `turn_end`, `SessionManager`, and `ctx.ui.setStatus()`.
- Pi `docs/session-format.md` — assistant message identity and usage fields.

## Outstanding Questions

None.

## Handoff Note

`CONTEXT.md` is the source of truth. Decision IDs are stable. Planning must cover D1 through D4 and preserve Pi's default footer.
