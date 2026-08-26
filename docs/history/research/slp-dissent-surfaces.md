# Research Digest — Dissent and StopAndAsk Surfaces in Bee (SLP Ticket 005)

- **Date**: 2026-08-26
- **Context**: `docs/discovery/slp-supervisor-lead-peer/tickets/005-dissent-stop-and-ask.md`
- **Scope**: SLP Cluster 2 (Dissent / StopAndAsk) — audit of existing `bee` surfaces, worker prompt contracts, schema gaps, and response obligations.

---

## 1. Executive Summary & Verdict

### Core Questions
1. What exists in `bee` today for a dispatched worker to object or stop-and-ask?
2. What is missing for SLP-style Dissent (`{target, claim, reasoning, alternative, severity}`) and StopAndAsk (`{boundary_hit, options, leaning}`)?
3. Does anything obligate the orchestrator to respond before work continues?

### Summary Answers
- **Current State in `bee`**: **No structured dissent or stop-and-ask channel exists in `bee` today.**
  - Workers in `bee` are strictly headless executors (`skills/bee-swarming/SKILL.md:155` states: *"Never wait silently; never ask a blocking question — you run headless"*).
  - Workers return only one of four status tokens: `[DONE]`, `[BLOCKED]`, `[HANDOFF]`, `[NOOP]`.
  - When an unexpected condition occurs, workers either make a silent departure and record it at cap time (`trace.deviations`), or abort execution immediately with `[BLOCKED]`.
- **Audit of Existing Surfaces**:
  1. `bee cells escalate`: Purely a compute/model tier selector door (runs a cell on the top session model under a 40% ration budget). It carries no objection, no argument, and is operated by the orchestrator, not the worker.
  2. **Cap Fields / Cells Schema**: No `concerns` or `suggestions` fields exist. The cap schema only accepts `deviations` (restricted to 4 closed categories), `friction`, and `sync_ack`. These fields record data *after* work completes, not before or during work.
  3. `bee state waiting-on set --kind question`: Designed for session-level turn marking between the orchestrator and human. Workers run headless and do not use this verb. It stores a single unstructured `subject` string and cannot handle concurrent worker questions.
  4. **The Mailboxes**:
     - `.bee/human-mailbox/`: Asynchronous letter store for humans about departures and run summaries.
     - `.bee/mailbox/<job-id>/`: Round-based file transport for herded workers. `result-N.json` schema is binary (`status: "done" | "blocked"`) with no payload structure for dissent or trade-off options.
  5. **Worker Prompt Contracts**: `skills/bee-swarming` explicitly commands workers to obey assigned specs without argument, forbids mid-flight questions, and limits multi-step interaction to 2 debugging advisor consults on test failure.
- **Orchestrator Obligation**: **Zero obligation exists.** The orchestrator is not required by code, schema, or lifecycle gate to respond to worker feedback, log decisions, or arbitrate trade-offs before continuing work.

---

## 2. Detailed Audit of Surfaces in Bee

### Surface 1: `bee cells escalate`

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1404-1480`: `set_escalation` and `run_escalate` implementation (`bee cells escalate --id <id> [--reason <text>] [--off]`).
  - `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:50-118, 354-365`: `ESCALATE_FIELD = "escalate"`, `cell_is_escalated`.
  - `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:942-974`: Dispatch model resolution (reads `cell_is_escalated` to omit `model` parameter and emit `[bee-tier: ceiling]`).
  - `skills/bee-swarming/references/swarming-reference.md:166-179`: Escalation usage guidance.

- **What It Can Carry**:
  - Sets a boolean `escalate: true` on `.bee/cells/<id>.json`.
  - Optional `--reason <text>` to override the 40% ceiling share refusal (`CEILING_SHARE_REFUSAL_MAX = 0.40`). The text is stored in `trace.escalation_reason`.

- **Who Reads It**:
  - `bee dispatch prepare`: Assigns the session model instead of the worker's role model.
  - `bee status` / `status_full`: Emits `role_mix` and the percentage of escalated cells.
  - Session preamble: Injects warnings when the escalated cell share is high.

- **Gap vs SLP Dissent & StopAndAsk**:
  - `bee cells escalate` is a resource management lever (compute allocation), not an objection channel.
  - It carries no target decision, no claim, no technical reasoning, no alternative proposal, and no boundary options.
  - Workers do not invoke it. The orchestrator invokes it at dispatch or during error recovery.

---

### Surface 2: Cell Cap Fields & Cells Schema (`concerns`, `suggestions`, `deviations`)

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:55-189`: `REPORT_KEYS = ["outcome", "commit", "files", "tests", "deviations"]`, `parse_report_flag`.
  - `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:84-127`: `CapFlags` struct (`id`, `outcome`, `friction`, `files_changed`, `deviations`, `deviation`, `override_reason`, `commit_pending`, `inline_reason`, `report`, `sync_ack`).
  - `skills/bee-swarming/references/worker-details.md:94-109`: Departure line format (`<what> — <why> — <kind>`).
  - `packages/bee-rs/crates/bee/src/verbs/knowledge/promote.rs:1-120`: Offline pattern mining from `trace.deviations`.

- **What It Can Carry**:
  - **No `concerns` or `suggestions` fields exist** in the cell schema or cap flags.
  - `deviations`: Array of strings or structured objects (`{what, why, kind}`). The `kind` field is restricted to four closed values:
    1. `hit an unforeseen obstacle`
    2. `found a better route`
    3. `the plan was wrong about a fact`
    4. `something else had to be fixed first`
  - `friction`: One-line trigger note from a closed list of 6 friction triggers.
  - `sync_ack`: Reason string for touching code outside predicted skill/rule boundaries.

- **Who Reads It**:
  - `cap_cell_from_flags`: Writes data to `.bee/cells/<id>.json` (`trace.deviations`, `trace.report`, `trace.sync_ack`).
  - `bee-capturing` & pattern promotion: Reads `trace.deviations` offline to propose rule updates.
  - `.bee/human-mailbox/`: Reads deviations to compile letters for humans.

- **Gap vs SLP Dissent & StopAndAsk**:
  - Cap fields are recorded at the **end** of execution (`bee finish` / `cells cap`). They cannot stop work before implementation or ask questions during implementation.
  - The four deviation kinds describe actions already completed, not proposed alternatives or objections.
  - The orchestrator validates JSON structure and test status only. It does not review, arbitrate, or respond to deviations during the swarming loop.

---

### Surface 3: `bee state waiting-on set --kind question`

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/state_group/waiting_on.rs:80-260`: `resolve_waiting_on_target`, `run_waiting_on_set`, `run_waiting_on_clear`.
  - `packages/bee-rs/crates/bee/src/verbs/workflow_store/record.rs:360-445`: `build_waiting_on`, `WAITING_ON_KIND_VALUES = ["gate", "question", "turn-end"]`.
  - `packages/bee-rs/crates/bee/src/hooks/activity.rs:44-50, 276-291`: Activity hook sync for `waiting_on`.
  - `RULE[/home/thanhsmind/Projects/goglbe/beehive/AGENTS.md]`: Communication rules for waiting marks.

- **What It Can Carry**:
  - Stored object in `.bee/state.json` or `.bee/runtime/workflows/<id>/state.json`:
    ```json
    {
      "kind": "question",
      "subject": "<question text>",
      "asked_at": "<ISO-8601 timestamp>",
      "session": "<session-id>"
    }
    ```
  - Carries only a single flat text string (`subject`).

- **Who Reads It**:
  - Session preamble: Injects the waiting note into the prompt context for the next turn.
  - Activity hook & screen classifier: Monitors whether a session is blocked on user input.
  - `bee status` / `bee status --json`: Reports waiting status to users and cockpit displays.
  - Cleared automatically on `UserPromptSubmit` when the human responds.

- **Gap vs SLP Dissent & StopAndAsk**:
  - Scoped exclusively for orchestrator-to-human communication.
  - Dispatched workers run headless and are explicitly prohibited from setting waiting marks or asking blocking questions.
  - The store supports only one `waiting_on` object per workflow. Multiple concurrent workers would overwrite each other's state.
  - It provides no mechanism to deliver structured options or receive an orchestrator decision.

---

### Surface 4: The Mailboxes (`bee mailbox` and `.bee/mailbox/<job-id>/`)

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/mailbox.rs:1-2007`: Human mailbox store (`.bee/human-mailbox/`, `bee mailbox mark`).
  - `packages/bee-rs/crates/bee/src/herding/mailbox.rs:1-475`: Herded worker file mailbox (`.bee/mailbox/<job-id>/`, `render_brief`, `MailboxResult`).
  - `packages/bee-rs/crates/bee/src/herding/run.rs:1-350`: Native poll loop for herding results.

- **What It Can Carry**:
  - **Human Mailbox (`.bee/human-mailbox/`)**: Markdown letters (`<timestamp>-<slug>.md`) describing deviations, caps, and run summaries for humans.
  - **Herding Worker Mailbox (`.bee/mailbox/<job-id>/`)**:
    - `brief-N.txt`: Task brief with file and tool boundaries.
    - `ack-N.json`: Delivery receipt (`{nickname, job_id, round, agent, received_at}`).
    - `result-N.json`: Worker outcome with strict schema:
      ```json
      {
        "status": "done" | "blocked",
        "summary": "<one-line description>",
        "files_changed": ["<path>", "..."],
        "proof": "<command and result evidence>"
      }
      ```
    - `activity.json`: Heartbeat and lifecycle status.

- **Who Reads It**:
  - Human Mailbox: Read by the human operator via `bee mailbox mark` or an external UI.
  - Herding Mailbox: Read by `bee herding run` poll loop upon worker completion.

- **Gap vs SLP Dissent & StopAndAsk**:
  - The herding mailbox status is binary (`done` or `blocked`).
  - A blocked worker can only return a flat `summary` and `proof`. It cannot return structured fields (`boundary_hit`, `options`, `leaning`, `target`, `alternative`).
  - When a worker reports `blocked`, the orchestrator can only retry (`--continue`) or abort. No interactive arbitration or decision logging protocol exists.

---

### Surface 5: Worker Prompt Contracts in `skills/bee-swarming`

- **Code Anchors**:
  - `skills/bee-swarming/SKILL.md:115-156`: Worker execution rules (Execute section).
  - `skills/bee-swarming/SKILL.md:72-76`: `[BLOCKED]` rescue ladder.
  - `skills/bee-swarming/references/swarming-reference.md:475-550`: Worker Prompt Template and Result Formats.
  - `skills/bee-swarming/references/worker-details.md:55-65, 172-175, 228-250`: Conformance rules, test failure protocols, and advisor consult loop.

- **What Workers Are Instructed To Do**:
  - *"Execute only the assigned cell... never select or accept other work."*
  - *"Never reinterpret a locked decision to make the cell fit."*
  - *"When reality disagrees with the cell: a bug in touched code → fix it, record the deviation; a missing piece the outcome depends on → add it, record; blocking breakage in your path → fix, record; anything architectural → `[BLOCKED]`."*
  - *"Never wait silently; never ask a blocking question — you run headless."*
  - Return exactly one status token: `[DONE]`, `[BLOCKED]`, `[HANDOFF]`, `[NOOP]`.
  - **Advisor Consult**: On the first test failure, a worker may consult an assigned advisor up to 2 times for debugging assistance before returning `[BLOCKED]`.

- **Gap vs SLP Dissent & StopAndAsk**:
  - Workers are explicitly instructed **not** to challenge decisions or ask questions.
  - Workers lack boundary sensitivity instructions (for example, identifying when an internal implementation detail changes a product trade-off, such as changing coordinate precision from `int16` to `int8`).
  - Workers have no vocabulary for proposing out-of-frame solutions (Proposal C).

---

## 3. Obligation to Respond Analysis

### Does Anything Obligate the Orchestrator to Respond?

**No. There is zero obligation in `bee` today.**

1. **No Intermediate Communication Channel**:
   - Communication between orchestrator and worker is an asynchronous batch dispatch.
   - The worker executes until it finishes or fails. It cannot pause and wait for an answer.

2. **No Response Requirements on `[BLOCKED]`**:
   - When a worker returns `[BLOCKED]`, the orchestrator follows an optional rescue ladder:
     1. Re-dispatch with added context.
     2. Escalate compute model via `bee cells escalate --id <id>` and re-dispatch.
     3. Surface the blocker to the human user.
   - The orchestrator is not required to log a formal response or choose from the SLP three-way response model:
     - Accept dissent and append to `bee decisions`.
     - Reject dissent with technical reasoning.
     - Escalate to Rung 3 (open multi-lane blind design).

3. **No Gate Enforcement**:
   - `bee close` and `bee worktree merge` check that capped cells have valid proof lines.
   - They do not check for unresolved worker objections, unaddressed deviations, or rejected alternatives.

---

## 4. Comparison Matrix: Bee Surfaces vs SLP Specification

| Field / Requirement | SLP Specification | Existing `bee` Surface | Status & Gap in `bee` |
|---|---|---|---|
| **Dissent Payload** | `{target, claim, reasoning, alternative, severity}` | None (`trace.deviations` has only `{what, why, kind}`) | **Missing**. No schema exists for targeting a decision, declaring severity (`blocker` vs `should-consider`), or submitting alternative proposals (Proposal C). |
| **Dissent Timing** | Raised **before** or **during** execution to pause work | End-of-cell cap only (`cells finish --report`) | **Missing**. Deviations are reported post-hoc after code is written and committed. |
| **StopAndAsk Payload** | `{boundary_hit, options[], leaning}` | None (nearest is `[BLOCKED]` prose summary) | **Missing**. No structured format for reporting boundary crossings with multi-option trade-offs and worker leanings. |
| **StopAndAsk Timing** | Mid-cell interruption; pauses affected work | `[BLOCKED]` token aborts execution entirely | **Missing**. Workers run headless and cannot pause for interactive answers. |
| **Orchestrator Response** | Exactly 1-of-3 required: (1) Accept + log `decision_log`, (2) Reject + reasoning, (3) Escalate to Rung 3 lane council | Optional rescue ladder (re-dispatch, escalate model, or report to human) | **Missing**. No command, record, or gate enforces a 1-of-3 response. |
| **Boundary Sensitivity Rules** | Written in worker prompt: API changes, data quality vs performance trade-offs, new dependencies | Conformance habits in `worker-details.md` (check helpers, match style, check imports) | **Missing**. Instructions do not warn against silent trade-offs (e.g. `int16` to `int8` quantization or minting unapproved APIs). |
| **Response Gate Teeth** | Work cannot proceed until Dissent/StopAndAsk is resolved | Proof checks at close/merge only (`<cmd> — <result> — <reason>`) | **Missing**. No lifecycle barrier blocks merge or close on open objections. |

---

## 5. Architectural Recommendations for Ticket 005 Implementation

To support SLP Dissent and StopAndAsk in `bee`, the following adaptations are recommended:

1. **Worker Protocol & Schema Extensions**:
   - Add structured status tokens and result schemas for workers:
     - `[DISSENT]`: `{target: "<decision-id>", claim: "<text>", reasoning: "<text>", alternative: "<text>", severity: "blocker" | "should-consider"}`.
     - `[STOP_AND_ASK]`: `{boundary_hit: "<boundary>", options: [{"name": "<str>", "tradeoff": "<str>"}], leaning: "<str>"}`.
   - Update mailbox result schemas (`result-N.json`) in `packages/bee-rs/crates/bee/src/herding/mailbox.rs` to validate these payloads.

2. **Orchestrator 1-of-3 Response Mechanism**:
   - Introduce a response CLI verb:
     ```bash
     bee cells answer-dissent --id <cell-id> --action accept|reject|escalate --reason "<rationale>"
     ```
   - Actions:
     - `accept`: Automatically appends an entry to `.bee/decisions.jsonl` via `bee decisions log --relation touches:<target>`.
     - `reject`: Records the rejection rationale on the cell trace and re-dispatches the worker.
     - `escalate`: Marks the cell for multi-lane blind design (Rung 3).

3. **Worker Prompt Instructions (Peer Voice)**:
   - Update `skills/bee-swarming/references/worker-details.md` and prompt templates:
     - Add explicit boundary triggers: public API changes, data quality or UX degradation for performance targets, new external dependencies, or contract minting.
     - Instruct workers to Dissent when a plan is technically flawed, and StopAndAsk when hitting a boundary.

4. **Lifecycle Enforcement**:
   - Enforce in `bee close` and `bee worktree merge` that no cell in the active slice remains in an unresolved dissent state.

---

File created: `docs/history/research/slp-dissent-surfaces.md`
