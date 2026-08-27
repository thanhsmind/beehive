# 2026-08-28 — The observer that may not act

**Feature:** slp-supervisor-heartbeat (cluster 1 of 4 in the SLP program)
**Cells:** sup-1, sup-2, sup-3, sup-5..sup-11 capped; sup-4 dropped
**Shipped in:** `bee supervisor` (record, list, pending, mark-delivered, away,
back, presence, report, metrics, consent-sweep) plus the `supervisor` role of
`bee herding control-loop`

## What was asked

Give bee an observer. Something that wakes on an interval, reads the state bee
already keeps, asks one small open question when a session looks stuck, and —
when the human steps away — hands back a single short report on return.

The thing that made it buildable was drawing the boundary first: an observer
that may **not act**. No dispatch, no merge, no approval, no product code.
Everything after that was a question of what it may read and what it may say.

## What actually shaped the code

### The boundaries had to be enumerations, not intentions

Three separate places in this feature ended up as a **named list** rather than a
rule the code applies:

- The supervisor's tool surface is an enumerated read-only set. Configuration
  cannot widen it; giving the role a write tool in config changes nothing.
- The presence mark has **exactly two** effects, and the test asserts that
  `away` and `back` write nothing under `.bee/` outside their own store. "Two
  effects" as prose would have drifted on the first convenient addition.
- The consent predicate refuses by **named variant** — gate, urgent,
  escalation, unknown kind, one-way at low confidence, not queued, already
  consented — not by a boolean. A boolean would have left nobody able to say
  why the machine waited.

The pattern underneath: when a rule's whole value is that it is narrow, the
narrowness belongs in a constant the tests can read, not in a sentence.

### Two failure directions, deliberately opposite

The notification path **fails open**: a dead notifier still leaves the record
written and the flow green. The consent path **fails closed**: anything but the
literal enabled record with a valid timeout reads as off.

Both are correct, and they are correct for the same reason — the failure mode
that must never happen differs. A broken notifier must not be able to silence
the observer. A broken config must not be able to grant it consent.

Writing them side by side in one module made the asymmetry legible. It would
have looked like an inconsistency in two separate features.

### "Not measurable" had to be first class

Seven derived counters, each with a two-sided band and an explicit sample
count. The design decision that mattered most was the fourth verdict:
`not-measurable`, which never renders as `in-band`.

A metric with no samples is not a healthy metric. Collapsing the two would have
let an empty window read as a good week — the exact failure a health report
exists to prevent.

The same instinct produced the blocked-rate denominator fix: the union of cells
claimed and cells blocked, because a swept-blocked cell carries no claim stamp,
and blocked-over-claimed can exceed one. A rate above 100% is a broken
denominator, not a bad week.

### The signal bee refused to invent

The 2x-estimate overrun signal wanted an estimate field on the cell schema.
The decision (ea02cb68) was to add none: compute the overrun only where an
estimate already exists, and otherwise report the literal string *no estimate
recorded*. Never a zero, which would read as "nothing overran".

A signal that needs a new field is a signal that will be measured against
guesses. Skip-until-present keeps the number honest and costs nothing.

## What went wrong, and what it taught

A dispatched worker died mid-cell on an API rate limit with its whole
implementation and test file on disk, uncommitted. The reflex — throw it away,
re-dispatch — would have paid twice for the same code.

Recovering it instead surfaced the sharper lesson: what a half-finished worker
most reliably has **not** reached is the last mechanical step. The cell had
every test written and had never declared its new verb in the generated
registry payload. No test caught it, because the registry contract tests only
walk what **is** declared. An undeclared verb is an absent row, and an absent
row is green.

Promoted as a critical pattern:
`docs/knowledge/patterns/20260827-a-dead-worker-has-the-code-and-is-missing-the-last-mechanical-step.md`.

## Where the behavior now lives

- `docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md`
- `docs/knowledge/areas/bee-herding/presence-wake-reports-and-earned-autonomy.md`

## What is not done

Clusters 2–4 of the SLP program — dissent / stop-and-ask, blind lanes, and
contract status with the verbatim original request — have locked map decisions
and no plan, no cells, and no feature directory. They are separate features by
a020319d, and the map lists their decision ids in build order.
