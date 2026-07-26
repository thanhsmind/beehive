# pic-1

**[DONE]** — addPbi's fold-read + id-generate-check-regenerate + append now run as one critical section under a new `backlog-pbi` store lock (bounded 16-attempt regenerate loop, typed `PbiIdGenerationExhaustedError`); id shape and the fold's first-add-wins backstop are unchanged.

Files touched: `packages/bee/lib/backlog.mjs`, `packages/bee/tests/test_backlog_capture.mjs`, `.bee/bin/lib/backlog.mjs` (mirror, regenerated), `.bee/onboarding.json`, `docs/history/codex-harness-hardening/release-manifest.json`.

Full trace/evidence: `.bee/cells/pic-1.json`.
