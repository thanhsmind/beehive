---
type: bee.pattern
title: A guard scoped inside a skippable loop is absent on the path that skips it
description: A guard scoped inside a skippable loop is absent on the path that skips it
tags: [failure, safety-guards, guard-placement, self-onboard, fail-open]
timestamp: 2026-07-15
bee:
  id: pattern-20260715-a-guard-scoped-inside-a-skippable-loop-is
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT5", "original feature: codex-harness-hardening", docs/history/learnings/20260715-codex-harness-hardening-1b.md]
  polarity: pitfall
  critical: false
---

# A guard scoped inside a skippable loop is absent on the path that skips it

A correct three-version downgrade preflight existed and had protected ordinary hosts for
months — but it lived *inside* the per-skill-target loop. On the self-onboard path every
target `self_skip`s with `continue` before the check runs, so the guard was skipped with
the targets, while the sibling `copy_lib`/`copy_helper` loops downgraded `.bee/bin`
unconditionally. The guard read run-global data (`hostVersion`) but had target-scoped
*placement*.

**Rule:** when a safety check depends only on run-global data, place it at run-global
scope, never inside a per-item loop that can be skipped wholesale. Before trusting an
existing guard, ask "on which code path is this guard's PLACEMENT skipped?" — not just
"does it read the right values?". And when you add an ungated mutation path (a copy/write
loop) beside a gated one, it inherits NONE of the old path's guards: audit every mutation
vector against the guard, not the guard against one vector. Fix generalizes as: hoist the
run-global check to fire unconditionally, fill the aggregate only when it's empty (no
double-block), then reuse the existing whole-apply abort. Full entry:
docs/history/learnings/20260715-codex-harness-hardening-1b.md
