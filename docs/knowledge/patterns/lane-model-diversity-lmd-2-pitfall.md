---
type: bee.pattern
title: A guard test that has never failed proves only that the guard agrees with itself
description: Falsify a new contract test with a temporary reverted mutation on both the refusal and admit arms before trusting it
timestamp: 2026-08-29
bee:
  id: lane-model-diversity-lmd-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/lmd-2.json]
  polarity: pitfall
---

# A guard test that has never failed proves only that the guard agrees with itself

## What happened

Cell lmd-2 pinned seat-role parity across the dispatch door and the marker
guard. Before trusting the new parity test, the cell ran two temporary
mutations of the guard's `known_role_named` wrapper — one that should trip
the refusal arm, one that should trip the admit arm — confirmed both failed
as expected, then reverted both mutations. A test that has passed since the
moment it was written, against code that has never changed, has not yet
proven it can fail.

## The lesson

A new guard/contract test earns trust only after it is shown to fail on a
deliberate, reverted mutation of both its refusal and its admit path — this
mirrors the existing "guard and its tests are one model" pattern, applied at
cell-authoring time rather than after the fact.

## Status

Candidate only. Naming the pattern, generalizing it beyond this cell, and
moving `bee.lifecycle` to `active` are a human or agent decision.
