---
artifact_contract: bee-plan/v1
mode: standard
---

# Plan: SLP Advisor Nudge

Mode: `standard` — 1 risk flag: existing-test-behavior (door snapshot and
contract tests enumerate the close doors and mailbox kinds; both sets grow).
Why this is the least workflow that protects the work: story-sized behavior
across the supervisor store, two doors, and two renderers — but every piece
copies a shipped pattern, so one plan and five cells cover it without an epic
map.

## Requirements (from CONTEXT.md)

- 3cfd9980: supervisor RECOMMENDS an advisor consult on poor-work evidence;
  the target lead summons the advisor itself; supervisor writes records only.
- 9e5eda5b: the recommendation is a RESPONSE DEBT — consult ran or reasoned
  decline recorded; cap/close refuse while unanswered; ignored twice
  escalates via the existing frequency machinery, never repeats.
- a7e6f237: needs-human-decision flag on every queued ask and report line;
  yes sorts first in letters and WakeReports.
- 704b691c (constraint): supervisor decides nothing, acts on nothing.
- c80debd7 (shipped): interventions are mailbox records delivered at the next
  turn boundary; frequency-cap state rides the record.
- 423871d7 (constraint): cold ticks; the debt derives from records alone.

## Discovery

Inspected the landed stores and doors in this worktree: the supervisor store
is one module (`verbs/supervisor.rs`, 5283 lines) holding the closed mailbox
kind set `["intervention","escalation","urgent"]` (MAILBOX_KINDS, :236), the
frequency cap over `CAPPED_KINDS` (:243), `pending`/`mark-delivered`, and the
WakeReport window; turn-boundary delivery runs through
`pending_delivery_for_session` (hooks/prompt_context.rs:277-290,349). The
debt-door precedent is dissent-debt: close arm at
`verbs/drivers/close.rs:1556,2134` (prefix const :47) and the merge-door
check at `verbs/worktree/phases.rs:197`. Evidence: `rg -n` runs recorded in
this plan's authoring session.

## Approach

Extend, never invent. Naming note: `advisor-nudge` is the record-kind
spelling of CONTEXT's "recommend-advisor" concept (naming discretion,
CONTEXT "Agent's Discretion") — one record, two names in prose.

- A fourth mailbox kind `advisor-nudge` in the existing closed set (per
  3cfd9980, c80debd7). "Existing machinery" means widening three fixed
  arrays and their guard test — `MAILBOX_KINDS` (supervisor.rs:236),
  `CAPPED_KINDS` (:243), `ALL_KINDS` (:247), `all_kinds_is_the_two_sets`
  (:245-246) — after which record/pending/mark-delivered/delivery render the
  kind with no fork. The signal vocabulary also grows: `KNOWN_SIGNALS`
  (:250) gains `budget-overrun` and `same-region-resubmit` so 3cfd9980's
  watch list (locked upstream by a8f4b8ab, da7cb49b) is expressible.
- Delivery must reach worktree sessions or the feature is hollow: the
  turn-boundary hook bails `Delegate` for any linked worktree
  (prompt_context.rs:144-147) with the gap named at :275-276. S1 lifts ONLY
  the supervisor pending-delivery read above that bail — the rest of the
  hook's worktree behavior is untouched.
- Feature derivation (CONTEXT deferred item): at RECORD time the nudge row
  derives `feature` from target_session's live claim (claim → cell →
  feature) and stores it on the row — derived once, records-only per
  423871d7. No claim ⇒ no feature ⇒ the row counts against nothing.
- The debt: unanswered `advisor-nudge` rows for a feature count at BOTH the
  cap path (`run_cap`, verbs/cells/util.rs:72 — the cell-level tooth) and
  the close door, plus the merge-door check (phases.rs:197 shape). Door
  exists at EVERY lane, mirroring dissent-debt (close.rs:2125-2130), not
  judge-debt's standard-up gating — the nudge only exists where a
  supervisor runs, so lane-gating would double-filter.
- Clearing DIVERGES from the dissent-deferral precedent deliberately: that
  escape matches tag+feature (dissent.rs:536-552) and would let one decision
  clear every row. Here a decision tagged `advisor-nudge` clears one row by
  carrying the row id in its text (`DecisionFilters.text` matching — new,
  small, stated). Covers both "consulted, outcome X" and "declined because
  Y" per 9e5eda5b.
- The a7e6f237 flag derives deterministically (waiting-on kind
  gate/question, mailbox kind escalation/urgent/advisor-nudge → yes) and
  sorts yes-first in the letter renderer (verbs/mailbox.rs:176-187 region)
  and the WakeReport assembly (supervisor.rs report window).

Rejected: a separate nudge store (a second registry — the drift shape
6f039742's family refuses); a hard mid-turn interrupt (c80debd7 forbids
it); supervisor-side summoning (704b691c forbids it); feature-level
clearing (contradicts per-row teeth, above).
Risk map: supervisor.rs closed sets MEDIUM (contract tests pin them —
red-first); worktree delivery lift MEDIUM (hook ordering — scoped read
only); cap/close/merge arms LOW (proven pattern); renderers LOW.

## Shape

Five cells map onto three phases: S1 = an-1, S2 = an-2 + an-3, S3 = an-4 +
an-5.

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| S1 (an-1) | `advisor-nudge` mailbox kind (three arrays + guard test), two new KNOWN_SIGNALS, record-time feature derivation onto the row, and the worktree delivery lift in prompt_context.rs | the record is the debt's substrate, and delivery to worktree sessions is what makes it real | supervisor records a nudge targeting a WORKTREE session; that session sees the delivery line at its next turn start | S2 |
| S2 (an-2, an-3) | an-2: per-feature unanswered-nudge debt counter + the per-row decision-tag escape; an-3: cap-path arm (`run_cap`), close-door arm, merge-door check — every lane | the teeth (9e5eda5b) | `bee cells cap` and `bee close` refuse with the three-line remedy while a nudge is unanswered; a tagged decision naming the row clears exactly that row | S3 |
| S3 (an-4, an-5) | an-4: needs-human-decision flag derived + yes-first sort in verbs/mailbox.rs letters and the supervisor.rs WakeReport; an-5: skills/bee-herding/references/supervisor-prompt.md gains the advisor-nudge signal wording | the reading surface (a7e6f237) | a letter lists yes-flagged items first, in derivation order | ship |

## Test matrix

Per phase, triad each:

- S1 happy: record `advisor-nudge` → `pending` lists it → delivery line
  renders at the turn boundary of a WORKTREE session (the lifted read).
  S1 edge: second nudge on the same (target, point_key) refuses and names
  escalation (cap parity); a nudge whose target session holds no claim
  derives no feature (423871d7 — records alone). S1 error: unknown kind and
  unknown signal still refused (closed sets, red-first).
- S2 happy: a tagged decision naming the row id clears exactly that row;
  cap and close go green after. S2 edge: two unanswered rows, one cleared —
  the other still refuses; feature-less rows count against nothing.
  S2 error: cap, close, and merge refusals each print the three-line
  headline/remedy/next form with the new prefix; a clearing decision
  missing the row id clears nothing.
- S3 happy: a letter and a WakeReport list yes-flagged items first, stable
  order within each group. S3 edge: flag derivation per mailbox kind and
  per waiting-on kind. S3 error: a malformed queue row derives flag=no and
  the line still renders (render never panics on data).

## Out of scope

- The waggledance supervisor, cockpit repo, weekly report (other repo).
- The spec-drop procedure and hat wave (slp-spec-drop-procedure, docs-lane).
- Any change to who summons the advisor or to consult machinery itself.
