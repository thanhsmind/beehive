# Waggledance completion report

Every task that arrives from waggledance (a `t-<hex>` correlation id) closes with a completion report in the `--kind result` mail body. The report carries the four parts the task asks for (`Done:`, `Proof:`, `Left:`, `Where:`) and, above them, this block:

```markdown
### [COMPLETION-REPORT] Task: <task name> / <t-xxxxxx>
- **Correlation ID:** t-xxxxxx
- **Status:** COMPLETED | PARTIAL | BLOCKED
- **Wall-Clock Duration:** <hh:mm:ss> (Start: <ISO 8601> | End: <ISO 8601>)
- **Total Tokens:** <n> (Input: <n> | Output: <n> | Cache: <n>)

#### Token Breakdown by Model:
| Model Name | Input Tokens | Output Tokens | Total Tokens | Accounting Source |
| :--- | :--- | :--- | :--- | :--- |
| <model> | ... | ... | ... | <source> |

#### Key Artifacts & Deliverables:
- Commit / Files: ...
- Verification & Test Proof: ...
```

## Rules for the numbers

- **Tokens come from telemetry only.** Never estimate, never average,
  never round a guess. The first source is `bee report usage --corr
  <t-hex>`: it prints the header and the per-model table for the task
  window, and the leader pastes its output. It also reads each herded
  worker session of this leader through the job mailbox
  (`.bee/mailbox/<job>/`: `leader_session` and `activity.json`
  `transcript_path`) and prints their count on the `Herded Sessions`
  line. The fallbacks, in order:
  the feature's `.bee/usage/<feature>.json` (`bee-usage/v1`, written by
  a green `bee close`); the session's latest `bee-perf/v1` row in
  `~/.config/beehive/performance.jsonl` (written at each turn end, main
  and `subagent_models` apart); or the `message.usage` fields of the
  session transcript and its `subagents/*.jsonl`. Name the source in
  the table's last column.
- **One meaning per number.** Input = every prompt token (fresh +
  cache write + cache read). Cache = the cache write + cache read part
  of Input. Total = Input + Output. One row per model; a model used by
  the leader and by workers sums into one row.
- **Wall clock is one span.** Start = the moment the leader received
  the task (the first transcript event of the task turn); End = the
  moment the report is composed. Never add up worker or cell durations
  — parallel time overlaps.
- **Name the gap.** Telemetry written after the report (the report
  turn's own tokens, a worker still running) is not in the numbers;
  say so on one line. A source that is missing gets "not recorded",
  never a number.
- **Workers do not report this.** Cells and workers report to the
  leader as before. Tokens of a subagent are already in the leader's
  `subagent_models` and transcript. Tokens of a herded session (a
  claude-session sub-session or a pane) are in its own transcript:
  `bee report usage` reads them through the job mailbox. Only the
  leader composes the block.
