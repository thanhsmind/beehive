---
type: bee.pattern
title: A coverage gate derives its ground truth; it never compares two hand-authored lists
description: "A gate that checks a hand-maintained inventory against hand-authored claims proves internal consistency, not coverage — and it drifts green."
tags: [verification, coverage, migration, gates]
timestamp: 2026-07-22
bee:
  id: pattern-20260722-coverage-gate-derives-ground-truth
  lifecycle: active
  areas: [okf-profile]
  sources: [docs/history/learnings/20260722-okf-foundation.md, "removed 2026-08-16: scripts/okf_migrate.mjs no longer exists on disk (the okf-foundation coverage-gate script this pattern diagnoses, retired with the JS→Rust migration) — the defect described is historical; bee knowledge check (packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs) is the current coverage gate for this bundle", docs/history/okf-foundation/reports/advisor-digest-s1.md]
  polarity: pitfall
  critical: true
---

# A coverage gate derives its ground truth; it never compares two hand-authored lists

A gate that proves "everything in source X reached destination Y" is only as honest as where its
**ground truth** comes from. `okf_migrate --check` asserts set-equality between a frozen
`ANCHOR_REGISTRY` constant, the pointer stub's anchor map, and each concept's `bee.sources` claims.
Two of those three are hand-authored, and the registry has no cryptographic tie to the
pre-migration blob — so an editor who shrinks the registry and the concepts together keeps the gate
green while coverage silently shrinks.

The gate as shipped is faithful (26/26 for `advisor-protocol`, 47/47 for the critical patterns,
both verified against a live `--inventory` of the pre-migration source). The defect is structural,
not present: it proves **internal self-consistency**, and only accidentally proves coverage.

## The rule

When a coverage gate claims everything in source X reached destination Y, derive the ground-truth
set by **mechanically re-scanning X at check time**, or by diffing against a git-pinned reference
blob — never from a hand-authored parallel list.

**Why this matters more than it looks:** a gate believed to be inviolable is more dangerous than no
gate, because people stop checking what it covers. If a gate cannot derive its own ground truth,
say so **in the spec that documents it** (this one is stated as a known limit in the profile's B8),
so nobody mistakes self-consistency for proof.

**Generalization:** any invariant asserted between an artifact and its derivative should recompute
the artifact's side at check time. This is the same discipline that makes a generated index
trustworthy — `index --check` re-renders in memory and byte-compares rather than trusting a stored
hash — and the same one the managed-file ledger violates when a release re-points a tag without
re-onboarding.
