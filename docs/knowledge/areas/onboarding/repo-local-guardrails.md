---
type: bee.area
title: Onboarding — repo-local guardrails and second-runtime lifecycle wiring
description: "The opt-in that is remembered so a project's local guardrails track the workflow's own version forever, how the second runtime's project hook file is merged without losing owner entries, and the Codex lifecycle capabilities bee participates in."
timestamp: 2026-07-26
bee:
  id: onboarding-repo-local-guardrails
  lifecycle: active
  areas: [onboarding]
  required_context: [areas/onboarding/overview.md]
  decisions: [9927fafb (a switch that narrows what an upgrade compares must equally narrow what it claims; repo-hook opt-in is sticky), b7af1bf9 (full compatible Codex lifecycle-hook parity), 73ed41d6 (workspace-scoped Codex executors; blanket bypass forbidden), d7d5f459 (current Codex dispatch contract first; custom profiles deferred), "codex-hook-state-parity D1-D3, D8-D14", "e0f3e40e (packages-restructure D1-D5: vendor payload relocated to packages/bee/, skills instruction-only, PLUGIN_ROOT-relative resolution)"]
  sources: ["sticky-repo-hooks (cell sticky-hooks-1, 2026-07-13; found auditing 8 host projects after the v0.1.30 rollout)", "codex-hook-state-parity cells 2, 3, 5 (paired Codex lifecycle audit, exclusive plugin-first/repo-copy distribution, and fresh-host handler delivery; capped traces and reports, 2026-07-16)", "codex-runtime-parity D2 (lifecycle enforcement contract, 2026-07-11)", "codex-runtime-parity D3 (nested-executor safety boundary, 2026-07-11)", "codex-runtime-parity D4 (dispatch-contract scope, 2026-07-11)", "docs/specs/onboarding.md#R6", "docs/specs/onboarding.md#R7", "docs/specs/onboarding.md#R8", "docs/specs/onboarding.md#E9", "docs/specs/onboarding.md#P13", "docs/history/packages-restructure/ (cells packages-restructure-1..4, 2026-07-25/26: vendor payload relocation, hook catalog move, distribution-surface roles, prose sweep)"]
  authoritative_for: "onboarding: repo-local guardrails and second-runtime lifecycle wiring"
---

# Onboarding — Repo-Local Guardrails and Second-Runtime Lifecycle Wiring

A project that once asked to carry its own copies of the lifecycle guardrails has
made a *choice*, not granted a one-time consent — so the choice is remembered and its
guardrails are refreshed on every later run, whether or not the request repeats the
switch. This concept owns that memory, the merge discipline that keeps a second
runtime's hook file correct without touching the owner's own entries, and the
lifecycle capabilities bee's mechanical belt participates in on Codex.

## Behaviors & Operations

**An opt-in is remembered, and what it opted into stays current (every run).**
Trigger: any run against a project that has previously opted into carrying its
own local copies of the lifecycle guardrails. What changes: those local copies
are refreshed to the current ones on **every** run thereafter — whether or not
the request repeats the opt-in switch. Why: the switch names a *choice the
project made*, not a consent owed again at each upgrade. What each actor
observes: an owner who opted in once sees their guardrails track the workflow's
own version, silently and permanently; a project that never opted in is still
never handed local guardrails by a plain run — the remembered choice is the only
thing that carries, never a default. What used to happen instead, and is the
reason this behavior is stated explicitly: a plain upgrade refreshed the standing
instruction sheet, the helpers, and the recorded version, left the guardrails at
whatever version they were first installed at, and **reported the project up to
date** — so a project could run current doctrine against its original guards
indefinitely, with no signal anywhere that it was doing so.

**Wire the second-runtime guards (repo-hook installs).** Trigger: any run for a
project that vendors repo-local hooks (the explicit opt-in flag or its sticky
record). What blocks it: nothing — the projection is derived from the same hook
catalog as the first runtime's wiring. What changes: the second runtime's
project hook file is created or merged so every guarded lifecycle event
(session start, prompt, pre-write guard, post-task sync, subagent close,
pre-compaction, session close) runs the same vendored guard scripts. Merge
discipline: entries the project owner added themselves are preserved verbatim;
bee-shipped entries in ANY historical shape — including wiring that resolved
through the first runtime's project variable (dead on the second runtime) and
the source-repository layout — are replaced by the canonical render, never
preserved beside it (a stale twin would fire every event twice); a pre-existing
file is backed up before the first rewrite; a second apply changes nothing.
Two pinned asymmetries with the first runtime, both catalog-declared: the
model-tier guard is not wired (the second runtime does not expose agent spawn
through a pre-tool event), and every command resolves the project root from the
session's working directory with a visible fail-open when there is none. What
the human observes: after updating, the second runtime's hook panel lists the
full bee guard set for the project (trust must still be granted once, in that
runtime, per project — the installer cannot grant it).

## Business Rules

- **R6** — On Codex, every lifecycle capability
  exposed compatibly by the host participates in bee's mechanical belt: session
  bootstrap, phase reminders, write/privacy/reservation checks, state refresh,
  worker-completion nudges, and close-time hygiene. Shared helper commands remain
  authoritative when a host path cannot be intercepted; such gaps fail open,
  stay visible to the operator, and have runtime-specific tests (decision
  b7af1bf9).
- **R7 (not yet implemented — P24)** — A nested Codex worker or reviewer starts
  with write access limited to the active workspace and keeps normal approval
  behavior. Bee never grants a blanket approval-and-sandbox bypass; broader
  access is a separate human decision for one named command (decision 73ed41d6).
- **R8 (not yet implemented — P24; profiles deferred to P25)** — Codex dispatch
  guidance matches the collaboration interface the runtime actually exposes,
  including explicit clean-context spawning and continuation. Bee does not ship
  named Codex agent profiles until swarming can select and verify those profiles;
  unused configuration is not parity (decision d7d5f459).
- **R9** — The second-runtime hook projection is skipped entirely when the
  target repository IS the hook catalog's own authority (bee's own source
  checkout): writing the repo-local projection there would clobber the
  generated catalog that repository already owns, so the catalog stays
  authoritative in place instead of being overwritten by a projection of
  itself. Self-recognition accepts either the catalog's current location or
  its legacy pre-restructure location, so a checkout still on an older bee
  release is still correctly self-identified — never forceable, only ever a
  backward-compatible fallback (packages-restructure D2; decision e0f3e40e).

## Edge Cases Settled

- A local guardrail file deleted or corrupted in a project that opted in → the
  next plain run restores it from source. Nothing else in the project is touched.

## Open Gaps

- P24 must replace executor presets that imply workspace isolation without
  actually enforcing it, and verify the effective sandbox/approval boundary.
- Custom Codex explorer/worker/reviewer profiles remain deferred under P25 until
  a live dispatch can select them and prove the resulting role configuration.

## Pointers (implementation)

- Self-recognition (R9): `repoOwnsHookCatalog()` in
  `packages/bee/scripts/bee onboard`, checked before both the
  `--repo-hooks` and codex-hybrid Codex-projection branches. Checks
  `packages/bee/hooks/catalog.mjs` OR the legacy `hooks/catalog.mjs` (repo-root,
  pre-packages-restructure) — an OR fallback, never a forced migration.
- `packages/bee/scripts/bee onboard` — `renderCodexHookEntries()`,
  `mergeCodexHooks()`, `isBeeCodexHookEntry()` (any-transport bee-entry
  matcher), `merge_codex_hooks` plan/apply action, `.codex/hooks.json`
  pseudo-entry in `buildManagedVersions`; `READING_MAP_STUB`/
  `SYSTEM_OVERVIEW_STUB` + `create_specs_stub` (create-only) — host contract:
  `.codex/hooks.json`, `docs/specs/reading-map.md`, `docs/specs/system-overview.md`.
