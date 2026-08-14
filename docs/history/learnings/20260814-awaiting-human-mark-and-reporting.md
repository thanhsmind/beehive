---
date: 2026-08-14
feature: awaiting-human
categories: [failure, pattern]
severity: critical
tags: [planning, cells, verification, delegation, census, judge-review]
---

# Learning: A capability named only inside another cell's prose has no cell that owns it

**Category:** failure
**Severity:** critical
**Tags:** [planning, cells, must-have-tracking]
**Applicable-when:** authoring a cell whose action text mentions a second
capability ("...and add the setter") that the plan intends a later cell to
build.

**Already logged as decision `2668665b`** (2026-08-14T14:08:03.945Z) — this
entry only points to it, per the harvest discipline against duplicate
records; read the decision for the full rule.

## What Happened

> "awaiting-human ah-1 action text said add the setter, but no must_have or
> artifact tracked it, so nothing at cap time or judge time asked whether it
> existed. The gap surfaced only when ah-2 tried to build the clear half and
> found nothing to pair with, and it had to be recovered after the fact as
> ah-4. The mark was fully built and fully unusable in the meantime."
> — decision `2668665b`, rationale

`ah-1` shipped the `waiting_on` mark and its setters fully built; nothing
outside the crate could call them, because `bee state waiting-on set/clear`
did not exist yet and no cell tracked that it needed to. The gap was invisible
at both cap time and judge time because it lived only in `ah-1`'s prose, never
in a `must_have` or an `artifact`.

## Root Cause

A cell's action text is not tracked by the cap/judge machinery the way a
`must_have` or `artifact` is — prose is read by a human or a worker, once,
and then forgotten by everything downstream.

## Recommendation

If a cell's action text names a capability, that capability needs its own
`must_have` or `artifact` in the SAME cell, or its own cell — never prose
alone. See decision `2668665b` for the durable rule.

---

# Learning: A worker's self-reported test count is narration, not evidence

**Category:** failure
**Severity:** critical
**Tags:** [verification, delegation, test-count]
**Applicable-when:** a worker or reworking agent quotes a passed-test count
in a cap report, a rework report, or a judge verdict, instead of citing the
stored full-suite record or re-running the declared test command.

**Already logged as decision `38f031b4`** (2026-08-14T14:08:04.004Z) — this
entry only points to it and adds the concrete evidence trail this feature
produced, per the harvest discipline against duplicate records.

## What Happened

Three separate quotes, on the same tree, each a partial count presented in a
shape indistinguishable from a full-suite result:

> "The cell report's '1642 passed' was a partial count (the src/main.rs
> unittest binary alone is 1647 at HEAD, and 15 integration suites add 121
> more); zero tests were lost..." — judge evidence, `.bee/cells/ah-3.json`

> "The rework report's own '1649 passed (unit)' at ah-3-rework.md:71 is again
> a single-suite count, not the run total — the number to quote is 1772."
> — judge evidence, `.bee/cells/ah-3.json`

The judge's own re-run, read-only, against the real declared command
(`cargo test --release --manifest-path packages/bee-rs/Cargo.toml`) totaled
all 16 `test result:` lines each time: 1768 passed before `ah-2`/`ah-3`'s
rework tests landed, 1772 after (`.bee/cells/ah-2.json:172`,
`.bee/cells/ah-3.json:176`). A fourth quote from a different feature on the
same day, cited in decision `38f031b4`'s own rationale, was `traceable-runs`
trun-2's 1569 against a real 1727. None of the four partial counts hid an
actual failure — but a smaller number reads as lost tests and a larger one
would mask them, and only the stored/re-run total can tell the difference.

## Root Cause

`cargo test` prints one `test result:` line per suite binary — a worker that
copies the first (or the most prominent) line quotes a real, correct number
that is nonetheless not the run.

## Recommendation

Never quote a bare passed-count as "tests green" without either summing every
`test result:` line the declared command actually printed, or checking the
stored record the project's `commands.test` run writes. See decision
`38f031b4` for the durable rule.

---

# Learning: A spec synced before a cell's judge-mandated rework can go stale on exactly the reworked points

**Category:** pattern
**Severity:** standard
**Tags:** [capture, scribing, rework, judge-review]
**Applicable-when:** resuming a capture pass after an interruption, when one
or more of the feature's `behavior_change` cells were reopened by a judge
verdict and re-capped AFTER the spec sync already landed.

## What Happened

`awaiting-human`'s spec sync (commit `59cfbada`) landed describing `ah-2`'s
three clearing paths and `ah-3`'s four reporting surfaces. Both cells were
then reopened by judge verdicts, reworked, and re-capped: `ah-2`'s rework
(`b7b422f9`) added the first test coverage of the human-message clearing path
through the REAL `UserPromptSubmit` hook entry point (the earlier tests only
proved the inner store function); `ah-3`'s rework (`c5dc3d70`) found and
wired a fifth reporting surface (the post-compaction capsule) and recorded a
deliberate deviation on a sixth (`bee status --brief`, frozen per
status-diet D1/D2) — none of which the already-landed sync could have known
about, since both reworks post-date it. Nothing in the sync commit or the
cell traces flagged staleness automatically; only a fresh read of the git log
(`b7b422f9`/`c5dc3d70` post-date `59cfbada`) surfaced it.

## Root Cause

A spec sync is a point-in-time merge of whatever cell state existed when it
ran. A cell reopened and re-capped after that point is invisible to the sync
unless something explicitly re-checks "did any capped `behavior_change` cell
this area's spec already claims to cover get re-capped after the sync commit
that claims it."

## Recommendation

When resuming an interrupted capture pass, diff the feature's cell commits
against the prior sync commit's timestamp/hash before treating the sync as
current — not just for gaps (uncaptured cells) but for cells that were
capped, synced, THEN reopened and re-capped. Re-verify every claim the sync
made about a reworked cell against the merged code, not against the sync's
own prose.

---

# Note: third occurrence of the incomplete-census pattern — no new record

`ah-3`'s original cap undercounted the reporting surfaces (four found, two
more surfaced only on judge-mandated rework: the compact capsule and
`status --brief`). This is the same defect class already promoted as
`pattern-20260807-fix-the-fan-out-not-the-cell-of-it-that-was-reported` and
extended once already by `traceable-runs`' `trun-9` recurrence (see
`docs/history/learnings/20260814-traceable-runs-capture.md`, third Learning
section). Per the promotion decision tree's recurrence-escalation step, a
third hit is not answered with a fourth doc line; the existing backlog row
tracking the general scan-copy census weakness (`p-804bb35b`, already
resolved for its own feature but naming the durable pattern) is the recorded
owner. No new backlog row filed here.
