promote proposal for work item "anti-pattern-principles" (docs/history/anti-pattern-principles/CONTEXT.md) — 1 capped cell(s): app-1
anchor: history — docs/history/anti-pattern-principles/CONTEXT.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/anti-pattern-principles/delivery.md

---
type: bee.delivery
title: anti-pattern-principles — delivery
description: "Delivery record proposed by bee knowledge promote for work item anti-pattern-principles: 1 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-11-04
bee:
  id: anti-pattern-principles-delivery
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [docs/history/anti-pattern-principles/CONTEXT.md]
  sources: [docs/history/anti-pattern-principles/CONTEXT.md, .bee/cells/app-1.json]
---

# anti-pattern-principles — Delivery

## What shipped

- **app-1** — bee orient lists lock-the-contract and split-on-a-real-dependency for feature, refactor and perf routes, each skill pointing at its own heading in expertise/planning.md (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **app-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test principle_index_parity --test skill_triggers --test pointer_integrity && .bee/bin/bee dev release-manifest --check` — principle_index_parity 5/5, pointer_integrity 10/10, skill_triggers 1 ok and 1 ignored (real-agent eval not run), release-manifest 599 files match; bee orient driven in a scratch repo with route clas…

## Deviations

- **app-1** — Ran the verify cargo step with the absolute path /home/thanhsmind/.cargo/bin/cargo instead of the CARGO_HOME PATH prefix — the worktree shell guard refused the runtime variable — hit an unforeseen obstacle
- **app-1** — Changed one Spike craft phrase from The section above to Build order versus discovery order — the new Split only on a real dependency heading made The section above point at the wrong section — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work anti-pattern-principles` from 1 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/anti-pattern-principles/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "anti-pattern-principles" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-04T04:11:05.782Z), the work item declares no bee.areas.

area doctrine-layer:
  - [app-1] bee orient lists lock-the-contract and split-on-a-real-dependency for feature, refactor and perf routes, each skill pointing at its own heading in expertise/planning.md — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/app-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell app-1 — save as docs/knowledge/patterns/anti-pattern-principles-app-1-pitfall.md

---
type: bee.pattern
title: anti-pattern-principles cell app-1 — pitfall candidate
description: "Pitfall candidate mined from cell app-1's capped trace: Ran the verify cargo step with the absolute path /home/thanhsmind/.cargo/bin/cargo instead of the CARGO_HOME PATH prefix — the worktree shell guard refused the…"
timestamp: 2026-11-04
bee:
  id: anti-pattern-principles-app-1-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/app-1.json]
  polarity: pitfall
---

# anti-pattern-principles cell app-1 — pitfall candidate

## What the cell did

bee orient lists lock-the-contract and split-on-a-real-dependency for feature, refactor and perf routes, each skill pointing at its own heading in expertise/planning.md

## Recorded evidence (verbatim from .bee/cells/app-1.json)

- **deviation** — Ran the verify cargo step with the absolute path /home/thanhsmind/.cargo/bin/cargo instead of the CARGO_HOME PATH prefix — the worktree shell guard refused the runtime variable — hit an unforeseen obstacle
- **deviation** — Changed one Spike craft phrase from The section above to Build order versus discovery order — the new Split only on a real dependency heading made The section above point at the wrong section — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 1 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 1 pattern candidate(s), 0 file(s) written.