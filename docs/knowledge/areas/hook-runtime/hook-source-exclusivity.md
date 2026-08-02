---
type: bee.area
title: Hook Runtime — exactly one active hook source per installation
description: "Why an installation runs the package projection or the project fallback but never both, how each transition proves the other source inactive before removing anything, and what survives every transition untouched."
timestamp: 2026-07-24
bee:
  id: hook-runtime-hook-source-exclusivity
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: ["codex-hook-state-parity D1-D3, D8-D13", cf511ff3 (installed plugin package is authoritative; source arbitration and cleanup are proof-gated), "i54-closeout D5, D9"]
  sources: ["codex-hook-state-parity cells 2, 3, 5 (paired Codex subagent audit, package authority, exclusive hook-source arbitration, and fresh-host handler delivery; capped traces and reports, 2026-07-16)", "docs/specs/hook-runtime.md#B14", "docs/specs/hook-runtime.md#R15", "docs/specs/hook-runtime.md#R16", "docs/specs/hook-runtime.md#P11", "docs/specs/hook-runtime.md#P12", "i54-closeout cell i54-closeout-5 (doctor hook_sources both-present regression test; trace in .bee/cells/, 2026-07-24)"]
  authoritative_for: "hook-runtime: exclusive arbitration between the package and project hook sources"
---

# Hook Runtime — exactly one active hook source per installation

Two delivery locations exist, so exactly one of them must be active at a time —
otherwise a checkpoint fires twice, or the one that fires is not the one anybody
inspected. Arbitration between them is proof-gated in both directions, and it is
scoped: only entries the catalog recognises as this workflow's own are ever
removed.

## Behaviors & Operations

**B14 — Exactly one bee hook source is active.** An installation selects the
package projection or the project fallback. Package activation is proved before
recognized fallback entries are removed; fallback activation is proved only
after the package is known inactive. User and foreign hook entries survive.

## Business Rules

- R15 — Codex plugin delivery loads
  the catalog-derived hook projection from the installed package. The checked-in
  project checkpoint file is a development and repo-fallback projection only;
  release/reinstall proof exercises the installed package, and project fallback
  success never substitutes for that package proof (codex-hook-state-parity
  D9/D13; decision cf511ff3).

- R16 — Plugin and project hooks are
  mutually exclusive bee sources. Migration to plugin delivery removes only
  catalog-recognized bee entries after installed-package integrity is proven;
  migration to project fallback first proves the plugin inactive. User hook entries
  survive both transitions unchanged (codex-hook-state-parity D10–D13; decision
  cf511ff3).

## Edge Cases Settled

- **Doctor's `hook_sources` row names the dual-source state explicitly instead
  of staying silent about it.** When both `packages/bee/hooks/hooks.json` (plugin
  projection) and `.codex/hooks.json` (repo fallback) are present on disk at
  once, the row's evidence text says so in plain terms: two hook sources
  exist, exactly-one-active is the law this concept states (B14), and the
  current premise underneath that law — plugin hooks are not-observed on the
  probed codex version (capability matrix row B1) — must be re-proved whenever
  the probed codex version changes. The row also distinguishes
  `packages/bee/hooks/claude-hooks.json` (the Claude manifest `plugin.json` declares) from
  `packages/bee/hooks/hooks.json` (the Codex projection) by name, so a reader is never left
  guessing which "hooks" file a warning means. None of this changes the row's
  verdict semantics: `active: unknown` stays honest (which source actually
  loaded has no runtime surface to check), and the row remains informational
  — never blocking, never degrading — exactly as before (i54-closeout D5, D9).

## Pointers (implementation)

- Doctor row: `doctorHookSourcesCodex` in `the bee binary`.
  Both-present regression test: `packages/bee/tests/test_bee_cli.mjs`
  (single-source baseline + both-present fixture). Evidence:
  `.bee/cells/i54-closeout-5.json`.

- Parity evidence: `.bee/cells/codex-hook-state-parity-{2,3}.json` and
  `docs/history/codex-hook-state-parity/reports/`.

- Evidence: `docs/history/codex-runtime-parity/` (red-baseline.md, cell reports);
  commits `d1777ed`, `5458b34`, `cf1ce51`, `a30fb0c`, `f0860ac`, `7499a71`.

## Open Gaps

- Fixture and installed-package proofs are green. Live proof that Codex loads
  the package-delivered projection in a real trusted session remains
  outstanding because this environment cannot write the user Codex home.
