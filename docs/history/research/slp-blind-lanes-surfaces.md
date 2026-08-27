# Research Digest — Blind Design Lanes and Convergence Surfaces in Bee (SLP Ticket 006)

- **Date**: 2026-08-26
- **Context**: `docs/discovery/slp-supervisor-lead-peer/tickets/006-blind-lanes.md`
- **Scope**: SLP Cluster 3 (Blind Lanes for Critical Decisions) — isolation audit, leak channels, workflow mapping, and convergence logging.

---

## 1. Executive Summary & Verdict

### Core Questions
1. What mechanisms exist in `bee` today to execute 2–3 isolated parallel design sessions and converge?
2. What mechanisms are missing for SLP blind lanes?

### Summary Findings & Verdict
- **Current Multi-Worker Capabilities**: `bee` supports parallel execution of approved code tasks via `bee dispatch wave` and `dispatch prepare`. However, `bee` currently has **no multi-agent blind design protocol**.
- **Isolation Capabilities & Leak Vectors**:
  - *Filesystem isolation*: Granted git worktrees (`bee worktree new --feature <slug>`) isolate code files, unstaged modifications, and local decisions (`.bee/decisions.jsonl`).
  - *Shared state leak vectors*: The shared control plane in main (`.bee/logs/dispatch.jsonl`, `.bee/sessions.jsonl`, `.bee/workers.jsonl`, `.bee/mailbox/`), git history/branch references, session preambles (injecting recent decisions), and prompt framing can expose design ideas before cross-critique.
- **Gray-Area Handling in Existing Skills**: `bee-shaping`, `bee-planning`, and `bee-wayfinding` process ambiguities through serial human interviews or single-worker research tickets. No existing skill fans out competing design options.
- **Decision Log Schema Support**: The current schema of `bee decisions log` **fully supports** convergence records today without schema changes:
  - Chosen option maps to `--decision`.
  - Rationale maps to `--rationale`.
  - Rejected options and rejection reasons map to `--alternatives`.
  - Revisit triggers map to `--trigger` (`.bee/triggers/`).
  - Active decision relationships map to `--relation supersedes:<id>|touches:<id>|none`.

---

## 2. Detailed Audit by Anchor

### Anchor 1: `bee dispatch wave` (Coordination Scope)

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1942-2200`: `dispatch wave` batch preparation engine.
  - `packages/bee-rs/crates/bee/src/verbs/cells/schedule.rs:1-350`: Topological dependency wave scheduler (`compute_schedule`).
  - `packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:1-400`: Claim and reservation manager.

- **What `dispatch wave` Coordinates**:
  - Operates exclusively during the `swarming` phase for cell execution.
  - Computes independent execution batches from the directed acyclic graph (DAG) of open cells.
  - Claims each cell under an auto-derived worker nickname `w-<cell_id>`.
  - Registers worker rows in `.bee/workers.jsonl` and acquires path reservations in `.bee/reservations.jsonl`.
  - Renders execution payloads with `worker-cell.md` template containing inlined cell JSON.
  - Isolates errors per cell: a skip (e.g. `reservation_conflict` or `already_claimed`) unwinds that cell and continues the wave.

- **Why It Does NOT Solve Blind Lanes**:
  - *Cooperative, not competitive*: Assumes all workers build complementary parts of one approved plan.
  - *Disjoint file writes*: Enforces path reservations to prevent file collisions. Blind design lanes need to propose alternative solutions for the *same* design problem.
  - *Requires approved cells*: Cannot run before Gate 2 approval. Blind lanes are needed before or during Gate 1 / Gate 2 shaping.

---

### Anchor 2: `dispatch prepare` Kinds and Model Roles

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:750-1150`: Kind validation and role resolution.
  - `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:1-250`: Open model role fall-through system.
  - `packages/bee/prompts/`: Built-in prompt templates (`advisor.md`, `gather.md`, `reviewer.md`, `worker-cell.md`).

- **Existing Kinds in `dispatch prepare`**:
  1. `cell`: Execution worker. Requires `--cell` and `--worker`. Acquires claims and reservations.
  2. `gather`: Read-only data extraction. Uses `gather.md` template and `bee-gather` subagent.
  3. `reviewer`: Read-only verification against the repository. Uses `reviewer.md` template and `bee-review` subagent.
  4. `advisor`: Independent technical opinion on a specific question. Uses `advisor.md` template. Requires dedicated `advisor` model slot.

- **Relevance and Limitations for Blind Lanes**:
  - The `advisor` kind and open model roles (`models.<runtime>.<role>`) provide the infrastructure to query independent models.
  - *Missing*: `dispatch prepare` has no `lane` or `design` kind. It lacks templates for isolated design proposal generation, prompt de-biasing, or reciprocal cross-critique.

---

### Anchor 3: State Isolation vs Information Leak Vectors

When 2–3 design lanes run in parallel, information can leak across several channels:

| Isolation Boundary | Protection Level | Leak Risks & Vectors |
|---|---|---|
| **Git Working Tree** | High (if in separate worktrees) | If workers run in separate worktrees (`bee worktree new`), file changes remain isolated. If workers run in the same checkout, file writes collide immediately. |
| **Data Plane (`.bee/decisions.jsonl`)** | High (in granted worktrees) | Granted worktrees maintain a local `.bee/decisions.jsonl`. Uncommitted decisions do not cross worktree boundaries. |
| **Control Plane (`.bee/logs/dispatch.jsonl`)** | None (Shared in main) | All dispatches append to main `.bee/logs/dispatch.jsonl`. A subagent inspecting dispatch logs can read prompts sent to other lanes. |
| **Session Registry & Mailbox** | None (Shared in main) | `.bee/sessions.jsonl`, `.bee/workers.jsonl`, and `.bee/mailbox/` reside in main checkout and are visible to any tool reading `.bee/`. |
| **Git Commit & Branch History** | Low | If a lane commits to a local git branch, sibling lanes running `git log --all` or `git branch` can inspect commit messages. |
| **Preamble Decision Injection** | Medium Leak Risk | `bee orient` and standard preambles inject recent active decisions. If Lane A logs a decision before Lane B runs, Lane B receives Lane A's decision. |
| **Orchestrator Prompt Framing** | High Leak Risk | The Lead / Orchestrator can inadvertently inject framing bias, preferred libraries, or partial solutions into the prompt. |

---

### Anchor 4: Gray-Area Handling in Existing Skills

- **`bee-shaping` (`skills/bee-shaping/SKILL.md`)**:
  - `Explore`: Uses single-thread interactive interviews with the user to resolve 2–4 unstated product decisions.
  - `Qualify`: Headless triage. Evaluates risk and clarity. If an item is vague, it parks the item in `Outstanding Questions` or creates a map stub (`bee discovery stub`).
  - `Lock`: Acts as the single writer for `docs/history/<feature>/CONTEXT.md`.
  - *Gap*: Employs no multi-agent debate or parallel proposal generation.

- **`bee-planning` (`skills/bee-planning/SKILL.md`)**:
  - Classifies lanes (`tiny`, `small`, `standard`, `high-risk`, `spike`).
  - Performs "SMALLER PATH" checks.
  - Uses `bee-researching` for evidence collection on unfamiliar dependencies.
  - *Gap*: Creates a single deterministic plan. Does not draft competing architecture proposals.

- **`bee-wayfinding` (`skills/bee-wayfinding/SKILL.md`)**:
  - Charts ambiguous initiatives into a map (`MAP.md`) and numbered tickets (`tickets/NNN-*.md`).
  - Spawns parallel research tickets via `bee-researching`.
  - *Gap*: Research subagents investigate orthogonal sub-questions (e.g. database schema vs UI design), not competing solutions for the same critical decision.

---

### Anchor 5: Convergence Record Mapping in `bee decisions log`

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs:200-300`: `LogParams` struct definition.
  - `packages/bee-rs/crates/bee/src/verbs/decisions/verbs_write.rs:533-730`: `do_log` decision event builder.
  - `docs/product-description/memory/decisions.md:1-168`: Decision log schema specification.

- **Decisions Schema Fields Audit**:

| SLP Convergence Field | `bee decisions log` CLI Option | Stored JSONL Event Field | Field Type & Validation |
|---|---|---|---|
| **Chosen Option** | `--decision "<text>"` | `"decision"` | String (required, safe prose checked) |
| **Rationale / Reasons** | `--rationale "<text>"` | `"rationale"` | String (required, safe prose checked) |
| **Rejected Options & Reasons** | `--alternatives "<text>"` | `"alternatives"` | String / Null (optional) |
| **Revisit Trigger Condition** | `--trigger "<trigger_id>"` | `"trigger"` | String (validated against `.bee/triggers/`) |
| **Relation to Existing Rules** | `--relation supersedes:<id>\|touches:<id>\|none` | `"supersedes"` / `"touches"` / `"relation"` | Array of IDs / String (required) |
| **Feature Attribution** | `--feature "<slug>"` | `"feature"` | String (defaults to bound lane) |
| **Confidence Level** | `--confidence <N>` | `"confidence"` | Number (optional integer) |
| **Topic Categorization** | `--tags "<tag1,tag2>"` | `"tags"` | Array of String slugs |

- **Schema Readiness Verdict**: **100% Ready**. The existing `bee decisions log` schema natively captures chosen options, rejected alternatives with rationales, and revisit triggers.

---

## 3. Mapping SLP Blind Lane Artifacts onto Bee

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           1. Lead Framing                               │
│  - Receives critical design decision (Gate Escalation / Gray Area)      │
│  - Authors LaneBrief (neutral constraints, objective eval criteria)     │
│  - Strips leading framing bias                                          │
└──────────────────┬──────────────────────────────────┬───────────────────┘
                   │                                  │
                   ▼ (Parallel Dispatches)            ▼
┌──────────────────────────────────────┐  ┌───────────────────────────────┐
│        2a. Lane A (Model Alpha)      │  │    2b. Lane B (Model Beta)    │
│  - Isolated context                  │  │  - Isolated context           │
│  - Produces LaneProposal A           │  │  - Produces LaneProposal B    │
└──────────────────┬───────────────────┘  └───────────┬───────────────────┘
                   │                                  │
                   ▼ (Cross-Critique Phase)           ▼
┌──────────────────────────────────────┐  ┌───────────────────────────────┐
│        3a. Critique by Lane A        │  │    3b. Critique by Lane B     │
│  - Praises strongest point of B      │  │  - Praises strongest point of A│
│  - Self-critiques weakest point of A │  │  - Self-critiques weakest of B│
└──────────────────┬───────────────────┘  └───────────┬───────────────────┘
                   │                                  │
                   └──────────────────┬───────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                   4. Convergence & Synthesis (Lead)                     │
│  - Compiles ConvergenceDossier                                          │
│                                                                         │
│  ├── [Clear Winner / Consensus]                                         │
│  │   └── Log settled agreement:                                         │
│  │       `bee decisions log --decision "…" --rationale "…" \`           │
│  │       `  --alternatives "…" --trigger "…" --relation "…"`            │
│  │                                                                      │
│  └── [Deadlock / Unresolved Trade-offs]                                 │
│      └── Escalate to Human:                                             │
│          `bee state waiting-on set --kind gate --subject "..."`         │
│          Present full ConvergenceDossier (No coin flips)                │
└─────────────────────────────────────────────────────────────────────────┘
```

### Artifact Definitions & Storage Locations

1. **`LaneBrief`**:
   - *Contents*: Verbatim `original_request`, problem statement, hard technical constraints, and evaluation criteria.
   - *Storage*: Rendered directly into subagent dispatch prompts, or saved to `docs/history/<feature>/lanes/brief.md`.
2. **`LaneProposal`**:
   - *Contents*: Architectural structure, trade-off analysis, migration impact, and implementation steps.
   - *Storage*: Returned in subagent results, or stored under `docs/history/<feature>/lanes/proposal-lane-1.md`.
3. **`CrossCritique`**:
   - *Contents*: Mandatory reciprocal critique:
     1. Praise strongest aspect of competing proposal.
     2. Acknowledge weakest aspect of own proposal.
     3. Objective technical rebuttal of remaining differences.
4. **`ConvergenceDossier`**:
   - *Contents*: Complete dossier containing `LaneBrief`, all `LaneProposals`, `CrossCritiques`, comparison matrix, and Lead synthesis.
   - *Storage*: `docs/history/<feature>/lanes/convergence-dossier.md` (or embedded in `CONTEXT.md`).
5. **Deadlock Escalation**:
   - If convergence reveals equally viable trade-offs or fundamental architectural deadlocks, the orchestrator sets `bee state waiting-on set --kind gate` and delivers the `ConvergenceDossier` to the human.

---

## 4. Gaps and Required Additions for SLP Blind Lanes

To fully implement SLP blind lanes in `bee`, four capabilities must be added:

1. **Neutral `LaneBrief` Prompt Sanitizer**:
   - Add prompt validation to remove subjective phrases (e.g. "I prefer", "we should probably use", unconstrained technology hints) before dispatching design workers.
2. **Blind Dispatch Orchestrator**:
   - Extend `bee dispatch prepare` (or a dedicated orchestration workflow) with a `lane` / `design` kind.
   - Dispatch workers across diverse models (e.g. Claude 3.7 Sonnet, OpenAI o3-mini, Gemini 2.5 Flash) via model role configuration.
3. **Two-Stage Blind Execution Protocol**:
   - *Stage 1*: Execute parallel blind proposal generation into isolated in-memory buffers or isolated markdown files.
   - *Stage 2*: Re-dispatch cross-critique prompts exchanging proposal summaries without revealing author identity or orchestrator preference.
4. **Deadlock Escalation Surface**:
   - Integrate the `ConvergenceDossier` output into the human gate review surface when consensus fails.

---

## 5. Verification & References

- `docs/specs/slp-supervisor-lead-peer/slp-supervisor-lead-peer.md` (Sections 3, 4.5, 4.6, 5-S2).
- `docs/specs/slp-supervisor-lead-peer/slp-supervisor-lead-peer-detail.md` (Chapter 2: Blind design and conformity mitigation).
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` (`dispatch wave` and `dispatch prepare`).
- `packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs` (`do_log` and decision schema fields).
- `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs` (Open model roles).
