---
type: bee.pattern
title: A fail-open host swallows fail-closed throws into an allow
description: A fail-open host swallows fail-closed throws into an allow
tags: [failure, hooks, fail-closed, guards, security]
timestamp: 2026-07-14
bee:
  id: pattern-20260714-a-fail-open-host-swallows-fail-closed-throws
  lifecycle: active
  areas: [hook-runtime]
  sources: ["docs/history/learnings/critical-patterns.md#PAT32", "original feature: fresh-session-handoff", docs/history/learnings/20260714-fresh-session-handoff.md]
  polarity: pitfall
  critical: true
---

# A fail-open host swallows fail-closed throws into an allow

The write-guard hook exits 0 (allow) on ANY crash by contract. A guard branch that
must fail closed therefore may NEVER throw — it must RETURN a typed deny verdict,
or the host converts the denial into a silent grant. The strict-reader precedent
(`readStateStrict` throws) is the wrong template inside a fail-open host. Prove
fail-closed paths through the real host process, not only in-process.

**Full entry:** docs/history/learnings/20260714-fresh-session-handoff.md
