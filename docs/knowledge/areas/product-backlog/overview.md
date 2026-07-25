---
type: bee.area
title: Product Backlog — Mechanical Passes and Submission
description: "The product backlog table (numbered work items tracked by status): counting rows by status, reordering by status priority, rendering summary badges, and directly submitting a new work item with an auto-assigned identifier."
timestamp: 2026-07-23
bee:
  id: product-backlog-overview
  lifecycle: active
  areas: [product-backlog]
  required_context: []
  decisions: [D1 backlog-submit-command, D2 backlog-submit-command, D3 backlog-submit-command, D4 backlog-submit-command, D5 backlog-submit-command]
  sources: ["docs/history/backlog-submit-command/ (cell backlog-submit-command-1, capped)", "docs/history/backlog-auto-commit/ (prior-art scoped CLI addition, unrelated store)", ".bee/backlog.jsonl friction 2026-07-23T10:20:34.557Z (known counting defect)"]
  authoritative_for: "product-backlog: mechanical passes and CLI verbs (counts/rank/badges/add/propose)"
---

# Product Backlog — Mechanical Passes and Submission

The product backlog is a table of numbered work items. Each row carries a status
(`proposed`, `in-flight`, or `done`) that tracks where the item stands. This
concept owns the mechanical operations available on that table: counting rows by
status, reordering them by status priority, rendering a summary of the counts,
and directly submitting a new item into the table.

## Entry Points & Triggers

- **An operator or agent asks for the current counts** — how many items are
  proposed, in-flight, or done, at any time.
- **An operator or agent asks to reorder the table** — active work floats to the
  top, finished work sinks to the bottom, at any time.
- **An operator or agent asks to refresh the project's summary badges** — the
  compact counts display shown to anyone browsing the project, at any time.
- **An operator or agent directly submits a new work item** — supplying the
  item's story and its completion criteria (and, optionally, a related feature
  label) — the table gains one new row on demand, without going through any
  other workflow first.

## Data Dictionary

### A backlog row — five fields

| Field | Meaning |
|---|---|
| Identifier | A sequential number, assigned once and never reused, that names the row for reference elsewhere |
| Story | The one-line description of what the item delivers |
| Completion criteria | What must be true for the item to count as done |
| Status | Where the item stands — `proposed` (not started), `in-flight` (active work), `done` (finished) |
| Related feature | An optional label naming the piece of work the item belongs to; unassigned rows carry a placeholder dash |

## Behaviors & Operations

- **Counting.** Every row's status is read and tallied into one of the three
  recognized totals. A row whose status cell does not match one of the three
  words *exactly* — including a row that carries the word plus extra
  descriptive text alongside it — is silently excluded from every total (see
  Open Gaps).
- **Reordering.** Rows are grouped by status priority — active work first,
  not-yet-started work next, finished work last — with each group's original
  relative order preserved. A row with an unrecognized status keeps a neutral
  position between the not-yet-started and finished groups.
- **Badge rendering.** The three totals are rendered as a compact, colored
  summary for display outside the table itself.
- **Submission (D1/D2/D3).** An operator or agent supplies a story and
  completion criteria; the related-feature label is optional and defaults to
  unassigned. The table gains exactly one new row, in the `proposed` state,
  identified one past the highest identifier ever used anywhere in the table —
  never a gap left behind by an earlier, no-longer-used identifier. Submission
  is the whole operation: it never triggers any further evaluation, triage, or
  work on the new item — that happens later, through the normal pickup flow,
  separately and on its own schedule.

## Actors & Access

Any operator or agent with write access to the project may trigger any of
these operations. There is no additional authorization surface — these are
internal project-management operations, not exposed to anyone outside the
project.

## Business Rules

- **R1** (D2, backlog-submit-command) — A newly submitted item's identifier is
  always one past the highest identifier ever used in the table. A gap left by
  an earlier, no-longer-used identifier is never filled.
- **R2** (D3, backlog-submit-command) — Submitting a new item never starts work
  on it. The item sits at `proposed` until a separate, later pass picks it up.
- **R3** (D1, backlog-submit-command) — A submission's content comes only from
  what the caller supplies directly (story, completion criteria, optional
  feature label) — it never imports or copies from any other record-keeping
  store.
- **R4** (D1, backlog-submit-command) — A story or completion-criteria text
  that is missing or exceeds its length limit is rejected before anything is
  written; the table is left exactly as it was.

## Edge Cases Settled

- Submitting into a table that already has a gap in its numbering does not
  fill the gap — the new item still gets the next number past the current
  highest (D2).
- Submitting with no related-feature label records the label as unassigned,
  matching the table's existing convention for other unassigned rows.

## Open Gaps

- **Counting/reordering/badges silently drop a row whose status carries extra
  text alongside the recognized word** (for example, a finished item's row
  that also links to its own record) — such a row is excluded from every
  total, not flagged as unusual. Confirmed affecting 4 finished rows in the
  live table as of 2026-07-23 (this area's own submission-feature row among
  them, once it finished and picked up the same link convention). Filed as a
  friction item the same day; not yet fixed.
- **A record already logged elsewhere (a routine note an agent files about its
  own work, for example) cannot be turned directly into a submitted item** —
  submission only accepts a story and completion criteria typed fresh; nothing
  auto-fills them from an existing record. Deferred; tracked as a proposed
  backlog item (P81) pending real need.
- **The other flows that already create table rows as a side effect of their
  own work** (locking in a new feature's scope; recording an idea that got
  pushed out of scope) **do not yet go through the submission operation
  itself** — they still add rows by hand, following their own separate
  instructions. Deferred; tracked as a proposed backlog item (P82) pending a
  real need to unify them.

## Pointers (implementation)

- **P1** — Counting/reordering/badges: `packages/bee/lib/backlog.mjs`
  (`readBacklogCounts`, `rankBacklog`, `renderBacklogBadges`); CLI:
  `bee backlog counts|rank|badges`.
- **P2** — Submission: `packages/bee/lib/backlog.mjs`
  (`proposePbiRow`), `packages/bee/bee.mjs`
  (`handleBacklogPropose`), `packages/bee/lib/command-registry.mjs`
  (`backlog.propose` entry); CLI: `bee backlog propose --story "<s>" --cos "<c>"
  [--feature <slug>]`. Mirrored to `.bee/bin/**` and the plugin/skill trees via
  `skills/bee-hive/scripts/onboard_bee.mjs --apply`. Tests:
  `packages/bee/tests/test_cli_cells.mjs`,
  `packages/bee/tests/test_bee_cli.mjs`.
- **P3** — Known counting defect (Open Gaps, first bullet):
  `packages/bee/lib/backlog.mjs` `normalizeStatus`/
  `readBacklogCounts` (exact-match only, no tolerance for a trailing
  annotation) — filed as friction `.bee/backlog.jsonl` 2026-07-23T10:20:34.557Z.
