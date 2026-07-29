---
type: bee.pattern
title: Promote an order to the always-loaded layer and its transport must ride along
description: Promote an order to the always-loaded layer and its transport must ride along
tags: [failure, doctrine, layering, hooks, dispatch]
timestamp: 2026-07-13
bee:
  id: pattern-20260713-promote-an-order-to-the-always-loaded-layer
  lifecycle: active
  decisions: [0023]
  sources: ["docs/history/learnings/critical-patterns.md#PAT30", "original feature: tier-transport-doctrine"]
  polarity: pitfall
  critical: true
---

# Promote an order to the always-loaded layer and its transport must ride along

Critical rule 12 (fan out the gathering) was promoted into AGENTS.block.md so it holds in
plain conversation turns — but HOW to dispatch (a `model` param or an anchored `[bee-tier:]`
marker, decision 0023) stayed in `bee-hive/references/routing-and-contracts.md`, which loads
only on skill invoke. So the rule fired exactly where its mechanics were absent: every host
session's first dispatch was born bare and `bee-model-guard` denied it, teaching the transport
at deny time, one wasted dispatch per session. When a standing rule commands an action that a
guard rejects in its bare form, the standing sheet carries the order AND the minimum needed to
obey it first try; only the rationale and elaboration may be referenced.
