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

Extend, never invent: a fourth mailbox kind `advisor-nudge` in the existing
closed set (per 3cfd9980, c80debd7 — the record, delivery, pending, and
frequency machinery come free); its unanswered rows count as a per-feature
debt cleared by a decision-log entry tagged `advisor-nudge` naming the row
(the named-escape pattern every debt door already uses — covers both
"consulted, outcome X" and "declined because Y", per 9e5eda5b's argue-yes
rule); one new debt arm in the close doors plus the merge-door check,
copying dissent-debt one for one; the flag of a7e6f237 derives
deterministically (waiting-on kind gate/question, mailbox kind
escalation/urgent/advisor-nudge → yes) and sorts yes-first in the letter and
WakeReport renderers. Rejected: a separate nudge store (a second registry —
the drift shape 6f039742's family refuses); a hard mid-turn interrupt
(c80debd7 forbids it); supervisor-side summoning (704b691c forbids it).
Risk map: supervisor.rs kind-set change MEDIUM (closed-set contract tests
pin it — red-first); door arm LOW (eighth copy of a proven arm); renderers
LOW.

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| S1 | `advisor-nudge` mailbox kind: record, validation, frequency cap counts it, pending + turn-boundary delivery render it | the record is the debt's substrate | supervisor records a nudge; target session sees it at its next turn start | S2 |
| S2 | response debt: per-feature unanswered-nudge counter + close-door arm + merge-door check + the decision-tag escape | the teeth (9e5eda5b) | `bee close` refuses with remedy while a nudge is unanswered; a tagged decision clears it | S3 |
| S3 | needs-human-decision flag derived + yes-first sort in letters and WakeReport; supervisor prompt doc gains the nudge signal wording | the reading surface (a7e6f237) | a letter lists yes-flagged items first | ship |

## Test matrix

- Happy: record `advisor-nudge` → appears in `pending` → delivered line at
  turn boundary; tagged decision clears the debt; close goes green after.
- Edge: second nudge on the same (target, point_key) refuses and names
  escalation (cap parity with intervention); a nudge whose target session
  holds no claim derives no feature and counts against nothing (423871d7 —
  records alone, no guessing); flag derivation on each mailbox kind.
- Error: unknown kind still refused (closed set); close/cap refusal prints
  the three-line headline/remedy/next form with the new prefix; a clearing
  decision missing the row id does not clear.

## Out of scope

- The waggledance supervisor, cockpit repo, weekly report (other repo).
- The spec-drop procedure and hat wave (slp-spec-drop-procedure, docs-lane).
- Any change to who summons the advisor or to consult machinery itself.
