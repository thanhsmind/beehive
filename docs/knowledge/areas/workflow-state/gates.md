---
type: bee.area
title: "Workflow State — starting a feature, the phase vocabulary, phase-owned routing, and closing"
description: "The guarded doors of a feature's life: the all-or-nothing start that can never inherit the previous feature's approvals (now also creating that feature's own workflow record), the closed phase vocabulary, the adviser consult high-risk execution approval demands (the one gate scoped to a plan revision), the phase-owned generic routing mutation, and the four-step tail that makes declaring a feature closed impossible."
timestamp: 2026-07-27
bee:
  id: workflow-state-gates
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: ["chain-integrity D1-REVISED/D2/D3/D4 (the tail of the chain: learning capture is produced not asserted, the sync demands executed work, the close demands zero spec debt, the waiver is audited)", "AO3/AO13 (execution-gate adviser precondition — folded from the old standalone Gate 3 into Gate 2, validation-diet D2/D14 — event-based staleness, never a TTL — cells ao-4-1/ao-4-2 2026-07-17)", codex-hook-state-parity D4-D6 (pre-phase routing ownership and review isolation), "scribing-integrity D1-D3/D6 (the wall at every door — feature-swap guard, lane-aware close, durable scribing ledger + orphan sweep, pre-ledger amnesty; cells si-1/si-3, 2026-07-24)", multisession-native D1/D7 (starting a feature also creates its own workflow record via three separate lock transactions; the execution gate records approved_for_plan_rev — docs/history/multisession-native/CONTEXT.md), "scribing-stamp-seam 5b2f963d (sss-1 — the close-door threshold folds in the durable ledger as fallback/max, not the record stamp alone)", "compounding-gate D1 (the close also demands recorded learning-capture evidence, fresh against the knowledge-sync stamp, waivable only with a logged decision; cells cg-1/cg-2, 2026-07-27)"]
  sources: ["chain-integrity cells ci-1/ci-2/ci-3 (traces in .bee/cells/, CONTEXT docs/history/chain-integrity/CONTEXT.md, 2026-07-14 — origin: an owner-supplied post-mortem of a real session in which the chain's tail was bypassed seven times)", "advisor-and-orchestration Slice 4 cells ao-4-1/ao-4-2 (adviser consult record + event-based staleness + high-risk execution precondition, live-throw verified, 2026-07-17)", "codex-hook-state-parity cell codex-hook-state-parity-1 (pre-phase routing ownership and review isolation; report and capped trace, 2026-07-16)", "docs/specs/workflow-state.md#B1", "docs/specs/workflow-state.md#B2", "docs/specs/workflow-state.md#B9a", "docs/specs/workflow-state.md#B19", "docs/specs/workflow-state.md#R1", "docs/specs/workflow-state.md#R2", "docs/specs/workflow-state.md#R3", "docs/specs/workflow-state.md#R19a", "docs/specs/workflow-state.md#R20a", "docs/specs/workflow-state.md#R21a", "docs/specs/workflow-state.md#R22", "docs/specs/workflow-state.md#R23", "docs/specs/workflow-state.md#R25", "docs/specs/workflow-state.md#R29", "docs/specs/workflow-state.md#R30", "docs/specs/workflow-state.md#R31", "docs/specs/workflow-state.md#E1", "docs/specs/workflow-state.md#E2", "docs/specs/workflow-state.md#P2", "docs/specs/workflow-state.md#P3", "docs/specs/workflow-state.md#P4", "docs/specs/workflow-state.md#P5", "multisession-native cell multisession-native-6 (startFeature creates a workflow record; trace .bee/cells/multisession-native-6.json, commit f4fe163, 2026-07-25)", "multisession-native cell multisession-native-9 (gates scoped to plan revision; trace .bee/cells/multisession-native-9.json, commit 2dd834f, 2026-07-25)", "scribing-stamp-seam cell sss-1 (trace .bee/cells/sss-1.json, capped 2026-07-26)", "compounding-gate cells cg-1/cg-2 (state compounding-run verb + close gate + mutation-proven suite; traces .bee/cells/cg-1.json/cg-2.json, 2026-07-27)"]
  authoritative_for: "workflow-state: feature start, the phase vocabulary, phase-owned routing mutation, and the closing tail"
---

# Workflow State — starting a feature, the phase vocabulary, phase-owned routing, and closing

A feature's life is bounded at both ends by a guard. At the start, one
all-or-nothing write that refuses unless the previous feature really finished —
so a new feature can never inherit the previous one's approvals or bury its
unfinished work. At the end, a three-step tail in which each step must prove the
step before it happened. Between them sit the two rules that keep every routing
write honest: the phase vocabulary is closed, and a generic routing mutation is
owned by the phase it started from.

## Behaviors & Operations

**B1 — Guarded feature start.** Starting a feature fails closed — with zero
changes to the record — unless ALL of: the prior phase is terminal; no handoff
record exists; no worker is registered; no file reservation is active; and the
prior feature has no nonterminal cell. An intentionally abandoned cell must
first be dropped through the explicit drop verb, which records the reason —
the start operation never clears work as a side effect. When the preconditions
hold, one atomic write sets the feature, its mode, a valid phase, resets all
four gates to ungranted, and updates the summary/next-action. Observers (the
next session's preamble, the status command) see either the old record intact
or the new feature fully reset — never a mixture.

**Since multisession-native slice 2, starting a feature also creates that
feature's own workflow record (D1), never as a side effect folded into the
legacy write.** The three writes — an idempotent seed of any live legacy
pipeline into a workflow record (so a first-ever workflow record in a repo
never erases mid-flight work), the unchanged legacy `state.json`/lane write,
and the new workflow-record creation — are three separate lock transactions,
never nested inside one another. Preconditions widen accordingly: the
nonterminal-cell and worker checks scope to *live workflows* plus
same-feature cells (not only the single legacy record), and the handoff
precondition is scoped per-feature on both the default and lane paths (a
different feature's pause snapshot never blocks this start). The schema, the
seeding, and the per-workflow lock this creates are owned by
`workflow-records-and-projections.md` — this concept keeps the guarded-start
rule (B1/R1/R2) itself; only *what backs* a successful start changed.

**B2 — Closed phase vocabulary.** Every phase write is validated against the
closed list; historical skill wording that used other names (e.g.
"exploring-complete", "validated") is invalid at the record layer.

**B9a — High-risk execution approval requires a live adviser consult.** Opening
the execution gate on a record in the high-risk mode refuses — typed error,
zero mutation, the corrective message naming every failed condition and the
exact consult flow — unless a non-stale adviser consult record exists. The
consult itself is orchestrator machinery, not a human checkpoint: it runs under
every autopilot level (autopilot lifts human stops, never mechanical
preconditions). The orchestrator resolves the configured adviser, runs it
**read-only** with an evidence bundle (plan summary, risk map, validation
findings — never session history, never secrets), and records the consult; a
workspace with no adviser configured records that fact and proceeds — the rule
adds one trigger, not a dependency on configuration. Revoking the execution
gate stamps the revocation moment, which makes any earlier consult stale.
Non-high-risk modes, the other three gates, and revocation writes are
untouched. Advice never approves a gate and never overrides a locked decision.
What each actor observes: the assistant sees either a clean approval or the
refusal with its fix; the audit trail gains the consult record; a worker's own
mid-flight consult loop (B9) is unchanged.

**The execution gate this opens is the one gate scoped to a plan revision
(D7).** Granting it stamps the workflow's *current* `plan_rev` onto the gate's
`approved_for_plan_rev`; a later `bee state plan-rev bump` on that same
workflow projects the gate back to ungranted without touching the stored
`approved` flag or any other workflow's gates. Context, shape, and review stay
unscoped. The full mechanics — the plan-rev-effective formula, the bump verb,
and what it does and does not invalidate — are owned by
`workflow-records-and-projections.md`; this concept keeps ownership of the
high-risk consult precondition itself.

**B19 — A generic routing mutation is phase-owned.** Trigger: a caller changes
phase, mode, feature, summary, or next action through the generic state command.
The command first reads the selected default or lane record strictly. A missing,
invalid, or mismatched pre-change owner refuses the operation with the record
byte-identical. A matching owner changes only that selected record; the owner is
not persisted, and a phase change makes the new phase the owner of the next
change. Gate writes remain separate and require no owner. Independent review
keeps its findings and decision inside its review-session record and never uses
generic routing mutation to change execution readiness.

### Closing a feature — the tail of the chain

Closing is the one stretch of the pipeline where each step must *prove* the step
before it happened. The phase vocabulary alone never granted that proof: the
names asserted history ("both the knowledge sync and the learning capture have
run"), while nothing checked whether either had. A feature could therefore be
marked closed straight from execution, and this is exactly what happened
repeatedly — the settled behavior of six completed units never reached the
specs, and the only trace was a knowledge-sync record that stayed empty.

Three rules now hold the tail together. Together they make "declare it closed"
impossible; the only way to close is to actually close.

**Entering learning capture is never an assertion.** The learning-capture phase
cannot be set directly, from any phase. It is *produced* — and only produced —
by recording a knowledge sync. Attempting to set it names the recording step as
the way. This means the phase is reachable if and only if a real sync was
stamped, because stamping it is the sole door.

**Recording a knowledge sync demands that work was executed.** The recording
step is refused unless the feature currently stands in a phase where execution
has actually happened (execution, independent review, or the sync itself). It is
not possible to sync the knowledge of work that was never done.

**Reaching the terminal state demands the phase before it AND zero spec debt.**
The terminal state may be entered only from learning capture, and only while no
completed behavior-changing unit is still missing from the specs. The refusal
names *every* such unit by identity — not a count — and discloses the waiver.
A refused close is side-effect-free: the phase is left exactly as it was.

**The waiver is a door, not a hole.** A feature whose settled behavior genuinely
belongs in no spec may still be closed, by waiving the debt explicitly. The
waiver permits the close and simultaneously records a durable decision naming
every unit whose behavior was left out. Nothing about it is silent, and nothing
about it is the default. It exists because a guard with no door gets a hole
punched in it — a fail-close with no sanctioned exit teaches its user to work
around the guard instead of through it.

**Reaching the terminal state also demands recorded learning-capture evidence
(compounding-gate D1, 2026-07-27).** The knowledge sync proved the specs are
current; nothing proved the learning capture itself ever ran — the phase name
asserted it, and under ceremony-cutting pressure the capture was skipped in
practice. Now the close is refused unless a learning-capture run was recorded
for this same feature at or after its knowledge-sync stamp; the refusal names
the recording step as the fix. The recording step is itself refused outside the
learning-capture phase and never changes phase. The same door discipline
applies: an explicit waiver may close without the evidence, and doing so logs a
durable decision naming the feature — sanctioned, audited, never the default.

Everything outside the tail stays permissive: moving backward to an earlier
phase is always legal (a failed feasibility check or a negative proof must be
able to return to planning), and returning to idle — the way an abandoned
exploration is dropped — is unaffected.

**The wall stands at every door, not only the front one (scribing-integrity
D1-D3, 8ef2bae6-adjacent decision of 2026-07-24).** Three holes let "done-looking"
work escape the close wall silently: a session that died after completing its
units never attempted the close at all; swapping the routing record to a NEW
feature abandoned the old one's debt with no session left to hit its wall; and
a per-feature lane close never computed debt (the wall read only the default
record). Now: swapping away from a feature with standing debt refuses exactly
like the close does (same exhaustive naming, same audited waiver); a lane close
checks the LANE feature's own debt against that lane record's own sync stamp;
and every sync stamp is also appended to a durable ledger
(`.bee/logs/scribing-runs.jsonl`) — the repair verb may stamp a feature that is
not the active one, so an orphan left by a dead session can be paid later. A
global sweep over every completed behavior-changing unit versus its feature's
best stamp (ledger, lane record, or the default record's own attributed stamp
— attribution by the stamp's OWN feature field, never by which feature happens
to be active) surfaces orphaned debt in the status payload and as one loud
session-start line. Historical pre-ledger features received one audited
backfill stamp (amnesty decision): the alarm starts at zero real debt, because
an alarm born crying 119 teaches everyone to ignore the 120th.

**The close door's own threshold now trusts the same ledger the sweep already
reads (scribing-stamp-seam, decision 5b2f963d).** Before this, the front
door's threshold came only from the record's own sync-stamp field, and that
field's write does not always survive a workflow-record-backed feature's next
rebuild (`workflow-records-and-projections.md` — the rebuild spreads a fresh
read of the record over the in-memory mutation, so an in-flight stamp can
vanish before it is ever read back). A feature whose sync had genuinely run
could therefore still be refused at the close door and pushed into the
audited waiver for no real reason. The threshold is now the later of the
record's own stamp and the durable ledger's newest entry for that same
feature — exactly the source the orphan sweep already trusts — so a sync that
reached the ledger clears the close door even when the record-field write did
not. A cell capped after the true sync still counts as debt, and a ledger
entry belonging to a different feature never clears this feature's own debt.

**What each actor observes.** The agent attempting a dishonest close gets a
refusal that says which step was skipped and how to perform it, and the record is
untouched. The human sees a feature that cannot be reported as finished until
its knowledge actually landed — the state and the specs can no longer disagree.

## Business Rules

- R1 — A new feature can never inherit gate approvals: all four gates reset in
  the same atomic write that sets the feature (codex-runtime-parity D2;
  plan-review P1 repair).
- R2 — Feature start never destroys evidence of unfinished work; abandonment
  is a separate, recorded act (drop verb) (codex-runtime-parity D2).
- R3 — Phase values outside the closed vocabulary are rejected at the record
  layer, whatever a skill's prose says.
- R19a — The learning-capture phase is never settable. It is produced only by
  recording a knowledge sync, which is its sole door; any attempt to set it
  directly is refused and names the recording step as the way. Consequently the
  phase is reachable if and only if a knowledge sync was truly stamped
  (chain-integrity D1-REVISED).
- R20a — Recording a knowledge sync is refused unless the feature stands in a
  phase where execution has happened (execution, independent review, or the sync
  itself). Knowledge of work that was never done cannot be synced
  (chain-integrity D3).
- R21a — The terminal state may be entered only from learning capture, and only
  while spec debt is zero. The refusal names every completed behavior-changing
  unit still missing from the specs, by identity, and leaves the phase untouched.
  A close whose debt is genuinely spec-irrelevant proceeds only through an
  explicit waiver, which records a durable decision naming every waived unit —
  never silently, never by default (chain-integrity D2/D4).
- R22 — Spec debt is advisory everywhere it is displayed and binding only at the
  close. Debt is a signal throughout the work and a wall at the door: blocking on
  it mid-work would fire while the sync is not yet due, and never blocking on it
  at all is precisely what allowed a feature to be closed with its settled
  behavior absent from every spec (chain-integrity D2).
- R23 — No instruction anywhere in the workflow may name a phase outside the
  closed vocabulary. A documented command that names a non-existent phase fails
  every time it is followed, and an agent whose documented command fails begins
  improvising the state machine — which is how the tail came to be bypassed in
  the first place. This rule is machine-checked, not remembered
  (chain-integrity D6).
- R25 — The gate bypass level is a strict ladder of floors, each honored
  literally: `off` stops for every gate; `normal` lifts only the
  tiny/small/standard gates 1-2; `full` additionally lifts high-risk and
  hard-gate gates 1-2; `total` lifts everything, including secret-file reads and
  a review's P1 findings, leaving no human checkpoint. A human who set `full` or
  `total` deliberately removed the high-risk floor — the workflow never
  re-erects a stop the human lifted at their chosen level. When bypass is active
  the agent does not pause: it records the recommended choice, logs a one-line
  audit decision, and continues. Whenever any level other than `off` is in
  force, the status surface and the session preamble print a loud level-specific
  banner (`NORMAL` / `FULL AUTOPILOT` / `TOTAL AUTOPILOT — ZERO STOPS`) so the
  lifted floor is never silent (decision 0010; user authorization dcf01d7b).
  This ladder is applied at **every** gate step, not just some: each
  gate-presenting step reads the active level and self-approves before it would
  present, so a runtime that follows a step literally (rather than inferring the
  rule from doctrine elsewhere) still honors the level. A machine-check asserts
  every gate surface carries the level-aware rule and none carries a stale
  floor-is-absolute phrasing, so the per-gate application cannot silently
  regress (decision 5aedc024; cell codex-bypass-per-skill-1). Bypass suppresses
  **approvals**, never genuine **information-gathering**: under `full`/`total`
  the agent never asks merely to be approved (it takes its own confident best
  answer and proceeds), but a question whose answer only the human holds — a
  preference or knowledge the agent cannot settle from evidence — is still asked,
  including during exploring. The litmus is "do I already have a confident best
  answer?": yes proceeds, no-and-only-the-human-knows still asks (decision
  a93994d3; cell bypass-info-vs-approval-1).
- R29 — Every generic routing mutation is authorized by the selected record's
  valid pre-change phase. Default and lane records follow the same rule.
- R30 — Routing ownership is derived, never persisted. A successful phase
  change transfers authority to the new phase for the next mutation.
- R31 — Gate mutation is a dedicated operation; review owns no active pipeline
  state, and validation alone decides execution readiness.
- R78 — The close-door and feature-swap threshold is the later of the
  record's own sync stamp and the durable ledger's newest entry for that
  feature — never the record stamp alone. This closes the seam where a
  workflow-record rebuild could drop an in-flight stamp write and force a
  genuine sync into the waiver path (scribing-stamp-seam decision 5b2f963d).

## Edge Cases Settled

- A capped prior-feature cell never blocks a new start; an expired-by-TTL
  reservation never blocks a new start (only active ones do).
- Refused starts are proven side-effect-free: the record is byte-identical
  after a refusal.

## Open Gaps

- **The record-field stamp can still be dropped by the workflow-record
  projection rebuild.** The ledger fallback (R78) means a dropped stamp no
  longer blocks the close, but the drop itself is unrepaired: a
  workflow-backed feature's sync-stamp field can still project as empty even
  though the sync genuinely ran and is durably recorded in the ledger.
  Repairing the projection itself — so the field survives the rebuild — was
  explicitly left out of scope when the ledger fallback shipped: the ledger
  fallback was judged the required fix, and the field-level repair a
  separate, larger change to the workflow-record write path.

## Pointers (implementation)

- Record: `.bee/state.json` (CLI-owned). Verbs: `bee.mjs state`
  (`start-feature` — new; set/gate/worker/scribing-run — existing);
  `startFeature()` + `isKnownPhase` in `packages/bee/lib/state.mjs`
  (byte-mirrored to `.bee/bin/lib/state.mjs`).
- Phase-owned routing: generic `state set --owner <pre-phase>` in
  `packages/bee/bee.mjs` and `.bee/bin/bee.mjs`; required-owner
  metadata in both command registries; phase-aware callers in exploring,
  planning, and compounding (there is no `validating` phase — validation-diet
  D3 retired it; the merged gate's execution component is carried by
  `planning`). Review stays local to its review
  record. Proof: state/CLI suites, `.bee/cells/codex-hook-state-parity-1.json`,
  and `docs/history/codex-hook-state-parity/reports/codex-hook-state-parity-1.md`.
- Tests: 15 start-feature rows in `packages/bee/tests/test_lib.mjs`.
- Evidence: commit `928abf1`; trace `.bee/cells/codex-parity-5.json`.
- Workflow-record creation on start (D1) and plan-rev-scoped gates (D7): see
  `workflow-records-and-projections.md` Pointers — `seedLegacyWorkflows`,
  `createWorkflow`, `workflowGatesToApprovedGates`, and the `plan-rev bump`
  verb. Evidence: traces `.bee/cells/multisession-native-{6,9}.json`, commits
  f4fe163, 2dd834f.
- Close-door threshold + ledger fallback: `scribingDebt`,
  `bestScribingStampMs`, `readScribingLedger` in `packages/bee/lib/cells.mjs`
  (mirrored `.bee/bin/lib/cells.mjs`). Evidence: trace
  `.bee/cells/sss-1.json`, decision `5b2f963d`.
