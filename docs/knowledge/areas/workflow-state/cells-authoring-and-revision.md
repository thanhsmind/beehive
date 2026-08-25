---
type: bee.area
title: "Workflow State — authoring a unit of work, revising its plan, and the frozen plan document"
description: "How a slice of work units is created all-or-nothing, which of a unit's plan fields may be revised afterwards and which are frozen audit, how a unit's change is classified at authoring, how a scope-derived regeneration obligation refuses authoring without it, why the approved plan document stops changing the moment its gate is granted, and why a feature still deciding its shape cannot have units authored into it at all."
timestamp: 2026-08-06
bee:
  id: workflow-state-cells-authoring-and-revision
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: ["lane-ceremony-v3 D1/D2/D9 (docs/history/lane-ceremony-v3/CONTEXT.md, 2026-07-19 — plan document frozen at shape approval, slice-in-units)", self-correcting-loop D3 with Validating amendment Δ4 (change classification and the advisory verification standard), "regen-obligation-derived D1/D2 (derived regen obligation refuses at authoring, recorded escape hatch; roots derived from the tools, never hard-coded — 2026-07-23)", "8ef2bae6 (cli-ergonomics D2 — whole-batch exhaustive refusal + --dry-run preview, 2026-07-24)", "worker-conformance D4/D5/D6 (the trailing test unit stays unconditional but its first mandated step is a coverage judgement, not authoring; test shape below the highest-risk lane is the happy-path/edge-cases/error-paths triad and the twelve-dimension checklist applies only to high-risk/hard-gate work; no numeric per-group test cap is added — 2026-07-29)", "worker-conformance D13 (the doctrine-vs-machine disagreement over batching the trailing test unit at high risk is recorded as an open gap, not fixed — the close-door predicate was deliberately left unchanged)", "hook-teeth D3/D7 (docs/history/hook-teeth/CONTEXT.md, 2026-08-04 — no units before the gate: authoring into an ungated feature refuses the whole batch, documentation-lane work exempt)", "counter-teeth D3/D6 (docs/history/counter-teeth/CONTEXT.md, 2026-08-04 — the ceiling tier's 40% share becomes a refusal with a recorded-reason escape; decision 0012's threshold value unchanged)"]
  sources: [cells-update-verb cell cuv-1 (2026-07-12), dispatcher-unify cells-batch-add suite rows (v0.1.27), "post-advisor-hardening cell pah-2 (cells add/update manifest-lint advisory, 2026-07-18)", "lane-ceremony-v3 cells lcv3-1..lcv3-5 (traces in .bee/cells/, reports docs/history/lane-ceremony-v3/reports/, 2026-07-19)", "worker-conformance cells wc-3/wc-6/wc-7 (coverage-judgement-first trailing test unit, triad test shape, ten doctrine overstatements corrected against the live predicates; traces .bee/cells/wc-{3,6,7}.json, reports docs/history/worker-conformance/reports/, CONTEXT docs/history/worker-conformance/CONTEXT.md, 2026-07-29)", "regen-obligation-derived cell ro-1 (12 suite rows + mutation red, commit e4ae329, 2026-07-23)", "docs/specs/workflow-state.md#B7", "docs/specs/workflow-state.md#B10", "docs/specs/workflow-state.md#B25", "docs/specs/workflow-state.md#B29", "docs/specs/workflow-state.md#R46", "docs/specs/workflow-state.md#E14", "docs/specs/workflow-state.md#P9", "docs/specs/workflow-state.md#P12", "hook-teeth cell bh-3 (gated authoring refuses the whole batch, lane record beats the default, docs lane exempt; trace .bee/cells/bh-3.json, 2026-08-04 — cells slice 83 passed)", "counter-teeth cell ct-4 (ceiling-share refusal with the --reason override persisted as trace.tier_reason; trace .bee/cells/ct-4.json, commit a5e564fa, 2026-08-04 — cells 71 passed, 0 failed)", "addcell-all-problems cell cap-1 (single-cell path joins all problems via the shared collector, 2026-08-17)", "cell-preflight cell cpf-1 (pre-flight checklist + mandatory dry-run in the planning flow, commit ea64d5b4, 2026-08-17)"]
  authoritative_for: "workflow-state: unit-of-work authoring, plan revision, and the frozen plan document"
---

# Workflow State — authoring a unit of work, revising its plan, and the frozen plan document

A unit of work is written down before it is done, and what is written down has
two halves with opposite rules: the PLAN, which may be revised while the unit is
still open, and the RECORD of what happened, which may never be. This concept
owns the authoring door itself — the all-or-nothing batch, the revision guard,
the authoring-time classification — plus the one document that stops being
revisable at all once its gate is granted.

## Data Dictionary

### Cell Fields

| Field | Meaning |
|---|---|
| `id` | Unique string identifying the cell (e.g. `auth-3`). |
| `feature` | The feature slug the cell belongs to. |
| `title` | One-line summary of what the cell delivers. |
| `lane` | Lane classification (`tiny`, `small`, `standard`, `high-risk`, `spike`). |
| `role` | The job this work is; the sole model selector. Required on `cells add`, exactly as `lane` is (model-role-split D7, store `4eaf1b71`). Any non-empty name is legal — validation checks presence and shape, never membership. `code`, `read`, `test`, `docs`, `review`, `design` are the recommended vocabulary (D8), authoring guidance only, never an enforced list. A role nothing in `models.<runtime>` configures still runs: the dispatch falls through to the next name it asked for and warns, it never refuses. |
| `status` | State of the cell (`open`, `claimed`, `capped`, `blocked`, `dropped`). |
| `deps` | Array of cell ids whose completion this cell depends on. |
| `decisions` | Array of decision ids or tags cited by this cell. |
| `files` | Flat array of repo-relative paths the cell is permitted to touch. |
| `read_first` | Flat array of repo-relative paths the worker must read before editing. |
| `affects_skills` | Flat array of repo-relative skill paths predicted to be affected by this cell (`[]` if none). Required on `cells add` across all lanes (D3). |
| `affects_specs` | Flat array of repo-relative spec/concept paths predicted to be affected by this cell (`[]` if none). Required on `cells add` across all lanes (D3). |
| `action` | Directive prose explaining what the assigned worker must do. |
| `verify` | Command string to verify the cell's changes. |
| `must_haves` | Object containing observable truths, substantive artifacts, key links, and prohibitions. |
| `trace` | Execution audit record (worker, outcome, deviations, friction, etc.). |

## Behaviors & Operations

**One-verb regen chain (workflow-lessons wfl-2, 2026-08-11).** `bee dev regen`
runs the three-step regen chain (render-skill-trees → onboard --repo-root .
--apply → release-manifest --write) in fixed order through the in-process
entry points, stopping at the first red with that step named; the
REGEN_OBLIGATION refusal text routes to the verb (three steps kept in
parentheses for cold readers). The remembered-sequence failure mode — source
shipped without the chain — is closed by construction. The chain writes
byproducts beyond a cell's named scope by design (cell-preflight cpf-1,
2026-08-17): the skill-tree render syncs every rendered target
(`.agents/skills/*`, `.claude/skills/*`, `.opencode/skills/*`) and the
onboarding apply touches its ledger record, so a unit whose scope names the
skill sources and the manifest record need not enumerate those mirrors —
they ride the mandated regen, not the unit's own edit set.

**B7 — Cell plans are revisable in place, execution records never.** A unit of
work's PLAN fields (title, action, scope files, reading list, dependencies,
cited decisions, acceptance contracts, verify command, lane, behavior flag,
affected skills, affected specs)
can be revised after creation through one guarded operation — the normal path
when a pre-execution review prescribes a fix. The door: only open or blocked
units accept revision (claimed = a live worker owns it; capped/dropped = the
frozen audit record); identity (id, feature), status, the execution trace, and
the model tier are refused by name with a hint at the owning operation; an
unknown field refuses the whole patch (the updatable list is derived from the
validator map, so a forgotten field is a refusal, not a leak); a
present-but-corrupt record refuses loudly with the file untouched; a revision
that would leave a standard/high-risk unit without acceptance truths is
refused. Observers see either the old plan or the fully revised plan — never a
partial merge.

**B10 — A whole slice of work units is created in one all-or-nothing call.**
Creating the current slice's units accepts the full batch in a single request;
every unit is validated (including duplicate identifiers within the batch)
before any is written, so one invalid unit means zero units created. A
single-unit request still works the same way. The refusal is exhaustive
(cli-ergonomics D2, 8ef2bae6): every failing unit is named with every one of
its problems — regeneration obligations included — in the one combined
refusal, so a large batch never needs re-sending to discover the next error.
Since addcell-all-problems (cell cap-1, 2026-08-17) the single-unit spelling
gives the same guarantee: one refusal joins every problem that one unit has,
through the same problem collector the batch rows read, so no path discovers
errors one at a time.
The authoring flow itself pre-flights before it drafts (cell-preflight
cpf-1, 2026-08-17, from user feedback after three avoidable rejections):
the planner walks the constraint checklist — id pattern and the
`<feature-slug>-<n>` convention, required fields, lane values, the
scope-derived obligations — then pipes the drafted batch through the
preview mode below and submits the real create only after a clean pass,
so first-submission rejections are the exception, not the loop.
The same door also offers a preview mode (`--dry-run`): the identical
validation pass over the whole batch, reporting per-unit verdicts, persisting
nothing in any outcome, and succeeding only when the batch is clean. After a successful create or
amend, the verb also lints each written unit for one known authoring trap: a
verification command that checks the release manifest while the unit's file
list omits the manifest itself (a cold implementer would end red with no
sanctioned fix). The lint is a loud advisory line naming the trap and the fix —
it never refuses the write, never changes the outcome, and tolerates malformed
shapes silently (cells pah-2, 2026-07-18). This advisory lint keeps exactly its
original coverage; the scope-DERIVED regeneration obligation is a separate,
refusing check (B37).

**B25 — The approved plan document is frozen; the current slice lives only in
work units.** Trigger: shape approval is granted for a feature whose lane keeps
a plan document (lane-ceremony-v3 D1/D2/D9). What happens: from that moment the
plan document's content is immutable — the only permitted post-approval write
is an approval stamp (status and time). Preparing execution never rewrites the
plan into a different readiness state; the current slice is represented solely
by the feature's open work units (each carrying its own touched paths,
acceptance contract, and verification command), and "next slice" means the next
batch of units created by planning — no slice document exists anywhere. What
consumers observe: the artifact the approver reviewed stays byte-identical
through execution; the plan fingerprint anchoring the adviser consult record
can change only before approval (or through a human superseding a decision);
implementation-plan projections drift — and demand re-rendering — only when
work units change, never because the plan moved after its gate. Lanes without a
plan document (tiny always; small by default) are untouched by this rule: their
work shape is the unit itself (doctrine-layer R13).

**B29 — Authoring a unit of work classifies its change, and an insufficient
verification plan is a warning, never a block.** Trigger: creating or revising
a unit of work. What happens: the unit's change classification is set
explicitly or derived only from the behavior-change flag; the recorded
verification plan is checked against that classification's minimum standard
and, if it falls short, a warning names the missing minimum on the same
advisory channel authoring warnings already use — it never fails the write and
never appears in the machine-parseable result. What each actor observes:
authoring behavior is otherwise unchanged; an author who ignores the warning
is informed, not stopped (self-correcting-loop D3, Δ4).

**B37 — A unit whose own scope implies a regeneration obligation cannot be
authored without carrying it.** Trigger: creating or revising a unit of work.
What happens: the unit's scope paths are checked against the roots the release
manifest tool itself declares it hashes — the roots are DERIVED from the tools
at check time, never copied into the guard, so a root added later is enforced
with no guard change. When any scope path falls under a hashed root, the unit's
verification command must include the release-manifest check and its scope must
include the manifest record itself; when a path additionally falls under a root
the runtime-mirror ledger covers, the verification command must also include
the ledger-parity check. A unit missing any of these is refused — in a batch,
zero units are written; in a revision, the unit is left untouched — with a
typed error naming the offending path, the root it hit, and the exact command
to add. The refusal is satisfiable by a deliberate recorded acknowledgement: a
named field on the unit carrying a non-empty written reason (a bare yes is
refused — the hatch must carry a reason); the refusal message itself names the
field, so skipping is always an act with a name in the unit's record, never an
oversight. A declared tool that is present but yields no derivable roots makes
the guard refuse loudly rather than pass blind; a host repo without the tools
owes nothing. What each actor observes: authors cannot forget the obligation,
only decline it on the record; the advisory lint (B10) is unchanged for its
narrower case (regen-obligation-derived D1/D2, cell ro-1, 2026-07-23).

**B44 — The trailing test unit stays unconditional, but its first mandated
step is a coverage JUDGEMENT, not authoring.** Trigger: planning shapes a
slice that touches code. What happens: the slice still gets a trailing test
unit — a code-touching slice with no test unit remains a planning defect, and
that floor is unchanged. What the unit is required to do FIRST changed: cite
the nearest existing tests by exact location, state whether they already cover
the slice's acceptance criteria, and author only the gap that remains.
"Already covered — no new rows" is a legitimate completed outcome; the unit
discharges by running those existing tests green and recording that judgement.
A test unit that authors no test is explicitly **not** a defect. What each
actor observes: the required THOUGHT is "do we need more coverage here?"
instead of the required OUTPUT being "write tests" — which is what turned a
volume brake into a volume generator. The feature-level coverage door is
unchanged by this and still demands the unit complete on recorded proof
(worker-conformance D4; the door itself is
areas/workflow-state/cells-completion-judge-and-archive.md B43).

**B46 — No units may be authored for a feature that is still deciding its shape
(hook-teeth D3, 2026-08-04).** Trigger: creating a slice of units for a feature
whose workflow still sits in one of the two phases that precede approval — still
exploring, or still planning — and whose execution approval has not been
granted. What happens: the whole batch is refused, exactly as a duplicate
identifier or a dependency cycle refuses it (B10), and the refusal names the
merged shape-and-execution approval as the way forward; nothing is written.
Which record decides: the feature's own lane record when it exists, otherwise
the default record, and only when that record names the same feature — a feature
neither record knows is never refused, so the first authoring in a fresh
repository stays open. What each actor observes: units cannot be smuggled in
ahead of the gate, while documentation-lane work is exempt outright, because
that lane never gates on execution in the first place.

**B52 — A unit whose scope touches guard source at a lane the close-time
judge door does not cover cannot be authored without acknowledging it
(pattern-20260812, cell jo-1).** Trigger: creating a unit of work whose
declared files include a path under a judge-required root — machine-guard
source: the hooks module tree, and any directory whose path carries a
`guard` segment. What happens: a unit at the standard or high-risk lane is
unaffected — the close-time judge-debt door already demands an independent
read there; a unit at any other lane (tiny, small, spike, docs) touching a
judge-required root is refused unless it carries a one-line recorded
acknowledgement (`judge_obligation_ack`), the same escape shape B37 already
gives the regen obligation. The reason this door exists at all: a guard and
tests written beside it by the same author are one model, so a green suite
only proves the model agrees with itself — three consecutive fixes to two
guards shipped green and wrong before an independent read caught each one
(pattern-20260812). The judge-required roots are pinned against the crate
source tree itself, in both directions, by tests living beside the check —
every declared root must exist on disk, and every guard-segment directory
under the crate's source must fall under a declared root — so a new guard
module shipped outside the list turns a test red instead of silently
escaping the door. What each actor observes: a unit that never touches
guard source is unaffected; one that does either raises its own lane or
records why it is skipping the independent read.

**B51 — The most expensive worker tier is a budget, and the budget refuses
(counter-teeth D3, 2026-08-04).** Trigger: assigning a unit the highest-cost
tier. What happens: the share that assignment would produce is computed across
the same feature's tiered units — the ceiling count over every unit carrying any
tier, with untiered units excluded from both sides — and the check reads the
share *after* the assignment, not before it. Above two fifths, the assignment is
refused, and the refusal states the counts, the resulting share, and the budget
it crossed. Only the highest tier is budgeted; assigning any cheaper tier is
never checked. Its one escape is a stated reason, which is persisted on the
unit's own trace, so an over-budget assignment is always readable afterwards as
a decision somebody made rather than a drift nobody noticed. What each actor
observes: the cost ceiling that used to be an advisory number in a decision
record now holds by itself, and the exceptions to it are named rather than
silent.

## Business Rules

- **A unit's verification command is authored at the narrowest scope that
  proves that unit (worker-proof-line-skew, cell wpls-1, 2026-08-21).** A name
  or module filter over the tests the unit's own change can break, or a parity
  check when the change is prose — never a copy of the project's declared test
  command. The copy buys nothing the push does not already buy, and it makes
  every worker pay a full build of everything before it can cap. The declared
  command belongs to continuous integration, which runs it on every push. The
  rule is authored here because the scope decision is made when the unit is
  written, not when it is capped.
- **A unit's verification command is one command that runs as written
  (agent-activity-hook aah-1/aah-2/aah-3, 2026-08-22).** The recurring defect
  is a filter list the test runner accepts only one of: three units of one
  feature were authored with two test filters in a single invocation, the
  runner refused each on first use, and every worker had to invent a
  replacement — so the proof recorded on the cap was never the command the
  author wrote. Two filters are two commands joined by `&&`, or one wider
  filter; never two positionals.

- R46 — A unit's change classification is set explicitly or derived only from
  the behavior-change flag — never any richer auto-derivation — and an
  insufficient verification plan is reported as an authoring-time warning on
  the advisory channel, never a refusal (self-correcting-loop D3, Δ4). The
  derivation also runs the OTHER way at authoring: a unit declared
  `change_class: behavior` whose payload does not explicitly set the
  behavior-change flag defaults it to TRUE, so the scribing-debt door arms
  itself for a declared behavior change; an explicit false in the payload is
  a deliberate opt-out and is respected (close-bookkeeping-p3 cell cbp-2,
  after review P3-5 found a behavior cell shipping with the flag silently
  false and the door never arming). The same defaulting covers the UPDATE
  door since review-p2-hardening cell rph-4 (2026-08-11): an update that
  sets `change_class: behavior` without explicitly setting the flag in the
  same call arms it too, through the one shared function the add path uses
  — re-classing away from behavior changes nothing (review B-P2-8). Worker
  records also stopped accumulating: registration upserts by
  nickname-plus-cell, so re-registering the same pair refreshes the live
  record instead of appending a stale twin (review B-P2-6).
- R56 — A regeneration obligation implied by a unit's own scope refuses the
  authoring write unless the verification carries the derived checks or the
  unit records a reasoned acknowledgement; the obligated roots are always
  derived from the regeneration tools themselves, never hard-coded in the
  guard (regen-obligation-derived D1/D2, 2026-07-23).
- R94 — *(Superseded 2026-07-31 — decision 412e9b3a,
  docs/specs/test-simple.md: the trailing-test-unit-per-slice mandate is
  deleted. Live rule: each unit's writer owns its tests TDD-style as part
  of the unit's own work; since 2026-08-18 (decisions 58ec9664/1f534837,
  refining 13ce1858) the writer also owns the proof scope: each cap
  records a proof line `<command> — <result> — <scope reason>` chosen per
  change type, `bee close`/`bee worktree merge` check that record and run
  nothing themselves, and CI runs the full declared command on every push.
  The coverage-judgement-first discipline itself survives as craft in
  `.bee/expertise/tests.md`. Kept below as the historical record.)*
  **Coverage judgement before coverage authoring.** The trailing test
  unit per slice is unconditional, and its first mandated step is to cite the
  nearest existing tests and judge whether they already cover the slice's
  acceptance criteria; only the uncovered gap is authored. Concluding "already
  covered, no new rows" is a legitimate outcome, never a defect
  (worker-conformance D4).
- R95 — **Test shape below the highest-risk lane is the triad at its smallest
  demonstrating size: happy path, edge cases, error paths.** The
  twelve-dimension edge checklist stops being the default at the standard lane
  and applies only to highest-risk and hard-gated work — read as a checklist
  to fill, it generated volume rather than coverage. *(The mechanical brakes
  this rule leaned on — the volume ceiling, the new-suite justification, and
  the refactor new-test-file refusal — were deleted 2026-07-31 by decision
  412e9b3a, docs/specs/test-simple.md; the triad shape and duplication
  judgment survive as craft in `.bee/expertise/tests.md`, enforced by
  review.)* (worker-conformance D5/D6).

- R98 — Authoring units into a feature that is still exploring or still
  planning, without its execution approval, refuses the whole batch and never
  part of it; the documentation lane is exempt, and a feature no record names is
  not refused (hook-teeth D3, cell bh-3, 2026-08-04).

- R102 — Assigning the highest worker tier refuses once the resulting share of
  the feature's tiered units would pass two fifths; the share counts tiered units
  only and is measured after the assignment, cheaper tiers are never budgeted,
  and the sole escape is a stated reason persisted on the unit's trace
  (counter-teeth D3, cell ct-4, 2026-08-04).
- R103 — A unit touching judge-required (guard) source at a lane below
  standard/high-risk refuses authoring unless it records
  `judge_obligation_ack`; the covered roots are pinned against the crate
  source tree itself in both directions — a stale root and an uncovered new
  guard directory each turn a test red — never hand-kept independent of it
  (pattern-20260812, cell jo-1, 2026-08-12).

## Edge Cases Settled

- One invalid unit in a batch slice-creation request → zero units written; a
  duplicate identifier inside the batch is refused the same way.
- A test unit that authors zero new rows because the existing suite already
  covers the criteria has discharged its mandate, not skipped it. It completes
  by running those tests green and recording the judgement with its citations
  (worker-conformance D4).

## Open Gaps

- *(Closed by supersession 2026-07-31 — decision 412e9b3a,
  docs/specs/test-simple.md: both sides of this gap — the trailing-test-unit
  mandate and the feature-level coverage door — are deleted, so the
  disagreement no longer exists. Kept as the historical record.)*
  **The planning doctrine and the machine disagree about batching the trailing
  test unit at the highest risk level, and the disagreement was recorded rather
  than resolved.** The doctrine says highest-risk work is never batched into a
  trailing test unit — each such unit proves itself red-first. The
  feature-level coverage door has no such exemption: it refuses the close of
  any feature holding completed code-touching behaviour work and no test unit,
  in every lane. A planner following the prose alone at the highest risk level
  leaves the feature unable to close, with no bypass level lifting it. The
  doctrine now states the gap where a planner meets it; the door's predicate
  was deliberately NOT changed, because loosening a close-door to match prose
  is the more dangerous of the two repairs. Named, not closed
  (worker-conformance D13).

## Pointers (implementation)

- Coverage-judgement-first trailing test unit and the triad test shape
  (B44/R94/R95): instruction text only — `skills/bee-planning/SKILL.md`,
  `skills/bee-planning/references/planning-reference.md`, and
  `skills/bee-planning/references/edge-dimensions.md` (the twelve dimensions,
  now scoped to high-risk/hard-gate). No source predicate implements these;
  the enforcing door is `testCellDebt` in `packages/bee/lib/state.mjs`.
  Evidence: traces `.bee/cells/wc-3.json`, `.bee/cells/wc-6.json`,
  `.bee/cells/wc-7.json`; reports
  `docs/history/worker-conformance/reports/wc-{3,6,7}.md`.
- Batch slice creation: `addCells` in `packages/bee/lib/cells.mjs`,
  CLI `bee cells add --stdin` (JSON array). Evidence: dispatcher-unify
  cells-batch-add suite rows (v0.1.27).
- Cell revision: `updateCell` + `UPDATE_FIELD_VALIDATORS`/`UPDATE_FROZEN_HINTS`
  in `packages/bee/lib/cells.mjs`; CLI `bee cells update --id ID
  --file patch.json | --stdin` (byte-mirrored to `.bee/bin/`). Evidence: cell
  `.bee/cells/cuv-1.json` (commit 127abb0), 7 suite checks.
- Derived regen obligation: `deriveManifestScope` + `REGEN_ACK_FIELD`
  (`regen_obligation_ack`) in `packages/bee/lib/cells.mjs`,
  enforced in `addCells`/`updateCell`; roots parsed from
  `scripts/release_manifest.mjs` and `scripts/ledger_parity.mjs`. Evidence:
  12 suite rows in `packages/bee/tests/test_bee_cli.mjs` +
  mutation red, commit e4ae329 (cell ro-1, 2026-07-23).
- Ceiling-tier budget refusal (B51/R102): `set_tier` and `ceiling_share_after`
  with `CEILING_SHARE_REFUSAL_MAX` (0.4) in
  `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:860-940` —
  the share is `ceiling / (extraction + generation + ceiling)` over
  `list_cells(root, feature, None)`, with the assigned cell removed and re-added
  under its new tier before the comparison. The escape is `--reason <text>`,
  persisted as `trace.tier_reason` (`handlers_close.rs:932-935`). Red-first per
  counter-teeth D6. Evidence: trace `.bee/cells/ct-4.json`, commit a5e564fa
  (cells 71 passed, 0 failed, 2026-08-04).
- Gated authoring refusal (B46/R98): `gated_add_refusal` in
  `packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs:115-156` — the
  gated phases are exactly `exploring` and `planning`, and a resolved record
  whose `mode` is `docs` is exempt. The refusal folds into
  `build_add_cells_report`'s per-row problems (`handlers_write.rs:179-197`), so
  one gated row fails the batch and nothing is written. Lane precedence mirrors
  `plan_freeze_shape_approved`'s. Evidence: trace `.bee/cells/bh-3.json` (cells
  slice 83 passed, 2026-08-04).
- Judge obligation for guard-touching units below standard/high-risk
  (B52/R103): `judge_obligation_refusal` / `assert_judge_obligation`,
  `JUDGE_REQUIRED_ROOTS`, and `JUDGE_ACK_FIELD` (`judge_obligation_ack`) in
  `packages/bee-rs/crates/bee/src/verbs/cells/obligation.rs`. The covered-lane
  check reads `JUDGE_DOOR_COVERED_LANES` (`["standard", "high-risk"]`),
  mirroring close.rs's judge-debt door gate (drivers/close.rs:697). The
  required-roots list is pinned against the crate source tree in both
  directions by `every_judge_required_root_exists_on_disk` and
  `every_guard_segment_directory_under_crate_src_is_covered_by_a_declared_root`
  in the same file. Evidence: trace `.bee/cells/jo-1.json`.
