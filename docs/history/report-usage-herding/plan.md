# report-usage-herding — Plan

**Lane:** small · **Class:** feature · **Date:** 2026-11-07

## Problem

`bee report usage` counts the leader transcript and its `subagents/*.jsonl`
only. A herded worker (a claude-session sub-session or a Paseo pane) writes
its own transcript elsewhere, so its tokens never reach the COMPLETION-REPORT.

Probe 2026-11-07: job `job-1794012325950-3541941-1` (claude-session, opus)
spent 103,589 input and 1,793 output tokens in
`~/.claude/projects/<proj>/37e070dd-….jsonl`; the leader report left them out.

`bee close` already counts these sessions through
`.bee/mailbox/<job>/activity.json` `transcript_path`
(`collect_herding_usage` in `verbs/drivers/close.rs`). The report verb does
not.

## Link

Each `.bee/mailbox/<job>/job.json` carries `leader_session` (the dispatching
leader's Claude session id). Each `activity.json` carries `transcript_path`.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | report usage reads only the leader transcript and its subagents dir | read | packages/bee-rs/crates/bee/src/verbs/report_usage.rs:299 | `let dir = transcript.with_extension("").join("subagents");` |
| 2 | a herding job records the dispatching leader session | ran | .bee/mailbox/job-1794012325950-3541941-1/job.json | `"leader_session":"51a0937f-e0af-4a92-9ada-379bd7fdf8ce"` |
| 3 | the job activity record holds the worker transcript path | ran | .bee/mailbox/job-1794012325950-3541941-1/activity.json | `"transcript_path": "/home/thanhsmind/.claude/projects/-mnt-data-Projects-goglbe-beehive/37e070dd-0834-4b99-b511-e0af992d49c8.jsonl"` |
| 4 | close already counts herded sessions through activity.json | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:2512 | `.get("transcript_path")` |

## Contract (cell RUH-2)

- Input: the resolved leader transcript `<sid>.jsonl`; the leader session id
  is its file stem.
- `bee report usage` adds, for every job under the control root's
  `.bee/mailbox/` whose `job.json` `leader_session` equals that id and whose
  `activity.json` `transcript_path` exists: that transcript and its
  `subagents/*.jsonl`, filtered by the same window, merged into the same
  per-model rows. Each transcript counts once (dedupe by path); the leader's
  own transcript never counts twice.
- A job with no `leader_session`, another leader's id, or a missing
  transcript is skipped, never an error.
- Output: text and `--json` gain a herding-session count beside the
  subagent-file count; the numbers stay `provider_telemetry`.
- `--help` text (registry) names the herding source.
- Proving tests in `verbs/report_usage.rs`: a matching job's tokens appear in
  the model row; another leader's job is excluded; events outside the
  window are excluded; a job whose transcript is missing is skipped.
- Docs: `LEADER.md` "Report back to waggledance" and
  `docs/knowledge/areas/bee-herding/claude-session-transport.md` name the
  herded-session source.
