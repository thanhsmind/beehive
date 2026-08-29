# SLP Advisor Nudge — Context

**Feature slug:** slp-advisor-nudge
**Date:** 2026-08-29
**Shaping session:** complete (Lock from the closed map docs/discovery/slp-human-up/MAP.md — no interview was owed)
**Scope:** Standard
**Domain types:** RUN | READ

## Feature Boundary

The supervisor gains a way to get a strong model's help to a struggling
session without deciding anything itself, and the human's reading surface
gains a sort key. Three pieces, one repo (bee): a `recommend-advisor`
intervention record the supervisor writes on poor-work evidence; a RESPONSE
DEBT enforced at the cap and close doors — the target lead either runs the
advisor consult or records a reasoned decline before the related work caps;
and a needs-human-decision flag on queued asks and report lines, yes-first.
It ends there: no supervisor authority change, no new consult machinery
(the advisor slot and worker Advisor line already exist), no waggledance-side
work.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. IDs are bee decision-log ids (search with
`bee decisions search`).

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| 3cfd9980 | The supervisor watches poor-work signals (struggling-loop, budget overrun, same-region resubmits) and, on supporting evidence, its intervention record RECOMMENDS an advisor consult for the struggling session; the struggling repo's own lead reads the record at its next turn boundary and summons the advisor itself; the supervisor still writes records only; the human sees it in reports and is asked only when the lead does not act and the signal repeats | the summons stays a lead power, so 704b691c (supervisor decides nothing) holds untouched |
| 9e5eda5b | The recommendation is a RESPONSE DEBT enforced the existing debt-door way: consult ran, or a reasoned decline recorded; a cap or close attempted while an unanswered recommendation targets that work REFUSES with the remedy printed; the same point ignored twice escalates into the human's report, never a repeat | copies the judge-debt/dissent-debt arm one for one; argue-yes, silence-no |
| a7e6f237 | Every queued ask and report line carries a needs-human-decision flag (yes/no); yes-flagged items sort first in letters, WakeReports, and the weekly report | |
| 704b691c | (constraint, supersedes 83baf03f) The supervisor observes, connects, and packages questions; it decides NOTHING and acts on nothing | any shape giving the supervisor an action here is wrong by construction |
| c80debd7 | (shipped, slp-supervisor-heartbeat) Interventions are file records in a mailbox, read at the target's NEXT turn boundary, never mid-turn; the record carries frequency-cap state — same point twice = escalate, never repeat | the recommend-advisor record is one more KIND of this existing record, and the ignored-twice escalation of 9e5eda5b rides the existing frequency-cap machinery |
| 423871d7 | (constraint) The supervisor is a cold tick; durable records are its only memory | the debt must be derivable from records alone — no in-process state |

### Agent's Discretion

Everything the decisions leave open is planning's choice: the record-kind
field name and schema, how a recommendation resolves to "the related work"
(cell vs feature targeting), the decline record's shape (a decision-log
entry tagged like the other debt escapes is the named precedent), which
door arms carry the debt below `standard` lane, and where the flag rides on
the letter/report renderers. Constraint: reuse the existing machinery — the
intervention mailbox and its frequency cap, the debt-door arm pattern, the
decision log — before inventing any subsystem.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Recommend-advisor record | An intervention mailbox record whose kind recommends an advisor consult for one target session, carrying the evidence |
| Response debt | The obligation the record creates: consult ran or reasoned decline recorded, before the related work caps/closes |
| Reasoned decline | A recorded refusal naming why the consult is not needed — the argue-yes path; silence has no path |
| Needs-human-decision flag | A yes/no bit on an ask or report line; yes sorts first |

## Specific Ideas And References

- The debt-door arm to copy sits in the close driver: lane-gated existence,
  a debt count, a named escape decision, a `command` remedy, and the
  three-line refusal emit (see slp-dissent-stop-and-ask CONTEXT, Reusable
  Assets — close.rs:1420,1457-1517,2016-2046 as of that scout).
- The intervention mailbox, its turn-boundary hook delivery, and the
  frequency-cap counters shipped with slp-supervisor-heartbeat; planning
  re-scouts their landed locations rather than trusting the pre-build
  anchors.
- Poor-work signals already exist: struggling-loop (da7cb49b) plus the 2×
  estimate and same-region-resubmit telemetry (a8f4b8ab).

## Existing Code Context

Deliberately thin — the heartbeat and dissent features landed after their
scouts, so planning runs its own scout over: the intervention record store
and hook injection point, `build_close_report_doors` and the cap path in
the cells handlers, the mailbox letter renderer and WakeReport renderer
(flag placement), and the supervisor role prompt
(skills/bee-herding/references/supervisor-prompt.md — gains the
recommend-advisor wording).

## Canonical References

- docs/discovery/slp-human-up/MAP.md — the closed map; every locked row
  above is a map decision
- docs/history/slp-supervisor-heartbeat/CONTEXT.md — the intervention
  mailbox this extends
- docs/history/slp-dissent-stop-and-ask/CONTEXT.md — the debt-door
  precedent and its exact code anchors
- docs/history/research/paseo-pi-team-human-elevation.md — the distill the
  flag and nudge derive from

## Outstanding Questions

### Resolve Before Planning

None. The map closed with no open tickets and no fog.

### Deferred To Planning

- [ ] Cell-level vs feature-level targeting for "the related work" — read
      how the intervention record names its target today and pick the
      narrower honest one.
- [ ] Which surfaces carry the flag first: human-mailbox letters and
      WakeReport are named; the weekly report belongs to the waggledance
      feature and is out of this repo's slice.
- [ ] Door coverage below `standard` — mirror the dissent-debt answer
      (authoring-time obligation check) or document the gap.

## Deferred Ideas — decided against or elsewhere, not owed

- The waggledance supervisor itself, the widened ask_state digest, the
  cockpit repo, and the weekly report — the wd-supervisor feature, other
  repo (map closing note).
- The spec-drop procedure and the hat wave — slp-spec-drop-procedure,
  docs-lane, separate feature.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are bee decision-log ids,
stable. Planning reads locked decisions, code context, canonical
references, and deferred-to-planning questions. Planning's Gate 2 shape
stage and reviewing use locked decisions for coverage and UAT.
