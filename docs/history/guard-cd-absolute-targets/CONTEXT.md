# Absolute write targets after a cd — Context

**Feature slug:** guard-cd-absolute-targets
**Date:** 2026-11-10
**Shaping session:** complete (the owner hit the refusal and asked for a root fix)
**Scope:** Small (one guard rule and its tests)
**Domain types:** RUN

## Feature Boundary

The Bash write guard refuses every write target that follows a `cd` in the
same compound command. That is correct for a relative target, because the
guard resolves it against the original working directory. It is wrong for an
absolute target, because a `cd` cannot change where an absolute path points.
After this feature, an absolute target after a `cd` goes through the same
checks as a target with no `cd`. Relative targets after a `cd` are still
refused.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | A write target after a `cd` is cd-opaque only when it is a relative path. An absolute target is resolved and checked like a target with no `cd`: containment, direct-edit and every other check still apply. Relative targets after a `cd`, and every target before a `cd`, keep their current behavior. (d199ac87, contract guard-cd-absolute-targets) | The `cd` rule of write-guard-hardening D4 exists only because cwd-relative resolution goes stale. |

### Agent's Discretion

- Where the absolute check lives: in `extract_bash_targets` when it builds `cd_opaque`, or at the refusal site.

### Known costs the owner accepted

- A `~/...` target after a `cd` stays refused. Only `Path::is_absolute` paths are exempt.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md | confirmed | States the `cd` refusal this feature narrows. |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs` | `extract_bash_targets` builds `cd_opaque`. |
| `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs` | The refusal site for a cd-opaque target. |
| `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs` | The cd-opaque tests. |

<!-- bee:not-a-deferral: template section names with no open question under them, not a promise to act later -->
## Outstanding Questions

None.
