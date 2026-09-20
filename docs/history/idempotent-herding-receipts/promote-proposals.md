promote proposal for work item "idempotent-herding-receipts" (docs/history/idempotent-herding-receipts/CONTEXT.md + docs/history/idempotent-herding-receipts/plan.md) — 2 capped cell(s): ihr-1, ihr-2
anchor: history — docs/history/idempotent-herding-receipts/CONTEXT.md, docs/history/idempotent-herding-receipts/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/idempotent-herding-receipts/delivery.md

---
type: bee.delivery
title: idempotent-herding-receipts — delivery
description: "Delivery record proposed by bee knowledge promote for work item idempotent-herding-receipts: 2 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-09-20
bee:
  id: idempotent-herding-receipts-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/idempotent-herding-receipts/CONTEXT.md, docs/history/idempotent-herding-receipts/plan.md]
  sources: [docs/history/idempotent-herding-receipts/CONTEXT.md, docs/history/idempotent-herding-receipts/plan.md, .bee/cells/ihr-1.json, .bee/cells/ihr-2.json]
---

# idempotent-herding-receipts — Delivery

## What shipped

- **ihr-1** — The Pi drain names the round in every result injection; a later round is no longer read as a replay (3 file(s) changed)
- **ihr-2** — bee herding run returns the stored receipt on a replay and refuses a reused job id carrying different work (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ihr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check` — 36 suites 0 failed; pi_plugin_contracts 79 passed (was 78, the new a_round_2_result_injected_under_an_already_seen_job_id_carries_round_2); manifest 376 files match, required because .pi/extensions i…
- **ihr-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — 36 suites 0 failed; the six new pre-flight cases pass by name (replay_with_matching_digest_returns_stored_receipt_and_spawns_nothing, replay_with_differing_digest_refuses_loudly_and_spawns_nothing, r…

## Deviations

- **ihr-2** — D6's files half is a no-op for bee herding run: the verb carries no file list, so the digest is task-only; write and read sites pass the same empty vec, so it is symmetric and correct today.
- **ihr-2** — D6's files half is a no-op for bee herding run: the verb carries no file list (run.rs:2466, :2910, :3359 all build BriefSpec with an empty vec, all predating this cell), so the digest is task-only. Write sites (:2648, :3396) and the read site (:3971) pass the same empty vec, so it is symmetric and correct today. Logged as its own decision rather than left silent.
- **ihr-2** — sync-ack: affects_skills was predicted [] at plan time and that prediction was wrong: the change made skills/bee-herding/references/operational-invariants.md stale (it said the verb always starts an agent). The skill WAS updated and regenerated in commit 94991b963 rather than acked past — this ack covers only the wrong prediction, not a skipped sync.

## Provenance

Proposed by `bee knowledge promote --work idempotent-herding-receipts` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/idempotent-herding-receipts/CONTEXT.md`, `docs/history/idempotent-herding-receipts/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "idempotent-herding-receipts" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-20T05:03:45.938Z), the work item declares no bee.areas.

area bee-herding:
  - [ihr-1] The Pi drain names the round in every result injection; a later round is no longer read as a replay — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/ihr-1.json)
  - [ihr-2] bee herding run returns the stored receipt on a replay and refuses a reused job id carrying different work — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/ihr-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell ihr-2 — save as docs/knowledge/patterns/idempotent-herding-receipts-ihr-2-pitfall.md

---
type: bee.pattern
title: idempotent-herding-receipts cell ihr-2 — pitfall candidate
description: "Pitfall candidate mined from cell ihr-2's capped trace: D6's files half is a no-op for bee herding run: the verb carries no file list, so the digest is task-only; write and read sites pass the same empty vec, so it …"
timestamp: 2026-09-20
bee:
  id: idempotent-herding-receipts-ihr-2-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/ihr-2.json]
  polarity: pitfall
---

# idempotent-herding-receipts cell ihr-2 — pitfall candidate

## What the cell did

bee herding run returns the stored receipt on a replay and refuses a reused job id carrying different work

## Recorded evidence (verbatim from .bee/cells/ihr-2.json)

- **deviation** — D6's files half is a no-op for bee herding run: the verb carries no file list, so the digest is task-only; write and read sites pass the same empty vec, so it is symmetric and correct today.
- **deviation** — D6's files half is a no-op for bee herding run: the verb carries no file list (run.rs:2466, :2910, :3359 all build BriefSpec with an empty vec, all predating this cell), so the digest is task-only. Write sites (:2648, :3396) and the read site (:3971) pass the same empty vec, so it is symmetric and correct today. Logged as its own decision rather than left silent.
- **deviation** — sync-ack: affects_skills was predicted [] at plan time and that prediction was wrong: the change made skills/bee-herding/references/operational-invariants.md stale (it said the verb always starts an agent). The skill WAS updated and regenerated in commit 94991b963 rather than acked past — this ack covers only the wrong prediction, not a skipped sync.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 1 pattern candidate(s), 0 file(s) written.