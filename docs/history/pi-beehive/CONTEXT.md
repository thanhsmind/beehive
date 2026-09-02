# pi-beehive — Context

**Feature slug:** pi-beehive
**Date:** 2026-09-02
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN

## Feature Boundary

bee's Pi belt reaches behavior parity with the Claude belt: every bee hook rule
the Claude belt fires, the Pi belt fires too on Pi's matching lifecycle event, and
the two verdict shapes bee already emits but Pi currently swallows — an
`updatedInput` repair and an `ask` verdict — become a real in-place argument patch
and a real confirmation dialog instead of a block. It ends at parity: no new
user-facing Pi surface, no change to how bee ships, no change to how bee dispatches
workers on Pi.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Scope is **parity with the Claude belt, nothing more**. In scope: wire every Pi lifecycle event bee already wires on Claude; apply bee's `updatedInput` repair by mutating `event.input` in place instead of blocking; turn bee's `ask` verdict into a real `ctx.ui.confirm` dialog instead of a block; enforce the continuation nudge on `agent_settled` via `pi.sendMessage(…, {triggerTurn: true})`. Store id **1edd8a90**. | Parity is a bounded, testable target the existing parity test already knows how to prove. |
| D2 | **Out of scope: any Pi-native user-facing front** — no `/bee` slash commands, no `ctx.ui` gate dialogs, no bee status widget, no message renderer. A Pi user sees nothing new; bee only behaves the same. Store id **1edd8a90**. | A native UX front is a second product surface with no parity oracle to test it against. |
| D3 | **Shipping is unchanged**: the belt stays in-repo at `.pi/extensions/bee-guard.ts`, populated by `bee onboard`, with `scripts/install.sh` / `scripts/install.ps1` as the one-line install path. No separate pi package, no `.pi-plugin` manifest, no new artifact in `scripts/release.sh`. Store id **ec85c30c**. | A package would add a release artifact and a second version surface for a belt onboarding already places correctly. |
| D4 | **Dispatch is untouched.** Pi worker dispatch stays herding-only; `dispatch prepare --runtime pi` keeps refusing every non-herding slot with `pi_requires_herding`. `model-guard` and `chain-nudge` stay NAMED EXCLUSIONS on the Pi belt. Store ids **9f5c6d17**, standing on **7f9c8518** and **8650ca7b**. | Reaffirmed by the user on 2026-09-02 with this brief's evidence in hand. |
| D5 | **"pi" means the `pi` binary only**, `@earendil-works/pi-coding-agent` 0.84.x, config at `.pi/` and `~/.pi/agent/`. The `omp` fork is not a target: no `.omp` belt, no `omp` entry in `RUNTIMES`, no omp-native surface used. Store id **5d87f14e**. | One documented surface for the parity test to prove. |
| D6 | The **two failure policies stay exactly two and never mix**: `tool_call` is BLOCKING and fails **closed**; every other wired event is ADVISORY and fails **open**. Every event added under D1 declares which one it takes. Existing rule, restated because D1 adds events. | A fail-open host swallowing a fail-closed throw turns a deny into an allow — the one failure mode the blocking path must never have. |
| D7 | **Passivity holds unchanged**: a repo with no `.bee` directory at the project root or the main worktree root runs nothing and prints nothing, re-checked on every call, on every event added under D1. Existing rule, restated because D1 adds events. | — |

### Agent's Discretion

Delegated to the agent, inside D1's boundary:

- Which Pi event carries which bee rule, where the mapping is not one-to-one —
  provided every Claude-side rule lands somewhere and the choice is recorded.
- Slice order and cell shape.
- Whether a Claude-side rule with no Pi analog becomes a named exclusion or an
  Open Gap — but the choice is asserted **by name** in the parity test either way,
  never left silent.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Parity | The Pi belt fires the same bee hook *rules* the Claude belt fires. It does **not** mean the same event names, the same number of events, or the same UI. |
| Belt | The single hand-written TypeScript file that translates one harness's events onto `bee hook <rule>`. Pi's is `.pi/extensions/bee-guard.ts`. |
| Named exclusion | A Claude-side rule the Pi belt deliberately does not wire, asserted by name in the parity test so the absence is proven, never assumed. |

## Specific Ideas And References

- The user asked to learn Pi communication patterns from `pi-workflows`. What that
  yields for D1 is **one** pattern, already evidenced: `pi.sendMessage(payload,
  {triggerTurn: true})` gated on `ctx.isIdle() && !ctx.hasPendingMessages()`, with
  the sent entry located afterwards by scanning `ctx.sessionManager.getBranch()`
  for your own `details` id — because `sendMessage` does not return the entry id.
  Everything else pi-workflows does (its graph engine, SQLite host, controllers)
  is out of scope by D1 and D2.

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### The parity table this feature closes

Claude belt rows, from `packages/bee/hooks/claude-hooks.json`, against the Pi belt
as it stands in `.pi/extensions/bee-guard.ts`:

| Claude event | bee rules | Pi analog | Wired today |
|---|---|---|---|
| `SessionStart` | `session-init` | `session_start` | yes |
| `UserPromptSubmit` | `prompt-context`, `activity` | `before_agent_start` | `prompt-context` only |
| `PreToolUse` | `write-guard`, `model-guard`, `activity` | `tool_call` | `write-guard` only; `model-guard` excluded per D4 |
| `PostToolUse` | `state-sync`, `tools-logger`, `activity` | `tool_result` | `state-sync` only |
| `PostToolUseFailure` | `activity` | `tool_result` with `isError` | no |
| `PermissionRequest` | `activity` | none confirmed | no |
| `SubagentStop` | `chain-nudge`, `state-sync` | none | excluded per D4 |
| `PreCompact` | `session-close` | `session_before_compact` | no |
| `Stop` | `session-close`, `state-sync`, `activity` | `agent_settled` | `session-close` only |
| `Notification` | `activity` | none confirmed | no |
| `SessionEnd` | `session-close`, `activity` | `session_shutdown` | no |

### Reusable Assets

- `.pi/extensions/bee-guard.ts` — the belt itself. Holds zero rules of its own;
  every verdict comes from `.bee/bin/bee hook <name>`. The two failure policies,
  the store/binary re-resolution, the passivity check and the tool map are already
  written and must be extended, not replaced.
- `.opencode/plugins/bee-guard.ts` — the sibling hand-written belt. Same shape,
  useful as the second data point for anything ambiguous.
- `packages/bee/hooks/claude-hooks.json` — the parity source of truth for which
  rule fires on which event.

### Established Patterns

- Helpers stay the first belt on every runtime: the belt translates, `bee hook
  <rule>` decides. No rule logic in TypeScript.
- Pi and OpenCode are named exclusions in `hook_manifests.rs`'s `Runtime` enum
  (`Claude, Codex` only) — their belts are hand-written files, not rendered
  manifests. This feature does not add a `Runtime::Pi` or a fourth projection.

### Integration Points

- `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` — the parity test
  derives its rows from the TS source. Every row added or excluded under D1 lands
  here.
- `packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs` — the catalog's
  named-exclusion comments for Pi.
- `docs/config-reference.md` (§ Pi) — the user-facing description of the belt.

## Canonical References

- `docs/history/research/pi-workflows-xia.md` — the evidence this feature rests on:
  Pi 0.84.4's real extension surface, bee's current Pi baseline, and the two
  now-stale gap claims this feature corrects.
- `~/.local/share/mise/installs/pi/0.84.4/pi/docs/extensions.md` — Pi's own event
  catalog and API shapes, version-matched. `tool_call`'s `event.input` is
  documented mutable ("Mutate it in place to patch tool arguments before
  execution"), and its return shape is `{ block, reason, terminate }`.
- `docs/history/pi-support/CONTEXT.md` — the belt's original decisions (D1-D8),
  extended here, retired nowhere.
- `docs/knowledge/work/pi-result-mailbox/` — the async result inbox this feature
  must not disturb.

## Outstanding Questions

### Resolve Before Planning

None. D1-D7 are sufficient to plan against.

### Deferred To Planning

- [ ] Does Pi 0.84.4 emit any event that maps to Claude's `Notification` or
      `PermissionRequest`? Neither appears in Pi's documented lifecycle. Read the
      full event catalog; if there is none, each becomes a named exclusion
      asserted in the parity test, per Agent's Discretion.
- [ ] Where does `activity` belong on Pi? Claude fires it on six events; Pi's set
      is different in both shape and count. The rule must land somewhere for every
      Claude row, and the mapping must be recorded.
- [ ] Can an `ask` verdict block on `ctx.ui.confirm` inside a `tool_call` handler
      without deadlocking the turn, and what does the handler do when
      `ctx.hasUI` is false (RPC, print, headless)? A no-UI session has no dialog
      to show, so the fail-closed policy of D6 decides the fallback.
- [ ] Does mutating `event.input` in place interact safely with bee's write-guard
      re-check, given Pi performs no re-validation after the mutation?
- [ ] `tools-logger` on `tool_result` — confirm the payload bee's helper expects
      is reachable from Pi's `tool_result` event fields.

## Deferred Ideas

Out-of-scope ideas captured during shaping. Not lost, not planned.

- A Pi-native front: `/bee` slash commands, gate dialogs via `ctx.ui.select`, a bee
  status widget, a message renderer for bee's injected result headers — declined
  under D2 as a second product surface.
- Publishing bee as a pi package (`pi install git:…`) — declined under D3.
- Wiring bee's own subagent tool on Pi via `pi.registerTool` — declined under D4;
  the evidence stays in the research brief.
- An `omp` belt — declined under D5.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
