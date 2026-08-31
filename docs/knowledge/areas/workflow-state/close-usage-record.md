---
type: bee.area
title: "Workflow State — the close usage record: token accounting, its home, and its persistence"
description: "How bee close accounts for a feature's token usage across every session bound to it, where the detailed record is written and by which commit it is actually persisted, and the one known gap between a linked worktree's record root and its stage root."
timestamp: 2026-08-30
bee:
  id: workflow-state-close-usage-record
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: ["close-usage-summary D1 (2d3abd12)", "close-usage-record D1 (e97cc9d4)", "usage-in-bee-store D1 (62331863)"]
  sources: ["docs/knowledge/work/usage-in-bee-store/delivery.md", "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs", "packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs", "packages/bee-rs/crates/bee/src/verbs/mailbox.rs"]
  authoritative_for: "workflow-state: the close usage record — accounting, home, and persistence"
---

# Workflow State — the close usage record: token accounting, its home, and its persistence

A green `bee close` accounts for the tokens a feature actually cost, across
every session that touched it, and leaves that accounting somewhere another
tool can read it back.

## Behaviors & Operations

- **Accounting (D1 · 2d3abd12).** For each session record bound to the
  closing feature — plus the calling session, de-duplicated — `bee close`
  reads that session's transcript with `rollup_transcript`
  (`hooks/session_close/perf.rs`) and prints a token-usage section: main
  tokens, subagent tokens, and a grand total. A session with no readable
  transcript is skipped and counted as skipped rather than silently
  dropped. `usage_session_ids` resolves against the **control root**, not
  the plain root, so a feature worked from a linked worktree still reports
  real tokens for every session bound to it, not just the ones physically
  inside that worktree.
- **The detailed record and its home (D1 · e97cc9d4, moved by D1 · 62331863).**
  On a green close, bee writes the detailed token record — per session:
  models, subagent_models, subagent_count, totals — plus feature totals and
  the skipped count, schema `bee-usage/v1`. The record lives at
  `.bee/usage/<feature>.json`, under the **control root** — it belongs to
  bee's own store, not the host project's `docs/` tree, so an external tool
  can aggregate usage across many projects by globbing `.bee/usage/*.json`.
  `.bee/usage/` is not gitignored. The feature close letter in the human
  mailbox gets a matching one-line Token usage section, and report walkers
  key on the `bee-usage/v1` schema plus the close result's `usage_record`
  path.
- **Persistence (implementation-confirmed).** `usage.json` is written by
  `close`, but it is not close's own commit that persists it — close's
  bookkeeping commit stages `.bee` only, the same as promote-proposals.
  It is the **merge-time auto-commit** that actually lands the file in
  history, when the feature's worktree merges back.

## Edge Cases Settled

- A session with an unreadable transcript is counted, not silently excluded
  — the skipped count in both the printed section and the JSON record
  accounts for it.
- Moving the record's home (docs/history → .bee/usage) kept the JSON shape
  and the letter line byte-identical; only the path changed.

## Open Gaps

- **Worktree record/stage-root split.** A close run from inside a linked
  worktree writes the usage record to **main's** `.bee` (the control root)
  but stages the change against the **worktree's** `.bee` — record root and
  stage root differ there, so the write can land somewhere the merge-time
  auto-commit does not look. Not yet fixed; flagged at the moment it was
  found (`c1064938`, 2026-08-30).

## Pointers (implementation)

- Accounting and the token-usage section: `verbs/drivers/close.rs`,
  `hooks/session_close/perf.rs` (`rollup_transcript`).
- Record write, schema `bee-usage/v1`, and the letter line:
  `verbs/drivers/close.rs`, `verbs/mailbox.rs`.
- Delivery trace: `docs/knowledge/work/usage-in-bee-store/delivery.md`.
