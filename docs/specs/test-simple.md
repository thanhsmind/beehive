---
area: test-simple
updated: 2026-08-03
migrated_to: docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md
---

# TEST-SIMPLE — one declared test command (migrated — pointer stub)

This area's current truth now lives in the knowledge bundle:
[`docs/knowledge/areas/verify-pipeline/`](../knowledge/areas/verify-pipeline/index.md).
`suite-topology-and-discovery.md` has cited this source and carried its rules
since the migration; this stub closes the loop by retiring the second copy.

**Changed since this source was written (2.1.0):** it described `commands.verify`
as "the close/merge-time chain" sitting above `commands.test`. That second
repo-wide command is RETIRED. `commands.test` is now the one declared test
command. Tests prove at the boundary: the green base check, `bee close`, `bee
worktree merge`, and CI all run it; a cap is commit-only proof and records
`tests: boundary` (test-cadence-boundary D1, decision `13ce1858`). Two commands
meant every surface had to say which door ran which, and they disagreed — the
config reference called `verify` "never a local obligation" while the green
base check told agents to run it locally before their first claim.

This path stays alive as a pointer stub — a migrated source path is never
deleted (okf-foundation D20) — so existing citations keep resolving.

## Anchor map

This source carried no numbered anchors: five numbered rules and a set of named
sections.

| Was | Now owned by |
|---|---|
| 1. Declaration (`commands.test` is the single declaration) | [suite-topology-and-discovery.md](../knowledge/areas/verify-pipeline/suite-topology-and-discovery.md) — and see the 2.1.0 note above: `commands.verify` is gone |
| 2. Deterministic runner (`bee test`, `.bee/logs/test-results.json`) | [suite-topology-and-discovery.md](../knowledge/areas/verify-pipeline/suite-topology-and-discovery.md) |
| 3-5. Cap door, close door, merge gate | [suite-topology-and-discovery.md](../knowledge/areas/verify-pipeline/suite-topology-and-discovery.md) · [returning-and-the-merge-gate.md](../knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md) |
| Deletions (proof tiers, evidence flags, debt doors, per-cell verify runs) | [suite-topology-and-discovery.md](../knowledge/areas/verify-pipeline/suite-topology-and-discovery.md) |
| The trade (per-finish runs vs deferred proof) | [suite-topology-and-discovery.md](../knowledge/areas/verify-pipeline/suite-topology-and-discovery.md) · [`docs/config-reference.md`](../config-reference.md) |
