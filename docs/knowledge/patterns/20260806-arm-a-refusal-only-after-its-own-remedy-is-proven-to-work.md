---
type: bee.pattern
title: Arm a refusal only after its own remedy is proven to work
description: "A refusal names the command that clears it, so the refusal is only as good as that command — counter-teeth found the route-recording verb bailing under any live worktree grant, which would have made every route-less claim refusal point at a remedy that could not run, and made fixing the remedy a prerequisite cell inside the same feature."
tags: [guards, refusals, remedies, sequencing, fix-first]
timestamp: 2026-08-06
bee:
  id: pattern-20260806-arm-a-refusal-only-after-its-own-remedy-is-proven-to-work
  lifecycle: active
  areas: [workflow-state, worktree-parallelism, hook-runtime]
  decisions: ["counter-teeth D5 (the remedy command was broken for code-touching lanes under any worktree grant, so the route granted-arm fix becomes a prerequisite cell landing before the deny)", counter-teeth D6 (a test proving the counter computes correctly lands before the flip to refusal; no flip ships with a known false positive), 3baa41f6 (counter-teeth proceeds without a route record — the very verb it needed was the one that was broken)]
  sources: ["counter-teeth cell ct-1 (granted-worktree arm ported natively; trace .bee/cells/ct-1.json, commit f6398f8e, 2026-08-04 — state_group 48 passed, 0 failed)", "counter-teeth cell ct-5 (the route-less claim deny landed only after ct-1; trace .bee/cells/ct-5.json, commits 4a0d1b82 and 95ec0639, 2026-08-04)", docs/history/counter-teeth/CONTEXT.md, "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:638-674 (the ported granted-worktree branch)", "cli-help-shape-guard cell chsg-1 (the CLI-shape guard denied the very `--help` its own Correction line named; trace .bee/cells/chsg-1.json, commit 8dd2e846, 2026-08-06 — full suite 1325 passed, 0 failed)"]
  polarity: practice
  critical: false
---

# Arm a refusal only after its own remedy is proven to work

Every refusal worth shipping ends with a way out: *do this, then retry*. That
sentence is a promise about a second piece of machinery, and nothing about
writing the refusal verifies the promise. When the named remedy is itself
broken, the refusal does not enforce a rule — it closes a door and hands over a
key that does not turn.

The instance: `counter-teeth` set out to escalate the route-less claim warning
into a refusal, whose remedy is *record the route*. The route-recording verb was
at that moment bailing out whenever any worktree grant was live, and bailing with
an argument-shape complaint that described nothing true about the situation.
Shipping the deny first would have stopped every claim on an untriaged feature
and pointed each one at a verb that could not run. The feature caught it in
shaping, recorded it as a locked decision, and made the remedy's repair a
prerequisite unit inside the same feature — the deny landed after it, in the same
slice. The feature ran without a route record of its own for exactly the same
reason.

## The rule

- Before arming a refusal, run its remedy. Not read it — run it, in the state
  the refused caller will actually be in (in a worktree, on a lane, mid-swarm).
  A remedy verified only by reading its source has not been verified.
- If the remedy is broken, that repair is the first unit of work, not a follow-up
  ticket. A guard sequenced ahead of its own remedy converts a small defect into
  a work stoppage for everyone who trips the guard.
- Prove the *condition* separately from the *refusal*: land the test that shows
  the counter or predicate computes correctly, then flip the behavior to refuse.
  A flip that ships with a known false positive is a guard that teaches people to
  route around it.
- Suspect a remedy that only ever runs in the happy environment. The route verb
  worked fine from the main checkout; it failed exactly in the multi-worktree
  configuration the refusal would have been most common in.
- The worst case is a remedy the guard blocks itself. bee's CLI-shape guard
  denied a command for missing required parameters and closed with "see
  `<that command> --help --json`" — which the same guard denied, for the same
  missing parameters. The way out pointed back at the door. Whenever a refusal
  names an explanatory surface, that surface must sit in FRONT of the check,
  exempt from it: help teaches the shape, so the shape check may never gate
  help (cell `chsg-1`, 2026-08-06 — reported from a Windows host after an agent
  spent several turns trying to disable the wrong config key instead).
