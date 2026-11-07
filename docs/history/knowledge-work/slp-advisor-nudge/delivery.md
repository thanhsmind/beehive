---
type: bee.delivery
title: slp-advisor-nudge — delivery
description: "Delivery record proposed by bee knowledge promote for work item slp-advisor-nudge: 5 capped cell(s), 9 recorded deviation(s)."
timestamp: 2026-08-29
bee:
  id: slp-advisor-nudge-delivery
  lifecycle: active
  required_context: [docs/history/slp-advisor-nudge/CONTEXT.md, docs/history/slp-advisor-nudge/plan.md]
  sources: [docs/history/slp-advisor-nudge/CONTEXT.md, docs/history/slp-advisor-nudge/plan.md, .bee/cells/archive/slp-advisor-nudge/an-1.json, .bee/cells/archive/slp-advisor-nudge/an-2.json, .bee/cells/archive/slp-advisor-nudge/an-3.json, .bee/cells/archive/slp-advisor-nudge/an-4.json, .bee/cells/archive/slp-advisor-nudge/an-5.json]
---

# slp-advisor-nudge — Delivery

## What shipped

- **an-1** — advisor-nudge mailbox kind with frequency cap, two new poor-work signals, record-time feature derivation, and turn-boundary delivery lifted to worktree sessions (3 file(s) changed)
- **an-2** — per-feature advisor-nudge debt counter with a per-row, id-naming decision escape (1 file(s) changed)
- **an-3** — advisor-nudge debt now refuses the cap, bee close, and bee worktree merge at every lane, all three reading one shared count with a per-row decision escape (7 file(s) changed)
- **an-4** — needs-human-decision flag derived once and sorted first in letters and the WakeReport (2 file(s) changed)
- **an-5** — The supervisor prompt teaches the advisor-nudge record — its mailbox form, the three poor-work signals that earn it, the lead-owned response and its cap-refusing debt; supervisor record's --kind/--signal help prose caught up with the widened closed sets (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **an-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml supervisor`
- **an-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
- **an-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
- **an-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
- **an-5** — `rg -c 'advisor-nudge' skills/bee-herding/references/supervisor-prompt.md packages/bee-rs/crates/bee/src/verbs/supervisor.rs packages/bee-rs/crates/bee/src/catalog.rs && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && bee dev release-manifest --check`

## Deviations

- **an-1** — Edited skills/bee-herding/references/supervisor-prompt.md, which the cell did not name — widening KNOWN_SIGNALS trips control_loop.rs the_shipped_prompt_pins_the_record_verbs_own_closed_sets, a guard whose whole purpose is to make the prompt learn a new word in the SAME change as the verb; reserved the path first and kept the edit to the two signal names, leaving the advisor-nudge KIND wording for an-5 — something else had to be fixed first
- **an-1** — Ran bee dev regen after that prompt edit — the skill projections (.claude/.opencode/.agents/.claude-plugin/.codex-plugin) and the release manifest are committed artifacts of skills/, so two parity tests went red naming their own remedy — something else had to be fixed first
- **an-3** — Added the merge-door tests to verbs/worktree/tests.rs and updated two door snapshots in verbs/drivers/tests.rs, both reserved under my nickname first — the merge test needs that file's git harness and the new door breaks the snapshots the plan itself flagged as a risk — hit an unforeseen obstacle
- **an-3** — Merge refusal is ONE line inside [WORKTREE_MERGE_ADVISOR_NUDGE_DEBT] rather than the plan's three-line headline/remedy/next form — every refusal in that function is a single typed line, and mixing two forms in one door reads worse than following the sibling — found a better route
- **an-5** — Put the --kind/--signal help prose fix in packages/bee-rs/crates/bee/src/generated/registry_payload.json instead of catalog.rs, and gave catalog.rs the flag-ratchet entry (199 -> 199, no new flag name) — the cell named catalog.rs, but that help text lives only in the hand-edited registry payload (bln-3 records that bee dev regen does not write that file); catalog.rs holds only the sup-2 ratchet comment, which is self-dated history and stays as written — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work slp-advisor-nudge` from 5 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/slp-advisor-nudge/CONTEXT.md`, `docs/history/slp-advisor-nudge/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
