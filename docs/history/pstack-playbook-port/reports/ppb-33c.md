# ppb-33c: check of the pstack teach merge in ppb-33

Checked against commit `77c60b029` (ppb-33). The source is the pstack skill
`teach` at `/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/teach/SKILL.md`,
the only file in that skill. The bee side is `skills/bee-teach/SKILL.md`, with
the procedures it rests on in `skills/bee-researching/references/trace-and-provenance.md`
("Trace", "Provenance sweep") and the close rule in
`skills/bee-hive/references/routing-and-contracts.md` ("Communication contract").
No file other than this report was edited.

## Verdict

The merge holds. Every pstack teach rule is present in bee-teach, or left out
with a named bee home or a named bee difference. None is missing. The port notes
in `port-notes/merge-ppb-33.md` list the same split.

- **pstack-only tool names:** none. A scan of `skills/bee-teach/SKILL.md` for
  `/loop`, `Graphite`, `bugbot`, `poteto`, `arena`, `architect`, `Cursor`,
  `disable-model-invocation` and the bare skill names `` `how` `` and `` `why` ``
  matches nothing. The pstack `how` and `why` invocations appear as Trace and the
  Provenance sweep, with `/bee-how` and `/bee-why` as the next-action doors.
- **bee rules removed:** none. The ppb-33 diff to `skills/bee-teach/SKILL.md`
  removes one token, `it.` at the end of "Give each concept one name and keep
  it.", and puts it back as `it, since switching between synonyms ...`. Every
  other change is an insertion. The gate-wins paragraph, the UNSWEPT rule, the
  one-next-action close, the user's-language rule, the Headless section and the
  Handoff are all unchanged.
- **Other checks:** the bee-teach description is unchanged at 269 characters,
  under 300. `skills/bee-teach/LICENSE.pstack` is byte-identical to the pstack
  `LICENSE`. The body and `LICENSE.pstack` match the source in all five rendered
  trees (`.claude-plugin`, `.codex-plugin`, `.agents`, `.opencode`, `.claude`).

## Rule by rule

P = present in bee-teach. L = left out, with the bee home or the bee difference.
M = missing.

### Frontmatter and opening

| # | pstack rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | Explain what a thing is, how it works and why, in one plain account at the person's pace; the goal is understanding, not change | P | Opening bold line, verbatim |
| 2 | Teach sits on top of `how` and `why`, real skill invocations that do their own digging | L (tool swap) | Opening paragraph names Trace and the Provenance sweep in `trace-and-provenance.md`; D2 swap |
| 3 | Get your bearings on what the work is and touches, then run them | P | Opening paragraph |
| 4 | Blend into one plain explanation, lead with what matters, go deeper when asked | P | Opening paragraph |
| 5 | Reword freely, but keep `why`'s confidence language intact; hedges are findings | P | Opening paragraph, widened to the empty and UNSWEPT categories |
| 6 | Let those skills investigate; do not redo it by hand | P | Opening paragraph and step 2 |
| 7 | `disable-model-invocation: true` | L (bee differs) | bee-teach triggers on a request to understand, per its description; port notes say so |
| 8 | Triggers "teach me this", "help me really understand X", "explain this change or subsystem" | P | bee-teach description ("teach me", "explain this", "help me understand") |

### Step 1, what they should walk away with

| # | pstack rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 9 | Decide the few things they should walk away understanding | P | Step 1 |
| 10 | Choose from why they ask and what they know, read from the conversation, not quizzed | P | Step 1, verbatim |
| 11 | Skip what they know; put the depth where their question is | P | Step 1 |

### Step 2, let how and why do the work

| # | pstack rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 12 | Read the code yourself to orient, then run `how` and `why` | P | Step 2, with Trace and the Provenance sweep |
| 13 | Run them in parallel and combine | P | Step 2 |
| 14 | Match the size to the question; one may be enough for a small change | P | Step 2 |
| 15 | Keep `why` narrow by default since the full sweep is slow | P | Step 2 |
| 16 | Put the narrowing in the ask itself so `why` records the skipped categories | L (bee differs) | Step 2 names each unswept category as UNSWEPT per `trace-and-provenance.md:263`; the sweep takes the scope directly |
| 17 | Widen only when the reasons are the point | P | Step 2 |

### Step 3, plain definition first

| # | pstack rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 18 | Start with a plain definition and its common name, as a senior engineer would say it | P | Step 3 |
| 19 | Tie it to the case, then build: how it works, reasons, edge cases | P | Step 3 |
| 20 | Explain how it works, don't just name it | P | Step 3, added by ppb-33 |
| 21 | For each part, the problem it solves and how it works | P | Step 3 |
| 22 | Walk through what happens as the person does the thing | P | Step 3 |
| 23 | Listing functions and constants is reference, not teaching | P | Step 3 |
| 24 | No framing labels (the listed phrases) | P | Step 3, same list |
| 25 | Smallest complete answer first, then stop; add layers when asked; never a wall | P | Step 3 |

### Step 4, a conversation

| # | pstack rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 26 | A conversation, not a lecture or a performance | P | Step 4 |
| 27 | Offer to go deeper or move on | L (bee differs) | Step 4 and Reply close on exactly one next action, never a menu (`routing-and-contracts.md:236`) |
| 28 | Follow their lead | P | Step 4 |
| 29 | No quizzes | P | Step 4 |
| 30 | No pacing theater (the listed phrases) | P | Step 4, same list |
| 31 | When you would pause, stop and let them respond | P | Step 4 |
| 32 | One-shot with no human: deliver cleanly, offer to go deeper at the end | L (bee home) | `## Headless` section: one pass, the one next action at the end |

### Step 5, show and build up

| # | pstack rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 33 | Show, don't only tell; open the diff, code or debugger | P | Step 5 |
| 34 | Draw when a picture lands faster than words | P | Step 5 |
| 35 | Three or more parts: a series, each diagram adds one part | P | Step 5 |
| 36 | The series is not a wall, since each step adds one idea | P | Step 5, added by ppb-33 |
| 37 | An all-at-once diagram, especially at the end, is reference | P | Step 5 |
| 38 | The A to B to C example, drawn three times | P | Step 5 |
| 39 | Three small growing diagrams beat one crowded one | P | Step 5, added by ppb-33 |
| 40 | Match the medium; mermaid where labels carry the meaning | P | Step 5 |
| 41 | Spatial ideas: the image-generation tool, whiteboard style, few labels; generate, don't describe | P | Step 5 (pre-dates ppb-33; see Observations) |
| 42 | The build-up rule holds for generated images | P | Step 5 |
| 43 | A single simple point needs no figure | P | Step 5 |
| 44 | A visual earns its place by teaching, not decorating | P | Step 5, added by ppb-33 |

### Voice

| # | pstack rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 45 | Write through the unslop skill | P | `bee-unslop` |
| 46 | Plain spoken English | L (bee differs) | "in plain spoken language, in the user's language" |
| 47 | The way you'd explain it to a colleague | P | Voice paragraph |
| 48 | Tight, not terse; cut filler, keep what makes it click | P | Voice paragraph |
| 49 | Padding is the enemy, not ideas | P | Added by ppb-33 |
| 50 | Don't list functions and constants like a changelog | P | Added by ppb-33 |
| 51 | Concrete mechanism, not metaphor, framing or preview | P | Voice paragraph |
| 52 | The target density example | P | Verbatim |
| 53 | Sentence case; no em dashes | P | Voice paragraph |
| 54 | Periods over commas; one or two commas a sentence; split piled clauses | P | Voice paragraph |
| 55 | One name per concept, because synonyms make the reader re-derive | P | Reason added by ppb-33 |
| 56 | No mirror sentences or tidy closers | P | Voice paragraph |
| 57 | The words are directions, not labels; don't echo the scaffolding as headers | P | "structure" replaces "scaffolding"; same rule |

### Reply

| # | pstack rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 58 | The explanation itself, never a report of what you did | P | Reply paragraph |
| 59 | Lead with the main point, then what, how and why | P | Reply paragraph |
| 60 | End with the threads worth chasing with `how` or `why` | L (bee differs) | One next action, a ready `/bee-how <question>` or `/bee-why <question>` |

Totals: 60 rules. 52 present, 8 left out with a named bee home or difference,
0 missing.

## Observations

- Rule 41 sends the reader to "the image-generation tool". Not every bee host
  has one. The line was in bee-teach before ppb-33 (`git show 77c60b029^`), so
  it is not a pstack name this merge added and is outside this cell. It matches
  the D2 concern, a step naming an absent tool, so it may be worth a separate
  look.
- `77c60b029` also adds `.claude/skills/bee-teach/CREATION-LOG.md` and rewrites
  44 lines of `.claude/skills/bee-teach/SKILL.md`. The `.claude` tree was behind
  the source before ppb-33, and the render brought it up to date. It now matches
  the source.
