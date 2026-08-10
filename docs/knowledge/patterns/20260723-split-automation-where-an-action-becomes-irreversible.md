---
type: bee.pattern
title: "Split automation where an action becomes irreversible — automate the recoverable half, keep the irreversible half a human gesture"
description: "When automation both produces work and lands it, the two halves have different blast radii. Autonomy should track blast radius, not convenience."
tags: [process, automation, safety-posture, unattended, review]
timestamp: 2026-07-23
bee:
  id: pattern-20260723-split-automation-where-an-action-becomes-irreversible
  lifecycle: active
  sources: ["herding-adopt cells h-2, h-3 (an unattended dispatch+merge loop adopted with merge demoted to a gesture; traces in `.bee/cells/`, 2026-07-23)", docs/history/learnings/20260723-adopting-a-contribution-means-reviewing-what-it-does-not-what-it-says.md]
  polarity: practice
  critical: false
---

# Split automation where an action becomes irreversible

An adopted contribution ran two unattended loops: one **started** work in an isolated copy, the other
**landed** it in the shared trunk. The original design gave both the same autonomy. But their blast
radii are not the same — starting work in a throwaway worktree is recoverable; landing it in main is
not. Every serious risk of the whole system concentrated in the landing half: merge authority in
unattended hands, a ~91-minute window where a stop gesture could not stop an in-flight merge, and the
project's verify running over code that unsandboxed agents had just written.

The adoption kept the **starting** loop unattended and demoted the **landing** loop to a single-shot
the owner runs by hand. One of four hard-gate risk flags dropped outright; the stop-latency problem
shrank to the harmless half; the cost was the owner being present when something lands — which for a
system whose own safety filter passed "delete the entire JS runtime" is a feature, not a limitation.

**The rule: split automation at the point where an action becomes hard to reverse, and keep the
irreversible side a human gesture while the reversible side loops unattended.** The test is not "can
this step be automated" — most can — but "what does one bad iteration of this step cost." A
recoverable step can afford to be wrong occasionally; an irreversible one cannot, and no amount of
in-loop checking makes an unattended irreversible step as safe as a present human.

**Corollary — the guards that made the irreversible side "safe enough" to automate are usually the
weakest part.** Here they were a language model counting panes, a language model reading a one-line
backlog row, and a stop file checked once per cycle. Each looked like a containment and enforced
nothing. When you find yourself adding guards to make an irreversible unattended step acceptable, that
is the signal to make it a gesture instead — the guards are doing the job a present human would do,
worse.
