promote proposal for work item "codex-parity-completion" (docs/history/codex-parity-completion/CONTEXT.md + docs/history/codex-parity-completion/plan.md) — 4 capped cell(s): cpc-1, cpc-2, cpc-3, cpc-4
anchor: history — docs/history/codex-parity-completion/CONTEXT.md, docs/history/codex-parity-completion/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/codex-parity-completion/delivery.md

---
type: bee.delivery
title: codex-parity-completion — delivery
description: "Delivery record proposed by bee knowledge promote for work item codex-parity-completion: 4 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-09-13
bee:
  id: codex-parity-completion-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/codex-parity-completion/CONTEXT.md, docs/history/codex-parity-completion/plan.md]
  sources: [docs/history/codex-parity-completion/CONTEXT.md, docs/history/codex-parity-completion/plan.md, .bee/cells/cpc-1.json, .bee/cells/cpc-2.json, .bee/cells/cpc-3.json, .bee/cells/cpc-4.json]
---

# codex-parity-completion — Delivery

## What shipped

- **cpc-1** — Installed Codex hooks protect observed native tools and retain real runtime evidence. (10 file(s) changed)
- **cpc-2** — Enforce configured Codex dispatch settings, bounded hermetic probes, exact observed native refusal, and actual prepared read-only filesystem boundary. (6 file(s) changed)
- **cpc-3** — Normalize native Codex transcript usage, identity and final text with Claude compatibility. (5 file(s) changed)
- **cpc-4** — Publish verified Codex runtime limits, dispatch instructions and executable sandbox recipe. (12 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **cpc-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test hook_contracts && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee hook_manifests && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee hooks_wiring` — 13/7/23 passed with TMPDIR=/var/tmp in docs/history/codex-parity-completion/cpc-1-checks.log; additional real installed canary exited 0 at /var/tmp/bee-codex-canary-ZPuEAI/evidence: patch/shell deny …
- **cpc-2** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee model_guard && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee verbs::drivers` — Leader ran exact groups under TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false: 70 and 382 passed, 8 ignored. Final amendment comment-only. Additional green:live actual prepared command/stdin in cpc-2-…
- **cpc-3** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee session_close && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee hooks::activity` — 50 session_close and 54 activity tests; leader rerun exited 0; real-binary fixture wait subject and session checked at /tmp/bee-verify/run/20260912-215549-1244378/repo/.bee/state.json
- **cpc-4** — `git diff --check` — final instruction scenarios passed in job-1789272594951-3409518-1; actual prepared a3600d96-c439-433d-90dc-4186da0994fd executed through documented bash-c helper: raw read-only denials both status1, …

## Deviations

- **cpc-1** — Use private authentication and deny opaque native dispatch — live input exposed inaccessible role metadata — hit an unforeseen obstacle
- **cpc-1** — Consolidate recovery commits at wave close — sibling commits intervene — found a better route
- **cpc-2** — Existing windows-sys Pipes feature added under decision 7741c278; no new crate. Separate recovery commit retained until wave-close consolidation because sibling commits intervene.
- **cpc-2** — Observed Codex 0.154.0 opaque native messages require named refusal; explicit configured external routes retained; no full native parity claim.
- **cpc-3** — Used path-scoped revision commits pending wave-close consolidation — a sibling commit separated the original cell commit from revisions — hit an unforeseen obstacle
- **cpc-3** — Classified the direct session-close fixture separately from live Codex execution — installed native hook proof belongs to the running cpc-1 canary — found a better route
- **cpc-4** — Leader owns wave-barrier regeneration and integration proof; worker amended cpc-4 through user-approved bounded revisions.

## Provenance

Proposed by `bee knowledge promote --work codex-parity-completion` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/codex-parity-completion/CONTEXT.md`, `docs/history/codex-parity-completion/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "codex-parity-completion" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-13T04:29:50.670Z), the work item declares no bee.areas.

area hook-runtime:
  - [cpc-1] Installed Codex hooks protect observed native tools and retain real runtime evidence. — feature-wide sync per the scribing stamp, 10 file(s) changed (trace .bee/cells/cpc-1.json)
  - [cpc-2] Enforce configured Codex dispatch settings, bounded hermetic probes, exact observed native refusal, and actual prepared read-only filesystem boundary. — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/cpc-2.json)
  - [cpc-3] Normalize native Codex transcript usage, identity and final text with Claude compatibility. — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/cpc-3.json)
  - [cpc-4] Publish verified Codex runtime limits, dispatch instructions and executable sandbox recipe. — feature-wide sync per the scribing stamp, 12 file(s) changed (trace .bee/cells/cpc-4.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell cpc-1 — save as docs/knowledge/patterns/codex-parity-completion-cpc-1-pitfall.md

---
type: bee.pattern
title: codex-parity-completion cell cpc-1 — pitfall candidate
description: "Pitfall candidate mined from cell cpc-1's capped trace: Use private authentication and deny opaque native dispatch — live input exposed inaccessible role metadata — hit an unforeseen obstacle"
timestamp: 2026-09-13
bee:
  id: codex-parity-completion-cpc-1-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/cpc-1.json]
  polarity: pitfall
---

# codex-parity-completion cell cpc-1 — pitfall candidate

## What the cell did

Installed Codex hooks protect observed native tools and retain real runtime evidence.

## Recorded evidence (verbatim from .bee/cells/cpc-1.json)

- **deviation** — Use private authentication and deny opaque native dispatch — live input exposed inaccessible role metadata — hit an unforeseen obstacle
- **deviation** — Consolidate recovery commits at wave close — sibling commits intervene — found a better route
- **failure_signature** — ec31dcdf2825

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell cpc-2 — save as docs/knowledge/patterns/codex-parity-completion-cpc-2-pitfall.md

---
type: bee.pattern
title: codex-parity-completion cell cpc-2 — pitfall candidate
description: "Pitfall candidate mined from cell cpc-2's capped trace: Existing windows-sys Pipes feature added under decision 7741c278; no new crate. Separate recovery commit retained until wave-close consolidation because siblin…"
timestamp: 2026-09-13
bee:
  id: codex-parity-completion-cpc-2-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/cpc-2.json]
  polarity: pitfall
---

# codex-parity-completion cell cpc-2 — pitfall candidate

## What the cell did

Enforce configured Codex dispatch settings, bounded hermetic probes, exact observed native refusal, and actual prepared read-only filesystem boundary.

## Recorded evidence (verbatim from .bee/cells/cpc-2.json)

- **deviation** — Existing windows-sys Pipes feature added under decision 7741c278; no new crate. Separate recovery commit retained until wave-close consolidation because sibling commits intervene.
- **deviation** — Observed Codex 0.154.0 opaque native messages require named refusal; explicit configured external routes retained; no full native parity claim.
- **failure_signature** — 5ad523254196
- **failure_signature** — de5035ea1670
- **failure_signature** — dbecaa3041f5

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell cpc-3 — save as docs/knowledge/patterns/codex-parity-completion-cpc-3-pitfall.md

---
type: bee.pattern
title: codex-parity-completion cell cpc-3 — pitfall candidate
description: "Pitfall candidate mined from cell cpc-3's capped trace: Used path-scoped revision commits pending wave-close consolidation — a sibling commit separated the original cell commit from revisions — hit an unforeseen obs…"
timestamp: 2026-09-12
bee:
  id: codex-parity-completion-cpc-3-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/cpc-3.json]
  polarity: pitfall
---

# codex-parity-completion cell cpc-3 — pitfall candidate

## What the cell did

Normalize native Codex transcript usage, identity and final text with Claude compatibility.

## Recorded evidence (verbatim from .bee/cells/cpc-3.json)

- **deviation** — Used path-scoped revision commits pending wave-close consolidation — a sibling commit separated the original cell commit from revisions — hit an unforeseen obstacle
- **deviation** — Classified the direct session-close fixture separately from live Codex execution — installed native hook proof belongs to the running cpc-1 canary — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell cpc-4 — save as docs/knowledge/patterns/codex-parity-completion-cpc-4-pitfall.md

---
type: bee.pattern
title: codex-parity-completion cell cpc-4 — pitfall candidate
description: "Pitfall candidate mined from cell cpc-4's capped trace: Leader owns wave-barrier regeneration and integration proof; worker amended cpc-4 through user-approved bounded revisions."
timestamp: 2026-09-13
bee:
  id: codex-parity-completion-cpc-4-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/cpc-4.json]
  polarity: pitfall
---

# codex-parity-completion cell cpc-4 — pitfall candidate

## What the cell did

Publish verified Codex runtime limits, dispatch instructions and executable sandbox recipe.

## Recorded evidence (verbatim from .bee/cells/cpc-4.json)

- **deviation** — Leader owns wave-barrier regeneration and integration proof; worker amended cpc-4 through user-approved bounded revisions.
- **failure_signature** — d1e75b18129b
- **failure_signature** — abf6b9f47d22
- **failure_signature** — c0f3ad03e66c

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 4 pattern candidate(s), 0 file(s) written.