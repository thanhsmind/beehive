# Knowledge Usable — Context

**Feature slug:** knowledge-usable
**Date:** 2026-08-10
**Shaping session:** complete (owner directive: "làm toàn bộ" over the reviewed knowledge-in-flow backlog; core goal stated by the owner: knowledge that gets written must be usable and used)
**Scope:** Deep
**Domain types:** CALL | READ | RUN | ORGANIZE

## Feature Boundary

One umbrella feature closing the remaining knowledge-in-flow backlog: the eight open PBIs land as nine cells, each cell citing its PBI. Delivery makes recorded knowledge reach the reader (always-loaded pull line, anchored digest, honest critical label), stay trustworthy (dangling-pointer check, flush pressure, promote convergence), prove itself (close-time pattern check, recurrence measurement), and extend beyond this repo (host-repo bundle bootstrap). Named deviation, logged: one feature instead of eight — eight separate shape/gate/merge chains would spend more on ceremony than on work; cells stay small and independently capped.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| U1 | (PBI p-c7db35b1) The session preamble names the pull move in ONE line every session — spelling `bee knowledge search --text "<symptom>"`, shown whenever a bundle exists; no flag documentation beyond that line | Pattern 20260713: an order promoted to the always-loaded layer must carry its transport |
| U2 | (PBI p-10e22a70) `knowledge check` gains a dangling-path finding: any repo-relative path token named in frontmatter `sources:` entries that does not resolve to an existing file is reported (concept path + missing target). URLs, prose, and non-path strings are never flagged. Default severity: warning; `--strict` makes it failing | The ks-2 BLOCKED class: pointers rot when files move |
| U3 | (PBI p-f893dcba) Capture-queue pressure: past a configurable threshold (default: 5 stubs OR oldest stub >7 days) the session-close nudge and the close door escalate wording to overdue/blocking-adjacent; never a hard block. Config key `capture_queue_threshold` `{count, days}` in `.bee/config.json`, validated | Stubs are durability; only flush turns them into knowledge |
| U4 | (PBI p-0a0fda78) Promote convergence: `bee close`'s knowledge-promote step ALSO appends one capture-queue stub per generated proposal (pointing at the proposal file). The proposal file keeps being written (audit trail). The preamble's unapplied-proposals block shrinks to one line naming count + newest path | The 22 dead files prove the standalone channel is write-only; the queue is the living path |
| U5 | (PBI p-7037485e) Anchor coverage: a bound feature with a `.bee/lanes/<feature>.json` record or ANY `docs/history/<feature>/` file resolves an anchor (ledger arm widened to accept the lane record alone); digest recency fallback becomes the exception. The fallback header keeps naming its reason | The digest this very session fell back on recency for a bound feature |
| U6 | (PBI p-355d4740) Critical label diet: `bee.critical` earns a written bar in the OKF profile area (recurrence-prone + cross-feature + costly-when-missed); existing patterns re-graded against it to at most ~30 critical; index regenerated. Re-grade is per-pattern judgment recorded in the commit, never a bulk strip | 85/101 critical = no filter; the ranker needs a selective pool |
| U7 | (PBI p-21583c96) Close-time pattern check: `bee close` (and `--dry-run`) gains a report-only door listing the critical patterns of the areas the feature's cells touched, each demanding a verdict word (violated/respected/not-applicable) from the closing agent; a recorded `violated` blocks close like a red test, naming the pattern. Verdicts ride the existing close/trace records — no new artifact files | Proof-of-read starts at close, where evidence already gathers |
| U8 | (PBI p-47d864b5) Recurrence measurement: each critical pattern may carry a `bee.signature` (grep-able incident signature); `bee knowledge report` counts, per pattern, decision-log and capture-queue entries matching its signature after the pattern's own date and renders per-pattern recurrence count + last-seen. Read-only report; no automatic writes | The KPI is repeat incidents; without a number, effectiveness stays faith |
| U9 | (PBI p-d494b04b) `bee knowledge bootstrap` stands up `docs/knowledge/` in a host repo: one area per existing `docs/specs/*.md` (spec body imported, OKF frontmatter added), `index.md` + subdir indexes generated, `bundle_mode` flips true, refusal (typed) when a bundle already exists. No code scanning in v1 — specs-only import, gaps named in the output | The whole machine is worthless to host repos while bundles exist only here |

### Agent's Discretion

Exact wording of preamble/nudge lines; finding codes and JSON shapes mirroring existing conventions; signature format for U8 (must be deterministic grep, not fuzzy); how U6's bar text reads — bounded by the ≤~30 target and per-pattern judgment.

## Existing Code Context

### Reusable Assets
- `packages/bee-rs/crates/bee/src/hooks/session_preamble/{budget.rs,render.rs}` — digest, promote lines, project map (U1, U4, U5)
- `packages/bee-rs/crates/bee/src/verbs/knowledge/{check.rs,anchor.rs,search.rs,routing.rs}` — U2, U5, U8, U9 homes
- `packages/bee-rs/crates/bee/src/hooks/session_close/nudges.rs` — capture-queue nudge (U3)
- `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs` — close doors, promote step (U3, U4, U7)
- `packages/bee-rs/crates/bee/src/generated/registry_payload.json` — hand-maintained registry (new verbs U8, U9)

### Integration Points
- Registry contract tests (`tests/registry_contracts.rs`) pin payload shape
- `docs/knowledge/index.md` regenerates via `bee knowledge index`

## Outstanding Questions

### Deferred To Planning
- [ ] U7 verdict transport: exact field on the close record — worker picks the smallest slot the close/trace schema already carries
- [ ] U8 signature backfill: how many existing criticals get signatures in this feature (minimum: the 3 provably-recurred ones from session evidence)

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable (U1-U9, each citing its PBI). Cells map 1:1 to U-decisions except U5+U6 which may share the ranker-pool cell pair. Planning reads locked decisions and code context; reviewing uses them for coverage.
