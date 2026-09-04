---
type: bee.area
title: "Onboarding — purpose, run modes, and actors"
description: "What onboarding installs and keeps current inside a host project, the difference between a check run and an apply run, who is allowed to run either, and which parts of the surface this area does not yet describe."
timestamp: 2026-07-22
bee:
  id: onboarding-overview
  lifecycle: active
  areas: [onboarding]
  decisions: [55ff17ef (release-version parity is fail-closed across every distributed projection), 09b776b5 (both installers prove complete greenfield/brownfield postconditions before success), cf511ff3 (plugin/package and repo-copy sources are mutually exclusive), 4cc1c355 (Codex plugin-first distribution), "3318374a (installer hardening: per-project skills default, global opt-in, default instructions import)"]
  sources: ["installer-hardening ih-1..ih-6 (cells, 2026-07-13; flushed capture stub 92c9bcf6)", "installer-version-parity-1-3-1 locked rules (fail-closed release tuple, full projection parity, greenfield/brownfield end-to-end success contract)", "codex-sandbox-baseline cells codex-sandbox-baseline-1/codex-sandbox-baseline-2 (real onboarding entrypoint through the shared isolated test runner; full onboarding suite green, 2026-07-16)"]
  authoritative_for: "onboarding: purpose, run modes, and actors"
  owns.code: ["packages/bee-rs/crates/bee/src/onboard/*"]
  owns.skills: []
  owns.tests: [packages/bee-rs/crates/bee/src/onboard/tests.rs, packages/bee-rs/crates/bee/tests/installer_contracts.rs, packages/bee-rs/crates/bee/tests/installer_invocations.rs, packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs]
---

# Onboarding (purpose, run modes, and actors)

Onboarding installs and keeps current everything bee manages inside a host project:
the agent-instructions block, the runtime state files, the vendored helper commands,
and — by default — the workspace status-display script. Re-running it
is always safe: it reports what would change before changing anything, and an
up-to-date project reports "nothing to do".

> **Coverage note:** this area describes status-display vendoring, the managed
> ignore section, distribution-mode selection, exclusive hook-source
> arbitration, fenced cleanup, and installed-package proof. Remaining surfaces
> are listed under Open Gaps.

## Entry Points & Triggers

- A check run: the agent asks onboarding what would change (report-only, no writes).
- An apply run: the agent authorizes onboarding to perform the reported changes.
- Both runs are executed by the agent, never handed to the human.

The two remaining entry points — the plugin-first default and the dry run that plans
the whole distribution transaction without mutating anything — belong to the source a
run installs from, and are owned by
[`distribution-source-exclusivity.md`](distribution-source-exclusivity.md).

## Actors & Access

- **Agent** — runs check and apply; the only actor that executes onboarding.
- **Human** — approves an apply when onboarding reports changes; owns the opt-in
  decision by editing their project's assistant settings (or not).

## Where the rest of this area lives

| Concept | What it owns |
|---|---|
| [`status-display-vendoring.md`](status-display-vendoring.md) | The default status display: detecting the settings entry, vendoring by default, respecting preference and --no-statusline, what the line renders, and the second runtime's machine-level status block |
| [`managed-ignore-section.md`](managed-ignore-section.md) | The delimited block onboarding owns inside the project's ignore list: what it silences, what it must never silence, and how it is created, appended, or rewritten |
| [`distribution-source-exclusivity.md`](distribution-source-exclusivity.md) | Selecting and proving exactly one distribution source, the fenced cleanup in both directions, and the whole-run snapshot revalidated before the first mutation |
| [`installer-entrypoints-and-source-staging.md`](installer-entrypoints-and-source-staging.md) | The installer entry points themselves: fetching the source without a full working tree, staging the complete release identity, and tolerating a runtime whose tool is present but broken |
| [`release-identity-and-version-parity.md`](release-identity-and-version-parity.md) | One release version across every projection, the downgrade refusal, honest drift reporting, source-origin classification, and the blast radius a forceable refusal must name |
| [`repo-local-guardrails.md`](repo-local-guardrails.md) | The remembered opt-in that keeps a project's local guardrails current, and the second runtime's lifecycle hook wiring |
| [`host-project-artifacts.md`](host-project-artifacts.md) | The artifacts a host project receives and keeps: the instructions import, the unified command surface, the state-layer landing pages, and the annotated configuration sample |

## Open Gaps

- The remainder of the onboarding surface is unspecified here: instructions-block
  merge rules, runtime-file creation, helper/lib vendoring, global skill sync, and
  the greenfield init lane. The runtime-**downgrade** protection is now specced
  (R15 — the self-install runtime-lib downgrade refusal, zero-mutation); the
  broader force-override reporting and per-skill-target sync details remain to
  harvest. Until then the authoritative description of the unspecced parts is the
  code and its test suites.

## Concepts in this area

- [Onboarding — hook vendoring import-closure completeness](hook-vendoring-import-closure-completeness-for-repo-local-gu.md) — Why a fresh repo-hooks onboard must vendor every module a vendored hook imports, transitively, not merely the hook entrypoints themselves, and the regression suite that guards the class of bug rather than the one instance.

## Patterns in this area

- [Agent-runtime discovery paths are version-moving targets — probe the binary, not memory](../../patterns/20260714-agent-runtime-discovery-paths-are-version-moving-targets.md) — Probe the binary, not memory
- [Non-ASCII in a .ps1 without BOM is a parse-time bomb on Windows PowerShell 5.1](../../patterns/20260714-non-ascii-in-a-ps1-without-bom-is.md) — Non-ASCII in a .ps1 without BOM is a parse-time bomb on Windows PowerShell 5.1
- [A guard scoped inside a skippable loop is absent on the path that skips it](../../patterns/20260715-a-guard-scoped-inside-a-skippable-loop-is.md) — A guard scoped inside a skippable loop is absent on the path that skips it
- [A test that runs the canonical source can never catch vendoring drift — only a fresh-install subprocess can](../../patterns/20260724-canonical-source-tests-cannot-see-vendoring-drift.md) — A guard crashed fail-open on every fresh install for a whole release cycle: its sibling import was never vendored, node --check passes on missing imports, and every suite ran the canonical file where the import always resolves. The only detector was a live canary spawning the actually-vendored copy from a freshly-onboarded fixture.
- [A managed-file ledger needs a removal path derived from its own diff, not a hand-maintained retired list](../../patterns/20260724-ledger-diff-derived-removal.md) — A managed-file ledger needs a removal path derived from its own diff, not a hand-maintained retired list
- [A guard comparing a value against its own derivation can never fail — anchor on independently derived evidence](../../patterns/20260726-a-guard-compared-to-its-own-derivation-never-fails.md) — A proposed identity anchor compared realpath(PLUGIN_ROOT/packages/bee/scripts) with SCRIPTS_DIR — but PLUGIN_ROOT is computed from SCRIPTS_DIR, so the check is true by construction and the live guard it replaced would have become a no-op. The repaired anchor asserts facts from an independent derivation: the skills tree exists under the package root and the payload file is readable. Same family: one variable (HIVE_DIR) silently carried three semantics — engine geometry, a shared function's input, and the tree sync copies — and a wholesale rename would have pointed sync at the wrong tree.
- [A migration's own regen and proof tooling is a consumer of the migration — inventory it before the move](../../patterns/20260726-migration-tooling-is-a-consumer-of-the-migration.md) — The first mandatory regen command of the move cell statically imported from the tree being moved; unfixed, the cell's own regen obligation would have crashed at its first step. ESM static imports resolve at load time, so a generator that proves a move can itself be broken by the move — and the companion rule: what a manifest enumerates must change in the same cell as the move, because every cell boundary is a potential ship point.
