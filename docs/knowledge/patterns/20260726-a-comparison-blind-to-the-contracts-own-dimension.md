---
type: bee.pattern
title: "A comparison blind to the contract's own dimension passes the regression it exists to catch"
description: "Assertions compared parsed values under a map type whose equality ignores key order, while the contract was byte-for-byte file equality. Two real key-order breaks lived in that blind spot simultaneously and every assertion stayed green; the fix was byte comparison plus a permanent meta-test asserting both that parsed equality is order-blind on this build and that the byte comparator flags the same pair."
tags: [proof-discipline, byte-compatibility, meta-test, assertion-quality]
timestamp: 2026-07-26
bee:
  id: pattern-20260726-a-comparison-blind-to-the-contracts-own-dimension
  lifecycle: active
  areas: []
  required_context: []
  decisions: []
  sources: ["rust-port (cells rust-port-15, rust-port-17, rust-port-21 and their goal-check verdicts)", docs/history/rust-port/reports/rust-port-17.md]
---

## The pattern

A test can compare two things faithfully along a dimension the contract does not care about, while being structurally incapable of seeing the dimension the contract is entirely about. It passes forever, including through the exact regression it was written to catch.

The instance: the contract is byte-for-byte equality of written store files against a frozen reference implementation. The assertions compared parsed values. Under the serializer's order-preserving feature, a parsed object's equality is set-like — it ignores key order by design. So a change that reordered keys in a file whose key order IS the contract left every assertion green. Two independent real breaks lived in that blind spot at the same time.

## How it was caught, and how to keep it caught

Not by reading the assertions — they looked right. It surfaced when a reviewer asked what the comparison could see, then demonstrated it: two texts differing only in key order parse equal, and the byte comparator flags them.

That demonstration became a permanent test in the suite. It asserts both halves — that parsed equality really is order-blind on this build, and that the byte comparator really does flag the same pair — with a loud message if the premise ever stops holding. A meta-test of this shape is the only thing that keeps a proof honest when the blind spot is a property of a dependency rather than of the code under test.

## What to do

- State the contract's dimension out loud (bytes, order, timing, identity), then ask of every assertion: can this observe that dimension at all? An assertion that cannot is decoration, whatever it asserts.
- When a comparison must tolerate volatile content, redact in the raw text and compare bytes — never parse to normalize. Parsing discards exactly what a byte contract is made of. A redactor that fails to match its target must fail closed (leaving the volatile values in place, so the comparison diverges), never silently skip.
- Beware the same trap in messages: an assertion whose failure text claims "byte-identical" while comparing parsed values teaches the next reader something false.

Related: [[20260726-a-dependency-feature-flag-can-alias-an-existing-call]] is the change this blindness hid, and [[20260726-a-harness-that-inherits-ambient-env-shrinks-its-proof]] is the second reason the same regression went unseen.
