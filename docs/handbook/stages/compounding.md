# Stage: compounding (`bee-capturing` — "Compound")

**Purpose** — Capture durable, reusable learnings and decisions from completed work
so the next session starts smarter, then close the feature.

**When it runs** — **Deferred by design, like review.** A green `bee close` records
capture as pending — the capture queue plus uncaptured `behavior_change` cells are
the signal, and `bee orient` keeps the reminder — and Compound runs when the owner
chooses, batching several closed features into one session when that is cheaper.
Deferred is never dropped: the reminder stands until it runs, and abandoning work
with lessons worth keeping still runs it.

## Inputs
- `CONTEXT.md`, `plan.md`, cell traces, worker reports, review findings, commit
  history — and the touched areas' *existing* learnings, read first so the harvest
  extends the layer instead of restating it.
- The capture queue (`bee capture list`).

## Outputs
- A drained capture queue: each stub gets its full spec merge, oldest first
  (`bee capture flush --id <id> --into <spec>`). A stub is never dropped or
  summarized away.
- One dated learnings file, `docs/history/learnings/YYYYMMDD-<slug>.md`.
- Promoted **critical patterns** — an executable check is preferred over prose.
- Logged decisions (superseding outdated ones, never editing them), backlog
  friction entries, a refreshed feedback digest.
- A close commit, then the close recorded in state and the head registered as a
  review candidate.

## Gate
None. It **registers the feature as a review candidate** and states it is
`unreviewed` — it never claims reviewed or approved.

## State touched
[`capture list/flush`](../register.md#beecapture-queuejsonl),
[`decisions log/supersede`](../register.md#beedecisionsjsonl),
[`backlog add`](../register.md#beebacklogjsonl),
[`feedback digest`](../register.md#logs--caches-read-mostly), `tmp sweep --feature`,
`state compounding-run`, `state set --owner compounding --phase
compounding-complete`, [`reviews candidate add`](../register.md#beereviewsreview-idjson),
git commit.

## Key rules
- **Delegate the reading, keep the synthesis** — read-only subagents mine the
  artifacts; the orchestrator writes. Never write-capable subagents.
- **Thin evidence means a thin file, never an invented finding.**
- **Promotion clears all three bars** — multi-feature relevance, meaningful waste
  prevented, generalizable — or it stays a learning.
- **`compounding-complete` is refused unless the run was stamped.** `state
  compounding-run` is legal only from phase `compounding`, and the terminal
  transition refuses unless `last_compounding_run` exists, names the *same* feature
  as `last_scribing_run`, and was stamped at or after it. The scribing-debt door
  reads the workflow record's stamp *and* the durable ledger
  `.bee/logs/scribing-runs.jsonl`; `--waive-scribing-debt` is the recorded, visible
  exception.
- **Capture debt stops the close itself.** `bee close` refuses while the feature has
  `behavior_change` cells capped since the last scribing stamp and nothing captured
  them. Drain it with `bee-capturing`, or log a `capture-deferral` decision naming the
  feature — the deferral is a record, not a skip.
- **Housekeeping warns, never blocks** — a failed digest refresh or scratch sweep
  is a one-line warning, never a delay or a reversal of the close.
- The phase is set **only after the close commit lands** — close, then flip.
- Historical records are never rewritten: decisions are superseded, learnings and
  logs appended.

## Source
`skills/bee-capturing/SKILL.md` ("Compound") + `references/promotion.md`;
craft in `.bee/expertise/knowledge.md` and `.bee/expertise/decisions.md`
