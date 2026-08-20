---
type: bee.area
title: Onboarding — the artifacts a host project receives and keeps
description: "The instructions import created by default, the single unified command surface that retires nine superseded helper scripts, the state-layer landing pages created but never overwritten, and the annotated configuration sample an operator copies from."
timestamp: 2026-07-22
bee:
  id: onboarding-host-project-artifacts
  lifecycle: active
  areas: [onboarding]
  required_context: [areas/onboarding/overview.md]
  decisions: ["3318374a (installer hardening: default instructions import, D1)", "bbc6bcea (shim-retire: unified command surface; retired helper scripts removed from hosts)", de967733 (advisor mode removed; stale config key warned-and-ignored), capture flush b57f6470 (shipped configuration sample is the annotated reference; 2026-07-19)]
  sources: ["shim-retire D2 retirement pass (cells shim-retire-2, shim-retire-6 self-onboard proof, 2026-07-14)", "installer-hardening ih-1..ih-6 (cells, 2026-07-13; flushed capture stub 92c9bcf6)", "fanout-delegation D1 (stale advisor key tolerance, 2026-07-12)", "capture stub b57f6470-bac2-422a-9dea-1bb4cc93bc0e (shipped config sample carries a per-surface doc block, all model-slot shapes, gate-bypass levels, hook toggles, and the external-command tier's gather-only contract; flushed 2026-07-19)", "docs/specs/onboarding.md#R13", "docs/specs/onboarding.md#R14", "docs/specs/onboarding.md#E6", "docs/specs/onboarding.md#E15", "docs/specs/onboarding.md#P14"]
  authoritative_for: "onboarding: the artifacts a host project receives and keeps"
---

# Onboarding — The Artifacts a Host Project Receives and Keeps

Beyond the distribution question of *where* things come from, a host project ends up
holding a small set of concrete artifacts: an import line that makes the standing
instructions load in a fresh session, exactly one vendored command surface, two
state-layer landing pages, and a configuration file whose shipped sample explains
itself. The discipline shared by all of them is the same: create by default, never
replace the owner's content without consent, and delete only what is exactly named.

## Behaviors & Operations

**Provide the assistant-instructions import by default.** Trigger: onboarding a
host whose assistant reads a project instructions file that can import the
standing instruction sheet. What changes: the import artifact is created (or its
managed import line added) by default; declining it is an explicit opt-out
switch, not an omission. Existing content outside the managed line is never
replaced without consent. What the human observes: a freshly onboarded project
"just works" in a new session without manually wiring instructions.

**Carry the shell doctrine a PowerShell host needs (every run).** Trigger:
onboarding a project whose resolved host shell is PowerShell. What blocks it:
nothing. What changes: an Environment section is rendered INSIDE the managed
instruction block — never as separate prose — stating which commands belong to
which shell (the user's own terminal is PowerShell; the agent's own shell tool,
and anything the user runs through the session's command prefix, is the POSIX
one), the PowerShell default for anything handed to the user, and the three
platform facts that break silently: path separator form inside a POSIX-tool
command, line endings, and executable-bit semantics on the platform's file
system. What the human observes: a freshly onboarded Windows project stops
receiving commands its own terminal cannot run.

The **project** decides that shell before the machine does: a recorded setting
in the project's own configuration wins, and only when it is absent or
unrecognised does the running host answer. Detection alone would churn — a
teammate on the other platform strips the section that the next run puts back —
and an unrecognised value falls back rather than refusing, so the setting is
always optional. Because the section lives inside the managed block, the same
splice that adds it removes it again when the answer changes.

**Retire superseded helper scripts (every run).** Trigger: any run against a
host that still carries one of the nine retired per-command helper scripts in
its vendored tools directory — hosts onboarded before the command surface was
unified into the single dispatcher. What blocks it: nothing. What changes: the
check run plans one removal per leftover retired script; the apply run deletes
exactly those files. Removal is scoped to the exact retired filenames inside the
managed tools directory — no other file is ever deleted by this pass. Side
effects: none. What each actor observes: after one apply, the host's tools
directory carries only the unified dispatcher with its libraries and guardrails;
a second run plans zero removals; a freshly onboarded host never receives the
retired scripts at all. The installer's own post-install verification and its
printed quickstart also speak only the unified dispatcher's status command.

**Guarantee the state-layer landing pages (every apply run).** Trigger: any
apply where the project lacks its reading map or its system overview in the
specs area. What blocks it: nothing. What changes: each missing file is created
as a small skeleton that names its owner (the spec-sync discipline) and points
the reader to a bootstrap pass; an existing file is NEVER touched, drifted or
not — content belongs to spec-sync, existence belongs to onboarding. What the
human observes: "read the spec before the code" and "where does X live" have a
landing page from day one in every onboarded project.

**Seed the annotated config sample (every apply run).** Trigger: any apply where
the project has no config sample of its own. What blocks it: nothing. What
changes: the sample is created from a copy of bee's own, embedded in the binary
at build time so the seeded file cannot drift from the one the project reads;
an existing sample is NEVER touched. What the human observes: a new project can
read every configurable surface, annotated key by key, without going to find
bee's source.

(The plan action that creates those two landing pages is named in
[`repo-local-guardrails.md`](repo-local-guardrails.md)'s pointer, which carries the
same source bullet.)

## Business Rules

- **R13** — The assistant-instructions import artifact is created by default on
  onboarding; declining it is an explicit opt-out. Content outside the managed
  import is never replaced without consent (decision 3318374a, D1).
- **R14** — The vendored command surface is a single unified dispatcher. The
  nine retired per-command helper scripts are deleted from a host on its next
  apply; removal is scoped to the exact retired filenames inside the managed
  tools directory and is idempotent (decision bbc6bcea, D1/D2).
- **R15** — The template source root is resolved from the INVOCATION's own
  project root (walking up from the working directory), never from the
  executable's physical location: a granted worktree whose vendored binary
  is a symlink into the main checkout renders its OWN checkout's templates,
  so an apply can no longer splice the main checkout's managed block over
  worktree edits. A root carrying no templates refuses with a typed error
  naming the invocation root and the missing template path — never a silent
  fallback to the executable's checkout (onboard-root-resolution cell orr-1,
  2026-08-10; live overwrite incident, backlog row 659). The walk is
  BOUNDED since review-p2-hardening cell rph-3 (2026-08-11): it stops at
  the first directory containing `.git`, so an ancestor beyond the
  repository boundary can never become the template source (review B-P2-3,
  template-poisoning consequence); and an explicit `--repo-root` is tried
  FIRST as a locate candidate, so onboarding a named bee checkout works
  from any working directory — a refusal names both candidates. Test
  discipline riding rph-3: every locate test neutralizes ambient
  `BEE_JS_ENTRY` with a scoped guard (the env short-circuit fires before
  the walk and had made the suite red under an exported entry), and
  `Engine::locate()` stays a pure one-line delegate so the `locate_from`
  pins cover production.

## Edge Cases Settled

- A host `.bee/config.json` still carrying the removed `advisor` key → parses
  normally with the key stripped from the parsed result; the onboarding report
  and the status command each surface one identical warning line telling the
  owner to delete it — warn, never error (feature fanout-delegation D1, decision
  de967733; the duplicated warning text is pinned by a drift test).
- The shipped configuration sample is annotated, not silent: every top-level
  surface carries an explanatory block alongside its example values —
  covering every hook's kill switch, every supported model-slot shape (a
  plain model name, model-plus-effort, an external-command executor, and an
  unset/budget slot), all four gate-bypass levels, and the external-command
  tier's gather-only contract. It is the same copyable reference an operator
  starts from and the one this document's own contracts are checked against
  (capture flush b57f6470, 2026-07-19).

## Pointers (implementation)

- Windows shell section: template `packages/bee/AGENTS.windows.md`, appended by
  `render_agents_block` and resolved by `host_shell_is_powershell` (config key
  `host_shell`, else `cfg!(windows)`) in
  `packages/bee-rs/crates/bee/src/onboard/merge.rs`; path field
  `Engine::agents_windows_template` (`onboard/source.rs`), wired at the
  `onboard/plan.rs` render call. Tests:
  `a_powershell_host_carries_the_shell_section_inside_the_same_markers`,
  `a_posix_host_renders_exactly_what_the_single_template_form_rendered`,
  `the_repository_decides_the_host_shell_before_the_machine_does`
  (`onboard/merge.rs`). Provenance: cell `wsd-1`, commit 8b93d749.

- `.bee/config-sample.json` — the annotated, copyable configuration reference
  (`_doc` block per top-level key); `.bee/config-sample-cli-executors.json` —
  full external-command executor examples; `scripts/tests/test_config_samples_safe.mjs`
  keeps both inert (never diffs against the live `.bee/config.json`).
