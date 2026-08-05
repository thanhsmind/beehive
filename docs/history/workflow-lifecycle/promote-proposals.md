promote proposal for work item "workflow-lifecycle" (.bee/logs/scribing-runs.jsonl) — 4 capped cell(s): wl-1, wl-2, wl-4, wl-5
anchor: ledger — .bee/logs/scribing-runs.jsonl
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/workflow-lifecycle/delivery.md

---
type: bee.delivery
title: workflow-lifecycle — delivery
description: "Delivery record proposed by bee knowledge promote for work item workflow-lifecycle: 4 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-07-28
bee:
  id: workflow-lifecycle-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [.bee/logs/scribing-runs.jsonl]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/cells/wl-1.json, .bee/cells/wl-2.json, .bee/cells/wl-4.json, .bee/cells/wl-5.json]
---

# workflow-lifecycle — Delivery

## What shipped

- **wl-1** — lib/state.mjs: added ensureWorkflowRecordForFeature (the ONE workflow-record creation seam, idempotent by feature) and routed startFeature's default+lane branches and seedLegacyWorkflows' C1 materialization through it; replaced fx-1's outgoing-feature close with close-by-feature (closeWorkflowsForFeature(root,{keepFeature}) closes every OTHER live record, default path only, lane path closes nothing and startLane is byte-untouched); exported the frozen interface contract listWorkflowRecords(root) + closeWorkflowsForFeature(root,{keepFeature}) from lib/state.mjs (they need controlRootFor, which the workflow-store leaf may not import). Path audit of every start/materialize/adopt site recorded as a header comment block. lib/workflow-store.mjs needed no change. (2 file(s) changed)
- **wl-2** — Added bee state workflows list/close CLI verb (bee.mjs + command-registry.mjs). list renders every workflow record (id/feature/status/phase/created_at, JSON + human, newest first). close supports --feature/--id/--all-but-active (mutually exclusive), refuses typed when nothing matches, and protects the calling context's active feature unless --id names it explicitly. Imports listWorkflowRecords/closeWorkflowsForFeature per MAIN's corrected contract (lib/state.mjs, not workflow-store.mjs). Smoke-tested read-only (list + close refusal paths); mutating close paths not exercised live to avoid touching concurrent swarms' real records. Two commits instead of one due to a shared-index git-hygiene incident (documented in deviations) plus MAIN's mid-work import-path correction. (2 file(s) changed)
- **wl-4** — Declared three new workflow-lifecycle exports (closeWorkflowsForFeature, ensureWorkflowRecordForFeature, listWorkflowRecords) in EXPECTED_STATE_EXPORTS. Test result: 118 passed, 0 failed. (1 file(s) changed)
- **wl-5** — Added runExample coverage for state.workflows.list and state.workflows.close registry examples; seeded a non-active stale-feature workflow record so the close example runs for real without touching the active feature's record. test_bee_cli.mjs: 346 passed, 0 failed. (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wl-1** — `node packages/bee/tests/test_state.mjs && node packages/bee/tests/test_workflow_store.mjs && node packages/bee/tests/test_state_projection.mjs`
- **wl-2** — `node packages/bee/tests/test_bee_cli.mjs`
- **wl-4** — `node packages/bee/tests/test_misc.mjs`
- **wl-5** — `node packages/bee/tests/test_bee_cli.mjs`

## Deviations

- **wl-2** — process: Interface-contract update mid-work: MAIN reported the sibling (wl-1, commit c01310e7) exported listWorkflowRecords/closeWorkflowsForFeature from lib/state.mjs, not lib/workflow-store.mjs as the cell text said (workflow-store.mjs cannot import controlRootFor). Adjusted the import line and every call site (both now called with plain `root`, not ctrlRoot, since they resolve controlRootFor internally) in a follow-up commit.
- **wl-2** — git-hygiene: The first wl-2 commit (99b989bf) was made while another concurrent process (skill-diet-wave2 lane) had unrelated files already staged in the same shared index (skills/bee-exploring/*, skills/bee-herding/*). `git commit -m` with no pathspec commits the whole index, so those files landed inside 99b989bf's diff even though only packages/bee/bee.mjs and lib/command-registry.mjs were `git add`-ed for this cell. A skill-diet-wave2 commit (fd27524d) then landed on top of 99b989bf before the mistake was caught. An initial `git reset --soft HEAD~1` orphaned fd27524d; this was immediately corrected with `git reset --hard fd27524d` (verified no content lost) before any further action. No history rewrite was attempted afterward — the import-path fix landed as a new, cleanly-scoped commit (b6d664c4, verified via `git diff --cached --name-only` before committing) rather than amending the polluted commit, to avoid re-orphaning fd27524d's descendants a second time. Net effect: correct content, but wl-2 now has two commits instead of one, and 99b989bf's diff also contains unrelated skill-diet-wave2 file changes. No data was lost; flagging for MAIN's awareness.
- **wl-2** — scope: MAIN offered a bonus: wiring the newly available ensureWorkflowRecordForFeature(root, {...}) into `bee state set --feature` (handleStateSet) to close the historical missing-record hole on that path. Left undone — it is a distinct verb from `state workflows list/close` and outside wl-2's must_haves; MAIN said to report rather than implement if it doesn't fit cleanly. Recommend a follow-up cell.

## Provenance

Proposed by `bee knowledge promote --work workflow-lifecycle` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "workflow-lifecycle" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-07-28T09:22:16.175Z), the work item declares no bee.areas.

area workflow-state:
  - [wl-1] lib/state.mjs: added ensureWorkflowRecordForFeature (the ONE workflow-record creation seam, idempotent by feature) and routed startFeature's default+lane branches and seedLegacyWorkflows' C1 materialization through it; replaced fx-1's outgoing-feature close with close-by-feature (closeWorkflowsForFeature(root,{keepFeature}) closes every OTHER live record, default path only, lane path closes nothing and startLane is byte-untouched); exported the frozen interface contract listWorkflowRecords(root) + closeWorkflowsForFeature(root,{keepFeature}) from lib/state.mjs (they need controlRootFor, which the workflow-store leaf may not import). Path audit of every start/materialize/adopt site recorded as a header comment block. lib/workflow-store.mjs needed no change. — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/wl-1.json)
  - [wl-2] Added bee state workflows list/close CLI verb (bee.mjs + command-registry.mjs). list renders every workflow record (id/feature/status/phase/created_at, JSON + human, newest first). close supports --feature/--id/--all-but-active (mutually exclusive), refuses typed when nothing matches, and protects the calling context's active feature unless --id names it explicitly. Imports listWorkflowRecords/closeWorkflowsForFeature per MAIN's corrected contract (lib/state.mjs, not workflow-store.mjs). Smoke-tested read-only (list + close refusal paths); mutating close paths not exercised live to avoid touching concurrent swarms' real records. Two commits instead of one due to a shared-index git-hygiene incident (documented in deviations) plus MAIN's mid-work import-path correction. — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/wl-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell wl-2 — save as docs/knowledge/patterns/workflow-lifecycle-wl-2-pitfall.md

---
type: bee.pattern
title: workflow-lifecycle cell wl-2 — pitfall candidate
description: "Pitfall candidate mined from cell wl-2's capped trace: process: Interface-contract update mid-work: MAIN reported the sibling (wl-1, commit c01310e7) exported listWorkflowRecords/closeWorkflowsForFeature from lib/s…"
timestamp: 2026-07-28
bee:
  id: workflow-lifecycle-wl-2-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/wl-2.json]
  polarity: pitfall
---

# workflow-lifecycle cell wl-2 — pitfall candidate

## What the cell did

Added bee state workflows list/close CLI verb (bee.mjs + command-registry.mjs). list renders every workflow record (id/feature/status/phase/created_at, JSON + human, newest first). close supports --feature/--id/--all-but-active (mutually exclusive), refuses typed when nothing matches, and protects the calling context's active feature unless --id names it explicitly. Imports listWorkflowRecords/closeWorkflowsForFeature per MAIN's corrected contract (lib/state.mjs, not workflow-store.mjs). Smoke-tested read-only (list + close refusal paths); mutating close paths not exercised live to avoid touching concurrent swarms' real records. Two commits instead of one due to a shared-index git-hygiene incident (documented in deviations) plus MAIN's mid-work import-path correction.

## Recorded evidence (verbatim from .bee/cells/wl-2.json)

- **deviation** — process: Interface-contract update mid-work: MAIN reported the sibling (wl-1, commit c01310e7) exported listWorkflowRecords/closeWorkflowsForFeature from lib/state.mjs, not lib/workflow-store.mjs as the cell text said (workflow-store.mjs cannot import controlRootFor). Adjusted the import line and every call site (both now called with plain `root`, not ctrlRoot, since they resolve controlRootFor internally) in a follow-up commit.
- **deviation** — git-hygiene: The first wl-2 commit (99b989bf) was made while another concurrent process (skill-diet-wave2 lane) had unrelated files already staged in the same shared index (skills/bee-exploring/*, skills/bee-herding/*). `git commit -m` with no pathspec commits the whole index, so those files landed inside 99b989bf's diff even though only packages/bee/bee.mjs and lib/command-registry.mjs were `git add`-ed for this cell. A skill-diet-wave2 commit (fd27524d) then landed on top of 99b989bf before the mistake was caught. An initial `git reset --soft HEAD~1` orphaned fd27524d; this was immediately corrected with `git reset --hard fd27524d` (verified no content lost) before any further action. No history rewrite was attempted afterward — the import-path fix landed as a new, cleanly-scoped commit (b6d664c4, verified via `git diff --cached --name-only` before committing) rather than amending the polluted commit, to avoid re-orphaning fd27524d's descendants a second time. Net effect: correct content, but wl-2 now has two commits instead of one, and 99b989bf's diff also contains unrelated skill-diet-wave2 file changes. No data was lost; flagging for MAIN's awareness.
- **deviation** — scope: MAIN offered a bonus: wiring the newly available ensureWorkflowRecordForFeature(root, {...}) into `bee state set --feature` (handleStateSet) to close the historical missing-record hole on that path. Left undone — it is a distinct verb from `state workflows list/close` and outside wl-2's must_haves; MAIN said to report rather than implement if it doesn't fit cleanly. Recommend a follow-up cell.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 1 pattern candidate(s), 0 file(s) written.