# dch-9 — Derive the model-guard fixture's model names from the config it copies

**[DONE]**

`packages/bee/hooks/test_model_guard.mjs` now derives every model name it
asserts on from the same `.bee/config.json` its enabled fixture copies, instead
of hardcoding `"sonnet"` at eight-plus sites as "a model that is configured".
The suite was live-red before the fix (nine rows) because the owner had
repointed `models.claude.generation`; the guard was correct and the literal was
not. No assertion weakened (115 `check()` calls before and after), no allowlist
added to the guard, `.bee/config.json` untouched.

Files touched:

- `packages/bee/hooks/test_model_guard.mjs`

Commit: `2ee3754628ea4ec17d2b1c1af1545f38052d1ad3`

Both-directions proof: green at `opus-4-5-20251101` (live), `sonnet`, `opus`
(collides with the review slot), and an all-slots-identical config (exercises
the mismatch fallback). Config restored byte-exact, sha256 verified.

Full trace, verify command and output: [`.bee/cells/dch-9.json`](../../../../.bee/cells/dch-9.json)
