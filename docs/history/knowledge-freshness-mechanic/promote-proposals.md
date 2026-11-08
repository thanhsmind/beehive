promote proposal for work item "knowledge-freshness-mechanic" (docs/history/knowledge-freshness-mechanic/CONTEXT.md + docs/history/knowledge-freshness-mechanic/plan.md) — 7 capped cell(s): kfm-1, kfm-2, kfm-3, kfm-4, kfm-5, kfm-6, kfm-7
anchor: history — docs/history/knowledge-freshness-mechanic/CONTEXT.md, docs/history/knowledge-freshness-mechanic/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

This proposal covers (b) area updates and (c) pattern candidates only; it proposes no delivery record.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "knowledge-freshness-mechanic" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-08T04:42:56.679Z), the work item declares no bee.areas.

area okf-profile:
  - [kfm-1] Grade concept freshness from anchors and git, with seed and confirm verbs — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/kfm-1.json)
  - [kfm-2] Print the grade wherever a concept is read — feature-wide sync per the scribing stamp, 7 file(s) changed (trace .bee/cells/kfm-2.json)
  - [kfm-3] Added check (d) to the cap sync door (suspect/orphan anchored concept refuses, unverified passes), the worker-contract re-confirm step and regenerated manifest; commit ae328f47a697ab020af766dfee2fee7b951b68ba — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/kfm-3.json)
  - [kfm-4] Delivery draft retired from promote, both bundle walks skip work/, preamble announces no work-item anchor, work/ tree moved to docs/history/knowledge-work and paths repointed; commit 5e715b425 — feature-wide sync per the scribing stamp, 14 file(s) changed (trace .bee/cells/kfm-4.json)
  - [kfm-5] record_timing now writes the env-bound feature (or null) on every row, and bee knowledge report prints a per-feature pull-rate table; commit edc0f9e6d — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/kfm-5.json)

area workflow-state:
  - [kfm-1] Grade concept freshness from anchors and git, with seed and confirm verbs — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/kfm-1.json)
  - [kfm-2] Print the grade wherever a concept is read — feature-wide sync per the scribing stamp, 7 file(s) changed (trace .bee/cells/kfm-2.json)
  - [kfm-3] Added check (d) to the cap sync door (suspect/orphan anchored concept refuses, unverified passes), the worker-contract re-confirm step and regenerated manifest; commit ae328f47a697ab020af766dfee2fee7b951b68ba — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/kfm-3.json)
  - [kfm-4] Delivery draft retired from promote, both bundle walks skip work/, preamble announces no work-item anchor, work/ tree moved to docs/history/knowledge-work and paths repointed; commit 5e715b425 — feature-wide sync per the scribing stamp, 14 file(s) changed (trace .bee/cells/kfm-4.json)
  - [kfm-5] record_timing now writes the env-bound feature (or null) on every row, and bee knowledge report prints a per-feature pull-rate table; commit edc0f9e6d — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/kfm-5.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell kfm-1 — save as docs/knowledge/patterns/knowledge-freshness-mechanic-kfm-1-pitfall.md

---
type: bee.pattern
title: knowledge-freshness-mechanic cell kfm-1 — pitfall candidate
description: "Pitfall candidate mined from cell kfm-1's capped trace: window rule changed from the plan.md first-parent wording to the vc..HEAD graph by owner direction t-ae0fad (cells answer on kfm-1)"
timestamp: 2026-11-07
bee:
  id: knowledge-freshness-mechanic-kfm-1-pitfall
  lifecycle: draft
  areas: [okf-profile, workflow-state]
  sources: [.bee/cells/kfm-1.json]
  polarity: pitfall
---

# knowledge-freshness-mechanic cell kfm-1 — pitfall candidate

## What the cell did

Grade concept freshness from anchors and git, with seed and confirm verbs

## Recorded evidence (verbatim from .bee/cells/kfm-1.json)

- **deviation** — window rule changed from the plan.md first-parent wording to the vc..HEAD graph by owner direction t-ae0fad (cells answer on kfm-1)
- **deviation** — run ceiling raised from 1800 s to 3600 s again for a release build
- **deviation** — the earlier worker job-1794068531857-202731-1 stalled idle for about 3 h and was cancelled before this re-dispatch
- **deviation** — four worker rounds: semantic judge rounds 2 and 3 found the unrelated-history and merge-commit gaps after the first rework
- **failure_signature** — grade window: a verified_commit off HEAD's first-parent chain (worktree confirm landed by --no-ff merge) with 2+ distinct vcs grades suspect on commits that are ancestors of vc (freshness.rs:169-177,206-209)
- **failure_signature** — grade window: verified_commits from two unrelated histories joined into HEAD (merge --allow-unrelated-histories) make merge-base exit 1 with empty stderr, and freshness.rs:165 marks the second-sorted vc dead as 'shares no history' so an ancestor-of-HEAD concept grades unverified, breaking 'window is exactly vc..HEAD; only non-ancestors stay unverified with the existing reason texts'
- **failure_signature** — grade window: a merge commit whose own tree change edits an anchor (evil merge or conflict-only resolution) lists no files because the single git log has no -m/--cc, so the concept grades confirmed instead of suspect

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell kfm-2 — save as docs/knowledge/patterns/knowledge-freshness-mechanic-kfm-2-pitfall.md

---
type: bee.pattern
title: knowledge-freshness-mechanic cell kfm-2 — pitfall candidate
description: "Pitfall candidate mined from cell kfm-2's capped trace: sync_door.rs edited for one doc-comment deletion under decision 9c7553ba (comment_baseline repair), outside the plan's file list for this cell"
timestamp: 2026-11-07
bee:
  id: knowledge-freshness-mechanic-kfm-2-pitfall
  lifecycle: draft
  areas: [okf-profile, workflow-state]
  sources: [.bee/cells/kfm-2.json]
  polarity: pitfall
---

# knowledge-freshness-mechanic cell kfm-2 — pitfall candidate

## What the cell did

Print the grade wherever a concept is read

## Recorded evidence (verbatim from .bee/cells/kfm-2.json)

- **deviation** — sync_door.rs edited for one doc-comment deletion under decision 9c7553ba (comment_baseline repair), outside the plan's file list for this cell
- **deviation** — sync-ack: sync_door.rs touched only to delete one doc-comment line (comment_baseline repair, decision 9c7553ba); no behaviour or skill text changes

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell kfm-3 — save as docs/knowledge/patterns/knowledge-freshness-mechanic-kfm-3-pitfall.md

---
type: bee.pattern
title: knowledge-freshness-mechanic cell kfm-3 — pitfall candidate
description: "Pitfall candidate mined from cell kfm-3's capped trace: run ceiling raised from 1800 s to 3600 s for a release build"
timestamp: 2026-11-07
bee:
  id: knowledge-freshness-mechanic-kfm-3-pitfall
  lifecycle: draft
  areas: [okf-profile, workflow-state]
  sources: [.bee/cells/kfm-3.json]
  polarity: pitfall
---

# knowledge-freshness-mechanic cell kfm-3 — pitfall candidate

## What the cell did

Added check (d) to the cap sync door (suspect/orphan anchored concept refuses, unverified passes), the worker-contract re-confirm step and regenerated manifest; commit ae328f47a697ab020af766dfee2fee7b951b68ba

## Recorded evidence (verbatim from .bee/cells/kfm-3.json)

- **deviation** — run ceiling raised from 1800 s to 3600 s for a release build
- **deviation** — the first rework judge went idle twice with no result and was cancelled and re-run
- **deviation** — sync-ack: the bee-swarming re-confirm step landed in this cell's first round (ae328f47a); the rework only changes how check (d) matches anchors, which the skill step does not describe
- **failure_signature** — cap door matches anchors by exact path only (sync_door.rs:151-156) while the grade treats directory anchors as covering files under them (freshness.rs:113-115); seed writes directory anchors, so a cell editing a file under an anchored directory caps unrefused

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell kfm-6 — save as docs/knowledge/patterns/knowledge-freshness-mechanic-kfm-6-pitfall.md

---
type: bee.pattern
title: knowledge-freshness-mechanic cell kfm-6 — pitfall candidate
description: "Pitfall candidate mined from cell kfm-6's capped trace: the installed binary could not be refreshed from the worktree (.bee/bin/bee is a symlink into main); the seed and checks ran with the freshly built target bina…"
timestamp: 2026-11-07
bee:
  id: knowledge-freshness-mechanic-kfm-6-pitfall
  lifecycle: draft
  areas: [okf-profile, workflow-state]
  sources: [.bee/cells/kfm-6.json]
  polarity: pitfall
---

# knowledge-freshness-mechanic cell kfm-6 — pitfall candidate

## What the cell did

Wrote the freshness-and-document-homes concept, bee-capturing paragraph and doc fixes, seeded anchors (122 concepts anchored, 17 none); knowledge check 0 errors, freshness: 47 confirmed, 76 suspect, 0 orphan, 17 unverified; commit c5855442590cdd61cd67c72e7723f7b8fab0aae8

## Recorded evidence (verbatim from .bee/cells/kfm-6.json)

- **deviation** — the installed binary could not be refreshed from the worktree (.bee/bin/bee is a symlink into main); the seed and checks ran with the freshly built target binary, and main's binary must be rebuilt after merge
- **deviation** — sync-ack: seed wrote bee.anchors frontmatter only on three rule-home concepts; applied_at files unchanged (predicted in the plan)

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 7 capped cell(s) mined, 10 area bullet(s), 4 pattern candidate(s), 0 file(s) written.