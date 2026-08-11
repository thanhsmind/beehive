# expertise-transplant + follow-ups — learnings (captured 2026-08-11)

One session imported the useful remainder of an external engineering skill set
(mattpocock) into bee's craft layer, then wired two of its disciplines into
live skill moments, then ran the new grooming architecture lens on bee itself.
Features: expertise-transplant (et-1..5, merged c095c81b), shaping-wayfinding
(sw-1, 21e10eed), grooming-arch-lens (ga-1, 8503f4c7), grooming-round-1
(gr-1..3, merged e722856b). Parent decisions c722ed5b, 65357b01, b1490ac4,
8e455376.

## What the import taught

- **Gap analysis before transplant.** Three gather digests mapped 18 external
  skills against the existing layer; only 8 real gaps survived — the rest was
  already covered, often deeper. Spec home:
  `areas/doctrine-layer/placement-and-anchoring.md` R7 (import needs a gap AND
  a live caller).
- **Master-first sync.** The additions landed in the installed `.bee/expertise/`
  first and had to be re-synced to the plugin-root `expertise/` masters after
  the user caught the gap — deploys read the masters. Now R8 in the same spec.
- **An older plugin's onboard apply is a rollback wearing a refresh's name.**
  It deleted merges.md and stripped four files, green-looking. Edge case
  recorded in the same spec (b1490ac4); harness issue filed for a version
  guard on the apply step.

## What the wave mechanics taught

- **Wave-barrier + shared test suite = every sibling caps red until the regen
  runs.** Three of five workers finished [BLOCKED] on the same
  `failure_signature`, each correctly diagnosing the structural red (one
  proved it with a stash-and-rerun). Working as designed, but each worker
  burned attempts re-diagnosing; the existing pattern
  20260713-a-shared-suite-red-is-not-yours-while.md carried them.
- **Worker registry rows do not survive to orchestrator re-finish.** Caps
  after a worker exits need the worker re-registered (`state worker add`) or
  an inline reason — cost three retries to learn.

## What the first architecture-lens run bought

The lens's top find (kctx.rs, a self-documented 1,053-line byte-copy) was
approved and killed the same day: prediction held, one surprise — the shared
functions were already `pub(crate)`, so the fold was cheaper than proposed.
40 of 68 pre-migration backlog rows closed with per-row evidence; 28 kept.
Entropy left the capped band in one round.

## Promotion judgment

No new critical pattern: the shared-suite-red and vendoring-drift patterns
already cover this session's incidents; the onboard-rollback lesson is spec'd
as an edge case with a filed executable-guard proposal upstream — prose
promotion would duplicate it.
