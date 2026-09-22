---
type: bee.area
title: Hook Runtime — the worker-outward guard
description: "Why a shell request from inside a linked worktree is refused a push, a GitHub write or an unsanctioned agent launch in every phase, which reads and which launches stay allowed, where the opt-out is read from, and why the main checkout's verdicts are byte-identical."
tags: [hook-runtime, guards, worktree]
timestamp: 2026-09-22
bee:
  id: hook-runtime-hook-runtime-the-worker-outward-guard
  lifecycle: active
  areas: [hook-runtime]
---

## Purpose

A helper that works inside a linked git worktree shares the machine, the network and the operator's credentials, and until this guard nothing stopped it from publishing: a push runs CI on every branch, rebuilds the public site on main, and builds release assets from a tag; a GitHub write opens or merges a pull request; a nested agent launch starts a second, unsupervised worker. The idle intake gate refused a push only in a terminal phase, so a worker mid-feature was unguarded (worker-outward-guard D1). This guard refuses those three outward forms from any shell request whose working directory is a linked worktree, in every phase, and tells the caller the sanctioned path for its own case.

Adapted from the Seatworks plugin, which denies `git push`, `gh` and agent launches for every seat on every harness (`docs/history/research/seatworks-xia.md`, item A8).

## Entry Points & Triggers

- Every shell request the write guard judges — the Claude `Bash` tool, the codex `exec` tool, and the pi and opencode belts, which hand their shell tool to this one check as a Bash request (worker-outward-guard D9).
- The location decides, not the caller: a cell worker, a herding pane and a human in their own worktree are all refused the same way (worker-outward-guard D1, D10).
- Reading the opt-out: one config key in the MAIN checkout's configuration, default on; the worktree's own tracked copy of the file is never read (worker-outward-guard D6, D11).

## Data Dictionary

| Term | Meaning |
|---|---|
| outward form | A command whose effect leaves the machine or starts another agent: a push, a GitHub write, a nested agent launch |
| linked worktree | A checkout whose `.git` is a file pointing back into the main repository's worktree list and whose back-link round-trips; the guard's resolution value for it is `linked-valid` (worker-outward-guard D1, D10) |
| read-only gh form | A `gh` command on the allowlist: pull-request, run, issue, release, repo, workflow, search, cache, label and auth reads, plus `api` with an explicit GET and `api graphql` with no mutation (worker-outward-guard D3, D10) |
| sanctioned launch | A nested agent command the dispatch door itself returns: one that starts with a cli command configured for a role, or a codex exec carrying a read-only sandbox (worker-outward-guard D10) |
| command head | The first token of a shell segment after wrappers, environment assignments, `sudo`, `nohup`, `timeout <n>`, leading flags and group openers are skipped (worker-outward-guard D10) |

## Behaviors & Operations

**Refuse a push** (worker-outward-guard D2): any git invocation in the request whose subcommand is `push`, in every spelling the guard already resolves — a repository flag before the verb, a wrapper shell, a compound line — is refused. A push inside a heredoc body is not seen, because bodies are fenced; a bare `echo git push` is refused, because the git finder scans every token.

**Refuse a GitHub write** (worker-outward-guard D3, D10): a `gh` command head is allowed only when its subcommand pair is on the read-only list after global flags are skipped; `--input` is always refused; a glued or `=`-joined method other than GET is a write; `api graphql` is a read unless a token is `mutation`.

**Refuse an unsanctioned launch** (worker-outward-guard D4, D10): a command head of `claude`, `codex`, `pi` or `opencode` is refused unless it is a sanctioned launch. The `bee` command is never judged, so dispatch, herding and every bee verb run as before.

**Say the remedy for the form** (worker-outward-guard D5, D11): a push refusal tells a cell worker to stop and report blocked and tells a human to land through the worktree merge from main, releases through the release script from main; a GitHub-write refusal names the same landing path and lists the reads that still run; a launch refusal names the dispatch door. Every refusal names the current worktree id and ends with the opt-out, spelled with the MAIN checkout's configuration file and the note that the worktree's copy is not read.

**Never delegate on an opaque request** (worker-outward-guard D11): when the tokenizer gives up on a wrapper nest, the guard scans the flat tokens and refuses on a push, a `gh` head or an agent head; otherwise it records the gap and allows, as the git check does.

**Leave main byte-identical** (worker-outward-guard D7): a request from the main checkout produces the same verdict and the same text as before this guard existed; the idle intake refusal of a push at a terminal phase is untouched. Inside a worktree at idle, the outward text takes precedence over the intake text.

## Business Rules

- One arm inside the existing git bash check, after the git invocations are found and before the empty-invocation return, so one fence and one tokenize pass serve both checks (hat wave, alternatives seat).
- The configuration is read only when a form is about to be refused, so a broken configuration never turns a harmless command into a delegate.
- The allowlist has one prose home, the configuration reference; the contracts document and the verify map point at it.

## Edge Cases Settled

- `git push --dry-run` is refused: the safe-form table has no push arm, and the idle gate refuses it too.
- A `gh` alias is refused: the matcher sees the alias token, never what it expands to.
- A worktree whose main is a submodule, or one set up with a separate git directory, resolves as ordinary and gets no refusal; the guard fails open to the old behavior there.
- A configuration copy flipped inside a worktree changes nothing there and reaches main only as an ordinary diff at merge; the merge diff is the signal.

## Open Gaps

- Refusing a raw push from the main checkout in execution phases, leaving the release script as the only push path, is filed as backlog; hosts push branches from main today.
- A doctor row that reports the key's state, the read-side signal for a flipped copy that landed at merge, is filed as backlog.
- The guard judges command lines, never script contents; a script that pushes is not seen.

## Pointers (implementation)

- `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs` — `outward_arm` inside `check_git_bash_command`; `gh_read_only_form`, `gh_api_read_only_form`, `gh_api_method` beside `idle_gate_safe_form`.
- `packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs` — `OutwardForm`, `outward_fix_line`.
- `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs` — the one-argument change at the shell call site.
- `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs` — the nine `worker_outward_*` tests.
- `docs/config-reference.md` § `guards.worker_outward` — the allowlist's prose home; `.bee/verify/verify-app/features/worker-outward-guard.md` — the drivable recipe.
- `docs/history/worker-outward-guard/CONTEXT.md` — decisions worker-outward-guard D1-D11; cells wog-1, wog-2 (capped 2026-09-23, merged at a05e1b0).
