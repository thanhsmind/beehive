# i54-closeout-7 — Lane-write ergonomics: resolveMutationTarget auto-resolves the calling session's bound lane

Status: [DONE]

Outcome: write-path lane resolution is now symmetric with the read path (D7). `resolveMutationTarget` resolves explicit `--lane` > calling session's bound lane (identity self-resolved at operation moment, B22) > default record, across all four callers — `state set`, `state gate`, `state scribing-run`, and `state advisor-ref record` (plan-check W3's fourth caller, covered deliberately). `--no-lane` forces the default record from a bound session; `--no-lane`+`--lane` is refused. Missing/corrupt bound lane refuses loudly with zero writes (B12/B13 — never a silent fall-back). `--owner` stays checked against the SELECTED record's pre-mutation phase (fsh-4), and `set --feature` is refused on the auto-resolved lane target too (identity-rename guard extended). Unbound sessions: zero behavior change (proved by a row that passes on both pre- and post-change code). Registry descriptions of the four verbs state the new resolution order and document `--no-lane`.

Verify: `node skills/bee-hive/templates/tests/test_cli_state.mjs && node scripts/release_manifest.mjs --check` — 78 passed / 0 failed, 510 manifest files match. Red-first: 74/4 against pre-change bee.mjs via git stash. Adjacent registry suite test_bee_cli.mjs: 287/0.

Files touched:

- skills/bee-hive/templates/bee.mjs (canonical)
- skills/bee-hive/templates/lib/command-registry.mjs (four verb descriptions + `--no-lane` parameter)
- skills/bee-hive/templates/tests/test_cli_state.mjs (5 new D7 rows)
- .bee/bin/bee.mjs, .bee/bin/lib/command-registry.mjs (vendored via onboard --apply, never hand-edited)
- docs/history/codex-harness-hardening/release-manifest.json (regen --write)
- plugin projections (.agents/.claude/.claude-plugin/.codex-plugin skill trees, regen)

Deviations (4, recorded in the trace): --feature guard extended to auto-resolved lane targets; `--no-lane` documented without a new registry examples[] row (examples are executed by test_bee_cli.mjs, outside this cell's files[] — behavior coverage lives in test_cli_state.mjs); `--no-lane` added to FLAG_ALONE_BOOLEANS; an implementation-time variable shadow (`target` phase string vs mutation target in handleStateSet) renamed to `targetPhase` after si-1 lane-close rows caught it red.

Commit: ca7f0f2 (one commit, cell id in message). Full trace and verification evidence: `.bee/cells/i54-closeout-7.json`.
