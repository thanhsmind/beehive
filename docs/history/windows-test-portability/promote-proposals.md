promote proposal for work item "windows-test-portability" (docs/history/windows-test-portability/plan.md) — 2 capped cell(s): wtp-1, wtp-2
anchor: history — docs/history/windows-test-portability/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/windows-test-portability/delivery.md

---
type: bee.delivery
title: windows-test-portability — delivery
description: "Delivery record proposed by bee knowledge promote for work item windows-test-portability: 2 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-09-11
bee:
  id: windows-test-portability-delivery
  lifecycle: active
  required_context: [docs/history/windows-test-portability/plan.md]
  sources: [docs/history/windows-test-portability/plan.md, .bee/cells/wtp-1.json, .bee/cells/wtp-2.json]
---

# windows-test-portability — Delivery

## What shipped

- **wtp-1** — Onboard statusline tests pinned to a POSIX host via posix_fixture() in onboard/tests.rs; the two empty-repo tests branch on merge::host_shell_is_powershell (1 file(s) changed)
- **wtp-2** — prompt-skew path check is separator-neutral (verbs/drivers/tests.rs:4739); enter-worktree compares dunce-canonical paths (verbs/worktree/tests.rs:5987) and the text check reuses the printed worktreeRoot (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wtp-1** — `cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee onboard::`
- **wtp-2** — `cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee prompt_skew && cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee enter_worktree_core`

## Deviations

- **wtp-1** — posix_fixture() also covers the 8 other section tests that were not failing (user-level entry, unparseable file, --no-statusline, etc.) — the cell said pin every fixture repo in the section, and without the pin those tests assert the PowerShell skip on Windows instead of the display — found a better route
- **wtp-1** — the_project_config_opt_out_behaves_exactly_like_the_flag keeps fixture() and writes host_shell posix plus statusline false in its own config, as the cell asked — followed the plan
- **wtp-2** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work windows-test-portability` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/windows-test-portability/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell wtp-1 — save as docs/knowledge/patterns/windows-test-portability-wtp-1-pitfall.md

---
type: bee.pattern
title: windows-test-portability cell wtp-1 — pitfall candidate
description: "Pitfall candidate mined from cell wtp-1's capped trace: posix_fixture() also covers the 8 other section tests that were not failing (user-level entry, unparseable file, --no-statusline, etc.) — the cell said pin eve…"
timestamp: 2026-09-11
bee:
  id: windows-test-portability-wtp-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/wtp-1.json]
  polarity: pitfall
---

# windows-test-portability cell wtp-1 — pitfall candidate

## What the cell did

Onboard statusline tests pinned to a POSIX host via posix_fixture() in onboard/tests.rs; the two empty-repo tests branch on merge::host_shell_is_powershell

## Recorded evidence (verbatim from .bee/cells/wtp-1.json)

- **deviation** — posix_fixture() also covers the 8 other section tests that were not failing (user-level entry, unparseable file, --no-statusline, etc.) — the cell said pin every fixture repo in the section, and without the pin those tests assert the PowerShell skip on Windows instead of the display — found a better route
- **deviation** — the_project_config_opt_out_behaves_exactly_like_the_flag keeps fixture() and writes host_shell posix plus statusline false in its own config, as the cell asked — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell wtp-2 — save as docs/knowledge/patterns/windows-test-portability-wtp-2-pitfall.md

---
type: bee.pattern
title: windows-test-portability cell wtp-2 — pitfall candidate
description: "Pitfall candidate mined from cell wtp-2's capped trace: followed the plan"
timestamp: 2026-09-11
bee:
  id: windows-test-portability-wtp-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/wtp-2.json]
  polarity: pitfall
---

# windows-test-portability cell wtp-2 — pitfall candidate

## What the cell did

prompt-skew path check is separator-neutral (verbs/drivers/tests.rs:4739); enter-worktree compares dunce-canonical paths (verbs/worktree/tests.rs:5987) and the text check reuses the printed worktreeRoot

## Recorded evidence (verbatim from .bee/cells/wtp-2.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.