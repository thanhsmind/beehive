---
type: bee.pattern
title: An invitation keyed on the old precondition survives the machinery that removed it
description: "Widening what a resolver accepts does not widen the invitation built on the resolver's old precondition unless every caller of that precondition is re-checked — the session preamble kept gating its knowledge-context invitation on has_work_item after the resolver stopped requiring one."
tags: [session-preamble, invitation, precondition, knowledge-layer, review-followthrough]
timestamp: 2026-08-05
bee:
  id: pattern-20260805-an-invitation-keyed-on-the-old-precondition-survives-its-removal
  lifecycle: active
  areas: [okf-profile]
  sources: ["docs/history/knowledge-loop/CONTEXT.md and plan.md (D1/D6/D8 — the three-arm resolver: work item, docs/history/<slug>/{CONTEXT.md,plan.md}, or the scribing-ledger stamp, taking reachable features from 2 to 164)", packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs and budget.rs (knowledge_context_lines gated on a hand-rolled has_work_item check), "knowledge-in-flow cell kf-1 (commit 03d2fdf1, 2026-08-05: knowledge_context_lines now gates on resolve_anchor instead of the work-item-only check; the invitation reaches the 162 of 164 anchorable features it previously skipped, and the retired \"author a work-item file\" advice line is gone)"]
  polarity: pitfall
  critical: false
---

# An invitation keyed on the old precondition survives the machinery that removed it

Widening a capability's reach does not widen the invitation that tells anyone to use it, unless the
invitation is re-checked against the new precondition explicitly. The two halves are independently
correct and independently wrong: the capability now reaches more inputs, the invitation still gates
on the narrower rule the capability itself just stopped needing.

The instance: `knowledge-loop` replaced a work-item-only `--work` lookup with a three-arm resolver
(work item, `docs/history/<slug>/`, or the scribing-ledger stamp), taking reachable features from 2
to 164. The session preamble's own invitation — "run `bee knowledge context`" — kept gating on
`has_work_item`, so 162 of those 164 newly-reachable features were never told the retrieval
existed, and the preamble kept printing advice to author the very work-item file the new design
made optional. The capability shipped; the thing that tells anyone to use it still spoke the old
rule. Cost was invisible in review because both halves were individually correct against their own,
now-divergent, understanding of the precondition.

## The rule

- When a feature widens what a downstream lookup accepts, grep every OTHER caller of the narrower
  predicate the lookup used to require — an invitation, a gate, a piece of advice — before calling
  the widening done.
- A capability and its own invitation are two surfaces, not one; a review that checks only the
  capability's own tests can pass while the invitation stays silently stale.
- "Individually correct" is not evidence of correctness at the seam: this defect was invisible
  precisely because reading either half alone found nothing wrong with it.

Fixed in `knowledge-in-flow` cell `kf-1` (commit `03d2fdf1`): `knowledge_context_lines` now gates
on `resolve_anchor` — the same three-arm check the retrieval itself uses — instead of the retired
work-item-only predicate.
