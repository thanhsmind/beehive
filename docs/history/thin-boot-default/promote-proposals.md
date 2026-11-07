promote proposal for work item "thin-boot-default" (docs/history/thin-boot-default/plan.md) — 2 capped cell(s): tbd-1, tbd-2
anchor: history — docs/history/thin-boot-default/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/thin-boot-default/delivery.md

---
type: bee.delivery
title: thin-boot-default — delivery
description: "Delivery record proposed by bee knowledge promote for work item thin-boot-default: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-11-07
bee:
  id: thin-boot-default-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/thin-boot-default/plan.md]
  sources: [docs/history/thin-boot-default/plan.md, .bee/cells/tbd-1.json, .bee/cells/tbd-2.json]
---

# thin-boot-default — Delivery

## What shipped

- **tbd-1** — dispatch.thin_peer_boot is now on unless explicitly false, via one function used by both the hook and herding env paths; tests and thin-peer-boot concept added; commit 24facfc0d (5 file(s) changed)
- **tbd-2** — Hook now reads the thin switch from the control root's merged config, thin tests are hermetic, docs match the code; commit d5525f0f4f84f075c63e2be5bea553056d6b93f1 (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **tbd-1** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml thin_peer` — PATH="$HOME/.cargo/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml (with the thin and BEE_HERDING env vars unset) — green:unit — covers the switch rule, hook…
- **tbd-2** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — PATH="$HOME/.cargo/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml — green:unit — the full suite run inside this herded worker with no vars unset, exit 0, 52…

## Deviations

- **tbd-1** — sync-ack: skills/bee-herding covers cockpit roles only and never described the thin boot switch; the switch lives in the thin-peer-boot concept and config-reference
- **tbd-2** — sync-ack: skills/bee-herding covers cockpit roles only and never described the thin boot switch; the switch lives in the thin-peer-boot concept and config-reference

## Provenance

Proposed by `bee knowledge promote --work thin-boot-default` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/thin-boot-default/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "thin-boot-default" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-07T01:52:39.817Z), the work item declares no bee.areas.

area bee-herding:
  - [tbd-1] dispatch.thin_peer_boot is now on unless explicitly false, via one function used by both the hook and herding env paths; tests and thin-peer-boot concept added; commit 24facfc0d — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/tbd-1.json)
  - [tbd-2] Hook now reads the thin switch from the control root's merged config, thin tests are hermetic, docs match the code; commit d5525f0f4f84f075c63e2be5bea553056d6b93f1 — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/tbd-2.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell tbd-1 — save as docs/knowledge/patterns/thin-boot-default-tbd-1-pitfall.md

---
type: bee.pattern
title: thin-boot-default cell tbd-1 — pitfall candidate
description: "Pitfall candidate mined from cell tbd-1's capped trace: sync-ack: skills/bee-herding covers cockpit roles only and never described the thin boot switch; the switch lives in the thin-peer-boot concept and config-refe…"
timestamp: 2026-11-07
bee:
  id: thin-boot-default-tbd-1-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/tbd-1.json]
  polarity: pitfall
---

# thin-boot-default cell tbd-1 — pitfall candidate

## What the cell did

dispatch.thin_peer_boot is now on unless explicitly false, via one function used by both the hook and herding env paths; tests and thin-peer-boot concept added; commit 24facfc0d

## Recorded evidence (verbatim from .bee/cells/tbd-1.json)

- **deviation** — sync-ack: skills/bee-herding covers cockpit roles only and never described the thin boot switch; the switch lives in the thin-peer-boot concept and config-reference

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell tbd-2 — save as docs/knowledge/patterns/thin-boot-default-tbd-2-pitfall.md

---
type: bee.pattern
title: thin-boot-default cell tbd-2 — pitfall candidate
description: "Pitfall candidate mined from cell tbd-2's capped trace: sync-ack: skills/bee-herding covers cockpit roles only and never described the thin boot switch; the switch lives in the thin-peer-boot concept and config-refe…"
timestamp: 2026-11-07
bee:
  id: thin-boot-default-tbd-2-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/tbd-2.json]
  polarity: pitfall
---

# thin-boot-default cell tbd-2 — pitfall candidate

## What the cell did

Hook now reads the thin switch from the control root's merged config, thin tests are hermetic, docs match the code; commit d5525f0f4f84f075c63e2be5bea553056d6b93f1

## Recorded evidence (verbatim from .bee/cells/tbd-2.json)

- **deviation** — sync-ack: skills/bee-herding covers cockpit roles only and never described the thin boot switch; the switch lives in the thin-peer-boot concept and config-reference

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 2 pattern candidate(s), 0 file(s) written.