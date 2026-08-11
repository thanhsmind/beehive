---
type: bee.area
title: "Workflow State — completion teeth, the judge verdict, and the archive transaction"
description: "What a unit must prove before it may be completed — proof scaled to change-class × lane, a diff_stats-backed test-shape guard, and the older hard door for red-first branches — the structured append-only judge verdict with its honest independence stamp and the reopen it can force on an already-capped unit, why archiving a cell is a journaled transaction serialized against every other mutator, and why a unit that changed files does not complete until a commit claims it by name."
timestamp: 2026-08-06
bee:
  id: workflow-state-cells-completion-judge-and-archive
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: ["self-correcting-loop D3/D4/D5 with Validating amendments Δ5-Δ6 (behavior-class completion teeth, judge-verdict schema, risk-scaled goal-check judge)", "gh-issue-fixes-172 D-GHF-C (the judge cap-guard: a needs-revision verdict blocks completion absent an audited override)", "565e68d0-327f-404e-b49e-d1c61ba81bfd (unchanged: the goal-check judge is never the user-invoked independent review)", "test-economy D1/D2/D3/D8 (proof-tier by change_class × lane replaces the flat behavior-vs-advisory split; diff_stats-backed test-shape guard at cap; the D8 negative-control floor — amending self-correcting-loop D3's completion door and narrowing decision 0009 / e54878b1 / 8ef2bae6 to the red-first branches only)", "derived-check-hardening E1/E9 (the cap door cross-checks the impact registry and warns, never refuses; the residual ships open and named)", "derived-check-hardening E6 (capCell resolves behavior_change from the top-level field or trace.behavior_change, forward-only)", worker-conformance D1/D10/D12/D14 (per-unit authored evidence stops being a completion precondition — exactly two doors become non-blocking recorded warnings; absence of proof is stamped as a distinct inert marker that arms only the feature-boundary door), "worker-conformance D11 (the feature-level trailing test-coverage door inherits the same treatment: an asserted pass with nothing recorded does not discharge it) + wc-2c (a withdrawn test unit owes no coverage and stands in for none)", worker-conformance D2 as corrected (the highest-risk lane raises the behaviour-bearing classes only — refactor/formatting stay suite-green and a coverage-authoring unit stays targeted-green even there), "worker-proof (the registered-execution-worker completion door for small-lane-or-larger units, escaped by a recorded inline reason; cell wp-1, commit 57738faa, 2026-08-04)", "hook-teeth D6/D7 (docs/history/hook-teeth/CONTEXT.md, 2026-08-04 — one commit per unit becomes a completion door: the unit's own commit trailer is verified over the feature checkout's recent history)"]
  sources: ["self-correcting-loop cells scl-1..scl-5 (traces in .bee/cells/, reports docs/history/self-correcting-loop/reports/, 2026-07-19)", hardening-1-7-10 cells 1710-1..1710-11 (2026-07-21 — journaled crash-recoverable cell archive; needs-revision reopen clears verify evidence), "test-economy cells te-1/te-2 (proof-tier matrix + diff_stats handler; test-shape guard — new_suite_reason + ratio ceiling; docs/history/test-economy/CONTEXT.md, traces in .bee/cells/, 2026-07-25)", "docs/specs/workflow-state.md#B30", "docs/specs/workflow-state.md#B31", "docs/specs/workflow-state.md#B32", "docs/specs/workflow-state.md#B34", "docs/specs/workflow-state.md#B35", "docs/specs/workflow-state.md#B36", "docs/specs/workflow-state.md#R47", "docs/specs/workflow-state.md#R48", "docs/specs/workflow-state.md#R49", "docs/specs/workflow-state.md#R50", "docs/specs/workflow-state.md#R53", "docs/specs/workflow-state.md#R54", "docs/specs/workflow-state.md#E25", "derived-check-hardening cells dch-1/dch-2/dch-8 (cap-door impact-registry warning, behavior_change resolution, lazy registry import surviving a vendored-lib fixture; traces .bee/cells/dch-{1,2,8}.json, reports docs/history/derived-check-hardening/reports/, 2026-07-29)", "worker-conformance cells wc-1/wc-2/wc-2c/wc-3/wc-4 (absence-of-proof marker stamped after the refusal chain; both debt doors armed on it with the freshness clock over the union; withdrawn test unit skipped before the count; two doors loosened to recorded warnings; per-door bypass rows — traces .bee/cells/wc-*.json, reports docs/history/worker-conformance/reports/, CONTEXT docs/history/worker-conformance/CONTEXT.md, feature verify green 117 suites 2026-07-29)", "worker-proof cell wp-1 (registered-execution-worker cap door + inline-reason escape; trace .bee/cells/wp-1.json, commit 57738faa, 2026-08-04)", "hook-teeth cell bh-6 (commit-trailer verification at completion over the feature branch history, --commit-pending escape on the trace, empty file list exempt; trace .bee/cells/bh-6.json, commit 08e95a4e, 2026-08-04 — cells 89 passed, full suite 1058 passed 0 failed)"]
  authoritative_for: "workflow-state: unit completion teeth, judge verdicts, and the cell archive transaction"
---

# Workflow State — completion teeth, the judge verdict, and the archive transaction

> **Superseded in part (2026-07-31 — decision 412e9b3a,
> docs/specs/test-simple.md).** The proof-economy tier system this concept
> describes — the proof-tier matrix (B30/B37/B38, R55–R57), the red-first
> evidence doors, the deferred-proof/absence-of-proof markers and the
> feature-boundary debt doors they arm (B41–B43, R89–R93), and the
> test-volume brakes — is deleted wholesale. The current completion door is
> one declared test path: `bee cells finish` runs `commands.test` through
> the deterministic `bee test` runner and writes
> `.bee/logs/test-results.json`; green caps, red refuses with the failing
> excerpt, an undeclared suite caps `tests: undeclared`, and `bee close`
> re-runs the full declared suite for the feature. The judge-verdict rules
> (B31/B32, R48–R50) and the archive transaction (B34–B36, R53/R54) remain
> current. Everything below about proof tiers is kept intact as the
> historical record of the superseded system.

Finishing a unit is a door, not a declaration. Three guards stand at it: proof
that a behavior change fixed something real, a structured verdict from a judge
whose independence is stamped honestly rather than assumed, and — for the unit
that turns out not to have been finished after all — a reopen that takes its
stale evidence away with it. The archive transaction sits here too, because it
is the other way a unit's record moves without a mutator having asked it to.

## Behaviors & Operations

**B30 — Completing a unit requires proof scaled to how risky its change class
and lane are, not one hard door for behavior change and advisory everywhere
else.** Trigger: completing a unit whose change is classified — explicitly,
or derived: an unclassified unit with `behavior_change: true` still derives to
`behavior`, and an unclassified `behavior_change: false` stays advisory-only,
exactly as before. What happens: the required proof is looked up from a fixed
matrix over change-class × lane (the proof-tier): `security`/`migration`
demand red-first proof in every lane; `bugfix` demands red-first only in the
`high-risk` lane and a single targeted-green test everywhere else;
`behavior`/`api` likewise demand red-first only in the `high-risk` lane and,
everywhere else, prove on the EXISTING targeted suite rather than on newly
authored rows; a unit whose whole mandate IS test coverage proves on its own
targeted suite green, in every lane including the highest-risk one, because it
has no prior production behavior to characterize a "before" for;
`refactor`/`formatting` demand only that the existing suite
still passes green, and are refused outright the moment their diff adds any
new test file at all — no evidence field can buy that door open, because a
refactor that needs a new suite was misclassified. Wherever the tier resolves
to red-first, the original hard door still applies unchanged: the recorded
proof-of-red evidence must exist, be long enough to be a real account rather
than a placeholder, and must not be identical to another unit's recorded
proof; falling short refuses completion, naming the missing minimum or the
colliding unit — the duplicate check tolerates an unreadable sibling record by
skipping it rather than failing the whole scan. A unit riding the existing
deliberate-exceptions door keeps that door's contract unchanged, with an
advisory noting it took that door instead. What each actor observes: a
behavior-changing unit in a low-risk lane no longer automatically pays the
full red-first cost — the same rigor now lands exactly on `security`/
`migration` in every lane and on the behaviour-bearing classes inside the
`high-risk` lane, never loosened for those. **The highest-risk lane does not
sweep every class into red-first**, and any statement that it does is wrong:
`refactor`/`formatting` stay at existing-suite-green there, and a
coverage-authoring unit stays at its own targeted green there — a unit with no
new behaviour to characterize cannot produce a real "before", so demanding one
would only create pressure to misclassify (worker-conformance, correcting the
"all classes" reading of that feature's D2)
(self-correcting-loop D3, Δ5; amended by test-economy D1/D2: decision 0009's
blanket behavior-change hard door and the self-correcting-loop D3 red-evidence
floor at e54878b1/8ef2bae6 keep their original shape but now apply only inside
this matrix's red-first branches, never outside them).

**B37 — The proof-tier's diff-shaped checks run on one `diff_stats` snapshot
computed per cap attempt, and skip themselves rather than block when git is
unavailable.** Trigger: a unit is being capped and its resolved proof tier
depends on what the diff actually changed (new test files, added-lines-to-
changed-lines ratio). What happens: the capping handler computes `diff_stats`
once — untracked new files from `git status --porcelain`, line deltas from
`git diff --numstat` — over the cell's declared changed files, deduping the
five template-mirror copies of any shared-library file down to one counted
instance before any check sees them, so editing one canonical template never
silently multiplies its own line counts across every mirror. That snapshot is
handed to the capping check as a plain value; nothing downstream recomputes
it. What each actor observes: when git errors or is unavailable, `diff_stats`
arrives as `undefined` and every diff-shaped check is skipped outright — fail
open, the same posture as this repo's other git-dependent guards — with a
warning recorded to the hooks log rather than a silent pass or a blocked cap
(test-economy D1).

**B38 — A new test file in the diff must justify itself, and a runaway
test-to-source ratio is capped by lane.** Trigger: capping a unit whose diff
(per B37's snapshot) adds one or more new `test_*.mjs` files, or whose
added-test-lines-to-changed-source-lines ratio is unusually high. What
happens: any new test file at all is an unconditional refusal for
`refactor`/`formatting` units (per B30) — no evidence overrides it. For every
other change class, a new test file requires a `new_suite_reason` of at least
20 characters recorded in the cap evidence; missing or short, the cap is
refused naming the requirement. Separately, the ratio of added test lines to
changed source lines is checked against a lane-scaled ceiling: `tiny`/`small`
lanes only warn above a ratio of 3; `standard`/`high-risk` lanes refuse above
a ratio of 4 unless the evidence carries a `ratio_waiver` of at least 20
characters justifying it — invoking the waiver is itself an audited event,
never a quiet exemption. What each actor observes: the two checks are
independent (a justified new suite can still trip the ratio ceiling, and vice
versa), and both trace back to the one diff snapshot from B37 rather than
re-deriving their own counts (test-economy D3).

**B39 — Completing a unit names every suite the dependency map ties to the
files it touched but its own check command leaves out.** Trigger: a unit is
completed and its record lists the files it changed. What happens: each of
those files is looked up in the project's own dependency map — the derived
record of which verification suites a given file can affect — and every
directly affected suite the map returns that the unit's recorded check command
does not mention is named back to the author on the same warning channel the
other completion advisories use. The door still opens: this is a loud warning,
never a refusal, and the unit completes either way. The lookup is resolved only
at the moment it is needed rather than pulled in when the completion path is
first loaded, and it sits inside the guard that already tolerates a map that
cannot be read — so a workspace where the map is absent, unreadable,
malformed, or where the component that derives it is not present at all
completes in silence rather than erroring. What each actor observes: an author
who scoped a check command too narrowly is told exactly which suites the map
says they left out, and can widen the command or accept the gap knowingly; a
workspace carrying no map behaves exactly as it did before the cross-check
existed. Why it reports rather than blocks: this door stands on the path of
every future unit, so making completion depend on the map being current would
let one stale or wrong edge block all work (derived-check-hardening E1/E9).

**B40 — The behavior-change flag is resolved from either place a unit may
record it.** Trigger: a unit is completed. What happens: the flag is taken from
the unit's explicitly set top-level value when one is present, and otherwise
from the value recorded inside the unit's own trace; an explicit top-level
setting always wins, and a unit that declares the flag in neither place is left
unset exactly as before. The resolution happens once, where the completion
reads the record, so every downstream consumer sees the same answer. The
correction is forward-only — units already completed are not revisited, because
the tooling refuses to rewrite a completed record. What each actor observes: a
unit authored the common way, declaring its change inside its own trace rather
than at the top level, now completes carrying the flag it actually declared,
and therefore falls inside both the spec-debt obligation (R21a/R22) and the
semantic goal-check judge's scope (B32) instead of silently escaping both
(derived-check-hardening E6).

**B41 — Completing a unit no longer asks a worker to AUTHOR evidence; exactly
two doors record the absence instead of refusing.** Trigger: a unit is
completed through the ordinary path (not the path that deliberately relocates
its proof to the feature boundary). What happens: two doors that used to refuse
now let the completion through and record a warning on the unit and on the
operator's channel — the door that demanded written evidence from a unit
declaring it changed behaviour, and the door that refused a small-or-larger
lane asserting its check passed with nothing recorded. Every other refusal at
this door survives untouched, including the one in the same block that demands
the unit list the files it actually touched: a file list states what was
touched, it is not authored proof, so it cannot drift into invention. Separately
from either warning, a completion that recorded NEITHER real check output NOR
supplied evidence is stamped with a distinct **absence-of-proof marker**. The
marker is computed only after the entire refusal chain has already run, on a
completion that is already going to succeed, so it can never decide whether a
completion is refused — only describe one that was not. What each actor
observes: a worker is never again asked to produce prose in order to pass a
gate; the absence of proof stops being either enforced per unit or silently
forgiven, and becomes a fact on the record (worker-conformance D1/D10/D14).

**B42 — The absence-of-proof marker is a new, inert field, and its only power
is arming the feature-boundary door.** Trigger: the marker is stamped by B41.
What happens: nothing, anywhere at the unit door — no refusal reads it, no
exemption keys on it, no brake is lifted or tightened by it. Its single
consumer is the feature-boundary door (R82), which it arms exactly as the
relocated-proof marker does. It is deliberately NOT that older marker: the
relocated-proof flag short-circuits six separate refusal sites at the unit
door, so routing an unproven completion onto it would have voided the red-first
tier and the test-volume brakes the instant the marker landed. A second,
powerless field was the whole design point. What each actor observes: a
completion carrying the marker looks and behaves exactly like any other
completed unit until the feature tries to close, at which point one real green
run at the feature boundary is owed and no bypass level lifts it
(worker-conformance D12).

**B43 — A feature owes trailing test coverage, and withdrawn work discharges
none of it.** Trigger: a feature attempts to leave execution or run its
knowledge sync. What happens: the door reads the feature's units and reports
one of two debts — *missing*, when the feature holds completed behaviour-or-
interface units that touched code and no test unit at all; or *not-green*, when
a test unit exists but is not completed, was completed on a recorded failing
check, or was completed carrying the absence-of-proof marker. That third case
is the one the evidence diet creates: the older judgement that an assertion is
not evidence outlives the per-unit door that used to enforce it, because an
asserted pass cannot be the coverage this door is holding out for. A
**withdrawn** test unit is neither an obligation nor a discharge — it is
skipped before the count of test units is taken, so a feature that drops its
only test unit falls through to *missing* rather than passing clean.
Withdrawing the work is never cheaper than doing it. Only withdrawal is exempt:
a test unit still open, claimed, or blocked is undischarged work somebody owes
and keeps refusing. Behaviour-or-interface units whose entire recorded file set
is instruction or knowledge text owe nothing here, and a missing or empty file
set counts conservatively as code so an unrecorded diff can never launder real
behaviour past the debt. What each actor observes: the trailing coverage
judgement is owed by the FEATURE, not by each slice's units individually
(worker-conformance D11; wc-2c, found live on this feature's own close-door).

**B44 — Completing a unit in the small lane or larger is refused unless the
worker its capping record names is registered for that unit, though an
explicit inline reason escapes the refusal.** Trigger: a unit whose lane is
`small` or heavier is being completed — the `tiny` lane never reaches this
door; it may run inline by contract, unchanged. What happens: the door reads
the worker named on the capping record and checks it against that unit's own
registered-worker record; when the two do not match, completion is refused,
naming the unit, the unmatched worker, and the two ways forward — register
that worker against the unit before capping, or supply an explicit inline
reason on this same call. The inline reason is never a quiet override: it is
recorded on the unit's own trace as the escape actually taken, so a later
reader always sees why the check was bypassed rather than finding a silent
gap in the record. What each actor observes: a `small`-or-larger unit can no
longer complete by naming a worker nobody registered for it and leaving no
trace of why; running the work inline stays permitted, but it is never silent
again (worker-proof, cell wp-1). The dispatch-preparation path no longer
under-registers: `dispatch prepare --claim`, on a successful claim, registers
the claimed-for worker through the same write path `state worker add` uses
(nickname, cell, tier from the cell's own field, status running), so this
door finds the worker without any manual remedy; a failed or absent claim
registers nothing, and a registration failure after a standing claim is loud
in the payload (`worker_registered: false` + `registration_error`), never
silent (dispatch-registers-worker cell dpr-1, 2026-08-10, closing the gap
first hit on knowledge-search ks-2 and filed as friction rows 670/707).
The payload keys are pinned through the REAL dispatch-prepare entry (an
out-of-process test child, since the entry resolves its root off the
process working directory); the registration-failure shape stays pinned on
the inner claim seam — the entry has no cleanly corrupt-able failure seam
of its own, a recorded split (review-p2-hardening cell rph-4, 2026-08-11).

**B31 — A judge verdict is a structured, append-only record with an honest
independence stamp.** Trigger: a judge examines a unit of work and renders a
verdict. What happens: the verdict is validated against one fixed shape before
it is accepted — free-form prose is not a verdict and is treated as a failed
judge run, re-dispatched once and then recorded as unverified rather than
accepted as free text. A valid verdict is appended to the unit's own record,
stamped with the builder's and judge's models as supplied by the dispatching
orchestrator at record time, and an independence status: `confirmed` only when
both models were pinned and genuinely differ, `same-model` when they match,
`unverified` otherwise — never a guess. What each actor observes: the verdict
history on a unit only ever grows; a malformed verdict never corrupts the
record, it simply fails validation with a named reason (self-correcting-loop
D5, Δ6).

**B32 — The goal-check's semantic judge scales with the lane's risk, and stays
inside the loop that finishes a unit, never the review gate.** Trigger: the
swarming goal-check evaluates a completed unit of work. What happens: tiny and
small lanes run mechanical checks only, unchanged; standard lanes dispatch one
judge per completed behavior-changing unit; high-risk lanes dispatch the same
judge with a preference for model independence from the builder, recording the
outcome honestly either way (B31). A needs-revision verdict whose finding
looks automatically fixable means the unit is not yet done — it is
re-dispatched with the exact failing checks, and the attempt history (B26)
gains a failed entry carrying the judge's failure signature; a needs-revision
verdict that needs a person escalates to the human instead of looping. What
each actor observes: this judge never creates, approves, or substitutes for
the user-invoked review session (R4/R11); the review gate, the
review-candidates ledger, and the "review runs only on request" rule are all
untouched — a unit can be judged clean here and still show up as `unreviewed`
until someone asks for a review (self-correcting-loop D4, Δ6; decision
565e68d0 unchanged).

**B34 — Archiving a cell is a guarded transaction, not a plain file move.**
Trigger: a cell is archived (moved out of the live cell tree) or unarchived
(brought back). What happens: before anything moves, a preflight check
refuses the operation outright if a cell of the same identity already exists
at the destination — a collision is caught before the first write, never
discovered after. Once preflight clears, a journal recording the intended
move is written before the first move happens, so a crash mid-archive leaves
enough evidence for the very next archive or unarchive call to detect the
interrupted transaction and roll it back cleanly rather than leaving the cell
split between two locations. The summary write that records the archive
outcome sits inside this same guarded section, not after it, so a summary is
never recorded for a move that did not actually complete. Unarchiving refuses
to overwrite an existing active cell — the same collision discipline as
archiving, checked in the other direction. What each actor observes: an
archive or unarchive either fully happens or leaves a recoverable trail; it
never leaves a cell silently duplicated, silently missing, or silently
overwritten (hardening-1-7-10).

**B35 — Archiving is serialized against every other cell mutator at the one
place all of them write.** Trigger: a cell archive transaction runs at the
same time as any ordinary mutation (claim, update, verify, cap, reopen, and
the rest) targeting the same or a related cell. What happens: every mutator
funnels through the same single write path (`writeCell`), and an archive
transaction takes a brief synchronous acquire of the archive lock at that
funnel before it proceeds — so a mutator and an archive transaction can never
interleave their writes. A cell that exists only in the archive tree can
never be resurrected by an ordinary write: any mutator attempting to touch it
receives the typed `CELL_ARCHIVED` refusal instead of silently recreating a
live record out of an archived one. Conversely, while an archive transaction
is actually in flight, an ordinary write against the affected cell fails fast
with the typed `CELLS_ARCHIVE_BUSY` refusal rather than blocking indefinitely
or racing the transaction. Every mutator — including reopen, tier changes,
budget reset, and judge-verdict recording — is subject to both refusals
equally; none of them has a side door around an archived or mid-transaction
cell. What each actor observes: an archived cell stays archived until
deliberately unarchived, and a cell mid-archive is never silently mutated out
from under the transaction (hardening-1-7-10).

**B36 — A needs-revision verdict on an already-capped unit reopens it, and
re-capping demands fresh proof, never stale evidence.** Trigger: a judge
verdict of needs-revision (B31) is recorded against a unit that has already
been capped — a later or asynchronous judge pass catching what an earlier
pass missed, for example. What happens: the cell is reopened from capped back
to open, its recorded verify evidence is cleared (a passing verify that the
judge has just contradicted cannot be allowed to keep satisfying the capping
requirement), and the claims store is reconciled to match the reopened state.
The judge ledger and the reopen event itself are both preserved — append-only,
exactly like every other judge verdict and audit trail — so the history of
what was found and when is never lost even though the cell's own live status
just changed. Re-capping the unit afterward is structurally impossible on the
old evidence alone: the cap door requires a fresh verify run AND a subsequent
passing verdict on that fresh run before it will accept completion again.
Stale evidence — the verify result that was true before the needs-revision
finding — can never be replayed to re-cap the unit. What each actor observes:
a unit that looked done can be pulled back into open work by a later verdict
without losing its history, and the only way back to capped is doing the work
over and proving it again (hardening-1-7-10).

**B49 — A unit that changed files does not complete until a commit claims it by
name (hook-teeth D6, 2026-08-04).** Trigger: completing a unit whose record
lists changed files. What happens: the recent history of the feature's own
working checkout is scanned for a commit whose message carries a line naming
that unit and nothing else — a mention inside prose does not satisfy it. The
feature's granted worktree is preferred over the caller's own checkout, because
completion is normally run from the integration checkout while the work landed
on the feature's branch; without a granted worktree, the caller's own history is
read. Missing the line refuses the completion, and the refusal offers one
escape: a declared commit-pending reason, stored on the unit's own trace. What
each actor observes: one commit per unit stops being an honour system, while a
unit that recorded no changed files is exempt — there is nothing for a commit to
carry. The door belongs to completion alone: capping a unit short of finishing
it never runs the scan. It fires after the test-green door, so a red run is
still the first thing anyone hears about.

## Business Rules

- R47 — *(Superseded twice, kept for lineage — read R55 and R89 for the live
  rule.)* A behavior-changing unit's completion requires substantial (not
  placeholder), non-duplicated proof-of-red evidence; every other change
  classification stays advisory-only in this version (self-correcting-loop D3).
  R55 narrowed the demand to the red-first branches of the proof-tier matrix;
  R89 then made the remaining blanket "declared behaviour change with no
  written evidence" door a recorded warning rather than a refusal.
- R48 — A judge verdict is accepted only in its one structured shape;
  free-form prose is a failed judge run, re-dispatched once, then recorded
  unverified — never accepted as the verdict itself (self-correcting-loop D5).
- R49 — A verdict's model-independence status is derived, never asserted:
  `confirmed` requires both models pinned and different; anything else is
  recorded honestly as `same-model` or `unverified` (self-correcting-loop D5,
  Δ6).
- R50 — The goal-check's semantic judge is verification inside the loop that
  finishes a unit, scaled by lane risk — it is never the user-invoked
  independent review, never opens or approves a review session, and never
  touches the review-candidates ledger or Gate 3 (self-correcting-loop D4;
  decision 565e68d0-327f-404e-b49e-d1c61ba81bfd unchanged).
- R53 — Archiving or unarchiving a cell is refused before any move when the
  destination already holds a cell of the same identity, is journaled before
  the first move so a crash mid-transaction can be rolled back at the next
  archive/unarchive call, and is serialized against every other mutator at the
  single write funnel: a mutator against an archived cell is refused typed
  `CELL_ARCHIVED`, and a mutator against a cell mid-transaction is refused
  typed `CELLS_ARCHIVE_BUSY` (hardening-1-7-10).
- R54 — A needs-revision judge verdict recorded against an already-capped unit
  reopens it with its verify evidence cleared; re-capping requires a fresh
  verify run followed by a subsequent passing verdict, so stale evidence can
  never satisfy the completion door a second time (hardening-1-7-10).
- R55 — The proof a unit must show at completion is
  `requiredProofTier(change_class, lane)`: `red-first` for `security`/
  `migration` in every lane and for `bugfix`/`behavior`/`api` in the
  `high-risk` lane; `targeted-green` for `bugfix` outside `high-risk` and for
  a coverage-authoring `test` unit in EVERY lane; existing-targeted-green for
  `behavior`/`api` outside `high-risk`; `suite-green` (existing suite passes,
  new test files refused outright) for `refactor`/`formatting` in every lane.
  The `high-risk` lane raises the behaviour-bearing classes ONLY — it never
  sweeps `refactor`, `formatting`, or a coverage-authoring unit into
  red-first, and any statement that it covers "all classes" overstates the
  door (worker-conformance, correcting that feature's own D2 wording). An
  unclassified unit derives
  `behavior` when `behavior_change: true` and stays advisory-only when
  `false` — a lighter tier is never available without declaring the class
  (test-economy D1, amending self-correcting-loop D3 and narrowing decision
  0009 / e54878b1 / 8ef2bae6 to the red-first branches only).
- R56 — Diff-shaped cap checks run against one `diff_stats` snapshot
  (untracked new files via `git status --porcelain`, line deltas via `git
  diff --numstat`, five-mirror duplicates deduped to one) computed once per
  cap attempt and passed in, never recomputed inside the check; a git
  failure yields `diff_stats: undefined` and every diff-shaped check fails
  open with a logged warning rather than blocking the cap (test-economy D1).
- R57 — A new test file in the capped diff is refused outright for
  `refactor`/`formatting`, and elsewhere requires a `new_suite_reason` of
  ≥20 characters in cap evidence; the added-test-to-changed-source line
  ratio warns above 3 for `tiny`/`small` and refuses above 4 for
  `standard`/`high-risk` unless a `ratio_waiver` of ≥20 characters is
  recorded — an audited exemption, not a silent one (test-economy D3).
- R58 — At completion, the unit's recorded check command is cross-checked
  against the dependency map's directly affected suites for the files it
  touched, and every omission is named — but the omission never refuses the
  completion. A map that is absent, unreadable, or malformed is a silent skip,
  never an error, and the lookup is resolved lazily so a workspace without the
  deriving component still completes normally (derived-check-hardening E1/E9).
- R59 — The behavior-change flag at completion resolves from the explicit
  top-level value when it is set and otherwise from the value recorded in the
  unit's trace; an already-completed unit is never retroactively corrected,
  because the correction is forward-only (derived-check-hardening E6).
- R89 — **A worker is never asked to AUTHOR evidence in order to complete a
  unit.** Exactly two doors became non-blocking recorded warnings: a
  behaviour-changing unit that supplied no written evidence, and a
  small-or-larger lane asserting its check passed with nothing recorded.
  Every other refusal at that door survives, including the demand — in the
  same block as one of the two — that the unit list the files it touched: a
  file list reports what was touched and is not authored proof. Proof was
  relocated to the feature boundary, never removed (worker-conformance D1).
- R90 — **Absence of proof is recorded, not ignored.** A completion that
  carried neither real check output nor supplied evidence is stamped with a
  distinct absence-of-proof marker, computed after the whole refusal chain has
  run. The marker arms the feature-boundary door (R82) and buys nothing else:
  it is read by no refusal, lifts no brake, and grants no exemption. It is
  deliberately not the older relocated-proof flag, which short-circuits six
  refusal sites — reusing that flag would have voided the red-first tier and
  the test-volume brakes the moment an unproven completion carried it
  (worker-conformance D10/D12).
- R91 — **Genuine evidence with empty output is never marked, and a workspace
  that declares it runs no verification is exempt.** The marker means NEITHER
  channel carried proof, so a unit holding real supplied evidence — a
  low-lane security unit whose proof-of-red already passed the red-first door
  is the canonical case — stays unmarked even with no recorded output, because
  it holds the strongest proof in the system. A workspace whose declared
  verification command is the explicit "none" sentinel is exempt outright:
  a feature-level verification can never run there, so marking would arm a
  door that workspace could never satisfy. The exemption keys on the
  verification declaration alone and is deliberately narrower than the
  no-test-workspace test — a workspace that declares only its impacted-test
  command as "none" can still run a real feature verification and must keep
  arming the door (worker-conformance D14).
- R92 — **Several surviving refusals are DEFERRED by the default completion
  path, never waived.** The path that relocates proof to the feature boundary
  is the default for dispatched workers, and four doors do not fire on it: the
  demand for a recorded passing check, the new-suite justification, the volume
  ceiling's refusal (which becomes a recorded warning there, because its
  waiver channel is the per-unit evidence that path refuses by construction),
  and the red-first proof-of-red. Calling them unconditional misleads every
  worker taking the default; calling them waived misstates what the feature
  boundary still owes. Deferred is the exact word, and the distinction is
  load-bearing (worker-conformance, wc-5/wc-7).
- R93 — **A withdrawn test unit owes no coverage and stands in for none.** The
  feature-level coverage door skips a withdrawn test unit BEFORE counting test
  units, so a feature that drops its only test unit falls through to the
  *missing* debt rather than passing clean. Only withdrawal is exempt — open,
  claimed, and blocked test units are undischarged work and keep refusing. The
  order is the whole guard against withdrawal becoming an escape hatch
  (worker-conformance, wc-2c).
- R94 — Completing a unit in the `small` lane or larger is refused when the
  worker its capping record names is not registered for that unit; an
  explicit inline reason recorded on the unit's own trace is the sanctioned
  escape, and the `tiny` lane is exempt by contract (worker-proof, wp-1).

- R100 — Completing a unit with recorded file changes requires a commit in the
  feature checkout's recent history whose message carries a line naming that
  unit; a missing line refuses the completion, a declared commit-pending reason
  escapes it onto the unit's trace, and a unit with no recorded file changes is
  exempt (hook-teeth D6, cell bh-6, 2026-08-04).

## Edge Cases Settled

- A behavior-changing unit riding the existing deliberate-exceptions door for
  its proof-of-red keeps that door's original contract untouched, with an
  advisory noting it took that door instead of meeting the length/duplicate
  floor (self-correcting-loop D3, Δ5).
- Every guard this feature loosens (the proof-tier matrix, the red-first
  scope narrowing, the impacted cap in verify-pipeline) ships its loosening
  in the same unit as a table-driven test proving both directions: the case
  now let through, and a case at the heavier tier that is still refused
  (missing red evidence on `security`, missing red evidence in `high-risk`,
  a new test file on `refactor`, a level-1 suite still running when the
  transitive cap trims it) — a widened threshold that stopped catching its
  own heavy case is a regression, not a relaxation (test-economy D8).
- A unit whose declared change is recorded only inside its own trace is the
  common authoring shape, not an exception. Reading the top-level field alone
  meant such units completed with the flag false while their own records
  plainly said otherwise — escaping the spec-debt obligation and the semantic
  judge together, and reporting nothing, because both consumers were handed a
  value that looked correctly resolved (derived-check-hardening E6).
- A unit holding real supplied evidence but no recorded check output is NOT
  marked absent-of-proof. The predicate was first written against recorded
  output alone, which would have armed the feature-boundary door for exactly
  the units carrying the strongest proof in the system; an adviser consult
  caught it before it shipped (worker-conformance D14).
- A feature that drops its only test unit reaches the *missing* coverage debt,
  not a clean close. Before the fix the door read one withdrawn unit two
  contradictory ways at once — counting it as a test unit that exists
  (suppressing *missing*) while also listing it as an offender for not being
  completed. Found live on this feature's own close-door
  (worker-conformance, wc-2c).
- A `tiny` unit never reaches the registered-worker check (B44) — it is
  permitted to run inline by contract, exactly as before this door existed
  (worker-proof, wp-1).

## Open Gaps

- **The dependency cross-check reports and never blocks.** A unit whose check
  command omits suites the map names still completes, and nothing obliges the
  author to act on the warning — so the coverage the warning describes stays
  advisory. The stronger form, refusing completion until the command covers
  every directly affected suite, was considered and deliberately declined:
  this door stands on the path of every unit, and a refusal here would make
  all work depend on the map being fresh. It ships open and named, never
  recorded as a closed finding (derived-check-hardening E1/E9).
- **Units completed before the absence-of-proof marker existed carry none, so
  the coverage door cannot see them.** A legacy test unit that asserted a pass
  with nothing recorded stays invisible to the *not-green* debt. Backfilling
  the marker onto historical records was out of reach from the door's own
  layer and was not attempted. Named, not closed (worker-conformance, wc-2).
- **The behaviour-change warning and the marker key on different things, so a
  behaviour-changing unit can complete warned but unmarked.** The warning fires
  on missing supplied evidence alone; the marker requires that neither output
  nor evidence was recorded. A behaviour-changing unit that recorded real check
  output therefore warns without arming the feature-boundary door. This is the
  intended reading of R91, but the asymmetry is real and worth stating rather
  than discovering (worker-conformance, wc-4).
- **A workspace declaring only its impacted-test command as "none" takes an
  automatic evidence waiver and completes unmarked**, even though it could run
  a real feature-level verification. That waiver is the older declared-no-test
  semantics, deliberately untouched by this feature; the two exemptions
  (waiver vs marker) read the two declarations differently on purpose, and the
  seam ships named rather than reconciled (worker-conformance, wc-1).

## Pointers (implementation)

- Dependency cross-check at the cap door (B39/R58): `capCell` in
  `packages/bee/lib/cells.mjs` (byte-mirrored to `.bee/bin/lib/cells.mjs`),
  around the `queryRegistry(registry, cellFiles, { level: 1 })` call. The map
  is `scripts/impact-registry.json`; `queryRegistry` is resolved by
  `await import('../../../scripts/impact_registry.mjs')` **inside** the
  existing guarded `try/catch` at the point of use — never a top-level import,
  so a fixture vendoring only `.bee/bin/lib/` with no sibling `scripts/` tree
  hits the same catch as a missing or malformed map and skips silently.
  `capCell`'s `withStoreLock` callback is `async` to carry that await. The
  warning follows the `ratioWarning` shape in the same module. Evidence:
  traces `.bee/cells/dch-1.json`, `.bee/cells/dch-8.json`.
- Behavior-change resolution (B40/R59): `resolveDeclaredBehaviorChange` at
  `packages/bee/lib/cells.mjs:1821`, consulted at the cap read
  (`cells.mjs:1869`) only when the top-level field is `undefined`. Evidence:
  trace `.bee/cells/dch-2.json`.
- Tests: `packages/bee/tests/test_cells.mjs` (the registry-warning rows and the
  two behavior-change resolution rows), `packages/bee/tests/test_cli_cells.mjs`.
- The two loosened doors (B41/R89): `capCell` in `packages/bee/lib/cells.mjs`
  (byte-mirrored to `.bee/bin/lib/cells.mjs`) — the `behaviorEvidenceWarning`
  branch (`cells.mjs:1969-1975`) and the `recordedProofWarning` branch
  (`:2186-2190`); the surviving non-empty `files_changed` refusal sits
  immediately below the second at `:2191-2195`. Throw count inside `capCell`
  went 18 → 16.
- The absence-of-proof marker (B42/R90/R91): `proofUnrecorded` at
  `packages/bee/lib/cells.mjs:2228-2237`, written as `trace.proof:
  "unrecorded"` at `:2273`; the workspace exemption is `isNoTestCommand` from
  the config reader applied to `commands.test`. Both doors read
  it in `packages/bee/lib/state.mjs`: `featureVerifyDebt` (`:2502-2560`) and
  `testCellDebt` (`:2570-2640`).
- The feature coverage door (B43/R93): `testCellDebt` in
  `packages/bee/lib/state.mjs` — the withdrawn-unit skip sits before the
  `testCellCount += 1` increment; the debt doors table is `DEBT_DOORS` in the
  same module. Evidence: traces `.bee/cells/wc-1.json`, `.bee/cells/wc-2.json`,
  `.bee/cells/wc-2c.json`, `.bee/cells/wc-4.json`; reports
  `docs/history/worker-conformance/reports/`.
- Tests for all of the above: `packages/bee/tests/test_cells.mjs` (marker
  table rows) and `packages/bee/tests/test_bee_cli.mjs` (end-to-end door rows
  driving real add/claim/verify/cap without hand-writing the marker, plus the
  per-door bypass rows).
- The registered-worker completion door (B44/R94): `registered_worker_for_cell`
  and its call site in `cap_cell_from_flags`,
  `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` — checks the
  capping record's `trace.worker` against `state.json`'s `workers[]` entries
  matching this cell's id (the shape `bee state worker add --nickname <n>
  --cell <id> --tier <t> --status <s>` writes). The escape is `--inline-reason
  "<why>"`, recorded as `trace.inline_reason`. Tests:
  `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs`. Evidence: trace
  `.bee/cells/wp-1.json`, commit `57738faa`.
- Commit-trailer completion door (B49/R100): `cell_commit_trailer` (the exact
  trimmed line `cell: <id>`), `commit_trailer_history_root`
  (`find_granted_worktree_for_feature`, else the caller's root) and
  `commit_trailer_present` (`git log -n 50 --format=%B%x00` through the crate's
  own `run_git`, never a shell) in
  `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:212-261`; fired
  from `handlers_close.rs:219-237` only when `finish` is true and
  `files_changed` is non-empty, after the test-green door and before the
  per-cell lock. The escape is `--commit-pending "<reason>"`, recorded as
  `trace.commit_pending` (`handlers_close.rs:363-368`). Evidence: trace
  `.bee/cells/bh-6.json`, commit 08e95a4e (cells 89 passed; full suite 1058
  passed, 0 failed, 2026-08-04).
