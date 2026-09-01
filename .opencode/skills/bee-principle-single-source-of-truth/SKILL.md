---
name: bee-principle-single-source-of-truth
description: "Apply when the same fact — a limit, a status list, a total, a documented rule — is about to live in two places. Give it one home and derive the rest."
---

# Single Source of Truth

Every fact gets one authoritative home. Everything else derives from it at
read time, or is regenerated from it mechanically. A cached or denormalized
copy is acceptable only when the derivation is its only writer; a copy kept in
step by parallel hand-edits is a data race with human hands in it.

When you find copies that already disagree, do not sync them. Elect the owner
and demote the rest to derivations.

**Why:** when the same fact lives in two places, one of them will be wrong,
and a reader cannot tell which one.

**Depth:** `.bee/expertise/architecture.md` § Single Source of Truth.
