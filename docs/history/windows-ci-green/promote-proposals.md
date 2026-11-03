promote proposal for work item "windows-ci-green" (docs/history/windows-ci-green/CONTEXT.md + docs/history/windows-ci-green/plan.md) — 3 capped cell(s): win-1, win-2, win-3
anchor: history — docs/history/windows-ci-green/CONTEXT.md, docs/history/windows-ci-green/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/windows-ci-green/delivery.md

---
type: bee.delivery
title: windows-ci-green — delivery
description: "Delivery record proposed by bee knowledge promote for work item windows-ci-green: 3 capped cell(s), 4 recorded deviation(s)."
timestamp: 2026-09-19
bee:
  id: windows-ci-green-delivery
  lifecycle: active
  required_context: [docs/history/windows-ci-green/CONTEXT.md, docs/history/windows-ci-green/plan.md]
  sources: [docs/history/windows-ci-green/CONTEXT.md, docs/history/windows-ci-green/plan.md, .bee/cells/win-1.json, .bee/cells/win-2.json, .bee/cells/win-3.json]
---

# windows-ci-green — Delivery

## What shipped

- **win-1** — Canonicalize the worktree test paths and make the release-dirt failure show its cause (2 file(s) changed)
- **win-2** — release-dirt runs under real Win32 bash on Windows; merge path expectation matches product (2 file(s) changed)
- **win-3** — new-worktree instruction expectation matches the raw path the product emits (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **win-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee` — 3689 passed, 0 failed, 20 ignored, re-run by the leader on Linux; both target tests executed and passed: test_release_dirt_script_filters_only_bee_paths and enter_and_merge_and_new_carry_session_runt…
- **win-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee` — the cell verify; 3689 passed 0 failed on Linux, Windows proof comes from the branch CI run
- **win-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee` — the cell verify; 3689 passed 0 failed on Linux, Windows proof comes from the branch CI run

## Deviations

- **win-1** — followed the plan
- **win-1** — sync-ack: No skill text changes: two test-only edits, no behavior a skill describes.
- **win-2** — the leader capped the cell, not the worker — the herding worker reported done with a narrow proof and skipped its finish command — something else had to be fixed first
- **win-3** — tiny cell run inline by the leader, no worker — one-line test edit, which the tiny lane allows — found a better route

## Provenance

Proposed by `bee knowledge promote --work windows-ci-green` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/windows-ci-green/CONTEXT.md`, `docs/history/windows-ci-green/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell win-1 — save as docs/knowledge/patterns/windows-ci-green-win-1-pitfall.md

---
type: bee.pattern
title: windows-ci-green cell win-1 — pitfall candidate
description: "Pitfall candidate mined from cell win-1's capped trace: followed the plan"
timestamp: 2026-09-19
bee:
  id: windows-ci-green-win-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/win-1.json]
  polarity: pitfall
---

# windows-ci-green cell win-1 — pitfall candidate

## What the cell did

Canonicalize the worktree test paths and make the release-dirt failure show its cause

## Recorded evidence (verbatim from .bee/cells/win-1.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: No skill text changes: two test-only edits, no behavior a skill describes.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell win-2 — save as docs/knowledge/patterns/windows-ci-green-win-2-pitfall.md

---
type: bee.pattern
title: windows-ci-green cell win-2 — pitfall candidate
description: "Pitfall candidate mined from cell win-2's capped trace: the leader capped the cell, not the worker — the herding worker reported done with a narrow proof and skipped its finish command — something else had to be fix…"
timestamp: 2026-09-19
bee:
  id: windows-ci-green-win-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/win-2.json]
  polarity: pitfall
---

# windows-ci-green cell win-2 — pitfall candidate

## What the cell did

release-dirt runs under real Win32 bash on Windows; merge path expectation matches product

## Recorded evidence (verbatim from .bee/cells/win-2.json)

- **deviation** — the leader capped the cell, not the worker — the herding worker reported done with a narrow proof and skipped its finish command — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell win-3 — save as docs/knowledge/patterns/windows-ci-green-win-3-pitfall.md

---
type: bee.pattern
title: windows-ci-green cell win-3 — pitfall candidate
description: "Pitfall candidate mined from cell win-3's capped trace: tiny cell run inline by the leader, no worker — one-line test edit, which the tiny lane allows — found a better route"
timestamp: 2026-09-19
bee:
  id: windows-ci-green-win-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/win-3.json]
  polarity: pitfall
---

# windows-ci-green cell win-3 — pitfall candidate

## What the cell did

new-worktree instruction expectation matches the raw path the product emits

## Recorded evidence (verbatim from .bee/cells/win-3.json)

- **deviation** — tiny cell run inline by the leader, no worker — one-line test edit, which the tiny lane allows — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 3 pattern candidate(s), 0 file(s) written.