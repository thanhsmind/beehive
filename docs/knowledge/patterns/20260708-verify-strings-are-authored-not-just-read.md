---
type: bee.pattern
title: "Verify strings are authored, not just read — two traps"
description: "A cell’s verify command must be dry-run once before it reaches a worker, not reviewed as prose"
tags: [failure, verify-strings, shell, validation, prose-cells]
timestamp: 2026-07-08
bee:
  id: pattern-20260708-verify-strings-are-authored-not-just-read
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT9", "original feature: harness10", docs/history/learnings/20260708-harness10.md]
  polarity: pitfall
  critical: false
---

# Verify strings are authored, not just read — two traps

A cell's `verify` command must be executed once before it reaches a worker, not reviewed as prose.
Two traps, both survived static review this feature:
1. **Metacharacter regex:** `grep -q '['` is an invalid regex and aborts the `&&` chain. Dry-run any
   verify containing regex/glob metachars (`[ * ? |`) in the target shell, or use `grep -F` for literals.
2. **Grep-for-prose gaming:** a verify that greps for an invented multi-word token rewards embedding that
   token verbatim into prose. Grep a **stable heading** the section needs anyway, never an invented phrase.

**Full entry:** docs/history/learnings/20260708-harness10.md
