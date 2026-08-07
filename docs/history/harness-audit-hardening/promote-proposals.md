promote proposal for work item "harness-audit-hardening" (docs/history/harness-audit-hardening/plan.md) — 7 capped cell(s): hah-1, hah-2, hah-3, hah-4, hah-5, hah-6, hah-7
anchor: history — docs/history/harness-audit-hardening/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/harness-audit-hardening/delivery.md

---
type: bee.delivery
title: harness-audit-hardening — delivery
description: "Delivery record proposed by bee knowledge promote for work item harness-audit-hardening: 7 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-07
bee:
  id: harness-audit-hardening-delivery
  lifecycle: active
  required_context: [docs/history/harness-audit-hardening/plan.md]
  sources: [docs/history/harness-audit-hardening/plan.md, .bee/cells/hah-1.json, .bee/cells/hah-2.json, .bee/cells/hah-3.json, .bee/cells/hah-4.json, .bee/cells/hah-5.json, .bee/cells/hah-6.json, .bee/cells/hah-7.json]
---

# harness-audit-hardening — Delivery

## What shipped

- **hah-1** — Teach plugin-migration cleanup the post-R6 hook command spelling (1 file(s) changed)
- **hah-2** — Refuse the hooks merge on an unparseable host settings file instead of clobbering it (4 file(s) changed)
- **hah-3** — Ignore the vendored bee binary and .bak backups in the managed gitignore block (2 file(s) changed)
- **hah-4** — Render every declared flag in per-command text help, not only the required ones (1 file(s) changed)
- **hah-5** — Print hook usage instead of panicking on bare `bee hook` (1 file(s) changed)
- **hah-6** — Correct INSTALL.md and README.md claims that drifted from the shipped code (2 file(s) changed)
- **hah-7** — Repair wrong-home cross references in the AGENTS block and bee-hive/bee-swarming skill docs (14 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hah-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml plugin_distribution`
- **hah-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml onboard`
- **hah-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml onboard && git ls-files .claude | grep -v settings.json.bak`
- **hah-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml router catalog`
- **hah-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml hooks`
- **hah-6** — `rg -n 'GATE BYPASS ON|\.spikes/|15 bee|6 entries|node -e' README.md INSTALL.md returns nothing stale`
- **hah-7** — `rg -n 'routing-and-contracts' skills/bee-hive/references/go-mode.md README.md skills/bee-swarming/references/swarming-reference.md shows no stale table/contract pointer; regenerated AGENTS.md contains the fixed lines; bee dev release-manifest --check`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work harness-audit-hardening` from 7 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/harness-audit-hardening/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell hah-1 — save as docs/knowledge/patterns/harness-audit-hardening-hah-1-pitfall.md

---
type: bee.pattern
title: harness-audit-hardening cell hah-1 — pitfall candidate
description: "Pitfall candidate mined from cell hah-1's capped trace: 42a47aa4ab3a"
timestamp: 2026-08-07
bee:
  id: harness-audit-hardening-hah-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/hah-1.json]
  polarity: pitfall
---

# harness-audit-hardening cell hah-1 — pitfall candidate

## What the cell did

Teach plugin-migration cleanup the post-R6 hook command spelling

## Recorded evidence (verbatim from .bee/cells/hah-1.json)

- **failure_signature** — 42a47aa4ab3a

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hah-5 — save as docs/knowledge/patterns/harness-audit-hardening-hah-5-pitfall.md

---
type: bee.pattern
title: harness-audit-hardening cell hah-5 — pitfall candidate
description: "Pitfall candidate mined from cell hah-5's capped trace: f77f756b8e84"
timestamp: 2026-08-07
bee:
  id: harness-audit-hardening-hah-5-pitfall
  lifecycle: draft
  sources: [.bee/cells/hah-5.json]
  polarity: pitfall
---

# harness-audit-hardening cell hah-5 — pitfall candidate

## What the cell did

Print hook usage instead of panicking on bare `bee hook`

## Recorded evidence (verbatim from .bee/cells/hah-5.json)

- **failure_signature** — f77f756b8e84

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 7 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.