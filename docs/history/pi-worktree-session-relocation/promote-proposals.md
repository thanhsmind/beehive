promote proposal for work item "pi-worktree-session-relocation" (docs/history/pi-worktree-session-relocation/CONTEXT.md + docs/history/pi-worktree-session-relocation/plan.md) — 3 capped cell(s): pwsr-1, pwsr-2, pwsr-3
anchor: history — docs/history/pi-worktree-session-relocation/CONTEXT.md, docs/history/pi-worktree-session-relocation/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/pi-worktree-session-relocation/delivery.md

---
type: bee.delivery
title: pi-worktree-session-relocation — delivery
description: "Delivery record proposed by bee knowledge promote for work item pi-worktree-session-relocation: 3 capped cell(s), 1 recorded deviation(s)."
timestamp: 2026-09-07
bee:
  id: pi-worktree-session-relocation-delivery
  lifecycle: active
  areas: [worktree-parallelism, hook-runtime]
  required_context: [docs/history/pi-worktree-session-relocation/CONTEXT.md, docs/history/pi-worktree-session-relocation/plan.md]
  sources: [docs/history/pi-worktree-session-relocation/CONTEXT.md, docs/history/pi-worktree-session-relocation/plan.md, .bee/cells/pwsr-1.json, .bee/cells/pwsr-2.json, .bee/cells/pwsr-3.json]
---

# pi-worktree-session-relocation — Delivery

## What shipped

- **pwsr-1** — Emit verified worktree session-transition intent from the CLI (3 file(s) changed)
- **pwsr-2** — Relocate live Pi sessions across verified worktree boundaries (2 file(s) changed)
- **pwsr-3** — Document and verify Pi worktree session relocation end to end (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pwsr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::worktree && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_contracts --test registry_dispatch`
- **pwsr-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts`
- **pwsr-3** — `/home/thanhsmind/.cache/cargo-target/release/bee dev release-manifest --check && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **pwsr-3** — release-manifest regeneration moved into pwsr-3 — pwsr-2 used the approved wave-barrier acknowledgment for its shipped Pi extension change — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work pi-worktree-session-relocation` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/pi-worktree-session-relocation/CONTEXT.md`, `docs/history/pi-worktree-session-relocation/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "pi-worktree-session-relocation" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-07T15:42:25.708Z), the work item declares no bee.areas.

area worktree-parallelism:
  - [pwsr-1] Emit verified worktree session-transition intent from the CLI — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/pwsr-1.json)
  - [pwsr-2] Relocate live Pi sessions across verified worktree boundaries — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pwsr-2.json)

area hook-runtime:
  - [pwsr-1] Emit verified worktree session-transition intent from the CLI — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/pwsr-1.json)
  - [pwsr-2] Relocate live Pi sessions across verified worktree boundaries — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/pwsr-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pwsr-1 — save as docs/knowledge/patterns/pi-worktree-session-relocation-pwsr-1-pitfall.md

---
type: bee.pattern
title: pi-worktree-session-relocation cell pwsr-1 — pitfall candidate
description: "Pitfall candidate mined from cell pwsr-1's capped trace: afa9bbd466a8"
timestamp: 2026-09-07
bee:
  id: pi-worktree-session-relocation-pwsr-1-pitfall
  lifecycle: draft
  areas: [worktree-parallelism, hook-runtime]
  sources: [.bee/cells/pwsr-1.json]
  polarity: pitfall
---

# pi-worktree-session-relocation cell pwsr-1 — pitfall candidate

## What the cell did

Emit verified worktree session-transition intent from the CLI

## Recorded evidence (verbatim from .bee/cells/pwsr-1.json)

- **failure_signature** — afa9bbd466a8
- **failure_signature** — 512d9b996700
- **failure_signature** — d1b0ef9af4ad
- **failure_signature** — 43b2b2aa7b8f
- **failure_signature** — b197674a0f23

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pwsr-2 — save as docs/knowledge/patterns/pi-worktree-session-relocation-pwsr-2-pitfall.md

---
type: bee.pattern
title: pi-worktree-session-relocation cell pwsr-2 — pitfall candidate
description: "Pitfall candidate mined from cell pwsr-2's capped trace: 8199edad1918"
timestamp: 2026-09-07
bee:
  id: pi-worktree-session-relocation-pwsr-2-pitfall
  lifecycle: draft
  areas: [worktree-parallelism, hook-runtime]
  sources: [.bee/cells/pwsr-2.json]
  polarity: pitfall
---

# pi-worktree-session-relocation cell pwsr-2 — pitfall candidate

## What the cell did

Relocate live Pi sessions across verified worktree boundaries

## Recorded evidence (verbatim from .bee/cells/pwsr-2.json)

- **failure_signature** — 8199edad1918
- **failure_signature** — c100994fdd51
- **failure_signature** — de146ceab37a
- **failure_signature** — 30e6a201ae6e

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pwsr-3 — save as docs/knowledge/patterns/pi-worktree-session-relocation-pwsr-3-pitfall.md

---
type: bee.pattern
title: pi-worktree-session-relocation cell pwsr-3 — pitfall candidate
description: "Pitfall candidate mined from cell pwsr-3's capped trace: release-manifest regeneration moved into pwsr-3 — pwsr-2 used the approved wave-barrier acknowledgment for its shipped Pi extension change — something else had…"
timestamp: 2026-09-07
bee:
  id: pi-worktree-session-relocation-pwsr-3-pitfall
  lifecycle: draft
  areas: [worktree-parallelism, hook-runtime]
  sources: [.bee/cells/pwsr-3.json]
  polarity: pitfall
---

# pi-worktree-session-relocation cell pwsr-3 — pitfall candidate

## What the cell did

Document and verify Pi worktree session relocation end to end

## Recorded evidence (verbatim from .bee/cells/pwsr-3.json)

- **deviation** — release-manifest regeneration moved into pwsr-3 — pwsr-2 used the approved wave-barrier acknowledgment for its shipped Pi extension change — something else had to be fixed first
- **failure_signature** — 82389d554029

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 3 pattern candidate(s), 0 file(s) written.