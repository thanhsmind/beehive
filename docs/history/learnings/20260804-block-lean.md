---
date: 2026-08-04
feature: block-lean
categories: [doctrine-layer, instruction-layer]
severity: low
tags: [operating-block, enforced-rule-signposting, context-budget, dead-guards, enforcement-map]
---

# block-lean — an enforced prohibition earns a signpost, not a paragraph

## What Happened

The operating block (`packages/bee/AGENTS.block.md`, loaded into every session
of every governed project) was leaned under a single law, recorded as L1a-L1d
in `docs/history/block-lean/CONTEXT.md`:

- **L1a** A rule whose prohibition a hook deterministically enforces — the
  deny naming its remedy — shrinks to a one-line signpost naming the
  sanctioned path.
- **L1b** A semantic/conversational rule with no possible enforcement keeps
  full text.
- **L1c** A sentence that is the canonical home cited by skills after
  prompt-diet keeps full text verbatim (gate ownership,
  cite-never-reinterpret, the 65% handoff, the worktree-first boundary).
- **L1d** A situational section may demote to a cited reference only if its
  rules are not every-turn.

Applied to the tier-transport, guardrail, and reservation-etiquette
paragraphs (plus halving the help-usage paragraph), the block fell **184 →
174 lines, ~8.6 KB** — and because the block is loaded into every session of
every governed project, the saving is paid back on every context load, not
once. No section was added, removed, or reordered (L7); every cut that would
have changed meaning was skipped.

## The enforcement-map dependency

The lean was only classifiable because guard-hardening's audit had just
produced the enforcement map: which block rules have a deterministic hook
backstop whose deny message names the remedy (containment allowlist, CLI-owned
state denies, tier carriage via model-guard, hold/reservation denies), and
which are markdown-only by necessity
(`docs/knowledge/areas/doctrine-layer/unenforced-obedience.md`). Without that
map, "enforced" is a guess and L1a is unsafe — the audit is the prerequisite,
not a coincidence of timing. The duplication boundary now records this as its
third axis, R9 in
`docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`.

## The dead-guard caveat

The two wording guards that historically pinned the block's wording —
every-turn reachability/anchor checks (`packages/bee/tests/test_misc.mjs`) and
the pinned-string suites (`scripts/tests/test_gate_bypass_doctrine.mjs`) —
have been dead since the R6 cutover: both trees were deleted and the
instruction-layer suites were never re-pointed (`plans/cutover-readiness.md`
records the gap). The L1d not-every-turn classification was therefore applied
manually and recorded in the block-lean CONTEXT rather than checked by
automation. Porting those guards is out of scope for block-lean and remains
chip-worthy separate work.

## Recommendation

- **Enforcement is a compression licence.** Once a prohibition has a
  deterministic deny that names its remedy, its always-loaded prose owes the
  reader only the sanctioned path — the hook carries the rest. Re-run the
  classification whenever the enforcement map changes.
- **A manual classification names the guard it substitutes for.** Where the
  automated check is dead, say so in the record that relies on it — the next
  reader must know the verdict was hand-applied, and the dead guard stays
  visible until it is re-pointed or retired.
