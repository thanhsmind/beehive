---
type: bee.area
title: Hook Runtime — the native spawn checkpoint and transport classification
description: "Why the second runtime's spawn checkpoint deliberately passes an override field through unjudged until a capability probe has observed it, how that probe's version- and configuration-scoped verdict is recorded and invalidated, and the cross-build regression that proved the version leg load-bearing."
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

The dispatch guard judges what it has observed. This concept is about what it has
deliberately chosen not to judge yet: a per-agent model, effort or fork-turn
override riding on a marker-anchored native spawn. Implementing that check
against a shape no client has ever actually sent would mean denying on an
assumption, so the check waits on evidence — and the evidence itself is a
scoped, invalidatable record rather than a remembered verdict.

## Data Dictionary

| Element | Meaning |
|---|---|
| native-transport classification | The three-way verdict a capability probe assigns a second-runtime client from observed evidence: `native_model_override` (a native per-agent model override is confirmed accepted), `native_budget_only` (today's default — no override proven), or `external_cli_only` (the base spawn transport itself is confirmed off). Unknown or absent evidence always reads `native_budget_only` — the native-override transport stays inert until proven (codex-native-transport D3). |
| native-transport probe record | A separate, gitignored, version- and configuration-scoped record — distinct from doctor-attest, whose legs cannot see a feature-flag change — holding the classification and the evidence it was derived from. Independent validity legs invalidate a stale verdict back to `native_budget_only` and name the reason: no record on disk, a repository-identity mismatch, a version mismatch, a corrupted configuration-scope hash, or a live re-check that disagrees with the recorded configuration scope (codex-native-transport D3, Δ2-amended). |

## Behaviors & Operations

**B19 — The Codex native spawn checkpoint deliberately does not judge an
override an anchored spawn carries.** When a marker-anchored native spawn
also names a per-agent model, effort level, or fork-turn count, the
checkpoint's decision is unchanged from a spawn without those fields: a valid
marker still allows, an unmarked message still denies, and the named fields
themselves are never read or compared against anything. This is a
deliberate defense-in-depth allow-hole, not an oversight — validating those
fields against the configured route was locked as a future rule, but no
observed client input has ever carried them into the checkpoint yet on any
version checked, so implementing the check now would mean denying against
assumed rather than observed shape. It stays this way until the capability
probe (native-transport probe record, above) observes that envelope on a
client version — only then does the route-check activate. Proven by an
allow/deny row pair: a marker-anchored spawn with override fields that
mismatch any plausible configured route is still allowed, unaffected; the
identical override fields on an unmarked message are still denied,
unaffected (codex-native-transport D6; decision 350f1e82).

## Business Rules

- R18 — The Codex native spawn checkpoint never judges a per-agent model,
  effort, or fork-turn override by name until the capability probe has
  observed that field arrive on some client version; until then, a
  marker-anchored spawn carrying such fields decides exactly as it would
  without them (B19; codex-native-transport D6, decision 350f1e82).

## Edge Cases Settled

- A native-transport capability probe is version- and configuration-scoped and
  lives in its own gitignored record, separate from doctor-attest. Doctor
  gains one purely informational row that only NAMES the unlock — the
  feature flag plus the metadata-visibility flag — when the client is not yet
  confirmed and the installed binary ships the flag; the row is never
  blocking and never degrading, and bee never flips the flag in the user's
  real configuration itself (a canary probe may only do so inside its own
  isolated per-run home) (codex-native-transport D3/D4). The probe's live
  check leg runs entirely inside that isolated per-run home — never the
  user's real configuration — and the isolation is independently verified
  byte-identical before and after each run, not merely asserted; it records
  whatever it observes, including a refused or absent outcome, into both the
  scoped machine record and a human-readable evidence report, and a separate
  offline self-check exercises the same isolation invariant without needing
  the client installed at all, so automated verification stays green when the
  client binary is absent (codex-native-transport D3/D4).

- A capability probe's live check observed a real cross-build regression, not
  a hypothetical one: an override surface confirmed accepted on one client
  version was refused outright on the very next patch version, with no
  advance signal available to the workflow. This is exactly the scenario the
  classification's version-validity leg exists to catch, and the disagreement
  between the two live runs proves that leg load-bearing rather than
  defensive programming (codex-native-transport D3).

## Pointers (implementation)

- Native-transport classification: `classifyNativeTransport(evidence)` (pure,
  `packages/bee/lib/dispatch-guard.mjs`). Probe record reader/
  writer and the D4 doctor row: `readNativeTransportClassification`,
  `writeNativeTransportProbe`, `doctorNativeTransportUnlock`
  (`the bee binary`, mirroring the doctor-attest pattern).
  Suite: `scripts/tests/test_native_probe.mjs`. Advisor-marker acceptance on the
  codex branch: `ANCHORED_CODEX_TIER_MARKER_RE` in `dispatch-guard.mjs`.
  Evidence: `.bee/cells/cnt-2.json`, `.bee/cells/cnt-3.json`,
  `docs/history/codex-native-transport/`.

- Override-field pass-through gap (B19): documented inline above
  `evaluateCodexSpawn` in `packages/bee/lib/dispatch-guard.mjs`
  (mirrored in `.bee/bin/lib/dispatch-guard.mjs`); canary rows
  `hooks/test_model_guard.mjs` rows 56-57. Evidence: `.bee/cells/cnt-4.json`,
  `docs/history/codex-native-transport/reports/cnt-4.md`.

- Capability probe live leg + offline self-check: `scripts/canary_codex.mjs`
  `--probe` / `--probe-selftest`; probe leg protocol recorded in
  `docs/decisions/ab-tiny-protocol.md`. Evidence: `.bee/cells/cnt-5.json`,
  `docs/history/codex-native-transport/reports/probe-evidence.md`.

## Open Gaps

- The Codex native spawn checkpoint's override-field route-check (validating
  a spawn's requested model/effort/fork-count against the configured route,
  per B19) is written as a design intent only. No client version checked so
  far has ever carried those fields into the checkpoint's real input, so
  there is nothing observed to validate against yet; implementing the check
  before that evidence exists would deny based on assumed rather than
  observed shape. It activates once the capability probe observes that
  envelope on some client version (codex-native-transport D6; decision
  350f1e82).
