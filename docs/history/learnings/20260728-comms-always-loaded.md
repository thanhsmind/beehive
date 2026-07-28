---
date: 2026-07-28
feature: comms-always-loaded
categories: [communication, doctrine-layer]
severity: high
tags: [always-loaded, thin-body-exception, dead-rule]
---

# comms-always-loaded — feature close learnings

## What Happened

The user reported that bee had once confirmed shipping an ADHD-shaped output
style, yet nothing about the conversation ever changed. Investigation: the
Communication contract existed, was well written, and had been dead since the
day it was authored (2026-07-26). It lived in
`bee-hive/references/routing-and-contracts.md`, a file loaded only when the
model chooses to open it — and `rg 'Communication contract' skills/` returned
exactly one hit: the reference itself. No body, no skill, nothing pointed at
it. The always-loaded layer carried a single compressed line ("work language,
purpose first") that captured none of the shape.

Fix: a 14-line `## Communication` section now sits in `AGENTS.md` and its
template (byte-identical, 15.4KB against an 18KB warn / 20KB hard budget),
usable as a checklist while writing a message; `bee-writing-skills` gained the
standing exception that per-turn rules are never exiled; and a census check
pins the turn shape and pre-send check to both files with mutation negative
controls.

## Root Cause

The thin-body doctrine (shipped this morning) is correct for domain law —
detail earns its cost only when a branch is hit. It is **wrong for per-turn
law**: chat shape applies to every turn, so lazy loading means never loading.
The doctrine had no exception category, so a communication contract was filed
like any other protocol and vanished.

## Recommendation

- **A rule's home is decided by its trigger frequency, not its length.** If it
  applies to every turn, it belongs where every turn can see it. If it applies
  to a branch, it belongs behind that branch. Ask "when does this fire?" before
  "how long is this?"
- **A rule with no inbound pointer is dead on arrival.** When authoring into a
  reference, the same change must add the line that forces it open — or a
  guard that proves it is reachable. Grep for inbound references as part of
  authoring, not as an audit later.
- **"Done" claimed for instruction text needs a reachability check**, the same
  way code needs a test. This one shipped, passed every fence, and did nothing.

## Ran under

R82 (worker ran no suites, capped pending; main verified once — green first
pass), R83 (one sync, one compounding at close), R84 (step ticks throughout).
