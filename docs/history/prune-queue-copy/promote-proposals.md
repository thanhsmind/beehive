promote proposal for work item "prune-queue-copy" (docs/history/prune-queue-copy/CONTEXT.md + docs/history/prune-queue-copy/plan.md) — 1 capped cell(s): pqc-1
anchor: history — docs/history/prune-queue-copy/CONTEXT.md, docs/history/prune-queue-copy/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/prune-queue-copy/delivery.md

---
type: bee.delivery
title: prune-queue-copy — delivery
description: "Delivery record proposed by bee knowledge promote for work item prune-queue-copy: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-10-15
bee:
  id: prune-queue-copy-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [docs/history/prune-queue-copy/CONTEXT.md, docs/history/prune-queue-copy/plan.md]
  sources: [docs/history/prune-queue-copy/CONTEXT.md, docs/history/prune-queue-copy/plan.md, .bee/cells/pqc-1.json]
---

# prune-queue-copy — Delivery

## What shipped

- **pqc-1** — Prune keeps a worktree for its capture queue only on a pending stub that main never names, and every doubt keeps (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pqc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee worktree::` — 213 passed 0 failed, the 12 new queue tests listed by name, scope is the worktree module the change lives in, bee dev comment-baseline --check also green, full suite not run, run with PATH=HOME/.carg…

## Deviations

- **pqc-1** — ran the verify with PATH=HOME/.cargo/bin instead of the CARGO_HOME expansion — the session isolation guard refused the variable expansion — same cargo binary and same test filter

## Provenance

Proposed by `bee knowledge promote --work prune-queue-copy` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/prune-queue-copy/CONTEXT.md`, `docs/history/prune-queue-copy/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "prune-queue-copy" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-15T14:07:23.482Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [pqc-1] Prune keeps a worktree for its capture queue only on a pending stub that main never names, and every doubt keeps — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/pqc-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pqc-1 — save as docs/knowledge/patterns/prune-queue-copy-pqc-1-pitfall.md

---
type: bee.pattern
title: prune-queue-copy cell pqc-1 — pitfall candidate
description: "Pitfall candidate mined from cell pqc-1's capped trace: ran the verify with PATH=HOME/.cargo/bin instead of the CARGO_HOME expansion — the session isolation guard refused the variable expansion — same cargo binary a…"
timestamp: 2026-10-15
bee:
  id: prune-queue-copy-pqc-1-pitfall
  lifecycle: draft
  areas: [worktree-parallelism]
  sources: [.bee/cells/pqc-1.json]
  polarity: pitfall
---

# prune-queue-copy cell pqc-1 — pitfall candidate

## What the cell did

Prune keeps a worktree for its capture queue only on a pending stub that main never names, and every doubt keeps

## Recorded evidence (verbatim from .bee/cells/pqc-1.json)

- **deviation** — ran the verify with PATH=HOME/.cargo/bin instead of the CARGO_HOME expansion — the session isolation guard refused the variable expansion — same cargo binary and same test filter

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.