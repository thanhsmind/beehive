# lane-road-in-refusals — CONTEXT

## What was asked

A live screenshot from another project: an agent tried to start new work, hit
`startFeature`'s refusal, and escalated exactly two choices to the human — park
the feature already occupying the pipeline, or finish it first. Both options
ask this agent to decide the fate of ANOTHER agent's in-flight work. The user
asked for the situation to be resolved properly rather than answered case by
case.

## What was found

The default pipeline (`.bee/state.json`) holds exactly one feature, so a second
agent on the same checkout cannot start its own feature there. `start_default`
refuses with:

> `A prior feature must finish or be explicitly wound down before a new feature
> starts. FIX: resume/close the current feature through its normal chain, or
> drop its remaining cells (bee cells drop), then retry.`

Every remedy that message names requires disturbing work that may not be the
caller's. The one remedy that does not is never mentioned: a **lane**.

Lanes already work for this. `start_lane` reads only the lane's own record — it
never reads `state.json` at all, so a lane starts cleanly while the default
pipeline is mid-flight on someone else's feature, touching none of it. The
project's own doctrine already says this is the paved road
(`skills/bee-hive/references/routing-and-contracts.md:144`):

> "the paved road is a lane, not a queue, whether or not another feature is
> already live"

So the rule is right and the machinery is right. The refusal text is what sent
the agent down the wrong road: doctrine lives in a reference file, the refusal
is in front of the agent at the moment of failure, and the refusal wins.

This is the third instance of one defect shape found in a single day. The
merge-scope fix and the concurrent-git-guard fix (`f7ab7870`) were both the
same thing: a refusal that named a heavy remedy while hiding the light one that
already worked.

## Decisions

- D1 — `start_default`'s refusal names the lane road. A caller blocked by an
  occupied default pipeline is told it can start its own work as a lane,
  with the flag spelled out, alongside the existing remedies.
- D2 — When the occupying feature belongs to ANOTHER live session, the refusal
  changes register rather than adding a line: it states plainly that the
  feature is a live peer's and must not be wound down by this caller, and names
  the lane as the way forward. An agent is never invited to close work that is
  not its own.
- D3 — "Another live session" reuses the predicate already settled one feature
  earlier (`default-pipeline-liveness` D1): a session record that is alive by
  heartbeat and bound to no lane is, by definition, on the default pipeline.
  The existing helper is reused or deliberately relocated to a shared home —
  never silently reimplemented, because two copies of a liveness rule is the
  next drift.
- D4 — The caller's own session never reads as its peer. A session resuming the
  feature it started itself still gets the original wording; only a genuine
  peer changes it.
- D5 — Refusal only. This cell does not auto-route the caller into a lane and
  does not relax the default-pipeline guard. Naming the road is the fix; taking
  it stays the caller's move.

## Out of scope

- Auto-routing a blocked start into a lane without asking. Tempting, and
  deliberately not taken: it would start a feature the caller never asked to
  start as a lane, and lane starts carry `--paths` the caller must declare.
- The skills' own wording. The doctrine already says the right thing; the gap
  was the machine's message, not the document.
