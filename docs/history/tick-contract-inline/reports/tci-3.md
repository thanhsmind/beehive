# tci-3 — Fail the build when an every-turn rule is reachable only from a reference

**[DONE]**

A new blocking suite exits non-zero when a rule the doctrine's own wording says
applies every turn is reachable only from an on-demand reference file and never
named by the always-loaded operating block. Green on the tree as it stands; red,
naming the offending section by file:line, when critical rule 17 is taken back
out of the block.

**Files touched**

- `scripts/tests/test_always_loaded_rules.mjs` (new — the suite)
- `scripts/impact-registry.json` (regenerated; any newly discovered suite makes it stale)

**Commit:** `69de4c1d` — 2 files, +328, purely additive.

**Deferred-To-Planning question, settled:** derivation from the rule's own
wording, *not* an authored every-turn marker. A marker would have to be applied
by the same author who just filed an every-turn rule in a reference, and its
failure mode is silence — the same defect this suite exists to catch, one level
up. Wording derivation fails toward a red build instead. Reasoning is recorded
in the suite header, not only here.

**T6 honored:** the header states plainly that this proves *reachability*, never
*emission*. Nothing in this repo observes agent chat output, so tick emission is
not enforced and the suite must never be cited as if it were.

Full trace, verify output, deviations and friction: `.bee/cells/tci-3.json`.
