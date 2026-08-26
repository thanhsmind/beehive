# Research Digest — Contract Status and Original Request Surfaces in Bee (SLP Ticket 007)

- **Date**: 2026-08-26
- **Context**: `docs/discovery/slp-supervisor-lead-peer/tickets/007-contract-status-original-request.md`
- **Scope**: SLP Cluster 4 (Contract Status Labels & Verbatim Original Request Transmission) — audit of existing bee state, schemas, dispatch pipelines, and recommended minimum viable mechanisms.

---

## 1. Executive Summary & Verdict

### Core Questions (from Ticket 007)
1. **Question A**: What exists in `bee` today that could carry a per-contract `CHỐT` / `CHƯA-CHỐT` (settled / unsettled) label an agent can cite before writing tests, preventing test minting on unsettled interfaces?
2. **Question B**: Where would a verbatim `original_request` live so it rides every dispatch untouched across all execution layers?

### Summary Findings & Verdict

- **Question A (Contract Status Labels)**:
  - `bee` currently has **no dedicated per-contract `status` field** in cells, plans, or knowledge records.
  - However, `bee` possesses three strong adjacent surfaces:
    1. `docs/history/<feature>/CONTEXT.md` `## Locked Decisions` table (canonical human/agent consensus artifact).
    2. `.bee/decisions.jsonl` append-only store with active projections (`active_decisions()`), tag taxonomies (`--tag`), and conditional triggers (`.bee/triggers/`).
    3. `must_haves` in cells (`truths`, `artifacts`, `invariants`), which act as per-cell behavioral contracts but not interface stability registries.
  - **Verdict**: A per-contract `CHỐT` / `CHƯA-CHỐT` label can be implemented cleanly as a **view over active decisions** (tagged `contract:<name>`) backed by a standardized `## Contracts` section in `CONTEXT.md`.

- **Question B (Verbatim Original Request Transmission)**:
  - Cell schema does **not** contain `goal` or `notes` or `original_request` fields (cells contain only `title` and `action`).
  - `bee` **already contains a purpose-built subsystem for immutable verbatim requests**: **`bee intent` (The Intent Anchor)** stored at `.bee/intent/<key>.json`.
  - The Intent Anchor already enforces `request: "VERBATIM — never trimmed, never truncated, never re-wrapped. Immutable once set"`, survives compactions, and tracks acceptance criteria (`acceptance`).
  - Currently, `bee dispatch prepare` does not pass the intent anchor to worker prompts.
  - **Verdict**: The verbatim `original_request` belongs in `.bee/intent/<feature>.json`, transmitted to workers by having `dispatch prepare` inject `original_request` into the `packages/bee/prompts/worker-cell.md` template.

---

## 2. Detailed Audit: Question A — Per-Contract Stability (CHỐT / CHƯA-CHỐT)

### Candidate A1: Locked Decisions in `docs/history/<feature>/CONTEXT.md`

- **Code & Doctrine Anchors**:
  - `skills/bee-shaping/references/context-template.md`: Defines the canonical `CONTEXT.md` structure.
  - `skills/bee-swarming/references/worker-details.md:62`: "referenced decision IDs resolve in `CONTEXT.md` and do not contradict the action".
  - `skills/bee-reviewing/references/reviewing-reference.md:90`: "check every suspected break against the locked decisions in the in-scope `CONTEXT.md`".

- **How Decisions are Structured and Cited**:
  - Structured in a markdown table: `| ID | Decision | Rationale |` with immutable IDs (`D1`, `D2`, etc.).
  - Followed by `## Outstanding Questions` (`### Resolve Before Planning`, `### Deferred To Planning`) and `## Deferred Ideas`.
  - Downstream cells cite them via the `cell.decisions` array (e.g. `["D1", "D2"]`) and in prose (`per D2`).

- **Fit for Contract Status**:
  - `CONTEXT.md` is the authoritative source of truth for all downstream planning and execution agents.
  - Produced at Gate 1 (shaping) and frozen before Gate 2 (planning/execution).
  - Natural human-visible surface for reviewing and approving interface commitments.

- **Gaps & Deficiencies**:
  - **No per-contract status field**: Decisions are statements of intent, not explicit interface states (`CHỐT` vs `CHƯA-CHỐT`).
  - **Feature-scoped**: `CONTEXT.md` lives in `docs/history/<feature>/`, making cross-feature interface contracts fragmented.
  - **Unstructured prose**: Requires LLM reading rather than deterministic detector parsing (e.g., regex/JSON inspection by a cheap detector or test pre-flight).

---

### Candidate A2: `docs/knowledge` Concept and Area Records (OKF)

- **Code & Schema Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/knowledge/frontmatter.rs`: YAML frontmatter parser and emitter.
  - `packages/bee-rs/crates/bee/src/verbs/knowledge/frame.rs:134-168`: `CONCEPT_TYPES` (`bee.area`, `bee.feature`, `bee.pattern`, `bee.decision`, etc.) and `BEE_KEY_ORDER`.
  - `packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs`: Conformance checks for knowledge documents.
  - `docs/knowledge/areas/` and `docs/knowledge/patterns/`: Repo-wide knowledge store.

- **Status and Lifecycle Fields**:
  - Frontmatter schema includes `bee.lifecycle` with standard values: `active`, `draft`, `superseded`.
  - Also includes `bee.review_status`, `bee.authoritative_for`, `owns.code`, `owns.tests`.

- **Fit for Contract Status**:
  - Repo-wide, durable, searchable via `bee knowledge search` and `bee knowledge list`.
  - Machine-validated by `bee knowledge check` against strict schema rules.
  - Supports `authoritative_for` to claim ownership over specific subsystems or APIs.

- **Gaps & Deficiencies**:
  - **Wrong abstraction layer**: `bee.lifecycle` (`active`/`draft`) describes the state of the *documentation article*, not whether a specific internal/external code interface is settled for TDD.
  - **High ceremony**: Creating or updating OKF files requires frontmatter metadata (`title`, `description`, `bee.id`, `bee.lifecycle`, `sources`, etc.), which is too heavy for rapid in-flight API design during feature development.
  - **No contract-level granularity**: An area doc covers an entire domain, not individual function signatures or wire formats.

---

### Candidate A3: `bee decisions` Tags, Relations, Triggers & Active View

- **Code & Spec Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/decisions/verbs_write.rs`: CLI handler for `bee decisions log`, `tag`, `supersede`.
  - `packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs`: CLI handler for `bee decisions active`, `search`.
  - `packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs`: Registry for deferred decisions (`.bee/triggers/<slug>__<short8>.json`).
  - `docs/knowledge/areas/decision-memory/overview.md`: Business rules R1–R9.

- **Existing Capabilities**:
  - **Structured logging**: Each event records `id`, `date`, `decision`, `rationale`, `alternatives`, `scope`, `tags[]`, `feature`.
  - **Mandatory relations**: `bee decisions log --relation supersedes:<id>|touches:<id>|none` ensures active truth is updated without stale graph drift.
  - **Active view (`active_decisions()`)**: Automatically projects the current active consensus by pruning superseded/redacted records and applying tag overlays.
  - **Deferred triggers**: If a decision cannot be settled, `bee triggers add --decision <id> --condition "..." [--predicate <p>]` registers a persistent trigger in state `waiting` or `due`.

- **Fit for Contract Status (Label as a VIEW over Active Decisions)**:
  - **High**: A contract stability label fits perfectly as a **projected view over active decisions**:
    - A decision logged with tag `contract:<interface_name>` and `--relation none|supersedes:...` is `CHỐT` (settled).
    - If a decision links to an unresolved trigger (`status: waiting|due`), or if no active decision exists for that interface, it is `CHƯA-CHỐT` (unsettled).
  - Can be queried natively via `bee decisions search --tag contract:<name> --json` or `bee decisions active --json`.
  - When an interface changes, `bee decisions log --relation supersedes:<old_id>` instantly updates the status and triggers a citation sweep.

- **Gaps & Deficiencies**:
  - No dedicated CLI sugar (e.g. `bee contracts list` or `bee contracts status <name>`).
  - Tag taxonomy in `docs/decisions/taxonomy.json` does not yet define a formal `contract:*` namespace convention.

---

### Candidate A4: Cells & Plan Records (`must_haves`, `decisions`, `plan.md`)

- **Code & Schema Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:224-430`: Cell validation rules.
  - `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:462-491`: `normalize_new_cell`.
  - `skills/bee-planning/references/planning-reference.md`: "must_haves are contracts: truths, artifacts, invariants".

- **Existing Fields**:
  - `cell.must_haves`: Object containing `truths` (observable behaviors), `artifacts` (files), `invariants`.
  - `cell.decisions`: Array of decision IDs (`["D1", "D2"]`).
  - `cell.action`: Implementation instructions.
  - `cell.verify`: Verification command.

- **Fit for Contract Status**:
  - Cells are the atomic unit of execution dispatched to workers.
  - Workers read their assigned cell JSON directly at startup (`{{cell_json}}`).

- **Gaps & Deficiencies**:
  - **Ephemeral**: Cells exist only during feature execution (open $\to$ claimed $\to$ capped $\to$ archived); they are not a durable contract catalog.
  - **No status field**: `must_haves` specify what the cell will produce, not whether external dependencies/contracts are settled.
  - **No contract-ref field**: Cells do not carry a `contract_status_refs` field today.

---

## 3. Detailed Audit: Question B — Verbatim `original_request` Transmission

### Candidate B1: Cell Schema Fields (`goal`, `notes`, `action`)

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:230`: Required string fields: `["id", "feature", "title", "action", "verify"]`.
  - `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:473`: Array fields: `["deps", "decisions", "files", "read_first", "affects_skills", "affects_specs"]`.

- **Fit**:
  - Cells are inlined into dispatch prompts via `{{cell_json}}` in `worker-cell.md`.
  - Any field on the cell is automatically available to the worker.

- **Gaps & Deficiencies**:
  - **Fields do not exist**: `goal`, `notes`, and `original_request` do not exist in the cell schema. Only `title` (short label) and `action` (task prose) are present.
  - **Severe payload bloating & redundancy**: Repeating a long verbatim user prompt across 10–20 individual cells of a feature wastes context and duplicates state across `.bee/cells/*.json`.

---

### Candidate B2: `bee intent set/show` (The Intent Anchor)

- **Code & Spec Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/intent_group.rs:1-350`: Native implementation of `bee intent`.
  - `docs/knowledge/areas/hook-runtime/the-intent-anchor-and-compaction-survival.md`: Full specification of the intent anchor.
  - Storage location: `.bee/intent/<key>.json` (where `<key>` is the feature slug or session ID).

- **Data Schema (`.bee/intent/<key>.json`)**:
  ```json
  {
    "schema_version": "1.0",
    "key": "feature-slug",
    "written_at": "2026-08-26T10:00:00Z",
    "request": "<VERBATIM USER ASK — never trimmed, never truncated, never re-wrapped>",
    "acceptance": "<DONE MEANS — explicit sentence of success criteria>",
    "next_action": "<single next step>",
    "feature": "feature-slug",
    "lane": "standard",
    "do_not_reverse": ["<constraint1>", "<constraint2>"],
    "stop_conditions": ["<halt condition>"]
  }
  ```

- **Fit**:
  - **Exact architectural match**: `bee intent` was explicitly designed to store the immutable, verbatim user request to prevent drift across compactions and multi-step executions.
  - Written once per feature at feature initialization (`bee intent set --request "..." --acceptance "..." --feature <slug>`).
  - Immutable by design: the `request` field cannot be modified once set (only `next_action` advances).
  - Already supports critical constraints via `do_not_reverse` and `stop_conditions`.

- **Gaps & Deficiencies**:
  - Currently, `bee intent` is only rendered into session start preambles and compaction capsules (`intent show --render precompact|resume`).
  - It is **not yet wired into worker dispatch generation** in `dispatch prepare`.

---

### Candidate B3: `dispatch prepare` Payload Assembly & Templates

- **Code Anchors**:
  - `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:704-1150`: `prepare_dispatch` and `prepare_dispatch_with_role`.
  - `packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs:33-100`: Prompt loader and template renderer.
  - `packages/bee/prompts/worker-cell.md`: Prompt template for cell execution workers.

- **Current Template Variables in `worker-cell.md`**:
  - `{{worker}}`, `{{cell_id}}`, `{{feature}}`, `{{worktree_root}}`, `{{control_root}}`, `{{cell_json}}`, `{{learned_context}}`, `{{expertise}}`, `{{prior_rounds}}`.

- **Fit**:
  - `dispatch prepare` is the **single mandatory chokepoint** for all subagent and worker dispatches across Claude and Codex runtimes.
  - `prompt_body_for` in `prepare.rs` has direct access to the repository root and feature slug.

- **Gaps & Deficiencies**:
  - `prompt_body_for` does not currently read `.bee/intent/<feature>.json`.
  - `worker-cell.md` does not have an `{{original_request}}` block.

---

### Candidate B4: Worker Task Text Conventions in `skills/bee-swarming`

- **Code & Doctrine Anchors**:
  - `skills/bee-swarming/SKILL.md:21-71`: Orchestration lifecycle.
  - `skills/bee-swarming/references/swarming-reference.md:94-103`: Worker isolation contract.

- **Current Convention**:
  - Mandates strict isolation: the prompt must contain only the cell ID, paths to `CONTEXT.md` and `plan.md`, global constraints, and the status-token protocol.
  - Explicitly states: "nothing else, never session history".

- **Fit / Gaps**:
  - Swarming doctrine enforces that orchestrators never hand-craft or paste conversational history.
  - Therefore, verbatim request pass-down must be handled mechanically by `dispatch prepare`, not by manual orchestrator copy-paste.

---

## 4. The Two Cheapest Workable Shapes

### Cheapest Workable Shape for Question A: Contract Status (`CHỐT` / `CHƯA-CHỐT`)

**Recommendation: A hybrid of `CONTEXT.md` (for human shaping) and `bee decisions` tags (for machine queries).**

1. **Authoring Surface (`CONTEXT.md`)**:
   - In `skills/bee-shaping/references/context-template.md`, add a standardized `## Contracts` table:
     ```markdown
     ## Contracts

     | ID | Contract / Interface | Status | Reference / Path | Notes |
     |---|---|---|---|---|
     | C1 | `POST /api/v1/auth/login` | CHỐT | `docs/specs/auth.md#B2` | Locked in D2 |
     | C2 | `PaymentGatewayAdapter` | CHƯA-CHỐT | `src/payment/adapter.rs` | Awaiting provider docs |
     ```
   - Decision IDs for contracts use standard format (e.g. `D-CON-1`).

2. **Machine-Readable Storage & Query (`bee decisions`)**:
   - Log contract decisions with tag `contract:<name>`:
     ```bash
     .bee/bin/bee decisions log --scope <feature> --tag "contract:<name>" --relation none --decision "Contract <name> locked: ..."
     ```
   - An unsettled contract with an open question registers a trigger:
     ```bash
     .bee/bin/bee triggers add --decision <id> --condition "Contract <name> pending review"
     ```
   - **Status Determination**:
     - `CHỐT` $\iff$ Active decision with tag `contract:<name>` exists AND no pending trigger attached.
     - `CHƯA-CHỐT` $\iff$ Trigger pending OR decision tagged `status:unsettled` OR contract listed with `CHƯA-CHỐT` in in-scope `CONTEXT.md`.

3. **Cell Citation**:
   - Cells declare referenced contracts in `read_first` or `decisions` (e.g. `decisions: ["C1"]`).
   - A worker checking `CONTEXT.md` or running `bee decisions search --tag "contract:..."` sees `CHƯA-CHỐT` and immediately halts/refuses with `[BLOCKED: contract unsettled]` instead of minting mock interfaces.

**Why this is the cheapest workable shape**:
- Zero database schema migrations.
- Zero breaking changes to `cell` validators or `.bee/cells/*.json`.
- 100% compatible with existing `bee decisions` and `CONTEXT.md` workflows today.

---

### Cheapest Workable Shape for Question B: Verbatim `original_request` Transmission

**Recommendation: Wire existing `bee intent` store directly into `dispatch prepare` and `worker-cell.md`.**

1. **Storage (Unchanged — 100% Existing)**:
   - Use the existing `bee intent set` command:
     ```bash
     .bee/bin/bee intent set --feature <slug> --request "<verbatim ask>" --acceptance "<criteria>"
     ```
   - Stored on disk at `.bee/intent/<slug>.json`.

2. **Dispatch Assembly (`packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`)**:
   - In `prompt_body_for`:
     ```rust
     if let Some(feature) = feature_slug {
         if let Ok(Some(anchor)) = read_anchor_at(root, &feature) {
             if let Some(req) = anchor.get("request").and_then(Value::as_str) {
                 vars.insert("original_request".into(), req.to_string());
             }
         }
     }
     ```

3. **Prompt Template (`packages/bee/prompts/worker-cell.md`)**:
   - Add an optional `original_request` section:
     ```markdown
     {{#if original_request}}
     Original Request (verbatim — immutable objective):
     > {{original_request}}
     {{/if}}
     ```

**Why this is the cheapest workable shape**:
- Reuses the existing `bee intent` subsystem without creating a redundant storage mechanism.
- Avoids bloating cell JSON files.
- Modifies only ~5 lines of Rust code in `prepare.rs` and ~4 lines in `worker-cell.md`.
- Works identically across Claude Code, Codex, and tmux-herding runtimes.

---

## 5. Candidate Comparison Matrix

| Candidate | Storage Home | Verbatim / Machine-Readable | Tool / CLI Support | Implementation Cost | Risk of Drift |
|---|---|---|---|---|---|
| **A1. `CONTEXT.md` Decisions** | `docs/history/<feature>/CONTEXT.md` | Partial (Markdown prose) | Read-only in agent prompts | Low (doc convention only) | Low within feature; high across features |
| **A2. `docs/knowledge` OKF** | `docs/knowledge/areas/` | High (YAML frontmatter) | `bee knowledge check/search` | High (ceremonial schema) | Low (machine-checked) |
| **A3. `bee decisions` + Views** | `.bee/decisions.jsonl` | High (JSON / tags / relations) | `bee decisions active/search` | Very Low (uses tags) | Very Low (append-only ledger) |
| **A4. Cell `must_haves`** | `.bee/cells/*.json` | High (JSON object) | `bee cells add/validate` | Medium (cell schema changes) | High (ephemeral lifecycle) |
| **B1. Cell Schema Fields** | `.bee/cells/*.json` | High (JSON string) | Requires Rust schema edit | Medium (cell schema churn) | High (duplicated per cell) |
| **B2. `bee intent` (Anchor)** | `.bee/intent/<slug>.json` | High (Verbatim immutable) | `bee intent set/show` | Zero (already built) | Zero (enforces immutability) |
| **B3. `dispatch prepare` Template** | `packages/bee/prompts/` | High (Prompt rendering) | `bee dispatch prepare` | Very Low (~10 lines total) | Zero (centralized dispatch door) |
| **B4. Swarming Task Text** | Agent instruction text | Low (Ad-hoc prompt prose) | Manual prompt authoring | Low | High (orchestrator drift) |

---

## 6. Conclusion and Next Steps

For SLP Ticket 007, `bee` does not require heavy new subsystems:
1. **Contract Status**: Adopt a standardized `## Contracts` table in `CONTEXT.md` during shaping, backed by `bee decisions log --tag contract:<name>` for machine-checked queries.
2. **Original Request**: Wire the already-existing `.bee/intent/<slug>.json` verbatim request into `dispatch prepare` so every dispatched worker receives the immutable user goal at the top of its prompt.

Deliverable file path: `docs/history/research/slp-contract-request-surfaces.md`
