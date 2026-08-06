---
date: 2026-08-06
feature: debt-door-archive
categories: [workflow-state]
severity: high
tags: [guards, enumeration-blindness, parity-suite, capture-debt, silent-failure]
---

# debt-door-archive — a guard computed from an enumeration must tell "nothing owed" apart from "nothing counted"

## What Happened

`bee close` reported `door scribing-debt: clear` for `doc-viewer-links` while
both of that feature's `behavior_change` cells were uncaptured and
`docs/knowledge/` held no trace of what it had shipped. The door was not
wrong about the arithmetic: it counted capped cells in `.bee/cells/`, found
none, and said clear. The cells had been archived — by `bee close` itself, on
the previous green close.

Four counters had the same blind spot: the close and swap walls, the status
payload's per-feature and orphan sweeps, the session-start line, and the
mid-session nudge. All four now read the archive as well, active copy winning
on a duplicate id, and a parity test drives all four over one fixture.

## What Was Learned

**A verdict computed from an enumeration reports on the enumeration.** `count
== 0` answered "how many did I find", and the door printed it as "how much is
owed". The two readings are identical in every passing case and opposite in
the failing one, which is why nothing caught it: a clear door and a paid debt
render the same bytes. Any guard whose answer is a count needs an explicit
answer to *"could the count be zero because the set was unreachable?"* — and
where that is possible, an unreachable set is a refusal, not a pass.

**The retirement step was the trigger, and it was ours.** Nothing external
moved the cells: the close's own auto-archive did, one step after the door it
silently disarmed. A cleanup that runs on the success path can retire the
evidence a later run of that same path depends on. Worth asking of every
sweep, prune, or archive: *what reads this after I move it?*

**The fix belonged at the counter, not at a new door.** The tempting patch was
a second guard on `bee cells archive` refusing to archive over unpaid debt.
That adds a door to defend a door. Making the count honest removes the whole
class instead — archiving can no longer hide debt, so no precondition is
needed there at all.

**Four hand-copied counters get a parity test, not a refactor.** They live in
four layers with different visibility and different callers; merging them
would have been the larger change. Pinning them to one answer over one fixture
costs a single test and fails loudly the moment a fifth copy drifts —
the same choice `derived-check-hardening E5` made for the terminal-phase
memberships.

**Measure the blast radius before widening an alarm.** Counting archived cells
against every feature's best stamp *before* the change yielded 0 features and
0 cells, so the fix could ship without an amnesty stamp and without teaching
anyone to ignore a newly-noisy alarm. Had it been large, the amnesty
precedent (`scribing-integrity`, the pre-ledger backfill) was the recorded
path.

## Evidence

- Cells `dda-1` (commit `fd5f8253`) and `dda-2` (commit `e44e56e9`);
  `list_cells_including_archive` in `verbs/drivers/guard.rs`, parity tests in
  `verbs/status_full/tests.rs`.
- The measured miss: `doc-viewer-links` closed clear with two uncaptured
  cells; see `docs/history/learnings/20260805-doc-viewer-links.md`.
- Behavior captured in `docs/knowledge/areas/workflow-state/gates.md`
  ("Retiring a unit's records never retires its debt").
