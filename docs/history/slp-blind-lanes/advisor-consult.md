# Advisor consult — blind-lanes shape gate

- Date: 2026-08-28 · Tier: advisor (fable) · Repo HEAD at consult: `95d1273e`
- Question: is the blind-lanes plan safe to approve at the shape gate?
- Verdict: **SAFE WITH NAMED CHANGES**

## 1. The lint is the wrong instrument for the claim it makes

The two guards D2(a) copies work because their vocabulary is CLOSED and
near-mechanical: `matches_supersession_prose` scans one stem plus four fixed
phrases (`verbs_read.rs:313-353`), `matches_deferral_prose` the same
(`:365-430`). A supersession cannot be written without one of those words, and
the refusal's remedy is a link, not a judgment.

Neutrality has no closed vocabulary. Real leaning is structural — the favored
option listed first with more detail, framing, a question that embeds its
conclusion, sunk-cost context. None of it has a stem.

- **False-pass rate: high.** A word list catches only the lazy first-person
  leak ("I recommend", "the right answer is").
- **False-fire rate: nonzero and painful.** "prefer", "should", "better" occur
  in quoted requirements and code comments. The zero-false-fire corpus test then
  pressures the author to shrink the word list until both directions pass —
  the 2026-08-12 pattern exactly: the corpus and the word list get co-tuned by
  one author into mutual agreement.

As the WHOLE neutrality story a word list is worse than nothing: it converts
"unlinted" into "certified neutral" at the door the feature's promise rests on.
As ONE instrument with an honest name it is cheap and catches the laziest leak.

**Named change.** Keep D2(a)'s lexical scan, narrowed to verdict stems, and add
a mechanical SHAPE rule that does the real work: a LaneBrief must not enumerate
candidate answers. Lanes exist to generate options; a brief that lists them has
already led the witness. Required sections (Question / Constraints / Read diet /
Digest contract) plus a refusal on an options-shaped section is deterministic
and testable red-first. This is delivery-PLUS, not an escalation — D2(a)'s
lexical refusal at the door still ships as written. Replacing the lexical scan,
or moving the check off the door, WOULD supersede D2(a) and needs the user.
Either way the refusal text and the skill prose must claim "leaning language
refused", never "neutrality enforced".

## 2. The blindness is stated, not built — and the plan stores the leaning on disk

`purpose_is_gather` (`prepare.rs:115-117`) only means bee injects no store
context and refuses `--claim`. The research digest concedes the hole itself: a
dispatched subagent inherits the OS cwd and can read `.bee/state.json`,
`.bee/decisions.jsonl`, `docs/discovery/*` — bee's hooks guard writes and
secrets, never reads (`slp-blind-lanes-surfaces.md:14`).

D3 is right — advisors genuinely lack session history and `learned_context`
(cell-only injection, `prepare.rs:582-593`). But the plan makes the hazard worse
than it must be: `bee blind open` records the open reason and the brief in
`.bee/blind/`, a readable path on the same disk, and D1 forces that open reason
to state why the decision is high-stakes — routinely the orchestrator's own
suspicion. Round-1 proposals land there too, before any straggler or re-dispatched
lane runs.

**Cheapest real hardening, no read hook needed.** The advisor digest contract
already requires "return the paths read". Make `bee blind proposal add` REFUSE a
proposal whose paths-read list falls outside the recorded diet, or names `.bee/`
at all — string containment at a verb slice 1 already builds, the same trust
level as D4's citation check. Plus two procedure rules: lint the open reason
with the same lint, and exclude `.bee/blind/` and `.bee/state.json` from every
diet by construction. Silent diet breach becomes a typed refusal or a recorded
lie, which is all bee's proof discipline ever claims anywhere.

## 3. The skeleton is two slices wearing one label

Cut at converge:
- **1a — the door and the record**: `--brief-file`, the vars slice, the
  `{{brief}}` placeholder, the lint refusal, `blind open`, `proposal add`.
  End-to-end blind run; convergence done by hand.
- **1b — `blind converge`**: dossier render, citation check, the printed
  `decisions log` + `triggers add` calls.

The citation check is its own risk surface with its own red-first proof; welding
it to the door change makes one big-bang gate out of two independent
verifications.

**Highest-risk cell: the `prepare.rs` change.** It edits the chokepoint every
dispatch passes through, and a false-firing lint blocks advisor consults —
including the one Gate 3 REQUIRES for high-risk work
(`high_risk_advisor_refusal`, `set_gate.rs`). A lint bug there can deadlock the
high-risk workflow itself.

## 4. What the plan misses, ranked

1. **Lint scope is unstated and can brick Gate 3.** Nothing says the lint fires
   ONLY on `--brief-file` content. If it touches `--purpose` or `--expertise`, a
   false fire blocks the mandatory high-risk consult and therefore Gate 3 — the
   guard would jam the gate machinery that approves guards. P1.
2. **The open reason is a stored leaning leak** (§2).
3. **Byte-identity (D2b) is not enforced where it matters.** `--brief-file <path>`
   is read independently per `dispatch prepare` call; the file can change between
   lane 1 and lane 3. A record-time equality check fires AFTER the lanes already
   ran on divergent briefs. Fix by construction: prepare renders from the stored
   run bytes (`--blind <run-id>`) or verifies a hash recorded at `blind open`.
4. **D4's containment check passes short fabricated citations.** "read-only" is
   contained in every proposal ever written. Needs a minimum citation length and
   per-lane scoping — a citation is lane-id plus quote, checked against that
   lane's bytes only.
5. **Slice 4's dependency claim for D6 is false.** The 5-Layer / Truth-Table /
   CRUD checklist material describes reviewer craft, not blind-lane behavior. If
   the feature stalls after slice 1 — the common case — a locked decision sits
   undelivered for no structural reason. D6 can ship independently at any time.

## 5. Verdict

Safe with named changes (a)–(d) above. Cutting slice 1 at `converge` is
recommended but not blocking.
