# Brief intent and WIP — Context

**Feature slug:** brief-intent-and-wip
**Date:** 2026-11-17
**Scope:** Standard
**Domain types:** BUILD

## What was asked

The owner read Addy Osmani's "Your coding agents made you a manager" and asked
which of its practices bee should take. Of the gaps found, the owner chose
option A: give each worker brief its reason and its non-goals, and add a
"should we build this" check plus a work-in-progress limit for new work.
Pruning hooks that only state a preference is not part of this feature.

## What will be done

A cell tells its worker why the work matters and what it must leave alone.
New backlog work is asked who needs it, how success shows and what stops it.
Opening a feature while many others are still open prints a warning.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | A cell's `must_haves` gains `why` (a non-empty string: the reason the work matters) and `out_of_scope` (an array of strings; `[]` means none). Lanes `standard` and `high-risk` require both at the plan-packet check of `bee gate --preview` only; `bee cells add` and `bee cells update` keep today's rules, because a cell added from an approved preview must equal its packet, and features approved before this change must still add their cells (refinement f459fb3f). The worker-cell prompt names both parts: the why is what the worker uses to choose where the cell is silent; out-of-scope work is never done, and a need there is reported in `found` or returned as `[PARKED]`. | 66f6a453 |
| D2 | bee-shaping's Qualify step asks three questions of every new item: who needs it, how we will know it worked, and what makes us stop. An item that cannot answer the first two parks, with the gap written into Outstanding Questions. | 66f6a453 |
| D3 | A config key `wip_limit` (a non-negative integer; absent means 5; 0 turns the check off). `bee feature start` counts the other lanes in a working phase (exploring, planning, swarming, compounding, scribing) and, when that count is at or above the limit, prints one warning line naming the count, the limit and the three oldest open features. It never refuses. | 66f6a453 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/doctrine-layer/per-task-route-tester-pairing-and-proof-floors.md | unverified | the worker prompt and cap rules biw-1 extends |
| docs/knowledge/patterns/20260821-instruction-text-is-an-untested-code-path.md | confirmed | the prompt and skill text changes need pinned tests |
| docs/knowledge/patterns/20260818-a-green-count-is-not-evidence-that-your-new-test-ran.md | confirmed | testers quote the filter count |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs` | the plan-packet check and the approved-preview match at add |
| `packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs` | add and update call the approved-preview match |
| `packages/bee/prompts/worker-cell.md` | what a worker reads about its cell |
| `packages/bee-rs/crates/bee/src/verbs/feature_group.rs` | `bee feature start` |
| `skills/bee-shaping/SKILL.md` | Qualify triage |
| `skills/bee-planning/references/cells.md` | the cell contract plan writers follow |
