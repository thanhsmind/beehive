---
type: bee.area
title: Hook Runtime — the two delivery targets and what a fallback checkpoint command must do
description: "The packaged and source-repository rendering targets, why the fallback projection is derived rather than authored, and the launch contract every rendered fallback command owes: resolve the project root itself, work from any nested or non-ASCII path, run identically under both native Windows shells, and never turn a launch failure into a changed decision."
timestamp: 2026-07-22
bee:
  id: hook-runtime-delivery-targets-and-the-fallback-command
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: ["codex-runtime-parity D1, D2", "codex-hook-state-parity D1-D3, D8-D13"]
  sources: ["codex-runtime-parity Safety foundation — cells codex-parity-2, 2b, 3, 4 (traces in .bee/cells/), reports in docs/history/codex-runtime-parity/reports/", "codex-runtime-parity repo-fallback capture 2026-07-12 — cells codex-parity-6a, 6b", "codex-hook-state-parity cells 2, 3, 5 (paired Codex subagent audit, package authority, exclusive hook-source arbitration, and fresh-host handler delivery; capped traces and reports, 2026-07-16)", "codex-command-windows cells 1/2 (shell-agnostic Windows command on every codex source-repository fallback entry + both transport forms unified on the committed onboarded handler path; traces in .bee/cells/, 2026-07-20; flushed capture stub a681f64a)", "hardening-1-7-10 cells 1710-1..1710-11 (2026-07-21 — the Windows hook command becomes a shell-agnostic node -e bootstrap that resolves the git root first, exits 0 silently outside a git repo, forwards stdin, and propagates the real handler's exit status, proven by a contract test run from a nested cwd)", "docs/specs/hook-runtime.md#B7", "docs/specs/hook-runtime.md#B8", "docs/specs/hook-runtime.md#B9", "docs/specs/hook-runtime.md#R7", "docs/specs/hook-runtime.md#R8", "docs/specs/hook-runtime.md#R8a", "docs/specs/hook-runtime.md#R8b", "docs/specs/hook-runtime.md#R9", "docs/specs/hook-runtime.md#R20", "docs/specs/hook-runtime.md#E6", "docs/specs/hook-runtime.md#P10", "docs/specs/hook-runtime.md#P13"]
  authoritative_for: "hook-runtime: rendering targets and the fallback checkpoint command's launch contract"
---

# Hook Runtime — the two delivery targets and what a fallback checkpoint command must do

One runtime can load its checkpoints from two places, and the whole point of the
arrangement is that the second place is not a fork of the first: both are
rendered from the same catalog at an explicit target. What differs is the
concrete command each target produces — and a command rendered for a project's
own repository has to stand up entirely on its own, from any working directory,
under either native Windows shell, without the environment the packaged
location would have handed it.

## Entry Points & Triggers

- One runtime loads its checkpoints from two possible delivery locations: a
  packaged location, and its own project's source-repository fallback
  location. Both are rendered from the same catalog of record, at an explicit
  rendering target, using the same per-checkpoint handler wiring — the
  fallback is never a hand-authored or forked copy of the packaged rendering.

## Data Dictionary

| Element | Meaning |
|---|---|
| rendering target | Which delivery location a projection is being produced for: the packaged location, or the project's own source-repository fallback location. Same catalog, same handlers; only the concrete checkpoint command differs by target. |
| source identity | An explicit marker a rendered checkpoint command passes to its shared handler, stating which rendering target launched it, so the handler can log or branch on provenance instead of guessing from environment. |

## Behaviors & Operations

**B7 — The source-repository fallback is derived, not authored.** The
fallback delivery location's checkpoint file is produced by rendering the
catalog of record at the source-repository target. Release proof renders fresh
root and nested-directory fixtures; a checked-in project snapshot is tested
separately as a development/fallback artifact and never substitutes for package
proof (codex-runtime-parity cell 6a; codex-hook-state-parity cells 2-3).
Fresh-host proof also requires every handler filename referenced by the rendered
projection to exist in the copied handler payload; structural parity without
artifact delivery is not a working fallback (codex-hook-state-parity cell 5).

**B8 — Fallback checkpoint commands are environment-independent.** A
checkpoint command rendered for the source-repository target does not depend
on any environment variable that only the packaged delivery location
provides. Instead, at launch it resolves the project root itself from the
current working directory and only then hands off to the shared checkpoint
handler, passing an explicit source identity so the handler knows it was
launched from the fallback rather than the packaged location. This
resolve-then-handoff step succeeds from the project root and from any working
directory nested below it, including paths containing spaces or non-ASCII
characters (codex-runtime-parity cells 6a, 6b).

**B9 — Launch-setup failure and a computed decision are different things.**
Before a fallback checkpoint command reaches its shared handler, it must
complete its own root-resolution/launch setup. If that pre-handoff step
cannot complete, the checkpoint today fails open **visibly**: it writes
exactly one diagnostic to the error stream, writes nothing to the output
stream, and exits success — never the silent crash this replaced. Once the
shared handler is reached, its own outcome — ordinary success or a deliberate
denial — passes through the launch step unchanged (codex-runtime-parity cell
6a; see Open Gaps — this default is under active revision for checkpoints
capable of denial).

## Business Rules

- R7 — The source-repository fallback checkpoint file is generated from the
  catalog of record at an explicit rendering target and is never
  hand-authored; the installation suite reproduces the rendering and fails on
  any byte drift (codex-runtime-parity cell 6a).

- R8 — A source-repository fallback checkpoint command must not depend on
  environment that only the packaged delivery location provides; it resolves
  the project root itself at launch and must succeed from the project root
  and from nested working directories, including paths with spaces and
  non-ASCII characters (codex-runtime-parity cells 6a, 6b).

- R8a — Every second-runtime source-repository fallback entry carries two
  transport forms: the POSIX command (login-shell transport, unchanged) and a
  Windows-specific command that is shell-agnostic — with no shell
  substitution, test, or exec constructs, and no dollar sign, percent sign, or
  backtick anywhere in the command string, so cmd.exe and PowerShell parse it
  identically instead of one native shell interpreting a construct the other
  does not — so it runs identically under both native Windows shells. The
  command is a small interpreter-language bootstrap, not a raw file
  invocation: it resolves the git repository root itself before doing
  anything else, so it works correctly from any nested working directory
  rather than only from the session's own working directory; outside a git
  repository it exits zero silently, the same fail-open parity the POSIX form
  already has; once resolved, it forwards its own stdin through to the real
  checkpoint handler unchanged and propagates that handler's exit status back
  as its own, rather than always reporting success (hardening-1-7-10,
  refining codex-command-windows cell 1's original shape). The contract test
  for this command executes the real, rendered command string from a nested
  working directory rather than only asserting on its literal text, so a
  regression in the git-root resolution itself would fail the suite, not just
  a text-shape check. The Windows form exists only
  on the second runtime's source-repository target: packaged (plugin)
  projections and all first-runtime projections are byte-unchanged by its
  rendering (codex-command-windows cell 1).

- R8b — Both transport forms reference one checkpoint path: the committed,
  onboarded handler location inside the workflow's own tool directory, which
  is tracked in version control so a fresh clone resolves it without an
  onboarding pass. The top-level handler source directory is plugin source
  only and never appears in a rendered fallback command (codex-command-windows
  cell 2, author decision).

- R9 — A fallback checkpoint's pre-handoff launch-setup failure fails open
  visibly today — one diagnostic on the error stream, nothing on the output
  stream, success exit — while the shared handler's own decision (ordinary
  success or deliberate denial) passes through that launch step unchanged
  (codex-runtime-parity cell 6a; under revision for deny-capable checkpoints,
  see Open Gaps).

- R20 — A Windows fallback checkpoint command resolves the git repository
  root itself before anything else, so it behaves identically whether the
  session's cwd is the project root or a nested directory; it contains no
  dollar sign, percent sign, or backtick, so cmd.exe and PowerShell parse it
  identically; and its own contract test runs the real rendered command from
  a nested cwd rather than only checking its literal text (R8a;
  hardening-1-7-10).

## Edge Cases Settled

- A fallback checkpoint's project-root resolution succeeds identically from a
  working directory whose path contains spaces and non-ASCII characters as it
  does from a plain path (proven against an isolated fixture, codex-runtime-parity
  cell 6b).

## Pointers (implementation)

- Package/fallback distribution proof: `packages/bee/scripts/plugin_distribution.mjs`
  and `packages/bee/scripts/tests/test_plugin_distribution.mjs`. The checked-in
  `.codex/hooks.json` is a development/fallback snapshot, not package proof.

- Codex source-repository fallback: `.codex/hooks.json`, generated only by
  `renderProjectionText("codex", { target: "repo" })` — never hand-authored;
  current runtime contract: `https://learn.chatgpt.com/docs/hooks`.

## Open Gaps

- The source-repository (dogfood) deny-capable checkpoint is a **guardrail
  against honest mistakes, not a security boundary against a hostile in-project
  actor** — per D2, hooks provide enforcement "without pretending hooks are a
  complete security boundary." A proposed hardening (cell 6c) to make the
  repo-fallback deny checkpoint spoof-proof was **scoped out and stopped**
  (decision f398aa60): three validation rounds each proved a new bypass in
  resolving the checkpoint's own root/handler from an untrusted working
  directory — nearest-ancestor marker resolution is exploitable downward,
  outermost-ancestor upward, both defeatable by a directory symlink under a
  write-allowlisted prefix (lexical vs realpath), and marker files can be
  planted through Bash primitives the write-target extractor does not model
  (`dd`, `install`, `python -c open`, `rsync`) — the same ungoverned-write
  class named in the first gap above. These are **recorded known limits of the
  guardrail, not defects to be closed by ever-more root-resolution hardening.**
  Two accidental (non-adversarial) failure modes remain open and would be the
  only justification for a future *minimal* fix: on a bare repository the
  pre-handoff resolution fails open (ALLOW) rather than closed, and a foreign
  nested working directory can make the deny checkpoint's launch crash rather
  than emit a visible diagnostic. A minimal fix (resolve the checkpoint root the
  same way the handler does, and fail the deny checkpoint closed on launch-setup
  failure) is available as a small fresh cell if ever wanted; it is not planned.
- Native-Windows sessions are now covered by the separate shell-agnostic
  Windows command on every fallback entry (R8a), but a non-POSIX Unix login
  shell (e.g., fish, nu) still receives the POSIX command through the
  login-shell transport and has no declared equivalent; that narrower gap
  remains open (codex-command-windows closed only the Windows half).
