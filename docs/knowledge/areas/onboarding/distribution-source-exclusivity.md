---
type: bee.area
title: Onboarding — distribution sources and their exclusive transition
description: "Selecting and proving exactly one distribution source per install, the Codex hybrid carve-out, the fenced cleanup in both directions, and the whole-run snapshot revalidated immediately before the first mutation."
timestamp: 2026-07-22
bee:
  id: onboarding-distribution-source-exclusivity
  lifecycle: active
  areas: [onboarding]
  required_context: [areas/onboarding/overview.md]
  decisions: [4cc1c355 (Codex plugin-first distribution), cf511ff3 (plugin/package and repo-copy sources are mutually exclusive; cleanup is integrity- and ownership-proof-gated in both directions), codex-hook-state-parity D9-D14, "3318374a (installer hardening: per-project skills default, global opt-in)", "codex-plugin-first-hybrid D1-D6 (GH #22 P0-1)"]
  sources: ["codex-hook-state-parity cells 2, 3, 5 (paired Codex lifecycle audit, exclusive plugin-first/repo-copy distribution, and fresh-host handler delivery; capped traces and reports, 2026-07-16)", "installer-version-parity-1-3-1 D7 managed-set cleanup fencing (cell installer-version-parity-1-3-1-2, 2026-07-16)", "codex-runtime-parity D1 (distribution contract, 2026-07-11)", "docs/specs/onboarding.md#R5", "docs/specs/onboarding.md#R12", "docs/specs/onboarding.md#R18", "docs/specs/onboarding.md#R19", "docs/specs/onboarding.md#R20", "docs/specs/onboarding.md#R24", "docs/specs/onboarding.md#R25", "docs/specs/onboarding.md#E1", "docs/specs/onboarding.md#P3", "docs/specs/onboarding.md#P6"]
  authoritative_for: "onboarding: distribution sources and their exclusive transition"
---

# Onboarding — Distribution Sources and Their Exclusive Transition

One installation activates exactly one source, so a skill or event never runs twice.
That single sentence generates everything in this concept: which source is the
default, what must be *proven* before a transition is allowed in either direction,
what a cleanup pass may and may not delete, and the snapshot revalidated immediately
before the first byte is written. The one deliberate exception — the Codex hybrid —
is part of plugin-first, never a third mode.

## Entry Points & Triggers

- Plugin-capable installs default to a plugin-first check/apply transaction;
  `repo-copy` is an explicit fallback mode.
- A dry run plans the complete distribution transaction and mutates nothing.

## Data Dictionary

| Element | Meaning |
|---|---|
| distribution mode | The exclusive source selected for one install: `plugin-first` or explicit `repo-copy`. The two are never active together. Codex carve-out (hybrid): because the Codex CLI loads no plugin hooks, plugin-first on a Codex-covering install still writes the repo-local Codex hook projection — skills from the plugin, hooks repo-local — and this is part of plugin-first, not a third mode. |
| release inventory | The complete, duplicate-free file set and package metadata that an enabled installed package must match before cleanup is authorized. |
| ownership ledger | The installer's exact record of user-runtime roots and directories it created; name similarity alone never grants deletion authority. |
| recognized bee hook entry | A hook entry whose event, matcher, and handler match the generated bee catalog. Foreign and user entries are never recognized by name alone. |
| whole-run snapshot | The inputs revalidated immediately before mutation: paths, aliases, symlinks, package status, inventory, ledger, and hook shapes. Any mismatch aborts the entire run with zero writes. |

## Behaviors & Operations

**Select and prove exactly one distribution source.** Plugin-first is the
default on a capable runtime. It proves an enabled installed package and its
complete release inventory, preflights the whole transaction, then removes only
direct plain `bee-*` skill directories and catalog-recognized bee hook entries
from project fallback roots. **Codex hybrid rule (GH #22 P0-1):** when the
selected runtime covers Codex, plugin-first additionally writes the repo-local
Codex hook projection (hook file + vendored handlers) in the same apply,
because the Codex CLI loads no plugin hooks — and the cleanup pass is scoped
so it never strips the Codex hook entries it just gained (the claude-side
stripping is unchanged). The projection fires from the caller-passed runtime
selection, never from recorded state. A failed hook write is a typed blocked
apply that aborts and rolls the plugin back — an install can never end with
plugin skills and no hooks. Repo-copy first proves the package inactive, then
generates the managed project projections; Codex-only receives the same hook
catalog as a combined-runtime install. Bash and PowerShell installers use the
same planner and proof rules. A symlink, alias, unknown target, invalid ledger,
package mismatch, or hook-shape mismatch aborts before any write. Release proof
uses a staged cachebuster without changing the canonical package/version tuple;
live user-home installation and fresh-thread loading remain outstanding UAT.
Every generated repository hook command must resolve to a handler included in
the same fresh-host onboarding payload; projection topology without referenced
file delivery is a failed install, even when catalog parity itself is green.

**Install skills into the project itself (every install/apply).** Trigger: an
install or apply against a host project. What changes: the workflow's skill set
is synced into the host project's own skill-discovery locations — one per
supported assistant runtime — and those copies are version-tracked in the host,
so every teammate receives working skills with a plain checkout. A machine-global
skill install happens only on an explicit opt-in switch, never by default; and
when the target is the workflow's own source tree, the per-project copy is
skipped (the source is already authoritative there). What each actor observes: a
fresh clone of an onboarded host has working skills with zero machine-level
setup; an operator who wants one shared machine-wide set asks for it explicitly.

R12 below is the later, narrower reading of that behavior: per-project copies are
created only by the explicit repo-copy fallback, and the Codex hook projection is
the one carve-out.

## Business Rules

- **R5** — Plugin-capable runtimes
  receive bee primarily as one installable package containing the shared workflow
  skills and compatible lifecycle hooks. Release and reinstall update the package
  the runtime loads directly; project-local skill and hook projections remain an
  explicit fallback and dogfood route. One installation activates exactly one
  source so a skill or event never runs twice (decision 4cc1c355, extended by
  codex-hook-state-parity D9; decision cf511ff3).
- **R12** — Plugin-first is
  the default distribution when the selected runtime can install the package.
  Per-project copies are created only by the explicit repo-copy fallback; a
  machine-global copy remains explicit opt-in. The workflow's source tree is not
  a host-install target (codex-hook-state-parity D9/D10; supersedes the
  default-copy portion of decision 3318374a; decision cf511ff3). Codex
  carve-out: the repo-local Codex hook projection is exempt from "per-project
  copies only via repo-copy" — plugin-first on a Codex-covering install always
  writes it (hybrid), fail-closed, with the cleanup pass scoped to preserve it
  (codex-plugin-first-hybrid D1-D6, GH #22 P0-1).
- **R18** — Plugin-first migration
  cleans duplicate skills and bee hook entries only after the installed package is
  reported enabled and its installed skills/hooks match the release inventory;
  command success alone is not proof. Skill cleanup candidates are derived by
  exact name from the `plugin_skill` records of the validated release inventory —
  never by name prefix — within the selected repository's Claude, shared-agent,
  and Codex skill roots; a directory whose name is not in that managed set (a
  project-owned `bee-custom` included) is skipped before any check. A missing,
  malformed, duplicate, or inconsistent inventory (zero managed skills, a
  non-`bee-` managed name, a bad path) refuses the whole cleanup before any
  mutation. Hook cleanup removes only catalog-recognized bee entries and preserves
  user entries and their container files. Non-bee entries, files, symlinks,
  aliases, unknown targets, and paths outside those roots make the whole cleanup
  refuse before mutation (codex-hook-state-parity D10/D11/D13; decision cf511ff3;
  managed-set fencing: installer-version-parity-1-3-1 D7, cell
  installer-version-parity-1-3-1-2, 2026-07-16).
- **R19** — Repo-copy fallback is
  the reverse exclusive transition: the installer first disables or uninstalls the
  bee plugin and verifies it inactive, then creates managed repository copies. If
  deactivation cannot be proven, nothing is copied or removed. User-runtime cleanup
  additionally requires a valid installer ownership ledger naming the exact root
  and directories; a name match alone does not authorize global deletion
  (codex-hook-state-parity D12/D14; decision cf511ff3).
- **R20** — Immediately before the first mutation, the installer revalidates a
  whole-run snapshot. Any path, symlink, alias, package, inventory, ledger, or
  hook-shape mismatch aborts the transaction with zero writes.
- **R24 (not yet implemented — installer-version-parity-1-3-1)** — Removing
  project fallback capabilities requires proof that each candidate belongs to
  the managed release set. Sharing the workflow's name prefix is not ownership;
  a project-owned capability remains untouched.
- **R25 (not yet implemented — installer-version-parity-1-3-1)** — Planning,
  preview, and dry-run never install, remove, or change a runtime package.
  Package transition begins only after confirmation and is exercised in an
  isolated environment that proves the ordering rather than bypassing it with a
  prewritten status response.

## Edge Cases Settled

- Skill distribution renders per runtime (codex-native-runtime-v2): source skill
  files may carry full-line runtime-conditional block markers; the sync stage
  filters each managed root's copy (claude-class targets drop codex blocks and
  vice versa, marker lines always stripped, unmarked content shared). A file
  with no markers passes through byte-identically (BOM/CRLF/final-newline
  preserved). Malformed markers (nesting, unclosed, stray end, unknown label,
  frontmatter or fenced placement) refuse the ENTIRE apply loudly with zero
  writes. Per-target drift compares installed bytes against the rendered form;
  release/package hashing stays canonical-source bytes; the downgrade preflight
  stays version-based. Rendered targets carry provenance metadata and are
  refused as onboarding sources for ANY target — canonical or plugin source
  required. Plugin routes ship committed per-runtime rendered trees, drift-
  pinned to a test-time re-render of canonical.

## Open Gaps

- Plugin-first/repo-copy implementation, fixture metadata, release inventory,
  lifecycle mapping, and repository verification are green. Real user-home
  install/reinstall plus fresh-thread loading remain outstanding because this
  environment exposes the Codex home read-only.

## Pointers (implementation)

- `packages/bee/scripts/plugin_distribution.mjs` and
  `packages/bee/scripts/tests/test_plugin_distribution.mjs` — shared strict
  planner/prover and the 22-case transaction suite used by both installers.
- Fresh-host handler-delivery proof: `.bee/cells/codex-hook-state-parity-5.json`
  and `docs/history/codex-hook-state-parity/reports/codex-hook-state-parity-5.md`.
