---
area: verify-pipeline
updated: 2026-07-22
migrated_to: docs/knowledge/areas/verify-pipeline/
---

# Verify Pipeline (migrated — pointer stub)

This area's current truth now lives in the knowledge bundle:
[`docs/knowledge/areas/verify-pipeline/`](../knowledge/areas/verify-pipeline/index.md)
(okf-foundation D20/D29/D37). Two concepts, split by TOPIC rather than the old
spec's headings: `suite-topology-and-discovery.md` owns how suites are shaped
and found — per-module suites with no monolith, the shared fixture helper,
convention-based discovery, the loud deletion guard, and Windows CI proving
the real suites through that same discovery mechanism; `concurrency-and-
hermetic-runs.md` owns how a run itself stays safe — locked atomic-swapped
regeneration, multi-worker checkout etiquette, and hermetic construction
proven by deterministic race and isolation suites. This path stays alive as a
pointer stub — it is never deleted in this feature (D20) — and the anchor map
below sends every numbered anchor the old spec exposed to the concept that now
owns it, so existing citations keep resolving. Coverage is machine-checked by
`scripts/okf_migrate.mjs --check verify-pipeline` in the verify chain (D35),
against the pinned pre-migration blob `eab70d7e` (`72fd828`, 14 anchors — 0 B /
5 R / 4 E / 5 P — 7 unparsed blocks — okf-migration-f2 F8/F9).

The 7 unparsed blocks are the source's "Behaviors & Operations" bullets — none
of them carries a numbered id, so none is invented into an anchor (D10); their
content is still carried, verbatim, into the concept whose topic it matches.

## Anchor map

| Anchor | Now owned by | Was |
|---|---|---|
| R1 | [docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md](../knowledge/areas/verify-pipeline/suite-topology-and-discovery.md) | a test suite is one file, one module/area, one temp root |
| R2 | [docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md](../knowledge/areas/verify-pipeline/suite-topology-and-discovery.md) | suite membership is discovered, never hand-registered |
| R3 | [docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md](../knowledge/areas/verify-pipeline/suite-topology-and-discovery.md) | check-count conservation is the required migration evidence |
| R4 | [docs/knowledge/areas/verify-pipeline/concurrency-and-hermetic-runs.md](../knowledge/areas/verify-pipeline/concurrency-and-hermetic-runs.md) | whole-tree regeneration must be lock-serialized and atomic-swapped |
| R5 | [docs/knowledge/areas/verify-pipeline/concurrency-and-hermetic-runs.md](../knowledge/areas/verify-pipeline/concurrency-and-hermetic-runs.md) | multi-worker checkout etiquette: cap → commit → release reservations |
| E1 | [docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md](../knowledge/areas/verify-pipeline/suite-topology-and-discovery.md) | discovery flip surfaced 4 orphan test files the hand registry never ran |
| E2 | [docs/knowledge/areas/verify-pipeline/concurrency-and-hermetic-runs.md](../knowledge/areas/verify-pipeline/concurrency-and-hermetic-runs.md) | known WSL2 host flakes under heavy concurrent load |
| E3 | [docs/knowledge/areas/verify-pipeline/concurrency-and-hermetic-runs.md](../knowledge/areas/verify-pipeline/concurrency-and-hermetic-runs.md) | the claim-race negative control's deterministic fs-barrier handshake |
| E4 | [docs/knowledge/areas/verify-pipeline/concurrency-and-hermetic-runs.md](../knowledge/areas/verify-pipeline/concurrency-and-hermetic-runs.md) | the nested-clone isolation regression |
| P1 | [docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md](../knowledge/areas/verify-pipeline/suite-topology-and-discovery.md) | `scripts/run_verify.mjs` — discovery roots, EXTRA/EXCLUDE, serial convention, pool |
| P2 | [docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md](../knowledge/areas/verify-pipeline/suite-topology-and-discovery.md) | `scripts/tests/test_verify_manifest.mjs` — floor + existence + membership guard |
| P3 | [docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md](../knowledge/areas/verify-pipeline/suite-topology-and-discovery.md) | `scripts/lib/test-fixture.mjs` — shared fixture/check-runner |
| P4 | [docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md](../knowledge/areas/verify-pipeline/suite-topology-and-discovery.md) | `packages/bee/tests/` — per-module suites (11 files) |
| P5 | [docs/knowledge/areas/verify-pipeline/concurrency-and-hermetic-runs.md](../knowledge/areas/verify-pipeline/concurrency-and-hermetic-runs.md) | `scripts/render_plugin_skill_trees.mjs`, `scripts/tests/test_render_race.mjs` — locked tmp-swap render + race proof |
