---
type: bee.area
title: Hook Runtime — the codex capability probe version pin and re-probe evidence
description: "How the probed-codex-version constant is bumped only on live canary evidence, and which capability rows update automatically versus keep their prior provenance until independently re-exercised."
timestamp: 2026-07-24
bee:
  id: hook-runtime-codex-capability-probe-version-pin
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md, areas/hook-runtime/health-checks-and-proof-surfaces.md]
  decisions: [i54-closeout D8 (capability pin bumps only on observed evidence), 103a5608 (i54-closeout scope lock)]
  sources: [packages/bee-rs/crates/bee/src/doctor.rs, "docs/history/i54-closeout/reports/validation-canary.md section 4 (post-fix full canary rerun, all probes green)", docs/history/i54-closeout/CONTEXT.md D8]
  authoritative_for: "hook-runtime: codex capability probe version pin and re-probe evidence"
---

# Hook Runtime — The Codex Capability Probe Version Pin and Re-Probe Evidence

A version-scoped capability verdict is only as honest as the evidence that pinned
it. This concept owns the attestation record and version pin that every such
verdict reads, and the rule that a pin only moves when a live run has actually
watched the new version behave.

## Data Dictionary

| Element | Meaning |
|---|---|
| `.bee/doctor-attest.json` | The attestation record (`packages/bee-rs/crates/bee/src/doctor.rs:46` `ATTEST_REL`), written by `bee doctor attest --runtime codex`. Its three legs are the hooks-file hash (`hooks_sha256`), the codex version (`codex_version`), and the repo identity (`repo_identity`). |
| `codex_version` | The pinned Codex CLI version string in `.bee/doctor-attest.json` that version-scoped capability rows are judged against. `read_attestation` (`doctor.rs:772-784`) compares it against the live CLI (`codex --version`), yielding `unprobed_version` when the live CLI does not answer and `version_changed` when it answers differently (never a blanket "unsupported"; owned by `health-checks-and-proof-surfaces.md`). |

## Behaviors & Operations

**The version pin bumps only on a live canary run's evidence, never
speculatively.** The validating canary run's evidence (`docs/history/i54-closeout/reports/validation-canary.md`
section 4) against the real installed binary — all probes green, including the
pre-spawn write-guard block that a separate vendoring bug (hook-vendoring
import-closure completeness) had been blocking — is what unlocked the bump from
`0.144.4` to `0.145.0`. The bump was gated behind that vendoring fix landing
first: the canary's own P5 probe could not pass until the fresh-install crash it
was catching was fixed, so the version pin could not honestly move until the
fix did.

**Version-scoped capability rows split into two groups on a bump.** Rows covered
by the attestation record's `codex_version` (the Codex trust rows checked by
`read_attestation`) update their validity when `bee doctor attest --runtime codex`
records the new probed version — no per-row edit needed. A capability row not
covered by the attestation and not itself exercised by the bump's own canary run
(such as row C2, `permission_mode`) is left with its prior provenance
untouched: a version bump proves only what that bump's own canary run actually
exercised, never every row a human might assume travels with it (R18 — never
judge an envelope no probe has seen).

## Business Rules

- A capability pin bump is evidence-gated, not date-gated or convenience-gated:
  it commits only after the exact canary run cited as its evidence is
  reproduced green on the target version (i54-closeout D8).
- A capability row not wired to the version pin and not independently
  re-exercised by the bump's own canary run keeps its prior provenance rather
  than silently inheriting the new version's confidence.

## Pointers (implementation)

- Attestation record: `.bee/doctor-attest.json` (`packages/bee-rs/crates/bee/src/doctor.rs:46` `ATTEST_REL`).
- Attest verb: `bee doctor attest --runtime codex` (`packages/bee-rs/crates/bee/src/doctor.rs:895` `run_attest`).
- Attestation evaluation: `read_attestation` (`packages/bee-rs/crates/bee/src/doctor.rs:768-789`), matching `hooks_sha256`, `codex_version`, and `repo_identity`.
- Evidence: `docs/history/i54-closeout/reports/validation-canary.md` (section 4, post-fix full canary rerun).
