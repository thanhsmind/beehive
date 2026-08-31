# Reflection Becomes Lesson — Context

**Feature slug:** reflection-becomes-lesson
**Date:** 2026-08-31
**Shaping session:** complete (gate_bypass full — recommended readings recorded, no interview)
**Scope:** Standard
**Domain types:** RUN | READ

## Feature Boundary

A mistake caught during a run must survive twice: once as a written
reflection in that run's letter, and again as a mined lesson when the
same mistake shape repeats. Two links in that chain are missing today,
and this feature closes both. Everything else about the mailbox —
filing, arming, recovery, digests, the letter's other sections — is
untouched.

The user's ask (verbatim, Vietnamese): "Tôi thấy trong dự án thực tế
thì nhiều task có những lỗi gây lỗi, nhưng không được ghi lại như 1 bài
học trong bức thư gửi human-inbox, từ đó sẽ không có dữ liệu nào được
lưu lại để đúc kết thành những bài học riêng như yêu cầu của tôi" — in
real projects many tasks hit errors, but the errors are not written into
the letter as a lesson, so no data is stored to distill into lessons.

## Evidence

Measured in this checkout on 2026-08-31, before any change:

- `bee mailbox reflect` exists (`packages/bee-rs/crates/bee/src/verbs/mailbox.rs:2751`).
- **Zero** files under `skills/`, `AGENTS.md`, or `CLAUDE.md` name it.
  The verb is orphaned: nothing ever tells an agent to call it.
- **Zero** reflection entries exist across the **30** filed letters in
  `.bee/human-mailbox/`, in the month since the feature shipped.
- letter-reflection LR4 (`bb73e821`) locks reflection entries out of the
  LD4 mining set, so even a recorded one stops at the letter.

Both halves were already filed as backlog proposals (`.bee/backlog.jsonl`,
the 2026-08-30 rows titled "Widen LD4 lesson mining to read reflection
entries as a trouble source" and "Surface bee mailbox reflect in
host-repo doctrine and worker prompts"), at P3, unworked.

## Locked Decisions

| ID | Decision |
|----|----------|
| D1 (db562f26) | A run may not end its work without an explicit mistakes answer. The close door refuses unless the run carries at least one reflection entry or an explicit no-mistakes statement — the same shape human-mailbox D5 (`e9cb4c15`) already uses for departures. |
| D2 (a240362a) | The explicit no-mistakes answer is its own stored entry kind. It never renders as a mistake in the letter and mining never reads it, so silence and a clean run cannot read alike. |
| D3 (0872f328) | Lesson mining reads reflection entries as a trouble source, beside the broken-or-unfinished bullets and the obstacle / plan-was-wrong / fix-first departure kinds. A shape repeated across two or more distinct runs becomes a lesson under the existing shape-token and never-relog rules. The no-mistakes statement is never mined. **Supersedes letter-reflection LR4 (`bb73e821`).** |
| D4 (c556c959) | The write-a-reflection instruction lands in three homes at once: this repository's `AGENTS.md`, the host-repository onboarding template, and the rendered worker and cap prompt. A missed home is a whole class of run that records nothing. |

### Agent's Discretion

Which close door carries D1's refusal and its flag spelling; the stored
shape of D2's no-mistakes entry (it should follow the existing entries
store); how a reflection's normalized shape token is derived for D3 (it
should reuse the existing `shape:<sha-12>` derivation, not a second
one); and the exact wording of D4's three doctrine lines. Constraints:
no new background process, no change to filing, arming, or recovery
behavior, and the letter frontmatter contract only grows — existing
consumers must parse filed letters unchanged.

## Existing Code Context

- `packages/bee-rs/crates/bee/src/verbs/mailbox.rs` — the entries store,
  `KIND_REFLECTION`, `read_reflection`, `run_reflect`, and the section
  composer. D2's new entry kind lands here.
- `packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs` — the LD4
  miner and its source set. D3 lands here.
- `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs` and
  `verbs/cells/handlers_close.rs` — the close and cap doors. D1's
  refusal lands at whichever door the plan picks.
- `AGENTS.md`, the onboarding templates, and the rendered worker prompt
  — D4's three homes.
- The departure rule this feature mirrors: human-mailbox D5 (`e9cb4c15`),
  D10 (`1fb69f4b`).

## Outstanding Questions

None. The four decisions above cover the whole ask; nothing was
deferred and nothing was guessed.
