---
type: bee.pattern
title: A stated ancestry direction is a coin flip until one real pair of commits is measured against it
description: An ordering written in prose — ancestor of, before, predates — reads equally plausible both ways round, so a decision that states one ships a 50% bug into a guard that then fires on every run and is tuned out
tags: [verification, decisions, planning, git, failure, evidence]
timestamp: 2026-09-01
bee:
  id: pattern-20260901-a-stated-ancestry-direction-is-a-coin-flip-until-measured
  lifecycle: active
  areas: [worktree-parallelism, workflow-state]
  sources: ["proof-strength-and-expiry D4 as first locked (e77c93d6, 2026-09-01) and its correction (d1cce18d, superseding it the same day)", "cell pse-2, commit 5c90f37c — the worker measured the direction against this repo's own commits before implementing it", "measured 2026-09-01: merge base e2072df5, cap commit 9ad04c73; the locked direction flagged all three of the feature's own clean caps"]
  polarity: pitfall
  evidence: wired
  evidence_ref: "worktree/tests.rs — one test proves a non-ancestor base fires the advisory while the merge still lands, one proves an ancestor base stays silent; the pair fails if the direction is swapped back"
---

# A stated ancestry direction is a coin flip until it is measured

A locked decision said a cap's proof is stale when "the cap commit is not an
ancestor of the merge base". It reads right. It is backwards, and it would have
shipped a warning that fires on every merge bee ever performs.

## What happened

The decision was written by people who understood the problem exactly: a proof
taken before the tree moved should not read like one taken after it. The
comparands were right — the cap's recorded commit, and the merge base. Only the
direction was wrong, and prose gives no signal about direction. "Predates",
"before", "is an ancestor of" all read as plausible either way round, and a
reviewer checking the sentence checks whether it *sounds* like the failure being
described. It did.

A branch commit descends from the merge base. So a cap taken on the branch — the
normal case, the case that happens every time — is never an ancestor of the
base. The literal rule flags every cap ever taken. Measured on the feature's own
merge: base `e2072df5`, cap commit `9ad04c73`, and the rule flagged all three of
its own clean caps. The advisory would have fired on the very merge that
shipped it.

The worker caught it by running `git merge-base` against real commits in the
repo instead of re-reading the sentence, then implemented the inverse and pinned
both directions with tests.

## Why it is worth a record

The damage is not the wrong boolean. An advisory that fires every time is worse
than no advisory: it trains its reader to skip the line, and the one merge where
it is telling the truth reads exactly like the hundred where it was not. A
guard's value is entirely in its silence.

And the failure mode is invisible to every review that reads. The sentence is
grammatical, the comparands are correct, the intent is correct. Only data
separates the two readings.

## What to do instead

- Any ordering, ancestry, or comparison direction stated in a decision, a plan,
  or a cell action is a **claim to measure**, not a claim to implement. Run it
  against one real pair from the repo at hand before writing the code.
- Prefer the measurement that would embarrass the rule: does it fire on the
  ordinary case? A guard silent on the ordinary case and loud on the named
  event is the only shape worth shipping.
- Pin both directions with tests, so a later "fix" that restores the plausible
  wording turns red instead of shipping.
- When the measurement contradicts a locked decision, the decision is what
  changes: supersede it with the corrected direction and the evidence, never
  quietly implement the opposite of what the record says.

## Related

- `20260812-a-guard-and-its-tests-are-one-model-so-green-proves-only-that-the-model-agrees-with-itself.md`
  — the neighbouring failure: here the model was wrong and only outside data
  could say so.
- `20260830-existence-is-not-evidence.md` — same family, one level up: reading a
  claim is not checking it.
