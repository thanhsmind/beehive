---
type: research
status: closed
claimed-by: none
blocked-by: none
---

## Question

Waggledance is read-only by design; the supervisor (2f4bf3b1) must observe
cross-project AND write its own records (observations, delegated decisions,
weekly report). Facts needed from the waggledance codebase: (a) what bee
state waggledance already reads and rolls up per registered project (the
"waiting on you" board's sources); (b) where an observer tick could run
(existing schedule/heartbeat machinery? a bee herding loop pointed at many
repos? a waggledance-side job?); (c) where supervisor records could live
without breaking read-only-by-design and without a cross-repo store —
per-target-repo `.bee/` writes, a waggledance-local db, or the cockpit's own
project; (d) what the MCP surface (`waggledance_ask_state`,
`waggledance_projects`, `waggledance_search`) already answers that the
observer would otherwise re-read from disk.

## Answer

Advisor-tier research (dispatch 7f98f9ef, 2026-08-29). Verdict: buildable
today as "external tick + MCP reads + per-repo bee-CLI writes", the only new
machinery being the tick owner and a slightly widened ask_state digest.

- (a) GREEN — `read_snapshot(root)` (waggledance-core bee.rs:1222) parses the
  full `.bee/` surface per project (state, cells, backlog, sessions+heartbeat,
  lanes, decisions, handoff, config+gate_bypass, reviews, capture queue,
  reservations, docs/history) and derives the attention list + a
  `waiting_on_live` flag; `read_rollup(roots)` maps it fleet-wide. Reads only,
  never writes (D4).
- (b) YELLOW — no cron machinery, but the daemon's notify PollWatcher already
  sweeps every bee root every 2s and pings the human via Telegram — a
  rule-based fleet observer tick exists. An LLM supervisor tick is either one
  more reconcile-slot loop (modest new code) or an EXTERNAL loop (cron /
  bee-herding role) calling the MCP surface — zero waggledance changes.
- (c) YELLOW-GREEN — the read-only rule bars waggledance itself from writing
  a project's `.bee/`, but the sanctioned pattern exists: spawn that repo's
  OWN bee CLI at its root (create_project_pbi and gate board actions already
  do this, server.rs:1613-1725). Supervisor records = `bee decisions log` /
  `pbi add` per target repo, plus a DEDICATED COCKPIT REPO (any repo with
  .bee/ registers like the rest) holding the supervisor's own records and
  weekly report — no new storage, waggledance renders it for free. A
  waggledance-local table would be new schema and hides records from git.
- (d) GREEN — `waggledance_ask_state` (no project) returns the cross-project
  rollup: feature/phase/mode, waiting_on_live, cell buckets, recent
  decisions, session heartbeats, attention items, pane inventory. Gaps worth
  widening: backlog PBIs, review queue, worktrees, reservations (present in
  the snapshot, absent from the digest). dispatch/await/runs give gated hands.

Decision logged: see MAP.md. Ticket 005 unblocked.

**2f4bf3b1** (the supervisor's cross-project home is the waggledance
layer) was built out by five further decisions: **12be1c0b** shapes the
waggledance supervisor itself — an external tick reading the fleet
rollup and writing records by spawning each target repo's own bee CLI;
**58796a73** keeps the human able to work a repo's lead directly, never
routed through the supervisor; **8fea3561** needs no new cross-repo
dissent bookkeeping — a repo's own dissent machinery is enough, the
waggledance supervisor just surfaces it in its rollup; **423871d7**
pins the tick as always cold, zero in-process memory; **b590e508**
keeps this per-repo supervisor running alongside the waggledance one
rather than being replaced by it.
