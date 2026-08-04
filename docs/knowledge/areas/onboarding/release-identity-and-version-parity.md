---
type: bee.area
title: "Onboarding — release identity, version parity, and honest reporting"
description: "One release version across every projection, the refusal to downgrade a project's vendored runtime, drift reported from real file content, the five source origins named rather than guessed, the blast radius a forceable refusal must enumerate before consent, and a retired library module removed from the project the moment its ledger entry drops."
timestamp: 2026-07-26
bee:
  id: onboarding-release-identity-and-version-parity
  lifecycle: active
  areas: [onboarding]
  required_context: [areas/onboarding/overview.md]
  decisions: [55ff17ef (release-version parity is fail-closed across every distributed projection), 09b776b5 (both installers prove complete greenfield/brownfield postconditions before success), "fe6593c0 (runtime-lib downgrade refusal targets the vendored copy path; zero-mutation, self-install included)", 485e949a (honest status drift reference = the onboarding managed-hash ledger), "579bbad7 (status drift is report-only, stays a boolean + optional detail; fail-open on absent/legacy ledger)", "ce4eee19 (SRC-01..06 shipped as a pure shared classifier, wrap-not-replace, consumed by status + onboarding)", 21be04f7 (status gains a report-only source field; unknown/legacy never implicit source), cba8b832 (release-version single-source), 9927fafb (a switch that narrows what an upgrade compares must equally narrow what it claims), "6eacf846 (auto-approved shape+execution for installer-verify-orphan-drift, bypass total)", "053a49fa (retired library modules are removed on apply via a ledger-diff derivation, not a hand-maintained list)", "e0f3e40e (packages-restructure D1-D5: vendor payload relocated to packages/bee/, skills instruction-only, PLUGIN_ROOT-relative resolution)", "a25ef382 (packages-restructure D6: release manifest package_payload role for packages/bee, plugin_hook role repointed to packages/bee/hooks)", "80b64c20 (packages-engine-move D1-D5: onboarding/distribution engine relocated to packages/bee/scripts, strict-flag validation universal, migration-tooling pattern)", "809215be (packages-engine-move Gate 3: classifySource contract unchanged, HIVE_DIR split into ENGINE_DIR/PLUGIN_ROOT/SKILLS_ROOT, identityOk re-anchored falsifiable)"]
  sources: ["installer-version-parity-1-3-1 locked rules (fail-closed release tuple, full projection parity, greenfield/brownfield end-to-end success contract; cells -4/-2/-3, 2026-07-16; field fix cells -5/-6)", "codex-harness-hardening cell codex-harness-hardening-1b-1 (runtime-lib downgrade guard R15; split-brain regression 3->0, 2026-07-15)", "codex-harness-hardening-1c cell codex-harness-hardening-1c-1 (honest status drift R16 via the onboarding managed-hash ledger; 5 drift tests, 2026-07-15)", "codex-harness-hardening-1d cells 1d-1/1d-2 (SRC-01..06 source-identity classifier R17 + status source field; 8 classifier/status tests, 2026-07-15)", "sticky-repo-hooks (cell sticky-hooks-1, 2026-07-13; found auditing 8 host projects after the v0.1.30 rollout)", "cell p49-force-downgrade-blast-radius-1 (PBI P49, v1.1.0 review P2; advisor-consulted)", "cell installer-verify-orphan-drift-1 (R27: retired-library removal on apply; repro was install.sh reporting version-parity failure/drift=true post-apply on a host still carrying a retired templates/lib module, 2026-07-24)", "docs/specs/onboarding.md#R15", "docs/specs/onboarding.md#R16", "docs/specs/onboarding.md#R17", "docs/specs/onboarding.md#R21", "docs/specs/onboarding.md#R22", "docs/specs/onboarding.md#R26", "docs/specs/onboarding.md#E8", "docs/specs/onboarding.md#P5", "docs/history/installer-verify-extra-drift/ (cell installer-verify-extra-drift-1: install.sh verify step tolerates extra-file-only drift, 2026-07-24)", "docs/history/packages-restructure/ (cells packages-restructure-1..4, 2026-07-25/26: vendor payload relocation, hook catalog move, distribution-surface roles, prose sweep)", docs/history/packages-engine-move/]
  authoritative_for: "onboarding: release identity, version parity, and honest reporting"
---

# Onboarding — Release Identity, Version Parity, and Honest Reporting

Every rule in this concept protects the same property: **what a run reports must be
what a project actually has.** A version claimed in one place and discovered in
another, a runtime silently downgraded, a drifted file reported as current, an
authoritative-looking source that is really a rendered copy, a refusal that hides
what forcing it would overwrite — each is the same defect wearing a different face,
and the strongest of them is the one that reports success.

## Business Rules

- **R15** — Onboarding never downgrades a project's vendored runtime. Before
  vendoring its own runtime helpers into a project, it compares the running
  installer's runtime version against the version already installed in the
  project; if the installer is older, it refuses the **entire** apply before
  making any change and reports a blocked downgrade. This holds even in the
  self-install case — where the installer's own source tree lives inside the
  project it targets, so the per-project skill syncs are skipped — because the
  refusal is drawn from the runtime version alone, independent of any
  per-location sync. A refused downgrade changes nothing anywhere (zero
  mutation across every vendored file and the installation ledger). A project
  with no runtime yet is a fresh install and proceeds; a project whose
  installed runtime version cannot be read is treated as unknown and is also
  refused — never overwritten by an older or unreadable source. An explicit
  force override is honored only when both the source and the installed version
  are known release numbers (decision fe6593c0; cell codex-harness-hardening-1b-1).
- **R16** — The status report tells the truth about the vendored runtime. At
  install, onboarding records a per-file content fingerprint of every vendored
  helper and library module. The status report recomputes those fingerprints
  from the files actually on disk and reports **drift** whenever any managed
  file's content differs from what was recorded, a managed file is missing or an
  unrecorded one has appeared, or the recorded version differs from the running
  one — so a runtime altered without re-onboarding is caught **even when its
  version string is unchanged**. Drift stays a single true/false fact; an
  optional companion list names which files drifted. The report only *reports*:
  it never repairs — bringing the runtime back into agreement is an apply run.
  When no fingerprint record exists yet (a legacy installation) or it cannot be
  read, the check degrades to the version comparison alone and the report still
  renders — it never fails on a missing or unreadable record (decisions 485e949a,
  579bbad7; cell codex-harness-hardening-1c-1).
- **R17** — Source origin is named, never guessed from the nearest path. A single
  shared detector classifies the bee source tree into exactly one of five
  origins: a **canonical development checkout** (the authored source, carrying its
  package manifest and version-control marker), a **project's vendored copy** (a
  projection living inside a host project's assistant-runtime folders), an
  **installed package** (a distributed manifested snapshot — usable as the source
  for the same project's runtime and copies, but never permitted to install
  shared/global targets), the **legacy shared location** (the old machine-global
  root — only reported or migrated, never an implicit source), and **unrecognized**
  (a missing or unreadable manifest, or anything ambiguous). The status report
  surfaces this origin (report-only), and onboarding names the same origin using
  the same detector, so the two never disagree about identity. Classification is
  pure — it only reads, never changes anything, and never fails — and an
  unrecognized origin is named as such, never silently treated as an authoritative
  source that may overwrite an installation (decisions ce4eee19, b5341fe7,
  21be04f7; cells codex-harness-hardening-1d-1, 1d-2). Post-move
  (packages-engine-move D2-D3; decisions 80b64c20, 809215be): the classifier's
  own contract is untouched — `classifySource` still takes one `hiveDir` input
  and returns the same five kinds — but the engine that used to live inside the
  tree it classifies now builds that input explicitly. Three previously
  conflated meanings of one `HIVE_DIR` constant (engine geometry, the
  classifier's `hiveDir` input, and the skills root skill-sync walks) split
  into `ENGINE_DIR` (`packages/bee`, geometry only), `PLUGIN_ROOT` (the same
  3×dirname arithmetic as before the move, now walked from `ENGINE_DIR`), and
  `SKILLS_ROOT` (`PLUGIN_ROOT/skills`, the tree skill-sync renders and
  `classifySource` reads) — `SKILLS_ROOT` is never derived from `ENGINE_DIR`,
  so a sync pass can never wander into `packages/`. The identity anchor that
  gates every sync target was re-anchored the same move: the pre-move check
  (a bare `realpath` self-comparison) was tautological — the running script
  IS that path, proving nothing about the skills tree it is about to render
  into projections — and the post-move anchor is falsifiable instead:
  `<SKILLS_ROOT>/bee-hive` must exist, its realpath must resolve inside the
  same `PLUGIN_ROOT`, and the payload the engine ships alongside must be
  readable; a failure blocks every sync target before any resolution, the same
  `blocked_no_source` semantics as before the re-anchor. A synced projection
  (`.claude/skills`, `.agents/skills`) still never carries the engine itself —
  running it from a projection-shaped root still classifies as
  `rendered_projection` and still blocks with `blocked_no_source`, now pinned
  against the new engine location by the rewritten split-brain regression
  suite.
- **R21 (not yet implemented — installer-version-parity-1-3-1)** — An install
  has one release version across its authoritative source, enabled package,
  project runtime, and every project-local assistant capability copy. Every
  required source marker must exist, be readable, and agree before the target
  changes. Target surfaces may be absent for a new project, but every applicable
  one must exist and equal the validated source version before success; an
  existing unreadable or different target is never ignored. A successful run
  can never report one version while an assistant discovers another (decision
  55ff17ef).
- **R22 (not yet implemented — installer-version-parity-1-3-1)** — A top-level
  installer reports success only after the target reports the requested release
  version for both onboarding and the selected package source, reports no managed
  drift, and an immediate second check reports nothing left to update. A mere
  "installed" flag is not a sufficient success condition (decision 09b776b5).
- **R26** — A refused apply that can be forced names its full blast radius
  before the operator consents. When a blocked downgrade is force-eligible, the
  refusal report enumerates — beyond the per-target skill actions it already
  carried — every vendored runtime file (helper and library path under the
  project's tools directory) that the force would overwrite, copied verbatim
  from the pending plan in plan order, never recomputed. The list is present
  and empty when a forceable refusal has no runtime drift, and absent entirely
  when the refusal cannot be forced (unknown version, unresolved source) — a
  refusal never advertises a force that cannot happen. Runtime-file entries
  carry plain project-relative paths with no target/scope tagging (those
  belong only to skill items, whose paths resolve against varying roots). The
  preview is exact: forcing applies precisely the enumerated runtime set, no
  more, no fewer (PBI P49, v1.1.0 review P2; advisor-consulted; cell
  p49-force-downgrade-blast-radius-1).
- **R27** — A vendored library module the managed-hash ledger no longer names
  is removed from the project on the next apply, not merely reported. R16's
  fingerprint set only ever grows or refreshes toward what the current source
  still ships, so a module the source has since retired would otherwise
  linger on disk forever, forever reported as drifted (an unrecorded file
  present) — the exact failure a top-level installer's final parity check
  cannot tolerate, since a project can never reach zero drift on its own.
  Removal compares the ledger's previous recorded set against the set the
  apply is about to write and deletes exactly the vendored library files
  whose names dropped out of it — never a name still present, never anything
  outside the vendored library location, and never a file the ledger already
  agreed was gone. Unlike the nine superseded helper scripts (a one-time,
  hand-enumerated migration), this removal is derived fresh from the ledger
  diff every run, so a future library retirement needs no enumerated list of
  its own. Re-running an apply after the removal reports nothing left to do
  (decision 053a49fa; cell installer-verify-orphan-drift-1).

- **R28** — The installer's own final verify step distinguishes real drift from
  extra-file-only drift before deciding whether to hard-fail. It keeps the
  existing fatal check unchanged for a `bee_version`/`plugin_version` mismatch
  against the expected release, and unchanged for any drift entry that is a
  real hash mismatch or a missing managed file. But when the only drift present
  is one or more entries suffixed "(extra)" — an unmanaged file sitting in the
  vendored library location, with every version already matching expected — the
  verify step does not hard-fail on that alone: it prints the specific extra
  file path(s) with guidance to remove them (or notes they self-heal on the
  next onboarding refresh) and continues to its up-to-date recheck. A real
  hash-mismatch or missing-file entry anywhere in the same drift set still
  hard-fails exactly as before — extra-file-only is the only tolerated shape
  (installer-verify-extra-drift cell installer-verify-extra-drift-1, 2026-07-24).
- **R29** — Since the vendored payload stopped shipping a version marker of its
  own inside every synced skill directory (skills are instruction-only), the
  per-target skill-sync version preflight reads a sync-owned version stamp
  written at the skill-sync root on every successful apply, for every target
  kind, and treats it as authoritative when present. Only when the stamp is
  genuinely absent does the preflight fall back to the legacy nested marker, so
  an already-onboarded host synced by a pre-restructure release still resolves
  a real version, applies, and gets the stamp written for every onboard after
  that — zero flags, zero manual steps. A stamp that is present but malformed
  or a symlink reads as unknown and is never treated as absent — it is never
  allowed to fall through to the legacy marker, which would reopen a
  downgrade-bypass path R15 already closed (packages-restructure D1;
  decision e0f3e40e; cell packages-restructure-1).
- **R30** — The release inventory records each file's executable bit as
  version control carries it, never the raw permission bits the filesystem
  reports. Raw filesystem bits are not comparable across machines: one
  platform synthesizes a fixed value for every file and cannot observe the
  executable bit at all, so an inventory built on one platform reported
  every one of its records as drifted when checked on another — on both the
  inventory surface and the installed-package proof. A checker running where
  the executable bit cannot be observed does not get to call it changed.

## Edge Cases Settled

- **A partial upgrade that reports success is worse than one that fails.** The
  upgrade path refreshed every part of a project it compared, and compared every
  part except the one the request had not explicitly named — so the unrefreshed
  part was not merely skipped, it was *excluded from the up-to-date judgment* and
  the project was pronounced current. Eight projects ran that way across several
  versions. Whenever a switch narrows what a run compares, the switch's absence
  must narrow what the run *claims*, never only what it does.

## Open Gaps

- R29's malformed/symlinked-stamp behavior (never falling back to the legacy
  marker) is correct by construction in the code path but has no dedicated
  negative regression test yet: a case proving "stamp present but malformed or
  a symlink, with a valid legacy marker also present, still resolves to
  unknown" is unpinned.

## Pointers (implementation)

- Source-identity geometry and the falsifiable anchor (R17 post-move):
  `ENGINE_DIR`/`PLUGIN_ROOT`/`SKILLS_ROOT` constants and the `identityOk`
  check in `packages/bee/scripts/bee onboard` (`computeSkillSync`'s
  identity-anchor block); `classifySource` itself in
  `packages/bee/lib/source-identity.mjs` is unchanged (packages-engine-move
  C3) — only its two call sites (`readSourceReleaseIdentity`,
  `computeSkillSync`) now build `hiveDir` from `SKILLS_ROOT`. Regression:
  `packages/bee/scripts/tests/test_bee onboard`,
  `packages/bee/scripts/tests/test_split_brain_regression.mjs` (rewritten
  stale-launcher scenario now asserts a synced projection carries no launcher
  and an engine copy invoked from a projection-shaped root reports
  `blocked_no_source`). Evidence: `.bee/cells/packages-engine-move-1.json`,
  `docs/history/packages-engine-move/reports/packages-engine-move-1.md`.
- Release-version single-source (decision cba8b832): the four physical homes of
  the version — `packages/bee/lib/state.mjs` + its `.bee/bin/lib` mirror, and the
  two plugin manifests' `.version` — are enumerated once in
  `scripts/lib/release-tuple.mjs` (a side-effect-free registry of location +
  read/write). `scripts/tests/test_release_tuple.mjs` (check) and
  `scripts/bump_version.mjs` (write) both import it, so a release sets every
  member from one command — `node scripts/bump_version.mjs <version>` — which
  also regenerates the hash manifest. The split-brain regression fixture's
  "current version" is derived at runtime from the canonical
  `packages/bee/lib/state.mjs` (no hand-edited anchor); `scripts/tests/test_bump_version.mjs`
  proves the writer covers every registry component and preserves each file's
  surrounding bytes. Plugin manifests keep a literal `.version` because external
  plugin systems read them as raw JSON and cannot import the JS const.
- Release manifest payload/hook roles (packages-restructure D5-D6; decision
  a25ef382; cell packages-restructure-3): `scripts/release_manifest.mjs` walks
  `packages/bee/` (excluding its `hooks/` subtree) under a new role,
  `package_payload` (78 records) — replacing the payload's old implicit
  coverage via the `skills/` walk — and walks `packages/bee/hooks/` under the
  pre-existing role `plugin_hook`, unchanged in name (18 records).
  `plugin_distribution.mjs`'s `PACKAGE_ROLES` set accepts both roles so
  `proveInstalledPackage` keeps proving the vendored payload ships.
  `.claude-plugin/plugin.json` and `.codex-plugin/plugin.json` each point their
  `hooks` config at `./packages/bee/hooks/claude-hooks.json` and
  `./packages/bee/hooks/hooks.json` respectively.
- Skill-sync version stamp (R29): `SKILLS_VERSION_STAMP` constant
  (`.bee-skills-version.json`) and the stamp-first/legacy-marker-fallback read
  in `computeSkillSyncTarget` (`packages/bee/scripts/bee onboard:1271-1281`)
  and the analogous global-root read (`:1461-1471`). Evidence:
  `.bee/cells/packages-restructure-1.json`.
- Retired-library removal (R27): `bee onboard`'s plan builder derives a
  `remove_lib` item per retired library module by diffing the previous
  `.bee/onboarding.json` `managed.lib` keys against the current
  `listTemplateLibModules()` set (section 3c, immediately after the existing
  `copy_lib`/`RETIRED_HELPERS` items it mirrors); apply executes it with the
  same exact-dirname safety `remove_helper` uses. Regression coverage:
  `packages/bee/scripts/tests/test_bee onboard` ("stale lib" block).
- Extra-file-only drift tolerance (R28): the verify node snippet at the tail of
  `scripts/install.sh` (piped after `printf '%s' "$STATUS" | node -e '...'`),
  reading `s.onboarding.drift_detail` (populated by `computeRuntimeDrift`,
  `the bee binary`). Regression test:
  `scripts/tests/test_installers_e2e.mjs` ("extra unmanaged .mjs in .bee/bin/lib/
  does not hard-fail install.sh's verify step" case). Evidence:
  `.bee/cells/installer-verify-extra-drift-1.json`.
