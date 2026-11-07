---
type: bee.delivery
title: leader-sees-team — delivery
description: "Delivery record proposed by bee knowledge promote for work item leader-sees-team: 3 capped cell(s), 6 recorded deviation(s)."
timestamp: 2026-09-09
bee:
  id: leader-sees-team-delivery
  lifecycle: active
  areas: [doctrine-layer, advisor-protocol]
  required_context: [docs/history/leader-sees-team/CONTEXT.md, docs/history/leader-sees-team/plan.md]
  sources: [docs/history/leader-sees-team/CONTEXT.md, docs/history/leader-sees-team/plan.md, .bee/cells/archive/leader-sees-team/lst-1.json, .bee/cells/archive/leader-sees-team/lst-2.json, .bee/cells/archive/leader-sees-team/lst-3.json]
---

# leader-sees-team — Delivery

## What shipped

- **lst-1** — bee reads the model out of the argv it builds for herding and cli dispatches and records it as declared, from both writers of the economics record (5 file(s) changed)
- **lst-2** — the preamble roster prints every role one per line as role, model, transport; bee team show leads with role, source and description, then model and transport, and its JSON carries model, transport and the raw slot (2 file(s) changed)
- **lst-3** — the pick-by-job rule is in the host AGENTS.md block, the delegation contract and the worker prompt; the false cli-exec sentence is rewritten; declared is in the vocabulary homes; B12/B13 cite D4 (9 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **lst-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee -- drivers model_guard`
- **lst-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee -- model_guard models_group session_preamble compaction`
- **lst-3** — `rg -q 'never the leader' packages/bee/AGENTS.block.md AGENTS.md && rg -q 'never the leader' skills/bee-hive/references/gates-and-delegation.md && rg -q 'config, not a signal' packages/bee/prompts/worker-cell.md && ! rg -q 'is always .null. there' skills/bee-swarming/references/swarming-reference.md && rg -q 'declared' docs/knowledge/areas/advisor-protocol/slots-and-tiers.md docs/config-reference.md && rg -q 'D4' docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md`

## Deviations

- **lst-1** — Capped with --sync-ack: read-only call into bee-herding-owned wave.rs; the doctrine homes were updated by lst-3.
- **lst-1** — sync-ack: the helper calls into herding/wave.rs read-only (resolve_agent_command_for_runtime) and changes no herding behaviour; the doctrine homes this feature owes were updated by lst-3 (gates-and-delegation, worker-details, swarming-reference), and no skills/bee-herding text describes the economics record
- **lst-2** — Capped with --sync-ack: display-only files an ownership map ties to bee-herding; the doctrine amendment for this display law (D4) landed in lst-3.
- **lst-2** — sync-ack: display-only change in model_guard.rs and models_group.rs; the roster and team show texts are governed by D4, whose doctrine amendment lst-3 already landed in model-roles-and-escalation.md B12/B13
- **lst-3** — Capped with --sync-ack: AGENTS.md is a rule home for agents-capture-line-at-close, which this one-sentence door-bullet edit does not touch.
- **lst-3** — sync-ack: the AGENTS.md edit adds one sentence to the dispatch-door bullet; it does not touch the agents-capture-line-at-close rule the sync door keys on, so that rule's applied_at files owe no update

## Provenance

Proposed by `bee knowledge promote --work leader-sees-team` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/leader-sees-team/CONTEXT.md`, `docs/history/leader-sees-team/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
