---
type: bee.area
title: Onboarding — installer entry points and source staging
description: "How a bootstrap install fetches the workflow source without materialising a full working tree, why the staged copy must carry the complete release identity, the parity the two platform entry points owe each other, and what happens when a runtime's tool is present but broken."
timestamp: 2026-07-26
bee:
  id: onboarding-installer-entrypoints-and-source-staging
  lifecycle: active
  areas: [onboarding]
  required_context: [areas/onboarding/overview.md]
  decisions: [09b776b5 (both installers prove complete greenfield/brownfield postconditions before success), 17bfc14a (Codex-safe onboarding tests preserve the real CLI entrypoint and observable process contract through an isolated Worker), a83a3613 (shared isolated runner for nested Node entrypoints; real Git/Bash/Codex integration remains external), "e0f3e40e (packages-restructure D1-D5: vendor payload relocated to packages/bee/, skills instruction-only, PLUGIN_ROOT-relative resolution)", "80b64c20 (packages-engine-move D1-D5: onboarding/distribution engine relocated to packages/bee/scripts, strict-flag validation universal, migration-tooling pattern)"]
  sources: ["gh-issue-fixes-172 cell ghf-2 (GH #26: Windows staged source carries the full release identity — sparse checkout includes both package manifests; absent-package removal probed before attempted; trace in .bee/cells/, 2026-07-20)", "installer-probe-quiet cells installer-probe-quiet-1/-2 (tolerated runtime-CLI probe failures — captured stderr, one condensed warning per broken tool, repo-copy proceeds / plugin-first names the broken tool; reports docs/history/installer-probe-quiet/, 2026-07-20)", "codex-sandbox-baseline cells codex-sandbox-baseline-1/codex-sandbox-baseline-2 (real onboarding entrypoint through the shared isolated test runner; full onboarding suite green, 2026-07-16)", "installer-version-parity-1-3-1 D2/D8 Linux Bash E2E (cells -3, 2026-07-16)", "docs/specs/onboarding.md#R20b", "docs/specs/onboarding.md#R23", "docs/specs/onboarding.md#R27", "docs/specs/onboarding.md#P1", "docs/specs/onboarding.md#P2", "docs/specs/onboarding.md#P4", "docs/history/packages-restructure/ (cells packages-restructure-1..4, 2026-07-25/26: vendor payload relocation, hook catalog move, distribution-surface roles, prose sweep)", docs/history/packages-engine-move/, docs/history/install-tty-fix/]
  authoritative_for: "onboarding: installer entry points and source staging"
---

# Onboarding — Installer Entry Points and Source Staging

Before onboarding can decide anything about a host project, the installer has to
*have* a source and *be runnable at all*. This concept owns that outer ring: what a
bootstrap install fetches, what the staged copy must contain to be a legitimate
source, what the two platform entry points owe each other, and how a runtime whose
command-line tool is present but broken is handled. Every rule here shares one
failure shape — an install that proceeds against something incomplete and then blames
the wrong cause.

## Behaviors & Operations

**Fetch the workflow source without a full working tree (bootstrap installs).**
Trigger: an install invoked with no local source, so the installer must fetch the
workflow from its published repository at some reference. What changes: the fetch
checks out only the trees the installer actually reads — the instruction-only
skill set, the vendored payload tree, and both platform manifests — never the
whole tree. The vendored payload is one standard code set (the CLI, its
libraries, tests, and status-display scripts), with its hook catalog and
handlers as a labeled subtree of that same payload; the skill set itself now
carries instructions only. Every path onboarding resolves against that payload
is relative to its own plugin root, never self-relative to wherever the
onboarding engine's own script happens to sit (packages-restructure D1-D3;
decision e0f3e40e). The engine itself later moved out of the skill tree it
renders (packages-engine-move D1; decision 80b64c20): `onboard_bee.mjs`,
`plugin_distribution.mjs`, and their three test suites now live at
`packages/bee/scripts/`, so `packages/bee/` is the complete standard code set
— payload *and* engine — and `skills/` is instruction-only in fact, not just
intent. The canonical entrypoint is
`node packages/bee/scripts/onboard_bee.mjs --repo-root <repo-root>`, run from
a source root (a checkout or an installed plugin package), never from a
projection. Why it matters: the workspace filesystem
of one supported platform rejects several characters that the source platform
allows in filenames (colon, asterisk, question mark, quote, angle brackets,
pipe), plus reserved device names and trailing dots or spaces; a single such path
anywhere in the reference aborts the *entire* working-tree materialisation, and
the install would otherwise proceed against an empty source and blame the
network. Narrowing the checkout makes the install independent of every path the
installer never reads, on any reference including historical released ones. After
the fetch the installer probes for the workflow's own bootstrap script and stops
with an explicit source error if it is absent — an empty source is never mistaken
for a network fault again. Companion rule (the other half of the same guarantee):
**every tracked path in this repository must be checkout-able on the restrictive
platform**, and the repository's verification command fails when any tracked path
carries a forbidden character, a reserved device name, or a trailing dot/space.

## Business Rules

- **R20b** — The staged source copy an installer fetches always contains the
  COMPLETE release identity: every version-marker file the onboarding
  release-tuple check requires (both assistant-package manifests included) is
  part of what gets staged, on every platform — a partial staging that passes
  its own probe but fails the tuple check downstream is the defect this rule
  bans (GH #26: the Windows entry point staged one manifest but not the other,
  producing `blocked_no_source` on an otherwise healthy install). Removal of a
  previously installed package during a fallback install is attempted only when
  a status probe shows the package actually present — attempting removal of an
  absent package, even harmlessly, must not surface an error line to the user
  (gh-issue-fixes-172 cell ghf-2, 2026-07-20).
- **R23 (not yet implemented — installer-version-parity-1-3-1)** — The Linux and
  Windows entry points prove the same observable outcomes for a new project and
  an already-onboarded project: all required onboarding information exists,
  owner content and prior workflow state survive an upgrade, managed capability
  copies move to the requested release, and repeating the run is idempotent
  (decision 09b776b5).
- **R27** — A runtime whose command-line tool is present on the path but not
  actually runnable (its capability probe exits with an error) does not by
  itself fail a repository-copy install: the probe result is treated as "no
  packaged capability found", the install proceeds, and the user sees exactly
  one short warning line per broken tool — never the tool's raw error output
  streamed through. A package-first install still refuses when its required
  tool is broken, but the refusal names the specific broken tool, shows its
  captured error, and states the concrete ways forward (repair the tool, choose
  the repository-copy source, or exclude that runtime). The tolerance and the
  one-warning-per-tool discipline apply identically to every supported runtime
  and on every platform. On platforms whose shell turns a native command's
  error stream into its own failure, the probe captures that stream through a
  guarded helper rather than a bare redirection, so a broken tool never crashes
  the installer (installer-probe-quiet-1 4799236 + release 1.7.5 c46a9e4;
  supersedes the earlier "probe failure is fatal" reading, 2026-07-20).

## Edge Cases Settled

- A non-interactive install run (no usable terminal) without the explicit accept-all option fails safe: it stops with guidance naming that option, never a crash — the terminal probe performs a real open, not a permission-bit check.

## Pointers (implementation)

- `packages/bee/` — the vendored payload's single standard layout (`bee.mjs`,
  `lib/`, `tests/`, `agents/`, `statusline/`, `AGENTS.block.md`), with
  `packages/bee/hooks/` as its hook-catalog subtree (hook modules, `catalog.mjs`,
  `claude-hooks.json`, `hooks.json`, and their test suites) and
  `packages/bee/scripts/` as the onboarding/distribution engine subtree
  (`onboard_bee.mjs`, `plugin_distribution.mjs`, `tests/`) — moved out of the
  skill tree it renders (packages-engine-move D1; decision 80b64c20); `skills/`
  now carries instruction-only content (`SKILL.md` + references only, no
  runnable engine of its own) — no version marker of its own travels inside a
  synced skill dir anymore. `onboard_bee.mjs`'s `PLUGIN_ROOT`, `TEMPLATES_DIR`, and
  `PLUGIN_HOOKS_DIR` resolve every payload path `PLUGIN_ROOT`-relative, the
  same mechanism `PLUGIN_HOOKS_DIR` already used before the move (`../templates`
  self-relative resolution retired). `scripts/install.ps1`'s bootstrap
  sparse-checkout set is `skills packages .claude-plugin .codex-plugin
  docs/history/codex-harness-hardening` (`scripts/install.sh`'s full clone is
  unaffected by the move). Evidence: `.bee/cells/packages-restructure-1.json`,
  `.bee/cells/packages-restructure-2.json` (packages-restructure D1-D3;
  decision e0f3e40e).
- `scripts/lib/run-module-worker.mjs` — shared isolated test-entrypoint runner;
  preserves arguments, environment, stdout, stderr, and exit status without
  changing the production entrypoint.
- `packages/bee/scripts/tests/test_onboard_bee.mjs` — the complete onboarding
  suite keeps its real and fixture-local entrypoints and all prior assertions
  while routing nested Node launches through the shared runner.
- `scripts/install.sh`, `scripts/install.ps1`, `.codex-plugin/plugin.json`,
  `.claude-plugin/marketplace.json`, and the release inventory/tuple tests —
  package wiring, cross-platform entrypoints, metadata, inventory, and staged
  cachebuster proof. Evidence: `.bee/cells/codex-hook-state-parity-3.json` and
  `docs/history/codex-hook-state-parity/reports/codex-hook-state-parity-3.md`.
