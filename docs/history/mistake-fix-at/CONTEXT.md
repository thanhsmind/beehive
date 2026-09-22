# Mistake Fix-At — Context

**Feature slug:** mistake-fix-at
**Date:** 2026-09-22
**Shaping session:** complete (two user answers; the rest recorded as recommended readings under gate_bypass full)
**Scope:** Standard
**Domain types:** RUN | CALL

## Feature Boundary

Every mistake an agent records names the layer that fixes it, and a
mistake that a check or an architecture change can fix becomes a
backlog row the first time it is seen. The reflection verb, the cap
flag, the close driver, the weekly lesson miner, and the three doctrine
homes change. Letter filing, arming, recovery, the digest's other
sections, and the promote proposal renderer are untouched.

Source of the idea: `docs/REFs/pstack-rules.md` (the pstack talk). Its
ladder — architecture > static analysis > rules and skills > style
guide — says a correction is only useful when it is pushed to the
strongest layer, at the moment of correction.

## Evidence

Measured in this checkout on 2026-09-22, before any change:

- 465 `reflection` entries and 104 `no-mistakes` entries exist under
  `.bee/human-mailbox/entries/`; 41 filed letters carry a
  "Mistakes & reflection" section. Reflections are written.
- Zero decisions tagged `lesson` exist, and zero `shape:<sha-12>`
  tokens appear in `.bee/decisions.jsonl` or its archive. The weekly
  miner (`packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs:568`)
  has never logged a lesson.
- Cause: the miner's shape token is `sha256` of the whole normalized
  sentence (`normalize_shape`, `shape_token` in `mailbox_digest.rs`),
  and two distinct runs never write the same sentence.
- No reader turns a reflection or a lesson into a hook, guard, doctor
  check, or test. `bee knowledge promote`
  (`packages/bee-rs/crates/bee/src/verbs/knowledge/promote.rs`) reads
  `trace.deviations` and failure signatures but never `trace.mistakes`;
  `skills/bee-capturing/references/promotion.md` names review findings,
  user corrections and deviations as its inputs, never the run's own
  mistakes.
- A reflection row carries `at, kind, what, files, commit, proof,
  departure, needs_you, better` (`mailbox.rs:640-693`). No field names
  a layer. `bee backlog add --layer` is free text meaning workflow area
  (observed: verification, state, workflow, hooks, …), consumed by the
  feedback digest as `layer`.
- Correction to an earlier backlog row filed this session ("Mistake
  reflections never reach promotion", P2): reflections DO reach the
  lesson miner (reflection-becomes-lesson D3, `0872f328`). What never
  happens is a match, and what never exists is a consumer that
  mechanizes the result.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. Changing one requires the user, a new D-ID or an
explicit supersession note, never a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 (af14e1d6) | `bee mailbox reflect` and `bee cells cap --mistake` take a required `--fix-at` value from the fixed vocabulary `architecture \| check \| doctrine \| none`. Missing or outside the vocabulary → refused, the same door `--wrong`/`--better` use. `--no-mistakes` is unchanged. | User's answer. Optional was rejected: an optional field stays empty, as 465 rows show today. |
| D2 (6e79d14e) | One occurrence is enough for a mechanizable mistake. At `bee close`, every reflection of the closing feature whose fix-at is `check` or `architecture` files exactly one backlog row: type `finding`, severity `P3`, layer `fix-at:<value>`, title = the `wrong` text, detail = the `better` text plus the run or cell it came from. The same entry is never filed twice. `doctrine` and `none` reflections file no row and stay on the two-run lesson rule. | User's answer. The row rides the existing free-text `--layer`, prefixed so the digest can filter it. |
| D3 (9173fbd2) | The weekly lesson miner keys a reflection by fix-at plus the normalized first four words of its `wrong` text, not by the hash of the whole sentence. The four-word minimum, the two-distinct-runs rule, and the once-ever token rule stay for `doctrine` and `none` reflections. | Whole-sentence hash matched zero times. Fuzzy similarity rejected: mailbox D8 forbids the miner adding judgement on top of what letters carry. |
| D4 (5c43da33) | The fix-at instruction lands in the three homes reflection-becomes-lesson D4 (`c556c959`) named: this repository's `AGENTS.md`, the host onboarding template, and the rendered worker and cap prompts. Each line states the vocabulary and that the flag is required. Existing reflections without fix-at are not backfilled; close and the miner read them as `none`. | Recommended reading. Backfilling would be the agent guessing a layer the author never named. |
| D5 (402cf686) | bee-capturing's promotion tree reads the closing feature's fix-at reflections as an input at step 1. A `check` or `architecture` reflection may be taken in-feature as a tiny cell that ships the check, in which case its D2 row is marked done; otherwise the row stands for grooming. A `doctrine` reflection routes through steps 3 and 4 unchanged. | Recommended reading. Today the tree never asks "Mechanizable?" of the run's own mistakes. |

### Agent's Discretion

The flag spelling on the cap side (`--fix-at` beside `--mistake`, or a
third segment of the `--mistake` line — the flag form is preferred so
the report's `mistakes` array gets a field of the same name); the
stored field name on the entry row and on `trace.mistakes` (one name,
used in both); how close dedupes D2 rows (a stable per-entry key that
survives a re-run of close); the exact wording of the three D4 doctrine
lines; and whether D5 lands as a skill edit only or also as a
`promote.rs` section. Constraints: no new background process; letter
frontmatter and entry rows only grow — existing consumers parse filed
letters and old entry rows unchanged; a close that cannot write the
backlog row warns and never refuses the close (the mailbox's fail-open
rule).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| fix-at | The strongest layer that can stop this mistake from recurring, named by the agent at the moment of recording. |
| architecture | A code or data-structure change in the repo makes the mistake impossible — there is nothing left to check. |
| check | A test, hook denial, guard, lint line, or `bee doctor` row catches the mistake mechanically. |
| doctrine | Only prose can carry it — a skill, rule, or AGENTS.md line — because it is judgement, taste, or product intent. |
| none | The mistake was a one-off with no layer worth naming (a typo, a misread); recorded so it is an answer, not silence. |
| mechanizable | fix-at is `check` or `architecture`. |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/mailbox.rs` — `Entry` (640-693), `Entry::reflection`, `read_reflection` (795-811, the two-part door D1 extends), `read_mistake` (830-849, parses the cap's `--mistake` line), `KIND_REFLECTION`, `run_reflect` (2892-2955).
- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` — `CapFlags.mistake` / `no_mistakes` (123-127), `read_mistakes_answer` (893-920), the `trace.mistakes` write (566-581), `record_cap_in_mailbox` (1030-1044).
- `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs` — `mistakes_debt` (267-282) and the mistakes door (1540-1562): the walk over the feature's capped cells D2 reuses.
- `packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs` — `normalize_shape`, `shape_token`, `reflection_what` (696-717), the lesson pass (568+): D3 lands here.
- `packages/bee-rs/crates/bee/src/verbs/backlog.rs` — `bee backlog add` validation (733-841): D2 rows go through this writer, never a hand append.

### Established Patterns

- Two-part refusal door (`read_reflection`): a required field missing → refused with the remedy named. D1 adds a third part the same way.
- Fail-open mailbox writes: a mailbox that cannot be written warns and never fails the verb. D2's row filing follows it.
- Prompt homes: `.bee/bin/prompts/worker-cell.md` and the cap prompt are rendered from `packages/bee/`; `packages/bee/AGENTS.block.md` is the host template — D4's three homes.

### Integration Points

- `docs/knowledge/areas/human-mailbox/overview.md` — the entry-kind table and rule 15/16 (lesson mining) must be synced at capture.
- `skills/bee-capturing/references/promotion.md` — Promotion Decision Tree step 1 (D5).
- `.bee/verify/verify-app/features/cells-and-proof.md` and `worktree-and-close.md` — the mapped features whose drive recipes cover cap and close; the close door's new output is user-facing.

## Canonical References

- `docs/REFs/pstack-rules.md` — the talk; the ladder and the "lint rule to stop the bleeding" instinct.
- `docs/history/reflection-becomes-lesson/CONTEXT.md` — D1–D4 (`db562f26`, `a240362a`, `0872f328`, `c556c959`): the mistakes door, the no-mistakes kind, mining reads reflections, the three doctrine homes.
- `docs/history/letter-reflection/CONTEXT.md` — LR1/LR2: the letter section and the entry kind.
- `docs/knowledge/areas/feedback-digest/data-model.md` — the `layer` field the digest carries.

## Outstanding Questions

### Deferred To Planning

- [x] Whether the D3 key change needs a migration note for the once-ever token rule — resolved in planning (plan.md claim 25) and proven by mfa-3's test `a_token_an_old_whole_sentence_lesson_spent_does_not_block_the_new_key`: old tokens digest the whole sentence, new tokens digest layer+head, so no old token collides and no migration note is needed.
- [x] The cheapest proof for D2 — landed as mfa-2's tests `a_green_close_files_one_backlog_row_per_mechanizable_mistake` and `closing_the_same_feature_again_files_no_second_row`.

## Deferred Ideas

- Backfill triage of the 465 existing reflections into fix-at layers by a reader agent — out of scope (D4). Trigger `grooming-asks-for-the-fix-at-layers-of-t__5c43da33` fires it when grooming asks for that history.
- A `bee doctor` row that counts `fix-at:check` backlog rows older than N days — the "stop the bleeding" debt meter. Trigger `more-than-20-backlog-rows-with-layer-fix__6e79d14e` fires it when those rows pile up.
