# Provenance — bee-exploring body rules

The exploring body states its rules bare (provenance exile, skill-token-diet D8). This table maps each
body rule to the decision(s) that authorize it and the rationale in one line. Long-form records:
`docs/decisions/`, `.bee/decisions.jsonl` (via `bee.mjs decisions search`), and
`docs/history/<feature>/CONTEXT.md` for the feature that locked each ID.

| Body rule | Decision IDs / labels | Rationale |
|---|---|---|
| Batch independent questions, serialize dependent ones | P20 batching (pre-migration Hard Gates) | Fewest rounds the real dependencies allow, without blind-bundling a question a prior answer could moot |
| Gather-altitude steps (scope reads, gray-area scout) delegate as I/O workers | delegation D2/D3 | Transport is mandatory on every dispatch; gather work runs down-tier as I/O, never decide-altitude |
| SEE mock — throwaway HTML mock under `.bee/spikes/<feature>/mocks/`, the one code-writing exception | P11, decision 0020 | React-instead-of-describe for a gray area the user knows-when-they-see-it but cannot describe |
| Backlog flip — feature-matching PBI moves to `in-flight` only here | D11a | This is the single place a PBI transitions to in-flight; the CONTEXT.md list stays the per-feature record |
| Brief check — a `bee-qualifying` park brief supersedes a fresh quick-scout | D9 | The brief's evidence is already settled ground; re-scouting it is wasted work |
| Command detection — `commands_detect.mjs` candidates, one pre-filled confirmation question | docs/09 item 1; harness10 D3 | Never invent command values; detection plus one confirm beats guessing or a blank open question |
| Materiality test — material, grounded, answerable | P20 | A failing question is never asked; it becomes a labeled assumption or moves to planning |
| Gate-bypass refinement — approval vs information questions under `full`/`total` | decisions 0010, dcf01d7b, a93994d3 | Bypass stops the agent asking merely to be approved; it never gags a genuine information need |
| Blindspot pass — teach before asking when the user is guessing | P9, decision 0020 | A decision locked from a guessed answer is a fake decision |
| Pinned terms — settled fuzzy words get a stable ID like a decision | P21 | Context Assembly and scribing's Data Dictionary both need a stable term source |
| Deferred Ideas also feed the product backlog | D8 | The CONTEXT.md list is per-feature; the PBI is the durable product-level intent |
| Fresh-eyes review — no-history reviewer, background where supported, blocks only Gate 1 | decision 0021 (reviewer slot); decision 0017 (background dispatch) | Independent verification without stalling CONTEXT.md assembly or the user conversation |
| Gate 1 bypass mechanics — read level first, stamp + audit line, no question presented when covered | decisions 0010, dcf01d7b | The human already chose the level; the recommended path is the approval at that level |
| Re-lane checkpoint (demotion once, evidence-only) | spec #81 P3 (bee-hive routing reference) | Same hive-wide rule exploring's scout evidence feeds; owned centrally, not restated here |
