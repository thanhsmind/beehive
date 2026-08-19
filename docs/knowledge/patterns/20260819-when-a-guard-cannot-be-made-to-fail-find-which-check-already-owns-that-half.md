---
type: bee.pattern
title: "When a guard cannot be made to fail, find which check already owns that half"
description: "A red-first proof that cannot go red the obvious way is information about the system, not a licence to weaken the guard — the fleet/bee crate boundary turned out to have two enforcement mechanisms, cargo's cycle check refusing any NORMAL `bee` dependency before a test body runs, and the new manifest test owning the dev, build, and target-conditional half cargo does not, so the red-first proof had to use a dev-dependency and the decision's 'only mechanism' wording is true of the boundary but false of any single check."
timestamp: 2026-08-19
bee:
  id: pattern-20260819-guard-that-cannot-fail-has-a-co-owner
  lifecycle: active
  areas: [rust-runtime]
  sources: ["capture stub 3e12df7e (herding-orchestration: red-first run found two enforcement mechanisms on the crate boundary)", packages/bee-rs/crates/fleet/tests/manifest_boundary.rs, docs/history/herding-orchestration/CONTEXT.md]
---

A red-first run set out to prove a new test guards the crate boundary:
add `bee` as a dependency of `fleet`, watch the test go red. It could
not be done the obvious way. Adding `bee` as a NORMAL dependency makes
cargo refuse the entire workspace with a cyclic-package-dependency
error before any test body runs — `bee` already depends on `fleet`,
so the cycle is structural. The test could never be exercised that
way; the red-first proof had to use a DEV-dependency, which does not
cycle and which cargo accepts happily.

That failure to fail was the finding. The boundary has TWO enforcement
mechanisms, not one: cargo's own cycle check owns normal dependencies,
and the manifest test owns dev, build, and target-conditional ones —
exactly the half cargo does not police. The decision's wording that the
crate edge is "the only mechanism" is true of the boundary but false of
any single check. One narrow gap stays open and is stated honestly: the
test matches the literal dependency name `bee`, so a dev-dependency
aliased to another name while pointing at `../bee` would pass it, and
cargo's cycle check does not close that case either.

**The rule:** when a guard cannot be made to fail the obvious way,
do not weaken the guard or fake the red — find out what is already
enforcing the case that refuses to break, and write down which check
owns which half. The proof then targets the guard's own half (here,
the dev-dependency), and the record names the co-owner so nobody later
reads one check as the whole boundary.
