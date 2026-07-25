---
type: bee.area
title: "Hook Runtime — governed paths, the always-writable set, and the intake gate"
description: "Which write targets escape the active feature's gate routing and which never do, why the always-writable set only ever shrinks, and why a finished feature's leftover approvals are not what decides whether the next source write is allowed."
timestamp: 2026-07-22
bee:
  id: hook-runtime-governed-paths-and-the-intake-gate
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: [8ed35504 (write-guard always-writable set shrinks), c2c46488 (the intake gate fires in every terminal state; approvals never outlive the feature that earned them)]
  sources: ["bee-footprint D2 (cell footprint-2, 2026-07-12)", "docs/specs/hook-runtime.md#B11", "docs/specs/hook-runtime.md#B12", "docs/specs/hook-runtime.md#R11", "docs/specs/hook-runtime.md#R12", "docs/specs/hook-runtime.md#P8"]
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

## Pointers (implementation)

- Always-writable set: `GATE_ALLOWED_PREFIXES` in
  `packages/bee/lib/guards.mjs` (`.bee/`, `docs/`, `plans/`,
  `AGENTS.md`; repo-root `.spikes/` removed per bee-footprint D2 — the
  workflow's own `.bee/spikes/` subfolder is already covered by `.bee/`);
  session-close nudge mirrors it as `NUDGE_ALLOWED` in
  `packages/bee/hooks/bee-session-close.mjs`.

## Open Gaps

- Recorded tradeoff (bee-footprint P3): the workflow's disposable-experiment
  subfolder is both always-writable and excluded from version control, so its
  contents never appear in a change listing. This is deliberate, not a defect
  — but a reviewer must not read a clean change listing as proof that nothing
  was staged in that location; confirming its contents requires looking at
  the location itself.
