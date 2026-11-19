# Orient principle list and onboarding knowledge notice — Context

**Feature slug:** orient-and-onboard-notices
**Date:** 2026-11-19
**Backlog items:** p-806a3911, p-b366b91c
**Shaping session:** complete under gate bypass full (owner said go on the 2026-11-19 backlog sweep)
**Scope:** Standard
**Domain types:** READ

## Feature Boundary

`bee orient --class <c>` prints a ranked core set of principles in full and
the rest by slug. Onboarding tells a project when its knowledge bundle is
missing or due for upkeep.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `principle_lines` (`packages/bee-rs/crates/bee/src/principles.rs`) prints the top `principles_cap` (default 7) principles in full and every other principle of the class by slug on one overflow line. The cap check on `--principle-used` still accepts any slug of the class. A test pins the shape. | p-806a3911. The cap already exists; the overflow line must name the slugs. |
| D2 | `bee onboard` prints a notice when `docs/knowledge/index.md` is absent (offer `bee-knowledge-setup`) or when `bee upkeep due` reports knowledge upkeep due (point at `bee-knowledge-upkeep`), next to the existing verify notice in `onboard/notices.rs`. `bee knowledge report` carries one upkeep-due line. Tests cover both notices and the silent case. | p-b366b91c; docs/history/knowledge-skill-pair/CONTEXT.md Out Of Scope. |

## Read set

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/principles.rs` | D1. |
| `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs` | D1 caller. |
| `packages/bee-rs/crates/bee/src/onboard/notices.rs` | D2. |
| `packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs` | D2. |
| `packages/bee-rs/crates/bee/src/verbs/upkeep_group.rs` | D2 due source. |

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
