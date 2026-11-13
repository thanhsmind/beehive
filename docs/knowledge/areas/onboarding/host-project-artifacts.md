---
type: bee.area
title: Onboarding — the artifacts a host project receives and keeps
description: "The instructions import created by default, the Claude settings baseline projected add-only into the host's .claude/settings.json (permissions.defaultMode bypassPermissions, the denied tools, the top-level switches) and its opt-out, the single unified command surface that retires nine superseded helper scripts, the state-layer landing pages created but never overwritten, and the annotated configuration sample an operator copies from."
timestamp: 2026-07-22
bee:
  id: onboarding-host-project-artifacts
  lifecycle: active
  areas: [onboarding]
  required_context: [areas/onboarding/overview.md]
  decisions: ["claude-settings-baseline D1-D7 (onboarding projects the owner's Claude settings baseline add-only into the host's own .claude/settings.json, on by default, with a durable opt-out; the projection announces itself whenever it runs; CONTEXT.md)", "verification-ships-to-hosts D1/D2/D3 (a host project also receives a verification skill pair and, once generated, its own project verification skill rendered into every runtime home; the generated drive command composes into the declared test command)", "0d3e4f89 (that render creates and updates only, never removes)", "3318374a (installer hardening: default instructions import, D1)", "bbc6bcea (shim-retire: unified command surface; retired helper scripts removed from hosts)", de967733 (advisor mode removed; stale config key warned-and-ignored), capture flush b57f6470 (shipped configuration sample is the annotated reference; 2026-07-19)]
  sources: ["claude-settings-baseline cell csb-1 (2026-11-10; the baseline constant, the add-only merge, the durable opt-out and the notice)", "claude-settings-baseline cell csb-2 (2026-11-10; installer pass-through in install.sh and install.ps1)", "verification-ships-to-hosts cells vsh-1..vsh-4 (2026-09-01; the skill pair, the copy-only render path, the offer notice and the consent split — capped traces, independently goal-checked, and driven end to end against a throwaway host repo)", "shim-retire D2 retirement pass (cells shim-retire-2, shim-retire-6 self-onboard proof, 2026-07-14)", "installer-hardening ih-1..ih-6 (cells, 2026-07-13; flushed capture stub 92c9bcf6)", "fanout-delegation D1 (stale advisor key tolerance, 2026-07-12)", "capture stub b57f6470-bac2-422a-9dea-1bb4cc93bc0e (shipped config sample carries a per-surface doc block, all model-slot shapes, gate-bypass levels, hook toggles, and the external-command tier's gather-only contract; flushed 2026-07-19)", "docs/specs/onboarding.md#R13", "docs/specs/onboarding.md#R14", "docs/specs/onboarding.md#E6", "docs/specs/onboarding.md#E15", "docs/specs/onboarding.md#P14"]
  authoritative_for: "onboarding: the artifacts a host project receives and keeps"
  anchors: [.claude/settings.json, packages/bee-rs/crates/bee/src/onboard/hooks_wiring.rs, packages/bee-rs/crates/bee/src/onboard/notices.rs, packages/bee-rs/crates/bee/src/onboard/templates.rs, packages/bee-rs/crates/bee/src/onboard/apply.rs, packages/bee/AGENTS.windows.md, packages/bee-rs/crates/bee/src/onboard/merge.rs, packages/bee-rs/crates/bee/src/onboard/plan.rs, packages/bee-rs/crates/bee/src/onboard/mod.rs, scripts/install.sh, scripts/install.ps1, skills/bee-deploying/references/deploy.sh.template, skills/bee-deploying/references/DEPLOY.template.md]
  verified_at: 2026-11-10
  verified_commit: 32878a03635a413a538709e9ca05050fccd6e6fb
---

# Onboarding — The Artifacts a Host Project Receives and Keeps

Read when: deciding what `bee onboard` writes into a host project, debugging why
an onboard run kept or overwrote one of the host's own files, or changing the
`.claude/settings.json` projection — the permission-mode baseline, its
add-only rules, or its opt-out.

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

**Project the Claude settings baseline (every run, unless opted out).**
Trigger: any onboard run against a repo whose `.claude/settings.json` is
missing, and whose planned baseline merge would change a byte. What blocks it:
nothing. What changes: one add-only merge writes the owner's Claude settings
baseline into the host repo's OWN settings file — `permissions.defaultMode`
`bypassPermissions`, `EnterWorktree` and `ExitWorktree` allowed, thirteen tools
denied, and six top-level switches — appending allow and deny names after the
existing entries in baseline order, writing `defaultMode` only when
`permissions` carries none, and writing a top-level key only when it is absent.
Every other key, entry and order stays in place, and no value the project
already set is overwritten. A parse failure, a non-object top level, a
non-object `permissions`, or a non-array `allow` or `deny` leaves the text
unchanged and plans nothing, so a broken host file never turns a default into
a blocked install. Re-runs converge: a second apply plans nothing.

**That projection is a permission-mode change, so it announces itself.** Every
plan or apply that includes the full baseline emits one notice: permission
prompts are off for this project, the baseline tools are denied, each baseline
key the project already sets to its own value is named, each baseline deny name
the project's allow list already carried is named, and the two opt-outs are
spelled out — `--no-claude-settings` for one run, or `.bee/config.json`
`"claude_settings": false` for good. The baseline is written whether or not
bee's guard hooks are wired, so on an install with `--no-hooks` bee's guards are
NOT the safety net behind bypass mode, and the notice says so.

The opt-out is narrower and older than the baseline: `--no-claude-settings`
(passed through by both `install.sh` and `install.ps1`), or the recorded
`"claude_settings": false`, leaves the two worktree-tool allow names as the
whole projection — exactly what onboarding did before the baseline existed. What
the human observes: a host session stopped asking permission for the worktree
tools bee's own worktree-first rule calls, then stopped asking altogether,
without either the user's `~/.claude/settings.json` or a foreign settings entry
being touched. (Feature claude-settings-baseline, CONTEXT.md D1-D7, widening
decision 5fd24d1b.)

**Cost the owner accepted, stated where a reader will meet it.** Denying
`AskUserQuestion` makes gate questions plain chat text in hosts. Denying
`SendMessage` removes a leader's way to steer a Claude worker. Both are how this
repo already runs; both surprise a host that did not ask for them, which is why
the notice names the denied set rather than mentioning it in passing.

**Publish the deploy skill to the skillstation hub (only with `--skillstation`).**
Trigger: an onboard run with `--skillstation` while the skill sync is on and not
blocked. What changes: the plan gains one `publish_skillstation` item per entry of
the constant `HUB_SKILLS`, which holds only `bee-deploying`; the item names the
skill, its rendered `.claude/skills/<name>` path and target `skillstation`. The
apply copies that rendered folder to a staging copy under
`.bee/tmp/skillstation-publish/<name>` and points the CLI at the copy, because
`skillstation skill commit <folder> --yes` (measured on skillstation 0.9.0)
closes and unlinks the folder it is given, which would delete the per-project
copy the same onboard just wrote. It then runs
`skillstation skill commit <copy> --yes --json`, then
`skillstation skill activate <skill> --json` and the confirm command it returns,
and removes the staging copy. bee never writes a hub file itself. What blocks it: nothing — a missing
`skillstation` binary, a non-zero exit or an absent rendered folder is a loud
skip, one `skipped_skills` entry `{skill, target: "skillstation", reason}`, and
the onboard still reports `applied`. What the human observes: the per-project
copy is always written; the hub copy is the opt-in second home. Without the flag
no item and no skip appear (locked in `docs/history/bee-deploying-skill/CONTEXT.md`,
D2 and D3; the staging copy is the execution-time decision recorded against
that feature, which touches D2 and changes nothing above).

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

## A way to prove the project works

A host project that declares no test command has no way to show that it works;
its agents can only report that unit tests passed, which proves that the tests
agree with the code and nothing more. Onboarding therefore names the gap out
loud: when no test command is declared, the run ends by offering to build a
project verification check, in the user's own language, and says plainly that
the choice is theirs to accept or refuse. Onboarding only offers. It never
builds anything, and it holds no record of a refusal — an offer declined
returns on the next run unless the project declares itself deliberately
test-free.

Accepting produces one project verification skill: it studies the project,
drives the real product the way a person does, and leaves a single command
anyone can run. That skill is authored once and belongs to the project, so it
is kept as the project's own source and rendered into every runtime skill home
from there — one act of authoring serves every agent runtime the project uses,
and the rendered copies are derived, never hand-edited.

The render **creates and updates; it never removes.** This is the boundary that
makes the feature safe to ship into someone else's repository: a verification
skill a team wrote by hand, which onboarding did not generate, is not merely
left alone — no step of the run ever names it. Keeping a rendered copy current
when its source changes is the render's job; deciding that a stale one should
go is a person's, done in a working tree where the deletion is visible and
reversible.

Wiring the check into the project's declared test command is a **separate
agreement**, asked after the check has been built and proven and carrying the
measured cost that proof run actually took. A project that already declares a
test command keeps it: the new check is appended, never substituted. The two
questions stay apart because a person who wanted proof their product works has
not thereby agreed that every future commit should become slower.

## Pointers (implementation)

- `.claude/settings.json` Claude settings baseline projection: the
  `CLAUDE_SETTINGS_BASELINE` constant (with `WORKTREE_TOOL_ALLOW` as its allow
  half), `claude_settings_missing(repo_root, full)` and
  `merge_claude_settings(path, full)` in
  `packages/bee-rs/crates/bee/src/onboard/hooks_wiring.rs`; the notice text in
  `claude_settings_notices` (`onboard/notices.rs`), fed by
  `claude_settings_kept` so an apply reports what the project had before the
  write. `full = false` is the opt-out and owns only the two allow names.
  Planned at the `5b-statusline` stage of `onboard/plan.rs` as one
  `merge_claude_settings` item and applied by that arm of `onboard/apply.rs`,
  which re-checks `claude_settings_missing` at apply time and takes one
  `backup_once` before the write. `full` is the `--no-claude-settings` flag AND
  the project's `"claude_settings": false` preference (`onboard/mod.rs`), never
  written by onboarding; passed through by `scripts/install.sh` and
  `scripts/install.ps1`. Tests: `merge_claude_settings_without_the_baseline_*`,
  `a_fresh_file_gets_the_whole_baseline`, `the_odd_shapes_and_a_non_array_deny_are_left_unchanged`
  and `the_kept_report_names_a_present_mode_and_an_allowed_then_denied_tool` in
  `onboard/hooks_wiring.rs`; `a_virgin_settings_file_gains_the_whole_baseline`,
  `no_claude_settings_on_a_fresh_repo_writes_only_the_two_allow_names`,
  `an_already_onboarded_host_gains_the_baseline_and_keeps_its_hooks_and_statusline`
  and `the_baseline_notice_names_a_kept_default_mode_and_an_allowed_then_denied_tool`
  in `onboard/tests.rs`. Provenance: cells `oawt-1` (decision 5fd24d1b), `csb-1`,
  and `csb-2` (feature claude-settings-baseline).

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
  full external-command executor examples; scripts/tests/test_config_samples_safe.mjs (retired with the Node runtime, 2026-08-02)
  keeps both inert (never diffs against the live `.bee/config.json`).

- Verification skill pair and the copy-only render: skills `bee-verifying` and
  `bee-verify-upkeep`; project source `.bee/verify/<name>/`, rendered to the
  three targets in `REPO_SKILL_TARGETS`. Planner `compute_verify_skill_items`
  and `verify_root_refusal` (`onboard/plan.rs`), applier
  `apply_copy_verify_skill` (`onboard/apply.rs`) — neither reaches
  `apply_remove_skill`, and `onboard/skills.rs` is unmodified by the feature.
  Offer constant `NO_TEST_VERIFICATION_OFFER` and the shared `declares_test`
  predicate (`onboard/notices.rs`). Key test:
  `a_host_authored_verify_skill_is_byte_identical_after_apply` — the only test
  that goes red when a prune arm is injected. Provenance: cells `vsh-1`..`vsh-4`,
  commits 55b9c9ea, 4721ad4c, 25370f52, 228be6ff, eb228cf0.

- Engine-free verify render: `bee onboard --verify-only [--apply]` serves a host
  with no bee checkout, because the verify path reads only `<repo>/.bee/verify`
  and `REPO_SKILL_TARGETS`. It branches in `try_native` before `Engine::locate`
  (`run_verify_only`, `onboard/mod.rs`) and reuses `apply_copy_verify_skill`
  through `apply_verify_only` (`onboard/apply.rs`), which rewrites only
  `managed.verify_skills` and leaves every other `.bee/onboarding.json` key,
  `updated_at` included, as found. Absent `.bee/onboarding.json` refuses with
  kind `not_onboarded`. Rejected: running the full apply with a stub engine,
  which would write bee-* skills, the AGENTS block and `.bee/bin` from nothing.
  The `engine_not_found` refusal names this command when `.bee/verify/` holds a
  skill directory. The registry flag entry is hand-edited in
  `generated/registry_payload.json` (no generator exists) and the distinct-flag
  pin in `catalog.rs` moved to 246 for `verify-only`. Key test:
  `verify_only_apply_renders_every_runtime_home_with_no_engine_visible`.
  Provenance: cell `vrs-1`.

- Skillstation hub publish: constant `HUB_SKILLS` and the `publish_skillstation`
  plan item in `packages/bee-rs/crates/bee/src/onboard/plan.rs`; the
  `publish_skillstation` arm of `onboard/apply.rs` runs the commit, activate and
  confirm calls and pushes the loud `skipped_skills` entry. Flag parse in
  `onboard/mod.rs`. Provenance: feature `bee-deploying-skill`, cell `dep-2`.

## Static checks are offered as members of `commands.test`

Read when: onboarding names a project's lint or type check, or a list-valued `commands.test` draws (or stops drawing) the "No standard commands recorded" notice.

`detect_static_checks` (`onboard/notices.rs`) reads the project's own root files, runs no tool, and returns at most one lint and one type-check candidate, a manifest script beating a convention. `commands_notices` only adds text and never writes: bee proposes, the owner confirms, and the answer is recorded as a decision (trust-layers D1/D5). Rejected: a new `commands` key, which would bring back the split `commands.verify` was retired for. A check counts as already present when any `commands.test` entry contains its marker word, because a whole-command match offered checks the project already ran. A non-empty list now counts as a recorded test command, so an accepted offer does not repeat on every onboard. The fixed notice text lives in `onboard/templates.rs` (`STATIC_CHECK_*`).

## A deploy written by bee-deploying drives the product before it goes live

Read when: a host's deploy script stops at the product drive, `--skip-product-drive` or exit code 2 appears in a deploy, or onboarding prints the "deploy goes live without driving the product" notice.

Both templates carry one `SAFE` step before the first `LIVE` step: `bash` on the one `.bee/verify/verify-app/control-*` file with `drive --full`, apart from the `## Verify` section that checks the live target afterwards (deploy-verify-gate D1-D7). A missing control script stops the deploy and names bee-verifying; `--skip-product-drive` lets the run go on, prints that the product was not driven, and is the user's choice for that run, never the agent's, like `--yes` (D2). The flag covers only a missing verify skill and never overrides a red drive. The template reads the control script's exit code: 0 passes, 1 is a red drive and stops the deploy, 2 means the entry point is unknown and stops it with a message pointing to bee-verify-upkeep (D7). Replay in bee-deploying checks that both files carry the step and adds it when either lacks it, reading the control script for the entry point instead of running a full drive (D3, D7). A deploy owned by a project release script or deploy skill is not edited; the user is told and asked (D7).

Onboarding tells a project that already has `docs/DEPLOY.md` or `scripts/deploy.sh` when any existing one of them lacks the text `.bee/verify/verify-app/`: `deploy_drives_no_product` in `onboard/notices.rs` pushes the constant `DEPLOY_DRIVES_NO_PRODUCT_NOTICE` (`onboard/templates.rs`) beside the verification offer, in every case of that arm (D5, D7). It is propose-only and silent with neither file. The bee repo gets no special case: it has neither file, and its release script owns its smoke (D6). Tests carry `deploy_verify` in their names.
