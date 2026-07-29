# tci-1 — Move the operative tick contract into the always-loaded operating block

**[DONE]**

The progress-tick rule now sits in the file an agent loads every turn: it lands in
`packages/bee/AGENTS.block.md` as critical rule 17 (one short chat line per perceivable
step, on by default; no bypass level at any tier silences a tick and a red or refusal is
never silence-able; the fixed `<glyph> <event>: <what> — <key fact>` format; the four-row
glyph table; `quiet` and `ship_visibility` both named). The 24-row worked-example catalog
stays in `routing-and-contracts.md`, reframed as examples that point at the block for the
rule (T2). Paid for by removal, not by moving a threshold (T3): 835 bytes of restated
prose came out, leaving the block at 13,394 B against the 13,396 B it started at, with
`WARN_BYTES`/`HARD_FAIL_BYTES` untouched. `EXPECTED_RULE_COUNT` moved 16 → 17 in this
same cell; appended, not inserted, so `TERMINAL_HOME_RULES` `[1, 5, 6, 11]` is unaffected
(T7).

## Files touched

- `packages/bee/AGENTS.block.md` — rule 17 + glyph table added; restated prose removed
- `AGENTS.md` — re-rendered through `onboard_bee.mjs`, never hand-edited
- `skills/bee-hive/references/routing-and-contracts.md` — contract clauses removed, catalog kept and reframed
- `skills/bee-hive/SKILL.md` — rule-10 pointer now names the block as the rule's home
- `scripts/tests/test_agents_budget.mjs` — `EXPECTED_RULE_COUNT` 16 → 17

Commit `c3c67005`. Full trace, verify command and evidence: `.bee/cells/tci-1.json`.

## Deviations

1. **Two stale pointers removed as part of the byte payment.** Block rules 9 and 10 both
   carried `Full rule: bee-hive skill, critical rule N` citations that were off by one
   (rule 9 → hive rule 10, rule 10 → hive rule 11), sending a reader to the wrong hive
   rule. Removing them is a fix, not a loss — the block states those rules more fully
   than the hive one-liners do.
2. **`skills/bee-hive/SKILL.md` paid its own budget.** The reworded rule-10 line put that
   body 76 B over `skill_budget_fence`'s baseline. Rather than raise the baseline, the
   line was tightened and hive rule 12's middle clause (`an unblocked write is not an
   approved write`, stated in full in the block's rule 11, which that same line points
   at) was dropped. Fence green at 0 findings.
3. **The block's stated size was wrong in the cell.** The cell and CONTEXT.md put the
   block at 12,692 B with ~550 B to remove; the file was actually 13,396 B. The removal
   target was recomputed from the measured size, not the briefed one.

## Note for the feature close (T6)

Nothing here enforces tick *emission*. This cell makes the rule reachable from the
always-loaded layer; it does not observe agent chat output, and no test asserts a tick
was ever written.
