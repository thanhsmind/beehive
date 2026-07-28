# agents-block-diet — plan (frozen at Gate 2)

Source of truth for decisions: `CONTEXT.md`. This file states the shape only.

## Shape

One file changes (`packages/bee/AGENTS.block.md`), one file is regenerated
(`AGENTS.md`), one suite is ratcheted (`scripts/tests/test_agents_budget.mjs`).
The cells are **serial** — all three touch the same text or its render.

## Slice

| Cell | Does | Why it is its own cell |
|---|---|---|
| `abd-1` | The cut: rewrite the block under D5's four target classes, keeping every rule, every pinned string, and full text for the terminal-home rules. | The judgement work. Every other cell only proves or locks it. |
| `abd-2` | Regenerate root `AGENTS.md` through onboarding and prove the whole pin set green. | The render is a separate mechanism (`onboard_bee.mjs`, SHA256 in `.bee/onboarding.json`) and can fail independently of the text. |
| `abd-3` | Ratchet the budget suite: lower warn/fail to lock the win, and add a structural guard that all 15 numbered rules are present. | Without a ratchet the file regrows into a 20 KiB fence that no longer bites; without a rule-count guard a future diet can silently drop a rule. |

## Cut budget (abd-1)

Measured starting point: **16,152 bytes**. Target ≤ **12,000** (D7).

| Target | Now | Approach |
|---|---|---|
| Startup 3, 5, 6, 7 | ~2,900 B | 3/5/6 restate the session preamble; 7 self-declares optional. Collapse to a short Startup with one-line pointers (R5). Handoff keeps `never auto-resume` and all six pinned verbs (D4) — the *kind* rules survive, their elaboration goes. |
| Gate-4 / bypass paragraph | ~1,300 B | Points at `bee-bypass-gate` + `routing-and-contracts.md`, both terminal. Keep the on-request `bee-reviewing` anchor and "never self-approve"; move levels detail behind the existing pointer. |
| Rules 2, 12, 13, 15 | ~3,400 B | Each already ends in a terminal pointer. Keep the rule statement and every pinned anchor; drop the restated elaboration. Rule 12 keeps all eight fan-out pins verbatim. |
| Communication | ~1,500 B | 14 bullets + a pointer to the full contract. Keep Open/Body/Close and the pre-send check; the rest resolves through the existing pointer. |
| Working files | ~700 B | Drop the planning-time D1/D3/D4 + decision-0009 conditionals from the `docs/history/` comment. |
| Guardrails, rules 1/5/6/11 | — | **Untouched** (D3): terminal homes. |

## Verify

- `node scripts/tests/test_agents_budget.mjs`
- `node packages/bee/tests/test_misc.mjs`
- `node scripts/tests/test_doctrine_parity.mjs`
- `node scripts/tests/test_skill_pointers.mjs` (R8 pointer-integrity gate)

MAIN runs these once at feature close (R82).

## Risks

- **A pin phrased as a regex over one line.** The native-wait pointer must keep
  its phrase and `routing-and-contracts.md` on the *same* line and stay
  unnumbered. Reflowing that line breaks it.
- **Byte-identity.** Editing the template without re-rendering the root file
  fails `test_agents_budget.mjs` — abd-2 exists for exactly this.
- **The target fighting the rules.** If ≤12,000 cannot be reached without
  touching D2/D3/D6, the rules win and the miss is reported (D7).
