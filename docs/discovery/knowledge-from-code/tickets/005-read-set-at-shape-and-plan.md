---
type: grilling
status: closed
blocked-by: none
---

## Question

Where does a feature record the knowledge concepts and the code entry
points it was read against, and who writes it: a `## Read set` section
in `docs/history/<feature>/CONTEXT.md` that shaping fills from
`bee knowledge context` and `bee knowledge search` (concept path, grade
at read time, one line on why it matters) and that planning extends
with the code paths each cell's `read_first` names; or a separate
`docs/history/<feature>/read-set.md` manifest the CLI emits; or nothing
new, with the existing `Existing Code Context` section of CONTEXT.md
carrying concept paths beside code paths?

D1 (2b11f025) holds either way: the read set is what the leader pulled,
recorded for the next reader; it is never pushed into a worker brief.

Agent recommendation: the first form. CONTEXT.md already has
`Canonical References` and `Existing Code Context`; a `## Read set`
section with two tables (knowledge: path, grade, why; code: path, what
it decides) costs one template change in bee-shaping, one check in
`bee gate` shape approval (section present and every knowledge path
resolves), and gives planning's `read_first` a source to cite instead of
a hunt.

## Answer

A `## Read set` section in `docs/history/<feature>/CONTEXT.md` with two tables (knowledge: path, grade, why; code: path, what it decides), filled by shaping from `bee knowledge context` and `bee knowledge search`, extended by planning whose `read_first` cites it; shape approval checks the section exists and every knowledge path resolves; nothing pushed into briefs. Owner agreed 2026-11-08. Logged as D11 (f3c5ad0f).
