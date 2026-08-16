---
type: bee.pattern
title: A debt derived in two places clears in one and haunts the other
description: "When the same debt (scribing debt, any outstanding-work signal) is derived independently by two reporting surfaces, completing the work clears only the surface wired to the settlement record; every unreconciled copy keeps reporting the debt forever. One derivation rule, shared by every surface, or an explicit reconciliation against the settlement store."
timestamp: 2026-08-14
bee:
  id: pattern-20260814-debt-derived-in-two-places
  lifecycle: active
  areas: [workflow-state]
  sources: [".bee/cells/archive/traceable-runs/trun-9.json (two failure signatures: preamble vs status/orient copies of scribing_debt)", "packages/bee-rs/crates/bee/src/verbs/status_full/cells.rs (reconciled scans, trun-9)"]
  polarity: pitfall
  evidence: wired
  evidence_ref: "packages/bee-rs/crates/bee/src/verbs/status_full/cells.rs (deferred-queue reconciliation, trun-9)"
  signature: debt-cleared-on-one-surface-still-reported-on-another
---

# A debt derived in two places clears in one and haunts the other

trun-9 (traceable-runs) hit this twice in one cell: completing a scribe record
cleared the session preamble's scribing-debt line but not the nudge's — the
preamble read an unreconciled second copy — and after that repair,
`bee status --json` and `bee orient` STILL reported the debt from a third and
fourth unreconciled scan (`verbs/status_full/cells.rs:436/:467`). Each surface
had derived the debt independently; the settlement only reached the surfaces
someone remembered to wire.

## The rule

- A debt is one derivation, not N. Every surface that reports it must read the
  same rule (or the same store) that the completing action writes.
- When adding a settlement path, grep for EVERY reader that derives the debt —
  the failure count in this instance was four surfaces, found two at a time.
- A test that exercises "complete the work, then read every reporting surface"
  is the only proof the copies agree; per-surface tests pass while the copies
  disagree.
