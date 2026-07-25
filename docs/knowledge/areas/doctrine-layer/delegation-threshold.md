---
type: bee.area
title: Doctrine Layer — the delegation threshold
description: "When mechanical gathering is handed to a lower-cost helper, what decide-altitude work never delegates, and why the threshold holds in every lane and every turn."
timestamp: 2026-07-21
bee:
  id: doctrine-layer-delegation-threshold
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/overview.md]
  decisions: [D1/D2/D3 delegation contract]
  sources: ["fanout-doctrine (cell fanout-doctrine-1, 2026-07-13, flushed capture stub 2f796f40)", "advisor-and-orchestration Slice 2A-iii (cells ao-2aiii-1/ao-2aiii-2 — dispatch-boundary enforcement + gather-purpose routing prose, 2026-07-17)", "docs/specs/doctrine-layer.md#B3", "docs/specs/doctrine-layer.md#R3", "docs/specs/doctrine-layer.md#R4", "docs/specs/doctrine-layer.md#R5", "docs/specs/doctrine-layer.md#E2", "docs/specs/doctrine-layer.md#P3", "docs/specs/doctrine-layer.md#P5"]
  authoritative_for: "doctrine-layer: the delegation threshold"
---

# Doctrine Layer — The Delegation Threshold

The resource being protected is the orchestrator's own limited attention, and it
is consumed in every turn. This concept owns the line between gather-altitude
and decide-altitude, the threshold that decides when a mechanical step is handed
off, and the fact that the smallest lanes never suspend it.

## Behaviors & Operations

**B3 — Mechanical gathering is delegated to helpers; deciding is not.** Trigger:
the assistant faces a mechanical step during any turn. What decides delegation:
the step is handed to a lower-cost helper when it requires reading more than
three sources, or when its content is needed only as a summary rather than word
for word. What each actor observes: the helper returns the sources it read, the
facts with precise anchors, and quoted material only where it was asked for; the
orchestrator never re-reads what a summary already answered, and never delegates
decide-altitude work. Why the rule exists as doctrine rather than stage
procedure: the resource being protected is the orchestrator's own limited
attention, and it is consumed in *every* turn — most damagingly in conversation
turns, where no stage is running to remind it (B2's failure mode, observed).

## Business Rules

- **R3** — Mechanical gathering delegates to a lower-cost helper when it needs
  more than three sources, or content wanted only as a summary. The orchestrator
  may override in either direction; the threshold is judgment, not a mechanism.
- **R4** — Decide-altitude work never delegates: gates, the mode decision,
  synthesis, accept/reject of a helper's result, state writes, and conversation
  with the human (D1).
- **R5** — The delegation threshold applies in every lane and every phase,
  including the smallest ones. The lanes' "no helpers" rule for small work means
  no *ceremony* helpers — reviewers, checkers, panels — never no gathering
  helpers (D3).

## Edge Cases Settled

- **The lanes' "zero helpers" rule is not zero helpers.** Small-work lanes
  suppress ceremony helpers, not gathering helpers. A one-file fix simply never
  crosses the delegation threshold on its own, so it stays in-session naturally —
  the suppression and the threshold never actually conflict.

## Pointers (implementation)

- The delegation contract in full (tiers, digest contract):
  `skills/bee-hive/references/routing-and-contracts.md` § Delegation contract —
  the *detail* legitimately lives there; the rule itself, and the transport it
  requires (a `model` param or an anchored `[bee-tier:]` marker, B3a), are
  critical rule 13 on the standing sheet. The guard that rejects a bare,
  config-disagreeing, or cli-tier-declared dispatch (declared tier read before
  the model param, 2A-iii): `packages/bee/hooks/bee-model-guard.mjs`.
- Model tiers behind R3: `.bee/config.json` `models` (extraction / generation /
  review / advisor slots), resolved per dispatch by `bee-swarming`.
