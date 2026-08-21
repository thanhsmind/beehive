promote proposal for work item "auto-wait-mark" (docs/history/auto-wait-mark/CONTEXT.md + docs/history/auto-wait-mark/plan.md) — 2 capped cell(s): awm-1, awm-2
anchor: history — docs/history/auto-wait-mark/CONTEXT.md, docs/history/auto-wait-mark/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/auto-wait-mark/delivery.md

---
type: bee.delivery
title: auto-wait-mark — delivery
description: "Delivery record proposed by bee knowledge promote for work item auto-wait-mark: 2 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-18
bee:
  id: auto-wait-mark-delivery
  lifecycle: active
  areas: [workflow-state, rust-runtime]
  required_context: [docs/history/auto-wait-mark/CONTEXT.md, docs/history/auto-wait-mark/plan.md]
  sources: [docs/history/auto-wait-mark/CONTEXT.md, docs/history/auto-wait-mark/plan.md, .bee/cells/archive/auto-wait-mark/awm-1.json, .bee/cells/archive/auto-wait-mark/awm-2.json]
---

# auto-wait-mark — Delivery

## What shipped

- **awm-1** — Widened the waiting_on kind vocabulary to gate/question/turn-end and stopped bee orient calling a turn-end mark a blocker; refusal-message assertions now pin all three values, proven red-then-green by mutation. OWED and deliberately not fixed here: verbs/state_group/tests.rs:586 and tests/workflow_verbs.rs:566 still carry the prefix-only two-value assertion — the first is reserved by worktree start-feature-reservation-scope (cell sfrs-1, whose commit 60f9d2c7 already rewrote that file in main, so this worktree's copy is stale), the second was found by the judge and belongs to the same re-triage pass after that merge. (9 file(s) changed)
- **awm-2** — The Stop hook writes a turn-end waiting mark on every genuine turn end, reusing the store's existing setter and D3 target resolution, never overwriting a declared gate or question mark, with the subject taken from the transcript's last non-empty assistant line. The transcript is read exactly once per Stop — one resolver, one read, proven by two read-counting tests including the late-perf-failure path that two earlier attempts leaked. (7 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **awm-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check`
- **awm-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work auto-wait-mark` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/auto-wait-mark/CONTEXT.md`, `docs/history/auto-wait-mark/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "auto-wait-mark" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-18T14:47:32.075Z), the work item declares no bee.areas.

area workflow-state:
  - [awm-1] Widened the waiting_on kind vocabulary to gate/question/turn-end and stopped bee orient calling a turn-end mark a blocker; refusal-message assertions now pin all three values, proven red-then-green by mutation. OWED and deliberately not fixed here: verbs/state_group/tests.rs:586 and tests/workflow_verbs.rs:566 still carry the prefix-only two-value assertion — the first is reserved by worktree start-feature-reservation-scope (cell sfrs-1, whose commit 60f9d2c7 already rewrote that file in main, so this worktree's copy is stale), the second was found by the judge and belongs to the same re-triage pass after that merge. — feature-wide sync per the scribing stamp, 9 file(s) changed (trace .bee/cells/archive/auto-wait-mark/awm-1.json)
  - [awm-2] The Stop hook writes a turn-end waiting mark on every genuine turn end, reusing the store's existing setter and D3 target resolution, never overwriting a declared gate or question mark, with the subject taken from the transcript's last non-empty assistant line. The transcript is read exactly once per Stop — one resolver, one read, proven by two read-counting tests including the late-perf-failure path that two earlier attempts leaked. — feature-wide sync per the scribing stamp, 7 file(s) changed (trace .bee/cells/archive/auto-wait-mark/awm-2.json)

area rust-runtime:
  - [awm-1] Widened the waiting_on kind vocabulary to gate/question/turn-end and stopped bee orient calling a turn-end mark a blocker; refusal-message assertions now pin all three values, proven red-then-green by mutation. OWED and deliberately not fixed here: verbs/state_group/tests.rs:586 and tests/workflow_verbs.rs:566 still carry the prefix-only two-value assertion — the first is reserved by worktree start-feature-reservation-scope (cell sfrs-1, whose commit 60f9d2c7 already rewrote that file in main, so this worktree's copy is stale), the second was found by the judge and belongs to the same re-triage pass after that merge. — feature-wide sync per the scribing stamp, 9 file(s) changed (trace .bee/cells/archive/auto-wait-mark/awm-1.json)
  - [awm-2] The Stop hook writes a turn-end waiting mark on every genuine turn end, reusing the store's existing setter and D3 target resolution, never overwriting a declared gate or question mark, with the subject taken from the transcript's last non-empty assistant line. The transcript is read exactly once per Stop — one resolver, one read, proven by two read-counting tests including the late-perf-failure path that two earlier attempts leaked. — feature-wide sync per the scribing stamp, 7 file(s) changed (trace .bee/cells/archive/auto-wait-mark/awm-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell awm-1 — save as docs/knowledge/patterns/auto-wait-mark-awm-1-pitfall.md

---
type: bee.pattern
title: auto-wait-mark cell awm-1 — pitfall candidate
description: "Pitfall candidate mined from cell awm-1's capped trace: orient.rs:407 hardcodes the \"turn-end\" literal instead of sharing record.rs's constant — two spellings of the kind value, K1's drift guard unmet"
timestamp: 2026-08-18
bee:
  id: auto-wait-mark-awm-1-pitfall
  lifecycle: draft
  areas: [workflow-state, rust-runtime]
  sources: [.bee/cells/archive/auto-wait-mark/awm-1.json]
  polarity: pitfall
---

# auto-wait-mark cell awm-1 — pitfall candidate

## What the cell did

Widened the waiting_on kind vocabulary to gate/question/turn-end and stopped bee orient calling a turn-end mark a blocker; refusal-message assertions now pin all three values, proven red-then-green by mutation. OWED and deliberately not fixed here: verbs/state_group/tests.rs:586 and tests/workflow_verbs.rs:566 still carry the prefix-only two-value assertion — the first is reserved by worktree start-feature-reservation-scope (cell sfrs-1, whose commit 60f9d2c7 already rewrote that file in main, so this worktree's copy is stale), the second was found by the judge and belongs to the same re-triage pass after that merge.

## Recorded evidence (verbatim from .bee/cells/archive/auto-wait-mark/awm-1.json)

- **failure_signature** — orient.rs:407 hardcodes the "turn-end" literal instead of sharing record.rs's constant — two spellings of the kind value, K1's drift guard unmet

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell awm-2 — save as docs/knowledge/patterns/auto-wait-mark-awm-2-pitfall.md

---
type: bee.pattern
title: auto-wait-mark cell awm-2 — pitfall candidate
description: "Pitfall candidate mined from cell awm-2's capped trace: transcript-read-doubled: turn_end_subject (mod.rs:357 read_jsonl) re-reads the transcript perf_refresh already read via rollup_transcript (perf.rs:378), so eve…"
timestamp: 2026-08-18
bee:
  id: auto-wait-mark-awm-2-pitfall
  lifecycle: draft
  areas: [workflow-state, rust-runtime]
  sources: [.bee/cells/archive/auto-wait-mark/awm-2.json]
  polarity: pitfall
---

# auto-wait-mark cell awm-2 — pitfall candidate

## What the cell did

The Stop hook writes a turn-end waiting mark on every genuine turn end, reusing the store's existing setter and D3 target resolution, never overwriting a declared gate or question mark, with the subject taken from the transcript's last non-empty assistant line. The transcript is read exactly once per Stop — one resolver, one read, proven by two read-counting tests including the late-perf-failure path that two earlier attempts leaked.

## Recorded evidence (verbatim from .bee/cells/archive/auto-wait-mark/awm-2.json)

- **failure_signature** — transcript-read-doubled: turn_end_subject (mod.rs:357 read_jsonl) re-reads the transcript perf_refresh already read via rollup_transcript (perf.rs:378), so every Stop performs two full std::fs::read of the same file — violating K2, P3, and CONTEXT.md:45-47's locked constraint. Fix: have perf_refresh or rollup hand back the parsed events and feed turn_end_subject from those instead of re-reading.
- **failure_signature** — Stop still reads the transcript twice when perf_refresh errors after its own read (html.rs:272-286 -> mod.rs:159 None -> mod.rs:386 fallback re-read), and mod.rs:365-368 documents that caller as non-existent

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 2 pattern candidate(s), 0 file(s) written.