---
artifact_contract: bee-plan/v1
mode: standard
approved_gate2: 2026-07-28 (gate bypass TOTAL — audit decision logged)
---

# Parallel By Default — Plan

CONTEXT: `docs/history/parallel-default/CONTEXT.md` (D1–D4).

## Mode-Gate Record

Flags: 1 (multi-domain: instruction doctrine + guard code). Story-sized doctrine
change → standard. Product files ≈ 6 (cells.mjs, its test, 4 instruction files).

## Approach

One slice, three cells; pd-1 (code) and pd-3 (doctrine text) have disjoint
product file sets once regen is deferred to the wave barrier (D2) — **they
dispatch in parallel, dogfooding D1**. pd-2 (test) trails per slice-tail rule.

- **pd-1** — guard message in `packages/bee/lib/cells.mjs` gains the
  wave-barrier alternative sentence; recognized ack value `"wave-barrier"`
  documented in the message. `regen_obligation_ack: "wave-barrier"` (barrier
  run by orchestrator at wave close).
- **pd-3** — flip the doctrine text at the four mapped sites (swarming body
  row :27, swarming-reference hardening-7 §, routing-and-contracts :293 lane
  row, :299 §, :527 class paragraph): parallel default on disjoint ownership
  (3–4 live workers), serial names its conflict, wave-barrier regen protocol
  stated (orchestrator's debt + close-commit timing). Byte budgets hold
  (swarming body 5-byte headroom → net-negative body edits, detail to
  references). Same ack.
- **pd-2** — extend the regen-guard suite (locate REGEN_OBLIGATION tests, else
  new fixture in the cells suite) to assert: message names wave-barrier; ack
  value recorded on cell; guard still refuses with neither. Trails pd-1.
- **Barrier (orchestrator, this wave's close):** render mirrors → onboard
  --apply → manifest --write/--check once, in the close commit — the first
  live run of D2.

## Risk Map

| Component | Risk | Proof |
|---|---|---|
| Guard message edit | LOW | pd-2 suite + existing cells tests |
| Doctrine flip within budgets | MEDIUM | skill fence + lint + instr fence green |
| Barrier discipline (orchestrator debt) | MEDIUM | manifest --check green at close; documented in doctrine |

## Test Matrix (targeted)

Message includes barrier alternative · ack recorded verbatim · missing ack+verify
still refuses · budgets: all fences green post-flip.
