# Validation — judgement-rules slice 1 (jr-1, jr-2)

Status: CLOSED — verdict READY WITH CONSTRAINTS at plan rev 2 (all BLOCKER/CRITICAL absorbed).

## Reality gate

| Check | Verdict | Evidence |
|---|---|---|
| MODE FIT | PASS | 2 flags (census contracts change; proof replaced with negative controls), ~6 product files → standard. plan.md Mode-gate record. |
| REPO FIT | PASS | Canonical template `packages/bee/AGENTS.block.md:34-58` exists; regen chain confirmed: `packages/bee/scripts/onboard_bee.mjs:100-104` (AGENTS_BLOCK_TEMPLATE), `scripts/render_plugin_skill_trees.mjs:1-41`; suites present on disk (`packages/bee/tests/test_misc.mjs` 189K, `scripts/tests/test_agents_budget.mjs`, `test_gate_bypass_doctrine.mjs`, `test_doctrine_parity.mjs`, `test_skill_pointers.mjs`). |
| ASSUMPTIONS | PASS w/ repair | Gather digest's census line refs spot-verified by reviewers; its `test_lib.mjs` anchor claim was FALSE (file removed in cs-2a/2b split; `fd test_lib` → only `scripts/tests/test_lib_mirror.mjs`) — jr-1 `files` repaired before review (cells update, jr-1 untouched otherwise). |
| SMALLER PATH | FAIL→standard justified | docs lane impossible (runtime-asserted test files must change); tiny/small caps exceeded and census rewrite is the widening-vs-fixing seam needing checker coverage. |
| PROOF SURFACE | PASS | Scoped verifies per cell + `release_manifest.mjs --check` + mandated two-direction negative control (pattern 20260723). |

## Feasibility matrix

| Assumption | Risk | Proof required | Evidence | Result |
|---|---|---|---|---|
| `run_verify --only <suite>` resolves the 5 suite names | LOW | suites on disk + prior usage | full verify run (this session) executed all 5 PASS; te-1 used identical `--only` form | PASS |
| Cell schedule acyclic, jr-1→jr-2 serial | LOW | `cells schedule` | waves `[[jr-1],[jr-2]]`, `cycles: []` | PASS |
| onboard `--apply` re-renders root AGENTS.md byte-identical | LOW | parity suite | `scripts/tests/test_agents_budget.mjs:96-107` asserts byte-parity; `test_onboard_bee.mjs` round-trip green in this session's full run | PASS |
| Rule-9 literal retention keeps `test_gate_bypass_doctrine` green with zero edits | MED | literal kept verbatim (D2) | `:450-454` requires substring ``whether or not the lane produced a `plan.md` (D3/D4)`` — in the KEEP list | PASS (checker re-verifies) |
| Rule-15 pointer target self-contained (R6) | MED | routing-and-contracts.md carries full census-checked contract | test_misc `assertOrderedWaitContract` runs on it independently today | PASS |
| Census rewrite keeps guard teeth | HIGH | negative control both directions | mutation-resistance self-test exists as template at test_misc ~:2272-2295; jr-1 must_haves mandate the updated fixture-FAIL proof | PASS (design; proven at cap) |
| Full-suite baseline green pre-change | LOW | CI + local | latest main CI green (run 2026-07-26T00:03Z); accidental full local run this session: all suites PASS | PASS |

## Plan-checker findings (review tier, adversarial) — verdict ITERATE → absorbed

3 BLOCKERs, 11 WARNINGs. All repaired in plan rev 2 + cell updates before any execution:

- **B1** `skills/bee-reviewing/SKILL.md:204` cites `(AGENTS.md rule 9)` — invisible to jr-2's original `critical rule` grep. → file added to jr-2, sweep widened to `rg -in 'critical rule|AGENTS\.md rule' skills/`.
- **B2** Final numbering had no authoritative artifact; `test_skill_pointers` cannot see rule cites (POINTER_RE only matches `references/` paths). → map declared authoritative in plan, jr-1 must_have pins relative order + records map in trace, jr-2 consumes the trace and BLOCKS on mismatch.
- **B3** Plan claimed Guardrails untouched; `AGENTS.block.md:81` "see critical rule 12" and rule 14's "as rule 12" must → 11. → both enumerated in plan + jr-1 action; doctrine-parity claim corrected (census still green: no retired sentence / hook-count prose touched).
- **W4** phantom `test_lib.mjs` → removed (verified: no third anchor location).
- **W5** cli-gather census (`test_misc.mjs:2887-2925`) missing from map → literals `cli gather branch` + `not an Agent dispatch` added to D2 keep list.
- **W6** `okf_instructions_fence` (chain-failing, declares AGENTS.md an instruction surface) → appended to jr-1 verify.
- **W7/W8** 6 live knowledge/spec cites + delegation-threshold B3/R3 numeric restatement → deferred to bee-scribing same feature, named in plan (not silently dropped).
- **W9** `~65%` number's home becomes bee-hive hive law 2 (jr-2 rewords; AGENTS new rule 5 keeps the principle).
- **W10** stale comment cites (run_verify:473, test_guards:54, gate_bypass:446/451/453, test_bee_cli:2819, test_misc×4) → in-scope files renumbered by jr-1; `packages/bee/bee.mjs:1603` + `lib/recovery.mjs:473` left as follow-up (comments only).
- **W11** `.bee/onboarding.json` added to jr-1 files. **W12** old-15 pointer = standalone unnumbered line under rule 14. **W13** line anchors corrected. **W14** `docs/03-workflow.md` red-flags list stays (authoring doc, not standing sheet) — stated in plan.

Clean checks: 16/16 rule coverage; keep-list vs censuses complete (after W5); Red-flags delete test-safe; all 5 `--only` tokens resolve 1:1; regen order correct; budget headroom 292 B (shrink helps); deps correct.

## Cell review findings (cold pickup) — jr-1 4 CRITICAL, jr-2 2 CRITICAL → all fixed

- **jr-1 C1 (wrong-answer bug):** original action ordered "critical rule 10/11 → 9/10" — those cites reference bee-hive SKILL.md's OWN hive-law list, not AGENTS numbering. Fixed: keep byte-identical; real intra-block renumber is rule 14 `as rule 12`→11 + Guardrails `:81`→11. Caught before dispatch — no suite could have detected the breakage.
- **jr-1 C2:** promised "bảng Approach" draft didn't exist as a table → action now says worker authors from Approach bullets + D1/D2; `.bee/decisions.jsonl` added to read_first.
- **jr-1 C3:** rule-13 census has no mutation harness (bare inline loop) → action now mandates `assertFanOutAnchors` + mutationRows mirroring `:2273-2295`.
- **jr-1 C4:** `writableContracts[0]` index-based reference invites silent negative-control deletion → action mandates drop 2 AGENTS entries + repoint reference BY NAME, mutation-block deletion prohibited.
- **jr-2 C1:** files omitted everything regen writes → 4 mirror roots + onboarding.json added.
- **jr-2 C2:** "no old numbers" grep untestable (new numbers reuse old literals) → replaced with exact 8-row target table checked line-by-line.
- Minors absorbed: idempotency proof moved to cap evidence (2nd onboard run + porcelain), manifest `--check`-after-`--write` demoted to bookkeeping, mirror-parity claim split (2 plugin trees via manifest / 2 repo-local trees via idempotent onboard).

## Approval block

- Verdict: **READY WITH CONSTRAINTS** — constraints: jr-1→jr-2 strictly serial; jr-2 consumes jr-1's trace map and BLOCKS on mismatch; scribing owes the 7 deferred knowledge-layer updates named in plan.
- Plan rev: 2 (re-approved via gate_bypass=total after absorbing findings).
- Gate 3: auto-approved via gate_bypass=total (standard lane; no hard-gate flag; advisor consult not required below high-risk).
- Full local suite baseline: green (accidental full run this session, all PASS); main CI green; no verify-red issue. Windows workflow red on main is pre-existing, filed as P2 friction (unrelated surface).
