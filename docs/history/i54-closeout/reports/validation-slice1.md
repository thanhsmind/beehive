# validation — i54-closeout slice 1 (9 cells)

## Reality gate

| Check | Verdict | Evidence |
|---|---|---|
| MODE FIT | PASS | standard; 3 flags (multi-domain, covered-contract change, external systems) — mode-gate record in plan.md; no hard-gate flag trips |
| REPO FIT | PASS | every cited anchor spot-verified by two independent reviewers (dispatch-prepare :389-397, guard :193, runOne :535, inject :219, bee.mjs :4505/:1911/:4788, resolveMutationTarget callers) |
| ASSUMPTIONS | PASS | the one MEDIUM unknown (real spawn_agent schema) resolved by three corroborating live probes → reports/validation-canary.md |
| SMALLER PATH | PASS | generator descoped to a consistency test (D6), AGENTS.md restructure descoped (D11), herding kept to an adapter seam (D4) |
| PROOF SURFACE | PASS | per-cell scoped verifies all runnable (flags `--only`, `--impacted-from-git --level 1`, `release_manifest --check`, `--probe-selftest` verified to exist); full canary rerun gates cell 8 |

## Feasibility matrix

| Assumption | Risk | Proof required | Evidence | Result |
|---|---|---|---|---|
| spawn_agent schema on codex 0.145.0 requires task_name, has no agent_type | HIGH | live probe | validation-canary.md §2 (400 "reserved…configured schema", `native_budget_only`) + §3 self-inspection `required:[message,task_name]`, corroborated by swarming-reference.md:134 and #54's independent reading | PROVEN |
| model-override spawn envelope unusable on 0.145.0 | MED | live probe | `.bee/native-transport-probe.json` `override_spawn_accepted:false` | PROVEN (R18: no row may claim it) |
| installed hook chain works on fresh installs | MED | canary | P1–P4 PASS; P5 FAIL = real vendoring regression (tokenize-command.mjs missing from HOOK_FILENAMES) | BUG FOUND → cell i54-closeout-9; cell 8 deps on it |
| verify runner flags used by cell verifies exist | LOW | file inspection | run_verify.mjs `--only` :497, `--impacted-from-git` :630, `--level` :641; release_manifest `--check` :309-318 | PROVEN |
| suite registry accepts new scripts/test_*.mjs suites | LOW | precedent | test_installers_e2e registration pattern (cell reviewer verified) | PROVEN |
| schedule has no cycles; wave shape sane | LOW | `bee cells schedule` | zero cycles/unsatisfiable deps; scheduler serializes the bee.mjs-sharing cells (3, 5, 7, 8 in separate waves) | PROVEN |
| codex CLI present for live legs | LOW | command | `codex-cli 0.145.0` on PATH | PROVEN |

## Cell review (cold pickup, review-tier subagent)

PASS: cells 2, 3, 4, 5, 6, 7. CRITICAL on 1 and 8 — both consumed
`reports/validation-canary.md`, which did not yet exist at review time; the
report has since been written (this phase's deliverable), closing both flags.
MINOR notes recorded: cell 3 shares `templates/bee.mjs` with 5/7/8 without an
explicit dep — mitigated by the scheduler's wave serialization and serial
dispatch; cell 4's SKILL.md line anchors unverified (files + targets exist).

## Plan checker (adversarial, review-tier subagent) — iteration 1, all repaired

2 BLOCKER + 4 WARNING, every one repaired in the cells before Gate 3:

- **B1** cell 3 targeted rendered `AGENTS.md` instead of canonical
  `templates/AGENTS.block.md` (byte-identity test + own regen would clobber)
  → files/action repaired to the template.
- **B2** cell 3 shared `templates/bee.mjs` with 5→7→8 without a dep → cell 5
  now deps on 3 (chain 3→5→7→8→9-independent).
- **W1** manifest writers 1/3/4/5 partially parallel in the dep graph →
  mitigated: scheduler waves already serialize them AND dispatch is serial
  (one execution worker at a time).
- **W2** "register the suite" instruction was wrong — run_verify
  auto-discovers `scripts/test_*.mjs` (DISCOVERY_ROOTS :42-47, :315-334);
  manual EXTRA_SUITES would double-run → cells 2/6/9 corrected (confirm glob
  discovery; fixtures named to avoid the glob).
- **W3** `resolveMutationTarget` has 4 callers, not 3 (advisor-ref record
  :2635 was unnamed) → cell 7 now covers all four with a test row.
- **W4** literal `--budget 20000` assertions (test_misc.mjs:723 under a
  `mode:'small'` fixture; test_bee_cli.mjs:2827) will break → cell 3 names
  them for coherent update.

Checker's clean list: D1–D11 coverage complete, all anchors verified real,
dep chain shape correct, scope sane, cell 8 environment-feasible.

## Approval block

Verdict: **READY.** Slice: 9 cells, waves `[1,2,6] → 3 → 4 → 5 → 7 → 9 → 8`
per `bee cells schedule` (zero cycles). Gate 3 under `gate_bypass=total` →
auto-approved with audit decision; serial dispatch, one execution worker at a
time, one commit per cell.
