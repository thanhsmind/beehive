# bee-herdr Skill — Context

**Feature slug:** herdr-skill
**Date:** 2026-08-30
**Shaping session:** complete
**Scope:** Quick
**Domain types:** ORGANIZE

## Feature Boundary

One new shipped skill, `skills/bee-herdr`, teaching correct herdr
terminal-pane transport usage, built RED-first per bee-writing-skills. No
code changes; bee-herding stays the cockpit-role home.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 (fe29c892) | The skill packages transport craft: prompt delivery only through `bee herding run` (never raw send-keys or typed text), submission verified as started, ready-wait before sending, stall recovery for typed-but-not-entered prompts, outcomes read through `herdr-result`. | Observed failure: a dispatcher typed a task into a pane and the Enter never registered. |
| D2 (148a185c) | Transport only; cockpit roles stay in bee-herding, cross-referenced, never duplicated. | One home per rule. |

## Existing Code Context

- `skills/bee-herding/` — cockpit roles (bootstrap/dispatch/merge); the new skill cross-references it.
- `docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md` — the run verb's signal ladder; source of truth for the rules the skill states.
- `packages/bee-rs/crates/bee/src/herding/run.rs` — the safeguards (ready-wait, verify, nudge) the skill routes agents onto.

## Outstanding Questions

### Deferred To Planning

(none — the bee-writing-skills cycle is the plan)

## Handoff Note

CONTEXT.md is the source of truth. The bee-writing-skills RED→GREEN→REFACTOR→VALIDATE cycle governs the build; the gather digest supplies the facts.
