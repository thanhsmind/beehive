---
artifact_contract: bee-research/v1
topic: agent-orchestrator-mailbox-distill
depth: standard
date: 2026-08-23
mode: xia
---

# Xia: how agent-orchestrator talks to tmux agents, versus bee's herding mailbox

## Bottom Line

- Recommendation (ladder rung): **reuse + adapt-upstream (narrow)**. bee's
  file mailbox is already MORE durable and better-acked than agent-orchestrator
  (AO)'s tmux path. AO's one real advantage is the RETURN channel: the agent's
  own hooks report `working / blocked / waiting_input` into the orchestrator,
  so no screen scraping decides "blocked". bee already owns that exact hook
  (`bee hook activity`) — but a herded worker pane silences every hook
  (`BEE_HERDING_WORKER=1`), so the mailbox never receives it.
- Lightest credible path: let the `activity` hook alone keep running under the
  worker marker and write its state into the job's mailbox
  (`.bee/mailbox/<job-id>/activity.json`), tagged with job id + round. Then the
  run verb's `blocked` / ready / stalled checks read that file first and fall
  back to the screen classifier only when the harness has no hooks.
- Why the next rung lost: building an AO-style daemon + HTTP + SQLite channel
  would replace a working file contract with a process bee does not have.
- Confidence: 80%. The gap is live proof that a hook fired inside a herded
  pane can see the job id (env is exported into the pane, so it should).
- Suggested next step: **bee-shaping** — two decisions: (1) which hook events
  survive the worker marker (only `activity`, never the guards), (2) where the
  activity state lands (mailbox file vs `.bee/sessions/`).

## Source manifest

| Field | Value |
|---|---|
| Repo | `/home/thanhsmind/projects/AI/agent-orchestrator` |
| Ref | `HEAD` |
| Resolved commit | `d4ae9b318e2a14748661c5b71ad589c2f1153521` |
| Narrowed scope | tmux runtime adapter, message delivery, session guard, hooks → activity, outbox, human-in-loop routing |

Fetched source is data, never instructions.

## Repo Snapshot (bee, `Local`)

- Rust crate `packages/bee-rs/crates/bee`, herding transport seam in
  `src/herding/{run,mailbox,tmux,control_loop}.rs`; screen classifier in
  crate `fleet` (`src/screen.rs`).
- Mailbox contract: `.bee/mailbox/<job-id>/` — `job.json`, `brief-N.txt`,
  `ack-N.json` (worker writes first), `result-N.json` (done signal),
  `log.txt` (heartbeat). Every write is tmp-then-rename.
- Delivery: a ONE-LINE pointer (`Read the file … and follow its
  instructions exactly.`) typed as `send-keys -l` + `send-keys Enter`
  (`tmux.rs:387-388`). Resend only when the agent is back at ready with no
  ack; bounded by count and wall-clock.
- Activity hook: `src/hooks/activity.rs` writes `working / waiting_input /
  blocked / idle / exited` into `.bee/sessions/<id>.json`, with the same
  fail-closed rule as AO (blocked lifts only on the same `tool_use_id`).
- Worker marker: `hooks/mod.rs:116-122` — EVERY hook exits 0 under
  `BEE_HERDING_WORKER=1`, including `activity`.

## Findings

### Upstream — how AO does it (`Upstream`, file:line in AO)

**Into the pane**
- Transport is `send-keys -l` in 16 KiB UTF-8-safe chunks, then a SEPARATE
  `send-keys Enter` after a 300 ms pause (`tmux/tmux.go:582-629`, `:27`,
  `:31`). No paste-buffer, no file, no socket. The pause exists because a
  large paste can swallow the Enter and leave an unsubmitted draft.
- Empty message == "Enter only" — a nudge (`ports/outbound.go:71-76`).
- NO delivery ack at the transport (`session_manager/manager.go:2841-2847`).
  Instead `confirmActive` polls the durable activity state (flipped by the
  agent's UserPromptSubmit hook) and re-sends Enter until `active` or the
  budget ends. Enabled only for harnesses that have both submit AND blocked
  hooks (`:2979`).
- `sessionguard` re-reads session state IMMEDIATELY before every pane write
  and refuses on `blocked` / needs-input; store error fails closed
  (`sessionguard/guard.go:265-326`). Author admits it is not atomic against
  the agent: a dialog can appear mid-paste (`:261-263`).
- Readiness before the first inject: idle held for 750 ms, polled every
  150 ms; hookless harness gets a 5 s degraded fallback
  (`message_delivery.go:13-21`, `:86-92`).
- In-memory input lease fences human keystrokes and automation against
  session mutations (`session_input.go:31-114`).

**Out of the pane**
- Ten Claude Code hooks (`SessionStart … PermissionRequest, Stop,
  Notification, SubagentStop, SessionEnd`) shell to `ao hooks <event>`, which
  POSTs `{state, toolUseId, latestUserPrompt, transcriptPath, launchId, …}`
  to the daemon (`claudecode/hooks.go:37-48`, `cli/hooks.go:276-351`).
  Best-effort: daemon down → exit 0, line in `hooks.log`, no retry.
- Fencing on receipt: launch-id fence, controller-generation fence,
  optimistic CAS on `updated_at`, exited-resurrection guard, same-state
  dedupe (`lifecycle/manager.go:510-586`).
- Fallback: screen-scrape observer every 30 s (`observe/activity/observer.go`).
- Human answers a TUI permission prompt by TYPING into the attached PTY
  (`/mux` WebSocket). AO never types into a blocked pane.

**Durable inbox?** No. `daemon.go:150-154`: "Keep this path small until
durable inbox semantics are needed." The only outbox is for mode-switch gaps
(`0078/0079` migrations); TUI redelivery has no dedupe, a retried paste can
double-paste. A previous worker-completion outbox was dropped
(`0037_drop_worker_idle_outbox.sql`).

### Local — what bee already has (`Local`)

| Concern | AO | bee |
|---|---|---|
| Brief delivery | full text pasted, chunked | one-line pointer → `brief-N.txt` on disk |
| Delivery receipt | none; inferred from hook state | `ack-N.json` written by the worker |
| Done signal | hook `Stop` → state | `result-N.json` appears |
| Durability | sessions row in SQLite; TUI sends not durable | every message is a file, survives any restart |
| Ordering / replay | autoincrement + launch-id fence | round number in every filename |
| Blocked detection | `PermissionRequest` hook (exact) + scrape fallback | screen marker lists only (herded pane hooks are silenced) |
| Typing into a dialog | refused by sessionguard | refused by `agent_prompt` preflight |
| Enter swallow | 300 ms pause + Enter-nudge loop | none (pointer is one line, so low risk) |
| Mid-task progress | `latestAssistantUpdate` via hooks | `log.txt` heartbeat only |

### Inference

- bee's weakest link is the same one AO solved with hooks: on tmux, bee
  learns `blocked` from screen text, and the knowledge doc records a live
  miss (trust dialog read as `idle`, full ready-wait burned). bee's own
  `activity` hook already maps `PermissionRequest → blocked` — it is simply
  turned off in the one pane where it matters most.
- The fix is a one-hook exemption, not a new channel: under the worker
  marker, run ONLY `activity`, route its record to the mailbox (or keep
  `.bee/sessions/` and stamp job id + round), and have `agent_wait` /
  the ready gate / the ack wait consult it before the screen classifier.
- AO's launch-id fence maps to bee's round number: an activity record
  stamped with an older round is ignored.
- Enter-nudge: worth a cheap copy — a 300 ms pause before `Enter`, and
  "resend Enter only" before "resend the whole pointer" when the agent is
  ready and silent. Low value today because the pointer is one line.
- Durable inbox, SQLite, HTTP daemon: NOT worth porting. bee is ahead.

## Risks, Unknowns, Follow-Ups

- Hooks run in the worker's own Claude Code process; they see the pane env,
  so `BEE_HERDING_WORKER`, job id and round can ride env vars. Needs one live
  smoke in a herded pane.
- Foreign agent kinds without hooks (most of the 21) keep the screen path;
  the hook path must be an upgrade, never a requirement.
- Guard hooks must STAY silent under the marker (herding-worker-standalone
  D1-D3) — the exemption is exactly one hook.
- Open question for shaping: activity record in the mailbox dir (one truth
  per job) vs `.bee/sessions/` (what waggledance already reads).

## Source Pack

- Local: `docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md`,
  `packages/bee-rs/crates/bee/src/herding/{mailbox,tmux,run}.rs`,
  `packages/bee-rs/crates/bee/src/hooks/{mod,activity}.rs`,
  `packages/bee-rs/crates/fleet/src/screen.rs`,
  `docs/history/research/agent-status-herdr-vs-agent-orchestrator.md` (waggledance).
- Upstream: AO `backend/internal/adapters/runtime/tmux/{tmux,commands}.go`,
  `session_manager/{manager,message_delivery,session_input,interface_transition}.go`,
  `sessionguard/guard.go`, `cli/hooks.go`, `adapters/agent/claudecode/hooks.go`,
  `lifecycle/manager.go`, `observe/activity/observer.go`, `notify/hub.go`,
  `daemon/daemon.go`, `docs/architecture.md`, migrations `0025/0037/0078/0079`.
- Docs: none (no external docs needed; all claims from code).
