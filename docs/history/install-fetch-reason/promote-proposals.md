promote proposal for work item "install-fetch-reason" (docs/history/install-fetch-reason/CONTEXT.md + docs/history/install-fetch-reason/plan.md) — 1 capped cell(s): ifr-1
anchor: history — docs/history/install-fetch-reason/CONTEXT.md, docs/history/install-fetch-reason/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/install-fetch-reason/delivery.md

---
type: bee.delivery
title: install-fetch-reason — delivery
description: "Delivery record proposed by bee knowledge promote for work item install-fetch-reason: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-09-07
bee:
  id: install-fetch-reason-delivery
  lifecycle: active
  required_context: [docs/history/install-fetch-reason/CONTEXT.md, docs/history/install-fetch-reason/plan.md]
  sources: [docs/history/install-fetch-reason/CONTEXT.md, docs/history/install-fetch-reason/plan.md, .bee/cells/ifr-1.json]
---

# install-fetch-reason — Delivery

## What shipped

- **ifr-1** — install.sh now names the transport error on both published-binary fallback lines and retries three times with a connect deadline; the contract test asserts both installers (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ifr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml --test installer_contracts --test installer_invocations --test instruction_laws && .bee/bin/bee dev release-manifest --check`

## Deviations

- **ifr-1** — Added a second test and a third assertion beyond plan.md — curl's multi-line stderr split the log verdict from its reason on the first driven run, so the one-line collapse is pinned as its own contract
- **ifr-1** — Added a third assertion and a second test beyond the plan: curl writes multi-line diagnostics, and the first driven run stranded the closing paren and the building-from-source verdict on their own lines — the reason had to be collapsed to one line to be readable, so the collapse is pinned too

## Provenance

Proposed by `bee knowledge promote --work install-fetch-reason` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/install-fetch-reason/CONTEXT.md`, `docs/history/install-fetch-reason/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell ifr-1 — save as docs/knowledge/patterns/install-fetch-reason-ifr-1-pitfall.md

---
type: bee.pattern
title: install-fetch-reason cell ifr-1 — pitfall candidate
description: "Pitfall candidate mined from cell ifr-1's capped trace: Added a second test and a third assertion beyond plan.md — curl's multi-line stderr split the log verdict from its reason on the first driven run, so the one-l…"
timestamp: 2026-09-07
bee:
  id: install-fetch-reason-ifr-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/ifr-1.json]
  polarity: pitfall
---

# install-fetch-reason cell ifr-1 — pitfall candidate

## What the cell did

install.sh now names the transport error on both published-binary fallback lines and retries three times with a connect deadline; the contract test asserts both installers

## Recorded evidence (verbatim from .bee/cells/ifr-1.json)

- **deviation** — Added a second test and a third assertion beyond plan.md — curl's multi-line stderr split the log verdict from its reason on the first driven run, so the one-line collapse is pinned as its own contract
- **deviation** — Added a third assertion and a second test beyond the plan: curl writes multi-line diagnostics, and the first driven run stranded the closing paren and the building-from-source verdict on their own lines — the reason had to be collapsed to one line to be readable, so the collapse is pinned too

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.