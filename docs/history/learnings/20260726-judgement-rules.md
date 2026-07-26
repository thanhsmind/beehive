# Learnings — judgement-rules (2026-07-26)

Feature: AGENTS.block.md Critical rules 16 → 14, judgement form (~1475 → ~450 words),
Red flags section deleted, censuses moved in lockstep. Decisions 5f0147ff/72a617dd/
d745a8e4/32c99ea4; validation report `docs/history/judgement-rules/reports/validation-slice1.md`.

## 1. Census-anchored doctrine is what makes safe compression possible

The rewrite dropped ~70% of the rules' words with zero guard loss because every
load-bearing phrase already had a suite-enforced anchor (placement-and-anchoring R2).
The method that worked: enumerate the anchors FIRST (from the tests, not from the
prose), rewrite around them, and give every reshaped census a mutation harness that
proves it still bites (negative control both directions — a fixture missing the new
pointer FAILS, a fixture resurrecting the dropped `>3 files` proxy FAILS). Compression
without the anchor inventory would have been the "widening the threshold" failure mode
(pattern 20260723).

## 2. A cite that names a number is ambiguous about WHOSE numbering it names

The near-miss of the feature: `AGENTS.block.md`'s "Full rule: `bee-hive` skill,
critical rule 10/11" cites bee-hive SKILL.md's OWN hive-law list, not AGENTS
numbering — the original cell ordered them renumbered 9/10, which would have silently
repointed two live pointers at the wrong rules. No suite could catch it (rule cites are
invisible to `test_skill_pointers`' POINTER_RE). Caught only by the cold-pickup cell
review. Rule of thumb: before renumbering any `rule N` cite, resolve which document's
numbering it tracks; a cross-document cite that happens to share the word "critical
rule" is a trap.

## 3. Index-based test references rot silently when the array reshapes

`writableContracts[0]` as the mutation-harness reference meant that dropping the two
AGENTS entries would silently repoint the reference at whatever landed at index 0 —
or invite deleting the whole mutation block for a green suite. Fixed by mandating
name-based lookup and banning the deletion in the cell's prohibitions. Same class as
"a shim can drop an unnamed side-effect" (pattern 20260725): the dangerous edit is the
one every visible assertion still passes after.
