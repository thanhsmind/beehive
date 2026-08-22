---
type: bee.pattern
title: A skip and a dedup leave the same array
description: "When one branch drops an entry and another de-duplicates it, both leave a list of the same length; a test that only counts entries is green against the broken code and proves nothing. Prove a de-duplication by the entry that must survive, and prove both intake paths with the same odd-shaped entry."
tags: [testing, dedup, proof-discipline, intake-paths]
timestamp: 2026-08-22
bee:
  id: pattern-20260822-a-skip-and-a-dedup-leave-the-same-array
  lifecycle: active
  areas: [workflow-state, rust-runtime]
  decisions: []
  sources: ["deviation-one-list cell dol-1 trace (2026-08-22): the report-side intake dropped non-string deviations while the file-side intake mined them, and the dedup test stayed green before the fix because a skipped entry and a deduplicated entry leave the same count", "docs/history/deviation-one-list/promote-proposals.md"]
  polarity: pitfall
  critical: false
  evidence: observed
  evidence_ref: "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs (the worker-report deviation union at completion)"
  signature: skip-and-dedup-same-length
---

# A skip and a dedup leave the same array

Two intake paths fed one list. One path silently dropped every entry that
was not a plain string; the other mined the same entries correctly. The
de-duplication test counted entries after the merge and passed on the
broken code — because an entry that was never admitted and an entry that
was admitted once and collapsed produce the same length.

## The rule

- Prove a de-duplication by the entry that must **survive**: assert the
  surviving entry's content, not the count.
- Prove every intake path with the **same odd-shaped entry** (a non-string, an
  empty value, a nested shape), so a path that skips it fails where a path
  that keeps it passes.
- A length-only assertion over a merge is not a proof of either branch.

## Where it bit

deviation-one-list dol-1 (2026-08-22): the cap-time union of a worker's
structured deviations into the unit's deviation list. The fix admitted
non-string entries on both sides and rendered them through the same
function the miner uses; the test was rewritten around the surviving entry.
