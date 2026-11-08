# Knowledge from code — discovery map

## Destination

A document-homes rule, built as a portable bee mechanic: four kinds of
knowledge live in docs/knowledge (vision, glossary, area map with
boundaries and invariants, decisions, patterns), three live in code
(verify features and tests, generated contracts, runnable runbooks);
every knowledge entry anchors to code and carries a freshness grade
derived from git; updates ride the cap and close doors; the mechanic
ships to every host through onboarding. Arrived means the mechanic is
in the binary and skills, and bee's own bundle has been cut to it.

## Notes

- Owner framing (2026-11-07): knowledge is the new-hire onboarding
  layer beside code. Code is the always-current document; knowledge
  supplements it with what code cannot say; the agent pulls it when it
  needs more understanding, and must be able to see how fresh it is.
- Measured on 2026-11-07, before this map: bundle 524k words (areas
  327k, patterns 81k, work 116k); 95 of 139 area concepts pointed at
  code that had changed since the concept was last touched; knowledge
  search invoked 89 times in 35,478 CLI calls; promote proposals 400
  generated, 40 unapplied; 114 capped cells never scribed.
- The 2026-11-07 pointer sweep (commit 9861c1b43) repointed 398 dead
  pointers; it fixed addresses, not the narrative.
- Verify features (`.bee/verify/verify-app/features/`) are the
  behaviour-spec home the owner accepted: tests of features, anchored
  to code as the newest document.
- Slice 1 (feature knowledge-freshness-mechanic, D1–D8) is built: seven
  cells capped on branch wt/knowledge-freshness-mechanic on 2026-11-07,
  awaiting the full-suite record and merge (2026-11-08).
- Owner framing (2026-11-08): the flow itself must say what to read at
  shaping and planning (knowledge and code), update knowledge at close
  as a door, and give writers templates that ask for information rather
  than shape; a document that nobody would miss is deleted.

## Decisions so far

- D1 (2b11f025): knowledge is the onboarding layer, pulled on demand,
  freshness visible at pull time — logged before this map.
- D2 (93e1001d): document homes rule — four kinds in OKF, three in
  code, every area entry anchored.
- D3 (27ad96a5): freshness derived from git — verified_at +
  verified_commit, grades confirmed / suspect / orphan, printed by
  search, show, context, preamble.
- D4 (e7e80d78): updates ride existing doors — cap sync door
  re-confirms anchored concepts, promote proposes patterns and
  invariant bullets only, work/ leaves the bundle.
- D5 (28fb074d): delete before write — first slice removes work/ and
  cuts areas; glossary and event catalogue wait on measurement.
- D6 (e7e133dc): pull rate is the measure, reported per feature.
- D7 (6a502ad0): the mechanic is portable through onboarding; bee's
  content cut is a separate bee-only slice.
- D8 (4148025d): an anchor is an entry in an explicit bee.anchors path
  list, seeded from resolving Pointers — agent's named guess, owner may
  override — tickets/001-anchor-definition.md
- D9 (5c5ae146): documentation is a working part of the flow — shaping
  and planning record the read set (knowledge concepts and code entry
  points, pulled, never pushed), cap and close update what changed,
  every document type has a named template that says what to add and
  where, substance over volume — owner direction 2026-11-08; the shape
  of the read set and the templates are tickets 005 and 006.
- D10 (e2f6e8c3): the area cut keeps its reasons — pre-cut tree frozen
  under docs/history/knowledge-archive/, each removed reason sentence
  cites or logs its decision, check refuses an area entry with no
  decision link — tickets/002-area-cut-without-losing-reasons.md
- D11 (f3c5ad0f): the read set is a `## Read set` section of CONTEXT.md
  (knowledge table with grade and why, code table with what it decides),
  filled by shaping, cited by planning's read_first, checked at shape
  approval — tickets/005-read-set-at-shape-and-plan.md
- D12 (2bc56fb6): every document type has a template whose sections ask
  one question and name their sources; rule bullets carry status and a
  verification pointer only where they differ from the concept; the cut
  area template is map, boundaries, invariants, entry points, pointers —
  tickets/006-template-per-document-type.md

## Features spawned

- knowledge-freshness-mechanic (D1–D8) — closed 2026-11-08.
- knowledge-docs-in-the-flow (D9, D11, D12, the check half of D10) —
  the mechanic: read set section and gate check, templates with
  prompts, rule-level fields, the no-decision-link refusal.
- The bee-only content cut (D5, the cut half of D10) — a later feature,
  after the template it needs exists.

## Not yet specified
- Whether an event catalogue can be generated from code —
  tickets/003-event-catalogue-from-code.md (research, deferred by D5)
- Glossary form and home — tickets/004-glossary-form.md (deferred by D5)
- Whether patterns need a freshness grade at all, or only a date
  (agent-suspected: date only, patterns are events)
- Whether the preamble's critical-patterns digest shrinks once areas
  carry invariants (agent-suspected)

## Out of scope

- Pushing the knowledge context manifest into every cell brief
  (rejected in D1; pull model).
- Instrumenting cell outcomes by knowledge-used arm before pull rate is
  read (rejected in D6).
