promote proposal for work item "verified-claims-triage" (.bee/lanes/verified-claims-triage.json) — 2 capped cell(s): vct-1, vct-2
anchor: ledger — .bee/lanes/verified-claims-triage.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/verified-claims-triage/delivery.md

---
type: bee.delivery
title: verified-claims-triage — delivery
description: "Delivery record proposed by bee knowledge promote for work item verified-claims-triage: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-11-07
bee:
  id: verified-claims-triage-delivery
  lifecycle: active
  required_context: [.bee/lanes/verified-claims-triage.json]
  sources: [.bee/lanes/verified-claims-triage.json, .bee/cells/vct-1.json, .bee/cells/vct-2.json]
---

# verified-claims-triage — Delivery

## What shipped

- **vct-1** — Added agents-verify-before-agreeing to LEADER.md, the two reference subsections and the rule index row; commit f2d73eaa1c452fff57cd345ebc7181cbaea2d664 (4 file(s) changed)
- **vct-2** — Stop hook now warns when the last turn agrees without a [verified] label (advisory only); claims.rs, mod.rs wiring, tests, LEADER.md and hook-runtime overview done, commit 16356d679 (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **vct-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity --test pointer_integrity` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity --test pointer_integrity — green:static — docs-only change; the p…
- **vct-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee session_close && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity --test pointer_integrity` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml — green:unit — full suite exit 0; session_close 79 passed, rule_index_parit…

## Deviations

- **vct-1** — sync-ack: added a new rule block in LEADER.md; agents-capture-line-at-close text unchanged, its applied_at files need no update
- **vct-2** — sync-ack: added one sentence inside the new agents-verify-before-agreeing block; other rules in LEADER.md unchanged

## Provenance

Proposed by `bee knowledge promote --work verified-claims-triage` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/verified-claims-triage.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell vct-1 — save as docs/knowledge/patterns/verified-claims-triage-vct-1-pitfall.md

---
type: bee.pattern
title: verified-claims-triage cell vct-1 — pitfall candidate
description: "Pitfall candidate mined from cell vct-1's capped trace: sync-ack: added a new rule block in LEADER.md; agents-capture-line-at-close text unchanged, its applied_at files need no update"
timestamp: 2026-11-07
bee:
  id: verified-claims-triage-vct-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/vct-1.json]
  polarity: pitfall
---

# verified-claims-triage cell vct-1 — pitfall candidate

## What the cell did

Added agents-verify-before-agreeing to LEADER.md, the two reference subsections and the rule index row; commit f2d73eaa1c452fff57cd345ebc7181cbaea2d664

## Recorded evidence (verbatim from .bee/cells/vct-1.json)

- **deviation** — sync-ack: added a new rule block in LEADER.md; agents-capture-line-at-close text unchanged, its applied_at files need no update

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell vct-2 — save as docs/knowledge/patterns/verified-claims-triage-vct-2-pitfall.md

---
type: bee.pattern
title: verified-claims-triage cell vct-2 — pitfall candidate
description: "Pitfall candidate mined from cell vct-2's capped trace: sync-ack: added one sentence inside the new agents-verify-before-agreeing block; other rules in LEADER.md unchanged"
timestamp: 2026-11-07
bee:
  id: verified-claims-triage-vct-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/vct-2.json]
  polarity: pitfall
---

# verified-claims-triage cell vct-2 — pitfall candidate

## What the cell did

Stop hook now warns when the last turn agrees without a [verified] label (advisory only); claims.rs, mod.rs wiring, tests, LEADER.md and hook-runtime overview done, commit 16356d679

## Recorded evidence (verbatim from .bee/cells/vct-2.json)

- **deviation** — sync-ack: added one sentence inside the new agents-verify-before-agreeing block; other rules in LEADER.md unchanged

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.