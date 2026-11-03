---
type: bee.pattern
title: A migration is not done until its instruction layer and its generated orientation are migrated too
description: Guards protect content; nothing tests prose. The surfaces that TEACH the model — deferred-to reference docs and injected session orientation — rot silently after a migration and keep teaching the retired one.
tags: [process, migration, silent-rot, instructions, preamble]
timestamp: 2026-07-22
bee:
  id: pattern-20260722-a-migration-is-not-done-until-its-instructions-are
  lifecycle: active
  areas: [doctrine-layer]
  sources: ["okf-integration-close-f4 (the seven-gap audit and its four closing cells; CONTEXT.md `docs/history/okf-integration-close-f4/CONTEXT.md`, 2026-07-22)", red evidence `docs/history/okf-integration-close-f4/reports/red-preamble-before.md`]
  polarity: pitfall
  critical: true
---

# A migration is not done until its instruction layer and its generated orientation are migrated too

A switchover shipped with every guard in place: new truth mechanically fenced into the new tree, a
coverage gate per migrated area, a conformance check failing the chain. All green, and it stayed
green. An audit afterwards still found **seven** surfaces routing to the retired model.

The two worst were found by *reading the session's own startup output*, not by inference: the
critical-patterns digest was emitting four lines of YAML delimiters and six of a pointer stub's
forwarding address — not one lesson — and the project map was counting the read-only compatibility
surface as content to read "before the code". The sharpest was measured with `grep -c`: the two
reference files the routing skill explicitly defers to for "the full routing table" and "the full
pipeline" mentioned the new tree **zero** times, against four and one for the old. An agent
following the *complete* instructions was taught the order the migration had replaced.

**Why it hides:** migration guards protect *content* — where truth may be written, whether it
parses, whether coverage is complete. Prose has no test, so it rots without a single red. And the
worst-hit surfaces are the two nobody opens during review: files a skill *defers to* rather than
files a skill *is*, and output *generated* into a session preamble rather than authored.

**Rule.** Before declaring a migration done, audit the instruction layer and the generated
orientation by MEASUREMENT, not by reading: grep every skill, reference, template and injected
preamble for the old path and the new one, and treat any file mentioning the old and not the new as
unmigrated. Weight two classes highest — deferred-to references, and anything rendered into session
orientation. For the latter, **read the emitted output**, never the code that emits it. A count of
zero for the new path in a file that governs routing is a finding, not a stylistic gap.

**Corollary.** Every such passage gets BOTH branches, so a consumer that never migrated reads
today's guidance unchanged and cannot tell the migration happened — and the unmigrated branch is
pinned by a fixture, because the repo that did migrate can no longer exercise it.
