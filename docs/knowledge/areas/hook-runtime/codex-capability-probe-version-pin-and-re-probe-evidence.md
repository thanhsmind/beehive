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
  sources: ["i54-closeout cell i54-closeout-8 (PROBED_CODEX_VERSION bumped 0.144.4 -> 0.145.0 on the validating canary's post-fix full-green rerun; trace in .bee/cells/, 2026-07-24)", "docs/history/i54-closeout/reports/validation-canary.md section 4 (post-fix full canary rerun, all probes green)", docs/history/i54-closeout/CONTEXT.md D8]
  authoritative_for: "hook-runtime: codex capability probe version pin and re-probe evidence"
---

# Hook Runtime — The Codex Capability Probe Version Pin and Re-Probe Evidence

A version-scoped capability verdict is only as honest as the evidence that pinned
it. This concept owns the single constant every such verdict reads, and the rule
that a pin only moves when a live run has actually watched the new version behave.

## Data Dictionary

| Element | Meaning |
|---|---|
| `PROBED_CODEX_VERSION` | The exact codex-cli version string every version-scoped capability row is judged against — currently `0.145.0`. A live CLI at a different version reads `unprobed_version` (re-probe suggested), never a blanket "unsupported" (owned by `health-checks-and-proof-surfaces.md`). |

## Behaviors & Operations

**The version pin bumps only on a live canary run's evidence, never
speculatively.** `node scripts/canary_codex.mjs`'s rerun against the real
installed binary — all probes green, including the pre-spawn write-guard block
that a separate vendoring bug (hook-vendoring import-closure completeness) had
been blocking — is what unlocked the bump from `0.144.4` to `0.145.0`. The bump
was gated behind that vendoring fix landing first: the canary's own P5 probe
could not pass until the fresh-install crash it was catching was fixed, so the
version pin could not honestly move until the fix did.

**Version-scoped capability rows split into two groups on a bump.** Rows wired
to the shared `PROBED_CODEX_VERSION` template constant (the F1 trust rows, the
A1/A2 `custom_agents` rows) update automatically the moment the constant
changes — no per-row edit needed. A capability row not wired to the constant and
not itself exercised by the bump's own canary run (row C2, `permission_mode`) is
left with its prior provenance untouched: a version bump proves only what that
bump's own canary run actually exercised, never every row a human might assume
travels with it (R18 — never judge an envelope no probe has seen).

## Business Rules

- A capability pin bump is evidence-gated, not date-gated or convenience-gated:
  it commits only after the exact canary run cited as its evidence is
  reproduced green on the target version (i54-closeout D8).
- A capability row not wired to the version constant and not independently
  re-exercised by the bump's own canary run keeps its prior provenance rather
  than silently inheriting the new version's confidence.

## Pointers (implementation)

- Constant: `PROBED_CODEX_VERSION` in `packages/bee/bee.mjs`.
- Canary: `scripts/canary_codex.mjs` (full run, `--probe`, `--probe-selftest`).
- Evidence: `docs/history/i54-closeout/reports/validation-canary.md` (section
  4, post-fix full canary rerun), `.bee/cells/i54-closeout-8.json`.

