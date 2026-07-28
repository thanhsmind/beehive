---
type: bee.area
title: "Hook Runtime — governed paths, the always-writable set, and the intake gate"
description: "Which write targets escape the active feature's gate routing and which never do, why the always-writable set only ever shrinks, why a finished feature's leftover approvals are not what decides whether the next source write is allowed, how many phases require that approval today, why a phase value the workflow does not recognize is now refused instead of silently allowed, and how a value left by a retired phase is translated rather than left to trip that refusal."
timestamp: 2026-07-28
bee:
  id: hook-runtime-governed-paths-and-the-intake-gate
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: [8ed35504 (write-guard always-writable set shrinks), c2c46488 (the intake gate fires in every terminal state; approvals never outlive the feature that earned them), "validation-diet D3/D13 (docs/history/validation-diet/CONTEXT.md, 2026-07-28)"]
  sources: ["bee-footprint D2 (cell footprint-2, 2026-07-12)", "docs/specs/hook-runtime.md#B11", "docs/specs/hook-runtime.md#B12", "docs/specs/hook-runtime.md#R11", "docs/specs/hook-runtime.md#R12", "docs/specs/hook-runtime.md#P8", "validation-diet cells vd-1/vd-2 (traces in .bee/cells/, reports docs/history/validation-diet/reports/vd-1.md,vd-2.md, 2026-07-28 — the gated phase set narrowed to two, the write guard's unrecognized-phase fall-through flipped from silently allowing to refusing, and a saved value left by the retired phase translated on read)"]
  authoritative_for: "hook-runtime: which write targets are governed and which are always writable"
---

# Hook Runtime — governed paths, the always-writable set, and the intake gate

The previous concept covers what the guard can read; this one covers what it
governs. Two questions decide every write: is the target inside the small,
deliberately shrinking set of locations that need no gate routing at all, and is
there active work whose gates could authorise it in the first place. The second
question is answered from the workflow's state, never from approvals a closed
feature left behind.

## Data Dictionary

| Element | Meaning |
|---|---|
| always-writable location | A small named set of locations a write may target without the active feature's gate routing, because the content is machine-local and disposable — today: the workflow's own state/log directory and, inside it, a dedicated subfolder for disposable experiment work. Removing a location from this set only tightens governance; adding one is a deliberate, reviewed decision. |
| gated phase | A phase in which a source write outside the always-writable set is refused until the workflow’s approval is granted; today exactly two of the phases that precede that approval carry this requirement. |

## Behaviors & Operations

**B11 — A repo-root disposable-experiment location is no longer
always-writable.** Trigger: a write targeting the former repo-root
disposable-experiment location. What blocks it: the same gate routing that
governs any other source path — the active feature's phase and gate state —
exactly as for a path outside the always-writable set; nothing exempts this
location anymore. What changes: this location moves from always-writable to
governed, strictly shrinking the always-writable set by one entry; disposable
experiment work itself continues unblocked, but now inside the workflow's own
always-writable directory, under a dedicated subfolder that location's
existing allowance already covers. Side effects: the close-time nudge's own
always-writable set shrinks identically, so a write left in the old location
is flagged there too, not only by the write guard. What actors observe: the
assistant sees the same corrective deny/allow outcome it would see writing to
any other governed source path; the human owner sees no new prompt — the
location simply stopped being an exception (bee-footprint D2).

**B12 — No active work means no source writes — a finished feature is not an
open door.** Trigger: any write to a governed path while the workflow sits in a
terminal state — either *nothing has started yet* or *the last feature has
closed* (workflow-state: the two terminal states, and the only two from which a
new feature may start). What blocks it: the intake gate, which denies the write
and names the terminal state it fired on, telling the assistant to route the
request through the workflow first. What is still allowed: the always-writable
set plus the knowledge locations (docs, plans, the workflow's own directory) —
the closing steps of a feature, spec sync and learning capture, must keep
writing after that feature closes. Why the state and not the gates decide this:
a closed feature leaves its approvals **behind it**, still recorded as
approved. Reading approvals alone, a finished feature is indistinguishable from
an approved one, so the guard reads the state — the phase — and the moment work
is no longer active the door is shut regardless of what the last feature was
allowed to do. Escape hatch: unchanged — a repository may disable the intake
gate entirely in its configuration, and doing so disables it for both terminal
states alike, never one but not the other (decision c2c46488).

**B13 — The gated set of phases lost the one entry a retired feasibility phase
used to hold (validation-diet D3, 2026-07-28).** Trigger: a source write while
the workflow sits in one of the phases that require approval before a write
outside the always-writable set proceeds. What blocks it: exactly the
existing behavior, unchanged in mechanism — an unapproved write is refused.
What changed: the gated set itself is one entry smaller, because the phase
that used to sit between the two that remain no longer exists at all — its
own reality check moved earlier, into the shape-drafting step of the phase
right before it. Nothing about which paths are governed, what unblocks
them, or how the always-writable set behaves changes; only the count of
phases carrying the approval requirement drops from three to two.

**B14 — A phase value matching none of the workflow’s real phases is now
refused outright, not silently passed through (validation-diet D13,
2026-07-28).** Trigger: a source write checked against a phase value that
matches none of the workflow’s real phases — a broken or stale record, never
a phase the workflow can actually be in. What changed: previously such a
value fell through every branch of the write check and reached an implicit
allow, so a broken record gated nothing at all, silently. Now the same
situation is refused outright, naming the unrecognized value. What stays
unaffected: every phase the workflow can actually reach keeps its own
behavior exactly as before — the two gated phases (B13), the states a new
feature may start from, the mid-execution state, and the four phases after
approval that were never given a dedicated branch of their own (independent
review, spec-sync, learning-capture, and housekeeping) all still write
freely once reached; a repository with no saved state, or one the workflow
cannot read, resolves to the very first phase, which has always had its own
branch and is untouched by this change. What actors observe: an assistant
working against a genuinely broken state record now sees a clear refusal
instead of writes silently going ungated.

**B15 — A saved phase value left over from the retired feasibility phase is
translated before the new refusal ever sees it (validation-diet D13,
2026-07-28).** Trigger: a repository whose saved state still names the
phase that used to sit between the two gated phases, written before that
phase was retired. What happens: reading the state translates that value,
transparently, to the phase that absorbed its role — before either the
approval check or B14’s refusal evaluates it, so it is treated as the
perfectly ordinary recognized phase it now is. What this avoids: without
the translation, such a repository would either be permanently unable to
leave the retired phase (only an explicit phase change moves it, and an
unrecognized value refuses that too) or would fall straight into B14’s new
refusal and find every write blocked. What actors observe: an existing
repository resumes exactly where it left off, still gated the same as it
always was, never bricked and never silently ungated.

## Business Rules

- R11 — The write guard's always-writable set no longer includes the
  repo-root disposable-experiment location; that work now lives inside the
  workflow's own already-writable directory, under a dedicated subfolder. The
  set of ungoverned writable locations only shrinks from this change, never
  grows; the session-close nudge's allowed-path set shrinks identically
  (bee-footprint D2).

- R12 — The intake gate fires in **every** terminal state, not merely the
  never-started one: a source write is governed whenever no feature is active,
  including immediately after a feature closes with its approvals still on
  record. Approvals belong to the feature that earned them and never outlive
  it; the active state, not the recorded approvals, decides whether the door is
  open (decision c2c46488).

- R13 — Every idle-gate/write-policy config read inside the guard follows the
  **resolved controlRoot**, never the raw `root` parameter: on a
  companion-mounted path the two name different projects' `.bee/config.json`,
  and reading `root`'s made the idle gate judge a different project's phase
  than the containment check had just resolved in the same call (GH #83, live
  incident 2026-07-27). All three call sites — the terminal-phase idle-gate
  branch, the write-policy-mode read, and the bash-command guard's own idle
  read — were aligned in one pass; half-fixing recreates the bug on the
  remaining path (gh-fix-batch cell gfb-2, 2026-07-28).

- R14 — The gated set — phases requiring approval before a source write
  outside the always-writable set — now holds exactly two entries; the
  retired feasibility phase’s membership disappeared with the phase itself
  (validation-diet D3).

- R15 — A phase value the workflow does not recognize as any of its real
  phases is refused at the write check; every phase the workflow can
  actually produce — gated or not — is unaffected, including the four
  phases after approval with no dedicated branch and the very first phase a
  repository with no or unreadable state resolves to (validation-diet D13).

- R16 — A saved phase value naming the retired feasibility phase is read as
  the phase that absorbed its gate, so an existing repository is neither
  locked out of progressing nor left with its writes silently ungated
  (validation-diet D13).

## Pointers (implementation)

- Always-writable set: `GATE_ALLOWED_PREFIXES` in
  `packages/bee/lib/guards.mjs` (`.bee/`, `docs/`, `plans/`,
  `AGENTS.md`; repo-root `.spikes/` removed per bee-footprint D2 — the
  workflow's own `.bee/spikes/` subfolder is already covered by `.bee/`);
  session-close nudge mirrors it as `NUDGE_ALLOWED` in
  `packages/bee/hooks/bee-session-close.mjs`.
- Gated set, unrecognized-phase refusal, and legacy-phase translation:
  `GATED_PHASES` and the `checkWrite` phase dispatch's final branch in
  `packages/bee/lib/guards.mjs`; the translation itself
  (`LEGACY_PHASE_COERCIONS`) lives in `packages/bee/lib/state.mjs`, applied at
  read time so every consumer of the state/lane record sees it automatically.

## Open Gaps

- Recorded tradeoff (bee-footprint P3): the workflow's disposable-experiment
  subfolder is both always-writable and excluded from version control, so its
  contents never appear in a change listing. This is deliberate, not a defect
  — but a reviewer must not read a clean change listing as proof that nothing
  was staged in that location; confirming its contents requires looking at
  the location itself.
