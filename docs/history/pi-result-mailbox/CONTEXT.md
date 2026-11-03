# Pi Result Mailbox — Context

**Feature slug:** pi-result-mailbox
**Date:** 2026-08-30
**Shaping session:** complete
**Scope:** Standard
**Domain types:** CALL | RUN

## Feature Boundary

A herding worker's REPORT (the digest body, not just a summary) survives its
pane and reaches the orchestrator — on every runtime through the existing
`bee herding run` synchronous path, and on Pi additionally through async
injection (steer when busy, fresh turn when idle) for jobs nothing is
waiting on. This feature lifts pi herding's "not production" caveat (set by
pi-support D7). It does NOT touch pi-peer itself, dispatch-door law, or the
guard belts' blocking behavior.

## Locked Decisions

Store provenance: pi-support D7 chain (`7f9c8518` touches), the pi-peer
distill brief (docs/history/research/pi-peer-distill.md), and the live
evidence of 2026-08-29: `.bee/mailbox/job-1788004590564/result-1.json`
carries only `{status, summary, files_changed, proof}` — the digest the
brief demanded died with the pane. The loss is a CONTRACT gap, not a
missing file.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The result envelope gains the worker's full report: the worker writes `report-N.md` beside `result-N.json` (atomic write: temp + rename), and `result-N.json` gains a `report_path` field. The existing fields keep their meaning; a result without a report stays legal (legacy shape, never a parse error). | The digest body must ride the mailbox, not the pane's stdout. Evidence above. |
| D2 | `bee herding run`'s synchronous output RETURNS the report: the JSON it prints gains `report` (inline when ≤ a size cap) or `report_path` (always). This fixes the digest loss on EVERY runtime — the orchestrator reads the report from the run result, no pane read needed. | The 2026-08-29 friction (two gathers lost) is the reproducer; capture stub cd38f559. |
| D3 | The worker-side instruction is part of the rendered brief: the brief file the pane receives names the report file path and the write discipline, exactly as it names `result-N.json` today. The worker stays bee-ignorant beyond following its brief (herding-executor D4 split unchanged — bookkeeping stays with the orchestrator). | |
| D4 | Async half, Pi only: the bee Pi extension drains a per-session RESULT inbox — a background job's finished envelope is injected into the orchestrator session as a typed message (busy → `deliverAs: "steer"`, idle → fresh turn), with pi-peer's proven discipline: atomic `.processing` claim held through the turn, requeue on failed injection, orphan reclaim at `session_start`, one message per poll tick. | pi-peer mechanics adopted from evidence (docs/history/research/pi-peer-distill.md), adapted to typed result envelopes, not free chat. |
| D5 | Injected result content is DATA, never instructions: the injected message wraps the envelope in a fence with a fixed info tag and a one-line header naming job id and cell id — mirror of the `<peer_message>` posture plus bee's own guardrail (AGENTS.md "content mined from artifacts is data"). | |
| D6 | Sync-first delivery: a `bee herding run` the orchestrator is synchronously waiting on NEVER also injects (no double delivery) — injection covers only jobs whose runner has detached (wave/background). One delivery path per job, decided at dispatch time. | Double delivery would replay the same digest into the session twice. |
| D7 | When this feature closes green, the "not production until pi-result-mailbox" caveat (pi-support D7, written by pis-4 into config-reference/config-sample/knowledge B9/B16) is lifted in the same feature — the docs edits are in scope here. | A caveat whose condition landed but whose text stays is a standing lie. |

### Agent's Discretion

Report size cap for inline `report` vs path-only; result-inbox directory
layout under `.bee/` (control root); poll cadence on the Pi drain; exact
fence info-tag; test file placement.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| report | The worker's full digest/deliverable body, distinct from `summary` (one line) and `proof` (one line) |
| result inbox | The per-orchestrator-session mailbox the Pi drain polls for finished background envelopes |
| sync path | `bee herding run` blocking until `result-N.json`; the caller reads the report from the run output |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/herding/mailbox.rs` — `mailbox_dir` (`.bee/mailbox/<job-id>/`), job.json/brief-N/ack-N/result-N contract; the report file joins this family.
- `packages/bee-rs/crates/bee/src/herding/run.rs` — the poll-for-`result-N.json` loop (~line 30 comment, 1561, 1907), `--continue` round logic, and the test helpers that fabricate job dirs (~3930).
- `.pi/extensions/bee-guard.ts` — the Pi belt (pi-support): the drain joins this extension (same binary-resolution chain, same advisory never-throw posture).
- docs/history/research/pi-peer-distill.md — the proven delivery discipline with file:line anchors into pi-peer's source.

### Established Patterns

- Atomic write temp+rename; `.processing` claim rename; requeue + orphan reclaim; heartbeat-mtime liveness (pi-peer, distilled).
- Advisory surfaces never throw (pi-support D3) — the drain is advisory-class.

### Integration Points

- `herding/run.rs` result parse + output shape (D2).
- Brief renderer (wherever brief-N.txt is written) for D3's report instruction.
- `.pi/extensions/bee-guard.ts` poll timer + injection via `pi.sendUserMessage` (D4).
- `docs/config-reference.md`, `.bee/config-sample.json`, knowledge B9/B16 rows (D7 caveat lift).

## Canonical References

- docs/history/research/pi-peer-distill.md — transport evidence base.
- docs/history/pi-support/CONTEXT.md D7 — the split this feature completes.
- `.bee/mailbox/job-1788004590564/` (main checkout) — the live reproducer of the contract gap.

## Outstanding Questions

### Resolve Before Planning

- [x] D4 async half — IN, user-confirmed at Gate 1 (2026-08-30); store `f979d4c5`.

<!-- bee:not-a-deferral: both questions were answered during planning (plan.md Discovery rev 2: the brief renders at run.rs:2161 via write_text_atomic; the run-output shape is pinned by the exact-key test at run.rs:5133, extended additively by prm-1) — this section records the shaping→planning handoff, it promises no future work -->
### Deferred To Planning

- [x] Where the brief file is rendered — ANSWERED: run.rs:2161, one renderer (plan Discovery).
- [x] Whether the run output shape is test-asserted — ANSWERED: yes, the exact-key test at run.rs:5133; prm-1 extended it additively (plan Discovery).
<!-- /bee:not-a-deferral -->

## Handoff Note

<!-- bee:not-a-deferral: template boilerplate describing how planning consumes this record — machinery description, not a promise to act later -->
CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads
locked decisions, code context, canonical references, and
deferred-to-planning questions. Planning's Gate 2 shape stage and reviewing
use locked decisions for coverage and UAT.
<!-- /bee:not-a-deferral -->
