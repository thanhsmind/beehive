# Learnings — tick-contract-inline (2026-07-29)

**Feature:** moved the operative progress-tick contract out of an on-demand reference
and into the always-loaded operating law, fixed the pointer matcher that had been
falsely reporting the rule unreachable, and added a gate that fails the build when an
every-turn rule is stored where the reader loads it only on demand.

**Scale:** 3 cells. **Outcome:** `PASS run_verify: 118 suite(s)` (117 → 118).
**Decisions:** T1-T7, `docs/history/tick-contract-inline/CONTEXT.md`.

---

## N1 — A rule stored where the reader loads it on demand is not in force

Progress ticks were specified as a *"mandatory ak-style per-step contract"* — one short
line per perceivable step, on by default, unconditional. They never happened, and the
reason was not carelessness. Three things stacked:

1. The rule lived only in a **reference file**, loaded on demand.
2. The **always-loaded summary** that stood in for it was narrower on two axes: it read
   as conditional on bypass, and scoped to four event types, when the rule is
   unconditional and covers every step.
3. The only check that would have caught the gap was **both broken and advisory** —
   it matched the wrong string, and it always exits 0.

The second is the subtle one. An agent following the summary exactly still fails the
contract, and has no way to notice: the summary reads as complete. A summary that
narrows a rule is worse than no summary, because it substitutes for the rule while
misrepresenting it.

**What generalises.** Scope of application decides storage location. A rule that applies
every turn belongs in the layer loaded every turn — anything else is a rule you have
written down but not put into force. When you summarise a rule into a smaller layer,
the summary owes the rule's *scope* even if it drops the detail.

## N2 — An advisory finding is worth what it can cost the person who ignores it

`skill_lint` had been reporting `bee-hive/SKILL.md has no pointer to "Progress ticks"`
for as long as that line carried two headings. The pointer was there and perfectly
readable; the matcher searched for a parenthetical containing exactly one quoted
heading, and the line reads `("Silent Bookkeeping", "Progress ticks")`.

Two workers **in this same session** read that warning, recorded it as *"pre-existing
advisory, unrelated"*, and moved on. Both were right to, individually. That is what a
false positive in a non-blocking check buys: it trains every reader to skip the output,
so the day it reports something real, nobody looks.

The compounding failure is worse than either half. A false-positive check that *can*
block gets fixed within a day, because it stops work. A false-positive check that
cannot block survives indefinitely and takes the credibility of its true positives with
it.

**What generalises.** A check has two properties that interact: whether it is correct,
and whether it can cost anything. Incorrect-and-blocking self-corrects.
Incorrect-and-advisory is stable, and its stability is the problem.

## N3 — Derive the trigger from wording, not from a marker the author must remember

The new gate has to know which rules claim to apply every turn. Two options:

| Approach | Failure mode |
|---|---|
| An explicit every-turn marker in the doctrine source | **Silence** — the marker is applied by the same author who just made the filing mistake |
| Deriving from the rule's own wording | **False positive** — a red build a human clears by moving or rewording the rule |

The worker chose wording-derivation on exactly that reasoning, and recorded it in the
suite header rather than only in the report. The argument holds generally: when a gate
depends on an annotation, the annotation is applied by the person the gate exists to
catch, so its miss-rate and the defect rate are the same number.

**Choose the direction your gate fails in.** A gate that fails toward flagging is
recoverable by a human in one step. A gate that fails toward silence is indistinguishable
from a passing build.

The seed the worker kept was six regexes for how English states per-turn scope — a
domain fact about wording, not a location and not a roster. Bare `every step` / `per
step` were tested and **rejected** because they produced four false positives. Naming
the seed and its rejected candidates is the honest form of "derive, don't hardcode."

## N4 — The gather's byte count was wrong and the executing worker caught it

Planning was built on a measured block size of 12,692 bytes, giving 1,308 bytes of
headroom. The actual size was **13,396** — the extraction-tier gather had reported a
stale number, and CONTEXT.md carried it forward into a locked decision's rationale.

The executing worker re-measured before trimming and recomputed the removal target from
the measured size rather than the recorded one. Had it trusted the cell, it would have
paid 550 bytes against a 704-byte deficit and shipped a warn-line breach.

**What generalises.** A number in a plan is a measurement with a timestamp, not a fact.
Any cell whose action depends on a threshold should re-measure at execution time — the
cost is one command and it converts a silent overrun into a non-event.

## N5 — Paying for text with text found two real bugs

The addition was ~833 bytes and the budget was zero, so 835 bytes had to come out of the
same document — no threshold raised, per the repo's own fence doctrine.

Hunting for genuinely removable prose surfaced **two stale cross-reference pointers**,
both off by one, sending readers to the wrong rule. They had been wrong long enough that
nobody had followed them.

That is not a coincidence. A budget of zero forces a reading of the whole document with
the question *"would anyone miss this?"* — which is also the question that finds text
nobody has read closely in months. Raising the threshold would have added the new rule
and left both pointers wrong.

## N6 — Adding a test file has a registry side-effect

Creating any `test_*.mjs` under the suites directory makes the impact registry stale,
which turns `test_impact_registry` red **and** fails a CI step. The suite count moves
(117 → 118) with no edit to the runner, because discovery is a glob.

The worker caught it and regenerated. Worth knowing before authoring a suite: the file
is not the whole change.

## Residual

- `p-c15fb6f5` — the pointer-integrity anchor check is worse than its report described:
  a parenthetical with a second heading matches *nothing*, so the entire citation is
  skipped and the target's existence goes unchecked. The scribe verified this by
  execution rather than by reading, and corrected the record.
- Tick **emission** stays unenforced and every artifact says so. Nothing in this repo
  observes agent chat output. The new gate proves a rule is *reachable*, never that it
  was *followed* — the concept, the suite header, and the close report all state that
  limit rather than letting a green check imply coverage it does not have.
