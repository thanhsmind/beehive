---
type: bee.pattern
title: A red that cannot compile proves nothing
description: A red that cannot compile proves nothing
tags: [testing, red-first, evidence, proof]
timestamp: 2026-08-28
bee:
  id: pattern-20260828-a-red-that-cannot-compile-proves-nothing
  lifecycle: active
  areas: [workflow-state, verify-pipeline]
  sources: [".bee/cells/archive/merge-door-precision/mdp-1.json", ".bee/cells/archive/slp-dissent-stop-and-ask/sd-4.json"]
  polarity: pitfall
  critical: false
  evidence: present
---

# A Red That Cannot Compile Proves Nothing

A red-first test written against a helper that does not exist yet fails as a
build error, not as a behavior gap. That red proves only that the code is
absent; it can never show the test would catch the defect once the code lands.

Two deliveries took the honest route independently, two days apart. One wrote
its refusal cases against the tool the product wraps, so the failing run
exercised the observed behavior rather than a missing symbol. The other could
not compile its cases before the new arm existed, so it reproduced the
pre-change state after the fact, ran every new case to a recorded failure, then
restored the change and ran green.

The rule: a red counts as evidence only when the failing run exercises the
behavior under test. Reach for one of the two honest routes — write the case
against a surface that already exists, or land the change, then re-create the
old state and record the failure it produces.
