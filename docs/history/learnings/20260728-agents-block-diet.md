---
date: 2026-07-28
feature: agents-block-diet
categories: [doctrine-layer, context-budget]
severity: medium
tags: [operating-block, duplication-boundary, size-budget]
---

# agents-block-diet — feature close learnings

## What Happened

`packages/bee/AGENTS.block.md` — the operating block bee renders into every host
repo's `AGENTS.md`, auto-loaded every session and re-read after every compaction
— had grown by accretion to 16,152 B against a 20 KiB fence that had never once
bitten. It fell to **12,573 B (−3,579, −22.2%)**; the rendered root file followed
to 12,692 B. All 15 numbered critical rules survived, numbered and ordered, with
every section heading and every pinned string intact.

## Finding 1 — a document that has been cut *toward* becomes harder to cut

The obvious plan was "these rules restate their own reference, so replace the
body with the pointer". Checking the real files first killed half of it.
`bee-hive/SKILL.md` had already been slimmed under `router-cost` **by deferring
to this file**: it says "Rules 2-4, 13 appear in full in `AGENTS.md`" and sends
readers to "`AGENTS.md` Guardrails". Those rules are therefore *terminal* here,
and answering a defer-back with a defer-out builds the pointer loop R6 forbids —
one where the rule's full text lives nowhere at all.

*Rule: before a second size pass on any document in a pointer network, check
which direction the boundary last ran. A prior cut does not leave a file easier
to cut again; it leaves parts of it load-bearing that were not before.*

## Finding 2 — the cheapest safe cut is content the reader is handed anyway

The single largest saving was startup steps restating what the SessionStart
preamble already prints unprompted, plus one step that described itself as
"optional… not mandatory every session" (2,919 → 1,554 B). Neither shows up in a
duplication list built by comparing *documents*, because the duplicate is a
generated runtime surface. Anything that arrives by itself, or declares itself
skippable, does not belong in an always-loaded file.

## Finding 3 — a byte budget alone is an unsafe instrument

A budget rewards cutting and cannot distinguish "cut 400 bytes of restated
elaboration" from "cut critical rule 7". Ratcheting the fence onto the achieved
size (20,480/18,000 → 15,000/14,000) was therefore shipped **only together with**
a structural guard: the 1..15 roster pinned on both the template and the render,
a negative control proving it bites, and a check refusing any terminal-home rule
compressed into a bare cross-reference. This closes the `router-cost` open gap
"no check measures the size" for the operating block, with the pairing named as
the condition.

The negative control earned its keep immediately: the first version blanked
*Startup step 7* rather than *critical rule 7*, and the roster check stayed
green. The section scoping is now proven rather than assumed.

## Finding 4 — the target missed, and that was the correct outcome

CONTEXT.md D7 set ≤12,000 B and said in advance that if the target and the
keep-every-rule decisions conflicted, the rules win and the miss gets reported.
The cut stopped at 12,573 — a 573 B miss — because what remained was rules,
why-clauses and pinned strings. *Writing the tie-break into the decision before
the work is what let the number be missed honestly instead of quietly making
room by cutting a why-clause.*

## Recommendation

- Inventory the pins from the **suites**, not from memory, before editing a
  contract file. One census (`cli gather branch`) was missed on the first pass
  and caught by `test_misc.mjs`; the rest held because they were extracted first.
- Keep why-clauses on safety rules. "An unblocked write is not an approved write"
  is what makes rule 11 survive an agent under pressure; the same rule compressed
  to a bare imperative gets reasoned around.
- Pair every size budget with a structural guard, or do not ship the budget.
