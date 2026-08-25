---
type: bee.area
title: Onboarding — the opt-in status display
description: "How onboarding detects a project's status-display opt-in, vendors and heals the pair, stays entirely out of projects that never opted in, what the line renders, and the second runtime's machine-level status block."
timestamp: 2026-07-22
bee:
  id: onboarding-status-display-vendoring
  lifecycle: active
  areas: [onboarding]
  required_context: [areas/onboarding/overview.md]
  decisions: [102efe08 (opt-in statusline vendor shape), "c6ee6b6e (Gate 4 onboard-statusline: anchored detection, sweep opt-in)", b7af1bf9 (full compatible Codex lifecycle-hook parity)]
  sources: ["cell onboard-statusline-1 (verification_evidence, 2026-07-11)", docs/history/onboard-statusline/reports/review-correctness.md, "codex-hook-state-parity cells 2, 3, 5 (paired Codex lifecycle audit, exclusive plugin-first/repo-copy distribution, and fresh-host handler delivery; capped traces and reports, 2026-07-16)", "docs/specs/onboarding.md#R1", "docs/specs/onboarding.md#R2", "docs/specs/onboarding.md#R3", "docs/specs/onboarding.md#R4", "docs/specs/onboarding.md#E2", "docs/specs/onboarding.md#E3", "docs/specs/onboarding.md#E4", "docs/specs/onboarding.md#E5", "docs/specs/onboarding.md#E7", "docs/specs/onboarding.md#P7", "docs/specs/onboarding.md#P8", "docs/specs/onboarding.md#P9", "docs/specs/onboarding.md#P10", "docs/specs/onboarding.md#P11", "docs/specs/onboarding.md#P12"]
  authoritative_for: "onboarding: the opt-in status display"
---

# Onboarding — The Opt-In Status Display

This concept owns one mechanism end to end: the script that renders the assistant's
per-session status line, which onboarding vendors **only** into projects that already
opted in. The defining property is restraint — onboarding never creates the opt-in,
never edits the settings file that declares it, and a project that never opted in is
untouched by this mechanism's existence.

The vendored script is the whole of what this mechanism copies. It was one half of a
two-file pair until the R6 cutover; the other half, a Node usage aggregator, is now
`bee dev statusline`, a subcommand of the bee binary that the script resolves at
render time. Onboarding vendors that binary through a different mechanism, and a host
that resolves none simply renders the first line without the token/cost line.

## Data Dictionary

| Element | Meaning |
|---|---|
| status-display script | The script that renders the assistant's per-session status line. It draws the session facts itself and asks `bee dev statusline` for the optional token/cost line. The canonical copy lives with bee's source; each opted-in project holds a vendored copy. |
| opt-in signal | The project's assistant-settings file declares a status-display command that points at the **project's own** copy of the display script — either anchored by the project-directory variable or written as a bare project-relative path. A reference to a user-level (home-directory) copy is NOT an opt-in. |
| managed status-display record | A fingerprint per vendored file, stored in the project's onboarding record **only when the project opts in**, so later runs can tell current from drifted. Projects that never opted in carry no such record. |

## Behaviors & Operations

**Detect (every run).** Onboarding reads the project's assistant-settings file and
derives the opt-in signal. What blocks it: nothing — an absent, unreadable,
unparseable, or unexpectedly-shaped settings file simply means "not opted in";
detection never fails a run. What the agent observes: opted-in projects with a
missing or altered vendored file see one planned copy action per affected file;
non-opted projects see zero status-display actions, always.

**Vendor (apply run, opted-in projects only).** Each planned file is written from
the canonical copy, whole-file, atomically. The plan walks the canonical directory,
so the count follows that directory rather than a fixed number. Side effects: none
beyond the files it copies — the settings file is never created, modified, or backed
up by this behavior. Afterwards the project's status display renders with the canonical
behavior, and an immediate re-check reports up to date.

**Heal drift.** A locally edited vendored file is treated as drift, not preference:
the next apply overwrites it with the canonical copy (same contract as every vendored
helper — the canonical source is bee's tree). A project that wants local
status-display behavior keeps its settings pointing at a user-level copy instead.

**Stay out (non-opted projects).** Projects without the opt-in signal never receive
the vendored file, never gain a managed status-display record, and their up-to-date
status is entirely unaffected by this mechanism's existence. Such a project still
shows a status line if the human's user-level settings name a user-level copy — but
that copy is outside onboarding's reach and may be arbitrarily stale. Keeping it
current is a manual copy from the canonical source, and nothing in bee detects or
reports its drift.

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

- **R1** — Onboarding syncs the status-display script only into projects already
  opted in; it never creates the opt-in and never touches the settings file in
  this stage (decision 102efe08).
- **R2** — Detection is fail-safe: any settings shape it does not positively
  recognize means "not opted in"; it never aborts or throws (decision 102efe08).
- **R3** — Only project-level references count as opt-in: the project-directory
  variable must anchor the script path itself, and bare relative references must
  not be preceded by another path segment. A user-level path containing the same
  script name, or the project-directory variable appearing elsewhere in the
  command, is not an opt-in (decision c6ee6b6e, review finding P2-1).
- **R4** — The canonical script and an opted-in project's vendored copy must be
  byte-identical; a one-sided edit anywhere (including deleting the vendored copy
  while still opted in) is drift and fails the standing guard
  (`packages/bee-rs/crates/bee/tests/statusline_contract.rs`) (decision c6ee6b6e,
  review finding P2-3).

## Edge Cases Settled

- Settings file unparseable → not opted in, run proceeds normally.
- Status-display command present but not a text value → not opted in.
- Project-directory variable used elsewhere in the command while the script path
  is user-level → not opted in (the review's adversarial case).
- Exactly one vendored file drifted → exactly that file is re-planned, any other
  untouched.
- Opting out after having been opted in → the stale managed record is inert but
  currently survives; recorded as a known gap (backlog, paired with the
  equivalent behavior in the hook-vendoring mechanism). Now sharper than when it
  was filed: the remembered opt-in is what keeps guardrails current, so it is
  also what a genuine opt-out would have to clear. (The remembered opt-in itself
  is owned by [`repo-local-guardrails.md`](repo-local-guardrails.md).)

## Open Gaps

- Opt-out manifest cleanup (see Edge Cases) — backlog item filed 2026-07-11.
- The vendoring path itself (plan stage 3b and the `copy_statusline` apply case)
  has no live test. The Node sandbox cases that covered it were deleted at the R6
  cutover and never ported; only opt-in detection and canonical/vendored
  byte-equality are guarded today.

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
- `src/onboard/hooks_wiring.rs:1023` — the opt-in detection cases. The vendoring
  plan/apply path has no live test (see Open Gaps).
- Host-side settings contract: `.claude/settings.json` → `statusLine.command`.
- Second runtime, machine-level: `src/onboard/templates.rs:179`
  (`CODEX_STATUS_LINE_BLOCK`), `src/onboard/hooks_wiring.rs:521-546`
  (`codex_user_config_path()`, `codex_statusline_missing()`,
  `codex_statusline_next_text()`), `src/onboard/plan.rs:679-680` (the
  `ensure_codex_statusline` action — `~/.codex/config.toml`, never repoRoot-joined).
