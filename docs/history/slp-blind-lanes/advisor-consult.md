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

---

# Round 2 — re-consult on the settled shape-B plan

- Date: 2026-08-28 · Tier: advisor (fable)
- Why: the user picked shape B at the scope fork, so `plan.md` changed and a new
  decision (`f0f21142`) landed. The execution gate refused the round-1 consult as
  stale, by design.
- Verdict: **SAFE WITH NAMED CHANGES** — three text-level edits, none of which
  touches the approved shape.

## A. Is the leak hazard reduced, or moved?

Split. **Reduced for proposals**: in shape A round-1 proposals sat in
`.bee/blind/` before stragglers ran; in shape B they exist nowhere on disk until
the dossier renders. **Moved, not reduced, for the open reason**:
`.bee/blind/` → `.bee/decisions.jsonl` is the same disk and the same
readability, and D1 locks "logs the reason at open time", so that window is a
locked decision rather than a plan choice.

The plan's claim that the diet check has "the same trust level as D4's citation
check" was FALSE and is corrected. D4 checks bytes the checker holds, so a
fabricating lane is caught whether or not it cooperates. The diet check reads the
lane's own paths-read list: a lane that reads `.bee/decisions.jsonl` and omits it
passes clean. It is a prompt instruction plus a confession requirement. What IS
structural is `prepare.rs:563-566` — zero store context reaches a non-cell
payload — so a breach takes active defiance of the prompt.

## B. Byte-identity verified rather than constructed — the failure walk

Lane 1 reads brief v1 and gets `sha_a`, appended to `.bee/logs/dispatch.jsonl`
by `append_prepare_record` (`prepare.rs:601-613`). The file is edited. Lane 3
gets `sha_b`. Nothing during the run compares anything. Three holes decision
`f0f21142` did not name:

1. **No chain of custody.** The dossier's lane section carried no `dispatch_id`,
   so `blind check` would compare the orchestrator's own transcriptions against
   each other — verifying the transcriber against itself.
2. **The authoritative record is fail-open.** `prepare.rs:599-600`: "a log
   failure never blocks the payload".
3. **Nothing forces the check.** No door gates the convergence decision on a
   green `blind check`.

Cost to the human: prevention became optional post-hoc detection over
self-transcribed data, found after two rounds across three lanes of spend, with
the cross-critique round already contaminated by proposals answering different
questions. Acceptable with two cheap fixes, both folded: the dossier carries a
per-lane `dispatch_id` and `blind check` verifies against `dispatch.jsonl`
(refusing by name when the id is absent); and slice 4's prose makes convergence
run `blind check` green before it logs the decision.

## C. Cold read of `bln-1` and `bln-2`

Every code anchor in both cells verified correct against HEAD — the render call
at `prepare.rs:565-566`, the ten-name `keys_known` list at `:1629`, the record
build at `:1389-1392`, `sha256_hex` at `leases.rs:89`, `PINNED_FLAG_COUNT` at
`catalog.rs:632`, and `prompt.rs:72/:118/:156/:176-181`. `unmapped_kind_refusal`
carries its `type` field; `advisor_not_configured` verifiably lacks it.

**`bln-1` is one cell, defensibly** — the prompt edit, rebuild, regen and
manifest rewrite are one atomic unit, and splitting the digest out would cost a
second regen cycle over the same files. It sits at the ceiling: 8 files, one
hand-edited generated file, six test authorships.

Guess points found and closed: the cell never said what `--brief-file` does on a
non-advisor kind (gather and reviewer templates carry no brief block, so it would
silently vanish) — now an explicit refusal for cell, gather and reviewer; the
regen step invited duplicate hand-runs of `onboard --apply` and
`release-manifest --write`, which `bee dev regen` already does
(`devtools/mod.rs:141-150`); and the reuse-check reason argued only against
`--purpose`/`--expertise` when the nearest existing spellings are `--file`,
`--task-file` and `--digest-file`. Most likely failure: the vendoring tail, or
the trailing-newline trap where a falsy `{{#if}}` block eats its own leading
newline — both caught red by the cell's own probes.

**`bln-2` carried one instruction that contradicted itself.** The corpus test
("run the guard over every `packages/bee/prompts/*.md`, assert zero fires")
cannot pass with the shape arm included: no prompt file carries the four required
sections, so the shape arm fires on all of them. The cold worker's only path
would have been silently re-scoping the corpus — the exact silent deviation the
cell forbids. Now scoped to the verdict-stem arm explicitly. Verified: zero of
the stems appear in any prompt file today. Also closed: the shape arm's section
matching is case-insensitive on trimmed heading text, and the scope test's
`--expertise` leg must use `<path> :: <purpose> :: <read-to>` lines or
`parse_expertise` (`prepare.rs:527-539`) refuses before the guard is reached.

## D. The stem list

Ship it. It is not pure false-confidence: the honest "leaning language refused"
naming already defuses the certification critique, and the shape arm cannot catch
a declarative verdict smuggled into a prose line of Constraints. Two cut:
**"the right answer"** and **"the right approach"** — they fire on neutral
interrogative phrasing ("What is the right approach for X?"), which the
impersonal stems have no natural use for. Seventeen remain.

## E. Ranked misses, all folded

1. Digest chain of custody — no `dispatch_id` in the dossier.
2. `blind check` wired to nothing; the traded-away by-construction guarantee was
   replaced by a verification nothing forces.
3. The self-contradictory corpus test.
4. An internal lint-scope contradiction (brief bytes only, versus "the brief and
   the open reason"), plus a fold-claim that overstated round 1 — "lint the open
   reason" was dropped, rightly, and now says so.
5. `bln-1`'s silence on non-advisor kinds.
