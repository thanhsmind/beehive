---
type: bee.delivery
title: role-slot-description — delivery
description: "Delivery record proposed by bee knowledge promote for work item role-slot-description: 1 capped cell(s), 4 recorded deviation(s)."
timestamp: 2026-08-26
bee:
  id: role-slot-description-delivery
  lifecycle: active
  required_context: [.bee/lanes/role-slot-description.json]
  sources: [.bee/lanes/role-slot-description.json, .bee/cells/archive/role-slot-description/rsd-1.json]
---

# role-slot-description — Delivery

## What shipped

- **rsd-1** — The dispatch-door roles line now prints a role slot optional description beside its model; resolution and the guard stay blind to the field (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rsd-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee model_guard session_preamble status_full — green, including a new test asserting a described role renders name=model ("desc") and an existing-config test asserting byte-identical output without descriptions; bee dev regen leaves the tree clean.`

## Deviations

- **rsd-1** — The cells-add help sentence went into packages/bee-rs/crates/bee/src/generated/registry_payload.json, not catalog.rs — catalog.rs only PARSES that payload and carries no help prose, so the sentence had nowhere to live in the named file — the plan was wrong about a fact
- **rsd-1** — The status_full validator test went into verbs/status_full/tests.rs, not store.rs — store.rs has no test module at all and every validate_models_config test in the crate already lives in tests.rs beside valid_models_configs_produce_zero_problems — found a better route
- **rsd-1** — The model_guard unit tests were added to model_guard.rs own tests module in addition to the preamble tests, so the clipping and resolution-blindness rules are proved where the code lives — found a better route
- **rsd-1** — Reserving registry_payload.json hit a live cross-worktree hold from main (feature reattribute-by-name, cell rbn-1) that expired at 03:51:04Z; I waited it out and reserved cleanly rather than writing through it — hit an unforeseen obstacle

## Provenance

Proposed by `bee knowledge promote --work role-slot-description` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/role-slot-description.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

