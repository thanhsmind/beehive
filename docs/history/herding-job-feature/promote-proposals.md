promote proposal for work item "herding-job-feature" (docs/history/herding-job-feature/plan.md) — 1 capped cell(s): hjf-1
anchor: history — docs/history/herding-job-feature/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-job-feature/delivery.md

---
type: bee.delivery
title: herding-job-feature — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-job-feature: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-11-07
bee:
  id: herding-job-feature-delivery
  lifecycle: active
  areas: [workflow-state, bee-herding]
  required_context: [docs/history/herding-job-feature/plan.md]
  sources: [docs/history/herding-job-feature/plan.md, .bee/cells/hjf-1.json]
---

# herding-job-feature — Delivery

## What shipped

- **hjf-1** — herding run stamps job.json feature (flag, cell, bound lane); prepare adds --feature to every herding run command; activity hook and bee close read it first; commit 72b7b66fce87ba70b4492be24932cf7bbf051f2e (8 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hjf-1** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml — green:unit — full suite, 42 targets, 5247 passed, 0 failed, 29 ignored (8…

## Deviations

- **hjf-1** — sync-ack: skills/bee-herding covers cockpit roles only; the job feature stamp is recorded in the close-usage-record concept and the herding run help text

## Provenance

Proposed by `bee knowledge promote --work herding-job-feature` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-job-feature/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-job-feature" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-07T02:26:36.015Z), the work item declares no bee.areas.

area workflow-state:
  - [hjf-1] herding run stamps job.json feature (flag, cell, bound lane); prepare adds --feature to every herding run command; activity hook and bee close read it first; commit 72b7b66fce87ba70b4492be24932cf7bbf051f2e — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/hjf-1.json)

area bee-herding:
  - [hjf-1] herding run stamps job.json feature (flag, cell, bound lane); prepare adds --feature to every herding run command; activity hook and bee close read it first; commit 72b7b66fce87ba70b4492be24932cf7bbf051f2e — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/hjf-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell hjf-1 — save as docs/knowledge/patterns/herding-job-feature-hjf-1-pitfall.md

---
type: bee.pattern
title: herding-job-feature cell hjf-1 — pitfall candidate
description: "Pitfall candidate mined from cell hjf-1's capped trace: sync-ack: skills/bee-herding covers cockpit roles only; the job feature stamp is recorded in the close-usage-record concept and the herding run help text"
timestamp: 2026-11-07
bee:
  id: herding-job-feature-hjf-1-pitfall
  lifecycle: draft
  areas: [workflow-state, bee-herding]
  sources: [.bee/cells/hjf-1.json]
  polarity: pitfall
---

# herding-job-feature cell hjf-1 — pitfall candidate

## What the cell did

herding run stamps job.json feature (flag, cell, bound lane); prepare adds --feature to every herding run command; activity hook and bee close read it first; commit 72b7b66fce87ba70b4492be24932cf7bbf051f2e

## Recorded evidence (verbatim from .bee/cells/hjf-1.json)

- **deviation** — sync-ack: skills/bee-herding covers cockpit roles only; the job feature stamp is recorded in the close-usage-record concept and the herding run help text

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 1 pattern candidate(s), 0 file(s) written.