# thin-boot-default — Plan

**Lane:** small · **Class:** feature · **Date:** 2026-11-07

## Problem

Thin peer boot (thin-worker-boot D1-D10) ships behind
`dispatch.thin_peer_boot` and is on only where a config sets it to `true`.
The owner chose on 2026-11-07 (option B) to make it the default on the
one-task pilot: -25% boot context, -22% fresh tokens.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | the hook path reads the switch as on only for an explicit true | read | packages/bee-rs/crates/bee/src/hooks/mod.rs:111 | `Some(serde_json::Value::Bool(true))` |
| 2 | the herding env path reads the switch as on only for an explicit true | read | packages/bee-rs/crates/bee/src/herding/run.rs:2939 | `let on = cfg.pointer("/dispatch/thin_peer_boot").and_then(Value::as_bool) == Some(true);` |
| 3 | the claude-session transport applies the same env | read | packages/bee-rs/crates/bee/src/herding/run.rs:3968 | `let session_env = thin_peer_env(&read_main_config(&opts.main_root), true, &BTreeMap::new());` |
| 4 | no knowledge concept describes thin peer boot | ran | `rg -ln "thin.peer\|thin_peer\|thin boot" docs/knowledge` | no output |

## Contract (cell TBD-1)

- One function owns the rule: `dispatch.thin_peer_boot` absent, `true`, or
  any non-boolean value means on; only an explicit JSON `false` means off.
  Both readers (`hooks/mod.rs` `thin_peer_boot`, `herding/run.rs`
  `thin_peer_env`) call it. The hook keeps its herding-worker marker
  requirement; the env keeps its Claude-only requirement.
- Tests: absent config turns thin boot on in both paths; explicit `false`
  turns it off in both; explicit `true` stays on; no marker still means no
  thin hook boot. Existing tests that assumed absent = off are updated to
  the new rule, not deleted.
- Docs: a new concept `docs/knowledge/areas/bee-herding/thin-peer-boot.md`
  (what a peer loads and does not load, the env layer, the switch and its
  default, the pilot numbers, what still loads: user and plugin skills and
  local MCP servers), linked from the area overview; cite
  `docs/history/thin-worker-boot/CONTEXT.md` D-numbers only.
