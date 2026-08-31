---
type: bee.pattern
title: bee herding run completes the worker but drops the gather digest when transport is not ready
description: bee herding run --task-file - dispatched from a session with transport_ready=false completes the worker but returns ONLY the job-summary JSON — the gather digest dies with the closed pane
tags: [herding, dispatch, gather, transport]
timestamp: 2026-08-29
bee:
  id: pattern-20260829-herding-run-drops-digest-no-transport
  lifecycle: active
  areas: [bee-herding]
  sources: ["2026-08-29 — two gathers lost this way in one day; content had to be re-read directly"]
  polarity: pitfall
  evidence: observed
---

# A gather's digest dies with the pane when transport is not ready

`bee herding run --task-file - --json` dispatched from a session with
`transport_ready=false` (no herdr pane) still completes the worker, but the
result payload carries only the job-summary JSON — the actual gather
digest, which the worker would otherwise have returned through the pane, is
gone with the closed pane. Lost this way twice in one day (2026-08-29);
the content had to be re-read directly to recover it.

## Fix direction (not yet implemented)

Either the result payload should carry the worker's report regardless of
transport state, or `bee dispatch prepare` should refuse or downgrade
gather dispatches when transport is not ready, rather than letting the
dispatch appear to succeed while silently dropping its output. Filed as a
defect observation only; no skill or behavior changed yet.
