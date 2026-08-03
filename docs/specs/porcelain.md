---
area: porcelain
updated: 2026-08-03
migrated_to: docs/knowledge/areas/rust-runtime/command-surface.md
---

# Porcelain surface — the flow verbs (migrated — pointer stub)

This area's current truth now lives in the knowledge bundle, as a concept of the
compiled runtime rather than an area of its own:
[`docs/knowledge/areas/rust-runtime/command-surface.md`](../knowledge/areas/rust-runtime/command-surface.md).
The surface is what the binary presents, so it belongs beside the binary's other
guarantees.

Two things in the old source were already stale when it was migrated, and the
concept carries the corrected form:

- Its verb table used the PLUMBING spellings (`bee state route`, `bee state
  gate`, `bee cells finish`) for verbs that have since grown flow aliases. The
  repo's own maintainer checklist says to write the flow spelling where one
  exists, so the table taught against the convention it was describing.
- The per-verb narrative sections for `orient`, `cells finish`, `dispatch
  prepare --claim` and `close` were command documentation living beside the
  presentation contract. Per-command contracts are generated —
  `bee <verb> --help`, `bee --help --all --json` — and a hand-copied second copy
  is a copy that drifts. The concept keeps the RULES; the registry keeps the
  contracts.

This path stays alive as a pointer stub — a migrated source path is never
deleted (okf-foundation D20) — so existing citations keep resolving.

## Anchor map

This source carried no numbered anchors: it is prose under named headings. The
headings map as follows.

| Was | Now owned by |
|---|---|
| Porcelain set (v1, 16 verbs) | [command-surface.md](../knowledge/areas/rust-runtime/command-surface.md) — Data Dictionary and R1/R2 |
| Help behavior | [command-surface.md](../knowledge/areas/rust-runtime/command-surface.md) — Entry Points, R3/R4 |
| teach-at-point-of-contact contract | [command-surface.md](../knowledge/areas/rust-runtime/command-surface.md) — Behaviors |
| New verb: `bee orient` | `bee orient --help` (contract) · [command-surface.md](../knowledge/areas/rust-runtime/command-surface.md) Edge Cases (the never-compute-state-twice rule) |
| New verb: `bee cells finish` | `bee finish --help` (contract) · [areas/verify-pipeline/](../knowledge/areas/verify-pipeline/index.md) (the test door) |
| Extended verb: `bee dispatch prepare --claim` | `bee dispatch prepare --help` (contract) |
| New verb: `bee close` | `bee close --help` (contract) · [areas/verify-pipeline/](../knowledge/areas/verify-pipeline/index.md) (the close doors) |
| Compatibility | [command-surface.md](../knowledge/areas/rust-runtime/command-surface.md) — R1 |
