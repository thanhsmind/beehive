promote proposal for work item "stale-worktree-lock" (.bee/lanes/stale-worktree-lock.json) — 1 capped cell(s): swl-1
anchor: ledger — .bee/lanes/stale-worktree-lock.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/stale-worktree-lock/delivery.md

---
type: bee.delivery
title: stale-worktree-lock — delivery
description: "Delivery record proposed by bee knowledge promote for work item stale-worktree-lock: 1 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-26
bee:
  id: stale-worktree-lock-delivery
  lifecycle: active
  required_context: [.bee/lanes/stale-worktree-lock.json]
  sources: [.bee/lanes/stale-worktree-lock.json, .bee/cells/swl-1.json]
---

# stale-worktree-lock — Delivery

## What shipped

- **swl-1** — Released this session's expired worktree lock and pruned the merged human-mailbox worktree, reclaiming 2.1 GB (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **swl-1** — `git worktree list | grep -c human-mailbox`

## Deviations

- **swl-1** — The lock and the idle gate formed a small deadlock: the lock could only be released by a session doing bee work, and releasing it was not itself bee work. Opened a tiny cell rather than disabling guards.idle_gate, which the refusal itself offers as a last resort. Kind: hit an unforeseen obstacle.

## Provenance

Proposed by `bee knowledge promote --work stale-worktree-lock` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/stale-worktree-lock.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell swl-1 — save as docs/knowledge/patterns/stale-worktree-lock-swl-1-pitfall.md

---
type: bee.pattern
title: stale-worktree-lock cell swl-1 — pitfall candidate
description: "Pitfall candidate mined from cell swl-1's capped trace: The lock and the idle gate formed a small deadlock: the lock could only be released by a session doing bee work, and releasing it was not itself bee work. Open…"
timestamp: 2026-08-26
bee:
  id: stale-worktree-lock-swl-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/swl-1.json]
  polarity: pitfall
---

# stale-worktree-lock cell swl-1 — pitfall candidate

## What the cell did

Released this session's expired worktree lock and pruned the merged human-mailbox worktree, reclaiming 2.1 GB

## Recorded evidence (verbatim from .bee/cells/swl-1.json)

- **deviation** — The lock and the idle gate formed a small deadlock: the lock could only be released by a session doing bee work, and releasing it was not itself bee work. Opened a tiny cell rather than disabling guards.idle_gate, which the refusal itself offers as a last resort. Kind: hit an unforeseen obstacle.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 1 pattern candidate(s), 0 file(s) written.