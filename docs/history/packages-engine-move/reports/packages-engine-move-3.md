# packages-engine-move-3 — report

**[DONE]**

Dispatcher-level unknown-flag rejection lands: `main()` in `bee.mjs` now
rejects, on STDERR with exit 1, any parsed flag absent from the invoked
verb's own registry schema — firing after `validate()` and strictly
before every handler dispatch, so it also wins the race for the two
pre-existing bespoke per-handler loops (`cells update`, `state worker
prune`), which stay in place unchanged. Corrected the C7 premise along the
way: `bee capture add --text x` was never a silent no-op — it already
exited 1 via `requireFlag('outcome')` — the real defect was message
quality (the refusal never named `--text`) plus an orchestrator habit of
reading only the last stderr line; logged as its own decision so the
corrected framing survives this cell's own trace aging out.

Declared the plan's 7 known registry gaps (`cells.claim`/`claim-next
--isolate`, `state.gate --owner`, `state.start-feature --isolate`,
`config.get`/`set`/`unset --local`) plus 5 more discovered live via the
targeted suite going red once the central check went in: `cells.verify`,
`cells.cap`, `cells.block`, `cells.unclaim`, `cells.reopen` all read
`session-id`/`force-ownership` indirectly through the shared
`ownershipFlags()` helper, never declared in their own schemas — the
validator was never loosened, every gap was declared instead.

Updated the now-stale comment at `validate-args.mjs:90` to point at the
new dispatcher-level check. Added one red-first regression row to the
existing DB3 section of `test_bee_cli.mjs` (no new test file); full
`test_bee_cli.mjs` red before the fix (295 passed, 1 failed, the exact
expected failure), green after (296 passed). Ran the regen chain
(`render_plugin_skill_trees`, self-onboard `--apply`, `release_manifest
--write`, `impact_registry --write`).

Files touched: see `.bee/cells/packages-engine-move-3.json`
(`trace.files_changed`) for the complete list — `packages/bee/bee.mjs`,
`packages/bee/lib/command-registry.mjs`, `packages/bee/lib/validate-args.mjs`,
`packages/bee/tests/test_bee_cli.mjs`, the release manifest, and the
regenerated `.bee/bin/*` mirrors + `.bee/onboarding.json`.

Full trace and verification evidence: `.bee/cells/packages-engine-move-3.json`.

No Advisor Consults on this claim.
