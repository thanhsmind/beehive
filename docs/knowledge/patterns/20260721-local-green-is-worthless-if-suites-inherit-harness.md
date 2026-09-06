---
type: bee.pattern
title: "Local green is worthless if suites inherit harness env — hermeticity must be structural, and the release gate is the exact-tag CI"
description: "Hermeticity must be structural, and the release gate is the exact-tag CI"
tags: [process, hermeticity, ci, release-gate, session-env, verify]
timestamp: 2026-07-21
bee:
  id: pattern-20260721-local-green-is-worthless-if-suites-inherit-harness
  lifecycle: active
  areas: [verify-pipeline]
  sources: ["docs/history/learnings/critical-patterns.md#PAT46", "original feature: hardening-1-7-10"]
  polarity: pitfall
  critical: true
---

# Local green is worthless if suites inherit harness env — hermeticity must be structural, and the release gate is the exact-tag CI

v1.7.9 shipped citing "53/53 green" while the tag's CI was red on every platform: test_state
passed locally ONLY because Claude Code exports CLAUDE_CODE_SESSION_ID, letting a sessionless
reserve dodge SESSION_REQUIRED. Proof was one command: `env -u CLAUDE_CODE_SESSION_ID` flipped
local green to the exact CI red. Structural fixes: run_verify scrubs session env from every child;
session-sensitive suites also scrub at bootstrap; close-out verifies in BOTH modes. Corollary the
same day: windows.yml invoked a deleted suite for a full release cycle because nothing tied
workflow steps to existing files — CI now reuses the runner's own suite discovery. Never cite a
local green as release evidence; the gate is the exact-tag CI run.
