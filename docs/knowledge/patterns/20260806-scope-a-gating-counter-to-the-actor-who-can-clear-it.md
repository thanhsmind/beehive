---
type: bee.pattern
title: Scope a gating counter to the actor who can clear the fault
description: "A once-then-refuse counter kept per feature would have refused every worker in a swarm but the first, for a fault none of them could fix from where they stand — scoping it per feature AND per session keeps the teeth while leaving each actor its own one warning, and the counter advances only on a claim that actually succeeded."
tags: [counters, refusals, concurrency, swarm, scoping]
timestamp: 2026-08-06
bee:
  id: pattern-20260806-scope-a-gating-counter-to-the-actor-who-can-clear-it
  lifecycle: active
  areas: [workflow-state]
  decisions: [counter-teeth D4 (the first route-less claim keeps the warning; the second and later refuse), "64ad772d (D4 refined during execution: the counter is scoped per feature and session, and the already-claimed contention refusal outranks the no-route deny)"]
  sources: ["counter-teeth cell ct-5 (per-(feature, session) counter under the control root, bumped only after a successful claim; trace .bee/cells/ct-5.json, commits 4a0d1b82 and 95ec0639, 2026-08-04 — cells 75 passed, concurrency 13 passed, full suite green)", "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs:723-815 (the counter's key, path, and bump site)", "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs:1081-1113 (the ordering comment: a real race loser must see the contention refusal, never this one)"]
  polarity: practice
  critical: false
---

# Scope a gating counter to the actor who can clear the fault

A "warn once, then refuse" counter looks like a property of the thing being
guarded. It is really a property of the conversation between the guard and one
actor: the warning is an education, and education is per learner. Key the count
to the wrong subject and the second learner is punished for the first one's
lesson.

The instance: the route-less claim counter. Kept per feature, the first worker in
a swarm would have burned the feature's single warning and every sibling worker
fanned out over the same feature would have been refused — for a missing triage
record none of them can write from inside a dispatched unit of work. Scoped per
feature *and* per session, each actor still gets exactly one warning naming the
remedy, and the second claim from that same session still refuses. Two further
details fell out of the same reasoning: the counter advances only when a claim
actually succeeds, so a refused claim never silently spends the warning; and the
contention refusal was ordered ahead of this one, so the loser of a genuine race
is told it lost the race rather than that the feature is untriaged.

## The rule

- Ask who is supposed to act on the warning. Key the counter to that actor —
  plus whatever scopes the fault itself. A count with only one of the two halves
  is either toothless or collectively punishing.
- Test the guard under fan-out before shipping it. A rule that behaves correctly
  for one session and pathologically for five is not a rule that has been tested;
  concurrency is the default condition, not the exotic one.
- Advance the counter on success only. A counter bumped on the attempt lets a
  refused caller exhaust its own allowance without ever having been let through.
- Order competing refusals so the one that describes the caller's real situation
  wins. Two true refusals are not interchangeable: the loser of a race needs to
  hear about the race.
