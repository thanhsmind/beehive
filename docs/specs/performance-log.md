---
area: performance-log
updated: 2026-07-22
migrated_to: docs/knowledge/areas/performance-log/
---

# Performance Log (migrated — pointer stub)

This area's current truth now lives in the knowledge bundle:
[`docs/knowledge/areas/performance-log/`](../knowledge/areas/performance-log/index.md)
(okf-foundation D20/D29/D37). Three concepts, split by TOPIC rather than the
old spec's headings: `sections-lifecycle-and-measurement.md` owns the
operator-driven lifecycle of a named section — opening, closing, one-shot
recording, reading/rendering, the section data dictionary, and the
measurement rules that make its numbers trustworthy;
`persistent-store-and-sync.md` owns the one shared, append-only, per-machine
store every section lands in and the automatic sync mechanism that populates
it from real session activity; `cross-project-matrix.md` owns the read-only
cross-project rollup view built from that same store. This path stays alive
as a pointer stub — it is never deleted in this feature (D20) — and the
anchor map below sends every numbered anchor the old spec exposed to the
concept that now owns it, so existing citations keep resolving. Coverage is
machine-checked by `scripts/okf_migrate.mjs --check performance-log` in the
verify chain (D35), against the pinned pre-migration blob `efdc9f2` (`46a56a4`,
23 anchors — 0 B / 11 R / 5 E / 7 P — 10 unparsed blocks —
okf-migration-f2 F8/F9).

The 10 unparsed blocks are the source's entire "Behaviors & Operations"
section — 7 bold-lead paragraphs (Opening/Closing/One-shot/Reading-rendering/
Populating the store/Building the matrix/Measurement rules) plus 3 of
Measurement rules' own un-ided sub-bullets — none of them carries a numbered
id, so none is invented into an anchor (D10); their content is still carried,
verbatim, into the concept whose topic it matches.

## Anchor map

| Anchor | Now owned by | Was |
|---|---|---|
| R1 | [docs/knowledge/areas/performance-log/sections-lifecycle-and-measurement.md](../knowledge/areas/performance-log/sections-lifecycle-and-measurement.md) | count each request once |
| R2 | [docs/knowledge/areas/performance-log/sections-lifecycle-and-measurement.md](../knowledge/areas/performance-log/sections-lifecycle-and-measurement.md) | exclude non-model events |
| R3 | [docs/knowledge/areas/performance-log/sections-lifecycle-and-measurement.md](../knowledge/areas/performance-log/sections-lifecycle-and-measurement.md) | running time is active time, never "alive" time |
| R4 | [docs/knowledge/areas/performance-log/sections-lifecycle-and-measurement.md](../knowledge/areas/performance-log/sections-lifecycle-and-measurement.md) | parallel means genuinely concurrent helpers |
| R5 | [docs/knowledge/areas/performance-log/sections-lifecycle-and-measurement.md](../knowledge/areas/performance-log/sections-lifecycle-and-measurement.md) | helper cost is attributed, not hidden |
| R6 | [docs/knowledge/areas/performance-log/persistent-store-and-sync.md](../knowledge/areas/performance-log/persistent-store-and-sync.md) | one shared location, per-machine, project-tagged |
| R7 | [docs/knowledge/areas/performance-log/persistent-store-and-sync.md](../knowledge/areas/performance-log/persistent-store-and-sync.md) | the store is append-only and machine-readable; reports are rendered on demand |
| R8 | [docs/knowledge/areas/performance-log/sections-lifecycle-and-measurement.md](../knowledge/areas/performance-log/sections-lifecycle-and-measurement.md) | a missing measurement never fails the operation |
| R9 | [docs/knowledge/areas/performance-log/persistent-store-and-sync.md](../knowledge/areas/performance-log/persistent-store-and-sync.md) | the persistent log is the source of truth; the view only reads it |
| R11 | [docs/knowledge/areas/performance-log/cross-project-matrix.md](../knowledge/areas/performance-log/cross-project-matrix.md) | projects are grouped by their last folder name |
| R10 | [docs/knowledge/areas/performance-log/persistent-store-and-sync.md](../knowledge/areas/performance-log/persistent-store-and-sync.md) | the automatic refresh is best-effort and never blocks a session's end |
| E1 | [docs/knowledge/areas/performance-log/sections-lifecycle-and-measurement.md](../knowledge/areas/performance-log/sections-lifecycle-and-measurement.md) | boundary events |
| E2 | [docs/knowledge/areas/performance-log/sections-lifecycle-and-measurement.md](../knowledge/areas/performance-log/sections-lifecycle-and-measurement.md) | no open section at close |
| E3 | [docs/knowledge/areas/performance-log/sections-lifecycle-and-measurement.md](../knowledge/areas/performance-log/sections-lifecycle-and-measurement.md) | unparseable trailing window |
| E4 | [docs/knowledge/areas/performance-log/persistent-store-and-sync.md](../knowledge/areas/performance-log/persistent-store-and-sync.md) | empty / missing log |
| E5 | [docs/knowledge/areas/performance-log/sections-lifecycle-and-measurement.md](../knowledge/areas/performance-log/sections-lifecycle-and-measurement.md) | no resolvable session at open |
| P1 | [docs/knowledge/areas/performance-log/persistent-store-and-sync.md](../knowledge/areas/performance-log/persistent-store-and-sync.md) | aggregation core, per-session rollup + cross-project scan, persistent store, matrix build-from-log, HTML, section schema — `packages/bee/lib/perf.mjs` |
| P2 | [docs/knowledge/areas/performance-log/sections-lifecycle-and-measurement.md](../knowledge/areas/performance-log/sections-lifecycle-and-measurement.md) | CLI surface `bee perf start\|stop\|section\|log\|render\|report\|sync` |
| P3 | [docs/knowledge/areas/performance-log/persistent-store-and-sync.md](../knowledge/areas/performance-log/persistent-store-and-sync.md) | automatic write + refresh at session close: `maybePerfRefresh` |
| P4 | [docs/knowledge/areas/performance-log/sections-lifecycle-and-measurement.md](../knowledge/areas/performance-log/sections-lifecycle-and-measurement.md) | open-section marker: `.bee/perf-open.json` |
| P5 | [docs/knowledge/areas/performance-log/persistent-store-and-sync.md](../knowledge/areas/performance-log/persistent-store-and-sync.md) | global store: `performance.jsonl`, `performance.html`, `scan-cache.json` |
| P6 | [docs/knowledge/areas/performance-log/sections-lifecycle-and-measurement.md](../knowledge/areas/performance-log/sections-lifecycle-and-measurement.md) | data source: Claude Code session transcripts |
| P7 | [docs/knowledge/areas/performance-log/cross-project-matrix.md](../knowledge/areas/performance-log/cross-project-matrix.md) | tests: `test_perf.mjs`, the perf blocks in `test_bee_cli.mjs` |
