---
type: bee.pattern
title: Hardcoded fixture file-lists rot silently — and fail-open makes rot look like PASS
description: "Two independent hand-kept fixture lists rotted silently, and the hook’s fail-open turned the resulting crash into a universal PASS"
tags: [failure, test-fixtures, fail-open, hooks]
timestamp: 2026-07-14
bee:
  id: pattern-20260714-hardcoded-fixture-file-lists-rot-silently
  lifecycle: active
  areas: [hook-runtime]
  sources: ["docs/history/learnings/critical-patterns.md#PAT7", "original feature: shim-retire", docs/history/learnings/20260714-shim-retire.md]
  polarity: pitfall
  critical: true
---

# Hardcoded fixture file-lists rot silently — and fail-open makes rot look like PASS

Two independent test fixtures each hand-enumerated "which lib files to vendor into the
sandbox"; both had rotted (missing `claims.mjs`), the hook crashed at import inside the
fixture, and the hook's fail-open turned the crash into universal green. When a fixture
must mirror a runtime file set, derive it with `readdirSync` of the real directory —
never a hand-kept list. And a fail-open guard's test suite needs at least one
sentinel-deny case, so universal fail-open can never read as all-pass.

**Recurred 2026-07-15 (p2-1):** `test_onboard_bee.mjs`'s fixture launcher hand-wrote
exactly `commands_detect.mjs` + `state.mjs` into `templates/lib`. The moment onboard
gained one new import (`fsutil` for the shared `hashFile`), every fresh-install test
crashed with `exit 1 status undefined` (the spawned launcher couldn't resolve the
missing dep). **Adding an import to any module a fixture copies is a hand-list
tripwire** — fixed by vendoring the whole real `templates/lib` via `readdirSync`.

**Full entry:** docs/history/learnings/20260714-shim-retire.md
