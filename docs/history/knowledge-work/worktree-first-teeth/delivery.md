---
type: bee.delivery
title: worktree-first-teeth — delivery
description: "Delivery record proposed by bee knowledge promote for work item worktree-first-teeth: 4 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-12
bee:
  id: worktree-first-teeth-delivery
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [docs/history/worktree-first-teeth/plan.md]
  sources: [docs/history/worktree-first-teeth/plan.md, .bee/cells/archive/worktree-first-teeth/wtf-1.json, .bee/cells/archive/worktree-first-teeth/wtf-2.json, .bee/cells/archive/worktree-first-teeth/wtf-3.json, .bee/cells/archive/worktree-first-teeth/wtf-4.json]
---

# worktree-first-teeth — Delivery

## What shipped

- **wtf-1** — Guard now judges the acting (lane-aware) record and denies a code-touching main-checkout write at phase swarming when the feature holds no granted worktree, naming bee worktree new --feature <feature>; corrupt registry still fails open (3 file(s) changed)
- **wtf-2** — Name the holding checkout on a reservation conflict (3 file(s) changed)
- **wtf-3** — Restored phase-independent worktree-first refusal and made unresolvable/unreadable grants fail open instead of falsely denying (3 file(s) changed)
- **wtf-4** — Narrowed the tiny-lane worktree-first exemption to the no-grant arm; granted arm now denies lane tiny too (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wtf-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **wtf-2** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **wtf-3** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **wtf-4** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work worktree-first-teeth` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/worktree-first-teeth/plan.md`. Every line above is copied from a trace or from the work item; Applied 2026-08-16 from docs/history/worktree-first-teeth/promote-proposals.md; area bullets declined (feature-wide scribing sync already stamped), no pattern candidates survived review.
