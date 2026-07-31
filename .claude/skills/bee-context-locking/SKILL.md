---
name: bee-context-locking
description: >-
  Write docs/history/<feature>/CONTEXT.md from a caller's resolved decisions -- the single shared writer for both the human-interactive path (bee-exploring) and the automatic path (bee-qualifying). Two modes: lock (write/update locked decisions) and park (write evidence + open questions into Outstanding Questions, and flip the backlog row to parked). Do NOT use to originate decisions, scope, or approach -- those come from exploring/qualifying. Never presents a gate.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: degraded
      reason: Reads bee records via the vendored .bee/bin helpers when the fresh-eyes review or backlog write needs them.
---

# Context Locking (the shared scribe)

`bee-context-locking` is the single place `docs/history/<feature>/CONTEXT.md` gets
written — whether the input decisions came from `bee-exploring`'s human Socratic
session or `bee-qualifying`'s automatic self-assessment. It renders; it does not
decide. Precedent: `bee-briefing` (renders one artifact from truth artifacts, never
originates content).

If `.bee/onboarding.json` is missing or stale, stop and invoke `bee-hive`.

## Hard Gates

- **Never originate a decision, boundary, term, or scope note.** Every locked-decision
  row, term, and boundary sentence comes from the caller's resolved input verbatim (or
  a light restatement of it) — inventing content to fill a section is the one failure
  this skill exists to prevent (mirrors `bee-briefing`'s own Hard Gate).
- **Single writer.** This is the only skill that writes `docs/history/<feature>/
  CONTEXT.md`. A caller about to write CONTEXT.md directly routes through here instead.
- **Never invent a park-brief file format.** A parked item's evidence and open
  questions land in CONTEXT.md's existing `Outstanding Questions` section — never a
  new file, never a new section name.
- **Never present a gate.** Gate 1 (and any Gate 2 the caller drives) stays the
  caller's job (`bee-exploring` step 6, `bee-qualifying` step 4a) — this skill only
  writes and reports back.

## Modes

| Mode | Caller | Trigger | Does |
|---|---|---|---|
| **lock** | `bee-exploring` (human path) or `bee-qualifying` clear path (step 4a.1) | resolved locked decisions ready | write/refresh `docs/history/<feature>/CONTEXT.md` from `references/context-template.md`; append `proposed` backlog rows for real Deferred Ideas; fresh-eyes review |
| **park** | `bee-qualifying` park path (steps 4b.1-2) | evidence gathered, item judged ambiguous/large | write evidence + open questions into CONTEXT.md's `Outstanding Questions`; flip the item's `docs/backlog.md` `Status` row to `parked`, same commit |

## Flow — lock mode

1. **Write CONTEXT.md** from `references/context-template.md`, using exactly the
   caller's resolved input: boundary, domain types, locked decisions table with
   D-IDs, pinned terms, scout paths, canonical references, open questions, deferred
   ideas. Concrete language only — no placeholders, TODOs, or vague preferences; a
   section the caller's input left silent is an Open Question, never a guess (mirrors
   `bee-briefing`'s own "source silent → Open Question" rule).
2. **Deferred Ideas feed the backlog.** Each Deferred Ideas entry that is real future
   work appends a `proposed` row to `docs/backlog.md` in the same turn — the
   CONTEXT.md list is the record for this feature, the backlog row is the durable
   product-level intent.
3. **Fresh-eyes review.** Spawn one reviewer with no conversation history (slot:
   `review` — default opus on Claude, falls back to generation) — in
   the background where the runtime supports it. It checks
   completeness, contradictions, vague decisions, missing D-IDs, and blockers. Fix
   findings and re-review — max two loops, then hand remaining doubts back to the
   caller in the report (the caller decides whether to surface them to the user;
   this skill never asks directly — it has no gate to ask at).
4. **Return to the caller.** Report: CONTEXT.md path, whether the review passed clean
   or has remaining doubts. The caller (`bee-exploring` or `bee-qualifying`) drives
   Gate 1 from here — this skill never does.

## Flow — park mode

1. **Write the brief.** Into the feature's `docs/history/<feature>/CONTEXT.md`
   `Outstanding Questions` → `Resolve Before Planning` section (creating the file
   from the template first if this is the item's first pass), record what the
   caller gathered — the evidence and what is unclear. Reuse this existing
   structure verbatim; never a new file, never a new heading.
2. **Flip Status to `parked`.** In the same commit as the brief, set the
   item's `docs/backlog.md` `Status` column to `parked`. This is the ONLY place a
   row becomes `parked` — mirrors the convention `bee-exploring` step 1 already
   uses for `in-flight`, extended with the 4th status value.
3. **Return to the caller.** Report: CONTEXT.md path, backlog row updated.
   `bee-qualifying` stops here — no further skill invoked; a human later picks
   the item up via `bee-exploring`, which loads this brief instead of re-gathering.

## Red Flags

- inventing content for a section the caller's input left silent — Open Question,
  never a guess
- writing CONTEXT.md, or the backlog `Status` column, from anywhere else in the
  codebase (this is the single writer)
- a new park-brief file or section instead of `Outstanding Questions`
- presenting or self-approving a gate — that is always the caller's job
- skipping the fresh-eyes review, or treating an unresolved review doubt as resolved
- flipping `Status` to anything other than `parked` in park mode, or touching
  `in-flight`/`done` (those stay each caller's own convention, untouched here)

When a rule's letter stops serving its purpose here, say so out loud and
deviate with a recorded reason — boundary rules (gates, state, secrets) hold
as written; silent deviation is the defect (bee-hive routing reference,
"Judgment contract").

## Handoff

- **lock mode:** CONTEXT.md written, review clean (or doubts reported). Return to
  the calling skill (`bee-exploring` or `bee-qualifying`) for its own Gate 1
  handling.
- **park mode:** brief written, backlog row `parked`. Return to the calling skill
  (`bee-qualifying`) — no further skill invoked.

Reference: `references/context-template.md` (kept in sync with `bee-exploring`'s
copy of the same template).
