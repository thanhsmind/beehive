# Provenance — bee-grooming body rules

The body states its rules bare (provenance exile, skill-token-diet D8). This table maps each
body rule to the decision(s) that authorize it and the rationale in one line. Long-form records:
`docs/decisions/`, `.bee/decisions.jsonl` (via `bee.mjs decisions search`), and
`docs/history/<feature>/CONTEXT.md` for the feature that locked each ID.

| Body rule | Decision IDs / labels | Rationale |
|---|---|---|
| Scope = the current project; harness (`.bee/`, `.claude/`, `.codex/`) is never project debt; a harness bug is a one-line upstream note | decision 0014 (grooming-project-first) | A real audit found grooming had drifted into auditing the harness instead of the user's project |
| Entropy score demoted to a short hive-housekeeping side-note, never the headline | decision 0014 | Six of seven score terms are `.bee/` bookkeeping; leading with it frames grooming as "is the hive tidy" not "is the project healthy" |
| Findings in plain project language, never bee-jargon | decision 0014 | Readers are non-bee project owners; "orphaned cells" means nothing to them |
| Duplicated area truth: two specs covering one surface is a stale/duplicate candidate | decisions 0001/0002 | The state layer (area specs + reading map) and the scribing skill are the source of truth this check protects |
| Test-prune hard gate: merge or delete only, never a raw line-count cut; every touched suite green AFTER the prune, same batch | test-economy D4 | Deleting a test narrows guard behavior — it ships under the same discipline as any other guard narrowing |
| Test-prune: surviving case(s) must still demonstrably catch what the pruned duplicates caught | test-economy D8 (negative-control principle) | A quieter suite is not proof of a safe prune; a still-triggering guard is |
| §1/§2 mechanical scans delegate as extraction/generation-tier I/O workers; dead-code proof stays generation; ad-hoc dispatch needs the `[bee-tier: ceiling]` marker | Delegation contract D2/D3 (`bee-hive/references/routing-and-contracts.md`) | Decide-altitude work stays with the orchestrator; mechanical gather/scan work dispatches down-tier with a declared transport |
