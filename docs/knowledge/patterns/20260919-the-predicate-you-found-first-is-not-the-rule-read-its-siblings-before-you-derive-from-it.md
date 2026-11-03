---
type: bee.pattern
title: The predicate you found first is not the rule — read its siblings before you derive from it
description: "Deriving behavior from another component's decision function by reading one predicate and inferring the rest: the guard sorted phases into four groups, not the two the first read assumed, and a table test that derived its expectation from the same predicate could not tell the difference."
tags: [failure, guards, tests, duplication, reading]
timestamp: 2026-09-19
bee:
  id: pattern-20260919-the-predicate-you-found-first-is-not-the-rule-read-its-siblings-before-you-derive-from-it
  lifecycle: active
  areas: [hook-runtime]
  sources: [".bee/cells/archive/stage-gate-phase-parity/sgpp-1.json", "commit 5bcebcba2", "docs/history/stage-gate-phase-parity/plan.md (\u00a7 Revision 1 — corrected after the independent read)"]
  polarity: pitfall
  evidence: wired
  evidence_ref: "packages/bee-rs/crates/bee/src/hooks/stage_tools.rs (allowed_tools_for, calling write_guard::is_gated_phase AND is_known_phase; the literal EXPECTED_WITHOUT_GATE table and stage_tools_table_covers_every_phase_the_write_guard_knows, which imports write_guard::KNOWN_PHASES so a phase added upstream fails here)"
---

The Pi tool gate had to answer one question: in which phases may the model hold
write tools? The write guard already owns that answer, so the fix was to ask it.
The agent opened `write_guard/paths.rs`, found `is_gated_phase` — `exploring` and
`planning` — and wrote the rule as "narrow where the guard gates, open everywhere
else". Every test passed. The live probe agreed. The rationale went into a plan,
a decision record and a doc comment.

It was false. Four lines above `is_gated_phase` sat `is_terminal_phase`, and one
file over sat `is_known_phase`. The guard does not sort phases into two groups;
it sorts them into four. A terminal-state write is refused outside four path
prefixes. A write under a phase the guard does not recognize is refused
outright. The shipped rule handed write tools to that last group — tools whose
every call the guard would deny.

Nothing caught it. The unit tests could not: they derived their expectation from
the same `is_gated_phase` call the implementation made, so a wrong rule and a
wrong expectation agreed. The live probe could not: it showed what the hook
answered, never what the guard would do with that answer. Only an independent
read of the guard found it, by opening the sibling predicates the first read had
walked past.

**A predicate that answers part of a question is not the rule.** When you derive
behavior from another component's decision function, the function you found
first is a candidate, not the answer. Before treating it as the rule:

- Read every sibling predicate over the same input. They cluster — `is_gated_phase`,
  `is_terminal_phase` and `is_known_phase` were within a few lines of each other,
  named alike, and all took a phase.
- Read the consumer, not just the definition. The dispatch that calls them in
  order is where the real grouping lives; a predicate alone never shows you how
  many branches there are.
- Count the branches and say the number out loud. "The guard has two groups" is
  a claim that can be checked against the call site in one minute. "The guard
  gates X" is a claim that hides how many other groups exist.

**A derived expectation is not a test.** A table test whose expected value comes
from the same function the code calls can only catch a reverted or inverted
rule. It cannot catch a rule that is wrong *through* that function. Spell the
table out literally — one row per input, the expected output written by hand —
and add a separate test asserting the table covers every input the upstream
component knows about. Then a new phase upstream fails loudly instead of falling
through a default.

**The cost when it is missed.** Here the independent read was owed by a guard on
the source root and was nearly acked away as ceremony for a tiny lane. Running
it cost one dispatch. Skipping it would have shipped a rule whose own rationale,
decision record and doc comment all stated something the code four lines away
contradicted — and the wrong branch was the silent one, since a phase nobody
recognizes is rare enough to surface long after the change.

Related: [[pattern-20260818-a-rule-checked-at-two-points-needs-one-shared]] is
the same defect seen from the code side — that one is about a rule with two
homes drifting apart, this one is about reading one home and believing it is the
whole rule. [[pattern-20260812-a-guard-and-its-tests-are-one-model-so-green-proves-only-that-the-model-agrees-with-itself]]
is why the green suite here proved nothing.
