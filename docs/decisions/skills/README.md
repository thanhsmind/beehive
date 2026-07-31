# Skill provenance archive

Development-history records evicted from the shipped skill tree under
"P1 — Provenance exile" (plans/harness-refocus.md). The product states its
rules bare; this directory keeps the rule→decision mapping and the authoring
logs for bee's own developers.

- `<skill>-provenance.md` — maps each (pre-P1) skill body rule to the
  decision IDs that authorized it, with one-line rationale. Long-form
  records: `docs/decisions/`, `.bee/decisions.jsonl`
  (`bee decisions search`), and `docs/history/<feature>/CONTEXT.md`.
- `<skill>-creation-log.md` — the authoring/pressure-test log from when the
  skill was written.
- `../cli-provenance.md` — the tag→command mapping stripped from CLI help
  text.

Nothing here is loaded by agents at runtime. When a skill rule changes,
update the skill; add provenance here only if the mapping is worth keeping
for bee's maintainers.
