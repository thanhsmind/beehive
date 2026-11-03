promote proposal for work item "windows-path-tests" (docs/history/windows-path-tests/plan.md) — 1 capped cell(s): wpt-1
anchor: history — docs/history/windows-path-tests/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/windows-path-tests/delivery.md

---
type: bee.delivery
title: windows-path-tests — delivery
description: "Delivery record proposed by bee knowledge promote for work item windows-path-tests: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-10-07
bee:
  id: windows-path-tests-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [docs/history/windows-path-tests/plan.md]
  sources: [docs/history/windows-path-tests/plan.md, .bee/cells/wpt-1.json]
---

# windows-path-tests — Delivery

## What shipped

- **wpt-1** — docs_root_for_feature uses roots::same_path; status_full test compares path via same_path; paseo test JSON-escapes path; all four CwdGuards recover a poisoned TEST_CWD_LOCK (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wpt-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- verbs::drivers::close verbs::status_full verbs::drivers::prepare verbs::state_group::set_gate verbs::cells` — cell verify run on Linux: 680 passed, 0 failed, 12 ignored; the alias and paseo tests were also seen red with the two fixes reverted; Windows not run here, Windows CI is the real proof after merge

## Deviations

- **wpt-1** — added docs_root_for_feature_matches_an_aliased_main_root and changed the paseo test dir to bin\dir so Linux exercises both fixes — leader note asked for tests that fail without the fix — found a better route
- **wpt-1** — sync-ack: path-compare and test-only fixes for Windows; no workflow behavior that a skill describes changed

## Provenance

Proposed by `bee knowledge promote --work windows-path-tests` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/windows-path-tests/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "windows-path-tests" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-07T06:37:28.440Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [wpt-1] docs_root_for_feature uses roots::same_path; status_full test compares path via same_path; paseo test JSON-escapes path; all four CwdGuards recover a poisoned TEST_CWD_LOCK — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/wpt-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell wpt-1 — save as docs/knowledge/patterns/windows-path-tests-wpt-1-pitfall.md

---
type: bee.pattern
title: windows-path-tests cell wpt-1 — pitfall candidate
description: "Pitfall candidate mined from cell wpt-1's capped trace: added docs_root_for_feature_matches_an_aliased_main_root and changed the paseo test dir to bin\\dir so Linux exercises both fixes — leader note asked for tests …"
timestamp: 2026-10-07
bee:
  id: windows-path-tests-wpt-1-pitfall
  lifecycle: draft
  areas: [worktree-parallelism]
  sources: [.bee/cells/wpt-1.json]
  polarity: pitfall
---

# windows-path-tests cell wpt-1 — pitfall candidate

## What the cell did

docs_root_for_feature uses roots::same_path; status_full test compares path via same_path; paseo test JSON-escapes path; all four CwdGuards recover a poisoned TEST_CWD_LOCK

## Recorded evidence (verbatim from .bee/cells/wpt-1.json)

- **deviation** — added docs_root_for_feature_matches_an_aliased_main_root and changed the paseo test dir to bin\dir so Linux exercises both fixes — leader note asked for tests that fail without the fix — found a better route
- **deviation** — sync-ack: path-compare and test-only fixes for Windows; no workflow behavior that a skill describes changed

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.