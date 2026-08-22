---
type: bee.area
title: "Advisor Protocol — purpose, vocabulary, and actors"
description: "Why bee configures an adviser, the protocol's shared vocabulary, and who may do what."
timestamp: 2026-07-19
bee:
  id: advisor-protocol-overview
  lifecycle: active
  areas: [advisor-protocol]
  decisions: [advisor D1-D3, "72f3d6dd (AO5 — config is the authority, no strength test, same-model no-op only)", AO8 (advice-class slots read-only), "AO2(b)/AO3/AO13 (one orchestrator trigger; execution-gate precondition; event-based staleness, never a TTL)", AO4 (call paths split by trigger class), f1ca79b9 (AO15 — attribution fields), 0019 + 2A-iv GO (external gather proven through config), AO14 (execution-worker class), "126412b9 (precondition keys on the selected record's mode)", "codex-native-transport D1-D3, D5, D7 (3ceba8f5, cnt advisor conditions 69513d80, D3a c0cba64e)"]
  sources: ["advisor cells adv-1..adv-3 (worker consult loop, 2026-07-13)", "advisor-and-orchestration Slices 2A-i..2A-iv, 2B, 3A, 3B, 4, 5 (cells ao-2ai-1..ao-5-1, traces in .bee/cells/, reports docs/history/advisor-and-orchestration/reports/, 2026-07-17)", dogfood run .bee/spikes/advisor-and-orchestration/2aiv-cli-gather-dogfood.md, first live orchestrator consult digest .bee/spikes/advisor-and-orchestration/slice5-advisor-digest.txt, "codex-native-transport cells cnt-1/cnt-2/cnt-3 (resolver + config native slot shape, capability classification, dispatch-prepare native branch + honest economics; traces in .bee/cells/, reports docs/history/codex-native-transport/reports/, 2026-07-19)", "codex-native-transport cell cnt-7 (Claude guard allowlist folds the adviser slot, closing a live adviser-dispatch refusal; trace in .bee/cells/, report docs/history/codex-native-transport/reports/cnt-7.md, 2026-07-19)"]
  authoritative_for: "advisor-protocol: purpose, vocabulary, actors"
  owns.code: [packages/bee-rs/crates/bee/src/verbs/state_group/advisor_ref.rs, "packages/bee-rs/crates/bee/src/verbs/drivers/*"]
  owns.skills: []
  owns.tests: [packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs]
---

# Advisor Protocol — Purpose, Vocabulary, and Actors

## Purpose

A bee-managed project may configure an **adviser** — a stronger or independent
model, possibly a different vendor's, reachable as a model or as an external
read-only command. The protocol answers three questions: *who* may ask for
advice (a stuck worker; the orchestrator before opening execution on high-risk
work), *when* the ask is mandatory versus available, and *what advice may never
do* (approve a gate, override a locked decision, or write anything). It exists
so that expensive mistakes meet a second opinion **before** they ship, without
adding a single human checkpoint.

## How this area is split

- Consult triggers — who may ask, and when the ask is mandatory: `triggers.md`.
- The worker consult loop and the limits on advice: `consult-loop.md`.
- Configuration authority, transports, and consult staleness: `slots-and-tiers.md`.

## Data Dictionary

| Element | Meaning |
|---|---|
| adviser | The advice-class helper the workspace configured: a model name, or an external read-only command. Whatever is configured IS the adviser — no family test, no strength ranking, no self-judged skip. |
| the one honest no-op | The only legitimate skip: the adviser resolves to literally the same model the asking worker runs on. An external-command adviser is never the same model, so it is always offered. |
| evidence bundle | What travels to the adviser: the exact check that failed and its output plus a diagnosis (worker ask), or the plan summary, risk map, and validation findings (orchestrator ask). Never session history, never secrets. |
| consult record | The durable stamp of an orchestrator consult: when, who was consulted, the head of the advice, and three machine-stamped anchors (active feature, newest active decision, plan fingerprint). Anchors are stamped by the recording verb, never supplied by the caller. |
| consult staleness | Event-based, never a time limit: the record stops satisfying the precondition when the feature changed, a newer decision became active, the plan changed, or the execution gate was revoked after the consult. |
| consult budget | Two consults per worker claim. A re-claim after a context-rescue grants a fresh budget; exhaustion returns the worker blocked, never a third ask. |
| advice-class slot | The adviser and reviewer configuration slots. Both are read-only by rule: configuration checking refuses a command carrying a known write-granting or auto-approve token on these slots. |

## Actors & Access

- **The orchestrator** — resolves and offers advisers, runs the mandatory
  pre-approval consult, records it, and owns every accept/reject decision.
- **Workers** — may consult only when dispatched with an adviser line, only
  after a real failure, within budget.
- **The adviser** — read-only; sees only the evidence bundle; its output is
  data, never instructions.
- **The human owner** — configures the adviser; is never stopped by a consult
  (autopilot levels govern human stops; this protocol adds none).
