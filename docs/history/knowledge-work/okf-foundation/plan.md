---
type: bee.plan
title: okf-foundation — Plan
description: "The seven-slice plan for the Bee OKF Profile foundation: format core, chain wiring, generators, data, consumer, startup bridge, promote v1."
tags: [okf, plan, high-risk]
timestamp: 2026-07-22
bee:
  id: okf-foundation-plan
  lifecycle: active
  required_context: [work/okf-foundation/work-item.md]
  decisions: [D2 (chain never red between cells), D22 (consumer in scope), D29, D30, D34, D35, D38 (loop closure over coverage)]
  sources: [docs/history/okf-foundation/plan.md, docs/history/okf-foundation/CONTEXT.md]
  lane: high-risk
  review_status: Approved
---

# okf-foundation — Plan

## Mode gate

Four flags counted: **data model** (bee's document model is replaced), **public contracts** (new
CLI command group + managed-file set + plugin renders change), **multi-domain** (CLI lib, verify
chain, hooks, skill templates, docs), **changes behavior an existing suite asserts**
(`ledger_parity.mjs` and `release_manifest --check` both gate the new files until refreshed) =
**high-risk**. No single-file path exists; the smallest honest unit is a slice.

## Approach

Build the format core first with the chain kept green at every slice boundary (the ledger
self-onboard rides the same cell that adds `knowledge.mjs`); generators next; then data (one
migrated area plus this feature's own work item); the consumer proven against that real data; then
the two loop-closing slices D38 ordered ahead of wide migration (startup bridge, promote v1).

## Slices

| # | Slice | Exit state |
|---|---|---|
| S1 | Format core: profile area spec, `knowledge.mjs` (parser + concept model + `check`), registry/dispatcher wiring, template mirror, test suite, ledger+manifest refresh, `docs/knowledge/` skeleton | `bee knowledge check --json` exits 0 on the skeleton; full chain green |
| S2 | Chain wiring: `knowledge check` as a chain suite | Chain green with the new suite in `EXTRA_SUITES` |
| S3 | Generators: `index` (byte-identical, `--check`) + `list` | Regenerating changes zero bytes; stale index detected |
| S4 | Data: migrate `advisor-protocol` (coverage report D35, stub + anchor map D37), `critical-patterns.md` -> `patterns/`, templates, `work/okf-foundation/` work item | Coverage report 100%/0 dup; zero dangling anchors; chain green |
| S5 | Consumer: `context --work --budget` + dogfood acceptance | `context --work okf-foundation --budget 20000` returns a manifest a cold session can act on |
| S6 | Startup bridge (D38): bee-hive/AGENTS.md step — when the active feature has a work item, run `context` and read the manifest; session preamble surfaces it | A fresh session on this repo loads okf-foundation from the manifest, ≤6 files |
| S7 | Promote v1 (D38): `knowledge promote --work <id>` proposes concept diffs from cell traces + delivery — proposals, never silent truth-writes | A capped cell's trace yields a reviewable concept diff; applying it passes `check` |

Cells are created for the current slice only (D2); S2+ cells are cut when their slice becomes
current. Slice boundaries honor every locked decision they touch; if any slice cannot, the answer
is SPLIT RECOMMENDED, never scope shrink.

## Risk map

| Component | Risk | Proof |
|---|---|---|
| Frontmatter parser (D12, no deps) | LOW-MED | Parses only what the profile emits; loud failure outside subset. |
| Ledger/manifest mid-feature (D34) | MED | okf-1's verify is the full chain — the cell cannot cap red. |
| Coverage report v1 (D35) | MED | Anchor accounting only — set equality over numbered ids. |
| `context` ranking (D27) | LOW | Manifest, not content; bytes/4 estimator declared. |
| Startup bridge (S6) | LOW | One prose step + one status field; no new runtime. |
| Promote v1 (S7) | HIGH | Semantic distillation — proposals only, never auto-writing truth. |

## Rejected Alternatives

Big-bang retrofit of `docs/` in place (593 files, three frontmatter schemas, unfixable timestamp
semantics); `workflow-state.md` as proof area (1464 lines, BA-template structure, ~700 lines
homeless under a heading split; its 9-concept map is locked in D30 for F2); report-only chain entry
(precedent was miscited; withdrawn in D22).
