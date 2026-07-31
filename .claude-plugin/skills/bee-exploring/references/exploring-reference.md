# Exploring Reference

Full mechanics for rules the body states as a pointer. Cross-referenced from
`SKILL.md`.

## Batching mechanics

*Independent* = its answer changes the framing of, and makes redundant, no
other pending question — ask all of a phase's independent survivors in ONE
`AskUserQuestion` message (the tool takes up to 4 questions, each with a
`header` ≤12 chars and 2-4 options — overflow makes the harness reject the
whole call as "Invalid tool parameters"; full schema in
`bee-hive/references/routing-and-contracts.md` → Gate Presentation Contract).

*Dependent/branching* = its wording or its very existence hinges on another
question's answer — ask it alone, only after that answer lands. Never
blind-bundle: a question a prior answer could moot is dependent, not
independent, and never rides in the batch.

**Pre-classify for batching** (produced by the delegated step-3 pre-pass, not
composed inline): tag each candidate question *independent* or *dependent*,
and for every dependent one name the question + answer it hinges on (the
dependency edge). This slate — questions + tags + edges — is the input to
Socratic Locking's batching: the interactive phase opens already holding the
whole plan instead of composing each question inline, one round-trip at a
time.

**Ordering in Socratic Locking:** start broad, then narrow — the broad
questions are the ones others depend on, so they lead: the independent batch
first, then the dependents it gates.

## Command detection

If `.bee/config.json` lacks `commands` (setup/start/test/verify), run
detection first: `node .bee/bin/lib/commands_detect.mjs` prints JSON
candidates from the repo's manifests. Present the candidates as **one**
pre-filled confirmation question (`key: value — source`), still skippable;
fall back to the open question when detection finds nothing. Write only
user-confirmed values to `.bee/config.json` `commands`. Never invent command
values.

## Backlog flip

When this feature matches an existing PBI, run
`node .bee/bin/bee.mjs backlog pbi status --id <id> --to in-flight --feature <slug>`
same turn (one move — status and slug together), then
`node .bee/bin/bee.mjs backlog render --write`; if the request never passed
through the backlog, run
`node .bee/bin/bee.mjs backlog pbi add --title "<story>" --cos "<CoS>"` first,
then the status flip. This is the only place a PBI goes `in-flight` (id/verbs
+ merge rules live in the scribing reference; prose-ruled, never
hook-enforced). A PBI whose status is `parked` is the one exception — see the
Brief check below before touching it.

## Brief check

If `docs/history/<feature>/CONTEXT.md` already holds a `bee-qualifying` park
brief under `Outstanding Questions` → `Resolve Before Planning`, load it
instead of a fresh quick-scout: its evidence is settled ground, and gray-area
questions come only from what the brief still leaves open. When this brief
was loaded, step 4's gray-area candidates are drawn only from what it still
marks unclear — skip the quick-scout entirely, its evidence already covers
that ground.

## Materiality test

Every candidate question passes three checks before it is asked:

- **material** — the answer changes scope, architecture, UX, data model, or
  acceptance criteria
- **grounded** — cites scout evidence or a concrete uncertainty, never
  generic preference
- **answerable** — the user can pick an option, approve a default, or supply
  a reference

A failing question is never asked: pin it as a labeled assumption for Context
Assembly to write into CONTEXT.md, or hand it to planning if only the
implementer cares about the answer.

## Gate-bypass refinement

Information vs approval. Read the active level (`gate_bypass_level`). Under
`full`/`total`, split every candidate question in two:

- An **approval** question — one where the agent already has a confident
  best answer and the user would only rubber-stamp it — is **NOT asked**;
  lock it from that recommendation (record it as a decision with its D-ID)
  and move on.
- Only an **information** question — one whose answer the agent genuinely
  cannot determine from evidence with a confident default, because it turns
  on a preference or knowledge only the user holds — is still asked, even
  under `total`.

The litmus is one line: *"do I already have a confident best answer?"* — yes
→ proceed with it; no, and only the user can supply it → ask. This never gags
a real information need (the user explicitly wants to still be asked for
those); it only stops the agent asking merely to be approved. Under
`off`/`normal`, ask per the materiality test as usual.

## Blindspot pass

Teach before asking. When the user signals unfamiliarity with a gray area's
domain — says so, answers with guesses ("chắc là…"), or asks what the options
mean — invert for that area: explain the 2-3 concepts needed to answer well
(one short outcome-framed message, no jargon), *then* ask. A decision locked
from a guessed answer is a fake decision. The user can also request a full
"blindspot pass" by name: sweep the unknown-unknowns (what good looks like,
common potholes, prior art in this repo) before locking begins.

## SEE mock

React instead of describe. For a `SEE` gray area the user knows-when-they-
see-it but cannot describe, you MAY build a throwaway HTML mock (2-4
variants, fake data, zero wiring) under `.bee/spikes/<feature>/mocks/` and
lock the decision from the user's reaction, citing the chosen variant. This
is the ONE exception to "exploring never writes code": mock files only, only
under `.bee/spikes/`, never imported by anything, never promoted to
production (spike-code rule applies).

## Pinned terms

When an answer settles the meaning of a fuzzy domain word, confirm the term
back and pin it like a decision; Context Assembly writes all pinned terms
into CONTEXT.md's `Terms` section, and scribing inherits them into the spec's
Data Dictionary.

## Deferred ideas backlog

Each Deferred Ideas entry that is real future work gets
`node .bee/bin/bee.mjs backlog pbi add --title "<story>" --cos "<CoS>"` (then
`backlog render --write`) in the same turn (announce-then-do) — the
CONTEXT.md list is the record for this feature, the PBI is the durable
product-level intent. Do not wait to be asked.

## Fresh-eyes review

Spawn one reviewer with no conversation history (slot: `review` — default
opus on Claude, falls back to generation) — **in the background where the
runtime supports it**: keep assembling CONTEXT.md, keep talking to the user;
the review blocks nothing until Gate 1. Collect the verdict before presenting
the gate — Gate 1 is never presented with the review still outstanding. It
checks completeness, contradictions, vague decisions, missing D-IDs, and
blockers. Fix findings and re-review — max two loops, then present remaining
doubts to the user.

## Gate 1 bypass mechanics

Read the active level FIRST (`node .bee/bin/bee.mjs status --json` →
`gate_bypass_level`), before presenting anything. If the level bypasses Gate
1 for this feature's lane — `normal` covers `tiny`/`small`/`standard`
non-hard-gate; `full`/`total` cover **every** lane incl. high-risk/hard-gate
(the human lifted that floor) — then **do not present the gate question.**
Instead:

1. Take the CONTEXT.md as locked (the recommended path).
2. Set `approved_gates.context` yourself:
   `node .bee/bin/bee.mjs state gate --name context --approved true`.
3. Log a one-line audit decision:
   `decisions log --decision "auto-approved Gate 1 (bypass): <feature>" --rationale "..."`.
4. Post a short non-question line:
   `⚡ auto-approved Gate 1 (bypass) — locking CONTEXT.md, invoking bee-planning`.
5. Continue straight to `bee-planning`.

Only when the level does NOT cover this gate (`off`, or `normal` on
high-risk/hard-gate) do you present the human layer and ask, per the Gate
Presentation Contract: plain-language layer in chat — what we decided / why
trustworthy / cost if wrong / what you are deciding — in the user's language,
CONTEXT.md linked not pasted; then verbatim: "Decisions locked. Approve
CONTEXT.md before planning?"

## Re-lane checkpoint

Same demotion rule as every lane-scaled skill: full text in
`bee-hive/references/routing-and-contracts.md` ("Re-lane checkpoint"). In
exploring it applies once the scout's touch set is counted — measured
evidence may demote `standard` → `small` once (files within threshold, zero
hard-gate flags, zero open gray areas — all three). Never `tiny`, never
twice. Log it, tick it.
