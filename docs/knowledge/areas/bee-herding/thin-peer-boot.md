---
type: bee.area
title: "Bee Herding — thin peer boot, what a herded Claude worker loads, and the switch that is on by default"
description: "The trimmed start a herded Claude worker gets: a four-variable env layer set by bee herding run, a hook that prints only the short peer preamble, the dispatch.thin_peer_boot switch that is on unless it is explicitly false, the pilot numbers behind that default, and what a peer still loads."
timestamp: 2026-11-07
bee:
  id: bee-herding-thin-peer-boot
  lifecycle: active
  areas: [bee-herding]
  required_context: [areas/bee-herding/overview.md]
  decisions: ["thin-worker-boot D1 (a dispatched peer knows no bee flow: peer role text plus brief)", thin-worker-boot D2 (one thin shared instruction file holds the hard boundaries; flow doctrine stays with the leader), thin-worker-boot D3 (the peer ends with the one finish command its brief names), thin-worker-boot D4 (pane peers launch trimmed; project-level guard hooks stay on), thin-worker-boot D6 (the thin boot ships behind a config switch and becomes default only on a measured saving), "thin-worker-boot D8 (two launch layers: an env layer always on with the switch, and an optional flag layer)", thin-worker-boot D10 (the four env variables plus the role-aware preamble; partial by design)]
  sources: [docs/history/thin-worker-boot/CONTEXT.md, docs/history/thin-worker-boot/plan.md, docs/history/thin-boot-default/plan.md]
  authoritative_for: "bee-herding: the thin peer boot — the env layer, the peer preamble hook, the dispatch.thin_peer_boot switch and its default"
  owns.code: [packages/bee-rs/crates/bee/src/hooks/mod.rs, packages/bee-rs/crates/bee/src/hooks/session_init.rs, packages/bee/prompts/peer-boot.md]
  anchors: [packages/bee/prompts/peer-boot.md, packages/bee-rs/crates/bee/src/hooks/mod.rs, packages/bee-rs/crates/bee/src/herding/run.rs, packages/bee-rs/crates/bee/src/hooks/session_init.rs]
  verified_at: 2026-11-14
  verified_commit: cdefa65590c37a8f7f9a716d23cd23df9a082c54
---

# Bee Herding — thin peer boot, what a herded Claude worker loads, and the switch that is on by default

Read when: a herded Claude worker boots without the context you expected, or you change the env a herding launch sets.

## What it is

A herded Claude worker is a peer: it does one job from its brief and hands the result back
(thin-worker-boot D1). The thin peer boot keeps the bee flow doctrine out of that worker's start,
so the worker's first turns carry the brief and not the leader's rulebook.

It has two layers (thin-worker-boot D8), and both read one switch.

## The env layer

`bee herding run` adds four variables to the environment of every Claude worker it starts
(thin-worker-boot D10):

| Variable | Value | What it drops |
|---|---|---|
| `CLAUDE_CODE_DISABLE_CLAUDE_MDS` | `1` | The project and user instruction files (AGENTS.md, CLAUDE.md) |
| `CLAUDE_CODE_DISABLE_AUTO_MEMORY` | `1` | The user's auto memory |
| `ENABLE_CLAUDEAI_MCP_SERVERS` | `false` | The claude.ai connector MCP servers |
| `CLAUDE_CODE_DISABLE_BUNDLED_SKILLS` | `1` | The skills Claude Code ships with |

The layer covers a Paseo claude agent, a herdr or tmux pane whose command is `claude`, and a
claude-session worker. A Pi, Codex or other agent gets nothing new. A value the agent's own `env`
block sets for one of these names wins over the default.

## The peer preamble

A herded worker normally exits 0 silently from `bee hook session-init`. With the thin boot on and
`BEE_HERDING_WORKER` set, that hook prints only `packages/bee/prompts/peer-boot.md` — the hard
boundaries and the one finish instruction (thin-worker-boot D2, D3) — and registers no session.
Without the marker the hook never takes this path, so a leader or any non-herded session keeps its
full boot.

## The switch, and why it is on by default

`dispatch.thin_peer_boot` decides both layers. One function owns the rule: absent, `true`, or any
non-boolean value means on; only an explicit `false` means off. With `false`, the env layer adds
nothing and the hook boots the worker in full, exactly as before the thin boot existed.

Both layers read one config view: the main checkout's tracked `.bee/config.json` merged with its
`.bee/config.local.json`. `bee herding run` reads it from the main root; the hook resolves the same
root from a linked worktree, whose gitignored `config.local.json` is absent. An explicit `false`
anywhere in that view therefore turns the env layer and the hook off together, and a worker never
boots half thin.

The thin boot shipped behind this switch, off, until a measured saving earned the default
(thin-worker-boot D6). A one-task pilot measured about 25% less boot context and about 22% fewer
fresh tokens, and on 2026-11-07 the owner chose to make it the default.

## What a peer still loads

The layer is partial by design (thin-worker-boot D10). A thin peer still loads user and plugin
skills and local MCP servers, and the project's guard hooks stay on (thin-worker-boot D4). Set
`dispatch.thin_peer_boot` to `false` in `.bee/config.json` to give every herded worker the full
boot again.

## Pointers

- `packages/bee-rs/crates/bee/src/hooks/mod.rs` — `thin_peer_switch_on`, the one function that holds the rule, and `thin_peer_boot`, which adds the marker requirement and reads the switch from the control root's merged config.
- `packages/bee-rs/crates/bee/src/herding/run.rs` — `thin_peer_env`, which calls the rule and adds the Claude-only requirement.
- `packages/bee-rs/crates/bee/src/hooks/session_init.rs` — prints the preamble when `thin_peer_boot` is true, before any session is registered.
- `packages/bee/prompts/peer-boot.md` — the preamble text.
