# Letter Reflection Section — Context

**Feature slug:** letter-reflection
**Date:** 2026-08-30
**Shaping session:** complete (gate_bypass full — recommended readings recorded, no interview)
**Scope:** Small
**Domain types:** RUN | READ

## Feature Boundary

The human-mailbox letter gains one self-reflection section: the mistakes
the agent made during the run, and what would have been better. The
section is fed only by reflection entries the agent records as it works;
the composing pass stays a renderer. Everything else about the letter —
filing, arming, recovery, digests, mining — is untouched.

The user's ask (verbatim, Vietnamese): "human-inbox nội dung nên có
section ghi lại những lỗi lầm mắc phải trong phiên làm việc đó nếu có,
1 dạng phản tư nhìn lại những gì mình đã làm và nếu mình thực hiện việc
gì thì sẽ tốt hơn" — the letter should carry a section recording the
mistakes made in that work session, if any: a reflection looking back at
what was done and what would have been better.

## Locked Decisions

| ID | Decision |
|----|----------|
| D1 (5dbdb0e2) | One new body section, "Mistakes & reflection", after "Broken or unfinished", before "Needs your call", in both the nightly and the feature-close letter. Empty set drops the section (D7's drop rule holds). Amends human-mailbox D7's section list; every other D7 rule stands. |
| D2 (b8291876) | The only source is a new entry kind, `reflection`, appended by the acting agent at the moment a mistake is noticed or at the run-end look-back, through a bee command. The composing pass renders stored reflection entries and may never author one — human-mailbox D8 holds unchanged, covered by the authorship-walk test. |
| D3 (ba9f06a4) | A reflection entry has two required parts: what went wrong, and what would have been better. Missing either part refuses the append. |
| D4 (bb73e821) | Lesson mining (letter-digest LD4) keeps its exact current sources. Reflection entries are not mined; widening LD4 is a separate backlog proposal, filed. |

### Agent's Discretion

The command's name and place in the verb tree, the entry storage shape
(it should follow the existing entries store), and the section's exact
rendering. Constraints: no new background process, no change to filing,
arming, or recovery behavior, and the frontmatter contract only grows —
existing consumers must parse filed letters unchanged.

## Existing Code Context

- `packages/bee-rs/crates/bee/src/verbs/mailbox.rs` — the store, the
  record, and every composing rule; the authorship-walk test lives with
  it. The new entry kind and section land here.
- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` — the
  cap's append and the departure door (shape mirror for D3's two parts).
- `bee mailbox mark` — the one existing mailbox command; the append
  command joins this group.

## Outstanding Questions

- [ ] Deferred to planning: whether the reflection entry rides the
  existing per-run entries file or needs its own kind marker inside it —
  resolved by reading the entries store shape in mailbox.rs.

## Deferred Ideas

- Widen LD4 lesson mining to read reflection entries — filed as a
  backlog proposal under this feature (D4).
