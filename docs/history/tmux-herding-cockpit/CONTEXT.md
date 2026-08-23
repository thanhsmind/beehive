# tmux Herding Cockpit — Context

**Feature slug:** tmux-herding-cockpit
**Date:** 2026-08-23
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | CALL

## Feature Boundary

Phase 2 of the tmux transport: the whole herding cockpit — waves, occupancy,
the control-pane allowlist, bootstrap, and the dispatch/merge roles — runs on
tmux when `herding.transport` is `tmux`, with herdr behavior byte-identical
when it is not. It ends at the pane verbs: no new role, no new safety rule,
no change to the enable interlock or the merge gesture.

## Locked Decisions

Inherited, cited never re-decided: tmux-herding-transport D1 (config key, no
auto-detect), D2 (split panes in the caller's window, split lock), D3 (dialog
= blocked, never type into it), D4 (screen status advisory, mailbox files the
truth), D5 (source manifest).

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Phase 2 is the FULL cockpit on tmux — waves, occupancy, allowlist, bootstrap, dispatch/merge roles — behind the same `herding.transport` key. | User's pick; one key keeps the two cockpits from diverging. |
| D2 | Cockpit roles and bootstrap act on panes ONLY through transport-neutral bee verbs: `bee herding pane <current\|list\|split\|run\|send-text\|read\|rename\|close\|layout\|tab-create\|tab-list\|tab-focus>`, `bee herding agent-start`, `bee herding pane-id --label`, `bee herding result`. No raw `herdr` or `tmux` in a role document or the bootstrap script. | A cold control agent reads one vocabulary. |
| D3 | tmux mapping: workspace = the caller's current tmux session; tab = window (`cockpit`, `runtime`); pane label = pane title (`select-pane -T`); chat pane = the pane bootstrap ran from; label lookup reads `list-panes` `pane_title`. | tmux has no workspace/label objects; these carriers survive reattach. |
| D4 | Waves and occupancy select the tmux backend from the same key; ONE screen classifier (markers + stability) serves both crates — it lives in `fleet`, and bee's `RealTmux` reuses it. | bee depends on fleet; two classifiers would drift. |

### Agent's Discretion

- The JSON envelope shape of the pane verbs (one uniform `{result: …}` per
  verb, stable across transports) and the trait split that carries the new
  operations.
- Exact tmux format strings; whether the bootstrap script stays bash or
  becomes a bee verb (bash, calling bee verbs, is the default).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| pane verb | A `bee herding pane …` / `agent-start` command that performs one pane action on whichever transport the key names |
| label | herdr pane label; on tmux the pane title |
| workspace | herdr workspace; on tmux the current session |
| tab | herdr tab; on tmux a window |

## Specific Ideas And References

- Phase-1 record: `docs/history/tmux-herding-transport/CONTEXT.md`, plan
  `docs/history/tmux-herding-transport/plan.md` (shape table, phase 2 row).
- Research: `docs/history/research/tmux-herding-transport.md`.

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/herding/run.rs:409` — `pub(crate) trait PaneTransport` (14 methods incl. `name()`), `RealHerdr` (private struct at :512), `transport_for_run(main_root)`, `read_main_config` (:2478).
- `packages/bee-rs/crates/bee/src/herding/tmux.rs` — `RealTmux`, `TmuxSettings`, `classify`, stub-tmux test helper.
- `packages/bee-rs/crates/bee/src/herding.rs:598-615` — `TransportKind`, `transport_kind`, `transport_kind_at`; router at :108-124; `herdr_result` (:854), `herdr_pane_id` (:915).
- `packages/bee-rs/crates/fleet/src/backend.rs` — `WorkerBackend` (canonical_id/start/status/send/read_output), `herdr.rs` (`new`, `with_test_seams`, `child_path` PATH-prepend), `fake.rs`; `fleet/tests/herdr_backend.rs` stub pattern (library crate: integration tests work here).
- `packages/bee-rs/crates/bee/src/herding/wave.rs` — `wave()` :630, `real_backend_ctor` :583, `live_pane_ids_via_herdr` :763, `occupancy()` :825 (caller :851).
- `packages/bee-rs/crates/bee/src/herding/control_loop.rs:212` `allowed_tools_for(role)` (byte-copied `Bash(herdr:*)…`), `resolve_iteration_argv(main_root, role, …)` :288.

### Established Patterns

- Runtime adapter keys under `herding.*` in `.bee/config.json`.
- Control pane runs an enumerated `--allowedTools`, never bypassPermissions (operational-invariants "permission posture").
- Bootstrap idempotency: refuse when a pane labelled `dispatch` already exists.

### Integration Points

- `skills/bee-herding/scripts/bootstrap-cockpit.sh` (19 herdr calls: workspace list/pane list/tab create×2/pane split×2/pane run/rename/close) and `references/role-bootstrap.md` (5).
- `skills/bee-herding/references/role-dispatch.md` (41 herdr lines: pane current/rename/layout/read/send-text/list/split/close, agent start, tab list/focus), `role-merge.md` (13), `wave-runs.md` (4), `dispatch-prompt.md` / `merge-prompt.md` ("the herdr workspace").
- `docs/knowledge/areas/bee-herding/{overview,waves-and-occupancy,agent-resolution-and-spawn-commands,the-run-verb-and-worker-outcomes}.md`, `operational-invariants.md` "Runtime adapter", `SKILL.md` frontmatter dependencies.

## Canonical References

- `docs/knowledge/areas/bee-herding/overview.md` — roles and safety boundaries (unchanged).
- `docs/history/tmux-herding-transport/CONTEXT.md` — D1–D5 of phase 1.

## Outstanding Questions

### Resolve Before Planning

- none

### Settled In Planning

<!-- bee:not-a-deferral: answered by plan.md; recorded as facts -->
- New pane operations ride a second trait `CockpitTransport: PaneTransport` implemented for both `RealHerdr` and `RealTmux` in the pane-verb module, so the phase-1 trait and its test fakes stay untouched.
- The bootstrap script stays bash and calls bee pane verbs.
- Live proof (phase 3) is an owner-run cockpit bootstrap on tmux; recorded as the uat test.
<!-- /bee:not-a-deferral -->

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and the settled-in-planning facts.
