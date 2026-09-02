---
type: bee.pattern
title: Instruction text is an untested code path
description: Instruction text is an untested code path
tags: [failure, prompts, doctrine, drift, tests]
timestamp: 2026-08-21
bee:
  id: pattern-20260821-instruction-text-is-an-untested-code-path
  lifecycle: active
  areas: [rust-runtime, workflow-state, doctrine-layer]
  sources: ["capture stub 10712761 (worker-proof-line-skew, cell wpls-1)", docs/knowledge/areas/workflow-state/cells-authoring-and-revision.md, docs/knowledge/areas/rust-runtime/prompt-files-and-learned-context.md, "first bee-verify-upkeep run, 2026-09-02 — the feature map taught a cap command the CLI refuses"]
  polarity: pitfall
  critical: true
---

# Instruction text is an untested code path

A prompt, a skill reference, a doctrine page — any text a reader
acts on — steers behavior exactly like code. Unlike code, nothing
compares its sentences to the function they describe. Change the
function and the sentence keeps running, unchanged and wrong, and no
suite goes red.

The cost is paid by every reader, quietly, for as long as it takes
someone to notice.

A worker prompt carried one sentence saying the cap door runs the
project's declared test command. That had been true; a later change
made the door RECORD a proof line the worker hands it and run nothing.
The sentence stayed. Every dispatched worker read it and ran the whole
release-profile suite as insurance before capping — a full build of
everything, per cell, for months. No test could catch it, because no
test asserts prompt prose against the code path it narrates. Nothing
was ever red. The waste was invisible precisely because the system was
working.

The sentence had also spread. When it was finally corrected in the
prompt, the same retired claim was still alive in two doctrine
pages — one of them arguing, from the false premise, that a rule
needed no prose home because the CLI already enforced it. A stale
instruction is rarely in one file: it was copied when it was true.

**The rule, in three parts.**

Changing a behavior means sweeping the text that describes it, not
just the code. Search for the claim, not for the file you remember —
the phrasing travels, and the copy that matters is usually the one you
did not write.

Correct the description at every surface in the same change. A
half-corrected claim is worse than a wholly stale one: the reader who
finds the fresh copy has no reason to doubt the stale one.

When instruction text and code disagree, the text is the defect until
proven otherwise — a reader already acted on it. And where a rule is
load-bearing, prefer an assertion that ties the sentence to the code
over trusting a future editor to remember both.

A related but distinct failure is a correct instruction that is not
running at all, because the artifact carrying it was never
rebuilt — see [[pattern-20260821-a-vendored-binary-is-a-second-place-the-feature-must-land]].
That one is absent-but-right; this one is present-and-wrong.

**Recurrence, 2026-09-02 — and what finally owns it.**

The first `bee-verify-upkeep` run found the same failure in a place
built to prevent it. `.bee/verify/verify-app/features/cells-and-proof.md`
told a driver to cap with a bare `green` result. That form had been
correct; `proof-strength-and-expiry` later closed the result segment over
`green:live` / `green:unit` / `green:static` and made a bare `green` a
refusal on write. The sentence stayed. Driving it returned exit 1.

Two details make this recurrence worth its lines. The map is a
*verification* artifact — the thing whose whole job is to be executed —
and it still rotted, because nobody executed it. And one of its stale
line citations had been moved by the very feature that shipped that same
week: a doc goes stale fastest against the change that touches it.

The escalation this pattern asks for is now partly paid. Where a claim
is about a constant or a rendered surface, a text-reading test owns it
(`verification_contract_parity.rs`, `agents_block_render_parity.rs`).
Where the claim is a RECIPE, no test can own it — only running it can.
That is what the upkeep skill's live pass is for, and it is a human-run
skill, so the durable owner here is a cadence, not a check. Prose stays
because nothing else can hold it: **a recipe is proven by being run, and
a verification map that has never been driven is documentation, not
verification.**
