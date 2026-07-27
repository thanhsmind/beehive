---
type: bee.pattern
title: A harness that inherits ambient environment shrinks its own proof without ever going red
description: "A parity harness inherited the launching shell's session identifier, which resolved to a session absent from the fixture, so the branch that serializes a full lane record never executed on any developer machine — and a real key-order break lived there. Both legs read the same environment, so correctness was never at risk; only the coverage silently shrank."
tags: [proof-discipline, harness, environment, coverage, parity]
timestamp: 2026-07-26
bee:
  id: pattern-20260726-a-harness-that-inherits-ambient-env-shrinks-its-proof
  lifecycle: active
  areas: []
  required_context: []
  decisions: []
  sources: ["rust-port (cell rust-port-15 rework, harness-determinism deviation)", docs/history/rust-port/reports/rust-port-15.md]
---

## The pattern

A harness that reads the developer's ambient environment can quietly stop exercising part of what it claims to prove. Nothing turns red, because both sides of the comparison read the same environment — correctness is never at risk. What shrinks is the proof.

The instance: a parity harness ran two implementations over the same fixture and diffed their output. It inherited the session identifier from the shell that launched it. That identifier resolves to a session with no record inside the fixture, so the code path that serializes a full lane record — reachable only when an active session resolves — never executed on any developer machine. A real key-order break lived in that unexecuted path and shipped green.

## Why it is worse than a flaky test

A flake announces itself. This announces nothing: the run is deterministic, fast, and green, and the coverage loss is invisible in the output. The harness reports what it compared, not what it failed to reach.

## What to do

- A harness clears the ambient environment it depends on and pins its own values per scenario. Both legs must receive identical environments — parity bought by feeding two implementations different inputs is not parity.
- Pin, then assert the pin worked: require a positive marker in the compared output proving the branch was reached (a specific key present, a specific record shape emitted). Without that assertion the branch can silently stop being exercised again tomorrow.
- Treat "which branches did this run actually enter?" as a first-class question about any proof harness. Byte counts of compared output are a cheap proxy: when a new scenario is added and the compared payload does not grow, the scenario probably reached nothing new.
- Ambient inputs that remain (home directory, tool config paths) should be listed explicitly with a stated reason they cannot shrink the proof — symmetric across legs and pinned by fixture content — rather than left unexamined.
