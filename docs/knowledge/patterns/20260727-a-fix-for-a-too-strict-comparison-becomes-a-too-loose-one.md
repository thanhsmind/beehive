---
type: bee.pattern
title: "A fix for a too-strict comparison becomes a too-loose one, and every test written for the fix points the wrong way"
description: "Normalizing both sides of a path comparison folded a character that is legal inside a filename, so two genuinely different directories compared equal and the identity check examined the wrong location. Three reviews and a red-first proof missed it; a reviewer asked to construct a false-equal found it immediately."
tags: [comparison, normalization, negative-control, review-questions, proof-discipline]
timestamp: 2026-07-27
bee:
  id: pattern-20260727-a-fix-for-a-too-strict-comparison-becomes-too-loose
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: []
  decisions: []
  sources: ["windows-path-identity (cell wpi-1, goal-check round 1)", docs/history/windows-path-identity/plan.md]
---

## The pattern

A fix for a comparison that was too strict very easily becomes a comparison that is too loose, and the second bug is worse than the first — it accepts wrong input where the original merely refused right input. The fix and its own failure mode live one line apart, and the tests written for the fix all point the same direction: they check that things that should be equal now are.

The instance: two path strings naming the same directory were compared byte-for-byte, so a legitimate operation was refused. The fix normalized both sides — including folding one separator character into another on every platform, on the stated belief that the character could not appear inside a filename. On the platforms where it *can*, a directory whose name contained that character then compared **equal** to a genuinely different nested directory, and the identity check went on to examine the wrong location. Every test added with the fix passed, because every one of them asked "are these two now equal?"

It survived a plan review, a cold-pickup review and the worker's own red-first proof. It was caught by a reviewer that was asked one specific question: *construct a case where two genuinely different things compare equal under this implementation.*

## Why the usual proofs miss it

Red-first evidence proves the fix does something. Coverage proves the new path runs. Neither asks the inverse question, because the inverse is not a regression of the old behaviour — it is a **new** acceptance the old code never made. Nothing in the diff looks like a loosening; the loosening is a side effect of a normalization that reads as obviously correct.

## What to do

- **For every comparison you loosen, write the negative control first**: two things that must stay distinct, which the new implementation might now conflate. If you cannot name one, you do not yet understand what the normalization does.
- **Interrogate each normalization step for what it destroys.** Case folding destroys case; separator folding destroys a character that may be data; resolving destroys relative structure; following links destroys the distinction between an alias and its target. Each is fine where the destroyed distinction is genuinely irrelevant, and a bug everywhere else.
- **Separate what the platform decides from what the medium decides.** Here separator meaning is a platform property while case behaviour is a per-volume one, and conflating them produced errors in both directions — a fold applied where it should not be, and an assumption made where the answer should have been asked for.
- **Make the refusal direction the default.** When a fact cannot be established, answer "different". A refused legitimate operation is a retry; an accepted wrong one is a corruption.
- **Ask a reviewer the inverse question explicitly.** "Find a case where this says equal and must not" produced the finding that three prior reviews missed.
