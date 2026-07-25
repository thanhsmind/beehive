# multisession-native-6

[DONE] `startFeature` (state.mjs) now creates a workflow record via workflow-store on every call, with preconditions re-scoped to live workflow records, F5 per-feature HANDOFF scoping on the default path, and C1's idempotent legacy-to-workflow seed.

Files touched: `skills/bee-hive/templates/lib/state.mjs`, `skills/bee-hive/templates/tests/test_state.mjs`, `skills/bee-hive/templates/tests/test_cli_state.mjs`, plus regen artifacts (plugin skill-tree mirrors, `.bee/bin/lib/state.mjs`, `.bee/onboarding.json`, `docs/history/codex-harness-hardening/release-manifest.json`, `scripts/impact-registry.json`).

Full trace/evidence: `.bee/cells/multisession-native-6.json`.

Commit: f4fe163.
