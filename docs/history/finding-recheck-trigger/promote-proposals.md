promote proposal for work item "finding-recheck-trigger" (docs/history/finding-recheck-trigger/plan.md) — 3 capped cell(s): frt-1, frt-2, frt-3
anchor: history — docs/history/finding-recheck-trigger/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/finding-recheck-trigger/delivery.md

---
type: bee.delivery
title: finding-recheck-trigger — delivery
description: "Delivery record proposed by bee knowledge promote for work item finding-recheck-trigger: 3 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-09-21
bee:
  id: finding-recheck-trigger-delivery
  lifecycle: active
  required_context: [docs/history/finding-recheck-trigger/plan.md]
  sources: [docs/history/finding-recheck-trigger/plan.md, .bee/cells/frt-1.json, .bee/cells/frt-2.json, .bee/cells/frt-3.json]
---

# finding-recheck-trigger — Delivery

## What shipped

- **frt-1** — path-changed predicate with HEAD anchor and HEAD-cached prompt count (2 file(s) changed)
- **frt-2** — Per-prompt reminder prints 'triggers due: N — bee triggers list --due' while predicate triggers are due; unchanged at zero (1 file(s) changed)
- **frt-3** — bee-reviewing Finish registers open findings as path-changed triggers; routing-and-contracts points at triggers list --due (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **frt-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee triggers && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts --test registry_dispatch` — cell verify, 17 triggers tests including 8 new path-changed, branch-join and prompt-cache tests seen by name, registry 12 and 9
- **frt-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee prompt_context` — the cell verify filter; 35 passed incl. 4 new tests (a_due_predicate_trigger_adds_the_due_line_and_changes_the_hash, the_due_line_survives_a_full_three_line_reminder, a_manual_waiting_trigger_adds_no…
- **frt-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pointer_integrity --test rule_index_parity && .bee/bin/bee dev release-manifest --check` — docs-only skill edit; pointer/rule-index parity (10+3 passed) and manifest check (376 files match)

## Deviations

- **frt-1** — moved run_add validation, anchoring and the write into a testable add_record that returns the refusal line — run_add reads cwd so tests could not drive it — found a better route
- **frt-1** — due_count_for_prompt returns 0 and writes nothing when .bee/triggers does not exist — avoids creating the store dir on every prompt — found a better route
- **frt-1** — due_count_for_prompt has no caller yet so the build warns it is unused until the prompt-line cell wires it — the cell scope stops at the function — something else had to be fixed first
- **frt-2** — followed the plan; added one extra test for the 4-line cap — the cap rule needed its own proof — found a better route
- **frt-2** — sync-ack: hook-only change to the per-prompt reminder; the feature's reviewing doctrine skill edit is its own planned cell, outside frt-2 files
- **frt-3** — left .bee/onboarding.json (regen agents_sync delta from the earlier opus role commit) uncommitted — it is outside the cell files and unrelated to this edit — something else had to be fixed first
- **frt-3** — committed the rendered mirrors under the five rendered skill trees — the action requires them though the files list names only sources — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work finding-recheck-trigger` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/finding-recheck-trigger/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell frt-1 — save as docs/knowledge/patterns/finding-recheck-trigger-frt-1-pitfall.md

---
type: bee.pattern
title: finding-recheck-trigger cell frt-1 — pitfall candidate
description: "Pitfall candidate mined from cell frt-1's capped trace: moved run_add validation, anchoring and the write into a testable add_record that returns the refusal line — run_add reads cwd so tests could not drive it — fo…"
timestamp: 2026-09-21
bee:
  id: finding-recheck-trigger-frt-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/frt-1.json]
  polarity: pitfall
---

# finding-recheck-trigger cell frt-1 — pitfall candidate

## What the cell did

path-changed predicate with HEAD anchor and HEAD-cached prompt count

## Recorded evidence (verbatim from .bee/cells/frt-1.json)

- **deviation** — moved run_add validation, anchoring and the write into a testable add_record that returns the refusal line — run_add reads cwd so tests could not drive it — found a better route
- **deviation** — due_count_for_prompt returns 0 and writes nothing when .bee/triggers does not exist — avoids creating the store dir on every prompt — found a better route
- **deviation** — due_count_for_prompt has no caller yet so the build warns it is unused until the prompt-line cell wires it — the cell scope stops at the function — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell frt-2 — save as docs/knowledge/patterns/finding-recheck-trigger-frt-2-pitfall.md

---
type: bee.pattern
title: finding-recheck-trigger cell frt-2 — pitfall candidate
description: "Pitfall candidate mined from cell frt-2's capped trace: followed the plan; added one extra test for the 4-line cap — the cap rule needed its own proof — found a better route"
timestamp: 2026-09-21
bee:
  id: finding-recheck-trigger-frt-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/frt-2.json]
  polarity: pitfall
---

# finding-recheck-trigger cell frt-2 — pitfall candidate

## What the cell did

Per-prompt reminder prints 'triggers due: N — bee triggers list --due' while predicate triggers are due; unchanged at zero

## Recorded evidence (verbatim from .bee/cells/frt-2.json)

- **deviation** — followed the plan; added one extra test for the 4-line cap — the cap rule needed its own proof — found a better route
- **deviation** — sync-ack: hook-only change to the per-prompt reminder; the feature's reviewing doctrine skill edit is its own planned cell, outside frt-2 files

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell frt-3 — save as docs/knowledge/patterns/finding-recheck-trigger-frt-3-pitfall.md

---
type: bee.pattern
title: finding-recheck-trigger cell frt-3 — pitfall candidate
description: "Pitfall candidate mined from cell frt-3's capped trace: left .bee/onboarding.json (regen agents_sync delta from the earlier opus role commit) uncommitted — it is outside the cell files and unrelated to this edit — s…"
timestamp: 2026-09-21
bee:
  id: finding-recheck-trigger-frt-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/frt-3.json]
  polarity: pitfall
---

# finding-recheck-trigger cell frt-3 — pitfall candidate

## What the cell did

bee-reviewing Finish registers open findings as path-changed triggers; routing-and-contracts points at triggers list --due

## Recorded evidence (verbatim from .bee/cells/frt-3.json)

- **deviation** — left .bee/onboarding.json (regen agents_sync delta from the earlier opus role commit) uncommitted — it is outside the cell files and unrelated to this edit — something else had to be fixed first
- **deviation** — committed the rendered mirrors under the five rendered skill trees — the action requires them though the files list names only sources — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 3 pattern candidate(s), 0 file(s) written.