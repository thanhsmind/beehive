---
date: 2026-08-18
feature: uat-approval-reaches-the-door
categories: [pattern, process]
severity: high
tags: [gates, workflow-state, worktrees, tests, judging]
---

# Learning: A write and its reader disagreed about which copy of the state was authoritative — on the one gate no bypass may auto-approve

**Category:** pattern
**Severity:** high
**Applicable-when:** a durable record and a projection of it can both be
written by the same command, and a blocking reader consults only one.

## What Happened

An owner approved the `uat` gate. `bee gate` printed success. `bee worktree
merge` refused `WORKTREE_MERGE_UAT_PENDING` anyway, while the lane file plainly
read `uat: true`. It happened three times in one session, and a second session
hit it independently within twenty minutes.

Each time the only exit was `--skip-uat`. So a genuine owner approval was
recorded as a skip — on the one gate that no `gate_bypass` level is permitted to
auto-approve.

## Root Cause

The write and the read landed in different files.

With the feature's workflow record already `closed`, the gate write took its
direct-write branch and put the approval on the lane record. Both doors that
block on the gate — merge-time and close-time, carrying byte-identical copies of
the same resolution — read the live workflow record, and failing that fell back
to the *default* state record filtered to that feature. Neither ever read the
lane record.

The record goes `closed` through ordinary housekeeping: any session may close
every live record whose feature is not its own. Nothing reopens one, so waiting
never helped.

## Resolution

One resolver, in the policy module the previous feature had already created for
this question. Live record first and alone; then, only when none stands, the
lane record or the default record. Both doors call it; both copies deleted.
Plus: the gate now says out loud when it could not reach the durable record,
in the text and as a typed field — the silent success line is what hid this.

## The Deviation, And Why It Is Recorded Rather Than Hidden

The frozen plan specified a strict cascade — lane record, *else* default record.
What shipped is an OR of the two. The cascade is unimplementable as written: the
shared gate default stamps "not approved" onto every merged record, so a lane
record that never mentioned the gate is indistinguishable after reading from one
that explicitly refused, and precedence by presence would read every silent lane
record as a veto.

A frozen plan is superseded by a decision, never edited to match what shipped.
Decision `8ca2378f` carries the amendment, the reason, and the cost it accepts:
a lane-side revocation cannot veto a stale default-record approval. That was
equally true before, so it is not a regression — it is what a working cascade
would have bought, and it is filed rather than smuggled.

The first re-cap put the deviation text in the report's own deviations field.
A judge caught that this is not the field the promote miner and the timeline
builder read. Text in the wrong field is not a record.

## What The Judges Caught That Green Did Not

- **The security question, answered properly.** A judge constructed the negative
  inputs and traced them rather than trusting the suite: the set of inputs the
  new resolver approves is exactly the old set plus a literal true on the lane
  record, written only by the gate command, which still refuses an automatic
  actor for this gate. No input approves without an owner approval naming that
  same feature.
- **A third reader of the same question**, in the staging verb, that the
  de-duplication did not collapse. Fails closed, display only, filed.
- **Five tests that never ran.** See below.

## The Expensive One

Resolving the merge conflict, the five new tests were re-inserted before the
file's last closing brace — which closed the last *function*, not the module.
They landed inside another test's body, compiled, and were never registered. The
suite passed, and the total even rose, because the other side's tests arrived in
the same merge.

They were found by running one by name and reading `0 passed`. Promoted as a
critical pattern:
`docs/knowledge/patterns/20260818-a-green-count-is-not-evidence-that-your-new-test-ran.md`.

## Open

- A lane-side revocation still cannot veto a stale default-record approval.
- The staging verb keeps its own copy of "is this gate approved".
- The upstream cause is untouched: housekeeping scoped to "everything that is
  not mine" closes other sessions' unfinished features. The pattern
  `20260818-...-strands-an-approval-written-to-the-projection.md` states the
  rule that follows from it; no code enforces it yet.
