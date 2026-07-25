---
type: bee.area
title: "Workflow State — completion teeth, the judge verdict, and the archive transaction"
description: "What a unit must prove before it may be completed — proof scaled to change-class × lane, a diff_stats-backed test-shape guard, and the older hard door for red-first branches — the structured append-only judge verdict with its honest independence stamp and the reopen it can force on an already-capped unit, and why archiving a cell is a journaled transaction serialized against every other mutator."
timestamp: 2026-07-25
bee:
  id: workflow-state-cells-completion-judge-and-archive
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: ["self-correcting-loop D3/D4/D5 with Validating amendments Δ5-Δ6 (behavior-class completion teeth, judge-verdict schema, risk-scaled goal-check judge)", "gh-issue-fixes-172 D-GHF-C (the judge cap-guard: a needs-revision verdict blocks completion absent an audited override)", "565e68d0-327f-404e-b49e-d1c61ba81bfd (unchanged: the goal-check judge is never the user-invoked independent review)", "test-economy D1/D2/D3/D8 (proof-tier by change_class × lane replaces the flat behavior-vs-advisory split; diff_stats-backed test-shape guard at cap; the D8 negative-control floor — amending self-correcting-loop D3's completion door and narrowing decision 0009 / e54878b1 / 8ef2bae6 to the red-first branches only)"]
  sources: ["self-correcting-loop cells scl-1..scl-5 (traces in .bee/cells/, reports docs/history/self-correcting-loop/reports/, 2026-07-19)", hardening-1-7-10 cells 1710-1..1710-11 (2026-07-21 — journaled crash-recoverable cell archive; needs-revision reopen clears verify evidence), "test-economy cells te-1/te-2 (proof-tier matrix + diff_stats handler; test-shape guard — new_suite_reason + ratio ceiling; docs/history/test-economy/CONTEXT.md, traces in .bee/cells/, 2026-07-25)", "docs/specs/workflow-state.md#B30", "docs/specs/workflow-state.md#B31", "docs/specs/workflow-state.md#B32", "docs/specs/workflow-state.md#B34", "docs/specs/workflow-state.md#B35", "docs/specs/workflow-state.md#B36", "docs/specs/workflow-state.md#R47", "docs/specs/workflow-state.md#R48", "docs/specs/workflow-state.md#R49", "docs/specs/workflow-state.md#R50", "docs/specs/workflow-state.md#R53", "docs/specs/workflow-state.md#R54", "docs/specs/workflow-state.md#E25"]
  authoritative_for: "workflow-state: unit completion teeth, judge verdicts, and the cell archive transaction"
---

# Workflow State — completion teeth, the judge verdict, and the archive transaction

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
demand red-first proof in every lane; `bugfix`/`behavior`/`api` demand
red-first only in the `high-risk` lane and a single targeted-green test
everywhere else; `refactor`/`formatting` demand only that the existing suite
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
`migration` and on `high-risk`-lane work, never loosened for those
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

## Business Rules

- R47 — A behavior-changing unit's completion requires substantial (not
  placeholder), non-duplicated proof-of-red evidence; every other change
  classification stays advisory-only in this version (self-correcting-loop D3).
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
  touches the review-candidates ledger or Gate 4 (self-correcting-loop D4;
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
  `high-risk` lane; `targeted-green` for `bugfix`/`behavior`/`api` outside
  `high-risk`; `suite-green` (existing suite passes, new test files refused
  outright) for `refactor`/`formatting`; an unclassified unit derives
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
