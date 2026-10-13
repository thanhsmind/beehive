# Claude read-only Paseo workers — Context

**Feature slug:** claude-read-only
**Date:** 2026-10-12
**Shaping session:** complete (gate bypass full; the user chose "Paseo claude, bản mới" for the codex gpt roles)
**Scope:** High-risk (a guard change)
**Domain types:** RUN

## Feature Boundary

The user moves the codex roles that use gpt-5.6-sol and gpt-6-astra to Claude
Sonnet 5.5 and Claude Opus 5.5 on the Paseo `claude` provider. Advisor, hat and
lane roles are read-only dispatches (paseo-leverage D2, store `168b8037`), and
`bee herding run --read-only` refuses every Paseo provider except `pi` today.
This feature makes a read-only Paseo `claude` worker read-only for real, so
those roles can run there. The pi path and herdr/tmux runs do not change.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | A read-only `bee herding run` on Paseo provider `claude` is allowed: it sets `BEE_HERDING_READ_ONLY=1` exactly as the pi path does and passes the configured `paseo.mode` unchanged. Every provider other than `pi` and `claude` still refuses before spawn with a FIX line naming both. | Spike `spike-claude-1` (2026-10-12): a Paseo claude worker carries `PASEO_AGENT_ID`, `BEE_HERDING_WORKER` and `BEE_HERDING_JOB_ID`, and bee's Claude hooks run inside it (it wrote `activity.json`). |
| D2 | The Claude hook manifest gains one PreToolUse group, Claude only, matcher `Edit|Write|MultiEdit|NotebookEdit|Bash`, running `bee hook worker-guard`, in both renderers (`devtools/hook_manifests.rs` and `onboard/hooks_wiring.rs`). Outside a herded worker the hook returns at once (`worker_guard::run` exits success when no marker env is set). | The guard already judges Claude-shaped Write, Edit and Bash payloads; only the wiring is missing. |
| D3 | Inside a Paseo claude worker the same hook also applies mode A (the worker-outward guard of paseo-pi-hardening D5) to its shell commands, and mode B to a Claude supervisor that runs with `BEE_SUPERVISOR_ALLOWED`. Both are intended. | One guard for every Paseo-carried worker; mode A was already meant for every Paseo worker, not only pi. |
| D4 | Hat-wave revision: a read-only run on provider claude fails closed before spawn. `execute_paseo` refuses with a FIX line unless `<main root>/.bee/bin/bee hook --help` lists `worker-guard` AND `<main root>/.claude/settings.json` has a PreToolUse entry running `hook worker-guard`. | The Claude hook command exits 0 when no binary resolves, and an old binary short-circuits the verb, so without this check a read-only claude worker could run unguarded; the pi belt already fails closed the same way. |
| D5 | Hat-wave revision: the worker-guard group matcher is `Edit|Write|MultiEdit|NotebookEdit|Bash|Monitor|EnterWorktree|ExitWorktree|mcp__.*`, with no status message. In read-only mode `worker_guard.rs` judges `Monitor` by its `command` field like Bash, and denies `EnterWorktree`, `ExitWorktree` and every `mcp__` tool by name; `cd` joins the read-only allowlist. | These tools write or run shell outside the first matcher; a status line on every tool call is noise for non-herded sessions. |

## Environment facts

- Paseo 0.11.2 claude provider models: `claude-opus-5-5`, `claude-sonnet-5-5` (`paseo provider models claude`).
- `.claude/settings.json` PreToolUse today: write-guard (Edit|Write|MultiEdit|Bash|Read|Glob|Grep|AskUserQuestion), model-guard (Agent|Task), activity (*). No worker-guard.

## Out of scope

- Usage and cost for claude workers (inspect `LastUsage` has no numbers; the Claude transcript is a later source; trigger `a-feature-picks-up-the-backlog-row-recor__ce30c059`).
- Changing the team role mapping (the user edits `.bee/config.json`; the config guard refuses the agent).
