---
type: grilling
status: closed
claimed-by:
blocked-by: none
---

## Question

Does bee need a first-class CHỐT/CHƯA-CHỐT label surface for
contracts/interfaces that a cell can cite (nearest today: locked
decisions in CONTEXT.md, docs/knowledge areas), so a worker can refuse
to write tests on an unsettled contract instead of minting one? And
should the user's verbatim original ask ride every cell/brief the way
the spec's original_request rides every TaskTicket (nearest: cell
goal, `bee intent`)?

## Answer

User halves: contract status is DERIVED from the decision log, never
a hand-kept registry (D ca9960f5); the verbatim original request rides
every cell/dispatch immutably, layers only add (D 3899fa60).
Mechanism half (D 9c0104e0, supersedes 2b553a89): the label is the
tag convention `contract:<name>` over the active decision set
(settled = active decision, no waiting trigger; unsettled =
waiting/due trigger); cells cite contract decisions in the existing
`cell.decisions` field; a prepare/claim-time tripwire refuses
dispatch on retired or trigger-waiting citations, plus a refusal rule
for test-writing cells citing no contract decision (the mint trap).
original_request is served by `bee intent`'s existing VERBATIM
anchor, read at dispatch prepare and rendered into every prompt
template (~10 lines; today it survives compaction but never reaches a
dispatch). Findings:
docs/history/research/slp-contract-request-surfaces.md.

Executed (feature slp-contract-original-request, 2026-08-29) with five
refinements: the mint trap ships built but RAMPS — it warns until the
first `contract:<name>` decision exists, then refuses (**d853e4c6**);
"retired" resolves to "not in the active decision set", no separate
state (**443a4999**); a `cell.decisions` entry counts as a store
citation only when it resolves against the active-plus-archive union,
short/ambiguous entries pass over silently (**45f0aeee**); the
original_request anchor is read FEATURE-KEYED only — no default or
session fallback (**36959e8b**); the citation tripwire is tag-blind
while the mint trap is tag-aware (**6f379e40**).
