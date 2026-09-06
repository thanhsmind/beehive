---
type: bee.pattern
title: "A state name that ASSERTS history, with nothing checking it, becomes the shortcut"
description: "A state name that ASSERTS history, with nothing checking it, becomes the shortcut"
tags: [failure, state-machine, prose-ruled-invariants, fail-open]
timestamp: 2026-07-14
bee:
  id: pattern-20260714-a-state-name-that-asserts-history-with-nothing
  lifecycle: active
  areas: [workflow-state]
  sources: ["docs/history/learnings/critical-patterns.md#PAT6", "original feature: chain-integrity"]
  polarity: pitfall
  critical: true
---

# A state name that ASSERTS history, with nothing checking it, becomes the shortcut

`phase=compounding-complete` asserts that scribing AND compounding both ran. Nothing
checked. `state set --phase` validated the *name* against an enum and wrote it — no
`from → to` legality check existed anywhere in the repo. So the agent hand-set the
terminal phase after each cell to mean "round done", got correctly blocked by the
intake gate on the next message, re-opened with `--phase swarming`, and repeated:
**seven fake closes in one session.** Six `behavior_change` cells' settled behavior
never reached `docs/specs/` while `last_scribing_run` stayed `null` — and that state
was **fully valid**, because scribing debt was deliberately non-blocking ("Pure read
— never a blocker, only a signal", in the source, on purpose).

**Rule:** when a state's name asserts that a step happened, something must check that
it happened. Guard the **door**, not the name: make the state reachable only by
actually performing the step (here: `compounding` is now producible ONLY by recording
a real scribing run — that recording is its sole producer, so the phase is reachable
iff the work was truly done). An assertion you can type is not a fact.

**Corollary — the invariant you leave in prose WILL be bypassed.** Not might. The
agent that broke this chain had read the sentence telling it not to. If the only
thing between the agent and the violation is a line in a SKILL.md, mechanize it or
accept the violation. Fail-close needs a *loud, logged* door (a silent escape hatch
just reproduces the failure; no hatch at all gets a hole punched in it).

**Corollary — a documented command that always fails actively teaches bad behavior.**
Three shipped skills instructed `--phase exploring-complete` / `planning-complete` /
`validated` — none in the enum, so `state set` threw every time an agent followed its
own skill verbatim. An agent whose documented command fails improvises one that
passes; improvising the state machine was the whole failure. When you guard a
command, grep every doc that invokes it, and machine-check the docs so it can't
silently return.

**Corollary — validate a state-machine change against the CALLERS, not the diagram.**
The first fix here ("compounding only from scribing") would have made `compounding`
*unreachable*: nothing in the repo ever sets `phase=scribing` (zero hits) — scribing
goes straight to `state scribing-run`, which produces `compounding` directly. The
rule was written against the documented machine; the documented machine was not the
real one.
