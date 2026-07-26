# i54-closeout-8 — report

**Status:** [DONE]
**Outcome:** `PROBED_CODEX_VERSION` bumped 0.144.4 -> 0.145.0 (D8), backed by the
validating canary's post-fix full-green rerun (`reports/validation-canary.md`
§4). Version-scoped capability rows (F1 trust rows, A1/A2 `custom_agents`)
auto-update via the shared template constant; row C2 (`permission_mode`, not
wired to the constant, not exercised by this canary) left untouched per R18.
Regen chain applied; doctor no longer reports `unprobed_version` for the live
0.145.0 CLI.

**Files touched:** `skills/bee-hive/templates/bee.mjs`,
`skills/bee-hive/templates/tests/test_bee_cli.mjs`,
`scripts/test_installers_e2e.mjs`,
`docs/history/codex-harness-hardening/release-manifest.json`, plus the
regen-synced vendored copies (`.bee/bin/bee.mjs`, `.agents/`, `.claude/`,
`.claude-plugin/`, `.codex-plugin/` skill trees) and `.bee/onboarding.json`.

**Note for the orchestrator:** `cells cap` succeeded (status `capped`,
`verify_passed: true`) but printed `JUDGE_STANDARD_INSUFFICIENT` — the D3
`red_failure_evidence` floor was not enforced because the evidence went
through the `deliberate_exceptions` door. Flagging this explicitly since it
may need the goal-check judge's attention; see the trace's
`verification_evidence.red_failure_evidence` for the full reasoning (a local
reversion re-test was environmentally inconclusive on this dev machine, whose
live codex happens to already be 0.145.0 — root-caused via inspection, not
swept under the rug).

**Full trace/evidence:** `.bee/cells/i54-closeout-8.json`

**Commit:** `948770b` — "feat(i54-closeout-8): bump PROBED_CODEX_VERSION to
live-probed 0.145.0 (D8)"

**Reservations:** released (4 paths + 4 cross-worktree holds).
