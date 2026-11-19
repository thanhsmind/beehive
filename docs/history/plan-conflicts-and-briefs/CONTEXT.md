# Plan-conflict derive, blind example, LaneBrief refusals, CoS check at authoring — Context

**Feature slug:** plan-conflicts-and-briefs
**Date:** 2026-11-19
**Backlog items:** p-34ce8a88, p-4251c625, p-64f29f32, p-0b2812f8
**Shaping session:** complete under gate bypass full (owner said go on the 2026-11-19 backlog sweep)
**Scope:** Standard
**Domain types:** READ | RUN

## Feature Boundary

Plan-conflict derive works before cells exist and takes a batch verdict. The
second registry example of `bee blind check` has an asserted outcome. The
LaneBrief lint carries real sub-questions and its refusals say how to rewrite.
`bee backlog pbi add` and `bee backlog propose` warn on a CoS that names nothing
checkable.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | When the feature has no cells, `bee state plan-conflicts derive` builds its term set from the feature's `plan.md` and `CONTEXT.md` (`docs/history/<feature>/`). A test proves a cell-less feature with a plan returns non-empty candidates. The saturating-term cut already shipped (plan_conflicts.rs `TERM_DF_MAX_PERCENT`) stays. | p-34ce8a88 (1). Derive must be meaningful before the execution gate. |
| D2 | `bee state plan-conflicts verdict` accepts several `--id` values (comma list) with one verdict and records one row per id. | p-34ce8a88 (3). |
| D3 | The second registry example of `bee blind check` is removed from the registry; the example dossier stays as documentation and states that the digest check needs runtime dispatch ids. A test asserts every registry example of `blind check` exits 0. | p-4251c625. A doc example cannot satisfy a check that reads runtime state; the digest check is not weakened. |
| D4 | LaneBrief lint (`verbs/drivers/brief_lint.rs`): (a) a bulleted line under `## Question` that ends with `?` is a sub-question and is carried; a bulleted line that does not end with `?` still refuses as `brief_question_enumerates_answers`, and the refusal names the sub-question rule. (b) A leaning-stem refusal in `## Constraints` names how to rewrite a format constraint without the stem (for example "Answer as digests with file:line anchors"). (c) The four-section set stays closed; the refusal names the closed set and the reason (a fifth section is where leaning context hides). Each change is red first. | p-64f29f32. Refines D2a of slp-blind-lanes; the enumerated-candidates refusal is not weakened. |
| D5 | `bee backlog pbi add` and `bee backlog propose` run the same checkable-CoS test that dispatch uses (a CoS names a command, a path, or an inspectable state) and print a warning naming the rule when it fails. The add still succeeds. | p-0b2812f8. Catch a vague item while its author is present; a warning, because authoring is not the gate. |

## Read set

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/state_group/plan_conflicts.rs` | D1, D2. |
| `packages/bee-rs/crates/bee/tests/registry_dispatch.rs` | D3. |
| `packages/bee-rs/crates/bee/src/verbs/blind/mod.rs` | D3. |
| `packages/bee-rs/crates/bee/src/verbs/drivers/brief_lint.rs` | D4. |
| `packages/bee-rs/crates/bee/src/verbs/backlog.rs` | D5. |

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
