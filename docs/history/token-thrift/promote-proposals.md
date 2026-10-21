promote proposal for work item "token-thrift" (docs/history/token-thrift/CONTEXT.md + docs/history/token-thrift/plan.md) — 4 capped cell(s): tt-1, tt-2, tt-3, tt-4
anchor: history — docs/history/token-thrift/CONTEXT.md, docs/history/token-thrift/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/token-thrift/delivery.md

---
type: bee.delivery
title: token-thrift — delivery
description: "Delivery record proposed by bee knowledge promote for work item token-thrift: 4 capped cell(s), 4 recorded deviation(s)."
timestamp: 2026-10-21
bee:
  id: token-thrift-delivery
  lifecycle: active
  required_context: [docs/history/token-thrift/CONTEXT.md, docs/history/token-thrift/plan.md]
  sources: [docs/history/token-thrift/CONTEXT.md, docs/history/token-thrift/plan.md, .bee/cells/tt-1.json, .bee/cells/tt-2.json, .bee/cells/tt-3.json, .bee/cells/tt-4.json]
---

# token-thrift — Delivery

## What shipped

- **tt-1** — Session-start stdout fits 9500 bytes: named drop guard, folded zero-frontier maps, and the preamble promote line capped at 3 clear commands (live 9224 bytes on this repo) (4 file(s) changed)
- **tt-2** — Dispatch door now prints one full uncut line per distinct rendered target listing its roles; new grouped test plus an all-18-role grouping test; compaction and preamble pins updated — 646c14cff (3 file(s) changed)
- **tt-3** — Orient's promote-proposal blocker names at most 3 features then `; +N more`, keeping the total count and byte-identical output for 3 or fewer; new 5-proposal test added — commit 81f81b039 (2 file(s) changed)
- **tt-4** — New concept token-economy-of-injected-context.md records host limit, 9500 budget, drop order, Omitted line, map fold, grouped door, orient and preamble promote caps, prior savings and the 8f63adb4 boundary; model-roles door line states the grouped shape (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **tt-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee session_preamble && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee session_init` — 67 and 24 passed; branch binary rendered bee hook session-init on this repo at 9224 bytes (main before: 11161), Omitted line names Recent decisions and Critical patterns with fetch paths; evidence /m…
- **tt-2** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee model_guard && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee compaction && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee session_preamble` — cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee model_guard && ... compaction && ... session_preamble — green:unit — 81 + 14 + 66 passed, 0 failed; these three suites…
- **tt-3** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee status_full` — cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee status_full — green:unit — all 119 status_full tests pass, including the new orient_caps_the_promote_proposal_blocker_…
- **tt-4** — `rg -n 'HOST_INLINE_LIMIT_CHARS|9500|Omitted to fit|8f63adb4|should_inject' docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md` — docs cell: cell verify matched all five terms (13 lines); bee knowledge check also 0 errors and no warning for the new concept

## Deviations

- **tt-1** — removed the dead #[allow(dead_code)] session_body wrapper in session_init.rs, superseded by session_body_within — found a better route
- **tt-1** — escalated onto the session model after the herded worker hit its 30-min ceiling; kept its code and restored the older comments it had rewritten — hit an unforeseen obstacle
- **tt-1** — added PROMOTE_CLEAR_COMMANDS_SHOWN = 3 in hooks/session_preamble/render.rs (D7 691fcac4) after the live render measured 11444 bytes: the promote line carried 74 clear commands — the plan was wrong about a fact
- **tt-4** — added a D7 sentence beside the orient blocker text, a D7 note in Edge Cases, decision 691fcac4 in frontmatter and hooks/session_preamble/render.rs in Pointers — the leader sent decision D7 mid-cell — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work token-thrift` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/token-thrift/CONTEXT.md`, `docs/history/token-thrift/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell tt-1 — save as docs/knowledge/patterns/token-thrift-tt-1-pitfall.md

---
type: bee.pattern
title: token-thrift cell tt-1 — pitfall candidate
description: "Pitfall candidate mined from cell tt-1's capped trace: removed the dead #[allow(dead_code)] session_body wrapper in session_init.rs, superseded by session_body_within — found a better route"
timestamp: 2026-10-21
bee:
  id: token-thrift-tt-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/tt-1.json]
  polarity: pitfall
---

# token-thrift cell tt-1 — pitfall candidate

## What the cell did

Session-start stdout fits 9500 bytes: named drop guard, folded zero-frontier maps, and the preamble promote line capped at 3 clear commands (live 9224 bytes on this repo)

## Recorded evidence (verbatim from .bee/cells/tt-1.json)

- **deviation** — removed the dead #[allow(dead_code)] session_body wrapper in session_init.rs, superseded by session_body_within — found a better route
- **deviation** — escalated onto the session model after the herded worker hit its 30-min ceiling; kept its code and restored the older comments it had rewritten — hit an unforeseen obstacle
- **deviation** — added PROMOTE_CLEAR_COMMANDS_SHOWN = 3 in hooks/session_preamble/render.rs (D7 691fcac4) after the live render measured 11444 bytes: the promote line carried 74 clear commands — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell tt-4 — save as docs/knowledge/patterns/token-thrift-tt-4-pitfall.md

---
type: bee.pattern
title: token-thrift cell tt-4 — pitfall candidate
description: "Pitfall candidate mined from cell tt-4's capped trace: added a D7 sentence beside the orient blocker text, a D7 note in Edge Cases, decision 691fcac4 in frontmatter and hooks/session_preamble/render.rs in Pointers …"
timestamp: 2026-10-21
bee:
  id: token-thrift-tt-4-pitfall
  lifecycle: draft
  sources: [.bee/cells/tt-4.json]
  polarity: pitfall
---

# token-thrift cell tt-4 — pitfall candidate

## What the cell did

New concept token-economy-of-injected-context.md records host limit, 9500 budget, drop order, Omitted line, map fold, grouped door, orient and preamble promote caps, prior savings and the 8f63adb4 boundary; model-roles door line states the grouped shape

## Recorded evidence (verbatim from .bee/cells/tt-4.json)

- **deviation** — added a D7 sentence beside the orient blocker text, a D7 note in Edge Cases, decision 691fcac4 in frontmatter and hooks/session_preamble/render.rs in Pointers — the leader sent decision D7 mid-cell — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.