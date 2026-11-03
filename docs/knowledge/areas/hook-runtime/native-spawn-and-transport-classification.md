---
type: bee.area
title: Hook Runtime — the native spawn checkpoint and transport classification
description: "How the second runtime's spawn checkpoint judges role markers and settings, why dispatch prepare classifies 0.154.0 as native_hook_input_opaque and refuses unverifiable native cell dispatch while model guard denies unmarked spawn as codex-spawn-unmarked (exit 2), how capability probe verdicts are recorded and invalidated, and the cross-build regression that proved the version leg load-bearing."
timestamp: 2026-07-22
bee:
  id: hook-runtime-native-spawn-and-transport-classification
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: ["codex-native-transport D3-D5 (3ceba8f5, D3a c0cba64e, Δ2-amended 760e9b05)", "350f1e82 (codex-native-transport cnt-4 rescope — override-field route-check deferred to a documented pass-through-open gap, pending observed evidence)"]
  sources: ["codex-native-transport cells cnt-2/cnt-3 (capability classification + probe record + doctor unlock naming; dispatch-guard marker extension; traces in .bee/cells/, reports docs/history/codex-native-transport/reports/, 2026-07-19)", "codex-native-transport cells cnt-4/cnt-5 (override-field route-check rescoped to a documented pass-through-open gap pending observed evidence; capability probe's live leg with isolation independently verified and a cross-build regression observed; traces in .bee/cells/, reports docs/history/codex-native-transport/reports/cnt-4.md and reports/probe-evidence.md, 2026-07-19)", "docs/specs/hook-runtime.md#B19", "docs/specs/hook-runtime.md#R18", "docs/specs/hook-runtime.md#E15", "docs/specs/hook-runtime.md#E16", "docs/specs/hook-runtime.md#P14", "docs/specs/hook-runtime.md#P15", "docs/specs/hook-runtime.md#P16"]
  authoritative_for: "hook-runtime: native spawn override pass-through and native-transport classification"
---

# Hook Runtime — the native spawn checkpoint and transport classification

The dispatch guard judges what it has observed. It validates role markers, configured
models, reasoning effort, and fork turns on native spawn requests. When host delivery
conceals the marker behind an opaque payload, `evaluate_codex_spawn` in `model_guard.rs`
cannot see the marker and returns transport `codex-spawn-unmarked` with exit 2. In turn,
`installed_native_transport_classification` in `prepare.rs` measures the client version,
classifies it as `native_hook_input_opaque`, and `bee dispatch prepare` refuses
unverifiable native cell dispatch rather than allowing an unverified run.
Capability classification and probe records govern whether verified
overrides can run natively or must route through explicit external CLI or herding
transports.

## Data Dictionary

| Element | Meaning |
|---|---|
| native-transport classification | The verdict `installed_native_transport_classification` in `prepare.rs` or a capability probe assigns a second-runtime client from observed evidence: `native_model_override` (a native per-agent model override is confirmed accepted), `native_budget_only` (default — no override proven), `external_cli_only` (the base spawn transport itself is confirmed off), or `native_hook_input_opaque` (the host delivers opaque tool messages concealing role markers, so dispatch prepare refuses native cell dispatch). Unknown or absent evidence reads `native_budget_only` — the native-override transport stays inert until proven (codex-native-transport D3). |
| native-transport probe record | A separate, gitignored, version- and configuration-scoped record — distinct from doctor-attest, whose legs cannot see a feature-flag change — holding the classification and the evidence it was derived from. Independent validity legs invalidate a stale verdict back to `native_budget_only` and name the reason: no record on disk, a repository-identity mismatch, a version mismatch, a corrupted configuration-scope hash, or a live re-check that disagrees with the recorded configuration scope (codex-native-transport D3, Δ2-amended). |

## Behaviors & Operations

**B19 — The Codex native spawn checkpoint enforces configured settings and refuses unverifiable overrides (codex-parity D1, cpc-2).**
The pre-spawn guard validates the role marker, model, reasoning_effort, and
fork_turns against the configured route. Full-history forks cannot carry
overrides; escalated roles preserve the parent model; read-only roles cannot use
native spawn because the callable schema has no filesystem sandbox field. On
Codex 0.154.0, native input arrives with an opaque message body: `evaluate_codex_spawn`
in `model_guard.rs` cannot see the required marker and denies unmarked spawn as
transport `codex-spawn-unmarked` (exit 2). At preparation time,
`installed_native_transport_classification` in `prepare.rs` classifies the version as
`native_hook_input_opaque`, and `bee dispatch prepare` refuses unverifiable native
cell dispatch, directing callers to configured herding/CLI routes.

## Business Rules

- R18 — The Codex native spawn checkpoint governs model, effort, and fork
  parameters against configured roles. Unverifiable native dispatch is refused
  at prepare time (`native_hook_input_opaque`), and unmarked native spawn calls
  are denied by `model-guard` as `codex-spawn-unmarked` (exit 2); read-only jobs
  require an enforceable execution boundary; full-history forks and escalated roles
  cannot carry model overrides (B19; codex-parity D1).
- R18a — The Codex onboarding note, written on fresh and refreshed onboarding,
  tells the user the current truth about Codex dispatch: a role that nothing
  configures resolves to no model and is held to budget in the prompt; roles
  and transports are configured in the team block for the Codex runtime;
  non-cell dispatches fall back to a read-only Codex command-line run; and
  native dispatch depends on runtime capability, with no proof of the model it
  really used. The old claim that Codex has no per-agent model selection by
  design is gone, and a regression test refuses its return
  (codex-reliability-closeout crc-3).

## Edge Cases Settled

- A native-transport capability probe is version- and configuration-scoped and
  lives in its own gitignored record, separate from doctor-attest. Doctor
  gains one purely informational row that only NAMES the unlock — the
  feature flag plus the metadata-visibility flag — when the client is not yet
  confirmed and the installed binary ships the flag; the row is never
  blocking and never degrading, and bee never flips the flag in the user's
  real configuration itself (canary probe isolation is scoped to `CANARY_CODEX_HOME`
  and `TMPDIR`; note that PATH probes can reach the host `mise` wrapper, so user-global
  configuration is not claimed to be untouched by all test automation) (codex-native-transport D3/D4).
  The probe's live check leg runs entirely inside that isolated per-run home,
  records whatever it observes into both the scoped machine record and a human-readable
  evidence report, and a separate offline self-check exercises the same isolation
  invariant without needing the client installed at all (codex-native-transport D3/D4).

- A capability probe's live check observed a real cross-build regression, not
  a hypothetical one: an override surface confirmed accepted on one client
  version was refused outright on the very next patch version, with no
  advance signal available to the workflow. This is exactly the scenario the
  classification's version-validity leg exists to catch, and the disagreement
  between the two live runs proves that leg load-bearing rather than
  defensive programming (codex-native-transport D3).

## Pointers (implementation)

- Native transport classification and dispatch preparation: `installed_native_transport_classification` and `native_transport_classification_with_cmd` in `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`.
- Native spawn guard: `evaluate_codex_spawn` (returns `codex-spawn-unmarked` with exit 2 when markers are absent or hidden) in `packages/bee-rs/crates/bee/src/hooks/model_guard.rs`.
- Doctor attestation: `read_attestation` and `run_attest` in `packages/bee-rs/crates/bee/src/doctor.rs`.
- Historical Node implementation (historical evidence):
  `classifyNativeTransport` in `packages/bee/lib/dispatch-guard.mjs`,
  `scripts/tests/test_native_probe.mjs`, and `scripts/canary_codex.mjs`.
- Evidence: `.bee/cells/cnt-2.json`, `.bee/cells/cnt-3.json`, `.bee/cells/cnt-4.json`,
  `.bee/cells/cnt-5.json`, `docs/history/codex-native-transport/`.

## Open Gaps

- The Codex 0.154.0 hook interface delivers `collaborationspawn_agent` with an
  opaque message body to `PreToolUse`. Native cell dispatches cannot verify role
  markers directly through this interface, so enforceable execution relies on the
  CLI read-only sandbox or configured herding/CLI transports.
