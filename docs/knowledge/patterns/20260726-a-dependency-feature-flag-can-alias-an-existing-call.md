---
type: bee.pattern
title: A dependency feature flag can re-alias an operation existing code already calls
description: "Enabling a serializer's order-preserving feature also re-aliased its map removal to a swap-with-last removal, silently reordering two files written by code the change never touched — one of them under a byte-for-byte compatibility contract. The diff was one manifest line; the affected call sites were nowhere in it."
tags: [dependency, feature-flag, byte-compatibility, blast-radius, deviation-record]
timestamp: 2026-07-26
bee:
  id: pattern-20260726-a-dependency-feature-flag-can-alias-an-existing-call
  lifecycle: active
  areas: []
  required_context: []
  decisions: []
  sources: ["rust-port (cell rust-port-15 rework, judge finding declared-deviations-justified)", docs/history/rust-port/reports/rust-port-15.md]
---

## The pattern

Enabling a dependency's feature flag can change the meaning of code that was written before the flag existed and was never touched by the change. The compiler does not warn, the type signature is identical, and nothing in the diff points at the affected lines.

The instance: a byte-compatibility port needed key order preserved on serialization, so the serializer's order-preserving feature was switched on for the crate. That feature also re-aliases the map's `remove` operation from an order-preserving removal to a swap-with-last removal — documented in the dependency's own source as perturbing the position of what used to be the last element. Two pre-existing call sites that had removed a key for years silently began reordering the maps they wrote, one of them a file whose byte-for-byte equality with the reference implementation is the whole contract.

## Why it survives review

The diff is one line in a manifest. The affected call sites are not in the diff, are not in the cell's file list, and read exactly as they always did. A deviation record written honestly at the time still said "no source was edited" — true of the letter, false of the effect.

## What to do

- When enabling a dependency feature, read the dependency's own source for what the feature re-aliases — not just what it adds. Feature flags that change a data structure's identity (ordered vs unordered, stable vs unstable) change every operation on it.
- Sweep every existing call site of the affected operations in the same change, and state the blast radius in the change record. "One additive flag" is a claim about the manifest, not about behavior.
- Prove the sweep the same way the contract is stated: if the contract is about bytes, the proof must compare bytes. See [[20260726-a-comparison-blind-to-the-contracts-own-dimension]] — the assertions guarding these very call sites stayed green through the whole regression, because they compared parsed values under a map type whose equality ignores order.
