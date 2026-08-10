---
type: bee.area
title: "Workflow State — starting a feature, the phase vocabulary, phase-owned routing, and closing"
description: "The guarded doors of a feature's life: the all-or-nothing start that can never inherit the previous feature's approvals (now also creating that feature's own workflow record), the closed phase vocabulary, the adviser consult high-risk execution approval demands (the one gate scoped to a plan revision), the phase-owned generic routing mutation, the four-step tail that makes declaring a feature closed impossible, and the soft promote door riding the green path once that tail clears."
timestamp: 2026-08-05
bee:
  id: workflow-state-gates
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: ["chain-integrity D1-REVISED/D2/D3/D4 (the tail of the chain: learning capture is produced not asserted, the sync demands executed work, the close demands zero spec debt, the waiver is audited)", "AO3/AO13 (execution-gate adviser precondition — folded from the old standalone execution gate into Gate 2, validation-diet D2/D14 — event-based staleness, never a TTL — cells ao-4-1/ao-4-2 2026-07-17)", codex-hook-state-parity D4-D6 (pre-phase routing ownership and review isolation), "scribing-integrity D1-D3/D6 (the wall at every door — feature-swap guard, lane-aware close, durable scribing ledger + orphan sweep, pre-ledger amnesty; cells si-1/si-3, 2026-07-24)", multisession-native D1/D7 (starting a feature also creates its own workflow record via three separate lock transactions; the execution gate records approved_for_plan_rev — docs/history/multisession-native/CONTEXT.md), "scribing-stamp-seam 5b2f963d (sss-1 — the close-door threshold folds in the durable ledger as fallback/max, not the record stamp alone)", "validation-diet D2/D14/D15 (the merged approval flips `shape` and `execution` together via `bee state gate --merge`, inherits the high-risk advisor-consult precondition D14 previously guarding execution alone, and stamps `approved_for_plan_rev` on both fields together so a later bump can never leave the merged approval half-revoked — docs/history/validation-diet/CONTEXT.md, cell vd-3, 2026-07-28)", "compounding-gate D1 (the close also demands recorded learning-capture evidence, fresh against the knowledge-sync stamp, waivable only with a logged decision; cells cg-1/cg-2, 2026-07-27)", derived-check-hardening E5 (the six hand-copied terminal-phase memberships are pinned by a parity suite rather than refactored to derive from the phase enum), "knowledge-loop D2/D9 (a soft promote door on the green path, after the tests and scribing-debt doors, computed before auto_archive_on_close retires the closing feature's cells; cells kl-3/kl-5, 2026-08-05)", "cc4b381a (advisor-gate-port Gate 2: port the consult anchors, the event-based staleness rule, the record/show verbs, and the high-risk execution precondition to the single remaining runtime — cells agp-1/agp-2, 2026-08-05)", "ae0a96ec (feature-swap-door Gate 2: the swap wall runs natively off the shared debt counter and gains the close door's capture-deferral escape; the waiver names the abandoned feature — cell fsd-1, 2026-08-05)", "2b35d98c (debt-door-archive Gate 2: every scribing-debt counter reads the archive as well as the active store, active copy wins on a duplicate id, and one parity test pins all four counters — cells dda-1/dda-2, 2026-08-06)", "20969403 (gate-door-refusal: the high-risk execution refusal states its own cause instead of rendering as an argument-shape complaint; the truthful refusal unblocks nobody and the real unblock deadlocks against the door it repairs — cell gdr-1, 2026-08-04)", "js-parity-cleanup D2 (docs/history/js-parity-cleanup/CONTEXT.md, 2026-08-04 — a stored approvals map merges over the defaults only when it is an object; every other shape takes the defaults whole, and the position-keyed spread inherited from a foreign language's accident is gone)", "counter-teeth D1/D6 (docs/history/counter-teeth/CONTEXT.md, 2026-08-04 — the close refuses uncaptured behavior-changing units unless a logged capture-deferral decision names the feature; the refusal names the units and both remedies, and the counter is proven before the flip)"]
  sources: ["chain-integrity cells ci-1/ci-2/ci-3 (traces in .bee/cells/, CONTEXT docs/history/chain-integrity/CONTEXT.md, 2026-07-14 — origin: an owner-supplied post-mortem of a real session in which the chain's tail was bypassed seven times)", "advisor-and-orchestration Slice 4 cells ao-4-1/ao-4-2 (adviser consult record + event-based staleness + high-risk execution precondition, live-throw verified, 2026-07-17)", "codex-hook-state-parity cell codex-hook-state-parity-1 (pre-phase routing ownership and review isolation; report and capped trace, 2026-07-16)", "docs/specs/workflow-state.md#B1", "docs/specs/workflow-state.md#B2", "docs/specs/workflow-state.md#B9a", "docs/specs/workflow-state.md#B19", "docs/specs/workflow-state.md#R1", "docs/specs/workflow-state.md#R2", "docs/specs/workflow-state.md#R3", "docs/specs/workflow-state.md#R19a", "docs/specs/workflow-state.md#R20a", "docs/specs/workflow-state.md#R21a", "docs/specs/workflow-state.md#R22", "docs/specs/workflow-state.md#R23", "docs/specs/workflow-state.md#R25", "docs/specs/workflow-state.md#R29", "docs/specs/workflow-state.md#R30", "docs/specs/workflow-state.md#R31", "docs/specs/workflow-state.md#E1", "docs/specs/workflow-state.md#E2", "docs/specs/workflow-state.md#P2", "docs/specs/workflow-state.md#P3", "docs/specs/workflow-state.md#P4", "docs/specs/workflow-state.md#P5", "multisession-native cell multisession-native-6 (startFeature creates a workflow record; trace .bee/cells/multisession-native-6.json, commit f4fe163, 2026-07-25)", "multisession-native cell multisession-native-9 (gates scoped to plan revision; trace .bee/cells/multisession-native-9.json, commit 2dd834f, 2026-07-25)", "scribing-stamp-seam cell sss-1 (trace .bee/cells/sss-1.json, capped 2026-07-26)", "compounding-gate cells cg-1/cg-2 (state compounding-run verb + close gate + mutation-proven suite; traces .bee/cells/cg-1.json/cg-2.json, 2026-07-27)", "derived-check-hardening cell dch-6 (terminal-phase parity suite over 12 discovered declarations vs KNOWN_PHASES; trace .bee/cells/dch-6.json, report docs/history/derived-check-hardening/reports/dch-6.md, 2026-07-29)", "knowledge-loop cell kl-3 (soft promote door; trace .bee/cells/kl-3.json, commit 384587a1, 2026-08-05)", "knowledge-loop cell kl-5 (promote computed before cell retirement; trace .bee/cells/kl-5.json, commit c8d25dff, 2026-08-05)", CONTEXT.md `docs/history/knowledge-loop/CONTEXT.md`, "advisor-gate-port cells agp-1/agp-2 (anchors + staleness + record/show verbs, then the high-risk execution precondition; traces .bee/cells/agp-1.json/agp-2.json, commits fb94ba8f and 6fefd6ee, capped 2026-08-05)", "feature-swap-door cell fsd-1 (native swap door + shared escapes + after-write waiver naming the abandoned feature, six run_set tests; commit 41d8b0e6, capped 2026-08-05)", "debt-door-archive cells dda-1/dda-2 (archive-aware debt enumeration behind all four counters plus the four-way parity test; commits fd5f8253 and e44e56e9, capped 2026-08-06)", "counter-teeth cell ct-2 (the close door and its refusal share one computed verdict; trace .bee/cells/ct-2.json, commit bf7f022f, 2026-08-04 — drivers 47 passed, 0 failed)", "gate-door-refusal cell gdr-1 (both refusal arms return a stated refusal naming the advisor cause instead of a bare unsupported-argument-shape error; trace .bee/cells/gdr-1.json, 2026-08-04 — state_group tests green)", "js-parity-cleanup cell jp-4 (one native approvals merge; the exotic-shape error branch and compaction's masking fallback deleted; trace .bee/cells/jp-4.json, 2026-08-04 — 999 passed, 0 failed)"]
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
four gate fields to ungranted, and updates the summary/next-action. Observers (the
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

**B2a — The hand-copied terminal-phase memberships are pinned to the
vocabulary by a standing check.** Trigger: the verification chain runs. What
is checked: the set of phases meaning no work is in flight is written out by
hand in six places under three different names, each place layering its own
semantics on the shared list, and every one of those copies is asserted to
agree with every other and with the closed vocabulary itself — the canonical
copies and their mirrored twins alike. The copies are found by scanning for
the declaration's shape rather than read from a list of locations, so a
seventh copy is covered the moment it appears and no maintained inventory can
go stale behind the check. What happens on drift: the run fails naming the
exact place and line that disagrees, not merely that something disagrees.
What each actor observes: the six copies stay a duplication with an alarm on
it rather than a duplication with no owner. Why a check and not a
consolidation: each copy carries its own layered meaning, so deriving all six
from the vocabulary is a much larger change, while the copy that governs
write-denial is the one that must never drift silently — the parity check
catches that same class at a fraction of the risk
(derived-check-hardening E5).

**B9a — High-risk execution approval requires a live adviser consult, on
whichever door opens it.** Opening the execution gate on a record in the
high-risk mode refuses — typed error, zero mutation, the corrective message
naming every failed condition and the exact consult flow — unless a
non-stale adviser consult record exists. The same precondition guards the
merged shape+execution approval (`bee state gate --merge`, validation-diet
D2/D14): a high-risk record cannot flip `shape` and `execution` together
through the merged path without that same live consult either — the merge
*inherits* the precondition rather than relaxing it, which is what moves the
check earlier in the chain, onto planning's single merged question, instead
of only a later standalone execution approval. The consult itself is
orchestrator machinery, not a human checkpoint: it runs under every autopilot
level (autopilot lifts human stops, never mechanical preconditions). The
orchestrator resolves the configured adviser, runs it **read-only** with an
evidence bundle (plan summary, risk map, validation findings — never session
history, never secrets), and records the consult; a workspace with no
adviser configured records that fact and proceeds — the rule adds one
trigger, not a dependency on configuration. Revoking the execution gate
stamps the revocation moment, which makes any earlier consult stale.
Non-high-risk modes, the context and review gates, and revocation writes are
untouched. Advice never approves a gate and never overrides a locked
decision. What each actor observes: the assistant sees either a clean
approval or the refusal with its fix, on either door; the audit trail gains
the consult record; a worker's own mid-flight consult loop (B9) is unchanged.

**B9b — What makes a consult live: four anchors, no clock.** A recorded
consult carries the state of the work it actually saw, and it is stale the
moment ANY of those four anchors moves: the feature it was taken for, the
newest recorded decision at the time it was taken, the content fingerprint of
the plan it read, and the execution-gate revocation stamp. Nothing expires on
elapsed time — a consult on unchanged work stays live indefinitely, and a
consult on work that moved is stale immediately, however recent. Trouble
reading the record is not a pass: a missing, unreadable, or malformed consult
record reads as *stale*, never as an error and never as approval, so the
door's failure direction is always toward refusing. The refusal names which
anchor moved, so the fix is the next action rather than a guess. Two read-only
verbs expose the same rule outside the door: one records a consult against the
current anchors, one shows the recorded consult and whether it is still live —
so the state can be inspected before the approval is attempted (cells agp-1
and agp-2, 2026-08-05).

**Approving shape and execution together now scopes both to the plan
revision; approving either alone still scopes only itself (validation-diet
D2/D15, widening multisession-native D7's execution-only scope).** Granting
the merged approval stamps the workflow's *current* `plan_rev` onto BOTH the
`shape` and `execution` fields' `approved_for_plan_rev`; a later `bee state
plan-rev bump` on that same workflow projects both back to ungranted
together — never a half-revoked merged gate — without touching the stored
`approved` flags or any other workflow's gates. Approving `shape` or
`execution` individually through the standalone `--name` path stamps only
that one field, exactly as before the merge existed. Context and review are
never plan-rev-scoped by any path. The full mechanics — the plan-rev-effective
formula, the bump verb, and what it does and does not invalidate — are owned
by `workflow-records-and-projections.md`; this concept keeps ownership of the
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

**The door itself had gone dark, independent of any feature's real debt
(terminal-phase-port, cell tpp-1, 2026-08-04).** Both routes into the terminal
state — the default record's and a lane record's alike — had been left
delegating to a runtime that no longer existed, since an earlier cutover
replaced it and never carried this one door along. The write refused every
time, with an error indistinguishable from a real precondition failure, so no
feature and no lane could enter the terminal state at all — however clean its
debt — until the door was rebuilt to run natively, the same day the gap was
found. The door as it now stands has two halves, both native and both loud:
the learning-capture freshness evidence described below, and the spec-debt
threshold R78 already defines.

**The waiver is a door, not a hole.** A feature whose settled behavior genuinely
belongs in no spec may still be closed, by waiving the debt explicitly. The
waiver permits the close and simultaneously records a durable decision naming
every unit whose behavior was left out. The door also accepts a second,
equally loud escape: an already-logged deferral decision that already names
the feature clears the debt without demanding a fresh waiver — the same
acknowledged gap is never logged twice. Nothing about either escape is silent,
and nothing about either is the default. It exists because a guard with no
door gets a hole punched in it — a fail-close with no sanctioned exit teaches
its user to work around the guard instead of through it.

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

**Retiring a unit's records never retires its debt (debt-door-archive, cells
dda-1/dda-2, 2026-08-06).** A green close moves the closed feature's units out
of the active store into its archive. Every debt counter used to read the
active store alone, so from that moment the count was structurally zero and a
clear door was indistinguishable from a paid debt — the door reported on the
enumeration, not on the question. Debt is now counted over the active store
AND the feature's archive, one unit per id with the active copy winning when
both hold the same id, and the threshold rule is unchanged: an archived unit
counts exactly when it completed after the feature's best sync stamp. Four
places compute this count — the close and swap walls, the status payload, the
session-start line, and the mid-session nudge — and one parity test pins them
to the same answer over a fixture that mixes an active unit, an archived one,
and one id in both places. The wall's own precondition for retiring units is
unchanged and needs no new door: with the counter honest, retiring records can
no longer hide what they owe.

**The swap wall asks the same question the close wall asks, and takes the same
two escapes (feature-swap-door, cell fsd-1, 2026-08-05).** Both doors now count
debt through one shared counter and read one shared deferral record, so they
can never disagree about what counts as unpaid: the swap door refuses on the
OUTGOING feature's standing debt, and clears on either the explicit waiver flag
or an already-logged capture-deferral decision naming that same outgoing
feature — the escape the close door has always accepted, now reaching the swap
too. A waived swap logs its own decision after the write succeeds, naming the
ABANDONED feature rather than the newly-set one, because by then the routing
record already holds the new feature and a record naming it would be a lie.
Nothing reaches disk on a refusal.

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

**A soft promote door rides the green path, after the tests door and the
scribing-debt door (knowledge-loop D2, cell kl-3).** Once a close request has
already cleared both hard doors above, close now also runs `bee knowledge
promote` for the closing feature in process, prints one headline naming its
proposal counts, and writes the full proposal to
`docs/history/<slug>/promote-proposals.md`. This door is SOFT: unlike the
tests and scribing-debt doors, it never refuses the close and never changes
close's exit code. A promote outcome of `None` (the retired Node delegate arm)
and a `Thrown` outcome (e.g. an `unknown_work` refusal) both degrade to the
same one warning line rather than two different failure shapes. Nothing the
door writes lands under `docs/knowledge/` — `promote` still only proposes
(B5/D38 in `context-and-promote.md`), so this door closes the loop only as far
as the proposal, exactly as that guarantee promises.

**The door computes its proposal before the close sequence retires the
feature's cells (knowledge-loop D9, cell kl-5).** `build_promotion` mines
`.bee/cells/*.json`, and the retirement step (`auto_archive_on_close`) moves a
closing feature's capped cells into `.bee/cells/archive/` — so a door that ran
after retirement always scanned an empty directory and the proposal came back
empty on every real close. `build_promotion` is read-only, so moving the call
ahead of retirement has no side effect other than letting it see the cells the
closing feature just capped; the headline still prints at the same place in
close's output.

**What each actor observes.** The agent attempting a dishonest close gets a
refusal that says which step was skipped and how to perform it, and the record is
untouched. The human sees a feature that cannot be reported as finished until
its knowledge actually landed — the state and the specs can no longer disagree.
And a feature that closes clean now also sees its earned knowledge proposed,
not merely permitted to sync — the promote door names the counts and the file,
never refusing the close over what it finds.

**B53 — A stored approvals map is read as an object or not at all
(js-parity-cleanup D2, 2026-08-04).** Trigger: any read of a record's approvals
field, by any of the many surfaces that project it. What happens: when the stored
value is an object, its keys are laid over the four defaults, so a record that
names only one gate still answers for all four; when it is anything else at all —
absent, empty, a list, a string, a number, a flag — the defaults are taken whole
and the stored value contributes nothing. There is no third outcome: no partial
read, and no refusal to answer. Why it is stated: the previous behavior derived
its rules from a foreign language's own accident, so a text value spread into
position-keyed entries and produced an approvals map whose keys were positions,
which nothing in the system would ever write and nothing downstream could use.
That path is gone. What each actor observes: a record hand-edited into a shape
the workflow never writes now reads as *no approvals recorded*, which is the safe
answer and the honest one — approvals are earned through the approval verb, and
a malformed record has earned none.

## Business Rules

- R104 — An approvals map merges over the four defaults only when it is stored as
  an object; every other shape yields the defaults untouched, and no shape is
  read partially or refused (js-parity-cleanup D2, cell jp-4, 2026-08-04).
- R1 — A new feature can never inherit gate approvals: all four gate fields reset in
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
  A close whose debt is genuinely spec-irrelevant proceeds through either of two
  escapes: an explicit waiver, which records a durable decision naming every
  waived unit, or an already-logged deferral decision that already names the
  feature — never silently, never by default (chain-integrity D2/D4;
  terminal-phase-port, cell tpp-1, 2026-08-04).
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
  tiny/small/standard Gates 1-2; `full` additionally lifts high-risk and
  hard-gate Gates 1-2; `total` lifts everything, including secret-file reads and
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
- R79 — Every hand-written copy of the terminal-phase membership must agree
  with every other copy and with the closed phase vocabulary; drift fails the
  verification chain naming the offending place and line. The copies are
  discovered by the shape of their declaration, never from a maintained list
  of locations, so the check cannot go blind to a copy nobody registered
  (derived-check-hardening E5).
- R80 — `bee close`'s promote proposal is computed after the tests and
  scribing-debt doors pass, but before the feature's cells are retired. It is
  a SOFT door: a `None` or a `Thrown` promote outcome both degrade to one
  warning line, close's exit code never changes because of this door, and
  nothing it writes lands under `docs/knowledge/` (knowledge-loop D2/D9).
- R81 — A `bee close` that ends green — including the no-declared-tests
  pass-through, which proceeds with a teaching note and is treated as green
  here — and is not `--dry-run` sweeps its own tracked bee-store dirt with a
  path-scoped commit: when the root is a git work tree (detected
  via `rev-parse --is-inside-work-tree`, so a linked worktree's `.git` FILE
  counts) and `git status --porcelain -- .bee` is non-empty, close runs
  `git add -A -- .bee` then `git commit -m "Record <feature> close
  bookkeeping in the bee store" -- .bee` and reports `bookkeeping_commit`
  ({committed, sha} or {committed:false, reason: clean | config_off |
  not_a_repo | git_failed:<line>}) in its output. The commit is path-scoped
  so unrelated dirty AND staged files are never swept (no-whole-tree-git
  law). Warn-never-block: a git failure is one warning line and close's exit
  stays green — and the failure path cleans up after itself: the scoped
  `git add` is undone best-effort (`git reset -- .bee`), the report carries
  `index_restored: true|false`, and the warning says "index restored" or
  "WARNING: .bee left staged"; the `git_failed:` reason is never bare — a
  silent failing hook renders as `git_failed:exit status <code>`
  (close-bookkeeping-hardening cell cbh-1, 2026-08-10, review P2-1/P2-2).
  `.bee/config.json` `close_commit_bookkeeping: false` opts out; a
  non-boolean value REFUSES the whole close up front — typed error naming
  the key and the offending value, before the tests run, nothing committed
  (worktree_cleanup_on_merge precedent, made real by cbh-1 after review
  P2-3 found the first ship silently read it as off). A red close and
  `--dry-run` never commit. The bookkeeping commit passes `--no-gpg-sign`
  and git runs with stdin nulled — bee's own bookkeeping commit is unsigned
  by chosen policy, so a signing repo's pinentry can never hang close
  (close-bookkeeping-p3 cell cbp-1). Named cost, chosen not accidental: the
  sweep is `.bee`-wide, so a CONCURRENT session's in-flight tracked
  bee-store dirt rides into this feature's bookkeeping commit under this
  feature's message — misattributed history, never data loss (review
  close-lands-bookkeeping-20260810 P3-6). This closes the dead-end where a worktree-session close dirtied
  main and `worktree merge` then refused `WORKTREE_MERGE_MAIN_DIRTY` on
  bee's own bookkeeping (backlog P2 row 708; close-lands-bookkeeping cell
  clb-1, 2026-08-10).
- R82 — `bee close` carries a PATTERN-CHECK door: critical patterns whose
  areas overlap the feature's touched areas are listed, and each wants a
  verdict via `--pattern-verdicts=<pattern-id>:<violated|respected|
  not-applicable>[,…]`. An unanswered pattern reports `pending` and the door
  stays NON-blocking; a `violated` verdict is the one blocking answer — the
  violation is the work. Verdicts land on the close record (knowledge-usable
  U7, cell ku-7, 2026-08-10).
- R83 — Close's promote proposal CONVERGES on the capture queue: alongside
  writing `docs/history/<feature>/promote-proposals.md` (R80), close
  enqueues one capture stub pointing at that proposal, so the flush loop —
  not a separate reminder channel — is what carries the review-then-merge
  obligation forward (knowledge-usable U4, cell ku-4, 2026-08-10).

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
- **Recording a knowledge sync checks only the phase, never whether any spec
  actually changed.** R20a refuses the recording step unless the feature
  stands in a phase where execution happened — but once that phase check
  passes, the recording step stamps the sync regardless of whether a single
  spec line changed alongside it. The spec-debt door (R21a/R78) therefore
  measures a timestamp, not the knowledge: any sync stamp taken after work
  completes clears the door, synced content or not. Found live: ten features
  were stamped as synced in one day while exactly one concept file had
  actually changed. Checking that the recorded areas correspond to a real
  diff is a separate, unshipped change.

- **The high-risk execution approval names its cause but cannot be satisfied
  from the command line.** The refusal is now honest — it names every failed
  advisor condition and the exact consult flow (B9a, made true in code by
  gate-door-refusal, 2026-08-04) — but the verb it points at for recording a
  consult is declared and not built, and the freshness precondition it describes
  is itself unported, so the arm refuses every high-risk execution approval
  unconditionally rather than checking the recorded reference. The approval
  therefore comes from the human or waits on that port. The repair was weighed
  and declined on purpose: porting the precondition and building the recording
  verb is itself high-risk work, so it deadlocks against the door it would
  repair (decision 20969403). What shipped instead was the honesty, never the
  unblock.

## Pointers (implementation)

- Record: `.bee/state.json` (CLI-owned). Verbs: `bee state`
  (`start-feature` — new; set/gate/worker/scribing-run — existing);
  `startFeature()` + `isKnownPhase` in `packages/bee/lib/state.mjs`
  (byte-mirrored to `.bee/bin/lib/state.mjs`).
- Phase-owned routing: generic `state set --owner <pre-phase>` in
  `the bee binary` and `.bee/bin/bee`; required-owner
  metadata in both command registries; phase-aware callers in exploring,
  planning, and compounding (there is no `validating` phase — validation-diet
  D3 retired it; the merged gate's execution component is carried by
  `planning`). Review stays local to its review
  record. Proof: state/CLI suites, `.bee/cells/codex-hook-state-parity-1.json`,
  and `docs/history/codex-hook-state-parity/reports/codex-hook-state-parity-1.md`.
- Tests: 15 start-feature rows in `packages/bee/tests/test_lib.mjs`.
- Evidence: commit `928abf1`; trace `.bee/cells/codex-parity-5.json`.
- Workflow-record creation on start (D1) and plan-rev-scoped gates (D7,
  widened by validation-diet D15): see `workflow-records-and-projections.md`
  Pointers — `seedLegacyWorkflows`, `createWorkflow`,
  `workflowGatesToApprovedGates`, and the `plan-rev bump` verb. Evidence:
  traces `.bee/cells/multisession-native-{6,9}.json`, commits f4fe163,
  2dd834f.
- Merged approval (validation-diet D2/D14/D15): `bee state gate --merge
  --approved true` flips `approved_gates.shape` and `approved_gates.execution`
  together in one call (mutually exclusive with `--name`, refused together);
  `requireFreshAdvisorForHighRisk` is shared by the standalone `--name
  execution` path and the `--merge` path (D14); `--merge` stamps
  `approved_for_plan_rev` on both fields via `findGateStamp` (D15); a plain
  `--name` approval is byte-for-byte unchanged. `handleStateGate` in
  `the bee binary` + `.bee/bin/bee`. Evidence: trace
  `.bee/cells/vd-3.json`; `test_cli_state.mjs`, `test_state_projection.mjs`,
  `test_bee_cli.mjs` all green.
- Terminal-phase parity (B2a/R79): `scripts/tests/test_terminal_phase_parity.mjs`
  — scans `packages/bee/lib` and `.bee/bin/lib` for top-level
  `const <NAME> = new Set([...])` under `TERMINAL_PHASES` / `NO_WORK_PHASES` /
  `TERMINAL_LANE_PHASES`, and compares all 12 discovered declarations (6
  canonical + their 6 mirrored twins) against `KNOWN_PHASES` in
  `packages/bee/lib/state.mjs`. The six canonical sites: `lib/guards.mjs:151`,
  `lib/compaction.mjs:81`, `lib/scratch.mjs:62` (`TERMINAL_PHASES`);
  `lib/inject.mjs:235`, `lib/intent.mjs:49` (`NO_WORK_PHASES`);
  `lib/recovery.mjs:40` (`TERMINAL_LANE_PHASES`). `guards.mjs` is the copy
  governing write-denial. Evidence: trace `.bee/cells/dch-6.json`.
- Close-door threshold + ledger fallback: `scribingDebt`,
  `bestScribingStampMs`, `readScribingLedger` in `packages/bee/lib/cells.mjs`
  (mirrored `.bee/bin/lib/cells.mjs`). Evidence: trace
  `.bee/cells/sss-1.json`, decision `5b2f963d`.
- Native terminal-state door (R21a, both escapes): the compounding-complete
  write in `packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs`
  reuses the same debt counter and deferral-decision reader the separate
  close driver (`drivers::close`) uses, so the two can never disagree about
  what counts as debt or what already-logged deferral clears it. Evidence:
  trace `.bee/cells/tpp-1.json`, commit `7f0381a5`; discovery decision
  `e6f7dfcb` (2026-08-03), fix decision `c9b5d916` (2026-08-04).
- Soft promote door (R80, knowledge-loop D2/D9): `build_promotion` called
  in-process from `bee close`'s green path in
  `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs`, ahead of
  `archive_feature_for_close`'s cell retirement; `write_text_atomic` (new,
  beside `write_json_atomic`/`append_jsonl`) in `fsutil.rs` writes
  `docs/history/<slug>/promote-proposals.md`. Evidence: trace
  `.bee/cells/kl-3.json`, commit `384587a1`; trace `.bee/cells/kl-5.json`,
  commit `c8d25dff`.
- Approvals-map shape coercion (B53/R104): `spread_gates` in
  `packages/bee-rs/crates/bee/src/state.rs:100-121` — one match arm for an
  object, one wildcard arm returning `default_gates()`; re-exported for
  `state_group`, and `hooks/compaction.rs`'s masking fallback deleted with the
  old error branch. Evidence: trace `.bee/cells/jp-4.json` (999 passed, 0 failed,
  2026-08-04); locked by js-parity-cleanup D2.
