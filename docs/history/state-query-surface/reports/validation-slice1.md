# Validation — state-query-surface, slice 1 (sqs-a → sqs-b1 → sqs-b2 → sqs-b3)

Standard lane. No hard-gate flag → advisor consult not a precondition (AO2b applies to high-risk only).

## Reality gate

| Lens | Verdict | Evidence |
|---|---|---|
| MODE FIT | PASS | 2 flags (public contract + multi-domain), no hard-gate → standard. Not small (>3 product files, 4 domains). |
| REPO FIT | PASS | Every insertion point confirmed with file:line in feasibility-gather.md; parity/regen tooling exists and runs green (below). |
| ASSUMPTIONS | PASS | The one blocking assumption (B1 structural field) was disproved by gather and the plan re-shaped to word-boundary before lock. |
| SMALLER PATH | PASS | Cannot go smaller: 4 distinct verbs across 4 domains + a guard; each is minimal (one verb / one flag / one fn). |
| PROOF SURFACE | PASS | Each cell verify is a runnable area test + the two managed-ledger checks; no full-suite substitute. |

## Feasibility matrix

| # | Assumption | Risk | Proof required | Evidence | Result |
|---|---|---|---|---|---|
| 1 | Propagation sequence yields byte-parity and is gated | MED | parity checks run & baseline-green; regen tools exist | `release_manifest --check` → "510 file(s) match" exit 0; `ledger_parity --check` → "matches" exit 0; `render_plugin_skill_trees.mjs` + `skills/bee-hive/scripts/onboard_bee.mjs` present | **PROVEN** |
| 2 | Guard can distinguish inline-eval from file import | MED | hook inspects full Bash command string | intake gate already fired on a `>` redirect this session (guard reads `toolInput.command`); `checkGitBashCommand` shape inspects content (guards.mjs:391+) | **PROVEN feasible** — negative-control is a test-design task |
| 3 | `--cell si-1` word-boundary excludes `si-10` | LOW | regex probe | `/(?<![\w-])si-1(?![\w-])/` and `/\bsi-1\b/` both: match "si-1", reject "si-10" | **PROVEN** |
| 4 | Reader sees both `kind:` and `type:` friction/finding rows | LOW-MED | both schemas exist in store | gather Q3: legacy `kind:` (backlog.jsonl:38-41) + current `type:` (bee.mjs:2825-33) | **PROVEN** |
| 5 | `--show` read-only doesn't trip write-verb validation | LOW | early read-only branch before `--feature/--areas` validation | by construction; reviewer confirming against the verb code | PROVEN by construction (reviewer verifying) |
| sched | No dependency cycle; serial wave shape | LOW | `cells schedule` | waves = [sqs-a, sqs-b1, sqs-b2, sqs-b3], `cycles: []` | **PROVEN** |

## Notes for execution
- **Word-boundary regex:** prefer `(?<![\w-])<id>(?![\w-])` over bare `\b…\b` for hyphenated ids (both pass the probe; the lookbehind form is unambiguous around `-`).
- **Propagation order (must run inside each cell):** `render_plugin_skill_trees.mjs` → `onboard_bee.mjs --apply` → `release_manifest.mjs --write`, then verify includes `release_manifest --check` + `ledger_parity --check`.
- **Red-first (JUDGE advisory):** each behavior cell must capture failing-test output before the fix.

## Plan-checker + cell review (bee-review, read-only) — folded

Baseline fully green (all four verify test files + both `--check` + write-guard test).

| Sev | Cell | Finding | Resolution |
|---|---|---|---|
| **BLOCKER** | sqs-a | Bash `tool_input.command` scaffold lives in `hooks/test_write_guard.mjs`, not `test_hook_contracts.mjs` (rooted in a wrong gather Q5 ref). | **FIXED** — cell files+verify+artifacts repointed to `test_write_guard.mjs`; gather Q5 corrected. |
| WARNING | sqs-b3 | `--show` trips `requireFlags` (bee.mjs:2345) unless the read-only branch returns **before** `rejectDryRun`/`requireFlags` (:2343). | **FIXED** — prohibition added: insert `--show` branch before rejectDryRun/requireFlags. Resolves plan open-Q2. |
| WARNING | sqs-a→b1 | Over-serial: sqs-a edits disjoint files; only the per-cell regen of `release-manifest.json` forces the a→b1 link. | **ACKNOWLEDGED, kept serial** — parallel sqs-a would race the shared regen artifact; the parallelism win isn't worth the collision risk (cli-ergonomics precedent). |
| MINOR | gather | Q5 named the wrong test bed. | **FIXED** in feasibility-gather.md. |

Verified clean: decision IDs exist (`.bee/decisions.jsonl:1443-1444`), every registry target real, sqs-b1 test home correct, guard fn genuinely new, all verifies runnable+green. Plan open-Q2/Q3 resolved by code. No BLOCKER remains.

**Verdict: READY.**
