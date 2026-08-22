---
type: bee.area
title: "Worktree Parallelism — the two kinds of parallelism, and where this area stops"
description: "The difference between swarm-worker worktrees that only remove git-index contention and independent-feature worktrees that each run their own full bee lifecycle, plus the surfaces this area deliberately leaves out of scope."
timestamp: 2026-07-22
bee:
  id: worktree-parallelism-overview
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: []
  decisions: ["worktree-feature-parallelism (shipped 2026-07-16, unreviewed)", "worktree-session-routing D7/D8/D9 (enter/return commands + routing rule, 2026-07-18, GH #21, unreviewed)", "cross-worktree-holds D1-D6 (the shared holds ledger, 2026-07-20)", "hardening-1-7-10 (atomic hold acquisition + heartbeat renewal, 2026-07-21, unreviewed)"]
  sources: [docs/history/worktree-feature-parallelism/, docs/history/worktree-session-routing/, "docs/specs/worktree-parallelism.md#S-what-problem-this-solves", "docs/specs/worktree-parallelism.md#S-boundary-out-of-scope"]
  authoritative_for: "worktree-parallelism: purpose, the two kinds of parallelism, and the area boundary"
  owns.code: ["packages/bee-rs/crates/bee/src/verbs/worktree/*", "packages/bee-rs/crates/bee/src/verbs/staging/*"]
  owns.skills: []
  owns.tests: [packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs]
---

# Worktree Parallelism — Purpose and Boundary

**Area:** how one session fans independent work into git worktrees, each running its
own bee lifecycle, reconciled to the main checkout on `git merge`.

**Status:** shipped 2026-07-16 (unreviewed); enter/return commands + routing rule added
2026-07-18 (worktree-session-routing, GH #21, unreviewed); cross-worktree hold acquisition
made atomic (single-lock conflict-check + reserve + ledger-insert) and holds gained
heartbeat-renewal on top of their TTL ceiling, 2026-07-21 (hardening-1-7-10, unreviewed).
History: `docs/history/worktree-feature-parallelism/`, `docs/history/worktree-session-routing/`.

Every other concept in this area describes one mechanism — the trust model, the
commands that enter and return, the routing rule, the shared holds ledger, the
store's lifecycle tiers. This one describes what the whole thing is for, and
which neighbouring problems it deliberately does not solve.

## What problem this solves

Two different kinds of parallelism exist in bee:

- **Swarm-worker worktrees (P40, pre-existing):** an orchestrator dispatches many workers
  into worktrees that all share ONE coordination store at the main checkout and work under
  ONE feature's single gate. Worktrees only remove git-index contention.
- **Independent-feature worktrees (this area):** a worktree runs its OWN full bee lifecycle
  — its own phase, gates, and store — so a session can advance several independent features
  at once and merge each back. This is what P40 deliberately did NOT provide.

The distinction is load-bearing everywhere else in this area. P40's worktrees are a
performance device: many workers, one feature, one gate, one store. This area's worktrees
are an isolation device: one feature each, its own gate, its own store — and therefore its
own trust question, its own return path, and its own way of staying visible to the
checkouts beside it.

## Boundary (out of scope)

- Same-feature / same-file concurrent WRITE-INTENT across worktrees is now covered by the
  shared holds ledger (above); concurrent read visibility across worktrees stays out of
  scope by design — digests and the merge gate remain the interface.
- Rollout to onboarded host repos — deferred; proven in bee's own repo first.
- P40 swarm-worker behavior — unchanged; this area coexists beside it.
