# Provenance — bee-swarming body rules

The swarming body states its rules bare. This table maps each rule that carries a decision id to its source. Long-form records: `docs/decisions/`, `.bee/decisions.jsonl`, and `docs/history/<feature>/CONTEXT.md`.

| Rule | Decision | Rationale |
|---|---|---|
| Never read a worker's transcript, pane log, log.txt or activity stream for progress; use `bee herding status --job <id>` | d9007aca | A transcript read costs the leader's context and says nothing the status line does not |
| A suspect concept anchored by a changed file is fixed or confirmed and committed with the cell | docs/history/knowledge-freshness-mechanic/CONTEXT.md D4 | The cap door refuses a cell whose anchored concept is left suspect |
| bee-guard runs at the review tier, or the generation tier when review is unconfigured | decision 0021 | An unconfigured review slot must still produce a reviewer |
| The Execute loop and the Tester contract live in `references/execute.md`; `worker-cell.md` is the one home of the Result form | 6d9d5f99 (semantic-token-diet D3) | A native worker loaded all of SKILL.md for a contract that is one section of it, and the Result form existed twice with different keys |
