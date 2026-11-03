promote proposal for work item "no-code-comments" (docs/history/no-code-comments/CONTEXT.md + docs/history/no-code-comments/plan.md) — 4 capped cell(s): ncc-1, ncc-2, ncc-3, ncc-4
anchor: history — docs/history/no-code-comments/CONTEXT.md, docs/history/no-code-comments/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/no-code-comments/delivery.md

---
type: bee.delivery
title: no-code-comments — delivery
description: "Delivery record proposed by bee knowledge promote for work item no-code-comments: 4 capped cell(s), 14 recorded deviation(s)."
timestamp: 2026-09-22
bee:
  id: no-code-comments-delivery
  lifecycle: active
  areas: [hook-runtime, doctrine-layer]
  required_context: [docs/history/no-code-comments/CONTEXT.md, docs/history/no-code-comments/plan.md]
  sources: [docs/history/no-code-comments/CONTEXT.md, docs/history/no-code-comments/plan.md, .bee/cells/ncc-1.json, .bee/cells/ncc-2.json, .bee/cells/ncc-3.json, .bee/cells/ncc-4.json]
---

# no-code-comments — Delivery

## What shipped

- **ncc-1** — comments.rs classifies a comment line once for every code root; 14 tests green (2 file(s) changed)
- **ncc-2** — Re-cap of the landed comment ratchet: artifacts verified at c3abfb65, the cell verify re-run fresh and green, no source change (4 file(s) changed)
- **ncc-3** — Write guard refuses the first comment line a change adds to a code file; re-capped after the rehomed why (5 file(s) changed)
- **ncc-4** — Landed the agents-no-code-comments rule in the block, the render, the rule index and the worker prompt; recorded the arm and the ratchet in the hook-runtime concept; added the comment-guard verify feature file and its README row; switched no_code_comments on for this repo and documented it false in the sample (12 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ncc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee comments::` — the cell verify, narrowed to the new module: 14 passed, 0 failed, 3827 filtered out; run from a script file because the worktree guard refuses a shell variable in a compound command
- **ncc-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee devtools::comment_baseline:: && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee catalog:: && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts --test registry_dispatch` — the cell's own verify string, run whole and unmodified on this re-cap: comment_baseline 9 passed (3857 filtered out; the ratchet test every_code_file_is_at_or_below_its_comment_baseline is one of the…
- **ncc-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee hooks::write_guard:: && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test hook_contracts && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee devtools::comment_baseline::` — the cell's own verify string, run whole and unmodified: 261 + 15 + 9 passed, 0 failed. It is narrowed by filter to the write-guard arm, the hook-contract binary and the comment ratchet that polices t…
- **ncc-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee verbs::drivers::tests && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test agents_block_render_parity --test rule_index_parity --test pointer_integrity --test instruction_laws && .bee/bin/bee dev release-manifest --check && rg -q no_code_comments packages/bee/AGENTS.block.md AGENTS.md packages/bee/prompts/worker-cell.md .bee/bin/prompts/worker-cell.md docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md .bee/verify/verify-app/features/comment-guard.md .bee/config.json && rg -q agents-no-code-comments docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md AGENTS.md` — the cell's own verify, run whole after ONE regen: verbs::drivers 305 passed, agents_block_render_parity 1, rule_index_parity 3, pointer_integrity 10, instruction_laws 9, release-manifest --check 376 …

## Deviations

- **ncc-1** — followed the plan
- **ncc-2** — re-cap after a record-level NEEDS_REVISION: no source file was changed, only a fresh verify and a new cap — the first cap's deviation line claimed a frozen in-flight comment line (789 vs 788) that does not exist, and the record correction is decision 3bc08aeb — the plan was wrong about a fact
- **ncc-2** — --write folds every unmerged wt/* branch on EVERY run, not only when no baseline exists as the cell text asked — a branch merged after the seed would otherwise push a file above its baseline and break the ratchet for a change nobody wrote — found a better route
- **ncc-2** — sync-ack: This cell touches no rule home. Commit c3abfb65 changes only comment_baseline.rs, devtools/mod.rs, registry_payload.json and .bee/comment-baseline.json (git show --name-only confirms). The no-comments rule text in AGENTS.md and the three skill homes is cell ncc-4's work, landed at 1f0fdda94; this re-cap owes no doctrine sync.
- **ncc-3** — re-cap after a NEEDS_REVISION on a deleted why: no source change this round; the why is rehomed in decision 658470bb — the plan was wrong about a fact
- **ncc-3** — capped with --sync-ack: the sync door named rule home AGENTS.md, which this cell's five write-guard files never touch — the AGENTS.md statement of the rule is sibling cell ncc-4 (commit 1f0fdda9), so the skill-file sync it asks for is the orchestrator's scope, not this cell's — hit an unforeseen obstacle
- **ncc-3** — reconstruct_target_text returns Result<(String,String), ReconstructFail> instead of Option<(String,String)> — a typed refusal lets the config arm keep its messages byte-identical — found a better route
- **ncc-3** — heredoc_writes returns a named HeredocWrite { target, append, body } instead of a (String, String) tuple — three named fields read better than a tuple at both call sites — found a better route
- **ncc-3** — the heredoc and apply_patch unit tests live in new #[cfg(test)] mod blocks inside guards.rs and detectors.rs — the tests sit beside the readers they cover — found a better route
- **ncc-3** — sync-ack: this cell's five files are write-guard sources and touch no rule home; the AGENTS.md statement of the no-comments rule is sibling cell ncc-4 (commit 1f0fdda9), and this cap is a re-cap with no source change
- **ncc-4** — added a one-line _doc entry for no_code_comments to .bee/config-sample.json beside the live false value — that file's _doc block is its only documentation (JSON has no comments), so a live key with no _doc line teaches a host nothing — found a better route
- **ncc-4** — rewrote the comment-guard.md bullet that told a verifier to run bee config set --key no_code_comments / bee config get — bee ships no config verb; the sandbox key is written with control-bee put .bee/config.json, the shape tests/hook_contracts.rs uses — the plan was wrong about a fact
- **ncc-4** — staged with git add .bee instead of naming .bee/config.json on the git add line — the config guard refuses ANY Bash command naming that path, git add included, and git add --dry-run .bee showed it resolved to exactly the six intended files — hit an unforeseen obstacle
- **ncc-4** — sync-ack: AGENTS.md is touched only to ADD the new rule agents-no-code-comments; the agents-capture-line-at-close rule and its wording are unchanged, so its applied_at files (skills/bee-capturing/SKILL.md, skills/bee-hive/SKILL.md, skills/bee-hive/references/routing-and-contracts.md) have nothing to sync

## Provenance

Proposed by `bee knowledge promote --work no-code-comments` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/no-code-comments/CONTEXT.md`, `docs/history/no-code-comments/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "no-code-comments" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-22T10:12:26.312Z), the work item declares no bee.areas.

area hook-runtime:
  - [ncc-2] Re-cap of the landed comment ratchet: artifacts verified at c3abfb65, the cell verify re-run fresh and green, no source change — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/ncc-2.json)
  - [ncc-3] Write guard refuses the first comment line a change adds to a code file; re-capped after the rehomed why — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/ncc-3.json)
  - [ncc-4] Landed the agents-no-code-comments rule in the block, the render, the rule index and the worker prompt; recorded the arm and the ratchet in the hook-runtime concept; added the comment-guard verify feature file and its README row; switched no_code_comments on for this repo and documented it false in the sample — feature-wide sync per the scribing stamp, 12 file(s) changed (trace .bee/cells/ncc-4.json)

area doctrine-layer:
  - [ncc-2] Re-cap of the landed comment ratchet: artifacts verified at c3abfb65, the cell verify re-run fresh and green, no source change — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/ncc-2.json)
  - [ncc-3] Write guard refuses the first comment line a change adds to a code file; re-capped after the rehomed why — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/ncc-3.json)
  - [ncc-4] Landed the agents-no-code-comments rule in the block, the render, the rule index and the worker prompt; recorded the arm and the ratchet in the hook-runtime concept; added the comment-guard verify feature file and its README row; switched no_code_comments on for this repo and documented it false in the sample — feature-wide sync per the scribing stamp, 12 file(s) changed (trace .bee/cells/ncc-4.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell ncc-1 — save as docs/knowledge/patterns/no-code-comments-ncc-1-pitfall.md

---
type: bee.pattern
title: no-code-comments cell ncc-1 — pitfall candidate
description: "Pitfall candidate mined from cell ncc-1's capped trace: followed the plan"
timestamp: 2026-09-22
bee:
  id: no-code-comments-ncc-1-pitfall
  lifecycle: draft
  areas: [hook-runtime, doctrine-layer]
  sources: [.bee/cells/ncc-1.json]
  polarity: pitfall
---

# no-code-comments cell ncc-1 — pitfall candidate

## What the cell did

comments.rs classifies a comment line once for every code root; 14 tests green

## Recorded evidence (verbatim from .bee/cells/ncc-1.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ncc-2 — save as docs/knowledge/patterns/no-code-comments-ncc-2-pitfall.md

---
type: bee.pattern
title: no-code-comments cell ncc-2 — pitfall candidate
description: "Pitfall candidate mined from cell ncc-2's capped trace: re-cap after a record-level NEEDS_REVISION: no source file was changed, only a fresh verify and a new cap — the first cap's deviation line claimed a frozen in-…"
timestamp: 2026-09-22
bee:
  id: no-code-comments-ncc-2-pitfall
  lifecycle: draft
  areas: [hook-runtime, doctrine-layer]
  sources: [.bee/cells/ncc-2.json]
  polarity: pitfall
---

# no-code-comments cell ncc-2 — pitfall candidate

## What the cell did

Re-cap of the landed comment ratchet: artifacts verified at c3abfb65, the cell verify re-run fresh and green, no source change

## Recorded evidence (verbatim from .bee/cells/ncc-2.json)

- **deviation** — re-cap after a record-level NEEDS_REVISION: no source file was changed, only a fresh verify and a new cap — the first cap's deviation line claimed a frozen in-flight comment line (789 vs 788) that does not exist, and the record correction is decision 3bc08aeb — the plan was wrong about a fact
- **deviation** — --write folds every unmerged wt/* branch on EVERY run, not only when no baseline exists as the cell text asked — a branch merged after the seed would otherwise push a file above its baseline and break the ratchet for a change nobody wrote — found a better route
- **deviation** — sync-ack: This cell touches no rule home. Commit c3abfb65 changes only comment_baseline.rs, devtools/mod.rs, registry_payload.json and .bee/comment-baseline.json (git show --name-only confirms). The no-comments rule text in AGENTS.md and the three skill homes is cell ncc-4's work, landed at 1f0fdda94; this re-cap owes no doctrine sync.
- **failure_signature** — cap trace ncc-2 records a frozen in-flight comment line in write_guard/tests.rs (789 vs 788) that does not exist: committed count at a5c80427 = baseline entry = 789, and the prescribed delete-then-write remedy has no line to delete and no effect while two unmerged branches carry 789.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ncc-3 — save as docs/knowledge/patterns/no-code-comments-ncc-3-pitfall.md

---
type: bee.pattern
title: no-code-comments cell ncc-3 — pitfall candidate
description: "Pitfall candidate mined from cell ncc-3's capped trace: re-cap after a NEEDS_REVISION on a deleted why: no source change this round; the why is rehomed in decision 658470bb — the plan was wrong about a fact"
timestamp: 2026-09-22
bee:
  id: no-code-comments-ncc-3-pitfall
  lifecycle: draft
  areas: [hook-runtime, doctrine-layer]
  sources: [.bee/cells/ncc-3.json]
  polarity: pitfall
---

# no-code-comments cell ncc-3 — pitfall candidate

## What the cell did

Write guard refuses the first comment line a change adds to a code file; re-capped after the rehomed why

## Recorded evidence (verbatim from .bee/cells/ncc-3.json)

- **deviation** — re-cap after a NEEDS_REVISION on a deleted why: no source change this round; the why is rehomed in decision 658470bb — the plan was wrong about a fact
- **deviation** — capped with --sync-ack: the sync door named rule home AGENTS.md, which this cell's five write-guard files never touch — the AGENTS.md statement of the rule is sibling cell ncc-4 (commit 1f0fdda9), so the skill-file sync it asks for is the orchestrator's scope, not this cell's — hit an unforeseen obstacle
- **deviation** — reconstruct_target_text returns Result<(String,String), ReconstructFail> instead of Option<(String,String)> — a typed refusal lets the config arm keep its messages byte-identical — found a better route
- **deviation** — heredoc_writes returns a named HeredocWrite { target, append, body } instead of a (String, String) tuple — three named fields read better than a tuple at both call sites — found a better route
- **deviation** — the heredoc and apply_patch unit tests live in new #[cfg(test)] mod blocks inside guards.rs and detectors.rs — the tests sit beside the readers they cover — found a better route
- **deviation** — sync-ack: this cell's five files are write-guard sources and touch no rule home; the AGENTS.md statement of the no-comments rule is sibling cell ncc-4 (commit 1f0fdda9), and this cap is a re-cap with no source change
- **failure_signature** — a5c80427 deletes the four-line 'Reconstruction reads the file actually being edited — never a fallback to the other config file' comment from write_guard/main.rs and rehomes the why nowhere, against D4 and D6.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ncc-4 — save as docs/knowledge/patterns/no-code-comments-ncc-4-pitfall.md

---
type: bee.pattern
title: no-code-comments cell ncc-4 — pitfall candidate
description: "Pitfall candidate mined from cell ncc-4's capped trace: added a one-line _doc entry for no_code_comments to .bee/config-sample.json beside the live false value — that file's _doc block is its only documentation (JSO…"
timestamp: 2026-09-22
bee:
  id: no-code-comments-ncc-4-pitfall
  lifecycle: draft
  areas: [hook-runtime, doctrine-layer]
  sources: [.bee/cells/ncc-4.json]
  polarity: pitfall
---

# no-code-comments cell ncc-4 — pitfall candidate

## What the cell did

Landed the agents-no-code-comments rule in the block, the render, the rule index and the worker prompt; recorded the arm and the ratchet in the hook-runtime concept; added the comment-guard verify feature file and its README row; switched no_code_comments on for this repo and documented it false in the sample

## Recorded evidence (verbatim from .bee/cells/ncc-4.json)

- **deviation** — added a one-line _doc entry for no_code_comments to .bee/config-sample.json beside the live false value — that file's _doc block is its only documentation (JSON has no comments), so a live key with no _doc line teaches a host nothing — found a better route
- **deviation** — rewrote the comment-guard.md bullet that told a verifier to run bee config set --key no_code_comments / bee config get — bee ships no config verb; the sandbox key is written with control-bee put .bee/config.json, the shape tests/hook_contracts.rs uses — the plan was wrong about a fact
- **deviation** — staged with git add .bee instead of naming .bee/config.json on the git add line — the config guard refuses ANY Bash command naming that path, git add included, and git add --dry-run .bee showed it resolved to exactly the six intended files — hit an unforeseen obstacle
- **deviation** — sync-ack: AGENTS.md is touched only to ADD the new rule agents-no-code-comments; the agents-capture-line-at-close rule and its wording are unchanged, so its applied_at files (skills/bee-capturing/SKILL.md, skills/bee-hive/SKILL.md, skills/bee-hive/references/routing-and-contracts.md) have nothing to sync

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 6 area bullet(s), 4 pattern candidate(s), 0 file(s) written.