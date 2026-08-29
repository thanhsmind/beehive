# Pi Support — Context

**Feature slug:** pi-support
**Date:** 2026-08-29
**Shaping session:** complete
**Scope:** Deep
**Domain types:** CALL | RUN | ORGANIZE

## Feature Boundary

bee runs under the Pi coding agent (0.84.x) as well as it runs under Claude
Code: a fourth harness belt. The belt is one repo-local Pi extension
(`.pi/extensions/bee-guard.ts`) translating Pi's extension events onto the
existing `bee hook <rule>` helpers, plus the `pi` runtime entry at the
dispatch door (`models.pi`, herding-only slots). The feature ends there:
the worker-result mailbox transport (pi-peer pattern) is a SEPARATE
follow-up feature (`pi-result-mailbox`), and Paseo needs no work at all
(Pi loads project extensions from the workspace cwd; Paseo inherits).

## Locked Decisions

Store provenance: `7f9c8518` (no native subagents), `4a6e38be` (one config
home), the settled model table (2026-08-29), `23de5362` (seat roles), and
the research briefs docs/history/research/pi-harness-support.md +
docs/history/research/pi-peer-distill.md.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The Pi belt is a TS **extension**, never JSON hook config: repo-local `.pi/extensions/bee-guard.ts`, auto-discovered by Pi from the workspace cwd (`<cwd>/.pi/extensions/`), shipped from the checkout's own tree by the onboard step exactly as `copy_opencode_plugin` ships the OpenCode plugin. No global install, no user config. | Pi has no hook-config surface (store: research brief, Pi 0.84.3 docs). |
| D2 | Helpers stay the FIRST belt: the extension execs `.bee/bin/bee hook <rule>` and translates verdicts. Event mapping: `tool_call` → write-guard/model-guard returning `{block: true, reason}`; `before_agent_start` → session-init/prompt-context (systemPrompt append + `event.prompt` read); `tool_result` → state-sync/activity; `agent_settled` → turn-end waiting mark. | The three-belt architecture's own law (opencode_plugin_contracts.rs). |
| D3 | Failure policy is the OpenCode belt's, byte-for-byte in intent: BLOCKING surfaces (write-guard, model-guard) fail CLOSED — deny, crash, or missing binary throws a block; ADVISORY surfaces fail OPEN, never throwing. | `bee-guard.ts:14-32` precedent; pattern 20260714 (fail-open host swallows fail-closed throws). |
| D4 | The hook catalog (`hook_manifests.rs`) gains the Pi belt's rows, and the three-belt parity test becomes FOUR-belt — rows derived from the catalog, never a hand list. | Pattern 20260722 (coverage gate derives ground truth). |
| D5 | `dispatch prepare --runtime pi` becomes legal, resolving `models.pi` in the one config home. On the pi runtime EVERY slot must be `kind: "herding"` (or a herding-resolvable form) — a native model slot under `models.pi` is refused by name, because Pi has no Agent-tool surface (`7f9c8518`). Seat roles (`23de5362`) ride along like any other role. | An Agent-tool payload emitted on pi would dispatch nothing. |
| D6 | The `models.pi` values follow the settled table: heavy roles (code, test, docs, review) → herding agents running `claude --model opus`; advisor → `claude --model fable`; cheap roles (read, extraction, generation, supervisor) → `agy-flash`. Herding constrains the TRANSPORT, not the model vendor. | User's call, 2026-08-29. |
| D7 | Worker-result transport (mailbox + steer/trigger injection, pi-peer pattern) is OUT of this feature — split to `pi-result-mailbox`, which lands before Pi herding is declared production-ready. Inside this feature, dispatches on pi use the existing `bee herding run` contract as-is, with its recorded digest-loss friction standing. USER-CONFIRMED at Gate 1 (2026-08-29). | Split keeps the belt shippable and testable alone; the transport is its own design space (envelope, at-least-once, GC). |
| D8 | Preamble on Pi: the full session preamble injects ONCE per `session_start`; each turn's `before_agent_start` appends only a slim delta (phase, gates, waiting-on, blocker counts) — never the full block per turn. USER-CHOSEN at Gate 1 (2026-08-29). | Token cost; Pi's chained systemPrompt makes once+delta natural. |

### Agent's Discretion

Extension file internals (state handling, env detection, TS idioms matching
`bee-guard.ts`), catalog row naming, onboard step naming, test file
placement. The extension must stay passive (no-op, no errors) when
`.bee/bin/bee` is absent from the workspace — bee-less repos that open Pi
must feel nothing except D3's blocking rule when bee IS present.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| belt | One harness's translation layer from its native events onto `bee hook <rule>` |
| blocking surface | A hook rule whose deny must stop the tool call (write-guard, model-guard) |
| advisory surface | A hook rule whose failure must never break the host (activity, state-sync, context) |

## Existing Code Context

### Reusable Assets

- `.opencode/plugins/bee-guard.ts` — the sibling belt: exec-helper shape, blocking/advisory policy table, Node-strippable TS. The Pi extension is this file re-targeted at Pi's event names.
- `packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs` — fixture suite pattern: stub `.bee/bin/bee` (deny/allow/crash/absent) under a real `node` subprocess + the belt parity test to extend.
- `packages/bee-rs/crates/bee/src/onboard/apply.rs:362-369` — `copy_opencode_plugin`: the ship-from-own-tree onboard step to mirror.
- `~/.pi/agent/extensions/herdr-agent-state.ts` (local machine) — working Pi extension proving the event API (`session_start`, `agent_start`, `agent_settled`, ctx.sessionManager).

### Established Patterns

- Pi 0.84.3 shipped docs `docs/extensions.md` (version-matched): `tool_call` returns `{block, reason, terminate?}`; `before_agent_start` may return `{systemPrompt}`; handlers chain; sibling tool preflights are sequential.
- `models.<runtime>` role tables + `herding.agents` argv entries (`pi-agy-flash-3.7`, `pi-opencode-free` already show the `pi -a --model <provider>/<model>:<thinking>` shape).

### Integration Points

- `packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs` — the catalog of record (D4).
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` + `models.rs` — runtime enum (`RUNTIMES`), `--runtime` validation, slot resolution (D5).
- `packages/bee-rs/crates/bee/src/onboard/apply.rs` — new copy step (D1).
- `.bee/config-sample.json` — `models.pi` sample block (D6).

## Canonical References

- docs/history/research/pi-harness-support.md — the belt research (event map, extension locations, install evidence).
- docs/history/research/pi-peer-distill.md — the transport research feeding `pi-result-mailbox` (D7).
- skills/bee-hive/references/gates-and-delegation.md — dispatch-door law the pi runtime must honor.

## Outstanding Questions

### Resolve Before Planning

- [x] D7 split — CONFIRMED by the user at Gate 1 (2026-08-29); recorded in the store.
- [x] Preamble cost — RESOLVED as D8 (once per session_start + slim per-turn delta); recorded in the store.

<!-- bee:not-a-deferral: both questions were answered in execution (pis-1, commit 9a36fe28): the carrier is before_agent_start per D8, and detection is the candidateBinaries chain — cwd .bee then git-common-dir main root, checked per call. This section records the shaping→planning handoff; it promises no future work -->
### Deferred To Planning

- [x] Prompt-context carrier — ANSWERED: `before_agent_start` (D8; pis-1).
- [x] Bee-repo detection — ANSWERED: cwd `.bee` then git-common-dir main-root walk, checked per call (pis-1).
<!-- /bee:not-a-deferral -->

## Handoff Note

<!-- bee:not-a-deferral: template boilerplate describing how planning consumes this record — machinery description, not a promise to act later -->
CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads
locked decisions, code context, canonical references, and
deferred-to-planning questions. Planning's Gate 2 shape stage and reviewing
use locked decisions for coverage and UAT.
<!-- /bee:not-a-deferral -->
