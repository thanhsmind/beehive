---
type: bee.area
title: "Project map — where bee lives in this repository, and which area to open next"
description: "The orientation file for a reader with no starting point: what this repository builds, the entry points, where each major component lives, the conventions that hold across the tree, the commands that build and test, and one routing row per knowledge area."
tags: [project, orientation, map]
timestamp: 2026-11-14
bee:
  id: area-project-overview
  lifecycle: active
  areas: [project]
  required_context: []
  decisions: ["knowledge-skill-pair D8 (e: the setup skill writes the orientation file at docs/knowledge/areas/project/overview.md)"]
  sources: [README.md, packages/bee-rs/Cargo.toml, packages/bee-rs/crates/bee/src/main.rs, packages/bee-rs/crates/fleet/Cargo.toml, .bee/config.json, .github/workflows/ci.yml, scripts/release.sh]
  authoritative_for: "project: the layout of the repository and the routing row for each knowledge area"
  owns.code: []
  owns.skills: [skills/bee-knowledge-setup/SKILL.md]
  owns.tests: []
  anchors: [README.md, packages/bee-rs/Cargo.toml, packages/bee-rs/crates/bee/src/main.rs, packages/bee-rs/crates/bee/src/router.rs, packages/bee-rs/crates/bee/src/verbs/mod.rs, packages/bee-rs/crates/fleet/Cargo.toml, .github/workflows/ci.yml, scripts/release.sh]
  verified_at: 2026-11-14
  verified_commit: cdefa65590c37a8f7f9a716d23cd23df9a082c54
---

# Project map

Read when: you open this repository cold and need to know where a component lives and which area concept to read next.

**What this is a map of.** The tree at commit `cdefa6559` (release 2.99.0),
read on 2026-11-14. It names where things live and which area concept to open
next. It holds no behavior: each area concept holds that. After a structural
change (a new crate, a moved directory, a new area), re-derive this file from
the tree. Do not patch it line by line.

## What the repository builds

bee is a gated workflow frame for coding agents (Claude Code, Codex, and the
herded runtimes). It ships as one native CLI, `bee`, plus the skills, hooks,
agent templates and craft guides that onboarding installs into a host
project. This repository is also a host of bee: it builds bee with bee, so
`.bee/` here is a live store, not a fixture.

## Entry points

| Entry | What starts there |
|---|---|
| `packages/bee-rs/crates/bee/src/main.rs` | The CLI. `main` wraps the router call in `output_spill::run`, then asks `router::try_native`; an argv shape no verb claims ends in `router::emit_unsupported_shape`. |
| `packages/bee-rs/crates/bee/src/verbs/mod.rs` | The probe order: one line per verb group. Open it to find which module serves a command. |
| `packages/bee-rs/crates/bee/src/hooks/` | The lifecycle hook handlers the host runtimes call (`session_init`, `prompt_context`, `write_guard`, `model_guard`, `worker_guard`, …). |
| `skills/<name>/SKILL.md` | The skill sources. Onboarding renders them into `.claude/skills/` and the Codex skill tree. Never edit the rendered copies. |
| `.bee/bin/bee` | The vendored binary that sessions in this repository run. It changes only with a release, so a source change is not live in `.bee/bin/bee` until then. |

## Where the major components live

Paths under `packages/bee-rs/crates/bee/src/` unless the row says otherwise.

| Component | Where | Area to open |
|---|---|---|
| Command verbs | `verbs/` (one module or directory per group: `cells`, `knowledge`, `worktree`, `state_group`, `status_full`, …) | [rust-runtime](../rust-runtime/overview.md), then the area that owns the verb |
| Workflow store and gates | `verbs/workflow_store/`, `verbs/state_group/`, `state.rs` | [workflow-state](../workflow-state/overview.md) |
| Hooks | `hooks/` | [hook-runtime](../hook-runtime/overview.md) |
| Onboarding (the installer) | `onboard/` | [onboarding](../onboarding/overview.md) |
| Worktrees, locks, leases, merge queue | `verbs/worktree/`, `roots.rs`, `lock.rs`, `lease_store.rs`, `integration_queue.rs` | [worktree-parallelism](../worktree-parallelism/overview.md) |
| Herding (panes, sessions, waves, control loop) | `herding/`, `herding.rs` | [bee-herding](../bee-herding/overview.md) |
| Generic worker coordination | `packages/bee-rs/crates/fleet/` — waves, tasks, waiting, result collection; it knows nothing about cells, gates or proof | [bee-herding](../bee-herding/overview.md) |
| Knowledge bundle tooling | `verbs/knowledge/` | [okf-profile](../okf-profile/overview.md) |
| Decisions | `verbs/decisions/` | [decision-memory](../decision-memory/overview.md) |
| Test runner and proof | `verbs/test_runner.rs`, `verbs/cells/` | [verify-pipeline](../verify-pipeline/overview.md) |
| Dev surface (`bee dev …`) | `devtools/` | [rust-runtime](../rust-runtime/overview.md) |
| Doctrine text the agents read | `packages/bee/LEADER.md`, `packages/bee/AGENTS.block.md`, `packages/bee/playbooks/`, `packages/bee/prompts/` | [doctrine-layer](../doctrine-layer/overview.md) |
| Agent templates, hook manifests, statusline | `packages/bee/agents/`, `packages/bee/hooks/`, `packages/bee/statusline/` | [onboarding](../onboarding/overview.md) |
| Craft guides | `expertise/` (vendored to `.bee/expertise/`) | — |
| Plugin manifests | `.claude-plugin/plugin.json`, `.codex-plugin/plugin.json` | — |

## Other knowledge areas

| Area | Open it when |
|---|---|
| [advisor-protocol](../advisor-protocol/overview.md) | A worker or leader consults an advisor, or a blind lane runs. |
| [discovery-wayfinding](../discovery-wayfinding/overview.md) | An idea has no nameable outcome yet, or a discovery map is open. |
| [feedback-digest](../feedback-digest/overview.md) | Friction is collected, ranked, or turned into a bee change. |
| [human-mailbox](../human-mailbox/overview.md) | An unattended run must leave a letter for the owner. |
| [performance-log](../performance-log/overview.md) | Token or timing numbers are recorded or reported. |
| [product-backlog](../product-backlog/overview.md) | A backlog item is added, ranked, or rendered. |

The full routed list of every concept is `docs/knowledge/index.md`.

## Conventions that hold across the tree

- **State goes through the CLI.** Every change to `.bee/*.json` or
  `.bee/*.jsonl` is a `bee` verb. The write guard refuses a direct edit.
- **No new code comments.** Three exceptions only: a `#!` line, a `SAFETY:`
  line, a license header. `.bee/comment-baseline.json` holds the count the
  old ported source still carries; the suite goes red when that count rises.
  The reason a rule exists lives in an area concept or a decision.
- **Refuse out loud.** An argv shape no verb claims, a failed guard, a held
  reservation: each one gives a named refusal and a non-zero exit.
- **Generated files carry a check.** The knowledge indexes, the hook
  manifests, the rendered skills and the release manifest each regenerate
  from one source, and each has a `--check` form that CI or the suite runs.
- **Tests live in two places.** Unit tests sit beside the code in
  `mod tests` blocks; contract suites sit in
  `packages/bee-rs/crates/bee/tests/` (for example `instruction_laws.rs`,
  `hook_contracts.rs`, `registry_contracts.rs`, `front_door.rs`).

## Build, test, release

| Job | Command |
|---|---|
| Build | `cargo build --release --manifest-path packages/bee-rs/Cargo.toml` |
| Full suite | `bee test`, which runs `commands.test` from `.bee/config.json`: `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` |
| CI | `.github/workflows/ci.yml` (Linux) and `.github/workflows/windows.yml` run the full suite on every push and pull request. |
| Release | `scripts/release.sh <VERSION>` only. It bumps the manifests, regenerates, runs the suite and the live smoke, tags, and waits for `release-binaries.yml`. |

## Dependencies that matter

The workspace has two crates, `bee` and `fleet`
(`packages/bee-rs/Cargo.toml`). `serde_json` runs with `preserve_order` so
the stores keep a stable key order. `unicode-normalization` exists for one
contract: the NFKC/NFD fold of knowledge subject keys, which std cannot do.

## Where other records live

| Path | What it holds |
|---|---|
| `docs/history/<feature>/` | Per-feature records: `CONTEXT.md` with the locked decisions, plans, reports. |
| `docs/specs/` | The read-only compatibility surface. New behavior goes into `docs/knowledge/`, not here. |
| `docs/knowledge/patterns/` | Cross-area patterns, each with its own read-when line. |
| `.bee/verify/verify-app/features/` | One file per user-facing feature: how to reach it and drive it. |
