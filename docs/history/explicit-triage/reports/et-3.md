# et-3 — Slice test: route record net behavior (D5)

[DONE]

Outcome: Added the deep behavioral net for `state route` (assertions a-e) to
`packages/bee/tests/test_bee_cli.mjs`, extending the owning suite (never
forking a new one). 11 new checks, each against its own hermetic temp-repo
fixture built through the real dispatcher — never live `.bee/`:

- (b) bad `--class`/`--lane`/`--flags`/negative `--files` each typed-refused,
  nothing written (verified via `--show`).
- (a) valid `--set` round-trips through `--show`, `status --json`, and the
  underlying workflow record (belt-and-suspenders, D1).
- (d) preamble `- Route: ...` line absent before any `--set`, exact D2 format
  present after.
- (e) a second `--set` (re-lane demotion) rewrites the SAME workflow record —
  lane/flags/files change, `updated_at` moves, no second record.
- (c) `cells claim` warns exactly once on stderr for a route-less feature
  (still claims) and stays silent once a route is recorded.

Files touched: `packages/bee/tests/test_bee_cli.mjs`

Verify: `node packages/bee/tests/test_bee_cli.mjs && node packages/bee/tests/test_state.mjs`
— 329 passed / 44 passed, 0 failed. Full trace/evidence:
`.bee/cells/et-3.json`.

Commit: (recorded in git log, message carries `[et-3]`)
