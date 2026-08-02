---
area: onboarding
updated: 2026-07-22
migrated_to: docs/knowledge/areas/onboarding/
---

# Onboarding (migrated — pointer stub)

This area's current truth now lives in the knowledge bundle:
[`docs/knowledge/areas/onboarding/`](../knowledge/areas/onboarding/index.md)
(okf-foundation D20/D29/D37). Eight concepts, split by TOPIC rather than the old spec's
headings: `overview.md` owns what onboarding is, its check/apply run modes and its actors;
`status-display-vendoring.md` owns the opt-in status-display pair — detecting the opt-in,
vendoring it, healing drift, staying out otherwise, what the line renders, and the second
runtime's machine-level status block; `managed-ignore-section.md` owns the delimited block
onboarding owns inside the project's ignore list; `distribution-source-exclusivity.md` owns
selecting and proving exactly one distribution source, the Codex hybrid carve-out, the fenced
cleanup in both directions, and the whole-run snapshot; `installer-entrypoints-and-source-staging.md`
owns the outer ring — fetching the source without a full working tree, staging the complete
release identity, and a runtime whose tool is present but broken;
`release-identity-and-version-parity.md` owns one release version across every projection, the
downgrade refusal, honest drift reporting, source-origin classification, and the blast radius a
forceable refusal must name; `repo-local-guardrails.md` owns the remembered opt-in and the second
runtime's lifecycle hook wiring; and `host-project-artifacts.md` owns the artifacts a host project
receives and keeps. This path stays alive as a pointer stub — it is never deleted in this feature
(D20) — and the anchor map below sends every numbered anchor the old spec exposed to the concept
that now owns it, so existing citations keep resolving. Coverage is machine-checked by
`scripts/okf_migrate.mjs --check onboarding` in the verify chain (D35), against the pinned
pre-migration blob `c78ca9b` (`a06f59d`, 58 anchors — 0 B / 28 R / 15 E / 15 P — 20 unparsed blocks
— okf-migration-f2 F8/F9).

Two things about this area's anchors are worth stating plainly. First, **R20b is
letter-suffixed** — it is exactly the id shape f2-4 widened the extractor for and f2-3 widened
the stub-row parser and the `bee.sources` claim matcher for; it carries the GH #26 staging defect
and is owned like any other rule. Second, the 20 unparsed blocks are the whole "Behaviors &
Operations" section: 16 unnumbered bold-lead paragraphs (Detect / Vendor / Heal drift / Stay out /
Context colour / Manage the ignore section / Warn on already-tracked silenced paths / Select and
prove exactly one distribution source / An opt-in is remembered / Install skills into the project
itself / Provide the assistant-instructions import / Retire superseded helper scripts / Fetch the
workflow source without a full working tree / Wire the second-runtime guards / Guarantee the second
runtime's status display / Guarantee the state-layer landing pages), the "What the status display
renders" lead paragraph, and the ignore section's three un-ided continuation bullets. None of them
carries an id, so none is invented into an anchor (D10); their content still travels, verbatim,
into the concept whose topic it matches.

## Anchor map

| Anchor | Now owned by | Was |
|---|---|---|
| R1 | [docs/knowledge/areas/onboarding/status-display-vendoring.md](../knowledge/areas/onboarding/status-display-vendoring.md) | the status-display pair is synced only into projects already opted in |
| R2 | [docs/knowledge/areas/onboarding/status-display-vendoring.md](../knowledge/areas/onboarding/status-display-vendoring.md) | detection is fail-safe: an unrecognized settings shape means "not opted in" |
| R3 | [docs/knowledge/areas/onboarding/status-display-vendoring.md](../knowledge/areas/onboarding/status-display-vendoring.md) | only project-level references count as opt-in |
| R4 | [docs/knowledge/areas/onboarding/status-display-vendoring.md](../knowledge/areas/onboarding/status-display-vendoring.md) | the canonical pair and an opted-in project's vendored copies must be byte-identical |
| R5 | [docs/knowledge/areas/onboarding/distribution-source-exclusivity.md](../knowledge/areas/onboarding/distribution-source-exclusivity.md) | plugin-capable runtimes receive bee primarily as one installable package |
| R6 | [docs/knowledge/areas/onboarding/repo-local-guardrails.md](../knowledge/areas/onboarding/repo-local-guardrails.md) | on Codex, every compatibly exposed lifecycle capability joins bee's mechanical belt |
| R7 | [docs/knowledge/areas/onboarding/repo-local-guardrails.md](../knowledge/areas/onboarding/repo-local-guardrails.md) | a nested Codex worker starts with write access limited to the active workspace |
| R8 | [docs/knowledge/areas/onboarding/repo-local-guardrails.md](../knowledge/areas/onboarding/repo-local-guardrails.md) | Codex dispatch guidance matches the interface the runtime actually exposes |
| R9 | [docs/knowledge/areas/onboarding/managed-ignore-section.md](../knowledge/areas/onboarding/managed-ignore-section.md) | the managed ignore section covers only machine-local runtime records |
| R10 | [docs/knowledge/areas/onboarding/managed-ignore-section.md](../knowledge/areas/onboarding/managed-ignore-section.md) | the section is created, appended with a guaranteed separator, or content-rewritten |
| R11 | [docs/knowledge/areas/onboarding/managed-ignore-section.md](../knowledge/areas/onboarding/managed-ignore-section.md) | onboarding never modifies the host project's version-control index |
| R12 | [docs/knowledge/areas/onboarding/distribution-source-exclusivity.md](../knowledge/areas/onboarding/distribution-source-exclusivity.md) | plugin-first is the default distribution; per-project copies only via repo-copy |
| R13 | [docs/knowledge/areas/onboarding/host-project-artifacts.md](../knowledge/areas/onboarding/host-project-artifacts.md) | the assistant-instructions import artifact is created by default |
| R14 | [docs/knowledge/areas/onboarding/host-project-artifacts.md](../knowledge/areas/onboarding/host-project-artifacts.md) | the vendored command surface is a single unified dispatcher |
| R15 | [docs/knowledge/areas/onboarding/release-identity-and-version-parity.md](../knowledge/areas/onboarding/release-identity-and-version-parity.md) | onboarding never downgrades a project's vendored runtime |
| R16 | [docs/knowledge/areas/onboarding/release-identity-and-version-parity.md](../knowledge/areas/onboarding/release-identity-and-version-parity.md) | the status report tells the truth about the vendored runtime |
| R17 | [docs/knowledge/areas/onboarding/release-identity-and-version-parity.md](../knowledge/areas/onboarding/release-identity-and-version-parity.md) | source origin is named, never guessed from the nearest path |
| R18 | [docs/knowledge/areas/onboarding/distribution-source-exclusivity.md](../knowledge/areas/onboarding/distribution-source-exclusivity.md) | plugin-first cleanup runs only after the installed package is proven |
| R19 | [docs/knowledge/areas/onboarding/distribution-source-exclusivity.md](../knowledge/areas/onboarding/distribution-source-exclusivity.md) | repo-copy fallback is the reverse exclusive transition |
| R20 | [docs/knowledge/areas/onboarding/distribution-source-exclusivity.md](../knowledge/areas/onboarding/distribution-source-exclusivity.md) | a whole-run snapshot is revalidated immediately before the first mutation |
| R20b | [docs/knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md](../knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md) | the staged source copy always contains the COMPLETE release identity |
| R21 | [docs/knowledge/areas/onboarding/release-identity-and-version-parity.md](../knowledge/areas/onboarding/release-identity-and-version-parity.md) | an install has one release version across every projection |
| R22 | [docs/knowledge/areas/onboarding/release-identity-and-version-parity.md](../knowledge/areas/onboarding/release-identity-and-version-parity.md) | a top-level installer reports success only after the target proves it |
| R23 | [docs/knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md](../knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md) | the Linux and Windows entry points prove the same observable outcomes |
| R24 | [docs/knowledge/areas/onboarding/distribution-source-exclusivity.md](../knowledge/areas/onboarding/distribution-source-exclusivity.md) | removing project fallback capabilities requires managed-set proof |
| R25 | [docs/knowledge/areas/onboarding/distribution-source-exclusivity.md](../knowledge/areas/onboarding/distribution-source-exclusivity.md) | planning, preview, and dry-run never install, remove, or change a package |
| R26 | [docs/knowledge/areas/onboarding/release-identity-and-version-parity.md](../knowledge/areas/onboarding/release-identity-and-version-parity.md) | a refused apply that can be forced names its full blast radius |
| R27 | [docs/knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md](../knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md) | a present-but-broken runtime tool does not by itself fail a repo-copy install |
| E1 | [docs/knowledge/areas/onboarding/distribution-source-exclusivity.md](../knowledge/areas/onboarding/distribution-source-exclusivity.md) | skill distribution renders per runtime; malformed markers refuse the entire apply |
| E2 | [docs/knowledge/areas/onboarding/status-display-vendoring.md](../knowledge/areas/onboarding/status-display-vendoring.md) | settings file unparseable → not opted in, run proceeds normally |
| E3 | [docs/knowledge/areas/onboarding/status-display-vendoring.md](../knowledge/areas/onboarding/status-display-vendoring.md) | status-display command present but not a text value → not opted in |
| E4 | [docs/knowledge/areas/onboarding/status-display-vendoring.md](../knowledge/areas/onboarding/status-display-vendoring.md) | the project-directory variable used elsewhere while the path is user-level → not opted in |
| E5 | [docs/knowledge/areas/onboarding/status-display-vendoring.md](../knowledge/areas/onboarding/status-display-vendoring.md) | exactly one pair file drifted → exactly that file is re-planned |
| E6 | [docs/knowledge/areas/onboarding/host-project-artifacts.md](../knowledge/areas/onboarding/host-project-artifacts.md) | a host config still carrying the removed `advisor` key → warn, never error |
| E7 | [docs/knowledge/areas/onboarding/status-display-vendoring.md](../knowledge/areas/onboarding/status-display-vendoring.md) | opting out after having been opted in → the stale managed record is inert |
| E8 | [docs/knowledge/areas/onboarding/release-identity-and-version-parity.md](../knowledge/areas/onboarding/release-identity-and-version-parity.md) | a partial upgrade that reports success is worse than one that fails |
| E9 | [docs/knowledge/areas/onboarding/repo-local-guardrails.md](../knowledge/areas/onboarding/repo-local-guardrails.md) | a local guardrail file deleted or corrupted → the next plain run restores it |
| E10 | [docs/knowledge/areas/onboarding/managed-ignore-section.md](../knowledge/areas/onboarding/managed-ignore-section.md) | no ignore list present at all → one is created holding only the managed section |
| E11 | [docs/knowledge/areas/onboarding/managed-ignore-section.md](../knowledge/areas/onboarding/managed-ignore-section.md) | ignore list missing a trailing line break → the section is still appended cleanly |
| E12 | [docs/knowledge/areas/onboarding/managed-ignore-section.md](../knowledge/areas/onboarding/managed-ignore-section.md) | a comment line resembling the marker text → never mistaken for the real marker |
| E13 | [docs/knowledge/areas/onboarding/managed-ignore-section.md](../knowledge/areas/onboarding/managed-ignore-section.md) | managed ignore section with Windows-style line endings → not reported as drift |
| E14 | [docs/knowledge/areas/onboarding/managed-ignore-section.md](../knowledge/areas/onboarding/managed-ignore-section.md) | a silenced path already tracked → the report warns and names the untrack command |
| E15 | [docs/knowledge/areas/onboarding/host-project-artifacts.md](../knowledge/areas/onboarding/host-project-artifacts.md) | the shipped configuration sample is annotated, not silent |
| P1 | [docs/knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md](../knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md) | shared isolated test-entrypoint runner — `scripts/lib/run-module-worker.mjs` |
| P2 | [docs/knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md](../knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md) | the complete onboarding suite — `packages/bee/scripts/test_bee onboard` |
| P3 | [docs/knowledge/areas/onboarding/distribution-source-exclusivity.md](../knowledge/areas/onboarding/distribution-source-exclusivity.md) | shared strict planner/prover and the 22-case transaction suite |
| P4 | [docs/knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md](../knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md) | package wiring, cross-platform entrypoints, metadata, inventory, cachebuster proof |
| P5 | [docs/knowledge/areas/onboarding/release-identity-and-version-parity.md](../knowledge/areas/onboarding/release-identity-and-version-parity.md) | release-version single-source — `scripts/lib/release-tuple.mjs` |
| P6 | [docs/knowledge/areas/onboarding/distribution-source-exclusivity.md](../knowledge/areas/onboarding/distribution-source-exclusivity.md) | fresh-host handler-delivery proof |
| P7 | [docs/knowledge/areas/onboarding/status-display-vendoring.md](../knowledge/areas/onboarding/status-display-vendoring.md) | statusline opt-in, plan stage 3b, `copy_statusline` — `bee onboard` |
| P8 | [docs/knowledge/areas/onboarding/status-display-vendoring.md](../knowledge/areas/onboarding/status-display-vendoring.md) | canonical pair — `packages/bee/statusline/` |
| P9 | [docs/knowledge/areas/onboarding/status-display-vendoring.md](../knowledge/areas/onboarding/status-display-vendoring.md) | section 9c sandbox cases — `test_bee onboard` |
| P10 | [docs/knowledge/areas/onboarding/status-display-vendoring.md](../knowledge/areas/onboarding/status-display-vendoring.md) | statusline byte-equality sweep — `packages/bee/tests/test_lib.mjs` |
| P11 | [docs/knowledge/areas/onboarding/status-display-vendoring.md](../knowledge/areas/onboarding/status-display-vendoring.md) | host-side settings contract — `.claude/settings.json` |
| P12 | [docs/knowledge/areas/onboarding/status-display-vendoring.md](../knowledge/areas/onboarding/status-display-vendoring.md) | the second runtime's machine-level status block — `ensure_codex_statusline` |
| P13 | [docs/knowledge/areas/onboarding/repo-local-guardrails.md](../knowledge/areas/onboarding/repo-local-guardrails.md) | Codex hook render/merge plus the create-only specs-stub action |
| P14 | [docs/knowledge/areas/onboarding/host-project-artifacts.md](../knowledge/areas/onboarding/host-project-artifacts.md) | the annotated configuration sample — `.bee/config-sample.json` |
| P15 | [docs/knowledge/areas/onboarding/managed-ignore-section.md](../knowledge/areas/onboarding/managed-ignore-section.md) | ignore-section markers, CRLF tolerance, tracked-path advisory |
