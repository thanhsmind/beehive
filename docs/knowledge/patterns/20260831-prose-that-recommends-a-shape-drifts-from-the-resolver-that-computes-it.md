---
type: bee.pattern
title: Prose that recommends a dispatch shape drifts from the resolver that computes it
description: Three documents each told an agent a different preferred subagent_type/shape for the same dispatch, and all three disagreed with the running guard — a resolver command that reads the live config is the only shape that cannot drift, because it has nothing of its own to remember.
tags: [dispatch, drift, delegation, doctrine]
timestamp: 2026-08-31
bee:
  id: pattern-20260831-prose-that-recommends-a-shape-drifts-from-the-resolver-that-computes-it
  lifecycle: active
  areas: [hook-runtime, doctrine-layer]
  sources: ["dispatch-one-door CONTEXT.md (docs/history/dispatch-one-door/CONTEXT.md) — 7 drift sites across model_guard.rs, gates-and-delegation.md, swarming-reference.md, status_full/store.rs", "dispatch-one-door D1 (c80e0220, 2026-08-21)", "cells dod-1..dod-6 (packages/bee-rs/crates/bee/src/hooks/model_guard.rs, status_full/store.rs, skills/bee-hive/references/gates-and-delegation.md, skills/bee-swarming/references/swarming-reference.md)"]
  polarity: pitfall
  critical: false
  evidence: exercised
  evidence_ref: "reporting host vnbptw-mapcompany (models.claude.generation = {kind:\"herding\"}) hit herding-tier-denied and bare-denied on every subagent dispatch; bee dispatch prepare already resolved the slot correctly, only the prose readers were not consulting it"
---

# Prose that recommends a dispatch shape drifts from the resolver that computes it

A host repo running `models.claude.generation = {kind:"herding"}` had every
subagent dispatch refused: `Agent(subagent_type: "bee-gather")` denied
`herding-tier-denied`, a bare `Agent(subagent_type: "Explore")` denied
`bare-denied`. Both refusals were correct — a PreToolUse hook can only allow
or deny, it cannot rewrite an `Agent` call into the `Bash` call a herding pane
actually needs. The defect was upstream of the guard: agents were told, by
prose, to name a `subagent_type` instead of asking the config.

Three separate documents each gave a different answer for the same dispatch,
and all three disagreed with the code:

- a skill reference said "prefer this shape: `subagent_type: bee-build|bee-gather|…`" —
  refused outright when the slot is `cli`/`herding`;
- another skill reference said an unrendered tier falls back to
  `general-purpose` — denied by name (`generic-type-denied`);
- a third said a `{kind:"herding"}` tier "does not exist yet" — it existed,
  shipped, and was exactly what the reporting host ran.

`bee dispatch prepare` already resolved every slot shape correctly the whole
time; nothing in the dispatch pipeline needed new code. The fix was entirely
in what readers were told: point every prose surface and every refusal's FIX
line at the one resolver verb, and stop letting any of them state — as a fact
that can go stale — what shape a slot currently takes.

The rule: when more than one document tells an agent how to invoke something
whose real answer lives in a config file, they will independently drift from
that config and from each other, because each one is a second copy of a fact
with no mechanism keeping it synced. The fix is never to patch each copy —
that just adds a fourth answer waiting to go stale — it is to make the
resolver itself the only thing anyone is told to consult, and let every
refusal's remedy name that resolver by command, not describe its current
output in prose.
