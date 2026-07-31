---
type: bee.area
title: "Workflow State — authoring a unit of work, revising its plan, and the frozen plan document"
description: "How a slice of work units is created all-or-nothing, which of a unit's plan fields may be revised afterwards and which are frozen audit, how a unit's change is classified at authoring, how a scope-derived regeneration obligation refuses authoring without it, and why the approved plan document stops changing the moment its gate is granted."
timestamp: 2026-07-29
bee:
  id: workflow-state-cells-authoring-and-revision
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: ["lane-ceremony-v3 D1/D2/D9 (docs/history/lane-ceremony-v3/CONTEXT.md, 2026-07-19 — plan document frozen at shape approval, slice-in-units)", self-correcting-loop D3 with Validating amendment Δ4 (change classification and the advisory verification standard), "regen-obligation-derived D1/D2 (derived regen obligation refuses at authoring, recorded escape hatch; roots derived from the tools, never hard-coded — 2026-07-23)", "8ef2bae6 (cli-ergonomics D2 — whole-batch exhaustive refusal + --dry-run preview, 2026-07-24)", "worker-conformance D4/D5/D6 (the trailing test unit stays unconditional but its first mandated step is a coverage judgement, not authoring; test shape below the highest-risk lane is the happy-path/edge-cases/error-paths triad and the twelve-dimension checklist applies only to high-risk/hard-gate work; no numeric per-group test cap is added — 2026-07-29)", "worker-conformance D13 (the doctrine-vs-machine disagreement over batching the trailing test unit at high risk is recorded as an open gap, not fixed — the close-door predicate was deliberately left unchanged)"]
  sources: [cells-update-verb cell cuv-1 (2026-07-12), dispatcher-unify cells-batch-add suite rows (v0.1.27), "post-advisor-hardening cell pah-2 (cells add/update manifest-lint advisory, 2026-07-18)", "lane-ceremony-v3 cells lcv3-1..lcv3-5 (traces in .bee/cells/, reports docs/history/lane-ceremony-v3/reports/, 2026-07-19)", "worker-conformance cells wc-3/wc-6/wc-7 (coverage-judgement-first trailing test unit, triad test shape, ten doctrine overstatements corrected against the live predicates; traces .bee/cells/wc-{3,6,7}.json, reports docs/history/worker-conformance/reports/, CONTEXT docs/history/worker-conformance/CONTEXT.md, 2026-07-29)", "regen-obligation-derived cell ro-1 (12 suite rows + mutation red, commit e4ae329, 2026-07-23)", "docs/specs/workflow-state.md#B7", "docs/specs/workflow-state.md#B10", "docs/specs/workflow-state.md#B25", "docs/specs/workflow-state.md#B29", "docs/specs/workflow-state.md#R46", "docs/specs/workflow-state.md#E14", "docs/specs/workflow-state.md#P9", "docs/specs/workflow-state.md#P12"]
  authoritative_for: "workflow-state: unit-of-work authoring, plan revision, and the frozen plan document"
---

# Workflow State — authoring a unit of work, revising its plan, and the frozen plan document

A unit of work is written down before it is done, and what is written down has
two halves with opposite rules: the PLAN, which may be revised while the unit is
still open, and the RECORD of what happened, which may never be. This concept
owns the authoring door itself — the all-or-nothing batch, the revision guard,
the authoring-time classification — plus the one document that stops being
revisable at all once its gate is granted.

## Behaviors & Operations

**B7 — Cell plans are revisable in place, execution records never.** A unit of
work's PLAN fields (title, action, scope files, reading list, dependencies,
cited decisions, acceptance contracts, verify command, lane, behavior flag)
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

## Business Rules

- R46 — A unit's change classification is set explicitly or derived only from
  the behavior-change flag — never any richer auto-derivation — and an
  insufficient verification plan is reported as an authoring-time warning on
  the advisory channel, never a refusal (self-correcting-loop D3, Δ4).
- R56 — A regeneration obligation implied by a unit's own scope refuses the
  authoring write unless the verification carries the derived checks or the
  unit records a reasoned acknowledgement; the obligated roots are always
  derived from the regeneration tools themselves, never hard-coded in the
  guard (regen-obligation-derived D1/D2, 2026-07-23).
- R94 — *(Superseded 2026-07-31 — decision 412e9b3a,
  docs/specs/test-simple.md: the trailing-test-unit-per-slice mandate is
  deleted. Live rule: each unit's writer owns its tests TDD-style as part
  of the unit's own work, and `bee cells finish` runs the declared suite.
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
  CLI `bee.mjs cells add --stdin` (JSON array). Evidence: dispatcher-unify
  cells-batch-add suite rows (v0.1.27).
- Cell revision: `updateCell` + `UPDATE_FIELD_VALIDATORS`/`UPDATE_FROZEN_HINTS`
  in `packages/bee/lib/cells.mjs`; CLI `bee.mjs cells update --id ID
  --file patch.json | --stdin` (byte-mirrored to `.bee/bin/`). Evidence: cell
  `.bee/cells/cuv-1.json` (commit 127abb0), 7 suite checks.
- Derived regen obligation: `deriveManifestScope` + `REGEN_ACK_FIELD`
  (`regen_obligation_ack`) in `packages/bee/lib/cells.mjs`,
  enforced in `addCells`/`updateCell`; roots parsed from
  `scripts/release_manifest.mjs` and `scripts/ledger_parity.mjs`. Evidence:
  12 suite rows in `packages/bee/tests/test_bee_cli.mjs` +
  mutation red, commit e4ae329 (cell ro-1, 2026-07-23).
