---
type: bee.pattern
title: A verify command that is a binary invocation is unowned by CI and rots silently
description: "One cell's verify was a parity binary run; the scheduled build ran only the compiler and the test suite, so nothing re-executed it. A later cell grew the shared fixture, the binary's safety check refused the new shape, and that verify stayed red across an entire slice while build, suite and branch status all stayed green."
tags: [verify, ci-ownership, shared-fixture, silent-red]
timestamp: 2026-07-26
bee:
  id: pattern-20260726-a-binary-invocation-verify-is-unowned-by-ci
  lifecycle: active
  areas: []
  required_context: []
  decisions: []
  sources: ["rust-port (cell rust-port-15 discovered rust-port-4's verify red since rust-port-19)", docs/history/rust-port/reports/rust-port-15.md]
---

## The pattern

A cell's verify command is the proof that cell shipped. When that command is a binary invocation rather than a test the suite collects, no scheduled run ever executes it again. It can rot the moment an unrelated cell changes what it reads, and nothing reports it.

The instance: one cell's verify was a parity binary run. Continuous integration for that workspace ran a release build and the test suite — never that binary. A later cell grew a real repository into the shared fixture; the binary's safety check refused any repository marker at the fixture root and had been failing since. The verify stayed red across an entire slice while every visible signal — the suite, the build, the branch status — stayed green. It surfaced only because a much later cell happened to run the binary by hand.

## Why the usual defenses miss it

Test collection is what makes a check recurring. A binary invocation is a one-time act performed by whoever typed it, at the moment they typed it. The cell record still says "verified" — truthfully, about a past moment — and the sentence reads identically whether the command still passes today or has been failing for weeks.

## What to do

- Every verify command that is not collected by the test runner needs a home that re-runs it: an integration test that shells the binary, or an explicit entry in whatever the scheduled run executes. Otherwise its guarantee has an expiry date nobody wrote down.
- Shared fixtures are the usual trigger: when one cell grows the fixture another cell's proof reads, the second proof is a dependent of the first. Treat fixture generators as having consumers, and check them in the same change.
- A safety check that refuses a whole class should distinguish the forms inside the class. Here the check refused any repository marker, when what it needed to refuse was the marker file that points outside the fixture — the directory form was exactly the thing the fixture now legitimately contained.
