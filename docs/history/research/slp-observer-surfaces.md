# Research Digest — Observer Surfaces and Signal Observables in Bee (SLP Ticket 001)

- **Date**: 2026-08-26
- **Context**: `docs/discovery/slp-supervisor-lead-peer/tickets/001-observer-surfaces.md`
- **Scope**: SLP Cluster 1 (Supervisor Heartbeat) & Cluster 2 (Detector) — observable read surfaces and 7-signal feasibility audit.

---

## 1. Executive Summary & Verdict

### Core Questions
1. What can a supervisor or detector read in `bee` today?
2. Which of the seven signal types from the SLP spec have a `bee`-native observable?

### Summary Answers
- **Available Read Surfaces**: `bee` provides 7 distinct readable surfaces:
  1. **Cockpit Pane Transcripts & Screen Classifier** (`bee herding pane read`, `fleet::screen::classify`).
  2. **Agent Activity Record & Transition Log** (`.bee/sessions/<id>.json` -> `activity` / `work`, `.activity.jsonl`, mailbox `activity.json`).
  3. **Waiting-On Marks** (`bee state waiting-on`, `.bee/state.json`, `.bee/workflows/<id>.json`).
  4. **Session Registry & Heartbeat Liveness** (`bee state session list`, derived `signal: live | no_signal`).
  5. **Wave Ledger & Occupancy** (`.bee/logs/wave-ledger.jsonl`, `bee herding occupancy`).
  6. **Cells, Claims, & Retry Budgets** (`.bee/cells/*.json`, `check_cell_budgets`).
  7. **Decisions Stream & Triggers** (`.bee/decisions.jsonl`, `bee decisions active / search`, `.bee/triggers/`).

- **Signal Observables Verdict (7 Signals)**:
  - **3 Native Observables Today**: `struggling-loop` (cell retry budgets & activity tool failures), `big-decision` (decisions log & approval gates), `danger-op` (write guard refusals, dialog blocked states, secret redaction).
  - **4 Non-Native (Gaps / Need Adaptation)**: `self-correction` (nearest: unparsed terminal scrollback), `boundary-approach` (nearest: path write-guard denials & waiting-on questions), `test-on-unstable-contract` (nearest: locked decisions in CONTEXT.md; no `contract_status` registry exists), `budget-80` (nearest: 100% hard limit refusal & session turn counter).

---

## 2. Detailed Audit of Observer Surfaces in Bee

### Surface 1: Tmux / Herdr Cockpit Pane Transcripts

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/herding/pane_verbs.rs:802-822`: `bee herding pane read <pane_id> [--lines <n>]`.
  - `packages/bee-rs/crates/bee/src/herding/tmux.rs:798-800`: `RealTmux::pane_read` -> `capture-pane -p -t <pane_id> -S <scrollback>`.
  - `packages/bee-rs/crates/fleet/src/screen.rs:28-160`: Shared screen classifier (`fleet::screen::classify`, `ScreenSettings`, `Screen::{Idle, Working, Blocked}`).
  - `packages/bee-rs/crates/fleet/src/backend/tmux.rs:190-200`: `capture-pane` backend execution.
  - `packages/bee-rs/crates/bee/src/herding/pane_verbs.rs:48-85`: `PaneRow` in `bee herding pane list` (`id`, `label`, `tab`, `cwd`, `command`, `foreground_cwd`, `agent_status`, `agent_session`).

- **What It CAN See**:
  - Raw visible terminal text up to the scrollback limit (`scrollback: 40` lines by default in `fleet::screen.rs:87`).
  - Terminal process state: foreground PID (`#{pane_pid}`) and running command name (`#{pane_current_command}`).
  - TUI status classification based on UI chrome markers (`Screen::Working` via `"esc to interrupt"`, `"ctrl+c to interrupt"`; `Screen::Blocked` via `"do you trust"`, `"paste your api key"`, `"press enter to submit"`; `Screen::Idle` when output settles).
  - Working directory divergence: initial directory (`cwd`) vs active process directory (`foreground_cwd`).

- **What It CANNOT See**:
  - Internal LLM reasoning, hidden chain-of-thought, or token probabilities (unless directly printed to terminal stdout).
  - Full conversation history beyond the scrollback tail.
  - Structured tool call parameters or return payloads (only formatted TUI text).
  - Work completion truth (`fleet::screen.rs:13-17` explicitly enforces no `Done` state; completion is verified only through result files).
  - Headless background processes not attached to a multiplexer pane.

---

### Surface 2: Herding Activity Hook & Activity Records

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/hooks/activity.rs:1-150`: Lifecycle checkpoint handler (`bee hook activity`).
  - `.bee/sessions/<session_id>.json`: `activity` object (`state`, `event`, `tool_name`, `tool_use_id`, `at`, `pane`, `pane_transport`, `cwd`, `feature`, `cell`, `waiting_on_set_by_hook`).
  - `packages/bee-rs/crates/bee/src/hooks/activity.rs:568-617`: `work` object (`title`, `text` up to 8000 characters, `status: "open"`, `opened_at`, `turns`, `updated_at`).
  - `packages/bee-rs/crates/bee/src/hooks/activity.rs:623-650`: Transition log `.bee/sessions/<session_id>.activity.jsonl` (capped at last 50 state transitions).
  - `packages/bee-rs/crates/bee/src/hooks/activity.rs:324-336`: Foreign worker mailbox sink `.bee/mailbox/<job-id>/activity.json`.
  - `packages/bee-rs/crates/bee/src/hooks/activity.rs:122-144`: Event mapping (`UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `PostToolUseFailure` -> `working`; `PermissionRequest` -> `blocked`; `Notification(agent_needs_input)` -> `waiting_input`; `Stop` -> `idle`; `SessionEnd` -> `exited`).

- **What It CAN See**:
  - Current operational state across 5 discrete states: `working`, `blocked`, `waiting_input`, `idle`, `exited`.
  - Identity of the active or failed tool (`tool_name`, `tool_use_id`).
  - Tool execution failures via `PostToolUseFailure` events.
  - ISO-8601 UTC timestamp of the most recent lifecycle event (`at`).
  - Conversation prompt text and turn counts (`work.turns`, `work.text` up to 8,000 characters with path and secret sanitization).
  - Associated feature lane and active cell claim.

- **What It CANNOT See**:
  - Tool arguments, file contents, or tool output data (strictly content-free per R21 / decision `2f782f51`).
  - Subagent lifecycle events (`SubagentStop` is deliberately ignored per decision `b17bfa89`).
  - State history beyond the 50 most recent transitions.
  - Un-hooked runtime environments (only runtimes wired with bee lifecycle hooks emit records).

---

### Surface 3: Waiting-On Marks

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/state_group/waiting_on.rs:1-120`: CLI interface (`bee state waiting-on set`, `bee state waiting-on clear`).
  - `packages/bee-rs/crates/bee/src/verbs/workflow_store/record.rs:360-445`: Core validation (`build_waiting_on`, `waiting_on_is_live`, `WAITING_ON_KIND_VALUES = ["gate", "question", "turn-end"]`).
  - `.bee/state.json` / `.bee/workflows/<id>.json`: Stored `waiting_on` object (`kind`, `subject`, `asked_at`, `session`).
  - `packages/bee-rs/crates/bee/src/hooks/activity.rs:276-291`: Automated hook synchronization (`waiting_on_set_by_hook`).

- **What It CAN See**:
  - Active human intervention requests: `gate` (approval required), `question` (agent asked human an open question), or `turn-end` (control returned to human).
  - Subject string describing the gate or question (`subject`).
  - Time elapsed since question was asked (`asked_at`) and owning session ID (`session`).
  - Distinguishes whether the mark was set explicitly by the agent or automatically by the activity hook.

- **What It CANNOT See**:
  - Unstated blockers (if an agent hangs or fails without calling `waiting-on set` or triggering a hook).
  - Multiple concurrent questions within a single workflow (stores exactly one `waiting_on` object per workflow).
  - Structured question metadata or proposed option trees (stores only free-text `subject`).

---

### Surface 4: Session Heartbeats & Session Registry

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs:113-198`: `bee state session list [--json]`.
  - `packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs:134-150`: Derived liveness calculation (`session_signal`, `SESSION_SIGNAL_WINDOW_SECONDS = 90.0`).
  - `.bee/sessions/<session_id>.json`: Session metadata (`id`, `started_at`, `last_heartbeat`, `lane`, `workspace_id`, `status`).

- **What It CAN See**:
  - Full inventory of all live and recent agent sessions across workspaces and worktrees.
  - Process heartbeat timestamp (`last_heartbeat`) vs activity freshness (`signal: "live" | "no_signal" | null`).
  - Workspace and lane bindings for each session.

- **What It CANNOT See**:
  - Granular activity details (only high-level heartbeat and derived 90-second liveness signal).
  - Token consumption rates or context window utilization.
  - Heartbeat history (only the most recent `last_heartbeat` timestamp is retained).

---

### Surface 5: Wave Ledger & Occupancy

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/herding/wave_ledger.rs:1-100`: Wave ledger engine (`.bee/logs/wave-ledger.jsonl`, `append_wave`, `fold_waves_by_wave_id`, `live_worker_count`).
  - `packages/bee-rs/crates/bee/src/herding/wave.rs:13-39`: CLI occupancy query (`bee herding occupancy [--main-root <path>]`).
  - Ledger record schema: `wave_id`, `started_at`, `workers: [{name, pane_id, worktree, task, outcome, evidence}]`.

- **What It CAN See**:
  - All multi-agent wave executions, assigned tasks, worktree paths, and pane IDs.
  - True live occupancy: intersects unresolved ledger workers (`outcome: null`) with live multiplexer pane IDs (`Occupancy::Live { occupied, max }`).
  - Degradation status: detects when multiplexer is unreachable and reports fallback state (`Occupancy::Fallback`).
  - Terminal outcomes (`finished`, `refused`, `timeout`, `unverifiable`) and evidence pointers.

- **What It CANNOT See**:
  - In-flight worker progress between spawn and terminal completion.
  - Live pane buffer contents (requires dedicated `pane read` calls).
  - Ad-hoc standalone workers started outside the herding subsystem.

---

### Surface 6: Cells, Claims, & Cell Budgets

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:124-128, 688-827`: Structural loop-safety check (`check_cell_budgets`, `BUDGET_KEYS = ["max_claims", "max_failed_attempts", "max_same_signature"]`, defaults `[3.0, 4.0, 2.0]`, hard caps `[9.0, 12.0, 6.0]`).
  - `packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:1-300`: Cell lifecycle management (`bee cells list`, `bee cells claim`, `bee cells unclaim`, `.bee/cells/<cell-id>.json`).
  - Cell schema: `id`, `goal`, `lane`, `status` (`open`, `claimed`, `capped`), `budgets`, `trace` (`attempts`, `budget_resets`), `proof` (`command`, `result`, `scope_reason`).

- **What It CAN See**:
  - Cell lifecycle progression and claim ownership (`claimed_by`, `claimed_at`).
  - Exhausted retry budgets (`CELL_BUDGET_EXHAUSTED` when `max_claims`, `max_failed_attempts`, or `max_same_signature` is breached).
  - History of failed attempts and failure signatures recorded in `trace.attempts`.
  - Verified proof lines on capped cells (`<command> — <result> — <scope reason>`).
  - Escalation flags (`escalate: true`).

- **What It CANNOT See**:
  - Intermediate attempt progress before claim failure or cap (e.g. no 80% budget consumption warning).
  - Token or dollar spend per cell (budgets track discrete claim/failure counts, not token metrics).
  - Worktree file diffs prior to cell cap commit.

---

### Surface 7: Decisions Stream & Triggers

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/decisions/`: Append-only decision store (`.bee/decisions.jsonl`, `bee decisions active`, `bee decisions search`, `bee decisions supersede`, `bee decisions log`).
  - `packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs:1-450`: Deferred decision triggers (`.bee/triggers/<slug>__<id>.json`, `bee triggers list`).
  - Decision schema: `id`, `date`, `decision`, `rationale`, `alternatives`, `scope`, `tags`, `relation` (`supersedes:<id>`, `touches:<id>`, `none`), `trigger`.

- **What It CAN See**:
  - Full history of explicit architectural decisions, technical trade-offs, and rejected alternatives.
  - Decision relationship graph (superseded decisions, touched decisions).
  - Registered triggers and condition definitions for deferred decisions.

- **What It CANNOT See**:
  - Implicit or undocumented trade-offs made in conversation without logging a formal decision.
  - Active discussions or debates before a decision entry is appended.
  - Automated detection of code diverging from recorded decisions.

---

## 3. Seven Signal Types Feasibility Matrix

Per the brief requirement, here is the exact one-line status for each signal type:

### One-Line Signal Status Summary

1. `self-correction`: **not observable, nearest unparsed terminal scrollback via `bee herding pane read <pane_id>` or session `work.text` prompt history in `.bee/sessions/<id>.json`**
2. `struggling-loop`: **observable today via cell failure and signature thresholds in `.bee/cells/<id>.json` (`check_cell_budgets`), repeated `PostToolUseFailure` in `.bee/sessions/<id>.activity.jsonl`, and herding idle-timeout stall detection**
3. `boundary-approach`: **not observable, nearest write-guard path and reservation denials in `.bee/logs/hooks.jsonl` or explicit agent questions via `bee state waiting-on set --kind question`**
4. `big-decision`: **observable today via the `.bee/decisions.jsonl` stream (`bee decisions active/search`), lane `approved_gates` in `.bee/state.json`, and explicit `bee state waiting-on set --kind gate|question` marks**
5. `test-on-unstable-contract`: **not observable, nearest locked decisions in `docs/history/<feature>/CONTEXT.md` and `docs/knowledge/` concept specs (no machine-readable `contract_status` CHỐT/CHƯA-CHỐT registry exists)**
6. `budget-80`: **not observable, nearest cell claim/failure attempt counters in `.bee/cells/<id>.json` (exhaustion only at 100%), session `work.turns` counter in `.bee/sessions/<id>.json`, and context compaction at ~65%**
7. `danger-op`: **observable today via write-guard refusals in `.bee/logs/hooks.jsonl`, terminal dialog stops (`Screen::Blocked` / `activity.state = "blocked"`), and `SECRET_PATTERNS` prompt redaction**

---

### Detailed Signal Analysis

| Signal Type | Status Today | Exact Observable / Anchor | Gap & Recommended Adaptation |
|---|---|---|---|
| **1. self-correction** | **Not Observable** | Nearest: `bee herding pane read <pane_id>` (`pane_verbs.rs:802`) or `work.text` (`activity.rs:595`). | Internal LLM hesitation or backtracking ("wait", "hold on", "let me rethink") is not parsed into events. A cheap regex/detector scan over pane scrollback or Claude Code transcript JSONL is needed. |
| **2. struggling-loop** | **Observable** | `check_cell_budgets` (`validate.rs:752`), `PostToolUseFailure` in `.activity.jsonl` (`activity.rs:124`), idle timeout (`run.rs:87`). | Bee already tracks repeated failed attempts (`max_failed_attempts = 4`) and identical error signatures (`max_same_signature = 2`). A detector can query `.activity.jsonl` transitions for failure bursts. |
| **3. boundary-approach** | **Not Observable** | Nearest: Write guard refusals in `.bee/logs/hooks.jsonl` (`guards.rs:121`) or `waiting_on` marks (`waiting_on.rs:80`). | Semantic design trade-offs (e.g. quantizing int16 to int8, modifying public interfaces) within authorized files are invisible. Requires Peer instruction discipline (`StopAndAsk`) and supervisor AST/diff checks. |
| **4. big-decision** | **Observable** | `bee decisions active` (`.bee/decisions.jsonl`), `approved_gates` in `.bee/lanes/*.json`, `waiting_on` marks (`record.rs:455`). | All formal architectural decisions and lifecycle gates (`shape`, `execution`, `uat`) are durable, queryable records. |
| **5. test-on-unstable-contract** | **Not Observable** | Nearest: `docs/history/<feature>/CONTEXT.md` locked decisions and `docs/knowledge/` area concepts. | Bee has no machine-readable `contract_status` registry with `CHỐT` / `CHƯA CHỐT` labels. Ticket 007 addresses adding this label surface. |
| **6. budget-80** | **Not Observable** | Nearest: Cell claim budget exhaustion at 100% (`validate.rs:775`), `work.turns` in `.bee/sessions/<id>.json` (`activity.rs:596`), compaction at 65% (`compaction.rs:40`). | Budgets are enforced only at claim boundary (100% exhaustion) and context limit (65%). No intermediate 80% proactive event exists today. |
| **7. danger-op** | **Observable** | Write guard hook refusals (`packages/bee-rs/crates/bee/src/hooks/write_guard/`), `Screen::Blocked` (`fleet/src/screen.rs:125`), secret scrub (`activity.rs:523`). | Attempted modifications to protected paths (`.bee/`, `.git/`, out-of-worktree), auth/permission prompts, and secret leaks are blocked or redacted. |

---

## 4. Architectural Recommendations for Detector & Supervisor

1. **Detector Placement (Cheap Signal Poller)**:
   - Poll lightweight JSON state files: `.bee/sessions/*.json`, `.bee/sessions/*.activity.jsonl`, `.bee/cells/*.json`, and `.bee/logs/hooks.jsonl`.
   - Run regex/keyword scanners on `bee herding pane read <pane_id> --lines 30` only when `activity.state == "working"` and `signal == "live"`.
   - Fire event payloads into a dedicated event log (`.bee/logs/detector-events.jsonl`).

2. **Supervisor Heartbeat Integration (Option C from Ticket 002)**:
   - Run `bee herding control-loop --role supervisor --interval 900` on a cheap model role (`models.claude.supervisor = "haiku"`).
   - Read detector events, active `waiting_on` marks, and session liveness to assemble periodic digests and generate wake reports.

3. **Bridge Missing Signals**:
   - Implement `contract_status` registry (Ticket 007) to enable `test-on-unstable-contract` detection.
   - Add warning threshold telemetry at 80% of cell retry budgets and turn ceilings.
   - Maintain peer instruction prompts enforcing `StopAndAsk` on boundary trade-offs.

---

File created: `docs/history/research/slp-observer-surfaces.md`
