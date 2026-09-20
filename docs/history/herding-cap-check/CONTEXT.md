# Herding Cap Check — Context

**Feature slug:** herding-cap-check
**Date:** 2026-09-20
**Shaping session:** complete
**Scope:** Quick
**Domain types:** RUN

## Feature Boundary

`bee herding run` stops reporting a clean run when its worker claimed success
but never capped the cell. On that one condition it exits non-zero and names
the cell and the verb that settles it. It ends there: it caps nothing, changes
no other outcome class, and leaves every dispatch that carries no cell id
exactly as it is today.

## Why now

Pattern `20260827-a-dead-worker-has-the-code-and-is-missing-the-last-mechanical-step`
is already promoted and already carries `critical: true`, and it keeps
happening. Four independent occurrences are on the record:

- `windows-ci-green` — "the leader capped the cell, not the worker — the herding
  worker reported done with a narrow proof and skipped its finish command"
- `three-findings` — "the worker proved with a filtered subset instead of the
  cell's approved verify, so the leader re-ran the full declared command"
- `pi-worker-surface` cell `pws-2`, 2026-09-20 — the worker returned
  `outcome: "done"` with a complete result payload and the right commit
  trailer; the cell stayed `claimed` with a completely empty trace, and
  `bee close` then reported only 1 capped cell for a two-cell feature
- the owner's own project memory, which already tracks it as PBI `p-6f2623c9`

The capture discipline says an already-promoted pattern that recurs is escalated
to a durable owner — hook, guard, doctor check or test — rather than restated as
prose. Prose has had four chances.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

Decision log: `85e32ca2` (refuse), `a27ce76d` (report only).

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | When a dispatched worker reports SUCCESS but its cell is still claimed with no `capped_at`, `bee herding run` REFUSES: non-zero exit, naming the cell id and the verb that settles it. It does not report the run as clean. | `85e32ca2`. The failure is silent today — exit 0 with a full result payload while the cell record stays empty. That silence is how the pattern slipped past four times. |
| D2 | On that refusal bee REPORTS ONLY. It never caps the cell itself, and does not print a ready-to-paste cap line either. | `a27ce76d`. A cap carries a proof line that the cap door and `bee close` check. bee cannot know the worker's verify ran clean; capping from the worker's own `result-N.json` would promote an unchecked self-report into recorded proof — worse than a missing cap, because it looks verified. |
| D3 | The check fires ONLY when the worker claimed success. A worker that returns blocked, timed out, died, or was refused leaves its cell claimed legitimately, and none of those outcomes is touched. | Otherwise the check would refuse the very outcomes that are already correct, and the rescue ladder depends on a blocked cell staying claimed. |
| D4 | The check fires only for a dispatch that carries a cell id. A gather, advisor, hat seat or reviewer dispatch has no cell and is unaffected. | `opts.cell_id` is already `Option<String>` on the run's options — absent means there is nothing to check. |
| D5 | Both launch paths are covered: the pane path and the no-pane child path. | They differ in transport, not in this obligation; covering one would leave the other silent. |
| D6 | Unknown cell state does not refuse. If the cell file is missing or unparseable, the run reports as it does today. | The guard must not turn an unreadable file into a failed run — the same fail-open-on-data-merely-read rule the write guard already holds. |

### Agent's Discretion

- The exact wording of the refusal, and its exit code value.
- Where in the run's completion path the check sits, provided it sees the
  resolved outcome and the cell id.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Claimed success | The worker's own reported outcome says the work is done. Distinct from a transport outcome, which says only that delivery worked. |
| Capped | The cell record carries `status: capped` and a `capped_at` stamp. A claimed cell with an empty trace is NOT capped. |

## Existing Code Context

From the quick scout only.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/herding/run.rs:180`, `:439` — `cell_id` is
  already carried on the run's options, parsed from the dispatch flags at
  `:370`. Nothing new needs threading through.
- `packages/bee-rs/crates/bee/src/herding/run.rs:2219` — `record_outcome`, which
  already runs at completion with both the resolved outcome and the options in
  hand. The natural seam.
- `packages/bee-rs/crates/bee/src/herding/mailbox.rs:490` — `MailboxResult` and
  its `status`, for reading what the worker actually claimed.

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/cells/` — the cell record this check
  reads. It only reads.
- The herding contract tests, which assert run outcomes and will need a case
  per D1, D3 and D6.

## Canonical References

- `docs/knowledge/patterns/20260827-a-dead-worker-has-the-code-and-is-missing-the-last-mechanical-step.md`
  — the promoted pattern this feature gives a durable owner.

<!-- bee:not-a-deferral: These sections are this file's own record of what was resolved. "Deferred To Planning" is a fixed heading in the CONTEXT template and both of its items are ANSWERED and checked off below, each with a file:line behind it; "Deferred Ideas" is empty; the Handoff Note is template boilerplate naming what a planning agent reads. Nothing here promises later action. -->
## Outstanding Questions

### Resolve Before Planning

None.

### Deferred To Planning

Both were answered by reading the source during planning, before the gate.

- [x] **Do the pane and no-pane paths share one completion seam?** **No.**
  `record_outcome` — which the scout above called "the natural seam" — runs only
  in `execute_new` (`run.rs:2737`) and `execute_continue` (`:3497`), both
  pane-only. The seam that covers both is `fn run` (`:4111-4131`), where the two
  launch paths converge and where the existing dissent transcription already
  matches on the same outcome and reads the same `opts.cell_id`. D5 is satisfied
  there and nowhere else.
- [x] **Which `RunOutcome` variants count as "claimed success"?**
  `RunOutcome::Result(r)` with `r.status == MailboxStatus::Done`, and only that.
  It is the single SUCCESS arm `exit_code_for` already uses (`run.rs:3524`), so
  D3's definition was already written in the file. `blocked` is a well-formed
  completion but not a claimed success, exactly as D3 requires.

## Deferred Ideas

None.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
<!-- /bee:not-a-deferral -->
