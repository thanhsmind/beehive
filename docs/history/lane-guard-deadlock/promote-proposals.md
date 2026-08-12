promote proposal for work item "lane-guard-deadlock" (.bee/lanes/lane-guard-deadlock.json) — 2 capped cell(s): lgd-1, lgd-2
anchor: ledger — .bee/lanes/lane-guard-deadlock.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/lane-guard-deadlock/delivery.md

---
type: bee.delivery
title: lane-guard-deadlock — delivery
description: "Delivery record proposed by bee knowledge promote for work item lane-guard-deadlock: 2 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-12
bee:
  id: lane-guard-deadlock-delivery
  lifecycle: active
  required_context: [.bee/lanes/lane-guard-deadlock.json]
  sources: [.bee/lanes/lane-guard-deadlock.json, .bee/cells/lgd-1.json, .bee/cells/lgd-2.json]
---

# lane-guard-deadlock — Delivery

## What shipped

- **lgd-1** — session bind refuses a lane with no .bee/lanes record, before the sessions lock, reusing the shared LANE_MISSING wording (2 file(s) changed)
- **lgd-2** — check_git_bash_command tokenizes and finds git invocations before resolving the acting record; with no git invocation it returns without resolving, so a broken lane binding no longer denies the unbind that escapes it (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **lgd-1** — `commands.test green`
- **lgd-2** — `commands.test green`

## Deviations

- **lgd-2** — Claimed after the edit, not before: the claim needed a route record, the route needed the lane bound, and the harness refused the control-plane call while the session was isolated in the worktree. Work, tests and commit were done first in the worktree, then the bookkeeping from main.

## Provenance

Proposed by `bee knowledge promote --work lane-guard-deadlock` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/lane-guard-deadlock.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell lgd-1 — save as docs/knowledge/patterns/lane-guard-deadlock-lgd-1-pitfall.md

---
type: bee.pattern
title: lane-guard-deadlock cell lgd-1 — pitfall candidate
description: "Pitfall candidate mined from cell lgd-1's capped trace: e47ff85140fd"
timestamp: 2026-08-12
bee:
  id: lane-guard-deadlock-lgd-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/lgd-1.json]
  polarity: pitfall
---

# lane-guard-deadlock cell lgd-1 — pitfall candidate

## What the cell did

session bind refuses a lane with no .bee/lanes record, before the sessions lock, reusing the shared LANE_MISSING wording

## Recorded evidence (verbatim from .bee/cells/lgd-1.json)

- **failure_signature** — e47ff85140fd

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell lgd-2 — save as docs/knowledge/patterns/lane-guard-deadlock-lgd-2-pitfall.md

---
type: bee.pattern
title: lane-guard-deadlock cell lgd-2 — pitfall candidate
description: "Pitfall candidate mined from cell lgd-2's capped trace: Claimed after the edit, not before: the claim needed a route record, the route needed the lane bound, and the harness refused the control-plane call while the …"
timestamp: 2026-08-12
bee:
  id: lane-guard-deadlock-lgd-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/lgd-2.json]
  polarity: pitfall
---

# lane-guard-deadlock cell lgd-2 — pitfall candidate

## What the cell did

check_git_bash_command tokenizes and finds git invocations before resolving the acting record; with no git invocation it returns without resolving, so a broken lane binding no longer denies the unbind that escapes it

## Recorded evidence (verbatim from .bee/cells/lgd-2.json)

- **deviation** — Claimed after the edit, not before: the claim needed a route record, the route needed the lane bound, and the harness refused the control-plane call while the session was isolated in the worktree. Work, tests and commit were done first in the worktree, then the bookkeeping from main.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.