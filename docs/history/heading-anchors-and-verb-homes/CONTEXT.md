# Heading-anchored section parsers and a doctrine home for every verb — Context

**Feature slug:** heading-anchors-and-verb-homes
**Date:** 2026-11-19
**Backlog items:** p-35008d7f, p-6061cd5f
**Shaping session:** complete under gate bypass full (owner said go on the 2026-11-19 backlog sweep)
**Scope:** Standard
**Domain types:** READ

## Feature Boundary

Section parsers find a heading only on a line that is the heading. A new test
fails when a shipped bee command is named in no doctrine home and is not in an
allowlist with a reason.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `parse_agents_rule_homes` (`packages/bee-rs/crates/bee/src/verbs/knowledge/ownership.rs`, near lines 130 and 152) and `tests/rule_index_parity.rs` (near line 110) find the section by a line that equals the heading text exactly at line start. A test proves that a paragraph quoting the heading earlier in the file does not move the section. One shared helper serves both callers. | p-35008d7f. First-occurrence `find()` can be hijacked by prose. |
| D2 | A test walks the command index of `bee --help --all --json` and fails on any command named in none of: `skills/`, `AGENTS.md`, `CLAUDE.md`, `packages/bee/AGENTS.block.md`, `packages/bee/LEADER.md`, `packages/bee/playbooks/`, `packages/bee/prompts/`, `docs/knowledge/`. Internal commands sit in an explicit allowlist file, each with a one-line reason. | p-6061cd5f; pattern docs/knowledge/patterns/20260831-a-shipped-verb-with-no-doctrine-home-is-unreachable.md. `docs/knowledge/` is added as a home because concepts are where current behavior lives. |
| D3 | The first run of D2 records the baseline: each unnamed command is either given a home (one line in the owning skill or concept) or added to the allowlist with a reason. No command stays silent. | The rule is "a named decision, never a silence". |

## Read set

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/knowledge/ownership.rs` | D1. |
| `packages/bee-rs/crates/bee/tests/rule_index_parity.rs` | D1. |
| `packages/bee-rs/crates/bee/src/verbs/help.rs` | Command index source for D2. |

## Not In Scope

- Rewriting doctrine beyond one naming line per command.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
