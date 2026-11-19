# Write guard: cli slots, research class, leader live proof — Context

**Feature slug:** guard-config-and-class
**Date:** 2026-11-19
**Backlog items:** p-28defd13, p-69bee217, p-b9e2a334
**Shaping session:** complete under gate bypass full (owner said go on the 2026-11-19 backlog sweep)
**Scope:** Standard
**Domain types:** RUN

## Feature Boundary

Three fixes in the write guard (`packages/bee-rs/crates/bee/src/hooks/write_guard/`).
The outward guard reads cli slots from both config keys. A research-class
route refuses a source edit. A leader session can run its own live proof in its
feature worktree, and a worker start there is still refused.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `configured_cli_command` (checks.rs, near line 1170) reads cli slots from `team` AND `models`, and a command declared in either is recognized. A test covers a config that holds both keys. | p-28defd13. Today `team.or_else(models)` hides every `models` slot when `team` exists. |
| D2 | While the recorded route class of the bound lane is `research`, the write guard refuses an Edit/Write to a source path (anything outside `docs/`, `.bee/`, and the feature's `docs/history/<feature>/`). The refusal names the class and the remedy: re-route with `bee orient --class <c>` to a writing class. A test proves the refusal and proves a docs write is still allowed. | p-69bee217. pstack-adoption D3 states research is read-only; nothing enforced it. |
| D3 | The worker-outward guard (`outward_arm`, checks.rs near 984-1078) lets a session start pi/claude/codex in a linked worktree when that session holds the worktree grant and has no claimed cell. A session with a claimed cell, or with no grant, is refused as today. Tests prove both arms. | p-b9e2a334. The leader could not run its own live proof (seen 4 times in docs/history/pi-1-0-upgrade/promote-proposals.md). This touches worker-outward-guard D1 (location, not caller): the location rule stays for workers; the leader with the grant is the one named exception. |

## Read set

### Knowledge

| Concept | Why it matters |
|---------|----------------|
| docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md | Guard arms and refusal texts. |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs` | D1, D2, D3. |
| `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs` | Guard tests. |
| `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs` | Where the route class is recorded (D2). |

## Not In Scope

- Any other class gaining a write rule.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
