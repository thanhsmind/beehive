---
type: bee.area
title: Onboarding — the managed ignore section
description: "The delimited block onboarding owns inside a project's version-control ignore list: what it silences, what always stays version-tracked, the three exhaustive create/append/rewrite cases, and the already-tracked-path warning onboarding never acts on itself."
timestamp: 2026-07-22
bee:
  id: onboarding-managed-ignore-section
  lifecycle: active
  areas: [onboarding]
  required_context: [areas/onboarding/overview.md]
  decisions: [26203bd3 (managed ignore-list section; machine-local vs team-durable split)]
  sources: ["bee-footprint D1 (managed ignore section, cells footprint-1/footprint-4, 2026-07-12)", "docs/specs/onboarding.md#R9", "docs/specs/onboarding.md#R10", "docs/specs/onboarding.md#R11", "docs/specs/onboarding.md#E10", "docs/specs/onboarding.md#E11", "docs/specs/onboarding.md#E12", "docs/specs/onboarding.md#E13", "docs/specs/onboarding.md#E14", "docs/specs/onboarding.md#P15"]
  authoritative_for: "onboarding: the managed ignore section"
---

# Onboarding — The Managed Ignore Section

Onboarding owns exactly one delimited block inside the host project's version-control
ignore list, and nothing else in that file. Two boundaries define this concept: the
**byte** boundary (everything outside the section's own markers belongs to the
project and is preserved exactly) and the **content** boundary (only machine-local
runtime records are silenced; team-durable knowledge never is). A third property
follows from both: onboarding reports what a project must do to its own index, and
never does it.

## Data Dictionary

| Element | Meaning |
|---|---|
| managed ignore section | A clearly-marked, start/end-delimited block that onboarding owns inside the project's version-control ignore list. Every byte outside the delimiters belongs to the project and is never touched. |
| machine-local runtime record | Content the managed ignore section silences: workflow state, reservations, worker scratch, logs, capture queue, feedback snapshot, injection cache, the pause/handoff record, and disposable experiment files. |
| team-durable knowledge | Content that always stays version-tracked, never silenced by the managed ignore section: vendored tooling, configuration, the decision log, the friction log, and work-cell records. |
| ignore-section fingerprint | A hash of the managed ignore section's expected content, stored in the project's onboarding record so a later run can detect drift in that section specifically. |

## Behaviors & Operations

**Manage the ignore section (every apply run).** Trigger: an apply run against
any host project, opted in or not — the managed ignore section is unconditional.
What blocks it: nothing; the three cases below are exhaustive and one always
applies. What changes: exactly one of —
- no ignore list exists yet → the list is created holding only the managed
  section;
- an ignore list exists without the section → the section is appended,
  inserting a guaranteed separating line break first even when the existing
  file's last line has none, so the section never fuses onto the file's last
  line;
- the section is already present but its content has drifted from the
  tracked fingerprint → only the bytes between the section's own markers are
  rewritten; every byte of the project's own content outside those markers is
  preserved exactly, unchanged.

A line that merely resembles the section's marker text inside a longer user
comment is never mistaken for the real marker, so it never triggers case two
or three by accident. Comparing the section's current content against the
fingerprint tolerates Windows-style line endings, so a line-ending-only
difference is never reported as drift and never causes a rewrite on every run.
Side effects: none beyond the ignore list file itself. What the agent
observes: the check run reports which of the three cases applies (or "current"
when the fingerprint matches) before anything is written; the apply run then
performs exactly that action and the ignore-section fingerprint in the
project's onboarding record is refreshed. What the human observes: an
up-to-date project's ignore list carries the section unchanged; a project with
drifted or missing section sees the report name the exact action before
approving it.

**Warn on already-tracked silenced paths (every run).** Trigger: any run,
check or apply, where one or more of the paths the managed ignore section is
meant to silence are already tracked in the host project's version-control
index — ignore-list entries are inert for paths already tracked, so the
managed section alone cannot silence them. What blocks it: nothing; this is
report-only. What changes: nothing to the host's version-control index —
onboarding never runs the untracking operation itself, in this or any other
behavior. Side effects: none. What the agent and human observe: the report
carries one warning naming the count of already-tracked paths and the exact
one-time command the operator must run to untrack them; the warning
disappears once no silenced path remains tracked.

## Business Rules

- **R9** — The managed ignore section covers only machine-local runtime
  records (workflow state, reservations, worker scratch, logs, capture queue,
  feedback snapshot, injection cache, the pause/handoff record, disposable
  experiment files); team-durable knowledge (vendored tooling, configuration,
  the decision log, the friction log, work-cell records) always stays
  version-tracked and is never added to the section (decision 26203bd3).
- **R10** — The managed ignore section is created, appended with a guaranteed
  separator, or content-rewritten depending on the ignore list's current
  state; every byte outside the section's own markers is preserved exactly,
  and a line only resembling the marker text is never treated as the marker
  (decision 26203bd3).
- **R11** — Onboarding never modifies the host project's version-control
  index; when a path the managed section is meant to silence is already
  tracked, the report warns and names the exact one-time untrack command for
  the operator to run instead (decision 26203bd3).

## Edge Cases Settled

- No ignore list present at all → one is created holding only the managed
  ignore section (feature bee-footprint, decision 26203bd3).
- Ignore list present but missing a trailing line break → the managed
  ignore section is still appended cleanly, on its own line, never fused onto
  the file's last line (feature bee-footprint, decision 26203bd3).
- A user comment line that merely resembles the managed section's marker text
  → never mistaken for the real marker, so the section is not duplicated or
  corrupted around it (feature bee-footprint, decision 26203bd3).
- Managed ignore section present with Windows-style line endings → not
  reported as drift and not rewritten; only real content differences trigger
  a rewrite (feature bee-footprint, decision 26203bd3).
- A path the managed section is meant to silence is already tracked in the
  host's version-control index → the report warns and names the exact
  one-time untrack command; onboarding never runs it itself (feature
  bee-footprint, decision 26203bd3).

## Pointers (implementation)

- `packages/bee/scripts/onboard_bee.mjs` — `GITIGNORE_MARKER_START`/`_END`,
  `GITIGNORE_START_RE`/`_END_RE` (marker-resemblance guard),
  `gitignoreBlockPresent`, `normalizeGitignoreForCompare` (CRLF tolerance),
  `create_gitignore_block`/`append_gitignore_block`/`update_gitignore_block`
  plan actions, `trackedGitignorePaths`/`trackedPathsNotices` (already-tracked
  advisory, never runs `git rm` itself), `gitignore_block` fingerprint field.
