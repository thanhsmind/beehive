# Semantic token diet — Context

**Feature slug:** semantic-token-diet
**Date:** 2026-11-16
**Scope:** Standard
**Domain types:** ORGANIZE (instruction text layout), RUN (preamble, brief and help renderers)

## What was asked

The owner asked for a review of each branch of the bee flow, to find where the
prompts, agent files and skills can use fewer tokens and stay as effective.
The cut must follow meaning, not a mechanical rule. After the audit, the owner
said "Làm" (do it).

## What was found

Eight read-only audits, one per branch, are in `reports/`:

- `A-doctrine.md` — CLAUDE.md, AGENTS.md, LEADER.md, the preamble
- `B-router.md` — bee-hive
- `C-playbooks.md` — route playbooks and principles
- `D-planning.md` — bee-planning
- `E-shaping.md` — wayfinding, shaping, researching
- `F-execution.md` — bee-swarming, agent files, prompts
- `G-close-review.md` — capturing, verifying, reviewing, knowledge-upkeep
- `H-herding-help.md` — bee-herding, bee-herdr, CLI help

The main cost is wrong loading, not long sentences: a worker loads the whole
bee-swarming skill, bee-hive marks a 43 KB reference as the default read, a
tiny plan reads all of planning-reference.md, and the help spill hides flags.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | One-off semantic diet. Every move keeps the rule, its why-clause and test-pinned wording. Text moves to the file read on the branch that needs it; a copy of a fact with an always-loaded home is cut. No size gate, byte budget or line limit is added. | 2a2604e3 |
| D2 | Stale or contradicting text is corrected in its owning file: planning "no trailing test cell" and "writer owns tests" lines, the design-sketch gate name, the dead "Proof floor" pointer in playbooks, the legacy `--cleanup` flag, the stale bug-fix anchor, the knowledge-upkeep "change them here" line, the herding status-verb claim and openai.yaml role list, and the scout-and-ticks and routing-and-contracts pointers to sections that no longer exist. | 74fb4b50 |
| D3 | bee-swarming Execute and Tester contract move to `skills/bee-swarming/references/execute.md`. `worker-cell.md` and the worker agent bodies point there, not at the whole skill. `worker-cell.md` is the one home of the Result form; the swarming-reference copies become pointers. Agent bodies drop rules the rendered prompt carries. contract:worker-prompt-execute-pointer. | 6d9d5f99 |
| D4 | The bee-hive References row for `routing-and-contracts.md` names real triggers, not "The default". Body sections that restate LEADER.md (Hard rules, gate and bypass prose, Headless) shrink to pointers. Copies of LEADER.md text inside routing-and-contracts.md and gates-and-delegation.md go. | 66dfeff7 |
| D5 | `planning-reference.md` splits into `references/cells.md` (every lane) and `references/plan-artifact.md` (standard and high-risk). Every inbound pointer is repointed. Hat-wave consumption keeps only the planning-only parts. | 90df38fd |
| D6 | `packages/bee/LEADER.md` keeps each rule and its why; waggledance and CLI-before-MCP mechanics move to read-when playbooks in `packages/bee/playbooks/`; Trigger index and Deep contracts fold to one line each; bullets that restate the AGENTS block or the preamble merge. The project LEADER.md moves the waggledance completion-report template to a project doc behind a two-line trigger and keeps one STE home. | 35d402aa |
| D7 | contract:preamble-trailer-orient. The preamble trailer names `bee orient` for routing and `bee status --json` only for missing detail, and drops sentences LEADER.md holds. The Proof-per-change-type preamble line goes (agents-proof-at-cap holds it). | f21a5bf3 |
| D8 | contract:brief-proof-only-for-cells. A herding brief renders `# Proof command` only for a job with a cell id. | bf647234 |
| D9 | contract:help-flags-before-details. Single-command help prints the first sentence, then flags, then the rest under a details label. The heaviest descriptions drop decision ids and history narration; control-loop help names every role. | cf38764c |
| D10 | `skills/bee-herding/references/role-dispatch.md` splits at the interlock: the spawn half moves to `references/role-dispatch-spawn.md`, the duplicate quick-reference table goes, measurements move to `spawn-proof.md`, section numbers stay. `operational-invariants.md` points at `bee herding status --job`. | e5f3d375 |
| D11 | Out of this feature, filed as proposed PBIs: orient principle cap (p-806a3911), trace-and-provenance split (p-77bfcb18), shaping and wayfinding question policy (p-6c3e4223), review.md split (p-b97e839a), herding status --json paging (p-6b237338). | 1ba311b3 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md | suspect | Preamble budget, trailer, and R4: no budget on authored text |
| docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md | suspect | Rule index and the duplication boundary for LEADER.md moves |
| docs/knowledge/areas/doctrine-layer/placement-and-anchoring.md | suspect | Where a rule lives and how pointers anchor |
| docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md | suspect | The herding brief that D8 changes |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs` | `TRAILER_LINE` and the proof-per-change line (D7) |
| `packages/bee-rs/crates/bee/src/herding/mailbox.rs` | `render_brief` and the `# Proof command` block (D8) |
| `packages/bee-rs/crates/bee/src/verbs/help.rs` | `render_help_text`, single-entry help order (D9) |
| `packages/bee/prompts/worker-cell.md` | the worker prompt line that loads bee-swarming (D3) |
| `packages/bee/LEADER.md` | the leader doctrine (D6) |

## Out of scope

- Any size gate on instruction text (decision 8f63adb4).
- The five items of D11.
