promote proposal for work item "spawn-busy-retry" (docs/history/spawn-busy-retry/plan.md) — 1 capped cell(s): sbr-1
anchor: history — docs/history/spawn-busy-retry/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/spawn-busy-retry/delivery.md

---
type: bee.delivery
title: spawn-busy-retry — delivery
description: "Delivery record proposed by bee knowledge promote for work item spawn-busy-retry: 1 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-10-07
bee:
  id: spawn-busy-retry-delivery
  lifecycle: active
  areas: [feedback-digest]
  required_context: [docs/history/spawn-busy-retry/plan.md]
  sources: [docs/history/spawn-busy-retry/plan.md, .bee/cells/sbr-1.json]
---

# spawn-busy-retry — Delivery

## What shipped

- **sbr-1** — fsutil::retry_executable_busy owns the ExecutableFileBusy retry; doctor installed_binary_bee_version, feedback ingest_issues and report gh() use it; stress 0/120 failures (was 8/120) (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **sbr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fsutil doctor verbs::feedback verbs::report` — touched fsutil, doctor, feedback, report: 126 passed, 0 failed; plus stress 6 parallel loops x 20 runs of the test binary on verbs::feedback verbs::report: 0/120 failures; whole suite not run, CI cov…

## Deviations

- **sbr-1** — followed the plan; also removed doctor's PROBE_ETXTBSY constants and their doc comments since the helper now owns the values — less code, no new comments
- **sbr-1** — ran cargo build --release --bin bee before verify because report_verb_reads_bee_gh_bin_end_to_end needs the built binary in the shared target dir and it was missing — hit an unforeseen obstacle
- **sbr-1** — sync-ack: internal spawn retry on ExecutableFileBusy; no user-facing feedback behavior or skill text changes

## Provenance

Proposed by `bee knowledge promote --work spawn-busy-retry` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/spawn-busy-retry/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "spawn-busy-retry" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-07T11:25:38.961Z), the work item declares no bee.areas.

area feedback-digest:
  - [sbr-1] fsutil::retry_executable_busy owns the ExecutableFileBusy retry; doctor installed_binary_bee_version, feedback ingest_issues and report gh() use it; stress 0/120 failures (was 8/120) — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/sbr-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell sbr-1 — save as docs/knowledge/patterns/spawn-busy-retry-sbr-1-pitfall.md

---
type: bee.pattern
title: spawn-busy-retry cell sbr-1 — pitfall candidate
description: "Pitfall candidate mined from cell sbr-1's capped trace: followed the plan; also removed doctor's PROBE_ETXTBSY constants and their doc comments since the helper now owns the values — less code, no new comments"
timestamp: 2026-10-07
bee:
  id: spawn-busy-retry-sbr-1-pitfall
  lifecycle: draft
  areas: [feedback-digest]
  sources: [.bee/cells/sbr-1.json]
  polarity: pitfall
---

# spawn-busy-retry cell sbr-1 — pitfall candidate

## What the cell did

fsutil::retry_executable_busy owns the ExecutableFileBusy retry; doctor installed_binary_bee_version, feedback ingest_issues and report gh() use it; stress 0/120 failures (was 8/120)

## Recorded evidence (verbatim from .bee/cells/sbr-1.json)

- **deviation** — followed the plan; also removed doctor's PROBE_ETXTBSY constants and their doc comments since the helper now owns the values — less code, no new comments
- **deviation** — ran cargo build --release --bin bee before verify because report_verb_reads_bee_gh_bin_end_to_end needs the built binary in the shared target dir and it was missing — hit an unforeseen obstacle
- **deviation** — sync-ack: internal spawn retry on ExecutableFileBusy; no user-facing feedback behavior or skill text changes

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.