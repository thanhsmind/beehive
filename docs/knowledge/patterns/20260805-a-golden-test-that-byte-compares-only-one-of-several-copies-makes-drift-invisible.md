---
type: bee.pattern
title: A golden test that byte-compares only one of several copies makes drift invisible
description: A golden test that byte-compares only one of several duplicated copies of a function lets the other copies drift underneath it while every run stays green — proven when `build_context_manifest` existed in three places and the only byte golden read one of them.
tags: [verification, golden-test, duplication, proof-discipline, drift]
timestamp: 2026-08-05
bee:
  id: pattern-20260805-a-golden-test-that-byte-compares-only-one-copy
  lifecycle: active
  areas: [okf-profile]
  sources: ["packages/bee-rs/crates/bee/src/verbs/knowledge/context.rs (build_context_manifest, refusal tagged D27)", "removed 2026-08-16: packages/bee-rs/crates/bee/src/verbs/drivers/kctx.rs no longer exists on disk — the verbatim build_context_manifest port copy this pattern is about was retired (R6 CLOSED, drivers/tests.rs:1431); there is exactly one build_context_manifest now, in packages/bee-rs/crates/bee/src/verbs/knowledge/context.rs:200", "packages/bee-rs/crates/bee/src/verbs/knowledge/promote.rs (its own resolver, refusal tagged D38, before knowledge-loop)", "removed 2026-08-16: the byte-for-byte golden tests (learned_context_agrees_with_the_knowledge_verb_port, learned_context_history_anchor_agrees_across_both_ports) no longer exist in packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs — retired with kctx.rs per the R6 CLOSED comment at tests.rs:1431 (a cross-copy-agreement assertion has nothing left to say now that only one copy remains)", "docs/history/knowledge-loop/plan.md (Discovery: the two copies had already drifted, carrying D27 vs D38 tags in otherwise identical messages)", "knowledge-loop cell kl-1 (anchor.rs unifies context.rs and kctx.rs in the same cell; commit 1b2a8253, 2026-08-05)", "knowledge-loop cell kl-2 (promote.rs consumes the same resolver; commit e6f99a7a, 2026-08-05)"]
  polarity: pitfall
  critical: false
---

# A golden test that byte-compares only one of several copies makes drift invisible

A golden test proves two things stay identical only for the copies it actually reads. When a
function is duplicated across several call sites and the golden reads just one, the rest are
outside its reach — every run stays green while they drift, and the golden is exactly the check
whose purpose was to catch that.

The instance: `build_context_manifest` existed in three places — `verbs/knowledge/context.rs`,
`verbs/drivers/kctx.rs`, and `verbs/knowledge/promote.rs`'s own resolver. The only byte-for-byte
golden, `drivers/tests.rs:881-905` (`learned_context_agrees_with_the_knowledge_verb_port`), called
the kctx copy alone. Editing `context.rs` and leaving kctx behind would have kept every test green
while the two copies drifted — the exact invariant the golden exists to hold. The drift was not
hypothetical: the two copies had already diverged, carrying different decision tags (`D27` in
`context.rs`, `D38` in `promote.rs`) in otherwise identical refusal messages.

## The rule

- Before trusting a golden, name every call site the thing it protects has, and check the golden's
  read path reaches every one of them — not just the copy that was easiest to fixture.
- When a duplication is discovered, move every copy in the SAME cell the golden protects, and add a
  test that goes red when only one copy moves. A golden pinned on one copy is worse than no golden:
  it certifies safety it does not have.
- Two independently-tagged, near-identical error messages (`D27` vs `D38` here) are themselves
  evidence of undetected drift — a decision citation attached to duplicated logic is worth grepping
  for siblings before either citation is trusted.

Fixed together in one cell: `verbs/knowledge/anchor.rs` now holds the one `resolve_anchor` function
all three sites call.
