promote proposal for work item "herding-leader-toil" (docs/history/herding-leader-toil/CONTEXT.md + docs/history/herding-leader-toil/plan.md) — 7 capped cell(s): hlt-1, hlt-2, hlt-3, hlt-4, hlt-5, hlt-6, hlt-7
anchor: history — docs/history/herding-leader-toil/CONTEXT.md, docs/history/herding-leader-toil/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-leader-toil/delivery.md

---
type: bee.delivery
title: herding-leader-toil — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-leader-toil: 7 capped cell(s), 8 recorded deviation(s)."
timestamp: 2026-10-06
bee:
  id: herding-leader-toil-delivery
  lifecycle: active
  areas: [bee-herding, workflow-state, worktree-parallelism]
  required_context: [docs/history/herding-leader-toil/CONTEXT.md, docs/history/herding-leader-toil/plan.md]
  sources: [docs/history/herding-leader-toil/CONTEXT.md, docs/history/herding-leader-toil/plan.md, .bee/cells/hlt-1.json, .bee/cells/hlt-2.json, .bee/cells/hlt-3.json, .bee/cells/hlt-4.json, .bee/cells/hlt-5.json, .bee/cells/hlt-6.json, .bee/cells/hlt-7.json]
---

# herding-leader-toil — Delivery

## What shipped

- **hlt-1** — cells finish --from-job caps from the job with the leader proof verdict (7 file(s) changed)
- **hlt-2** — allow-listed control verbs serve main from a granted worktree (7 file(s) changed)
- **hlt-3** — cells judge-record --from-text records fenced verdicts (6 file(s) changed)
- **hlt-4** — the Paseo wait loop polls a paseo agent wait child and workers are titled by cell and agent (2 file(s) changed)
- **hlt-5** — opted-in Pi workers get their own Pi folder (4 file(s) changed)
- **hlt-6** — doctor paseo_ready catches the AppImage CLI and broken isolated Pi folders (2 file(s) changed)
- **hlt-7** — docs describe the shipped leader-toil changes (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hlt-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee cells && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee catalog` — cells 406 and catalog 15 tests green, run by the leader in the worktree; covers from-job caps, git fallback and every named refusal
- **hlt-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts` — whole bee bin 4101 passed 0 failed and registry_contracts 12 passed on d73d40256, after the judge revisions; run by the leader
- **hlt-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee cells && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee catalog && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding` — cells 406+ and catalog tests green incl. from-text extraction and refusals, and herding green after the run.rs one-liners; re-run by the leader after the judge revision c183c15d2
- **hlt-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding` — 741 herding tests green incl. the fake WaitSource cases, cancel on exit, titles and labels; run by the leader
- **hlt-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding` — herding tests green incl. isolated Pi folder creation, idempotence and env injection; run by the leader
- **hlt-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee doctor` — 57 doctor tests green incl. AppImage version output, wrapper script and isolated-folder checks; run by the leader
- **hlt-7** — `.bee/bin/bee knowledge check --json && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee no_shipped_command_spelling` — knowledge check in the worktree: 0 errors, and the documented-invocations test green inside the whole-bin run (4101 passed); the leader dropped the new comment lines the comment baseline refused

## Deviations

- **hlt-1** — verbs/knowledge/tests.rs gained the three new CapFlags fields so the crate compiles
- **hlt-1** — sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7
- **hlt-2** — sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7
- **hlt-3** — sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7
- **hlt-4** — sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7
- **hlt-5** — sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7
- **hlt-6** — sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7
- **hlt-7** — sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7

## Provenance

Proposed by `bee knowledge promote --work herding-leader-toil` from 7 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-leader-toil/CONTEXT.md`, `docs/history/herding-leader-toil/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-leader-toil" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-06T10:05:18.148Z), the work item declares no bee.areas.

area bee-herding:
  (no capped behavior_change cell exists for this feature)

area workflow-state:
  (no capped behavior_change cell exists for this feature)

area worktree-parallelism:
  (no capped behavior_change cell exists for this feature)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell hlt-1 — save as docs/knowledge/patterns/herding-leader-toil-hlt-1-pitfall.md

---
type: bee.pattern
title: herding-leader-toil cell hlt-1 — pitfall candidate
description: "Pitfall candidate mined from cell hlt-1's capped trace: verbs/knowledge/tests.rs gained the three new CapFlags fields so the crate compiles"
timestamp: 2026-10-06
bee:
  id: herding-leader-toil-hlt-1-pitfall
  lifecycle: draft
  areas: [bee-herding, workflow-state, worktree-parallelism]
  sources: [.bee/cells/hlt-1.json]
  polarity: pitfall
---

# herding-leader-toil cell hlt-1 — pitfall candidate

## What the cell did

cells finish --from-job caps from the job with the leader proof verdict

## Recorded evidence (verbatim from .bee/cells/hlt-1.json)

- **deviation** — verbs/knowledge/tests.rs gained the three new CapFlags fields so the crate compiles
- **deviation** — sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hlt-2 — save as docs/knowledge/patterns/herding-leader-toil-hlt-2-pitfall.md

---
type: bee.pattern
title: herding-leader-toil cell hlt-2 — pitfall candidate
description: "Pitfall candidate mined from cell hlt-2's capped trace: sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7"
timestamp: 2026-10-06
bee:
  id: herding-leader-toil-hlt-2-pitfall
  lifecycle: draft
  areas: [bee-herding, workflow-state, worktree-parallelism]
  sources: [.bee/cells/hlt-2.json]
  polarity: pitfall
---

# herding-leader-toil cell hlt-2 — pitfall candidate

## What the cell did

allow-listed control verbs serve main from a granted worktree

## Recorded evidence (verbatim from .bee/cells/hlt-2.json)

- **deviation** — sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7
- **failure_signature** — close promotion reads the worktree's .bee (no cells) whenever the feature has a granted worktree, from main too; most served state verbs do not name the main root

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hlt-3 — save as docs/knowledge/patterns/herding-leader-toil-hlt-3-pitfall.md

---
type: bee.pattern
title: herding-leader-toil cell hlt-3 — pitfall candidate
description: "Pitfall candidate mined from cell hlt-3's capped trace: sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7"
timestamp: 2026-10-06
bee:
  id: herding-leader-toil-hlt-3-pitfall
  lifecycle: draft
  areas: [bee-herding, workflow-state, worktree-parallelism]
  sources: [.bee/cells/hlt-3.json]
  polarity: pitfall
---

# herding-leader-toil cell hlt-3 — pitfall candidate

## What the cell did

cells judge-record --from-text records fenced verdicts

## Recorded evidence (verbatim from .bee/cells/hlt-3.json)

- **deviation** — sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7
- **failure_signature** — file_with_from_text_refuses can never fail and writes a timing row into the real main store

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hlt-4 — save as docs/knowledge/patterns/herding-leader-toil-hlt-4-pitfall.md

---
type: bee.pattern
title: herding-leader-toil cell hlt-4 — pitfall candidate
description: "Pitfall candidate mined from cell hlt-4's capped trace: sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7"
timestamp: 2026-10-06
bee:
  id: herding-leader-toil-hlt-4-pitfall
  lifecycle: draft
  areas: [bee-herding, workflow-state, worktree-parallelism]
  sources: [.bee/cells/hlt-4.json]
  polarity: pitfall
---

# herding-leader-toil cell hlt-4 — pitfall candidate

## What the cell did

the Paseo wait loop polls a paseo agent wait child and workers are titled by cell and agent

## Recorded evidence (verbatim from .bee/cells/hlt-4.json)

- **deviation** — sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hlt-5 — save as docs/knowledge/patterns/herding-leader-toil-hlt-5-pitfall.md

---
type: bee.pattern
title: herding-leader-toil cell hlt-5 — pitfall candidate
description: "Pitfall candidate mined from cell hlt-5's capped trace: sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7"
timestamp: 2026-10-06
bee:
  id: herding-leader-toil-hlt-5-pitfall
  lifecycle: draft
  areas: [bee-herding, workflow-state, worktree-parallelism]
  sources: [.bee/cells/hlt-5.json]
  polarity: pitfall
---

# herding-leader-toil cell hlt-5 — pitfall candidate

## What the cell did

opted-in Pi workers get their own Pi folder

## Recorded evidence (verbatim from .bee/cells/hlt-5.json)

- **deviation** — sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hlt-6 — save as docs/knowledge/patterns/herding-leader-toil-hlt-6-pitfall.md

---
type: bee.pattern
title: herding-leader-toil cell hlt-6 — pitfall candidate
description: "Pitfall candidate mined from cell hlt-6's capped trace: sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7"
timestamp: 2026-10-06
bee:
  id: herding-leader-toil-hlt-6-pitfall
  lifecycle: draft
  areas: [bee-herding, workflow-state, worktree-parallelism]
  sources: [.bee/cells/hlt-6.json]
  polarity: pitfall
---

# herding-leader-toil cell hlt-6 — pitfall candidate

## What the cell did

doctor paseo_ready catches the AppImage CLI and broken isolated Pi folders

## Recorded evidence (verbatim from .bee/cells/hlt-6.json)

- **deviation** — sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hlt-7 — save as docs/knowledge/patterns/herding-leader-toil-hlt-7-pitfall.md

---
type: bee.pattern
title: herding-leader-toil cell hlt-7 — pitfall candidate
description: "Pitfall candidate mined from cell hlt-7's capped trace: sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7"
timestamp: 2026-10-06
bee:
  id: herding-leader-toil-hlt-7-pitfall
  lifecycle: draft
  areas: [bee-herding, workflow-state, worktree-parallelism]
  sources: [.bee/cells/hlt-7.json]
  polarity: pitfall
---

# herding-leader-toil cell hlt-7 — pitfall candidate

## What the cell did

docs describe the shipped leader-toil changes

## Recorded evidence (verbatim from .bee/cells/hlt-7.json)

- **deviation** — sync-ack: herding, cells and doctor internals change no owned skill procedure; docs land in hlt-7

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 7 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 7 pattern candidate(s), 0 file(s) written.