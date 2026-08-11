---
type: bee.pattern
title: A sweeping consolidation eats the call site that was different on purpose
description: "Folding seven copies of a helper into one converted every caller to the new default — including the one guard whose measure is dictated by an external validator, which the sweep could not tell apart from drift; the same sweep carried a comment whose stated reason for the surviving exception was contradicted by the code's own test."
tags: [refactoring, consolidation, exceptions, review, rationale]
timestamp: 2026-08-06
bee:
  id: pattern-20260806-a-sweeping-consolidation-eats-the-call-site-that-was-different-on-purpose
  lifecycle: active
  areas: [rust-runtime, hook-runtime]
  decisions: ["js-parity-cleanup D3 (consolidate the helpers, switch display caps to character counting, keep one named exception)", "bc2e2d44 (judge pass over the consolidation: two findings — the question-heading guard had been swept with everything else, and the surviving exception's rationale was inverted — spawning fix cell jp-9)"]
  sources: ["js-parity-cleanup cell jp-5 (seven duplicated helpers plus three inline copies folded into one module across eighteen files; trace .bee/cells/jp-5.json, 2026-08-04 — 1004 passed, 0 failed)", "js-parity-cleanup cell jp-9 (the guard restored, the rationale corrected; trace .bee/cells/jp-9.json, 2026-08-04 — 1006 passed, 0 failed)", "packages/bee-rs/crates/bee/src/hooks/write_guard/detectors.rs:155-161 (the restored call site, with the reason written beside it)", docs/knowledge/areas/rust-runtime/text-measurement-and-the-two-counting-units.md]
  polarity: pitfall
  critical: false
  evidence: wired
  evidence_ref: "packages/bee-rs/crates/bee/src/hooks/write_guard/detectors.rs (the AskUserQuestion header-length guard, measured in utf16 code units to match the external Claude Code validator)"
---

# A sweeping consolidation eats the call site that was different on purpose

A consolidation reads N copies of the same logic and produces one. Its whole
premise is that the copies are accidents. That premise is right about most of
them and wrong about the one that was deliberately different — and nothing in
the sweep distinguishes *deliberately different* from *drifted*, because both
look identical in the diff: a caller that does not match the new rule.

The instance: seven duplicated text-length helpers plus three inline copies were
folded into one module, and the display-truncation caps moved from counting code
units to counting characters — the correct default, decided and logged. The same
pass converted one more call site that was not a display cap at all: a guard
whose limit is set by an external validator that counts code units. Measured in
characters, the guard began passing requests that the validator would then
reject — the exact failure the guard exists to prevent. The full test suite was
green through all of it, because no test asserted the *reason* for that site's
measure, only its result on plain text, where the two units agree.

The same sweep also rewrote the surviving exception's comment with a rationale
that named the wrong artefact: it claimed a comparator kept a manifest's own
path list reproducible, while the file's own test asserted that ordering would
*not* reproduce it. The real constraints were two other stored artefacts. Both
findings came from an independent judge pass over what everyone involved,
correctly, called a pure refactor.

## The rule

- Before a sweep, list the call sites that are *supposed* to differ and why. If
  that list is empty, you have not looked — a helper duplicated seven times has
  almost always accumulated one genuine special case.
- A special case survives only if its reason lives at the call site. A rule kept
  in a plan, a decision record, or someone's memory is invisible to the next
  sweep, which is the sweep that will remove it.
- State an exception's reason in terms of what it is pinned to — this validator,
  this stored artefact, this test — never in terms of history ("kept for
  compatibility"). A historical reason cannot be checked, so it decays into
  something nobody dares delete and nobody can verify.
- Verify the rationale against the artefact it names. Here the comment named a
  file whose own test contradicted it; one grep would have caught it, and did,
  once someone looked.
- A green suite does not clear a sweep. Tests assert results, and a
  deliberately-different call site usually agrees with the default on every input
  the tests use — the disagreement lives in the inputs nobody wrote a test for.
  Give the "pure refactor" its independent read.
