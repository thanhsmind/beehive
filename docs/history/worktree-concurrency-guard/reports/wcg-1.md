# wcg-1 — shared nested/companion checkout detection primitive

**Status:** [DONE]

**Outcome:** Added the exported primitive `isSharedNestedCheckoutTarget(root, targetPath, opts)` to `guards.mjs` — concurrency-gated (`isConcurrentMode`) structural detection that flags a verified companion mount and a plain nested `.git` in a shared checkout, and excludes a `.gitmodules`-registered submodule (covering both directory-`.git` and file-`.git`/absorbed-gitdir shapes). Unwired per D1 — Epic 2/3 wires it into `bee-write-guard.mjs`/`bee.mjs`. Verify green.

**Files changed:**
- `guards.mjs` (canonical `skills/bee-hive/templates/lib/` + rendered `.agents/`, `.claude/`, `.claude-plugin/`, `.codex-plugin/` trees + vendored `.bee/bin/lib/`) — the new primitive and its private helpers.
- `hooks/test_write_guard.mjs` — 8 new rows (70-77): the 3 confirmed spike baselines as regression assertions plus the primitive's intended flag/no-flag behavior (concurrent vs. solo for D6).
- `.bee/onboarding.json` — managed-hash ledger regen (onboard `--apply`).
- `docs/history/codex-harness-hardening/release-manifest.json` — the 4 guards.mjs content-hash entries only (mode-drift noise excluded, per the cell's `regen_obligation_ack`).

**Verify:** `node --test hooks/test_write_guard.mjs && node scripts/ledger_parity.mjs --check` → exit 0 (tests 1/pass 1/fail 0; ledger matches). Red-first proven: before the primitive existed, the test import failed with `SyntaxError: does not provide an export named isSharedNestedCheckoutTarget`.

**Deviations / friction:** the cell's file list named only the vendored `.bee/bin/lib/guards.mjs`; guards.mjs is canonical across 5 plugin trees and the onboarding ledger, so the edit was applied at the canonical source and propagated via the sanctioned regen (render + onboard `--apply`, repo-scoped). Recorded as friction for planning. Full trace, deviations, and evidence: `.bee/cells/wcg-1.json`.
