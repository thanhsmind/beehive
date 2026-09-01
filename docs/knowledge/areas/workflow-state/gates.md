---
type: bee.area
title: "Workflow State — starting a feature, the phase vocabulary, phase-owned routing, and closing"
description: "The guarded doors of a feature's life: the all-or-nothing start that can never inherit the previous feature's approvals (now also creating that feature's own workflow record), the closed phase vocabulary, the adviser consult high-risk execution approval demands (the one gate scoped to a plan revision), the phase-owned generic routing mutation, the four-step tail that makes declaring a feature closed impossible, and the soft promote door riding the green path once that tail clears."
timestamp: 2026-08-16
bee:
  id: workflow-state-gates
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: ["chain-integrity D1-REVISED/D2/D3/D4 (the tail of the chain: learning capture is produced not asserted, the sync demands executed work, the close demands zero spec debt, the waiver is audited)", "AO3/AO13 (execution-gate adviser precondition — folded from the old standalone execution gate into Gate 2, validation-diet D2/D14 — event-based staleness, never a TTL — cells ao-4-1/ao-4-2 2026-07-17)", codex-hook-state-parity D4-D6 (pre-phase routing ownership and review isolation), "scribing-integrity D1-D3/D6 (the wall at every door — feature-swap guard, lane-aware close, durable scribing ledger + orphan sweep, pre-ledger amnesty; cells si-1/si-3, 2026-07-24)", multisession-native D1/D7 (starting a feature also creates its own workflow record via three separate lock transactions; the execution gate records approved_for_plan_rev — docs/history/multisession-native/CONTEXT.md), "scribing-stamp-seam 5b2f963d (sss-1 — the close-door threshold folds in the durable ledger as fallback/max, not the record stamp alone)", "validation-diet D2/D14/D15 (the merged approval flips `shape` and `execution` together via `bee state gate --merge`, inherits the high-risk advisor-consult precondition D14 previously guarding execution alone, and stamps `approved_for_plan_rev` on both fields together so a later bump can never leave the merged approval half-revoked — docs/history/validation-diet/CONTEXT.md, cell vd-3, 2026-07-28)", "compounding-gate D1 (the close also demands recorded learning-capture evidence, fresh against the knowledge-sync stamp, waivable only with a logged decision; cells cg-1/cg-2, 2026-07-27)", derived-check-hardening E5 (the six hand-copied terminal-phase memberships are pinned by a parity suite rather than refactored to derive from the phase enum), "knowledge-loop D2/D9 (a soft promote door on the green path, after the tests and scribing-debt doors, computed before auto_archive_on_close retires the closing feature's cells; cells kl-3/kl-5, 2026-08-05)", "cc4b381a (advisor-gate-port Gate 2: port the consult anchors, the event-based staleness rule, the record/show verbs, and the high-risk execution precondition to the single remaining runtime — cells agp-1/agp-2, 2026-08-05)", "ae0a96ec (feature-swap-door Gate 2: the swap wall runs natively off the shared debt counter and gains the close door's capture-deferral escape; the waiver names the abandoned feature — cell fsd-1, 2026-08-05)", "2b35d98c (debt-door-archive Gate 2: every scribing-debt counter reads the archive as well as the active store, active copy wins on a duplicate id, and one parity test pins all four counters — cells dda-1/dda-2, 2026-08-06)", "20969403 (gate-door-refusal: the high-risk execution refusal states its own cause instead of rendering as an argument-shape complaint; the truthful refusal unblocks nobody and the real unblock deadlocks against the door it repairs — cell gdr-1, 2026-08-04)", "js-parity-cleanup D2 (docs/history/js-parity-cleanup/CONTEXT.md, 2026-08-04 — a stored approvals map merges over the defaults only when it is an object; every other shape takes the defaults whole, and the position-keyed spread inherited from a foreign language's accident is gone)", "counter-teeth D1/D6 (docs/history/counter-teeth/CONTEXT.md, 2026-08-04 — the close refuses uncaptured behavior-changing units unless a logged capture-deferral decision names the feature; the refusal names the units and both remedies, and the counter is proven before the flip)", "traceable-runs D2 (docs/history/traceable-runs/CONTEXT.md, 2026-08-14 — gate_bypass decides whether a run halts, never whether the approval record exists; an auto-approval must carry its bypass level and reason, tying R25's ladder to a per-gate audit trail owned by workflow-records-and-projections.md)", "merge-closes-the-lane D2 (f220f461, 2026-08-18 — bee close is the only command entitled to write a terminal lane phase; a green, non-dry-run close sets the lane to idle, never compounding-complete, which stays gated on a fresh recorded compounding run; --dry-run, a door-blocked close, and an already-terminal lane all write nothing, and a failed write warns without reddening the close; commit 939771ec)"]
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
record exists; no worker is registered; no file hold stands in this start's own
way (scoped — see R86, never "any hold anywhere"); and the
prior feature has no nonterminal cell. An intentionally abandoned cell must
first be dropped through the explicit drop verb, which records the reason —
the start operation never clears work as a side effect. When the preconditions
hold, one atomic write sets the feature, its mode, a valid phase, resets all
gate fields (all five, since uat joined the vocabulary) to ungranted, and updates the summary/next-action. Observers (the
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

**The terminal-state door runs natively, on both the default record's and a
lane record's route alike (terminal-phase-port, cell tpp-1).** It has two
loud halves: the learning-capture freshness evidence described below, and the
spec-debt threshold R78 defines.

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

**A green, non-dry-run close also closes the feature's own lane
(merge-closes-the-lane D2, f220f461).** `bee close` is the only command
entitled to write a terminal lane phase at all: on a green, non-dry-run
close, the lane is set to `idle` — never `compounding-complete`, which stays
gated on a fresh recorded compounding run (R21a) rather than manufactured by
close finishing. Four guards keep the write narrow: `--dry-run` writes
nothing; a close blocked at any hard door above writes nothing; a lane
already sitting in a terminal phase (`idle` or `compounding-complete`) is
left untouched rather than rewritten; and a failed lane write warns on its
own line without turning the close red.

**The wall stands at every door a feature can leave through — the front close
door, an abandoning swap, and a per-feature lane close alike (scribing-integrity
D1-D3; feature-swap-door, cell fsd-1).** Swapping the routing record to a new
feature refuses on the OUTGOING feature's standing debt exactly like close does
— same exhaustive naming, same audited waiver, naming the ABANDONED feature
(the waiver logs after the write succeeds, since by then the routing record
already holds the new feature). A per-feature lane close checks the LANE
feature's own debt against that lane record's own sync stamp. Every sync stamp
is also appended to a durable ledger (`.bee/logs/scribing-runs.jsonl`) — a
repair verb may stamp a feature that is not the active one, so an orphan left
by a dead session can be paid later. A global sweep over every completed
behavior-changing unit versus its feature's best stamp (ledger, lane record, or
the default record's own attributed stamp — attribution by the stamp's OWN
feature field, never by which feature happens to be active) surfaces orphaned
debt in the status payload and as one loud session-start line; every
pre-ledger feature carries one audited backfill stamp (amnesty decision), so
the alarm starts at zero inherited debt.

**Debt is counted over the active cell store AND the closing feature's
archive, one unit per id with the active copy winning on a duplicate
(debt-door-archive, cells dda-1/dda-2).** Retiring a unit's records into the
archive on a green close never retires its debt: an archived unit counts
exactly when it completed after the feature's best sync stamp. Four places
compute this count — the close and swap walls, the status payload, the
session-start line, and the mid-session nudge — and one parity test pins them
to the same answer over a fixture that mixes an active unit, an archived one,
and one id in both places.

**The close door's own threshold is the LATER of the record's own sync-stamp
field and the durable ledger's newest entry for that feature, never the record
field alone (scribing-stamp-seam, decision 5b2f963d).** A workflow-record
rebuild spreads a fresh read of the record over the in-memory mutation, so an
in-flight stamp write can vanish before it is ever read back
(`workflow-records-and-projections.md`); trusting the ledger as a fallback —
the same source the orphan sweep already trusts — means a sync that reached
the ledger still clears the close door even when the record-field write did
not. A cell capped after the true sync still counts as debt, and a ledger
entry belonging to a different feature never clears this feature's own debt.

**A knowledge-freshness door blocks close on stale pointers inside the
feature's own touched areas and work bundle (knowledge-distill-trigger D1,
cell kdt-1).** After the tests, scribing-debt, judge-debt, and pattern-check
doors, close reuses `bee knowledge check`'s own findings — filtered to
`areas/<touched-area>/` (the same touched-file-to-area match `promote`'s own
area-update section already applies) plus `work/<feature>/` — and blocks when
a `dangling_source` or `dangling_required_context` warning falls inside that
scope; a feature never blocks on a pointer it never touched, so an in-flight
sibling feature's own stale pointers never tax this close. `not_canonical` and
`invalid_evidence_state` findings stay report-only in the door's detail;
prose contradictions (stale claims with no machine detector) are this door's
named, not silently dropped, limitation. The refusal names every stale
pointer and its remedy; the same escape the other hard doors use applies
here too — fix each pointer, or clear it with a logged
`knowledge-freshness-deferral` decision naming the feature.

**An impact door blocks close when a doc still cites one of the closing
feature's own decisions without having been reconciled (doc-impact-synthesis
D1, cell kds-2).** Close collects the feature's decide events by their
structured `feature` field (never a time window), sweeps `docs/**` for
citations of each id, and blocks on every surviving hit with file:line and
the fix-and-rerun remedy — the sweep re-runs fresh each close, so a fixed
doc clears itself. Excluded, never blocking: the generated
`docs/decisions/index.md`, the feature's own `docs/history/<feature>/`
records, and the write-guard's generated trees. Escape: a logged
`impact-deferral` decision naming the feature.

**A routing door blocks close when a locked D-ID in the closing feature's own
CONTEXT.md decision table has no area-spec citation and no feature-local
record (doc-impact-synthesis D2, cell kds-3).** The parser reads the
canonical `## Locked Decisions` pipe table only; a legacy-form CONTEXT
(bullet list, split sub-tables) degrades to a loud report-only notice
naming the historical campaign row — the door's teeth apply to every
post-door shaping, never retroactively. Routed means the bundle carries
`<slug> D<n>` (plain, range `D1-D3`, or slash `D1/D3` form) or the
decision's logged short8; a D-ID cited in more than one area is a
report-only duplication warning. Escape: `routing-deferral`.

**A doc-deferral door blocks close when deferral-shaped prose in the
feature's touched docs names no registered trigger (doc-impact-synthesis
D3, cell kds-3).** The scan set is the capped cells' changed doc files plus
the feature's own `docs/history/<feature>/` files — full text, bounded by
that set, never a repo scan; fenced code is exempt. A flagged line clears
with a same-line trigger citation (backtick trigger id or
`[[trigger:<id>]]`) resolving in the trigger registry. Escape:
`doc-deferral`.

**That door blocks only lines a BASELINE does not already carry
(doc-deferral-baseline D1/D6, cell ddb-1).** It had fired five times across
five features and every flagged line on every occasion was prose DESCRIBING
deferral machinery rather than deferring work — zero true positives — because
this repo's own domain is deferral queues and triggers, so `defer`, `later`
and `for now` are its nouns. The word list, the scan set, the fence exemption
and both escapes are unchanged; what changed is that a line already recorded
in the baseline no longer counts. Identity is the line's NORMALIZED CONTENT
per file, never its line number, so inserting text above a baselined line does
not resurrect it, and one normalization function serves both recording and
matching.

**The baseline is seeded once per repo, REPO-WIDE, by the door itself
(doc-deferral-baseline D6, superseding D2).** The first run finding no
baseline file walks every markdown file under `docs/` — not the door's own
per-feature scan set — records every deferral-shaped line in the whole tree,
writes the file, and passes; enforcement afterwards stays per-feature over the
unchanged scan set. Seeding from the scan set instead would freeze only the
docs one feature happened to touch, so the next feature touching a different
long-lived doc would enter enforcement against an empty entry and inherit
every pre-existing line in it — the false positives returning on a delay. The
seed ALWAYS writes, even when it flags nothing, because an absent file IS the
seed state: skipping the write leaves the next close reading a missing
baseline and ADOPTING the first genuine deferral line anyone adds. The file is
git-tracked so a clone or fresh worktree inherits it instead of re-seeding,
and `--dry-run` never writes — it reports the door non-blocking and names the
count it would baseline, spelling out a bounded sample rather than every
message, since the repo-wide set runs to four figures.

**A line the baseline does not cover has exactly two ways through, and
hand-editing the baseline is not one (doc-deferral-baseline D4, later widened
by D8).** Cite a registered trigger inline, or log a `doc-deferral`-tagged
decision naming the feature. D8 then admitted a third, built in the
doc-deferral-scope lane: a reasoned marker pair naming why a passage documents
deferral machinery, where an empty or missing reason exempts nothing. The
baseline forgives the PAST automatically and repo-wide; the marker is how NEW
prose states intent at the site, which a baseline cannot express — asked why a
line is forgiven, a baseline can only answer that it was already there.

The marker is an HTML comment pair — `<!-- bee:not-a-deferral: <reason> -->`
opening it and `<!-- /bee:not-a-deferral -->` closing it — and the opener needs
a non-empty reason: a reasonless opener exempts nothing, an unclosed one
exempts to end of file, and a marker and a code fence nest independently,
neither closing the other. The door's refusal names the marker as a third
remedy beside citing a registered trigger and logging a tagged decision, so a
writer who hits a genuine false positive learns the escape from the message
itself (doc-deferral-scope, cell dds-1).

**Once every hard door above (tests, scribing-debt, judge-debt, pattern-check,
knowledge-freshness) has cleared, close also runs `bee knowledge promote` for
the closing feature in process — a SOFT door (knowledge-loop D2/D9, cells
kl-3/kl-5).** It prints one headline naming its proposal counts and writes the
full proposal to `docs/history/<slug>/promote-proposals.md`; it never refuses
the close and never changes close's exit code — a `None` promote outcome (the
retired Node delegate arm) and a `Thrown` outcome (e.g. an `unknown_work`
refusal) both degrade to the same one warning line. `build_promotion` runs
BEFORE `archive_feature_for_close` retires the closing feature's cells, since
`build_promotion` mines `.bee/cells/*.json` and a door that ran after
retirement would scan an empty directory. Nothing this door writes lands under
`docs/knowledge/` — `promote` still only proposes (B5/D38 in
`context-and-promote.md`).

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
value is an object, its keys are laid over the defaults (five since uat joined), so a record that
names only one gate still answers for all of them; when it is anything else at all —
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

- R106 — The gate vocabulary is five names: context, shape, execution, review,
  and uat (uat-gate-before-merge D1, cells ug-1..3, 2026-08-17). The uat gate
  is the user's acceptance stop between execution-complete and the merge to
  main: `bee worktree merge` refuses a standard or high-risk feature whose uat
  gate is unapproved (typed, zero-mutation, the refusal naming its three
  exits — user approval, a one-merge skip flag, or the repo-wide config
  opt-out). No bypass level ever auto-approves it: recording it with the auto
  actor is refused outright, so the approval can only carry the user's own
  word. It becomes visible in the preamble only once the execution gate is
  approved.
- R136 — Plan-time conflict candidates are DERIVED, never guessed, and they
  live on the workflow record beside `plan_rev` as
  `conflict_review: {plan_rev, derived_at, candidates: [{id, kind, title,
  verdict, note}]}` (knowledge-one-home D5, cell koh-8, 2026-08-22).
  `bee state plan-conflicts derive` builds the candidate list from the
  feature's own open and capped cells — their titles plus the stems of every
  path in `files`, `affects_skills` and `affects_specs` — and selects two
  kinds against it: every ACTIVE decision the `decisions log` conflict-hint
  scorer picks, widened to every decision scoring two or more term hits, and
  every homed knowledge rule whose home or `applied_at` patterns intersect
  those same paths. The verb is lane-targeted and takes the same two locks in
  the same order `plan-rev bump` takes; resolution landing on the default
  (non-lane) record is refused, because `conflict_review` is as
  lane-scoped as `plan_rev` itself. The field is seeded ABSENT: a record that
  was never derived against carries no field at all, which is a different
  fact from a record that was derived and came back empty. Zero candidates is
  a valid derive, and it is the ONLY state in which "0 conflicts" is a true
  statement about a plan revision.
- R137 — Each derived candidate carries exactly one recorded verdict out of a
  closed three: `compatible`, `conflicts`, or `retires-prior`
  (knowledge-one-home D5, cell koh-8, 2026-08-22). `bee state plan-conflicts
  verdict --id <candidate> --verdict <value> [--note <text>]` sets one
  candidate and leaves every other candidate — and the derived list itself —
  as it stands; an id no candidate carries, or a value outside the three, is
  refused by name and writes nothing. Re-deriving REPLACES the whole list
  rather than patching it, which is how a re-derive (and therefore a
  `plan-rev bump` followed by one) clears every verdict already recorded. A
  recorded `conflicts` verdict does not refuse anything by itself — it is a
  contradiction taken on with eyes open, and the `--note` is where the prior
  id a `retires-prior` verdict replaces is named.
- R138 — The merged gate never opens on a lane whose plan-time conflict check
  does not stand behind it (knowledge-one-home D2/D5, cell koh-9,
  2026-08-22). An approval that includes the execution component — `bee gate
  --merge --approved true`, or `--name execution --approved true` — targeting
  a LANE is refused, before any lock or write, on exactly three causes:
  the lane's workflow record carries no `conflict_review` at all (the check
  has never run against this plan); the recorded review's `plan_rev` differs
  from the lane's current `plan_rev`; or any derived candidate still carries
  no verdict, in which case the refusal NAMES the unverdicted ids. Each
  refusal names the fix — `bee state plan-conflicts derive`, then `bee state
  plan-conflicts verdict` per candidate. The plan-rev cause is the reset:
  a `plan-rev bump` invalidates the review by itself, so there is no separate
  clear step, only a fresh derive. The precondition is the advisor
  precondition's twin in shape and placement — same pre-lock peek, same
  post-lock recompute against the locked read, same fail-closed reading (a
  workflow store that cannot be read is an error, never a pass) — and it is
  LANE-ONLY: `conflict_review` is as lane-scoped as `plan_rev`, so the
  default (non-lane) record's gate behaviour is unchanged, and so is an
  unapprove (`--approved false`), which never carries the check. A recorded
  `conflicts` verdict is the deliberate exception (R137): it does not refuse.
  The approval succeeds and names those candidates on a second output line
  and under `conflicts_acknowledged` on the JSON result — present only when
  non-empty — so the contradiction is approved with eyes open rather than
  found after approval.
- R139 — A shape or merged approval never opens over a plan.md whose
  load-bearing claims table is missing, malformed, or still guessing
  (existence-is-not-evidence D1/D2, cell eine-rust-claims-gate, 2026-08-30).
  The check runs under its OWN guard — `approved && (merge || name ==
  "shape")` — never on plain `--name execution` (the plan is frozen after
  shape approval, so a later refusal would be undischargeable) and never on
  an unapprove. It reads `advisor_plan_path(root, feature)` with the
  advisor precondition's M1 feature selection (a lane approval reads the
  lane's own feature); `ErrorKind::NotFound` is inapplicable — tiny/small
  lanes legitimately carry no plan.md — while any other read error refuses,
  fail-closed (portable fixture: plan.md as a directory). Refusal causes:
  heading absent, zero rows, a row missing label/anchor/evidence, a label
  outside {read, ran, guessed}, any `guessed` row, or a `read` row whose
  `path:line` anchor names a path absent under root. The refusal names the
  offending rows, the expected shape, and the remedy (upgrade the label
  with a real read/run, or move the claim to the plan's `## Open
  Questions`). Parser and rules live in
  `verbs/state_group/plan_claims.rs`; the wrapper sits beside the other
  two preconditions in `set_gate.rs`, runs AFTER them in the merged path,
  and takes a single pre-lock call site — a plan-file read has no
  peek/lock record race (named deviation from the twins' two-site shape).
- R140 — The plan-conflict term set drops terms that saturate the decision
  store, so the derived candidate list stays proportionate to the plan
  (plan-conflicts-scope D1/D2/D3/D4/D5, cells pcs-1 and pcs-2, 2026-09-01).
  R136's `>= 2 term hits` rule is a FIXED threshold meeting an UNBOUNDED term
  set: a four-cell plan produces ~31 terms, and 31 moderately common terms
  yield hundreds of meaningless two-hit coincidences. Measured on bee's own
  store — 2589 active decisions — the rule returned 694 candidates, each
  needing its own `plan-conflicts verdict` call before the merged gate (R138)
  would open. It scaled the wrong way: a bigger plan got more noise, not more
  precision. The fix lands in the TERM SET, never in the scorer, which is the
  seam `plan_conflicts.rs` already named for the older length-and-stopword
  filter: `count_term_hits` and `conflict_candidates` are untouched, so
  `decisions log`'s own hints are unchanged. A term whose document frequency
  exceeds `TERM_DF_MAX_PERCENT` (3) of the active store is dropped — 694
  candidates became 36 — and the cut applies only from
  `TERM_DF_MIN_DECISIONS` (200) rows up, because document frequency is
  meaningless at N=2 and a freshly onboarded host repo, like the small
  fixtures, must behave exactly as before. `MAX_DECISION_CANDIDATES` (50)
  is the rail above that for a store ten times this size; a list the cap
  truncates SAYS so in the verb's own output rather than truncating silently,
  and the ranking is applied only when the cap actually bites, so an under-cap
  list keeps its previous order. The stored `conflict_review` shape is
  unchanged. Proven end to end: the same lane that derived 753 candidates
  before the fix derived 15 after it.
- R104 — An approvals map merges over the gate defaults only when it is stored as
  an object; every other shape yields the defaults untouched, and no shape is
  read partially or refused (js-parity-cleanup D2, cell jp-4, 2026-08-04).
- R1 — A new feature can never inherit gate approvals: every gate field resets in
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
- R78a — The gate write path validates its inputs with hard allowlists: an
  unknown gate name is refused naming the offending value and the legal
  names (derived from the same constant the write path uses), and a
  non-boolean approval value is refused naming the value — on the plain and
  merged paths alike, before any lock, writing nothing. This shipped as
  production behavior before it was ever pinned; gate-input-validation cell
  giv-1 (2026-08-10) added the five regression pins after PBI p-94ecc5a2
  suspected the validation missing.
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
  (close-bookkeeping-p3 cell cbp-1) — and since review-p1-batch-fixes cell
  rpb-2 (2026-08-11) that claim is PROVEN by failing-signer stub tests on
  both this path and `worktree merge`'s own commit, which now carries the
  same `--no-gpg-sign` + stdin-null hardening (the defense had shipped
  unpinned and per-copy; review B-P1-2) — and since review-p2-hardening
  cell rph-1 (2026-08-11) both paths call ONE shared unsigned-commit
  helper, so removing the flag there reds both stub tests at once: the
  hardening lives on the mechanism, not on copies. Same cell made the
  config refusal honest: the message names the file actually carrying the
  offending value across the merged overlay (`.bee/config.local.json`
  checked first), and `null` reads as UNSET (defaults on) instead of
  refusing — null is bee's unset idiom. Named cost, chosen not accidental: the
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
- R84 — Every gate approval or rejection, on the bypass ladder (R25) or off
  it, is stamped on the gate entry itself: who (`actor`: `user`/`auto`),
  under what bypass level, and why — `bee state gate --actor auto` refuses
  outright unless both `--bypass-level` and `--reason` are given, so R25's
  loud level banner is never the ONLY trace of an auto-approval; the
  per-gate record (schema and every authoring path) is owned by
  `workflow-records-and-projections.md` (traceable-runs D2, trun-2,
  2026-08-14).
- R83 — Close's promote proposal CONVERGES on the capture queue: alongside
  writing `docs/history/<feature>/promote-proposals.md` (R80), close
  enqueues one capture stub pointing at that proposal, so the flush loop —
  not a separate reminder channel — is what carries the review-then-merge
  obligation forward (knowledge-usable U4, cell ku-4, 2026-08-10).
- R85 — A green, non-dry-run `bee close` writes its own lane's phase to
  `idle` — never `compounding-complete`, which stays gated on a fresh
  recorded compounding run. `--dry-run` writes nothing; a door-blocked close
  writes nothing; an already-terminal lane (`idle` or `compounding-complete`)
  is left untouched; a failed write warns without failing the close
  (merge-closes-the-lane D2, f220f461).
- R86 — The feature-start hold precondition is SCOPED, and its remedy is
  always one the caller may take alone. A start refuses on an active file
  hold in exactly two situations: the hold belongs to the starting session
  itself — its own leftover state, whose remedy names that caller's own
  holder — or the hold belongs to a different session AND covers a path this
  start explicitly declares as its own working scope, whose remedy is to wait
  for release or expiry, or to start over non-overlapping paths. A different
  session's hold over a path this start does not declare refuses nothing, no
  matter which working copy it was taken in. Both the lane road and the
  default road now state the rule the same way, and the default road accepts
  a declared working scope for exactly this reason. No refusal on this path
  may name a remedy that would strip another session's holds — a refusal
  whose only cure touches someone else's resources is a mis-scoped refusal
  (start-feature-reservation-scope D1, e62d1311).

## Edge Cases Settled

- **A feature's record of its own finished work trips the deferral door.** A
  deviation line saying a shape belongs to a later unit stays deferral-shaped
  after that later unit lands and caps, so the feature cannot close on the
  record of work it has already done. Registering a condition is wrong by
  construction here — a trigger for finished work can never fire — and the
  baseline forgives only lines that were already there. The fitting remedy is
  the reasoned marker pair around the passage, naming why the prose records a
  past routing decision rather than an open promise.
- A capped prior-feature cell never blocks a new start; an expired-by-TTL
  reservation never blocks a new start (only active ones do), and an active
  one blocks only within R86's scope.
- A start that declares no working scope of its own can still be refused by
  its own session's leftover holds, but never by another session's — with
  nothing declared there is no overlap to find.
- Refused starts are proven side-effect-free: the record is byte-identical
  after a refusal.
- **The candidate scorer reads a NORMALIZED term set, and without it "zero
  candidates" is unreachable** (R136, knowledge-one-home D5, koh-8). Before a
  plan's titles and paths are scored against the decision store, every term is
  lowercased, stripped of surrounding punctuation, dropped when shorter than
  four characters, and filtered against a 24-word stop list. Unnormalized, an
  ordinary cell title contributes its articles and prepositions, those clear the
  two-hit threshold against nearly every decision in the store, and the derive
  returns an unusable list every time. Since R136 makes "0 conflicts" a true
  statement only when the derive genuinely returned nothing, a list that can
  never be empty is not a check — it is noise wearing a verdict field. The
  scorers themselves are untouched; only what is handed to them changed.

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

- **Starting a feature on the default record closes other live workflow
  records, which can leave a lane whose conflict derive also refuses.** The
  default start path closes the live records it finds; a lane whose record was
  closed that way has nothing for `bee state plan-conflicts derive` to write to,
  so the derive refuses and the merged gate's precondition (R138) then refuses
  for the absent-review cause. The behaviour predates this door — the door only
  makes it visible, because a closed record used to cost nothing until a gate
  started reading one. Named rather than repaired: reworking what a default
  start does to sibling records is a change to the workflow store's own write
  path, not to the gate (knowledge-one-home D5, koh-9).

- **The plan-revision bump's transaction is COPIED into the lane path rather
  than shared with it.** The lane-targeted conflict verbs take the same two
  locks in the same order as `plan-rev bump`, but by carrying their own copy of
  that transaction rather than calling one shared helper. Both are correct
  today; nothing keeps them correct together, so a future change to one lock
  order silently leaves the other behind. Recorded as drift risk, not repaired —
  factoring the shared transaction out is a change to the bump path itself
  (koh-8).

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
- Lane-closing write (R85, merge-closes-the-lane D2): the tail of the green,
  non-dry-run path in `close_handler`
  (`packages/bee-rs/crates/bee/src/verbs/drivers/close.rs`) reuses
  `run_set_body` to write the lane phase to `idle`. Evidence: commit
  `939771ec`; full suite `cargo test --release --manifest-path
  packages/bee-rs/Cargo.toml` — 2033 passed, 0 failed.
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
- Plan-time conflict check (R136/R137, knowledge-one-home D5):
  `packages/bee-rs/crates/bee/src/verbs/state_group/plan_conflicts.rs` — the
  two handlers plus `derive_candidates` / `build_conflict_review` /
  `apply_conflict_verdict`, routed from `set_gate.rs`'s `try_native` table
  beside `plan-rev bump` and sharing its lock order verbatim. Nothing here is
  a new scorer: it calls `decisions::read`'s own `conflict_candidates` and
  `count_term_hits` for the decision half and
  `knowledge::ownership::load_ownership` + `matches_owned` for the rule half.
  The record field is documented beside `plan_rev` in
  `verbs/workflow_store/record.rs` (`base_workflow_defaults`), deliberately
  unseeded. Help text for both spellings lives in
  `src/generated/registry_payload.json` (hand-edited — no generator exists in
  this repo, decision `3358743e`), with `catalog.rs`'s `PINNED_FLAG_COUNT` at
  180 for the one new flag name `--verdict`. Proof: the six
  `plan-conflicts` rows in `verbs/state_group/tests.rs`, plus
  `tests/registry_contracts.rs` and `tests/registry_dispatch.rs`. Hygiene note:
  `plan_conflicts.rs` carries a file-wide `allow(unused_imports)` taken from the
  module it was split out of — it hides a real unused import from the compiler
  and is worth narrowing whenever the file is next touched (koh-8).
- The merged gate's conflict precondition (R138, knowledge-one-home D2/D5):
  `conflict_review_refusal` + `lane_conflict_review` /
  `unverdicted_candidate_ids` / `acknowledged_conflicts` in
  `packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs`, wired into
  `run_gate_body` at BOTH `high_risk_advisor_refusal` call sites (the
  pre-lock peek and the post-lock recompute). It reads the LIVE WORKFLOW
  RECORD, never the lane record the gate mutates, because the lane
  projection does not copy `conflict_review` down — which also takes the
  review and the `plan_rev` it is compared against from one read. A lane
  with no live workflow record is the same C1 shape
  `write_through_projection` and the durable gate stamp already take: there
  is no `plan_rev` to compare and no record a derive could have written to,
  so the precondition does not apply. Help text for both `gate` spellings is
  hand-edited in `src/generated/registry_payload.json`; koh-9 adds no flag
  name, so `catalog.rs`'s `PINNED_FLAG_COUNT` stays 180. Proof: the ten
  `koh9-*` rows in `verbs/state_group/set_gate.rs`'s own test module, beside
  the advisor-precondition cases they are modelled on.
- Approvals-map shape coercion (B53/R104): `spread_gates` in
  `packages/bee-rs/crates/bee/src/state.rs:100-121` — one match arm for an
  object, one wildcard arm returning `default_gates()`; re-exported for
  `state_group`, and `hooks/compaction.rs`'s masking fallback deleted with the
  old error branch. Evidence: trace `.bee/cells/jp-4.json` (999 passed, 0 failed,
  2026-08-04); locked by js-parity-cleanup D2.
