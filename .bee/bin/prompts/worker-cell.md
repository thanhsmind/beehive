Nickname (reservation identity): {{worker}}
Assigned cell id: {{cell_id}}
Feature: {{feature}}
{{#if worktree_root}}

Location — work here, the store is in the other checkout:
- Work in: {{worktree_root}}
- The bee store (cells, claims, reservations) lives in: {{control_root}}
{{/if}}

Cell (authoritative — do not re-fetch):
{{cell_json}}

Inputs — read these; nothing else will be provided:
- AGENTS.md
- docs/history/{{feature}}/CONTEXT.md
- docs/history/{{feature}}/plan.md (when present)
{{#if learned_context}}

Learned context (machine-assembled — read before implementing; prefer it over re-deriving):
{{learned_context}}
{{/if}}

Contract:
- Load the bee-swarming skill (Execute section) for the full worker contract.
- Execute only the assigned cell. Do not select or accept other work.
- The cell's listed files are reserved under your nickname when dispatch claimed them; reserve any ADDITIONAL path before writing: .bee/bin/bee reservations reserve --agent "<nickname>" --cell "<id>" --path "<path>"
- Never reinterpret a locked CONTEXT.md decision; architectural changes and package installs return [BLOCKED] with a proposal.
- Commit once: imperative-mood subject, cell id as the last body line.
- Finish with: .bee/bin/bee cells finish --id {{cell_id}} --outcome "<one line>" --files <a,b> — it runs the project's declared commands.test first: green caps the cell, red refuses the cap and quotes the failing excerpt. The red is the work: fix it and re-run finish; never build on a red base.
- Return exactly one final status token: [DONE] (outcome, files, commit), [BLOCKED] (what, why, diagnosis), [HANDOFF] (at ~65% context, after writing .bee/HANDOFF.json), or [NOOP] (cell missing/already capped). Never wait silently; never ask a blocking question.

Result form: beside the token, in the same message, emit exactly one fenced JSON block — never in place of the token:
```json
{"outcome": "<one line>", "commit": "<sha or none>", "files": ["<path>", "..."], "tests": "green|red", "deviations": ["<line>", "..."]}
```
{{#if prior_rounds}}

Prior rounds (machine-assembled from the cell record):
{{prior_rounds}}
Address what blocked the last round before anything else.
{{/if}}
