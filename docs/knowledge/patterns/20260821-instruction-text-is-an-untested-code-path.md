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
  sources: ["capture stub 10712761 (worker-proof-line-skew, cell wpls-1)", docs/knowledge/areas/workflow-state/cells-authoring-and-revision.md, docs/knowledge/areas/rust-runtime/prompt-files-and-learned-context.md]
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
