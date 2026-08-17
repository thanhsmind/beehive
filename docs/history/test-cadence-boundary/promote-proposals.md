promote proposal for work item "test-cadence-boundary" (docs/history/test-cadence-boundary/CONTEXT.md + docs/history/test-cadence-boundary/plan.md) — 4 capped cell(s): tcb-1, tcb-2, tcb-3, tcb-4
anchor: history — docs/history/test-cadence-boundary/CONTEXT.md, docs/history/test-cadence-boundary/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/test-cadence-boundary/delivery.md

---
type: bee.delivery
title: test-cadence-boundary — delivery
description: "Delivery record proposed by bee knowledge promote for work item test-cadence-boundary: 4 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-08-17
bee:
  id: test-cadence-boundary-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/test-cadence-boundary/CONTEXT.md, docs/history/test-cadence-boundary/plan.md]
  sources: [docs/history/test-cadence-boundary/CONTEXT.md, docs/history/test-cadence-boundary/plan.md, .bee/cells/tcb-1.json, .bee/cells/tcb-2.json, .bee/cells/tcb-3.json, .bee/cells/tcb-4.json]
---

# test-cadence-boundary — Delivery

## What shipped

- **tcb-1** — cap stops running tests; cap records tests: boundary; --report tests validation accepts boundary/undeclared, refuses green/red with a teach line (4 file(s) changed)
- **tcb-2** — close defers the tests door to bee worktree merge when a granted worktree exists; no worktree still runs fresh, red stops close (3 file(s) changed)
- **tcb-3** — Synced every instruction surface to the boundary-only test cadence wording (D1) (20 file(s) changed)
- **tcb-4** — Reworded shipped cells finish/cap help to boundary cadence (no per-cap test run/red-refusal promise); registry_payload.json has no regen source, hand-edited per precedent (ef7187cd) (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **tcb-1** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **tcb-2** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **tcb-3** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **tcb-4** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **tcb-3** — Reserved .bee/config-sample.json (an additional path outside the cell's declared files list, named in the cell action text) after writing it, not before

## Provenance

Proposed by `bee knowledge promote --work test-cadence-boundary` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/test-cadence-boundary/CONTEXT.md`, `docs/history/test-cadence-boundary/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "test-cadence-boundary" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-17T11:34:50.777Z), the work item declares no bee.areas.

area workflow-state:
  - [tcb-1] cap stops running tests; cap records tests: boundary; --report tests validation accepts boundary/undeclared, refuses green/red with a teach line — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/tcb-1.json)
  - [tcb-2] close defers the tests door to bee worktree merge when a granted worktree exists; no worktree still runs fresh, red stops close — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/tcb-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell tcb-1 — save as docs/knowledge/patterns/test-cadence-boundary-tcb-1-pitfall.md

---
type: bee.pattern
title: test-cadence-boundary cell tcb-1 — pitfall candidate
description: "Pitfall candidate mined from cell tcb-1's capped trace: 5ad24de89c82"
timestamp: 2026-08-17
bee:
  id: test-cadence-boundary-tcb-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/tcb-1.json]
  polarity: pitfall
---

# test-cadence-boundary cell tcb-1 — pitfall candidate

## What the cell did

cap stops running tests; cap records tests: boundary; --report tests validation accepts boundary/undeclared, refuses green/red with a teach line

## Recorded evidence (verbatim from .bee/cells/tcb-1.json)

- **failure_signature** — 5ad24de89c82

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell tcb-2 — save as docs/knowledge/patterns/test-cadence-boundary-tcb-2-pitfall.md

---
type: bee.pattern
title: test-cadence-boundary cell tcb-2 — pitfall candidate
description: "Pitfall candidate mined from cell tcb-2's capped trace: b752a32b1bcd"
timestamp: 2026-08-17
bee:
  id: test-cadence-boundary-tcb-2-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/tcb-2.json]
  polarity: pitfall
---

# test-cadence-boundary cell tcb-2 — pitfall candidate

## What the cell did

close defers the tests door to bee worktree merge when a granted worktree exists; no worktree still runs fresh, red stops close

## Recorded evidence (verbatim from .bee/cells/tcb-2.json)

- **failure_signature** — b752a32b1bcd
- **failure_signature** — f298807a1f5c
- **failure_signature** — f298807a1f5c
- **failure_signature** — f298807a1f5c
- **failure_signature** — f298807a1f5c
- **failure_signature** — 5ad24de89c82
- **failure_signature** — 5ad24de89c82
- **failure_signature** — 5ad24de89c82
- **failure_signature** — 5ad24de89c82
- **failure_signature** — 5ad24de89c82
- **failure_signature** — ff8e7c67b6eb

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell tcb-3 — save as docs/knowledge/patterns/test-cadence-boundary-tcb-3-pitfall.md

---
type: bee.pattern
title: test-cadence-boundary cell tcb-3 — pitfall candidate
description: "Pitfall candidate mined from cell tcb-3's capped trace: Reserved .bee/config-sample.json (an additional path outside the cell's declared files list, named in the cell action text) after writing it, not before"
timestamp: 2026-08-17
bee:
  id: test-cadence-boundary-tcb-3-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/tcb-3.json]
  polarity: pitfall
---

# test-cadence-boundary cell tcb-3 — pitfall candidate

## What the cell did

Synced every instruction surface to the boundary-only test cadence wording (D1)

## Recorded evidence (verbatim from .bee/cells/tcb-3.json)

- **deviation** — Reserved .bee/config-sample.json (an additional path outside the cell's declared files list, named in the cell action text) after writing it, not before

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 3 pattern candidate(s), 0 file(s) written.