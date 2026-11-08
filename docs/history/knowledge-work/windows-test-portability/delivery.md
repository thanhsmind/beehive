---
type: bee.delivery
title: windows-test-portability — delivery
description: "Delivery record proposed by bee knowledge promote for work item windows-test-portability: 2 capped cell(s), 3 recorded deviation(s)."
timestamp: 2026-09-11
bee:
  id: windows-test-portability-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [docs/history/windows-test-portability/plan.md]
  sources: [docs/history/windows-test-portability/plan.md, .bee/cells/archive/windows-test-portability/wtp-1.json, .bee/cells/archive/windows-test-portability/wtp-2.json]
---

# windows-test-portability — Delivery

## What shipped

- **wtp-1** — Onboard statusline tests pinned to a POSIX host via posix_fixture() in onboard/tests.rs; the two empty-repo tests branch on merge::host_shell_is_powershell (1 file(s) changed)
- **wtp-2** — prompt-skew path check is separator-neutral (verbs/drivers/tests.rs:4739); enter-worktree compares dunce-canonical paths (verbs/worktree/tests.rs:5987) and the text check reuses the printed worktreeRoot (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wtp-1** — `cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee onboard::`
- **wtp-2** — `cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee prompt_skew && cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee enter_worktree_core`

## Deviations

- **wtp-1** — posix_fixture() also covers the 8 other section tests that were not failing (user-level entry, unparseable file, --no-statusline, etc.) — the cell said pin every fixture repo in the section, and without the pin those tests assert the PowerShell skip on Windows instead of the display — found a better route
- **wtp-1** — the_project_config_opt_out_behaves_exactly_like_the_flag keeps fixture() and writes host_shell posix plus statusline false in its own config, as the cell asked — followed the plan
- **wtp-2** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work windows-test-portability` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/windows-test-portability/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
