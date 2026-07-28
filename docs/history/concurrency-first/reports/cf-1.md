# cf-1 — one concurrency law, lanes first-class

**[DONE]** — the core concurrency law now lives in the always-loaded layer and
the full protocol (mandatory concurrency plan, exhaustive legal-serial
reasons, lanes-first-class routing, tick) is written into
`routing-and-contracts.md`.

Files touched: `AGENTS.md`, `skills/bee-hive/references/routing-and-contracts.md`,
`skills/bee-hive/SKILL.md`. Commit: `586fdd3a`.

Known gap (see friction on the cell): `packages/bee/AGENTS.block.md` (the
template) is out of byte-identical sync with the rendered root `AGENTS.md`
— left untouched per the HARD BOUNDARY (sibling feature holds
`packages/bee/**`). `test_agents_budget.mjs` will fail on this until a
follow-up cell syncs the template.

Full trace/evidence: `.bee/cells/cf-1.json`.
