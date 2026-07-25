---
type: bee.area
title: Hook Runtime — how the guardrails are inspected and proven
description: "The read-only doctor command's evidence-carrying rows and three-state verdict, the host-topology and deep-inventory checks behind them, and the suites, isolated runners and real-CLI canary that prove the guard chain rather than assert it."
timestamp: 2026-07-22
bee:
  id: hook-runtime-health-checks-and-proof-surfaces
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: [a83a3613 (shared isolated runner for nested Node entrypoints; real Git/Bash/Codex integration remains external), "codex-runtime-parity D1, D2"]
  sources: ["codex-sandbox-baseline cells codex-sandbox-baseline-2/codex-sandbox-baseline-4 (nested test entrypoints use the shared isolated runner; external integration keeps real status/output grading, 2026-07-16)", "codex-runtime-parity Safety foundation — cells codex-parity-2, 2b, 3, 4 (traces in .bee/cells/), reports in docs/history/codex-runtime-parity/reports/", codex-native-runtime-v2 cnr2-13/cnr2-14 (read-only doctor rows; conformance suite over real binaries with negative-state assertions), gh22-completion g22-3/g22-4/g22-6 (three-state doctor verdict and static attestation; deep skill-inventory audit; scripted real-CLI canary), pre-162-fixes p162-1 (doctor resolves hook handlers at host topology), "docs/specs/hook-runtime.md#E2", "docs/specs/hook-runtime.md#E3", "docs/specs/hook-runtime.md#E9", "docs/specs/hook-runtime.md#E10", "docs/specs/hook-runtime.md#E11", "docs/specs/hook-runtime.md#E12", "docs/specs/hook-runtime.md#E13", "docs/specs/hook-runtime.md#E14", "docs/specs/hook-runtime.md#P1", "docs/specs/hook-runtime.md#P2", "docs/specs/hook-runtime.md#P9"]
  authoritative_for: "hook-runtime: health reporting and the proof surfaces behind the guardrails"
---

# Hook Runtime — how the guardrails are inspected and proven

A guardrail nobody can inspect is a guardrail nobody should trust. Two audiences
need that inspection and they get different surfaces for it: a human owner runs
a read-only health command whose every row carries its own evidence and whose
verdict distinguishes "mechanically broken" from "structurally unknowable", and
the automated chain drives the real guard and command binaries as subprocesses
against isolated fixtures, asserting that a denied action changed nothing.

## Edge Cases Settled

- Regenerating the RED-baseline evidence report is timestamp-stable in content;
  only noise fields differ.

- Simultaneously requesting the evidence-baseline and catalog-only test modes
  is rejected as contradictory.

- A read-only, fail-closed doctor command reports per-runtime health: every row
  carries value + evidence + ok/warn/unknown/unsupported; capability verdicts
  are version-scoped; the command performs zero writes, including the
  dispatcher's manifest-hash cache (codex-native-runtime-v2, cnr2-13).

- Doctor's overall verdict is THREE-STATE (gh22-completion g22-3, supersedes
  the binary ready/not_ready): `blocked` = a mechanical blocking row is
  not-ok (hooks file missing, baseline drift, handlers unresolvable, skills
  missing/drifted); `degraded` = mechanical green but trust surfaces the
  runtime cannot expose are structurally unknown (the user is pointed at
  /hooks); `ready` = mechanical green plus, on the second runtime, a VALID
  static attestation. `bee doctor attest --runtime codex` records
  {hooks-file sha256, CLI version, repo identity} into gitignored runtime
  state; validity = all three match live state (no liveness leg — the runtime
  exposes no hook-fire event surface, and the reason text says so honestly);
  any drift names its reason (hash_changed/version_changed/identity_changed/
  no_attestation) and the verdict falls back to degraded. Trust wording is
  probe-version-scoped: a CLI version other than the probed one reads
  `unprobed_version` (re-probe suggested), never a blanket "unsupported".

- Doctor resolves hook handlers at HOST topology: each handler filename is
  checked at both `.bee/bin/hooks/` and `hooks/` (dual-location, evidence
  names which resolved) — a normal host repo without the dev repo's root
  hooks/ dir is judged correctly (pre-162-fixes p162-1).

- Skill install checks are a DEEP INVENTORY audit (gh22-completion g22-4):
  the render sidecar is `bee-render/2` `{schema, target_runtime,
  skills:[{name, sha256}]}` with per-skill digests over the rendered
  per-runtime bytes, single-sourced beside the renderer; doctor recomputes
  and names missing/stray/drifted skills (blocking); a legacy `bee-render/1`
  sidecar degrades to a non-blocking "inventory unavailable" warn.

- A scripted canary drives the REAL second-runtime CLI against a throwaway
  fixture (isolated CODEX_HOME so trust writes never touch the user's real
  config; per-hook trust bypass does NOT bypass per-project trust — both must
  be seeded in the fixture): session-init fires end-to-end, and the installed
  guard chain is proven via synthetic envelopes through the installed hook
  files (spawn deny/allow, state-sync, intake deny). Skip-guarded; nightly /
  manual only, never a push gate (gh22-completion g22-6).

- A scripted conformance suite drives the guard and CLI binaries as
  subprocesses against isolated fixtures with negative-state assertions
  (denied action changed nothing); agent-behavior scenarios live in a manual
  checklist with named metrics and are never auto-asserted
  (codex-native-runtime-v2, cnr2-14).

## Pointers (implementation)

- `scripts/lib/run-module-worker.mjs` — shared isolated runner for nested test
  entrypoints used by the hook, command, onboarding, and metadata suites.

- `scripts/tests/test_portable_paths.mjs` and `hooks/test_hook_contracts.mjs` — real
  external integration remains external; assertions grade concrete exit status,
  stdout, and stderr even when the execution environment adds a launch warning.

- Suites: `hooks/test_hook_contracts.mjs` (modes: default, `--baseline`,
  `--catalog-only`, `--repo-route-only`), `hooks/test_write_guard.mjs`,
  `hooks/test_model_guard.mjs`; parity check in
  `skills/bee-hive/scripts/tests/test_onboard_bee.mjs`.

## Open Gaps

- Fixture and installed-package proofs are green. Live proof that Codex loads
  the package-delivered projection in a real trusted session remains
  outstanding because this environment cannot write the user Codex home.
