---
type: bee.delivery
title: bee-report-issues — delivery
description: "Delivery record proposed by bee knowledge promote for work item bee-report-issues: 3 capped cell(s), 8 recorded deviation(s)."
timestamp: 2026-10-06
bee:
  id: bee-report-issues-delivery
  lifecycle: active
  areas: [feedback-digest, doctrine-layer]
  required_context: [docs/history/bee-report-issues/CONTEXT.md, docs/history/bee-report-issues/plan.md]
  sources: [docs/history/bee-report-issues/CONTEXT.md, docs/history/bee-report-issues/plan.md, .bee/cells/bri-1.json, .bee/cells/bri-2.json, .bee/cells/bri-3.json]
---

# bee-report-issues — Delivery

## What shipped

- **bri-1** — bee report issue reads the gh list JSON from the first line starting with [ and the issue URL from the last https://github.com/ line, and checks the 120-char title limit after the scrub (1 file(s) changed)
- **bri-2** — gh issue list output is parsed from the first line starting with [ so a wrapper noise line no longer skips ingest (1 file(s) changed)
- **bri-3** — Host rule agents-bee-defect-report, rule-index row, bee-evolving issue steps, bee-report-issues concept and bee_report.ingest on are in place; regen committed (7 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **bri-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -- report distinct_flag_vocabulary registry` — touched only report.rs gh output parsing and title check; both new tests seen ok by name; full suite not run; not driven live against the mise gh wrapper
- **bri-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml feedback` — cell verify filter over the touched feedback.rs (run with cargo already on PATH): 34 passed incl. new gh_noise_lines_before_the_json_are_skipped (red before the fix) and an added noise-then-bad-JSON …
- **bri-3** — `.bee/bin/bee knowledge check && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test agents_block_render_parity --test rule_index_parity` — docs cell: knowledge check 0 errors (128 pre-existing warnings, none on the new concept), render parity 1/1, rule-index parity 3/3; also bee dev release-manifest --check 419 files match, bee knowledg…

## Deviations

- **bri-1** — followed the plan
- **bri-2** — followed the plan
- **bri-2** — sync-ack: skills/bee-evolving sync is planned cell bri-3 of this feature
- **bri-3** — Skipped the bee-writing-skills RED/GREEN pressure run for the bee-evolving edit — the worker-outward guard refuses starting claude from a worker, so no sub-agent scenario could run; the leader can run it before merge — hit an unforeseen obstacle
- **bri-3** — Ran bee knowledge index and committed five regenerated index files (feedback-digest, the two parent indexes, and bee-herding and hook-runtime which were already stale on the base) outside the cell file list — the new concept made feedback-digest/index.md stale and the generator rewrites all stale indexes together — something else had to be fixed first
- **bri-3** — Committed with a path-scoped commit instead of staging into the shared index — the concurrent-worker guard refuses staging while the bri-1 worker is live in this checkout — hit an unforeseen obstacle
- **bri-3** — Requirements landed: rule text in packages/bee/AGENTS.block.md section Care for the session (agents-bee-defect-report markers), rendered into AGENTS.md; row with spoken line in the doctrine-layer rule homes; skills/bee-evolving/SKILL.md step 1 (issue entries and the gh issue view exception), step 2 (ref and count), step 3 (Fixes trailer), step 6 (release comment), plus one red flag; concept docs/knowledge/areas/feedback-digest/bee-report-issues.md linked from overview.md; .bee/config.json bee_report.ingest true — followed the plan
- **bri-3** — sync-ack: AGENTS.md changed only by adding the new rule agents-bee-defect-report (rendered from AGENTS.block.md); no existing rule text changed, so no other rule's applied_at files need an update; the new rule's applied_at files are both in this commit

## Provenance

Proposed by `bee knowledge promote --work bee-report-issues` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/bee-report-issues/CONTEXT.md`, `docs/history/bee-report-issues/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
