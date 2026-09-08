---
type: bee.area
title: Onboarding — the default status display
description: "How onboarding detects a project's status display, vendors it by default, respects preference and --no-statusline, what the line renders, and the second runtime's machine-level status block."
timestamp: 2026-07-22
bee:
  id: onboarding-status-display-vendoring
  lifecycle: active
  areas: [onboarding]
  required_context: [areas/onboarding/overview.md]
  decisions: ["4cac0774 (default statusline install; supersedes 102efe08)", 102efe08 (superseded: opt-in statusline vendor shape), "c6ee6b6e (Gate 4 onboard-statusline: anchored detection, sweep opt-in)", b7af1bf9 (full compatible Codex lifecycle-hook parity)]
  sources: ["cell onboard-statusline-1 (verification_evidence, 2026-07-11)", docs/history/onboard-statusline/reports/review-correctness.md, "codex-hook-state-parity cells 2, 3, 5 (paired Codex lifecycle audit, exclusive plugin-first/repo-copy distribution, and fresh-host handler delivery; capped traces and reports, 2026-07-16)", "docs/specs/onboarding.md#R1", "docs/specs/onboarding.md#R2", "docs/specs/onboarding.md#R3", "docs/specs/onboarding.md#R4", "docs/specs/onboarding.md#E2", "docs/specs/onboarding.md#E3", "docs/specs/onboarding.md#E4", "docs/specs/onboarding.md#E5", "docs/specs/onboarding.md#E7", "docs/specs/onboarding.md#P7", "docs/specs/onboarding.md#P8", "docs/specs/onboarding.md#P9", "docs/specs/onboarding.md#P10", "docs/specs/onboarding.md#P11", "docs/specs/onboarding.md#P12"]
  authoritative_for: "onboarding: the default status display"
---

# Onboarding — The Default Status Display

This concept owns one mechanism end to end: the script that renders the assistant's
per-session status line, which onboarding vendors by default. The defining property is
restraint — onboarding adds the key when it is absent and never rewrites one that is
present. A project whose settings file already declares a status line is treated as
preference and left byte-for-byte alone.

The vendored script is the whole of what this mechanism copies. It was one half of a
two-file pair until the R6 cutover; the other half, a Node usage aggregator, is now
`bee dev statusline`, a subcommand of the bee binary that the script resolves at
render time. Onboarding vendors that binary through a different mechanism, and a host
that resolves none simply renders the first line without the token/cost line.

## Data Dictionary

| Element | Meaning |
|---|---|
| status-display script | The script that renders the assistant's per-session status line. It draws the session facts itself and asks `bee dev statusline` for the optional token/cost line. The canonical copy lives with bee's source; each managed project holds a vendored copy. |
| absent status entry | The project's assistant-settings file is missing or contains no `statusLine` key. This is the trigger for the default install. |
| project-level status entry | The project's assistant-settings file declares a status-display command that points at the **project's own** copy of the display script — either anchored by the project-directory variable or written as a bare project-relative path. A reference to a user-level (home-directory) copy is NOT a project-level entry. |
| managed status-display record | A fingerprint per vendored file, stored in the project's onboarding record whenever the project-level entry is present, so later runs can tell current from drifted. Projects without a project-level entry carry no such record. |

## Behaviors & Operations

**Detect (every run).** Onboarding reads the project's assistant-settings file.
Detection produces one of three outcomes:
1. Absent key or absent file: plans a settings write to add the canonical entry plus a copy of the script.
2. Project-level entry present: plans a copy or drift heal of the script only (settings file is not touched).
3. Anything else (user-level path, non-object, unparseable file): plans zero status-display actions.
Detection is fail-safe and never fails or blocks a run.

**Vendor (apply run).** Each planned file is written from the canonical copy,
whole-file, atomically. The plan walks the canonical directory, so the count
follows that directory rather than a fixed number. When the settings key was
absent, the settings file is created or gains the canonical `statusLine` key,
with a `.bak` backup taken if the file already existed — and exactly one backup
per run even when the hooks merge writes the same file. Afterwards the project's
status display renders with the canonical behavior, and an immediate re-check
reports up to date.

**Heal drift.** A locally edited vendored file is treated as drift, not preference:
the next apply overwrites it with the canonical copy (same contract as every vendored
helper — the canonical source is bee's tree). A project that wants local
status-display behavior keeps its settings pointing at a user-level copy instead.

**Stay out (--no-statusline, .bee/config.json statusline:false, or a present foreign entry).**
When `--no-statusline` is passed, `.bee/config.json` sets `"statusline": false`, or the
host shell is PowerShell (`host_shell_is_powershell`), onboarding skips the statusline
steps entirely. Similarly, a project that already declares a foreign or user-level
`statusLine` is treated as user preference and left alone: it never receives the
vendored file, never gains a managed status-display record, and its up-to-date status
is unaffected. Keeping a user-level copy current is a manual copy from the canonical
source; nothing in bee detects or reports its drift.

### What the status display renders

One line of session facts, then an optional second line of per-model token and cost
totals. Every segment is omitted when its fact is unavailable, and an unavailable
fact never fails the line (a missing subscription-usage report, a project outside
version control, a model without an effort setting — each simply drops its segment).

| Segment | Fact |
|---|---|
| location | The session's working directory, and its version-control branch when there is one |
| model | The model's display name, plus its reasoning-effort level when that level is not the default |
| context | Percentage of the context window **remaining** — never the percentage used |
| session usage | Percentage of the rolling short-window subscription limit consumed, when the runtime reports one |
| weekly usage | Percentage of the rolling weekly subscription limit consumed, when the runtime reports one |
| cost | Per-model new/cached token totals and their billed cost, aggregated over the session and every subagent transcript. Rendered by `bee dev statusline`; the whole line is omitted when the script resolves no bee binary |

**Context colour is a workflow signal, not a gauge.** The colour of the context
segment answers one question — "does the human need to think about a handoff?" —
so its thresholds track bee's handoff mark (rule: agents-context-handoff-65),
i.e. ~35% of the window remaining, not an even split of the scale:

| Context remaining | Colour | Meaning |
|---|---|---|
| above ~35% | calm (green) | routine work; nothing to do |
| ~20-35% | caution (yellow) | the handoff mark is here; start wrapping up |
| below ~20% | alarm (red) | write the handoff and pause |

The rule this encodes: an alarm colour that is on for most of a working session is
not an alarm. Any future retune keeps the caution band anchored on the handoff mark
for exactly that reason. The subscription-usage segments follow the same principle —
quiet by default, emphasized only once consumption is high enough to matter.

**Guarantee the second runtime's status display (machine-level, add-only).**
Trigger: any run on a machine whose second-runtime user config exists but
carries no status-line key — the second runtime has no per-project status
surface and no custom-script support, only a fixed-segment list in the user
config. What blocks it: the tool being absent (no user config file — onboarding
never creates it) or the key already being present, custom segments included: a
present choice is preference, never drift, and is left untouched byte-for-byte.
What changes: the canonical segment list (working directory, branch, model with
effort, context remaining, both rate-limit windows, tokens used — mirroring the
first runtime's status line) is spliced under the existing TUI section or
appended as a new one, with a backup written first. What the human observes:
after one apply per machine, the second runtime shows the same status story as
the first; a re-run plans nothing.

## Business Rules

- **R1** — Onboarding writes the project-level entry when the key is absent,
  creating the settings file if needed; a present key is preference and is left
  byte-for-byte alone; `--no-statusline` and `.bee/config.json` `"statusline": false`
  suppress both the settings write and the script copy (decision 4cac0774,
  supersedes 102efe08).
- **R2** — Detection is fail-safe: an unrecognized settings shape means leave
  alone; it never aborts, throws, or blocks the run (decision 102efe08).
- **R3** — Only project-level references count as a project-level status entry:
  the project-directory variable must anchor the script path itself, and bare
  relative references must not be preceded by another path segment. A user-level
  path containing the same script name is a present key and therefore preference,
  not a project-level entry (decision c6ee6b6e, review finding P2-1).
- **R4** — The canonical script and a project's vendored copy must be
  byte-identical; a one-sided edit anywhere (including deleting the vendored copy
  while carrying the project-level entry) is drift and fails the standing guard
  (`packages/bee-rs/crates/bee/tests/statusline_contract.rs`) (decision c6ee6b6e,
  review finding P2-3).
- **R4a** — **Canonical is the only edit site of the pair.** R4 says the two
  copies must match; this says which one to change. `onboard --apply` (and
  `bee dev regen` through it) overwrites the vendored copy from canonical, so a
  one-sided fix of the COPY is erased on the next apply, and a one-sided fix of
  CANONICAL is invisible until one runs. Commit 00a8fdf4 is the standing
  example: it repaired `.claude/statusline-command.sh` and left
  `packages/bee/statusline/statusline-command.sh` resolving the binary at
  `<repo>/bee` instead of `<repo>/.bee/bin/bee`, so the usage segment vanished,
  fail-open and silent, on any host re-vendored from canonical
  (statusline-binary-lookup, 2026-09-01).
- **R5** — The command bee writes must satisfy R3's own detector, pinned by a
  test — otherwise a host silently un-adopts on the next run.

**Turning an opt-in into a default promotes its rough edges into the vendor's
bugs.** The status display had shipped for a year with a hard `jq` dependency
that printed a RED error on every prompt when `jq` was missing, and a settings
pointer to a script the host might not have. While the human had *chosen* the
feature, both were their problem. The moment onboarding installs it unasked,
both become bee's. The rule generalises past this feature: before flipping any
opt-in to opt-out, audit every failure mode the feature can show a user who
never asked for it, and make each one silent. The two supporting rules settled
in the same pass are stated above — one backup per run rather than per writer,
and an unparseable host config read as "leave alone" rather than "absent",
because a refusal turns one broken file into a blocked install.

## Edge Cases Settled

- Settings file unparseable → not absent, treated as broken, zero status-display actions, run proceeds normally.
- Status-display command present but not a text value → not absent, treated as foreign shape, zero status-display actions.
- Project-directory variable used elsewhere in the command while the script path
  is user-level → treated as preference (foreign value), zero status-display actions.
- Exactly one vendored file drifted → exactly that file is re-planned, any other
  untouched.
- Opting out after having been opted in → this is now the ordinary path, and it
  splits: a hand-deleted key in `settings.json` is re-added on the next run
  (the durable opt-out is `.bee/config.json` `"statusline": false` or a foreign
  value), while `--no-statusline` on a host that already carries the record leaves
  a stale managed record and an orphaned vendored script (recorded as a known gap;
  paired with the equivalent behavior in the hook-vendoring mechanism).

## Open Gaps

- Opt-out manifest cleanup (see Edge Cases) — backlog item filed 2026-07-11.
  Includes the sub-case where `--no-statusline` leaves a stale managed record
  and an orphaned vendored script on an existing host.
- `LEDGER_GROUPS` mismatch: `plan.rs:239` maps `statusline` to
  `.bee/bin/statusline` while the vendor target is `.claude/` (`plan.rs:788`),
  so the regen obligation (`verbs/cells/obligation.rs:112`) guards a directory
  that does not exist.

## Pointers (implementation)

All Rust paths below are relative to `packages/bee-rs/crates/bee/`.

- `src/onboard/hooks_wiring.rs:584-598` — `statusline_opt_in()`, the anchored and
  bare-relative detection of R3.
- `src/onboard/source.rs:56` — `templates_statusline_dir`, the canonical source
  directory.
- `src/onboard/plan.rs:537-546` — plan stage 3b, the `copy_statusline` items;
  `list_template_statusline()` (l. 81) walks the canonical directory, and
  `build_managed_versions()` / `subset_managed()` (l. 236, l. 298) carry the
  conditional `statusline` key.
- `src/onboard/apply.rs:372-378` — the `copy_statusline` apply case.
- `packages/bee/statusline/statusline-command.sh` — the canonical script, one file.
- `src/devtools/statusline.rs` — `bee dev statusline`, the token/cost line the
  script appends when it resolves a bee binary.
- `tests/statusline_contract.rs` — the standing guard: canonical/vendored
  byte-equality plus the script's binary lookup.
- `src/onboard/hooks_wiring.rs:1023` — statusline detection test cases;
  `src/onboard/tests.rs` covers the full vendoring plan, apply, and convergence lifecycle.
- Host-side settings contract: `.claude/settings.json` → `statusLine.command`.
- Second runtime, machine-level: `src/onboard/templates.rs:179`
  (`CODEX_STATUS_LINE_BLOCK`), `src/onboard/hooks_wiring.rs:521-546`
  (`codex_user_config_path()`, `codex_statusline_missing()`,
  `codex_statusline_next_text()`), `src/onboard/plan.rs:679-680` (the
  `ensure_codex_statusline` action — `~/.codex/config.toml`, never repoRoot-joined).
