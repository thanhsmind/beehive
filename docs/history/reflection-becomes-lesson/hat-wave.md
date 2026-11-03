# reflection-becomes-lesson — plan-step hat wave

**Date:** 2026-08-31 · **Seats:** five (high-risk) · **Quorum:** all five
returned inside the ceiling · **Synthesis:** the leader.

The wave was the plan check and the high-risk advisor consult, per the
absorption rule. It changed the plan in four material ways.

## What the wave changed

| Seat | Finding | Disposition |
|---|---|---|
| facts-gaps | **B1 BLOCKER** — reflections are stored per run; `bee close` resolves its own session's run, which under the default `uat_stop: "close"` is a different run from the workers'. A run-scoped door refuses every close and leaves the clean-run flag as the only available answer. | **Adopted** — D5 `20fe96d3`: the answer rides the cell trace and the door walks the feature's capped cells. |
| facts-gaps | **B2 BLOCKER** — rows 11 and 12 anchored `worker-cell.md:122`; the file is 56 lines. Fabricated line numbers under a true claim. | **Adopted** — re-anchored at 44 and 47, verified by `wc -l` and `rg`. |
| facts-gaps | **B3/B4 BLOCKER** — "a close files unconditionally" and "the deviations lane demonstrably works" were load-bearing prose with no claims row. | **Adopted** — rows 10 and 16 added, both verified. |
| facts-gaps | W1, W5, W6 — three anchors off by a few lines. | **Adopted** — corrected. |
| facts-gaps | **W3** — the plan never said which text is shape-tokenized; tokenizing the rendered bullet defeats D3's repeat rule. | **Adopted** — rbl-2 tokenizes `item.what`, stated explicitly. |
| facts-gaps | **W4** — a run that caps cells and never closes a feature still ends silent. | **Named as a known limit**, not fixed: D1's door is the close door. |
| facts-gaps | W7 — `CONTEXT.md` absent from the worktree. | **Fixed** during the wave. |
| alternatives | **FAIL** — the three-cell split was false: rbl-1's collection lane and rbl-3 both edit `worker-cell.md`. | **Adopted** — folded into one cell; two cells, two slices. |
| alternatives | Rejected a merged rbl-1+rbl-2 (the miner deserves its own revert; red-first is cleaner against an unmodified `trouble_lines`) and rejected riding `PLAN_FOLLOWED` (D2 locks the encoding, and `plan_followed` is a per-cell trace flag, never a mailbox entry). | **Recorded** in the SMALLER PATH section. |
| alternatives | No doctrine home derives from another — `pointer_integrity.rs:35` lists the two AGENTS sources separately, with no parity sync. | **Adopted** as claim row 15. |
| user-impact | The two-part shape produces signal only when written at the moment of notice; demanded cold at close it produces filler. | **Adopted** — the doctrine wording is part of rbl-1 and must carry LR2's moment-of-notice rule and demand a concrete noun. |
| user-impact | Drafted the refusal text: name what the letter cannot say, offer the reflect command first and the clean-run flag second, end on the human reason. | **Adopted** as the refusal's shape. |
| user-impact | **The lesson ceiling** — `normalize_shape` does lowercase plus digit-collapse only, so free-prose reflections about the same mistake rarely hash alike. Auto-mined lessons will be rare. | **Adopted** — stated openly in "The honest ceiling". |
| user-impact | The enforcement grain (per run at close) is coarser than the user's ask (per task). | **Resolved by D5** — the cell-scoped record is per task. |
| risks | **F1 MAJOR** — same seam as B1, reached independently: a resumed or handed-over session can only answer mechanically, not truthfully. | **Adopted** — D5. |
| risks | **F2 MAJOR** — a hollow clean-run answer has no defense. Counted the precedent: 65 of 115 caps carry a substantive departure object (57%), zero hollow plan-followed objects. | **Adopted** — D6 `7760339d`: the ratio is reported, so collapse is visible. |
| risks | **F3 SAFE** — exit 1 at close does not loop or stall: no script, hook, CI job, or the herding control loop calls `bee close`; all doors run before any write. | **Adopted** as the "Cost if the shape is wrong" evidence. |
| risks | **F4 MINOR** — an old or reverted binary could render the new kind as a Done bullet or elect it as the subject. | **Adopted** — the new kind is excluded by the `is_reflection` predicate family and its `what` must fail `check_subject`. |
| risks | F5 LOW — frontmatter consumers hold while the additive discipline is kept. | **Adopted** as claim row 17. |
| risks | Suggested a config escape valve for the refusal. | **Rejected** — the departure door it mirrors has none, and a switch that turns the record off is a switch that will be left off. Recorded in "Cost if the shape is wrong". |
| value | **The headline evidence was inflated** — "30 filed letters over a month" was 4 letters and 26 entry stores over one day; doctrine-only was never actually attempted. | **Adopted** — corrected in both `plan.md` and `CONTEXT.md`, and the SMALLER PATH argument restated on the structural ground (an unnamed verb is unreachable) rather than the empirical one. |
| value | The lesson miner has never produced a row: 0 `lesson`-tagged decisions, 0 digest files. rbl-2 widens a pipe that has never run. | **Adopted** — folded into "The honest ceiling"; the widening stays, guarded red-first. |
| value | `docs/knowledge/patterns/` (174 files, produced daily) is the pipe that already works — but it is discipline-triggered and voluntary, and it does not run in host repos. | **Adopted** — the difference is real; the feature's value is host repos and dispatched workers, where the letter is the only artifact the human sees. |

## The one thing the wave did not resolve

Whether a Task-dispatched worker inherits the orchestrator's session
environment, and therefore shares its run id. D5 makes the question
non-load-bearing — the door reads cells, not runs — so it is recorded
here rather than chased.
