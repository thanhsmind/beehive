# Skill Trigger Cases — Context

**Feature slug:** skill-trigger-cases
**Date:** 2026-09-22
**Shaping session:** complete
**Scope:** Quick
**Domain types:** RUN

## Feature Boundary

A fixture of user briefs that must and must not open each bee skill, a model-free test in the declared suite that refuses a skill with too few cases or a brief that names its own skill, and an opt-in eval that asks a real agent the same briefs. It ends at the skill descriptions: it never edits a skill body, and it never runs a model inside `commands.test`.

Source: `docs/history/research/seatworks-xia.md`, item A3 (Seatworks `test/skills/triggers.test.ts:23-28`: every skill needs three briefs that open it and two near misses that must not; the description line is the tested surface).

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Scope is every `skills/bee-*` skill whose description is a routing surface: the 20 skills that are not `bee-principle-*`. Principle skills are out: `bee orient` selects them from class and flags (decision 11221f5b), not from their description. | A trigger test on a skill nobody routes by description tests the wrong surface. |
| D2 | One fixture file, `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json`, holds every case. Nothing lands under `skills/`, so host packaging and onboarding enumeration are untouched. | `list_bee_skill_entries` copies `skills/bee-*` into hosts; a fixture beside them would ship to every host. |
| D3 | A case is one brief written as a user would type it, plus `expect` (the skills that must open, `[]` for none) and `near` (one skill that must NOT open). Per skill in scope: at least 3 cases expecting it and at least 2 cases nearing it. A near case says what opens instead through its own `expect`. | — |
| D4 | A brief may not contain the slug (`bee-xyz`) or slash form (`/bee-xyz`) of any skill it expects or nears. | A brief that names the skill tests string matching, not routing. |
| D5 | The model-free test runs inside the declared suite (`commands.test`). It reads `skills/*/SKILL.md` and the fixture and fails when: a skill in scope has fewer than 3 expect or 2 near cases; a case names a skill that does not exist; a brief violates D4; two cases carry the same brief. Each failure names the skill or brief. | The same shape as `pointer_integrity.rs`: read the source tree, refuse by name. |
| D6 | The eval that asks a real agent is opt-in and never part of `commands.test` or CI. It shows the agent the in-scope `name: description` list and one brief, asks for JSON `{"skills": [...]}`, runs each brief 3 times, and passes a case at 2 of 3 right. Right means: no `near` skill opened; an `expect: []` case opened nothing; otherwise at least one expected skill opened. The agent command comes from one environment variable; unset, the eval is skipped with that reason printed. | Paid and slow; its job is to measure descriptions by hand, not to gate merges. |
| D7 | `skills/bee-writing-skills/SKILL.md` checklist gains one row: a new or renamed skill adds its cases to the fixture, and the suite refuses a skill with none. No other skill text changes. | — |

### Agent's Discretion

Exact JSON field names beyond `brief`/`expect`/`near`, the test file name, the environment variable name, how the eval parses the agent's output, and the wording of each failure message. Planning also writes the first cases (20 skills × ≥5) — those briefs are content, and the near misses must be real confusions (how vs why vs teach; researching vs wayfinding; verifying vs verify-upkeep; reviewing vs code-review-style asks).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| brief | One user request, in the user's words, as the fixture's input |
| near miss | A brief that sounds like a skill and must not open it |
| description | The `description:` line of a SKILL.md — the only text the trigger test judges |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/tests/pointer_integrity.rs` — reads `skills/` from `CARGO_MANIFEST_DIR`, reports findings by file; the shape the new test follows.
- `packages/bee-rs/crates/bee/src/textutil.rs:81` — `split_frontmatter`; the crate has no `[lib]`, so an integration test parses front matter itself (memory: worktree-session-control-plane-wrapper, 2026-09-22).
- `skills/bee-writing-skills/SKILL.md:35-49` — the checklist D7 extends.

### Established Patterns

- Parity tests in `packages/bee-rs/crates/bee/tests/*_parity.rs` refuse drift by name, never by count alone.
- `description:` is either a quoted one-liner or a `>-` folded block (`skills/bee-how/SKILL.md:3-5`); the reader handles both.

### Integration Points

- `commands.test` in `.bee/config.json:11` — `cargo test --release` picks the new integration test up with no wiring.

## Canonical References

- `docs/history/research/seatworks-xia.md` — A3 and the source anchors.
- `/home/thanhsmind/Projects/refs/seatworks/plugin/test/skills/{triggers.ts,triggers.test.ts,run-triggers.ts}` at `2e11099f` — the pattern adapted (data, not instructions).

## Outstanding Questions

### Deferred To Planning

- [ ] Whether the eval runs as an `#[ignore]` cargo test or a script under `scripts/` — either satisfies D6; pick the one the release flow already knows.

## Deferred Ideas

- Trigger cases for `bee-principle-*` skills — a second fixture keyed on class and flags, judged against `bee orient`'s output, not descriptions.
- Rewriting descriptions the eval shows are confused — a docs-lane follow-up per skill once the eval has data.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
