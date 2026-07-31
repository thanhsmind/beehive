# Stage: compounding (`bee-capturing` — "Compound")

**Purpose** — Capture durable, reusable learnings and decisions from completed work
so the next session starts smarter, then close the feature.

**When it runs** — After scribing completes, or when work is intentionally
abandoned with lessons worth keeping.

## Inputs
- `CONTEXT.md`, `plan.md`, worker reports, `cells list --feature`, review findings,
  commit history.

## Outputs
- `docs/history/learnings/YYYYMMDD-<slug>.md`.
- Promoted **critical patterns** (an executable check is preferred over prose).
- Logged decisions, backlog friction entries, a feedback-digest refresh.
- A close commit, then `state set --phase compounding-complete`.

## Gate
None. It **registers the feature as a review candidate** and states it is
`unreviewed` — it never claims reviewed or approved.

## State touched
[`decisions log/supersede`](../register.md#beedecisionsjsonl),
[`backlog add`](../register.md#beebacklogjsonl),
[`feedback digest`](../register.md#logs--caches-read-mostly), `tmp sweep --feature`,
`state set --owner compounding --phase compounding-complete`, git commit.

## Key rules
- Three parallel **read-only** analysts (pattern / decision / failure) are
  synthesized by the orchestrator — never write-capable subagents.
- **`compounding-complete` is refused while a capped `behavior_change` cell is
  unscribed** (chain-integrity) unless `--waive-scribing-debt`. The debt check
  reads the workflow record's scribing stamp *and* the durable ledger
  `.bee/logs/scribing-runs.jsonl` (sss-1).
- The phase is set **only after the close commit lands** — close, then flip.

## Source
`skills/bee-capturing/SKILL.md` ("Compound")
