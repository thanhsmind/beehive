# Parallel By Default — Context

**Feature slug:** parallel-default
**Date:** 2026-07-28
**Exploring session:** complete (user directive; bypass TOTAL — gray areas locked from evidence)
**Scope:** Standard
**Domain types:** ORGANIZE (doctrine text), RUN (regen guard)

## Feature Boundary

Flip bee's execution doctrine from serial-default to **parallel-default**: cells
in the same slice run concurrently whenever ownership is disjoint (reservations
prove it); serial is the exception that names its conflict. Unlock the main
false-serializer — per-cell regen of shared generated artifacts — via a
**wave-barrier regen** convention the existing `regen_obligation_ack` escape
already supports. Verify concurrency stays at its measured optimum (no change).

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | **Parallel is the default** for cells of the same slice, every lane: dispatch concurrently (3–4 live workers, ak-calibrated cap) whenever every cell's *product* file set is disjoint; reservations are the proof and the police. Serial requires a named conflict recorded in the dispatch note. Supersedes "serial stays the default" (routing-and-contracts Parallel criterion) and rescopes hardening-7's small-lane serial doctrine (small-lane cells parallelize too when disjoint; still one worker per cell). | User philosophy directive 2026-07-28 (decision logged): bee's state/cell/reservation design was built for this; ak comparison showed ceremony, not machinery, is where wall-clock goes. |
| D2 | **Wave-barrier regen:** a cell touching a manifest-hashed root may set `regen_obligation_ack: "wave-barrier"` — it skips in-cell regen; the ORCHESTRATOR then owes the full regen chain (mirror render → onboard --apply → manifest --write/--check) exactly once at wave close, in the wave-close/close commit, before the wave is declared clean. This removes shared generated artifacts (manifest, mirrors, onboarding ledger, baseline files) from cells' effective file sets, so the scheduler's overlap check sees真disjoint sets. The guard's refusal message names this alternative explicitly. | The regen targets were the near-universal overlap forcing serial (all 6 diet migrations serialized on them). The ack field already exists — this names a recognized value and pins the orchestrator's debt. |
| D3 | **Verify concurrency unchanged** at `min(5, cpus)` (override via `BEE_VERIFY_CONCURRENCY`). | Measured: 5 → 32s stable 8/8 consecutive; 6 flaked (suite starvation); unbounded 16 flaky. "Max parallel" means max *useful* — the evidence already found it. |
| D4 | Validating stays wave-parallel as shipped (spec #77: reviewer + matrix concurrent, delta cache) — no further change in this feature. | Already landed; re-verified by audit. |

### Agent's Discretion

- Exact wording/placement of doctrine text, within migrated-body byte budgets
  (swarming body has 5 bytes headroom — net growth goes to references).
- Guard-message phrasing for the wave-barrier alternative.

## Existing Code Context

- `packages/bee/lib/cells.mjs` ~:380-400 — REGEN_OBLIGATION refusal + `regen_obligation_ack` escape (recorded on the cell).
- `skills/bee-swarming/SKILL.md:27` (small-lane row), `references/swarming-reference.md:25-40` (hardening-7 + parallel criterion), `skills/bee-hive/references/routing-and-contracts.md:293,299,527` (lane table, Small-lane serial doctrine, execution-worker class) — the serial-doctrine text to flip.
- `scripts/run_verify.mjs:1275-1281` — measured concurrency rationale (D3 evidence).
- `bee cells schedule` — already auto-serializes real file overlap into waves; shrinking overlap widens waves with zero scheduler change.

## Outstanding Questions

### Deferred To Planning
- [ ] Which existing suite covers the regen guard message (extend vs new fixture).

## Deferred Ideas

- Scheduler-computed concurrency hint (`cells schedule --json` emitting max-width) — not needed for the doctrine flip.

## Handoff Note

Decision IDs stable; planning implements exactly.
