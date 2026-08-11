---
type: bee.area
title: Performance Log — CLI Self-Timing
description: "Every CLI invocation measures its own wall time — one fail-open JSONL line per run plus one stderr summary, stdout untouched; the raw material for finding and fixing slow commands."
timestamp: 2026-07-24
bee:
  id: performance-log-cli-self-timing
  lifecycle: active
  areas: [performance-log]
  required_context: []
  decisions: [4439bd7e (work-visibility D3)]
  sources: ["work-visibility cell wv-1 (capped, verified; trace .bee/cells/wv-1.json)", docs/history/work-visibility/reports/wv-1-done.md, timing tests a-d in the CLI test suite]
  authoritative_for: "performance-log: per-invocation CLI self-timing"
---

# Performance Log — CLI Self-Timing

Every direct run of the CLI measures its own wall time at the dispatch
boundary. When the command settles, two things happen, in this order of
authority:

- **B1 — One JSONL line per invocation**, appended fail-open to
  `.bee/logs/timings.jsonl`: `{ts, cmd, ms, ok}` — ISO timestamp, resolved
  `<group> <verb>` (`unknown` when resolution failed), wall milliseconds, and
  `ok` reflecting the exit code. The append is wrapped so a logging failure
  (unwritable directory, missing repo root) never changes the command's
  outcome or output.
- **B2 — One stderr summary line** `[bee] <cmd> <ms>ms`. stderr only, ever:
  stdout stays byte-identical for every verb, so every `--json` consumer and
  every test that parses stdout is untouched.

## Rules

- **R1** — Timing runs only on direct CLI runs, never when the dispatcher is
  imported (tests import it without side effects).
- **R2** — The log is analysis material, not ceremony: its purpose is finding
  slow commands to optimize. The aggregation verb SHIPPED as `bee timings
  report` (timings-report cell tr-1, 2026-08-11, closing PBI p-10caed3f):
  per-command count, total, median, p95, and max ms, ranked slowest-median
  first, `--limit` defaulting to 15; a missing or empty log is a clean empty
  report, malformed lines are skipped and counted, and the verb resolves its
  store through the wide door so it answers from a granted worktree too.
- **R3** — Failures are recorded too (`ok:false`), so slowness and breakage
  in the same command are visible in one place.
- **R4 — Telemetry is exempt from state-integrity hashes (release-1-15-0
  rb-2/rb-3, 2026-07-24).** Everything under `.bee/logs/` is fail-open,
  append-only runtime telemetry — never managed content and never state. Two
  tree-hash guards (compact-check's mutates-nothing control and the repeat-
  install byte-idempotence check) went red the day self-timing shipped,
  because every CLI invocation appends a timing line; both now exempt
  `.bee/logs/**` as a directory-scoped convention, and the compact-check
  exemption is paired with a negative control proving a genuine state
  mutation still turns the check red.
