---
name: bee-reviewing
description: >-
  Run the multi-agent review gate — severity findings, artifact verification, and user acceptance — over an immutable scope the user explicitly asked to review. Use only when the user requests an independent review: "review this", "review today's work", "review feature A and B", "review the diff from X to Y", "review everything unreviewed before release". A finished cell, slice, or feature is never a trigger by itself, and neither is "merge"/"ship"/"release" alone.
metadata:
  version: '0.2'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: Review sessions and their records live in bee state, driven through the vendored bee binary. The binary is vendored into the repo by onboarding; no Node runtime is involved.
---

# Reviewing — independent inspection

Run only when the user explicitly asks for one. Finished work is never
a trigger by itself, nor "merge"/"ship"/"release" alone — for those,
report coverage (`bee reviews status`), ask one yes/no question, and
let silence stay unreviewed. Gate bypass never creates or approves a
review.

## Scope — the user owns the boundary

Resolve the request to one explicit boundary: named feature(s), a
stated range, everything unreviewed since the last baseline, or a time
window. Ambiguous → exactly one boundary question. Work still in
progress is excluded with a stated reason, never swept in or waited for.

`bee reviews create` freezes the scope — baseline, head, included,
excluded — and checks verification evidence before any reviewer spends
a token; if it refuses, surface its reason instead of dispatching
reviewers to compensate. Show the user the preview before dispatch;
from here the diff is immutable.

## The wave

Spawn the core four in parallel — `code-quality`, `architecture`,
`security`, `test-coverage` — plus any conditional reviewer whose
trigger the diff matches, capped at six. A small scope takes one
correctness reviewer; high-risk content (auth, migration, data loss,
external providers) warrants the full wave. Role cards — Purpose,
Scope, Method — and the dispatch prompt shape:
`references/reviewing-reference.md` ("Reviewer roles"). Each reviewer
gets the cumulative diff, the in-scope features' `CONTEXT.md` and
`plan.md`, and nothing else — never session history. Review-tier
model, inline persona, never another plugin's agent type.

What a finding is, how to calibrate severity, and why every finding is
verified before filing: `.bee/expertise/review.md`. Severity here:

- **P1** — security breach, data loss, breaking change, production
  blocker. Blocks approval.
- **P2** — real performance, architecture, reliability, or test gap.
- **P3** — cleanup, docs, future debt.

Every finding carries an axis label — `standards` (is the code well
made: quality, architecture, security, tests) or `spec` (does it do
what the locked decisions promised). Synthesize only after every
reviewer returns: deduplicate, independent corroboration promotes one
level, disagreement takes the conservative route, uncertain lands at
P2 — these severity rules are unchanged and apply within and across
axes; a P1 blocks regardless of axis. The synthesis report stays ONE
report, grouped by axis, spec-axis group first — axes are never
collapsed back into one undifferentiated ranked list. Record each
finding as it settles: `bee reviews record --kind finding`.

The report ends with ONE required line, the last line in the document:

```
<N> finding(s) — P1 <a>, P2 <b>, P3 <c> · axis: spec <s>, standards <t>.
```

When zero findings survive verification, that last line is verbatim —
`No findings. Scope clean — <N> file(s), <M> capped cell(s) verified.` —
never padded with observations to look like work.

## Dismissed — show what the lead rejected

Directly above that last line, a review that dismissed anything carries a
**Dismissed** section: every suspicion the reviewers raised and this
review dropped, one line each — the suspicion, its `path:line`, and the
reason it went. It exists to be overridable. An invisible filter asks the
human to trust the lead's judgement; a shown one lets them reverse it —
the same honesty rule that keeps the zero-findings line verbatim. Two
independent studies of pstack reached this recommendation on their own
(`docs/history/research/pstack-xia.md:110`).

The reviewers' drops arrive with their reasons already attached
(`.bee/expertise/review.md`, "Verify before reporting"); synthesis adds
its own — a duplicate, a suspicion that did not survive the conservative
route. Line shape: `references/reviewing-reference.md` ("Dismissal
shape"). Dismissals are not findings: no severity, no axis, no
`bee reviews record --kind finding` entry, and the summary line above
stays exactly as it is — they never enter its counts.

A review that dismissed nothing shows NO Dismissed section — not an empty
one, and never one padded to look thorough.

## Verify the artifacts, not the story

- Every capped behavior-change cell: read the verification evidence in
  its trace. Missing or vague ("covered by existing tests", no test
  named) is a P1; the remedy is re-verifying, never a backfill document.
- A judge flag on a cell means the judge may have been moved, not
  passed: diff the flagged files for weakened assertions, skipped
  tests, softened verify commands. A weakened judge is a P1.
- Deliverables: exists + substantive + wired is OK; unwired P2; missing or hollow P1.

## Acceptance — with the human

Walk every SEE/CALL/RUN decision with the user, in the wording of
`references/reviewing-reference.md` ("Human UAT"): fail → P1 fix cell,
rerun the item after the fix caps; a skip needs a recorded reason;
intermittent is a Fail. Record: `bee reviews record --kind uat`.

After a P1 fix caps, re-review the delta AND sweep the whole scope for
that defect class — the same bug hides in siblings. A fix that crossed
a boundary (a public contract, another feature's assumption) gets an
expanded re-review proposed to the user, never silently chosen.

Then the merge question, verbatim — P1 > 0: "P1 findings block merge.
Fix before proceeding?" · P1 = 0: "Review complete. Approve merge?"
Silence is not acknowledgment; the session stays blocked until every
P1's fix and delta re-review pass.

## Finish

Run the project's build/test/lint gates and quote fresh output. P2/P3 go
to the backlog (`bee backlog add`), never as blockers; if filing fails,
write findings to `docs/history/<feature>/reports/residual-findings.md` —
nothing evaporates. Close with `bee reviews record --kind decision`: that
closes the review, not any feature; their already-closed state stays untouched.

## Boundaries

Review reports; it does not repair, decide, or close. Every excluded
concern has one route:

- **Fixing a finding** → a cell in the feature's worktree, claimed after
  the report lands; a reviewer never edits the scope it read.
- **Changing a locked decision** → the user, via `bee decisions log`. A
  spec-axis finding names the gap against `CONTEXT.md` and stops there.
- **Merging** → the human's answer to the merge question, then
  `bee worktree merge`.
- **Closing the feature** → `bee close`. Review closes its own record only.
- **A learning worth keeping** → `bee-capturing`, never the report body.
- **Work still in progress** → the next review, with this one stating the
  exclusion reason.

## Hard rules

- No reviewer before the scope is frozen and previewed; no synthesis
  before every reviewer returns.
- The panel scales to the scope's risk — never reduced by bypass or by
  the originating feature's lane; an unchanged, already-reviewed range
  is never re-dispatched.
- Merge is never self-approved: headless runs report-only, and the merge
  question belongs to the human (AGENTS.md).

## References

| File | When to load |
|---|---|
| `references/reviewing-reference.md` | Reviewer role cards (Purpose/Scope/Method), finding schema, UAT wording |
| `.bee/expertise/review.md` | Finding quality, severity calibration, adversarial reading, verification |
| `.bee/expertise/INDEX.md` | A reviewer lens needs domain grounding — stored data, a caller-facing contract, a trust boundary, a rollout, a speed budget, a surface people use: route from the index, one guide per lens |
