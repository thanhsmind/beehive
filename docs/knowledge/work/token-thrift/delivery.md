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
