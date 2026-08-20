# Defaults And Agent Env — Context

**Feature slug:** defaults-and-agent-env
**Date:** 2026-08-20
**Shaping session:** complete (user-directed defaults change, standard lane)
**Scope:** Quick

## Feature Boundary

Flip two shipped absent-key defaults (staging off, uat door at close so the merge lands on main while the worktree is held for testing), pre-seed the herd registry with `claude-sonnet` and `agy-flash` so `bee herding run --agent` works with zero config, and let a registry entry carry its own env vars applied in the pane before the agent starts. Public-contract change; samples and docs updated in the same slice.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `uat_stop` absent-key default flips `Merge` → `Close`: with both keys absent, `bee worktree merge` lands the feature on main and the `uat` door sits at `bee close`, with the worktree branch HELD while uat is pending. The `uat_before_merge` alias mapping is untouched (`true`→merge, `false`→off) | User default: test on main directly, keep the wt branch around for convenient testing; explicit keys still override |
| D2 | `staging_before_merge` absent-key default flips `true` → `false`: staging is opt-in; absent or `false` refuses `bee staging add`/`rebuild` with `STAGING_DISABLED`, explicit `true` enables. Refusal text updated to name the new default | User default: no staging step; worktree → merge to main → uat at close |
| D3 | Built-in default herd registry: when `herding.agents` is absent or missing these names, the registry pre-seeds `claude-sonnet` → `["claude","--model","sonnet","--permission-mode","bypassPermissions"]` and `agy-flash` → `["agy","--dangerously-skip-permissions"]`. Config entries with the same names override the built-ins; `UnknownAgent` listings include the built-ins | `--agent agy-flash` / `--agent claude-sonnet` must work on a fresh repo with no herding block |
| D4 | A `herding.agents` entry accepts a second shape: `{"argv": [...], "env": {"KEY": "value"}}` beside the plain argv array. Env is applied inside the freshly split pane BEFORE `agent start`, via herdr's pane text/run channel as one `export K='v' ...` line; keys must be `[A-Za-z_][A-Za-z0-9_]*`, values newline-free and single-quote-escaped — anything else drops the entry (same fail-open-per-entry rule the registry already uses). Array-shape entries and the default `agent_command` path carry no env | Per-agent credentials/endpoints (e.g. a different API key per herd agent) need to reach the agent's process without leaking into bee's own env |

## Evidence

- `packages/bee-rs/crates/bee/src/uat.rs:30-47` — current absent-key default `Merge`.
- `packages/bee-rs/crates/bee/src/verbs/staging/mod.rs:350-356` — current absent-key default `true`.
- `packages/bee-rs/crates/bee/src/herding/wave.rs:139-164,171-186,208-227` — registry parse (array-only today) and resolution.
- `herdr pane --help` — `send-text`, `send-keys`, `run` exist; `herdr agent start` has no env flag, so env must be set in the pane shell before start.
