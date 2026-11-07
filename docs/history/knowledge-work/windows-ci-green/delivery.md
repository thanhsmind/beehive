---
type: bee.delivery
title: windows-ci-green — delivery
description: "Delivery record proposed by bee knowledge promote for work item windows-ci-green: 3 capped cell(s), 4 recorded deviation(s)."
timestamp: 2026-09-19
bee:
  id: windows-ci-green-delivery
  lifecycle: active
  required_context: [docs/history/windows-ci-green/CONTEXT.md, docs/history/windows-ci-green/plan.md]
  sources: [docs/history/windows-ci-green/CONTEXT.md, docs/history/windows-ci-green/plan.md, .bee/cells/archive/windows-ci-green/win-1.json, .bee/cells/archive/windows-ci-green/win-2.json, .bee/cells/archive/windows-ci-green/win-3.json]
---

# windows-ci-green — Delivery

## What shipped

- **win-1** — Canonicalize the worktree test paths and make the release-dirt failure show its cause (2 file(s) changed)
- **win-2** — release-dirt runs under real Win32 bash on Windows; merge path expectation matches product (2 file(s) changed)
- **win-3** — new-worktree instruction expectation matches the raw path the product emits (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **win-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee` — 3689 passed, 0 failed, 20 ignored, re-run by the leader on Linux; both target tests executed and passed: test_release_dirt_script_filters_only_bee_paths and enter_and_merge_and_new_carry_session_runt…
- **win-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee` — the cell verify; 3689 passed 0 failed on Linux, Windows proof comes from the branch CI run
- **win-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee` — the cell verify; 3689 passed 0 failed on Linux, Windows proof comes from the branch CI run

## Deviations

- **win-1** — followed the plan
- **win-1** — sync-ack: No skill text changes: two test-only edits, no behavior a skill describes.
- **win-2** — the leader capped the cell, not the worker — the herding worker reported done with a narrow proof and skipped its finish command — something else had to be fixed first
- **win-3** — tiny cell run inline by the leader, no worker — one-line test edit, which the tiny lane allows — found a better route

## Provenance

Proposed by `bee knowledge promote --work windows-ci-green` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/windows-ci-green/CONTEXT.md`, `docs/history/windows-ci-green/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
