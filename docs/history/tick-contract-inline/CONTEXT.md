# Tick Contract Inline — Context

**Feature slug:** tick-contract-inline
**Date:** 2026-07-29
**Exploring session:** complete
**Scope:** Standard
**Domain types:** READ (doctrine surfaces), RUN (verify suite)

## Feature Boundary

Put the operative progress-tick contract where an agent actually reads it — the
always-loaded operating block — instead of behind a pointer to a reference file,
fix the lint matcher that misreports that pointer as missing, and add a check
that can actually fail when an every-turn rule is unreachable from the body.
Ends when the full verify is green with the new suite in place. Does not attempt
to enforce tick emission.

## Feature Origin

Diagnosis this session: progress ticks are specified as a "mandatory ak-style
per-step contract" but did not apply. Three causes stacked — the rule lives only
in `skills/bee-hive/references/routing-and-contracts.md:238-303`, the
always-loaded summary understates it as a bypass-scoped outcome line, and the
one check that would have flagged the gap (`scripts/skill_lint.mjs:98-110`) is
both broken and advisory-only.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| T1 | The **operative contract** moves into `packages/bee/AGENTS.block.md`: every perceivable pipeline step emits exactly one short chat line, on by default; no bypass level ever silences a tick; the fixed format `<glyph> <event>: <what> — <key fact>`; the four-row glyph table; and both silence switches named (`quiet` in config, and `ship_visibility` covering only the two PR ticks). | These are the clauses an agent must have in context every turn to comply. The always-loaded summary today reads as *bypass = one outcome line per cap/slice/wave/re-lane*, which is narrower than the rule on two axes — it sounds conditional on bypass and scoped to four event types, when the rule is unconditional and covers every perceivable step. |
| T2 | The **24-row worked-example catalog stays** in `routing-and-contracts.md`. It is examples, not the contract. | Byte math, not taste: the full section is 5,848 bytes; the block has 1,308 bytes of headroom to the warn line and 2,308 to the hard fail. Inlining the catalog lands the block at ~18.5K — over the hard fence by 3.5K. The contract without the catalog is ~1,850 bytes and fits once T3 pays for it. |
| T3 | The new bytes are paid for by **removing bytes from the block**, not by raising a threshold. `WARN_BYTES` (14000) and `HARD_FAIL_BYTES` (15000) in `scripts/tests/test_agents_budget.mjs` are unchanged. | The repo's own fence doctrine — *"pay for new text by removing text, not by appending"* — and this addition is exactly the kind of growth the warn line exists to resist. Raising it here makes the next raise easier. |
| T4 | `scripts/skill_lint.mjs:107` accepts a pointer whose parenthetical carries **more than one** quoted heading. | It searches for the literal `("Progress ticks")`. `skills/bee-hive/SKILL.md:120` reads `("Silent Bookkeeping", "Progress ticks")` — a reachable pointer that the matcher cannot see. It has been reporting a false missing-pointer for as long as that line has had two headings, and two workers this session recorded it as "pre-existing advisory, unrelated" and moved on. |
| T5 | A new suite fails when a rule marked as applying **every turn** is reachable only from a reference file and not from the always-loaded body. It runs in the full verify chain and exits non-zero on a finding. | `skill_lint.mjs` always exits 0, so its findings never block; that is why this specific gap survived. The new check targets the root cause — a rule stored where the agent does not load it — rather than the symptom. |
| T6 | Tick **emission** is not enforced, and the close report says so plainly. Nothing in the repo observes agent chat output: no hook parses it, no test asserts on it. | Recording T5 as if it closed the emission gap would make the check surface claim coverage it does not have. T5 guarantees the rule is *reachable*, never that it was *followed*. |
| T7 | Every prose token pinned by `scripts/tests/test_gate_bypass_doctrine.mjs` and `packages/bee/tests/test_misc.mjs`'s census survives the block edit unchanged, and the rule roster stays contiguous at `EXPECTED_RULE_COUNT`. | Cell vd-14 had to restore four tokens a prior block rewrite dropped. The trimming in T3 is the same class of edit and carries the same risk. |

### Agent's Discretion

- Which specific block prose is trimmed to pay for the contract, provided T7 holds.
- Whether the contract lands as an extension of the existing work-language rule or as its own rule (if its own, `EXPECTED_RULE_COUNT` moves in the same cell).
- The detection technique in T5, provided it derives its own ground truth rather than comparing two hand-authored lists.

## Existing Code Context

### Integration Points

- `skills/bee-hive/references/routing-and-contracts.md:238-303` — the source section (5,848 bytes). The catalog stays; the contract clauses move.
- `packages/bee/AGENTS.block.md:44` — critical rule 10, the current work-language rule that understates the contract.
- `skills/bee-hive/SKILL.md:120` — the two-heading pointer the lint matcher cannot see.
- `scripts/skill_lint.mjs:98-110` — the matcher, and the always-exit-0 behavior that makes it unable to block.
- `scripts/tests/test_agents_budget.mjs` — `HARD_FAIL_BYTES` 15000, `WARN_BYTES` 14000, `EXPECTED_RULE_COUNT` 16, `TERMINAL_HOME_RULES` `[1, 5, 6, 11]`; also asserts root `AGENTS.md` is byte-identical to the template between the BEE markers.
- `packages/bee/scripts/onboard_bee.mjs` — renders the template into root `AGENTS.md`. Never hand-edit the rendered file.

## Canonical References

- `docs/knowledge/patterns/20260728-a-derivation-the-tooling-computes-but-doctrine-forbids-where-it-is-needed.md` — the same shape as T5: a check exists but cannot reach the decision that needs it.

## Outstanding Questions

### Deferred To Planning

- [ ] Whether T5's "applies every turn" marker is an explicit annotation in the doctrine source or derived from the rule's own wording. An explicit marker is checkable but must itself be maintained; derivation is fragile but self-updating.

## Deferred Ideas

- Enforcing tick emission by observing chat output — would need a new hook surface that reads assistant messages. Real, and much larger than this cut. File as a PBI.
- Raising `skill_lint.mjs` from advisory to blocking — it currently carries at least one other unresolved advisory, so flipping it red is its own cleanup feature.

## Handoff Note

CONTEXT.md is the source of truth. T2 and T3 are the binding constraints: the
catalog cannot fit, and no threshold moves. T6 is what the close report owes the
user — this feature makes the rule reachable, not obeyed.
