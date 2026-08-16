---
date: 2026-08-16
feature: worktree-first-enforcement
categories: [pattern]
severity: normal
tags: [worktrees, dispatch, herding, doctrine]
---

# Learning: A mechanical guard without triage doctrine still stalls the run

**Category:** pattern
**Severity:** normal
**Tags:** [worktrees, dispatch, contention]
**Applicable-when:** an unattended session meets file overlap with an
in-flight cell or live worktree and has to decide between working,
deferring, or asking.

## What Happened

A beedashboard session (bee 2.6.3) stopped mid-run to ask the user how
to proceed: every remaining backlog item touched files an in-flight
cell held. The worktree-first write guard was working as designed — the
stall was doctrinal, not mechanical. No rule told the agent that
overlap is triage data it resolves alone, and herding dispatch ranked
candidates overlap-blind, able to spawn an agent straight into a known
merge collision.

## Root Cause

Enforcement shipped in two halves and only one existed. The Rust guard
(hard deny on same-workspace holds, advisory leases across worktrees)
covers the mechanical half; the behavioral half — what an agent does
when semantic overlap appears — was never written down, so the agent
escalated instead of triaging.

## Resolution

Doctrine and skill text only (CONTEXT.md D3 — no code change):

- AGENTS block, Multi-session etiquette: overlap is triage data —
  disjoint items first, natural scope split second, overlapped
  remainder deferred with a recorded reason and one report line; ask
  the user only when the deferred set is the entire explicit ask.
- `skills/bee-herding/references/role-dispatch.md` §7 "Rank
  overlap-aware": dispatch skips candidates overlapping held files.
- Synced `docs/knowledge/areas/worktree-parallelism/routing-and-visibility.md`
  ("Contention is triage data", 2026-08-16).

Shipped in commit b25101fe. Zero cells: docs-lane feature, so the
promote proposal was empty by construction — its review result is
decision `66e985af`.

## Takeaway

When a guard is already correct and a run still stalls, look for the
missing behavioral rule before touching the guard. A hook can deny a
write; only doctrine can tell the agent what to do next.
