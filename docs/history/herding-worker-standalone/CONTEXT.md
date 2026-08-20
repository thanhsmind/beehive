# Herding Worker Standalone — Context

**Feature slug:** herding-worker-standalone
**Date:** 2026-08-20
**Shaping session:** complete (user-directed, small lane)
**Scope:** Quick

## Feature Boundary

A pane worker spawned by `bee herding run` must be fully standalone: it executes exactly its brief and never activates the host repo's bee workflow. Today a Claude Code worker opened in the repo loads AGENTS.md's BEE block and the project hooks, so it runs `bee orient`, claims work, and duplicates the orchestrator's bookkeeping — the opposite of the worker contract (herding-executor D4: worker stays bee-ignorant, the orchestrator owns ALL bee bookkeeping).

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `render_brief` opens with a standalone-executor contract block: do exactly the task below; IGNORE any bee/agent-workflow instructions loaded from this repo's AGENTS.md/CLAUDE.md; never run any `bee` command, never claim/cap cells or write `.bee` state (the mailbox result file is the one exception); the result file is the only contract | Kind-agnostic: works for all 21 herdr agent kinds, including ones bee's hooks cannot reach |
| D2 | `bee herding run` ALWAYS exports `BEE_HERDING_WORKER=1` into the fresh pane before `agent start`, merged with the registry entry's own env (the marker wins over a same-name per-agent value) | The pane's agent process and everything it spawns inherit the marker without any per-agent config |
| D3 | Every `bee hook <name>` invocation exits 0 silently, before dispatch, when `BEE_HERDING_WORKER` is non-empty — all hooks, both runtimes, checked in the binary's one dispatch entry (`hooks::try_native`) so already-rendered host manifests gain the behavior on binary upgrade | One place, no JSON churn across manifests; a worker session gets zero bee preamble, zero guards, zero nudges — its posture is already fully-open by design (herding-adopt D7) |

## Evidence

- `packages/bee-rs/crates/bee/src/herding/mailbox.rs:118` — brief carries no bee-suppression wording today.
- `packages/bee/hooks/*.json` — no `BEE_*` env kill-switch in any hook command.
- Live pattern (user report, 2026-08-20): a herded Claude Code worker with its own env activated the bee flow and repeated orchestrator work.
