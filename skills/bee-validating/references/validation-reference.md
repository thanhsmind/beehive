# Validation Reference

Load after bee-validating is selected and the required inputs exist. Formats here are normative — reports must use them.

## Protocol

1. Orient: `node .bee/bin/bee.mjs status --json`, mode/lane, approved `plan.md`, current cells.
2. Reality gate report (below), evidence attached.
3. Feasibility matrix for every blocking assumption.
4. Spike/probe any unproven assumption that can invalidate the current work.
5. Plan-checker subagent, max 3 iterations.
6. Cell review (cold pickup); fix CRITICAL flags.
7. Decision, then the Gate 3 approval block.

## Required Inputs in full

- `docs/history/<feature>/CONTEXT.md`.
- `docs/history/<feature>/plan.md` — approved and frozen at Gate 2: its content sections are immutable once `approved_gates.shape` is set, so what validating reads is byte-identical to what the human approved.
- The discovery and approach content: `discovery.md` and `approach.md` if they exist; otherwise the `## Discovery` and `## Approach` sections folded into `plan.md` (separate files are written only for L2+ discovery or high-risk lanes).
- Current-slice cells exist: `node .bee/bin/bee.mjs cells list --feature <feature>` (the current slice lives only in cells; there is no separate slice document).

If `plan.md` is absent, unapproved, or the current-slice cells do not exist, stop and return to bee-planning. Never validate an unapproved shape. A missing `discovery.md`/`approach.md` is **not** a failure when `plan.md` carries the equivalent sections — read those; stop only if neither exists.

## Reality Gate Report

```text
REALITY GATE REPORT
Mode: <tiny|spike|small|standard|high-risk>
Current work: <one sentence>
MODE FIT: PASS|FAIL       — lane matches the mechanical risk flags; least honest workflow
REPO FIT: PASS|FAIL       — named files/APIs/commands exist in this repo today
ASSUMPTIONS: PASS|FAIL    — every blocking assumption is listed in the matrix
SMALLER PATH: PASS|FAIL   — no smaller path delivers the locked decisions
PROOF SURFACE: PASS|FAIL  — every cell's verify command runs in this repo
Decision: proceed | revise planning | run spike first | collapse mode
Evidence: <file paths / command output / runtime evidence per line above>
```

Fail on: nonexistent code paths, unsupported commands, stale versions, missing credentials, unreachable services, hidden architecture work, or excess ceremony.

## Feasibility Matrix

Required whenever blocking assumptions remain; always for the high-risk lane.

```text
FEASIBILITY MATRIX
Assumption | Risk | Proof Required | Evidence | Result | Sources
```

`Sources` is what the row was proven from, and it is what makes the row cacheable: `path@sha256[:12]` for file evidence, `command@output_sha[:12]` for command evidence. Mirror the same rows into the machine cache with `bee.mjs state validation-cache record --slice <n> --rows-file <f>`; on the next slice, `bee.mjs state validation-cache check --json` reports which rows are still hash-fresh. A carried-forward row keeps its original Evidence text and reads `Result: cached (slice N, sources unchanged)`. Rows with no Sources are never cacheable — they re-prove every slice.

Accepted evidence: existing implementation, file/API/type inspection, command output, build/typecheck/test result, official version/doc proof, runtime/API probe, or `.bee/spikes/<feature>/` proof. "Should work", "likely", "expected", or model knowledge → the row (and the matrix) is **NOT READY**. A cached row is Accepted Evidence — it was proven once and its sources are hash-verified unchanged — but it is held to the identical bar: plausibility language in a cached row auto-fails exactly as it does in a fresh one.

## Spike / Probe Rules

- One spike = one yes/no question.
- Disposable proof lives under `.bee/spikes/<feature>/`.
- NO → return to bee-planning with the failed assumption and the plan change it forces.
- YES → record constraints for planning and execution.
- Spike code must never silently become production implementation.
- **Debug discipline (hypothesis before repro, read before rerun):** before writing any repro script, record the hypothesis in one line plus the file:line evidence from reading the code that grounds it — a repro script with no prior hypothesis is not a spike, it's a guess with extra steps. Cap the loop at **2** failed repro rounds: after the second wrong repro, stop running scripts and go back to reading/instrumenting the actual code path instead of a third guess blind. Prose law; the machine-enforced proof-tier matrix lives in `bee-executing/SKILL.md`.
- **Verify scripts and any executable code never go in `docs/history/`** (`docs/history/` is the tech-agnostic knowledge layer — `.md` only). A cell's `verify` is a runnable command; a multi-line harness lives in the project's own scripts (committed with the product, so `verify` points at it) or, if disposable, in `.bee/spikes/<feature>/`. The write-guard denies a code-extension file (`.sh`, `.mjs`, `.py`, …) written under `docs/history/`, and any scratch-shaped write landing in a tracked directory outside `.bee/tmp/`/`.bee/spikes/`.

## Repair Routing

| Finding | Route |
|---|---|
| False assumption / wrong mode or lane | back to bee-planning |
| Locked decision uncovered by any cell | `plan.md` + new/edited cells (cite the D-ID) |
| Cell dependency, file-scope, or test gap | edit the cell (`node .bee/bin/bee.mjs cells show --id <id>` first) |
| Broken or unrunnable verify command | fix the cell's `verify`; re-run PROOF SURFACE |
| Unreachable exit / integration hole | `plan.md` (key links) then cells |
| Scope reduction of a locked decision | prohibited — SPLIT the work instead, via planning |

## Review Wave in full

**A wave, not a chain.** At stage start dispatch **simultaneously** the merged reviewer below and — when the D2 rubric fires — the orient/extraction worker, then run the reality gate and feasibility matrix on the session model **while the wave runs**: the stage costs max(reviewer, matrix), not their sum. **Sync point (wave-wide):** findings block nothing until the Gate 3 presentation — or its bypass self-approval — and neither ever happens while **any** wave member is outstanding.

**One dispatch, two mandates, both vocabularies.** One `bee-review`-class dispatch on the **`review` slot** (default opus on Claude, generation fallback; state the model explicitly; if the runtime cannot select per-agent models, cap its reads and output instead) replaces the former plan-checker + cell-reviewer pair and returns **one report, two sections**: **Structure** — the adversarial check over its 5 dimensions, every finding **BLOCKER** or **WARNING**; and **Cells** — the cold-pickup review, every finding **CRITICAL** (all fixed before approval) or **MINOR** (may ship with a recorded note). Merging the dispatches never merges the finding classes. Prompt, dimensions, and both flag lists: below.

<!-- bee:only claude -->
On Claude Code, spawn `subagent_type: "bee-review"` when `.claude/agents/bee-review.md` exists — bee's own rendered agent for the review tier, never `general-purpose` (a model-guard denies that pairing).
<!-- bee:end -->
<!-- bee:only codex -->
Codex has no per-agent subagent type, so the tier stays enforced as a read budget + output cap only.
<!-- bee:end -->

It is a **read-only gather**, never a cell: a cli-shaped review slot resolves with the purpose-scoped `resolveTier(root, 'review', runtime, {for:'gather'})` — a bare 3-arg resolve of one now refuses; a model-shaped slot is unaffected by purpose.

**One shot, then at most one blocker pass.** The merged reviewer runs **once**. WARNING-level and mechanically fixable findings (a missing link, a vague verify command, a dependency typo) the orchestrator applies **directly to the cells** — legal because cells are mutable before Gate 3. Only **unresolved BLOCKERs** trigger a **second and final** pass, scoped to those blockers. No third pass: a BLOCKER open after pass 2 escalates to the user with both positions.

**Small-diff standard: same mandates, no dispatch.** When the counted touch set is ≤5 product files with zero hard-gate flags, the merged reviewer is not dispatched — the session model runs both mandates itself: Structure over the same 5 dimensions, Cells as a cold-pickup pass, findings in the same vocabularies, recorded in the validation report. Same sync point, same one-shot-then-one-blocker-pass cap. A hard-gate flag, a 6th product file, or genuine doubt about self-review independence restores the dispatch. `high-risk` never takes this path.

## Merged Reviewer Subagent Prompt

One dispatch on the **review** slot replaces the former plan-checker and cell-reviewer
pair (spec #77 P3). Two mandates, two finding vocabularies, one report. Verify, do not
redesign. On slice 2+ the scope is the new/changed cells and stale rows only — the plan
is frozen and was checked on slice 1.

```text
You are a merged plan reviewer. Two mandates. Assume the work is flawed until proven so.
Inputs: docs/history/<feature>/CONTEXT.md, approach.md, plan.md, and the current-work
cells (node .bee/bin/bee.mjs cells list --feature <feature>).

MANDATE 1 — STRUCTURE. Verify exactly 5 dimensions:
1. Requirement/decision coverage — every locked D-ID lands in at least one cell.
2. Cell completeness — each cell has files, read_first, directive action, must_haves
   (per lane tier), and a runnable verify.
3. Dependency correctness — deps form a DAG; no cell depends on a future slice.
4. Key links — integration points named in plan.md are owned by a specific cell.
5. Scope sanity — no cell is doing hidden architecture work or exceeds its lane.
Report every structural finding as BLOCKER (structurally unsound) or WARNING
(survivable, note it).

Small-diff `standard` (≤5 product files, zero hard-gate flags) runs these same
dimensions as an inline self-review on the session model — no dispatch (SKILL.md,
"Review Wave", lane-lean D3). Both finding vocabularies and the one-blocker-pass cap
apply unchanged.

MANDATE 2 — CELLS, COLD PICKUP. You have NO session history. For each cell, answer:
could a worker who has read only CONTEXT.md, plan.md, and this cell implement and
verify it without guessing?
Flag CRITICAL: assumed context, vague acceptance, scope overload, unproven feasibility,
broken verify command.
Flag MINOR: missing rationale, implicit file assumption, fuzzy boundary, known tradeoff
not recorded.

Return ONE report with both sections below. Never merge the two vocabularies: structure
findings are BLOCKER/WARNING, cell findings are CRITICAL/MINOR.
Do not propose redesigns. Do not soften findings. Quote file/cell evidence per finding.
```

```text
REVIEW REPORT
Work: <current slice / direct task>

STRUCTURE
BLOCKERS: <dimension> problem / evidence / fix
WARNINGS: <dimension> problem / evidence / note

CELLS  (reviewed: <N>)
CRITICAL FLAGS: <cell-id> problem / evidence / fix
MINOR FLAGS: <cell-id> problem / evidence / suggestion
CLEAN CELLS: <cell-id>, <cell-id>

SUMMARY: <2-3 sentences>
```

**One shot, then at most one blocker pass.** WARNING-level and mechanically fixable
findings — a missing link, a vague verify command, a dependency typo — the orchestrator
applies directly to the cells, which is legal because cells are mutable before Gate 3.
Only unresolved BLOCKERs earn a second and final pass, scoped to those blockers. There
is no third pass: a BLOCKER still open after pass 2 escalates to the user with both
positions. All CRITICAL cell flags are fixed before Gate 3; MINOR flags ship with a
recorded note.

### High-Risk Persona Panel

For the high-risk lane, scale this same merged dispatch to a small panel: **coherence**
and **feasibility** personas always; add conditional lenses — **security**, **product**,
**scope-guardian** — chosen by the diff of concerns (auth/data → security; user-visible
behavior → product; growing surface → scope-guardian). Each persona gets the same inputs
and both vocabularies. Dedupe overlapping findings, then synthesize into two buckets:
**auto-fix** (apply, record) and **present-for-decision** (user judgment required).

## Advisor Consult in full

The orchestrator resolves the advisor from config (`resolveAdvisor(root, runtime)`):

- **cli-shaped** advisor → run the configured command verbatim, read-only, with an evidence bundle on stdin (plan summary, risk map, validation findings, open questions — never session history, never secrets) and capture the digest.
- **model-shaped** advisor → dispatch a `bee-review`-class read-only run with the same evidence bundle.
- **unconfigured** advisor (`resolveAdvisor` returns `null`) → record that fact and proceed. This adds one trigger; it is not a hard dependency on an advisor being configured.

Then record the consult: `node .bee/bin/bee.mjs state advisor-ref record --advisor "<identity>" --digest-file <path>` (the verb stamps the staleness anchors itself — the caller supplies only the advisor identity and the digest file). Nothing is written to the gate until a non-stale `advisor_ref` exists; this is CLI-enforced, not optional ceremony.

## Approval Gate Block

Two layers (Gate Presentation Contract, bee-hive routing reference). The machine block goes into the **report file** `docs/history/<feature>/reports/validation-<slice>.md`, together with the reality gate report, feasibility matrix, plan-checker findings, and cell review above. It is never pasted into chat:

```text
VALIDATION COMPLETE - APPROVAL REQUIRED BEFORE EXECUTION
Mode: <mode>
Work: <current slice / direct task / spike>
Reality gate: PASS
Feasibility: READY | READY WITH CONSTRAINTS
Structure: PASS after <N> iterations
Spikes: <none | passed | constraints recorded>
Cell review: PASS (<N> cells, 0 CRITICAL open)
Unresolved concerns: <none | list>
```

The **chat message** is the human layer only — in the user's language, jargon-free:

```text
What I'm about to do: [the change in the user's terms, one sentence — what changes for them, not the mechanism].
Why it's trustworthy: [the single strongest piece of evidence, plain words — e.g. "a dry run rebuilt all 3 pages byte-for-byte identical"].
If it goes wrong: [what breaks for the user + how we'd notice — loud failure, rollback path].
You are deciding: whether I may start editing real files — this slice of work only.
Full validation report: docs/history/<feature>/reports/validation-<slice>.md
Feasibility validated. Approve execution?
```

Litmus: the user can restate what they are approving in their own words. Optionally offer a cross-model second opinion first: agreement → mention it; disagreement → quote both positions; never auto-resolve.

**The `bee-briefing` refresh** (patching the implement plan's Validation Plan section with the evidence links) is presentation, not evidence: when the bypass level covers Gate 3, skip it here and run it lazily at the next human-facing event instead — a real gate presentation, an explicit `bee-briefing` invocation, or feature close-out. With bypass off, or when this session is the one stopping at the gate, refresh now as before, so the Gate 3 message links a current brief.

**On approval**, `validated` is never written as a phase — it never was one: `state gate --name execution --approved true` records the approval, then `state set --owner validating --phase swarming ...` moves the phase straight to `swarming`. An agent that hits a refusal and invents an intermediate phase value is exactly the failure this note exists to prevent.

Approval is for the current work only. On yes: update `.bee/state.json` (`approved_gates.execution: true`) and hand off to bee-swarming. In headless mode, stop here — emit both layers in the terminal report and exit without approval.

## Red Flags

- skipping reality or feasibility gates because everything "looks right"
- plausibility accepted as proof under time pressure
- continuing after a NO spike
- iteration 4 of the plan checker
- cells not tied to the current work slice
- a small fix generating epic ceremony; a hard-gate change validated as small
- Gate 3 asked with CRITICAL cell flags still open
- the machine block pasted into chat, or a gate message the user cannot restate in their own words
