---
artifact_contract: bee-plan/v1
mode: standard
approved_gate2: 2026-08-22 (auto, gate_bypass normal)
---

# Plan: agent-activity-hook

Mode: `standard` — 3 risk flags: public-contracts, external-systems, cross-platform
Why this is the least workflow that protects the work: a new on-disk shape that another
product reads, fed by a harness payload bee does not control, across hook code, two manifest
renderers, and the status readers — a plan plus three bounded cells keeps the writer, the
wiring, and the readers from agreeing on three different shapes.

## Requirements (from CONTEXT.md)

- **D1** — one Rust verb `bee hook activity`, stdin payload, keyed on `hook_event_name`;
  eight events, never SubagentStop; always exit 0; capped `.bee/logs/hook-activity.log`;
  declared in the manifest source, rendered into `.claude/settings.json`.
- **D2** — `.bee/sessions/<id>.json` gains `activity{...}`; `<id>.activity.jsonl` keeps the
  last 50 transitions, atomic trim; a missing session file is created minimally.
- **D3** — event→state mapping; sticky `waiting_input`/`blocked`; `blocked` clears only on
  the matching `tool_use_id` (or `tool_name` fallback) or a turn boundary.
- **D4** — `signal: live|no_signal` computed at read (90 s) in `state session list --json`
  and `status --json`; never stored.
- **D5** — `waiting_input`/`blocked` set the `waiting_on` mark (question/gate); a turn
  boundary clears a hook-set mark; an agent-set mark is never overwritten.

## Discovery

Inspected (gather digest, 2026-08-22): `hooks/mod.rs:74` `HOOK_NAMES` + match arm `:129`;
`hooks/tools_logger.rs:44` as the handler model (`read_hook_context` → `bee_installed` →
`hook_enabled` → work → `log_crash` → `SUCCESS`); session records are untyped
`serde_json::Map` under `<control_root>/.bee/sessions/<id>.json`, written with
`fsutil::write_json_atomic` (hooks wrap it in `state_sync.rs:230 write_json_atomic_retry`);
every session enumerator filters on `.json`, so `.activity.jsonl` is invisible to them.
`state session list` emits raw records (`state_group/sessions.rs:147`) — `activity` surfaces
for free, `signal` must be computed there; `status --json` has a projected `workers` row at
`status_full/cells.rs:767-790` that must gain the fields explicitly. Two manifest renderers
exist and are NOT derived from each other: `devtools/hook_manifests.rs:106 CATALOG` (plugin
projections, byte-compared by `hook_manifests_match_disk`) and
`onboard/hooks_wiring.rs:77 render_repo_hook_entries` (repo `.claude/settings.json`).
PostToolUseFailure, PermissionRequest, Notification have no precedent row in either.
`waiting_on` store functions live in `verbs/workflow_store` (`set_workflow_waiting_on`,
`set_default_state_waiting_on`, `clear_*`); target resolution is
`waiting_on.rs:80 resolve_waiting_on_target`; after a lane write both
`rebuild_lane_projection` and `rebuild_state_projection` must run.

## Approach

Recommended (D1–D5): one new hook module that owns the state machine and the store writes;
the two renderers gain the same rows (Claude-only — Codex hooks have no such events); the
readers derive `signal`. Rejected: a shell shim per event (second mechanism, no fail-open
guarantees); waggledance installing its own hooks (it would own a second writer into `.bee/`).

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| `hooks/activity.rs` | MEDIUM | sticky-state logic, concurrent hook invocations on one file | unit tests over the transition table; `hook_contracts` adversarial-stdin matrix |
| manifest renderers | MEDIUM | two sources must agree; drift gate byte-compares disk | `hook_manifests_match_disk`, `hooks_wiring` tests green after `--write` |
| readers (`session list`, `status`) | LOW | additive fields | unit test on the 90 s rule |
| `waiting_on` from a hook | MEDIUM | must never overwrite an agent-set mark | unit test: agent mark survives a hook `blocked` |

## Shape

One slice, three cells — the feature is one capability, not a sequence of demos.

| Cell | What changes | Why now | Proof |
|---|---|---|---|
| aah-1 writer | `hooks/activity.rs` + registration; D2/D3/D5 | everything else reads what it writes | `cargo test activity`, `hook_contracts` |
| aah-2 wiring | CATALOG + `hooks_wiring` rows, rendered projections, `.claude/settings.json` | the verb must exist before it is wired | `cargo test hook_manifests hooks_wiring` |
| aah-3 readers | `signal` in `session list` / `status`, knowledge docs | independent of the writer; reads the recorded shape | `cargo test sessions`, `bee knowledge check` |

Deps: aah-2 → aah-1. aah-3 runs in parallel with aah-1.

## Test matrix

- Happy: UserPromptSubmit→working; PermissionRequest→blocked; matching PostToolUse→working;
  Notification agent_needs_input→waiting_input; Stop→idle; SessionEnd(other)→exited.
- Edge: PostToolUse with a different `tool_use_id` leaves `blocked`; SessionEnd reason
  `clear`/`resume` does not mark exited; 51st transition trims to 50; missing session file is
  created; `activity.at` 91 s old → `no_signal`, dead session → no `signal`.
- Error: empty/garbage/2 MB stdin → exit 0, a line in `hook-activity.log`, no session write;
  outside a bee repo → exit 0.

## Out of scope

waggledance's reader; Codex/opencode equivalents; SubagentStop; changing heartbeat staleness.
