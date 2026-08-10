---
type: bee.pattern
title: "Empty child-process output can be a sandbox denial, not a regression"
description: "Empty child-process output can be a sandbox denial, not a regression"
tags: [failure, codex, sandbox, child-process, verification]
timestamp: 2026-07-12
bee:
  id: pattern-20260712-empty-child-process-output-can-be-a-sandbox
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT27", "original feature: harness-integration-adopt", docs/history/learnings/20260712-harness-integration-adopt.md]
  polarity: pitfall
  critical: false
---

# Empty child-process output can be a sandbox denial, not a regression

A baseline run reported 40 CLI failures whose only visible symptoms were empty output and secondary
JSON parse errors. The actual child-process result carried `spawnSync ... EPERM`; the unchanged
verify passed `215/0` outside the sandbox and onboarding had zero failures. When a CLI-heavy suite
fails this way, inspect the spawn error first and rerun unchanged with the required execution
permission before creating a fix cell or weakening assertions.

**Full entry:** docs/history/learnings/20260712-harness-integration-adopt.md
