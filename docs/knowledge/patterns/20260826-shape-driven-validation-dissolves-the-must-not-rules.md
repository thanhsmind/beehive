---
type: bee.pattern
title: Shape-driven validation silently dissolves the must-not rules
description: Rebuilding a validator from allow-list to walk-what-is-configured is right for open sets, but every existing MUST-NOT rule needs an explicit deny arm carried over — otherwise the forbidden key becomes legal without anyone deciding it.
tags: [validation, config, refactor]
timestamp: 2026-08-26
bee:
  id: pattern-20260826-shape-driven-validation-dissolves-the-must-not-rules
  lifecycle: active
  areas: [rust-runtime, doctrine-layer]
  sources: ["role-edge-hardening reh-1, 2026-08-25 — a well-formed models.<rt>.ceiling key passed the rebuilt validator with zero problems, though decision 0015 forbids the key outright", "packages/bee-rs/crates/bee/src/verbs/status_full/store.rs (validate_models_config, the ceiling-not-a-role arm)"]
  polarity: pitfall
  critical: false
  evidence: exercised
  evidence_ref: "a_configured_ceiling_key_is_named_never_silently_accepted was shown red against the shape-driven validator (zero problems on a forbidden key) before the deny arm landed"
---

# Shape-driven validation silently dissolves the must-not rules

`validate_models_config` was rebuilt to walk what the config carries instead of
asking a closed list what is allowed — the right inversion for an open role
set, and the commit's own comment says why: a closed list meant an invented
role's junk value was "dropped by the parser and reported by nobody".

The inversion had a blind spot the other way. Under the old allow-list, a
forbidden key never validated because it never appeared; under shape-walking,
the forbidden key validated **successfully** — a well-formed
`models.<rt>.ceiling` value passed every shape check, because shape checks were
all that remained. Decision 0015 forbids that key; the rule had no code home
anymore, so it dissolved without anyone deciding to drop it. Downstream, the
silently-accepted key made `dispatch prepare` stamp a `[bee-tier: ceiling]`
marker beside a model param — the exact pair the guard denies.

The rule: when validation moves from *allow-list* to *shape-driven*, enumerate
the old regime's implicit MUST-NOTs and give each an explicit deny arm before
the switch. An allow-list encodes prohibitions by omission; shape-walking
encodes none, so every prohibition must become code or it becomes legal.
