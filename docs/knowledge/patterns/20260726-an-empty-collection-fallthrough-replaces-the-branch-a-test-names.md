---
type: bee.pattern
title: "An empty-collection fall-through silently replaces the branch a test's name claims"
description: "A test named for a synchronization step's authoritative rebuild seeded no work records, so every case took the no-records shortcut; the assertions were real, the oracle was real, and the branch the cell existed to port was never executed. The removal proof — neutralize the seeding and require the test to fail — is what separates a named branch from an exercised one."
tags: [proof-discipline, coverage, fixtures, branch-coverage, table-driven-tests]
timestamp: 2026-07-26
bee:
  id: pattern-20260726-an-empty-collection-fallthrough-replaces-the-branch-a-test-names
  lifecycle: active
  areas: []
  required_context: []
  decisions: []
  sources: [rust-port (cell rust-port-17 goal-check finding state-sync-side-effect-parity; cell rust-port-21 worker-ladder observation), docs/history/rust-port/reports/rust-port-21.md]
---

## The pattern

A test's name is not evidence that its fixture reaches the branch the name describes. The commonest way this fails is an empty-collection fall-through: the code checks whether a collection has entries, and the fixture — which seeds none — quietly takes the do-nothing path while the test asserts an outcome that both paths happen to satisfy.

The instance: a synchronization step rebuilds projected state from active work records. Its authoritative rebuild branch is the reason the step exists. No fixture ever seeded a work record, so every case fell through the "no records" shortcut, and a test named for the rebuild proved only that the shortcut is silent. It passed for as long as it existed.

## Why review does not catch it

The assertions are real, the oracle is real, the comparison is byte-level. Everything about the test is honest except which code it runs. Reading the test tells you what it checks; only reading the branch condition next to the fixture's contents tells you where it lands.

## What to do

- For every branch a cell exists to port or change, name the fixture that reaches it and state why it reaches it — the condition, evaluated against the fixture's actual contents. "There is a test called X" is not that statement.
- Seed through the real writer, not by hand-authoring the record. A hand-built record that is subtly wrong lands in the fall-through just as silently.
- Prove it by removal: neutralize the seeding step and require the test to fail. A test that stays green without its fixture was never testing the branch. Apply the same removal proof to table-driven cases — five rows do not prove five paths unless each row's distinguishing field is the deciding one, and a row set where no case carries two competing fields cannot detect a priority-order regression at all.
