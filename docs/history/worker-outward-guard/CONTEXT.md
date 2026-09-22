# Worker Outward Guard — Context

**Feature slug:** worker-outward-guard
**Date:** 2026-09-22
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN

## Feature Boundary

The write guard refuses, from any shell call whose working directory is inside a linked feature worktree, the three outward-facing forms an execution worker never owns: `git push`, a `gh` command that writes, and a nested agent launch. It holds in every phase. The main checkout keeps its verdicts byte for byte, so `scripts/release.sh`, orchestrator `gh` reads, and the existing idle-gate push refusal are untouched.

Source: `docs/history/research/seatworks-xia.md`, item A8 (Seatworks denies `git push`, `gh`, `paseo` and starting another agent for every seat on every harness: `plugin/harness/claude/settings.json:26-33`, `codex/rules/seat.rules:1-7`).

## What was found

- `git push` is refused today only in a terminal phase: `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:830-862` returns early when `!is_terminal_phase(phase)`, and the push arm sits below that return. Tests cover the idle fixture only (`tests.rs:2647-2655`, `4263-4270`). In an execution phase a worker can push.
- `gh` and the agent CLIs (`claude`, `codex`, `pi`, `opencode`) are not judged by any guard (no hit in `write_guard/*.rs`).
- Herding working agents run `bypassPermissions` with no tool list as an accepted risk (`skills/bee-herding/references/operational-invariants.md:9-33`); hooks still run in that mode, so a guard is the one fence that reaches them.
- The guard already knows the worktree fact: `ctx.worktree_resolution` (`main.rs:209`), and already reads a shell request past its first command, through wrappers and compound lines (`docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md`; `tests.rs:4263`).

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The guard's scope is the location, not the caller: every shell call the write guard judges whose cwd resolves inside a linked feature worktree, in every phase. Calls from the main checkout are unchanged. | Execution workers and herding panes both work in worktrees; main takes integration and release (AGENTS.md § Bee workflow). A location is checkable; a caller identity is not, for a pane. |
| D2 | Refused form 1: `git push`, in every spelling the guard already resolves a git invocation from (`git -C <path> push`, wrappers, compound lines, heredoc-adjacent). | — |
| D3 | Refused form 2: any `gh` command except a read-only allowlist: `gh pr view|list|status|checks|diff`, `gh run list|view|watch`, `gh issue view|list`, `gh release view|list`, `gh repo view`, `gh auth status`, and `gh api` with no `-X`/`--method` other than GET and no `-f`/`-F`/`--input`. Every other `gh` form is refused. | Mirrors the idle-gate git model: a read-only list plus safe spellings, deny the rest (decision bd57f530). |
| D4 | Refused form 3: a nested agent launch — command word `claude`, `codex`, `pi` or `opencode`, directly or through `npx`, `bunx` or `env`. `bee` itself is never judged; dispatch goes through `bee dispatch prepare`. | A worker executes exactly one cell (AGENTS.md); spawning is the leader's. |
| D5 | Every refusal names its remedy in the message: land through `bee worktree merge` from main; push and GitHub writes through `scripts/release.sh` from main; dispatch through `bee dispatch prepare`; and the opt-out key. | The guard's own rule: a deny names its remedy (AGENTS.md § Guardrails). |
| D6 | One config key, `guards.worker_outward` (boolean, default on; `false` turns the three refusals off), read the same way as `guards.idle_gate` (`checks.rs:835-838`). | A host with a PR-based flow can switch it off without a fork. |
| D7 | Deny-more only: for every request judged from the main checkout, the verdict and its text are byte-identical before and after. A test proves it, as decision 41a67ee2 did. | — |
| D8 | Red before green: the first test shows `git push`, a writing `gh`, and `claude -p` allowed from an executing-phase linked-worktree fixture, and goes green only when the guard refuses them. | bee-principle-red-before-green. |
| D9 | The codex belt carries the same three refusals under the existing parity gate (`docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md`); a difference is named, never silent. | — |

### Agent's Discretion

Where the check lives inside `write_guard/` (a new check in `checks.rs` or a sibling), the exact allowlist matching for `gh api`, the refusal wording beyond D5's four named remedies, and whether one doctor row reports the key's state.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| outward | A command whose effect leaves the machine or starts another agent: push, a GitHub write, a nested launch |
| linked worktree | A checkout `bee worktree new` created; `ctx.worktree_resolution` says so |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:830-900` — the idle-gate git check: phase read, config key read, READONLY list, safe-form table, the push arm, `intake_refusal`.
- `packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs:528-536` — `intake_fix_line`, the remedy pattern with the config opt-out named.
- `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs:2567-2573, 2590-2655, 4255-4270` — fixtures by phase, deny/allow helpers, compound-line cases.
- `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:205-215, 322-330` — shell detection and the worktree-resolution branch.

### Established Patterns

- Guard precision over blanket strictness: admit the read-only spelling, refuse the mutating one (decision bd57f530).
- A guard resolves its scope from the checkout it runs in, not the session binding (decision 41a67ee2).

### Integration Points

- `.bee/config.json` `guards` object — D6's key beside `idle_gate`.
- `docs/knowledge/areas/hook-runtime/` — the concept the feature adds or extends (capture at close).
- `.bee/verify/verify-app/features/` — no feature file maps the write guard's shell checks today; `comment-guard.md` is the nearest shape.

## Canonical References

- `docs/history/research/seatworks-xia.md` — A8 and the source anchors.
- `docs/knowledge/areas/hook-runtime/guard-precision-exemptions-and-remedies.md` — the safe-form model D3 mirrors.
- `docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md` — how a shell request is read.

## Outstanding Questions

### Deferred To Planning

- [ ] Does `ctx.worktree_resolution` distinguish a linked worktree from a nested or foreign checkout well enough for D1, or does D1 need the grant ledger? — read `main.rs:209` and its resolver.
- [ ] Which pi extension belt file mirrors D9 for the third runtime — check the parity coverage gate.

## Deferred Ideas

- Refusing raw `git push` from the main checkout in execution phases too, keeping only `scripts/release.sh` as the push path — needs its own shaping; hosts push PR branches from main.
- Reading a script's contents for pushes it contains — the guard judges command lines, not files.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
