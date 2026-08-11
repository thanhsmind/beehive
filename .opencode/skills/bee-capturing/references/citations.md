# Citation Discipline

How decisions are cited from the artifacts that embody them, so a
superseded decision can find every passage that assumed it.

## Short8 ids

Any artifact that encodes a decision — a spec's Business Rules line, a
backlog row's Story/CoS, a CONTEXT or plan passage — cites the
decision's **short8 id** (the log entry's id, first 8 hex chars, e.g.
`b9b9fee3`) alongside any CONTEXT-local label (`D4`, `D11b`); the
label alone is not enough. The `decisions supersede` propagation sweep
matches short8 word-boundary hits across `docs/**` — it finds only
what is cited that way, so a passage carrying only a local label is
invisible to the scan. An uncited embodiment is the residual risk: the
decision changes, but nothing points a sweep at the passage that
assumed it.

## What must carry a citation

- Every numbered Business Rule in an area spec.
- Every config value whose number was *chosen* (thresholds, windows,
  retry counts) — a tuned number without its why is half-lost
  knowledge.
- Backlog rows whose Story or CoS restates a settled decision.
- CONTEXT/plan passages that build on a previously locked decision.

When a spec's frontmatter `decisions` list is reconciled at merge
time, it is reconciled against the active set — so the reconcile step
is itself sweepable.
