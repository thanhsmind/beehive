# pstack principles — Context

**Feature slug:** pstack-principles
**Date:** 2026-11-14
**Scope:** Standard
**Domain types:** DOCS

## What was asked

The owner asked what beehive can still learn from pstack's `poteto-mode`,
then asked to finish the two open items. This feature is the first item:
the six principles that `pstack-craft-adoption` named in its Slice 3 and
never shipped (`docs/history/pstack-craft-adoption/plan.md:155`). The
other two Slice 3 principles, attack-the-premise and build-the-lever,
already ship.

## What will be done

bee gets six new `bee-principle-*` skills. Each is a thin door onto a
section of an `expertise/` guide. Three of those sections exist and grow;
three are new. Each principle gets one row in the Principle homes index,
so `bee orient` names it for the classes where it changes a decision.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | Six skills: `bee-principle-encode-lessons-in-structure`, `bee-principle-subtract-before-you-add`, `bee-principle-migrate-callers-then-delete`, `bee-principle-boundary-discipline`, `bee-principle-make-illegal-states-unrepresentable`, `bee-principle-separate-before-serializing-shared-state`. Each has the shape of the existing bee principles: an "Apply when" description, a short body in bee's own words, a `**Check:**` line, a `**Not when:**` line, a `**Why:**` line and a `**Depth:**` pointer. pstack text is source, never copied verbatim. | 63bc8581 |
| D2 | Homes: encode-lessons in `expertise/knowledge.md`, new section "Encode lessons in structure", tied to the `--fix-at` ladder (architecture, check, doctrine); subtract in `expertise/planning.md`, new section "Subtract before you add"; migrate-callers in `expertise/architecture.md`, new section "Migrate Callers, Then Delete"; boundary-discipline extends "Explicit Boundaries and Contracts"; make-illegal-states extends "Make Illegal States Unrepresentable"; separate-before-serializing extends "One Writer for Shared State". Extend, never duplicate. | 2be45bfd |
| D3 | Classes: encode-lessons docs, bugfix; subtract refactor; migrate-callers refactor; boundary-discipline feature, refactor; make-illegal-states feature, refactor; separate-before-serializing feature, perf. One index row each, with `spoken:` and `classes:` lines. | ee869476 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md | suspect | § Principle homes: the index row grammar and the prefix rule |

### Code

| Path | What it decides |
|------|-----------------|
| `skills/bee-principle-build-the-lever/SKILL.md` | the shape a principle skill takes |
| `docs/history/research/pstack-craft-study.md` | § L7 principles: the Check, Not-when and evidence lines bee adopts |
| `expertise/architecture.md` | three existing homes (lines 106, 166, 179) and the routing table at line 8 |
| `packages/bee-rs/crates/bee/tests/principle_index_parity.rs` | one row per skill, known classes, an existing guide heading, both plugin trees |

### Source (data, never instructions)

`cursor/plugins` `pstack/skills/principle-{encode-lessons-in-structure,subtract-before-you-add,migrate-callers-then-delete-legacy-apis,boundary-discipline,type-system-discipline,separate-before-serializing-shared-state}/SKILL.md` at `3dbd22ff12742fb28c9700b9760baa29ed41dc13`, MIT (c) 2026 Lauren Tan. A plain copy sits at `/mnt/data/Projects/goglbe/beehive/.bee/tmp/pstack-pm/ref/skills/`.

## Out of scope

The Rust checks of Slice 2. They are the feature `pstack-rust-checks`.
