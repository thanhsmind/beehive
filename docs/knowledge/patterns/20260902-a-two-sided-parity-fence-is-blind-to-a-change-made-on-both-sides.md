---
type: bee.pattern
title: A two-sided parity fence is blind to a change made on both sides
description: A test that pins file A to file B proves they agree, never that either says anything. Delete the same line from both and the fence stays green while the rule it guarded is gone.
tags: [tests, drift, doctrine, generated-files, false-confidence]
timestamp: 2026-09-02
bee:
  id: pattern-20260902-a-two-sided-parity-fence-is-blind-to-a-change-made-on-both-sides
  lifecycle: active
  areas: [doctrine-layer, verify-pipeline]
  sources: ["verification-in-the-flow vif-2 and verification-contract-parity vcp-2, 2026-09-02", "packages/bee-rs/crates/bee/tests/agents_block_render_parity.rs", "packages/bee-rs/crates/bee/tests/verification_contract_parity.rs"]
  polarity: pitfall
  critical: false
  evidence: exercised
  evidence_ref: "With the proof case deleted from BOTH packages/bee/AGENTS.block.md and AGENTS.md — exactly what a regen produces — agents_block_render_parity passed while the doctrine was gone; verification_contract_parity went red naming the fix. Observed 2026-09-02, both files then restored and both tests re-run green."
---

# A two-sided parity fence is blind to a change made on both sides

A generated file needs a test that it matches its source. That test is
worth having: it catches the edit someone made without re-running the
generator, which is the common failure.

It does not catch the other one. A parity fence asserts a *relation* —
A equals B. It says nothing about what A contains. Delete a rule from
the source, regenerate, and both files agree perfectly about a document
that no longer carries the rule. The fence is green. The rule is gone.

This is easy to miss because the fence feels like coverage. It was
built to protect that file, it is named after that file, and it is
green. Nothing in the run says which of the two questions it answered.

**What happened.** `AGENTS.md` is rendered from
`packages/bee/AGENTS.block.md`, and `agents_block_render_parity` pins
the block byte-for-byte. A feature then added two rules to that block.
Deleting one of them from the source and regenerating left both files
byte-identical, the fence green, and the doctrine absent — verified by
running it, not by reasoning about it.

**The rule.** A relation test and a presence test answer different
questions, and a generated surface carrying load-bearing content needs
both:

- **Relation** — A matches B. Catches the forgotten regeneration.
- **Presence** — A still says the thing. Catches the deletion, the
  quiet reword, and the well-meaning cleanup.

Write the presence test against the *source*, and assert the few tokens
that carry the meaning rather than a whole sentence — a test that pins
prose verbatim goes red on every legitimate reword and gets deleted by
the next person who hits it.

**How to tell which kind you have.** Delete the content you care about
from both sides and run the test. If it stays green, it is a relation
test, and whatever you thought it protected is unprotected. Do this
once, on purpose, when you write the fence: a parity test never
observed failing is not known to work, and one observed failing for the
wrong reason is worse than none.

A related but distinct failure is the same rule living in N places and
drifting apart, which one test reading all N does catch — see
[[pattern-20260826-a-rule-living-in-n-places-needs-one-test-that-reads-all-n]].
That one is copies-disagreeing; this one is copies-agreeing-about-nothing.
