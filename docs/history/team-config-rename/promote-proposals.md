promote proposal for work item "team-config-rename" (docs/history/team-config-rename/CONTEXT.md + docs/history/team-config-rename/plan.md) — 4 capped cell(s): tcr-1, tcr-2, tcr-3, tcr-4
anchor: history — docs/history/team-config-rename/CONTEXT.md, docs/history/team-config-rename/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/team-config-rename/delivery.md

---
type: bee.delivery
title: team-config-rename — delivery
description: "Delivery record proposed by bee knowledge promote for work item team-config-rename: 4 capped cell(s), 5 recorded deviation(s)."
timestamp: 2026-09-09
bee:
  id: team-config-rename-delivery
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [docs/history/team-config-rename/CONTEXT.md, docs/history/team-config-rename/plan.md]
  sources: [docs/history/team-config-rename/CONTEXT.md, docs/history/team-config-rename/plan.md, .bee/cells/tcr-1.json, .bee/cells/tcr-2.json, .bee/cells/tcr-3.json, .bee/cells/tcr-4.json]
---

# team-config-rename — Delivery

## What shipped

- **tcr-1** — models folds into team once at config load; every production reader now reads team; the regression gate is empty (11 file(s) changed)
- **tcr-2** — fresh hosts are generated on the team key; the compiled-in sample and the stale-advisor warning moved with it (2 file(s) changed)
- **tcr-3** — every user-facing Rust string now says team.<runtime>; the preamble roster line, the guard refusal, the status problem messages, the addCell refusal and the porcelain line included (8 file(s) changed)
- **tcr-4** — bee team show is the verb; bee models show is a router alias with a stderr notice; doctor and the preamble each carry one legacy-key line (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **tcr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee -- drivers::models model_guard status_full && ! rg -n 'get\("models"\)' packages/bee-rs/crates/bee/src --glob '*.rs' | grep -v 'hooks/session_close/perf.rs' | grep -v 'verbs/status_full/build.rs:3[34][0-9]' | grep -v '/tests\|tests.rs\|#\[cfg(test)\]' | grep -q .`
- **tcr-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee -- onboard::templates && ! rg -q '"models"' .bee/config-sample.json`
- **tcr-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee && ! rg -n 'models\.(claude|codex|pi|opencode|<runtime>|<rt>|\{)' packages/bee-rs/crates/bee/src --glob '*.rs' | grep -v '/tests\|tests.rs' | grep -q .`
- **tcr-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee -- models_group router doctor session_preamble catalog`

## Deviations

- **tcr-1** — Touched onboard/templates.rs (one test assertion, rewritten from get("models") to contains_key so the regression gate stays empty) — tcr-2 file, edited after tcr-2 committed, no clobber.
- **tcr-1** — Capped with --sync-ack: wave.rs is bee-herding-owned code and its two-line key rename touches no bee-herding skill; the skill prose sweep is slice 2.
- **tcr-1** — sync-ack: wave.rs change is a two-line key rename (models to team) with behaviour preserved; no bee-herding skill text depends on the JSON key name, and the skill-side prose sweep of models.<runtime> is slice 2 (docs lane) per plan.md revision 2
- **tcr-3** — Capped with --sync-ack: string-only change on files an ownership map ties to bee-herding; the skill prose sweep is slice 2.
- **tcr-3** — sync-ack: string-only sweep of user-facing prose (models.<runtime> to team.<runtime>) across files an ownership map ties to bee-herding; no bee-herding skill text depends on those Rust strings, and the skill-side prose sweep of the same words is slice 2 (docs lane) per plan.md revision 2

## Provenance

Proposed by `bee knowledge promote --work team-config-rename` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/team-config-rename/CONTEXT.md`, `docs/history/team-config-rename/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "team-config-rename" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-09T04:40:09.257Z), the work item declares no bee.areas.

area doctrine-layer:
  - [tcr-2] fresh hosts are generated on the team key; the compiled-in sample and the stale-advisor warning moved with it — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/tcr-2.json)
  - [tcr-3] every user-facing Rust string now says team.<runtime>; the preamble roster line, the guard refusal, the status problem messages, the addCell refusal and the porcelain line included — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/tcr-3.json)
  - [tcr-4] bee team show is the verb; bee models show is a router alias with a stderr notice; doctor and the preamble each carry one legacy-key line — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/tcr-4.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell tcr-1 — save as docs/knowledge/patterns/team-config-rename-tcr-1-pitfall.md

---
type: bee.pattern
title: team-config-rename cell tcr-1 — pitfall candidate
description: "Pitfall candidate mined from cell tcr-1's capped trace: Touched onboard/templates.rs (one test assertion, rewritten from get(\"models\") to contains_key so the regression gate stays empty) — tcr-2 file, edited after t…"
timestamp: 2026-09-09
bee:
  id: team-config-rename-tcr-1-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/tcr-1.json]
  polarity: pitfall
---

# team-config-rename cell tcr-1 — pitfall candidate

## What the cell did

models folds into team once at config load; every production reader now reads team; the regression gate is empty

## Recorded evidence (verbatim from .bee/cells/tcr-1.json)

- **deviation** — Touched onboard/templates.rs (one test assertion, rewritten from get("models") to contains_key so the regression gate stays empty) — tcr-2 file, edited after tcr-2 committed, no clobber.
- **deviation** — Capped with --sync-ack: wave.rs is bee-herding-owned code and its two-line key rename touches no bee-herding skill; the skill prose sweep is slice 2.
- **deviation** — sync-ack: wave.rs change is a two-line key rename (models to team) with behaviour preserved; no bee-herding skill text depends on the JSON key name, and the skill-side prose sweep of models.<runtime> is slice 2 (docs lane) per plan.md revision 2

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell tcr-3 — save as docs/knowledge/patterns/team-config-rename-tcr-3-pitfall.md

---
type: bee.pattern
title: team-config-rename cell tcr-3 — pitfall candidate
description: "Pitfall candidate mined from cell tcr-3's capped trace: Capped with --sync-ack: string-only change on files an ownership map ties to bee-herding; the skill prose sweep is slice 2."
timestamp: 2026-09-09
bee:
  id: team-config-rename-tcr-3-pitfall
  lifecycle: draft
  areas: [doctrine-layer]
  sources: [.bee/cells/tcr-3.json]
  polarity: pitfall
---

# team-config-rename cell tcr-3 — pitfall candidate

## What the cell did

every user-facing Rust string now says team.<runtime>; the preamble roster line, the guard refusal, the status problem messages, the addCell refusal and the porcelain line included

## Recorded evidence (verbatim from .bee/cells/tcr-3.json)

- **deviation** — Capped with --sync-ack: string-only change on files an ownership map ties to bee-herding; the skill prose sweep is slice 2.
- **deviation** — sync-ack: string-only sweep of user-facing prose (models.<runtime> to team.<runtime>) across files an ownership map ties to bee-herding; no bee-herding skill text depends on those Rust strings, and the skill-side prose sweep of the same words is slice 2 (docs lane) per plan.md revision 2

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 3 area bullet(s), 2 pattern candidate(s), 0 file(s) written.