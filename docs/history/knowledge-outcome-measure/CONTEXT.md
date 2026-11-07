# knowledge-outcome-measure — Context

**Date:** 2026-11-07 · **Owner request:** find out whether the knowledge bundle helps the work.

## Locked decisions

- D1 — Knowledge usefulness is measured by cell outcomes: blocked, reopened
  for rework, extra attempts, and recorded mistakes. Cells are compared by the
  knowledge their brief carried. The `bee.signature` literal count is not the
  measure: no signature string appears in `.bee/decisions.jsonl` or
  `.bee/capture-queue.jsonl`, so that count always reads 0. (decision
  `eeb666c4`)
- D2 — A deterministic 10% holdout gives the comparison group. A cell whose id
  hashes into the holdout bucket is dispatched with no "Learned context"
  block. The prepare-time dispatch record names the arm (`knowledge` or
  `holdout`) and the learned-context paths. A config switch turns the holdout
  off. The owner accepts a small quality cost on 10% of cells for a causal
  answer. (decision `dafd05a1`)

## Facts found before planning

- Every cell brief already carries a "Learned context" block of up to 8
  knowledge paths, on every dispatch channel (herding pane and Agent tool).
  `bee knowledge context` runs inside `dispatch prepare` and logs no knowledge
  verb, so timings counts miss it.
- Every brief carries at least the 3-pattern critical floor, so there is no
  natural no-knowledge group.

## Out of scope

- Signature backfill on the 53 unmeasured critical patterns.
- The dispatcher-picked "Expertise" block. It stays in both arms: it is the
  dispatcher's choice, not the machine-assembled manifest.
