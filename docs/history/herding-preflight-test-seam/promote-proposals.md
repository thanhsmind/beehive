promote proposal for work item "herding-preflight-test-seam" (docs/history/herding-preflight-test-seam/CONTEXT.md + docs/history/herding-preflight-test-seam/plan.md) — 1 capped cell(s): hpts-1
anchor: history — docs/history/herding-preflight-test-seam/CONTEXT.md, docs/history/herding-preflight-test-seam/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-preflight-test-seam/delivery.md

---
type: bee.delivery
title: herding-preflight-test-seam — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-preflight-test-seam: 1 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-09-20
bee:
  id: herding-preflight-test-seam-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-preflight-test-seam/CONTEXT.md, docs/history/herding-preflight-test-seam/plan.md]
  sources: [docs/history/herding-preflight-test-seam/CONTEXT.md, docs/history/herding-preflight-test-seam/plan.md, .bee/cells/hpts-1.json]
---

# herding-preflight-test-seam — Delivery

## What shipped

- **hpts-1** — The pre-flight decision is a pure function; no test in this crate spawns a real agent (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hpts-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — 36 suites 0 failed, twice; and the leak proof the suite itself cannot assert: session files under ~/.claude/projects/*crates-bee/ stayed at 12 and agent panes stayed at 3 across two full runs, agains…

## Deviations

- **hpts-1** — plan.md committed from main, not with the cell: the plan-freeze guard refuses intent-to-add while the concurrent-worker guard requires it. Filed P3.
- **hpts-1** — plan.md is not in this cell's commit. The plan-freeze guard refuses intent-to-add on a gated plan.md, while the concurrent-worker guard requires intent-to-add to get an untracked file into a path-scoped commit — the two contradict. plan.md lands in the close commit from main instead. Filed P3.
- **hpts-1** — sync-ack: No bee-herding skill text is stale: this cell changes no user-visible behavior. The skill's replay-rule section (written at the idempotent-herding-receipts close) states the placement 'in the PARENT process, immediately after options parse, above the transport choice and above the detached re-launch' and the five outcomes — both still true character for character. Only the internal seam moved: deciding split from acting so tests stop spawning.

## Provenance

Proposed by `bee knowledge promote --work herding-preflight-test-seam` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-preflight-test-seam/CONTEXT.md`, `docs/history/herding-preflight-test-seam/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-preflight-test-seam" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-20T06:30:27.354Z), the work item declares no bee.areas.

area bee-herding:
  - [hpts-1] The pre-flight decision is a pure function; no test in this crate spawns a real agent — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/hpts-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell hpts-1 — save as docs/knowledge/patterns/herding-preflight-test-seam-hpts-1-pitfall.md

---
type: bee.pattern
title: herding-preflight-test-seam cell hpts-1 — pitfall candidate
description: "Pitfall candidate mined from cell hpts-1's capped trace: plan.md committed from main, not with the cell: the plan-freeze guard refuses intent-to-add while the concurrent-worker guard requires it. Filed P3."
timestamp: 2026-09-20
bee:
  id: herding-preflight-test-seam-hpts-1-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/hpts-1.json]
  polarity: pitfall
---

# herding-preflight-test-seam cell hpts-1 — pitfall candidate

## What the cell did

The pre-flight decision is a pure function; no test in this crate spawns a real agent

## Recorded evidence (verbatim from .bee/cells/hpts-1.json)

- **deviation** — plan.md committed from main, not with the cell: the plan-freeze guard refuses intent-to-add while the concurrent-worker guard requires it. Filed P3.
- **deviation** — plan.md is not in this cell's commit. The plan-freeze guard refuses intent-to-add on a gated plan.md, while the concurrent-worker guard requires intent-to-add to get an untracked file into a path-scoped commit — the two contradict. plan.md lands in the close commit from main instead. Filed P3.
- **deviation** — sync-ack: No bee-herding skill text is stale: this cell changes no user-visible behavior. The skill's replay-rule section (written at the idempotent-herding-receipts close) states the placement 'in the PARENT process, immediately after options parse, above the transport choice and above the detached re-launch' and the five outcomes — both still true character for character. Only the internal seam moved: deciding split from acting so tests stop spawning.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.