# Research Digest — Supervisor Heartbeat Placement and Model Costing (SLP Ticket 002)

- **Date**: 2026-08-26
- **Context**: `docs/discovery/slp-supervisor-lead-peer/tickets/002-supervisor-placement.md`
- **Scope**: SLP Cluster 1 (Supervisor Heartbeat) — runtime placement, model role resolution, and operational constraints.

---

## 1. Executive Summary & Verdict

- **Question**: Where can a 15-minute supervisor heartbeat RUN in bee today, on what model, and at what cost?
- **Verdict**: **Option (c) — a `bee herding`-style role (`--role supervisor`) invoked per tick via `bee herding control-loop` running on a cheap `supervisor` model role (`haiku`)** is the cheapest, cleanest, and most reliable option.
- **Key Reasons**:
  1. Reuses the existing native Rust control loop engine (`packages/bee-rs/crates/bee/src/herding/control_loop.rs`) which already solves timeouts, consecutive-failure backoff, stop-file signalling (`.bee/tmp/bee-herding.stop`), and cross-platform child lifecycle across Linux and Windows.
  2. Spawns cold on every 15-minute tick (96 ticks/day), preventing monotonic context window accumulation and token bloat.
  3. Uses the open fall-through role architecture (decision `06e49368` in `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs`), making `models.claude.supervisor = "haiku"` pure configuration without code changes in the resolver.
  4. Enforces the three standing constraints (R2 human merge, R3 owner dispatch interlock, R4 permission split) through an enumerated, read-focused `--allowedTools` surface.

---

## 2. Option Comparison & Costing

### Option (a): Cron / Scheduled Session
*Sub-variants: `bee triggers`, external OS cron, Paseo / IDE heartbeat.*

- **What exists today (with anchors)**:
  - `bee triggers`: Implemented in `packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs:1-450`. Manages deferred-decision trigger records under `.bee/triggers/<slug>__<id>.json` (`triggers add`, `triggers list`, `triggers resolve`). Condition evaluation occurs on-demand when commands run (e.g. `bee orient`, `status_full/orient.rs:192-197`).
  - External OS cron / systemd timer / Windows Task Scheduler: Can execute arbitrary CLI commands (`bee ...`, `claude ...`).
  - External agent scheduler (e.g. Paseo / agent toolkit timers): Background timer API for external orchestration.
- **What is missing**:
  - `bee triggers` is **not** an active execution daemon or scheduler. It contains no loop, no process monitor, and no timer triggers. It cannot initiate execution on its own.
  - External cron lacks repository lifecycle awareness. It cannot easily inspect cockpit panes or tmux environment variables (`$TMUX`, `$HERDR_ENV`) without fragile wrapper scripts. It breaks cross-platform uniformity (Linux crontab vs Windows Task Scheduler).
  - Paseo/IDE timers depend on external tool runtimes and are not self-contained within the standalone `bee` CLI distribution.
- **Rough Cost & Complexity**:
  - **Complexity**: High / Fragile. Requires external infrastructure setup outside repository boundaries.
  - **Cost**: Low token cost (~$0.35/day on Haiku), but high maintenance and operational friction.

---

### Option (b): Dedicated Pane in the tmux / herdr Herding Cockpit

- **What exists today (with anchors)**:
  - `skills/bee-herding/scripts/bootstrap-cockpit.sh:220-295`: Builds the cockpit layout (Chat, Dispatch, Merge panes, plus Runtime tab) using transport-neutral `bee herding pane` verbs (`packages/bee-rs/crates/bee/src/herding/pane_verbs.rs`).
  - Transport abstraction in `packages/bee-rs/crates/fleet/src/backend/tmux.rs` and `packages/bee-rs/crates/bee/src/herding/tmux.rs`.
  - Splitting and managing panes via `bee herding pane split`, `pane run`, `pane read`, `pane list`.
- **What is missing**:
  - A dedicated persistent interactive session in a cockpit pane keeps its process and context open indefinitely. Over a multi-day cockpit run, 96 ticks per day would accumulate massive conversation history, rapidly hitting context limits (~200k tokens) and causing extreme token burn or degradation.
  - Adding a persistent interactive supervisor pane requires layout modification in `bootstrap-cockpit.sh` and continuous session compaction / restart protocols.
  - Note: If the pane runs a periodic command loop rather than a single continuous session, it reduces directly to Option (c) hosted inside a pane.
- **Rough Cost & Complexity**:
  - **Complexity**: Medium. Layout adjustment in `bootstrap-cockpit.sh` is straightforward, but managing persistent context growth is expensive and error-prone.
  - **Cost**: High token cost if run as a continuous persistent session (due to compounding context size on every tick).

---

### Option (c): `bee herding`-Style Role Invoked Per Tick (`bee herding control-loop`)

- **What exists today (with anchors)**:
  - `packages/bee-rs/crates/bee/src/herding/control_loop.rs:1-660`: Native Rust control loop driver (D8, ho-14). Supports `--interval` (e.g. `--interval 900`), `--timeout`, `--turn-ceiling`, `--max-iterations`, `--max-consecutive-failures`, and `--once`.
  - Stop file protocol: Atomically checks `.bee/tmp/bee-herding.stop` before and after each iteration (`control_loop.rs:536-586`).
  - Process safety: Wall-clock execution ceiling with cross-platform termination/SIGKILL (`control_loop.rs:361-466`), capped exponential backoff on failure (`control_loop.rs:514-518`).
  - Cold iteration discipline: Spawns a fresh child process per tick using role-specific prompt templates (`skills/bee-herding/references/<role>-prompt.md`) and enumerated allowed tools (`control_loop.rs:220-235`).
- **What is missing**:
  - Adding `Supervisor` to `Role` enum in `packages/bee-rs/crates/bee/src/herding/control_loop.rs:54-75`.
  - Defining `allowed_tools_for(Role::Supervisor, transport)` with an enumerated read/query surface (`Bash(tmux:capture-pane:*)` or `Bash(.bee/bin/bee:*)`, `Read`).
  - Authoring `skills/bee-herding/references/supervisor-prompt.md`.
  - Optional cockpit layout integration in `bootstrap-cockpit.sh` to launch `bee herding control-loop --role supervisor --interval 900` in a background pane or process.
- **Rough Cost & Complexity**:
  - **Complexity**: Low. Directly leverages existing, battle-tested Rust control loop machinery.
  - **Cost**: Lowest possible. Each cold tick uses a minimal prompt (~2-3k input tokens, ~200 output tokens), costing ~$0.35/day on Haiku.

---

## 3. Model Role Architecture & Escalation Verification

### Verification of Decision `06e49368` in `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs`

- **Open Fall-Through Role Set**:
  - Decision `06e49368` (documented in `docs/discovery/model-role-split/MAP.md:120-127` and implemented in `models.rs:36-52, 251-274`) retired closed slot membership checks.
  - `normalize_models` (`models.rs:251-274`) preserves all role keys defined under `models.<runtime>`.
  - `resolve_role` and `resolve_role_named` (`models.rs:496-609`) resolve any arbitrary role name passed in an ordered list against `models.<runtime>`.
  - If a role is configured (e.g. `models.claude.supervisor`), it resolves directly without code changes.
  - If unconfigured, `role_is_unknown` warns and the resolver cleanly falls through to subsequent entries or built-in defaults (`models.rs:584-605`).

### Supervisor Model & Stronger Escalation Configuration

1. **Base Supervisor Role (`supervisor`)**:
   - Configured in `.bee/config.json` under `models.claude`:
     ```json
     {
       "models": {
         "claude": {
           "supervisor": {
             "kind": "native",
             "model": "claude-3-5-haiku-20241022",
             "effort": "low"
           }
         }
       }
     }
     ```
   - Alternatively, string shorthand: `"supervisor": "haiku"`.

2. **Escalation Mechanism (Pure Configuration & Protocol)**:
   - **Infrastructure / Provider Failures**: Configured via `retry.fallbackChains` (`models.rs:734-918`):
     ```json
     {
       "retry": {
         "fallbackChains": {
           "supervisor": ["claude-3-5-haiku-20241022", "claude-3-5-sonnet-20241022"]
         }
       }
     }
     ```
   - **Semantic / Anomaly Escalation**:
     - When the supervisor detects a high-severity anomaly (e.g. struggling loop, boundary violation, unhandled error cascade), it does not attempt complex synthesis on Haiku.
     - It writes an intervention flag / wake report citing the observation and triggers an advisor consult (`resolve_advisor` in `models.rs:672-682`, which resolves the high-tier `advisor` model, e.g. Opus/Sonnet) or posts an open question to the Lead / human in the chat pane.

### Daily Token & Economic Cost Model (15-minute Heartbeat)

- **Cadence**: 4 ticks per hour = 96 ticks per 24-hour day.
- **Estimated Payload per Tick**:
  - Input (prompt + session list + waiting-on marks + recent activity + diff digest): ~3,000 tokens.
  - Output (heartbeat status / signal evaluation / short note): ~300 tokens.
- **Pricing Basis** (Claude 3.5 Haiku: $0.80 / MTok in, $4.00 / MTok out):
  - Daily Input: $96 \times 3{,}000 = 288{,}000$ tokens $\rightarrow$ **$0.230** / day.
  - Daily Output: $96 \times 300 = 28{,}800$ tokens $\rightarrow$ **$0.115** / day.
  - **Total Baseline Cost**: **~$0.35 / day** (~$10.50 / month).
- **Escalated Ticks** (Sonnet 3.5 on anomaly, ~5% of ticks):
  - Incremental cost: ~5 ticks $\times$ $0.015 $\approx$ **+$0.08 / day**.

---

## 4. Operational Invariant & Boundary Compliance

All mechanisms strictly maintain the boundaries established in `docs/knowledge/areas/bee-herding/overview.md`:

| Invariant | Requirement | How Supervisor Complies |
|---|---|---|
| **R2: Merge stays human** | Merge is a single-shot gesture (`herding-adopt D11`), never unattended or looped. | Supervisor is strictly an observer. It never invokes `bee worktree merge`, never resolves merge conflicts, and never touches main landing logic. |
| **R3: Dispatch behind owner interlock** | Unattended dispatch requires the durable enable marker (`.bee/markers/dispatch-enabled`). | Supervisor does not claim PBIs or dispatch working agents. It only inspects running sessions, evaluates health signals, and records observations. |
| **R4: Permission split** | Control panes run with narrow enumerated `--allowedTools` (`D7`/`D13`); working agents run open inside isolated worktrees. | The supervisor runs with an enumerated read/query surface (`Read`, `Bash(.bee/bin/bee:*)`, `Bash(tmux:capture-pane:*)`). It is never granted `bypassPermissions`. |

---

## 5. Architectural Recommendations for Ticket 002

1. **Adopt Option (c)** as the supervisor runner:
   - Extend `packages/bee-rs/crates/bee/src/herding/control_loop.rs` to support `--role supervisor`.
   - Set the default supervisor interval to 900 seconds (`--interval 900`).
2. **Standardize Role Configuration**:
   - Declare `supervisor` in `models.claude` defaulting to Haiku.
   - Use `advisor` or `ceiling` escalation paths for semantic alarm escalation.
3. **Persist Signal Findings**:
   - Supervisor writes heartbeats and wake reports to durable records (`docs/history/...` or `.bee/tmp/supervisor-wake-report.jsonl`), preserving observations across restarts.

