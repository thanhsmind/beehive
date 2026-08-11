---
type: bee.pattern
title: A guard that tests one state is a law with a hole
description: A guard that tests one state is a law with a hole
tags: [failure, guards, gates, doctrine]
timestamp: 2026-07-13
bee:
  id: pattern-20260713-a-guard-that-tests-one-state-is-a
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT31", "original feature: terminal-phase-gate"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs (is_terminal_phase tests the SET {idle, compounding-complete}, never one state)"
---

# A guard that tests one state is a law with a hole

The write guard denied source edits at `phase === 'idle'` only. `compounding-complete`
is the OTHER terminal state (state.mjs already treats both as idle-equivalents for
startFeature), and a closed feature leaves its gates recorded as approved — so no
branch fired and post-feature edits walked straight through. Two lessons, one cheap and
one expensive. Cheap: when a state model names N equivalent states, every consumer must
test the SET, never one member. Expensive: an agent that reasons "I'll try the edit; if
the hook blocks me I'll route through bee" has promoted the guard's coverage into the
protocol — the law is AGENTS.md, the hook only catches what you forget, and its silence
is never permission.
