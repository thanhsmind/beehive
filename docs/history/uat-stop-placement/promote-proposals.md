promote proposal for work item "uat-stop-placement" (docs/history/uat-stop-placement/CONTEXT.md + docs/history/uat-stop-placement/plan.md) — 7 capped cell(s): usp-1, usp-2, usp-3, usp-4, usp-5, usp-6, usp-7
anchor: history — docs/history/uat-stop-placement/CONTEXT.md, docs/history/uat-stop-placement/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/uat-stop-placement/delivery.md

---
type: bee.delivery
title: uat-stop-placement — delivery
description: "Delivery record proposed by bee knowledge promote for work item uat-stop-placement: 7 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-18
bee:
  id: uat-stop-placement-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [docs/history/uat-stop-placement/CONTEXT.md, docs/history/uat-stop-placement/plan.md]
  sources: [docs/history/uat-stop-placement/CONTEXT.md, docs/history/uat-stop-placement/plan.md, .bee/cells/usp-1.json, .bee/cells/usp-2.json, .bee/cells/usp-3.json, .bee/cells/usp-4.json, .bee/cells/usp-5.json, .bee/cells/usp-6.json, .bee/cells/usp-7.json]
---

# uat-stop-placement — Delivery

## What shipped

- **usp-1** — uat.rs: one policy module for where the uat stop sits and which lanes it covers (3 file(s) changed)
- **usp-2** — worktree merge: honor uat_stop, invert the post-merge lane write, hold the worktree (2 file(s) changed)
- **usp-3** — The close-time uat door and the merge side now classify the lane from one read, crate::uat::uat_lane_mode, so a record whose mode and route.lane disagree can no longer make the uat stop vanish between merge and close. (2 file(s) changed)
- **usp-4** — Teach the docs and skills that the uat stop has two possible positions (6 file(s) changed)
- **usp-5** — merge_finish now computes uat_wait_set directly from the fail-closed precheck, so a missing lane record no longer lets cleanup tear down a worktree with a pending uat under uat_stop close. (2 file(s) changed)
- **usp-6** — Deleted uat_merge_precheck's inline lane-mode read; it now calls crate::uat::uat_lane_mode (1 file(s) changed)
- **usp-7** — Fail cleanup suppression closed for a merge whose feature cannot be resolved, matching the merge-time precondition's existing fail-closed read (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **usp-1** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml uat`
- **usp-2** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml worktree`
- **usp-3** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml close && PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml uat`
- **usp-4** — `python3 -c "import json;json.load(open('.bee/config-sample.json'))" && rg -n 'uat_stop' docs/handbook/register.md docs/config-reference.md .bee/config-sample.json skills/bee-hive/references/gates-and-delegation.md skills/bee-swarming/SKILL.md && PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity && PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test instruction_laws && .bee/bin/bee dev release-manifest --check`
- **usp-5** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml worktree`
- **usp-6** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml worktree && PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml uat && PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml close`
- **usp-7** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml worktree`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work uat-stop-placement` from 7 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/uat-stop-placement/CONTEXT.md`, `docs/history/uat-stop-placement/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "uat-stop-placement" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-18T13:52:20.032Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [usp-2] worktree merge: honor uat_stop, invert the post-merge lane write, hold the worktree — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/usp-2.json)
  - [usp-3] The close-time uat door and the merge side now classify the lane from one read, crate::uat::uat_lane_mode, so a record whose mode and route.lane disagree can no longer make the uat stop vanish between merge and close. — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/usp-3.json)
  - [usp-5] merge_finish now computes uat_wait_set directly from the fail-closed precheck, so a missing lane record no longer lets cleanup tear down a worktree with a pending uat under uat_stop close. — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/usp-5.json)
  - [usp-7] Fail cleanup suppression closed for a merge whose feature cannot be resolved, matching the merge-time precondition's existing fail-closed read — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/usp-7.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell usp-3 — save as docs/knowledge/patterns/uat-stop-placement-usp-3-pitfall.md

---
type: bee.pattern
title: uat-stop-placement cell usp-3 — pitfall candidate
description: "Pitfall candidate mined from cell usp-3's capped trace: The close-time uat door classifies the lane through feature_route (route.lane first) while the merge side classifies through the record's mode; the two disagre…"
timestamp: 2026-08-18
bee:
  id: uat-stop-placement-usp-3-pitfall
  lifecycle: draft
  areas: [worktree-parallelism]
  sources: [.bee/cells/usp-3.json]
  polarity: pitfall
---

# uat-stop-placement cell usp-3 — pitfall candidate

## What the cell did

The close-time uat door and the merge side now classify the lane from one read, crate::uat::uat_lane_mode, so a record whose mode and route.lane disagree can no longer make the uat stop vanish between merge and close.

## Recorded evidence (verbatim from .bee/cells/usp-3.json)

- **failure_signature** — The close-time uat door classifies the lane through feature_route (route.lane first) while the merge side classifies through the record's mode; the two disagree on 12 of 95 real lane records (knowledge-loop: mode=standard, route.lane=small), so under uat_stop=close a merge sets waiting_on gate "uat: <feature>" for a feature bee close then exempts — automatic: the must_have itself names the merge side as canonical, so a follow-up cell can lift that mode read into crate::uat and call it from close with no human decision.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 7 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 1 pattern candidate(s), 0 file(s) written.