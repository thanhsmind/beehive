promote proposal for work item "review-p2-hardening" (docs/history/review-p2-hardening/plan.md) — 4 capped cell(s): rph-1, rph-2, rph-3, rph-4
anchor: history — docs/history/review-p2-hardening/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/review-p2-hardening/delivery.md

---
type: bee.delivery
title: review-p2-hardening — delivery
description: "Delivery record proposed by bee knowledge promote for work item review-p2-hardening: 4 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-10
bee:
  id: review-p2-hardening-delivery
  lifecycle: active
  areas: [workflow-state, worktree-parallelism, onboarding]
  required_context: [docs/history/review-p2-hardening/plan.md]
  sources: [docs/history/review-p2-hardening/plan.md, .bee/cells/rph-1.json, .bee/cells/rph-2.json, .bee/cells/rph-3.json, .bee/cells/rph-4.json]
---

# review-p2-hardening — Delivery

## What shipped

- **rph-1** — shared commit_unsigned helper in worktree/git.rs consumed by close + merge (single-flag removal reds both stub tests); config refusal names the offending file across the overlay; null reads as unset; defense-arm test restored (3 file(s) changed)
- **rph-2** — register validates feature slug pre-join; worktree new surfaces cellsSync skip; tracked-set parse fails closed; dest archive in symlink-checked set; pruned names reported; ceiling guard on fail-safe test; fixture seeds --no-gpg-sign (4 file(s) changed)
- **rph-3** — locate walk stops at first .git boundary; repo-root is first candidate and LocateError names both; BEE_JS_ENTRY neutralized in tests; locate() pure delegate pinned via locate_from (3 file(s) changed)
- **rph-4** — push_worker_record upserts by nickname+cell; worker_registered pinned through real CLI entry (out-of-process child), failure shape on inner seam per noted split; cells update arms behavior door via shared arms_behavior_door (5 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rph-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **rph-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **rph-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **rph-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work review-p2-hardening` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/review-p2-hardening/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "review-p2-hardening" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-10T23:42:09.116Z), the work item declares no bee.areas.

area workflow-state:
  - [rph-1] shared commit_unsigned helper in worktree/git.rs consumed by close + merge (single-flag removal reds both stub tests); config refusal names the offending file across the overlay; null reads as unset; defense-arm test restored — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/rph-1.json)
  - [rph-2] register validates feature slug pre-join; worktree new surfaces cellsSync skip; tracked-set parse fails closed; dest archive in symlink-checked set; pruned names reported; ceiling guard on fail-safe test; fixture seeds --no-gpg-sign — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/rph-2.json)
  - [rph-3] locate walk stops at first .git boundary; repo-root is first candidate and LocateError names both; BEE_JS_ENTRY neutralized in tests; locate() pure delegate pinned via locate_from — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/rph-3.json)
  - [rph-4] push_worker_record upserts by nickname+cell; worker_registered pinned through real CLI entry (out-of-process child), failure shape on inner seam per noted split; cells update arms behavior door via shared arms_behavior_door — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/rph-4.json)

area worktree-parallelism:
  - [rph-1] shared commit_unsigned helper in worktree/git.rs consumed by close + merge (single-flag removal reds both stub tests); config refusal names the offending file across the overlay; null reads as unset; defense-arm test restored — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/rph-1.json)
  - [rph-2] register validates feature slug pre-join; worktree new surfaces cellsSync skip; tracked-set parse fails closed; dest archive in symlink-checked set; pruned names reported; ceiling guard on fail-safe test; fixture seeds --no-gpg-sign — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/rph-2.json)
  - [rph-3] locate walk stops at first .git boundary; repo-root is first candidate and LocateError names both; BEE_JS_ENTRY neutralized in tests; locate() pure delegate pinned via locate_from — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/rph-3.json)
  - [rph-4] push_worker_record upserts by nickname+cell; worker_registered pinned through real CLI entry (out-of-process child), failure shape on inner seam per noted split; cells update arms behavior door via shared arms_behavior_door — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/rph-4.json)

area onboarding:
  - [rph-1] shared commit_unsigned helper in worktree/git.rs consumed by close + merge (single-flag removal reds both stub tests); config refusal names the offending file across the overlay; null reads as unset; defense-arm test restored — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/rph-1.json)
  - [rph-2] register validates feature slug pre-join; worktree new surfaces cellsSync skip; tracked-set parse fails closed; dest archive in symlink-checked set; pruned names reported; ceiling guard on fail-safe test; fixture seeds --no-gpg-sign — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/rph-2.json)
  - [rph-3] locate walk stops at first .git boundary; repo-root is first candidate and LocateError names both; BEE_JS_ENTRY neutralized in tests; locate() pure delegate pinned via locate_from — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/rph-3.json)
  - [rph-4] push_worker_record upserts by nickname+cell; worker_registered pinned through real CLI entry (out-of-process child), failure shape on inner seam per noted split; cells update arms behavior door via shared arms_behavior_door — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/rph-4.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 12 area bullet(s), 0 pattern candidate(s), 0 file(s) written.