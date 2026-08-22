---
type: bee.pattern
title: "A rule checked at two points needs one shared read — a fixture that cannot diverge proves nothing"
description: "A rule checked at two points needs one shared read — a fixture that cannot diverge proves nothing"
tags: [failure, guards, tests, fixtures, duplication]
timestamp: 2026-08-18
bee:
  id: pattern-20260818-a-rule-checked-at-two-points-needs-one-shared
  lifecycle: active
  areas: [worktree-parallelism]
  sources: [".bee/cells/usp-3.json", ".bee/cells/usp-5.json", ".bee/cells/usp-6.json", "original feature: uat-stop-placement, round 2", ".bee/cells/archive/guard-herding-fallback/hgf-1.json", "recurrence: guard-herding-fallback (the dispatch-prepare door published a model the model-guard door refused; the fix reads the producer's own default-model table rather than a second copy, and the consumer's member set is the exact image of what the producer can publish, narrowed deliberately where the producer cannot reach)"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "packages/bee-rs/crates/bee/src/uat.rs (uat_lane_mode, the one shared lane-classification read); packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs (uat_merge_precheck and merge_finish's uat_wait_set, both reading uat_lane_mode); packages/bee-rs/crates/bee/src/verbs/drivers/close.rs (the uat door, reading uat_lane_mode instead of feature_route)"
---

# A rule checked at two points needs one shared read — a fixture that cannot diverge proves nothing

`uat_stop: "close"` needs its uat door enforced twice: once when the merge decides
whether to set the wait, once when `bee close` decides whether to block. Both points
classify the same feature into the same lane rule (standard/high-risk only), and both
shipped as their OWN read of "what lane is this feature in" — merge read a live
workflow's or lane record's `mode`; close read `feature_route`, which prefers
`route.lane`. On 12 of 95 real records in `.bee/lanes` those two fields name different
lanes (`knowledge-loop`: `mode: "standard"`, `route.lane: "small"`). The result: a merge
set the wait, `bee close` on the same feature read it as exempt, and the uat stop —
the exact thing the feature exists to guarantee — vanished silently. Neither run threw;
the classification itself was just wrong.

The regression suite guarding this door was green the whole time it shipped. Its
exempt-lane test wrote a lane record carrying only `mode`, no `route` at all — the
one shape under which `mode` and `route.lane` cannot possibly disagree, because the
second field does not exist to disagree with. The fixture was structurally incapable of
expressing the divergence the code was vulnerable to. A judge found it only by reading
both call sites side by side; no amount of the existing suite passing could have.

Two lessons, one for the code and one for the test:

**One rule, one home.** When the same rule is enforced at two points in a flow —
a precondition and a later door, a writer and a reader, two services agreeing on a
classification — the two points must call the SAME function to derive the shared fact,
never independently reimplement "the same" read against overlapping-but-different
fields. `uat_lane_mode` is now that one function; `uat_merge_precheck` (merge) and the
close-time `uat` door both call it, and the merge side's own former inline copy of the
read was deleted rather than kept as a second, driftable path (usp-6). Two copies of a
rule are not redundancy, they are a race with no referee — they will drift, and nothing
short of a judge reading both sides catches it until they do.

**A fixture that cannot express the disagreement proves nothing about it.** Before
writing a test for "these two paths agree", ask whether the fixture is even CAPABLE of
making them disagree. A lane record with only `mode` set cannot distinguish "reads
`mode`" from "reads `route.lane`" — every assertion against it passes for the wrong
reason. The fix added fixtures that set `mode` and `route.lane` to different lanes, in
both directions, including the exact `knowledge-loop` shape the judge found live. Test
the axis the bug lives on, not a shape where that axis is degenerate.

A related, cheaper failure mode in the same round: cleanup suppression was computed from
whether a bookkeeping WRITE succeeded (did the lane file get rewritten with the wait?)
rather than from the fail-closed PRECHECK the merge-time refusal already trusted — so a
feature with no `.bee/lanes/<feature>.json` on disk had its worktree torn down with a
uat still owed, the one place the fix could have been written (usp-5). The general form:
when a downstream decision (suppress cleanup, block an action) depends on a fact, derive
it from the same source the upstream decision already trusted — never from a side effect
(a write succeeding, a file existing) that can diverge from that fact for reasons that
have nothing to do with it.
