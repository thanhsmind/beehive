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
  decisions: ["close-usage-summary D1 (2d3abd12)", "close-usage-record D1 (e97cc9d4)", "usage-in-bee-store D1 (62331863)", "report-usage-verb (af457159)"]
  sources: ["docs/knowledge/work/usage-in-bee-store/delivery.md", "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs", "packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs", "docs/history/herding-job-feature/plan.md", "packages/bee-rs/crates/bee/src/herding/run.rs", "packages/bee-rs/crates/bee/src/hooks/activity.rs", "packages/bee-rs/crates/bee/src/verbs/mailbox.rs", "packages/bee-rs/crates/bee/src/verbs/report.rs"]
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
  (`packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs`) and prints a token-usage section: main
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
- **Per-task report (report-usage-verb · af457159).** `bee report usage
  --corr <t-hex>` prints the waggledance COMPLETION-REPORT header and
  per-model token table for one task window, read from the session
  transcript and its `subagents/*.jsonl` through `aggregate_usage`.
  Input = fresh + cache write + cache read, Cache = the cached part,
  Total = Input + Output; a missing transcript is a typed error, never
  zeros. AGENTS.md names it the first token source.
- **Herded sessions in the per-task report (report-usage-herding plan,
  contract RUH-2).** A herded worker — a claude-session sub-session or a
  pane — writes its own transcript, outside the leader's `subagents/`.
  `bee report usage` reads every `.bee/mailbox/<job>/` under the control
  root whose `job.json` `leader_session` equals the leader transcript's
  file stem, and adds that job's `activity.json` `transcript_path` (and its
  `subagents/`) to the same per-model rows, window-filtered, each
  transcript once. A job of another leader, with no `leader_session`, or
  with a missing transcript is skipped. The report prints
  `Herded Sessions: <n>` (`herding_sessions` in `--json`); `n` counts the
  linked sessions, including any with no tokens inside the window.
  `bee close` reaches the same transcripts by feature instead of by
  leader (`collect_herding_usage`).
- **The job's feature is the join key (herding-job-feature plan,
  `docs/history/herding-job-feature/plan.md`).** A herded job's feature has
  one home, `job.json` `feature`. `bee herding run` stamps it from
  `--feature`, else the `--cell-id` cell's feature, else the calling
  session's bound lane, and writes no key when none resolves; `bee
  dispatch prepare` puts `--feature` on every `herding run` command it
  returns once it has resolved a feature. The activity hook labels the
  job's `activity.json` from that stamp before any worktree-grant or
  directory-name rule, and `herding_jobs` in `bee close` joins on it
  before falling back to the `activity.json` label. A main-checkout
  reviewer, advisor or scout is therefore counted by the feature that
  dispatched it, not by whatever `.bee/state.json` held at hook time. A job
  with no stamp keeps the old label.

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
  `packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs` (`rollup_transcript`).
- Record write, schema `bee-usage/v1`, and the letter line:
  `verbs/drivers/close.rs`, `verbs/mailbox.rs`.
- Per-task report (`bee report usage`): `verbs/report_usage.rs`
  (`herded_transcripts` for the mailbox join), over `aggregate_usage` in
  `packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs`.
- Job feature stamp: `herding/run.rs` (`resolve_job_feature`,
  `stamp_feature`), `verbs/drivers/prepare.rs` (`--feature` on the herding
  command), `packages/bee-rs/crates/bee/src/hooks/activity.rs` (`resolve_herded_feature`),
  `verbs/drivers/close.rs` (`herding_jobs`).
- Delivery trace: `docs/knowledge/work/usage-in-bee-store/delivery.md`.
