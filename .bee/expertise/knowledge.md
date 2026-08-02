# How to build a project's knowledge base

A knowledge base holds what is true *in this project* and expensive to
re-derive. It is not a wiki, not a changelog, and not a second copy of the
code. Every rule below follows from one question: what does the next person
to open this repo — a teammate, a stranger, an agent with no memory of
yesterday — need to know that reading the code will not tell them cheaply?

## Where to look

| Situation / goal | Entry |
|---|---|
| Deciding whether a fact belongs here at all | Two layers, one question each |
| A newcomer has no idea where anything is | The orientation file |
| A piece of work just finished | Harvest from what was observed |
| An entry is agent-written and a reader must judge it | Trust is recorded, never assumed |
| Tempted to write down a confidence, priority, or importance | Signals, not scores |
| About to write down something you learned | One fact, one home |
| The base has grown past a comfortable read | Route, never pile |
| Tempted to write down everything learned | What never enters |
| Something will be loaded into every session | The always-loaded budget |
| Deciding what a reader must tolerate versus what an author must supply | Read permissively, degrade rather than fail |
| An entry could go out of date and nobody would notice | Freshness is a date, not a feeling |
| A migration, rename, or rewrite just landed | Migrations are when the layer starts lying |
| An entry is now wrong or obsolete | Retire out loud |

## Two layers, one question each

Knowledge splits into two layers, and putting a fact in the wrong one is the
most common way a base becomes unreadable.

- **Craft knowledge** answers *how is this done well anywhere* — how to write
  a test, structure a module, judge a decision. It is written once, holds in
  every repo, and does not change when this project changes.
- **Project knowledge** answers *what is true here* — where the entry points
  are, which conventions the reviewers actually enforce, which trap cost
  someone a day last month, why a boundary sits where it sits.

**The transplant test** decides which layer an entry belongs to: imagine the
entry pasted into an unrelated repo. If it still reads as sound advice, it is
craft knowledge and does not belong in the project base — it will be
generic, unactionable, and it will crowd out the entries that only this
project can supply. If it becomes false or meaningless, it is project
knowledge, and this is its home.

```markdown
Bad — true everywhere, so it teaches nothing about this project:
  "Validate user input before persisting it."

Good — false anywhere else, so only this base can carry it:
  "Input reaching the store is already validated at the boundary; the
   store's own guards exist for the migration path, and a second
   validation there is what reviewers ask you to remove."
```

## The orientation file

The base needs exactly one file that answers "where am I?" for a reader with
no starting point: what the project does in a paragraph, the entry points,
a table of major components and where they live, the conventions that hold
across the codebase, the commands that build and test, and the dependencies
that matter. It is a map, not an audit — a reader should finish it knowing
which file to open next, not knowing everything.

Two rules keep it from becoming a liability:

- **Derive it, do not maintain it.** Write it by reading the tree, and
  rewrite it the same way after any structural change. An orientation file
  patched by hand, line by line, drifts into a composite of several past
  layouts — each line true once, the whole no longer describing anything.
- **A wrong fact is worse than a missing one.** A reader who finds a gap
  goes and looks; a reader who finds a confident wrong path follows it. When
  you are unsure, write the gap.

Say in the file itself what it is a map *of* and how to re-derive it. An
orientation file that does not state its own basis gives a reader no way to
tell whether it still holds.

## Harvest from what was observed

Knowledge comes from finished work, and the harvest happens while the
evidence is still on disk. Rank the sources by who actually observed them:

- **What an independent reader or a command saw** — review findings,
  verification output, a failing run. Strongest.
- **What a participant believed** — working notes, progress reports, a
  summary written by whoever did the work. Second: honest, and still
  self-assessed.
- **What was intended before contact** — the plan, the design doc, the
  estimate. Weakest. A claim that appears only in the plan is not a finding;
  it is an intention that was never tested against the work.

**Mine the project, not the change.** The question is never "what did this
piece of work do" — that is what the history is for — but "what will still be
true for the next piece of work." A convention a reviewer enforced twice, a
constraint the work surfaced, a trap that cost real time: those are entries.
The specific files, ids, and line numbers are the *evidence* for an entry,
never the entry itself.

```markdown
Bad — an entry that cannot outlive its own change:
  "In the retry refactor we moved the cap check into the loop body."

Good — the durable rule, with the change as its evidence:
  "The retry cap is enforced inside the loop, not by the caller. A caller-side
   cap was tried and reverted: the loop is reachable from three entry points
   and only one of them went through that caller."
```

## Trust is recorded, never assumed

Once a knowledge layer is written by agents rather than only by people, the
question "who says so?" stops being answerable by looking at the file. Two
facts must be recorded separately, because they are separate:

- **Who produced the content, and when.** An agent and a model version, a
  script, or a person — plus the timestamp of the last meaningful change,
  which is what lets a reader tell a recent edit from an old fact.
- **Who confirmed it, and when.** Confirmation is a different event with a
  different actor, and it can happen many times: a person signs off, a
  nightly job re-checks. Content can change without re-confirmation, and a
  fact can be re-confirmed without being rewritten — so one field cannot
  carry both.

From those two, a reader derives the only distinction that matters at read
time: **nobody confirmed this** / **a machine confirmed it** / **a person
confirmed it**. Derive the tier, never store it — a stored tier is a claim
that ages the moment either field changes.

**Absence is information, not an error.** An entry with no confirmation
recorded is not invalid; it is *unconfirmed*, which is exactly what a
reader needs to know before acting on it. What it must never be is
indistinguishable from a confirmed one.

```yaml
# Good — two events, two actors, two times. The tier is derivable.
generated: { by: agent/model-x, at: 2026-06-20T22:53:05Z }
verified:  { by: human:alex,    at: 2026-06-25T09:00:00Z }

# Bad — one field, and it silently answers neither question.
last_updated: 2026-06-25
```

## Signals, not scores

The pull is always toward writing down a judgment: a confidence number, a
priority, an importance flag. Resist it. A stored judgment is subjective
(the next author scores differently), unportable (it means nothing to a
reader with different needs), and it goes stale silently — nothing
recomputes it when the world moves.

Record the **objective signals** the judgment was made from — who authored
the source, how often it is actually exercised and over what window, when
the source itself last changed — and let each reader infer what they need
at read time, against their own question.

**A flag that most entries carry has stopped being a flag.** This is how
the failure shows up in practice: a judgment field gets applied generously
because each individual application is defensible, and the field ends up
selecting nothing. Watch the ratio; when the marked share climbs, the field
has become a formality and the ranking a reader actually needs must be
derived from signals instead.

## One fact, one home

Before writing anything, read what the base already holds for the area you
touched. Reversing that order is how a base fills with near-duplicates, and
a duplicate is not free: each copy is separately plausible, separately
maintained, and when the fact changes, someone must find every copy or leave
a lie behind.

When an existing entry is merely incomplete, **extend that entry**. Filing a
neighbor beside it splits the topic in two, and the next reader finds
whichever half the index happens to show first — usually not both. The same
rule holds for a fact that contradicts an existing entry: replace the line,
never leave both standing for a reader to arbitrate.

## Route, never pile

A base is read by someone with a question, never front to back. Past a
handful of entries, the index stops being a list and becomes the primary
interface: one row per entry, with the **situation that should send a reader
to it** — not a summary of its contents.

```markdown
Bad — a table of contents; the reader must open each one to find out.
  - caching.md — about caching
  - locks.md — about locks

Good — a router; the reader's situation matches a row and they open one file.
  - caching.md — read when adding a cached read path, or a stale value is
    suspected in a bug report
  - locks.md — read when a code path takes more than one lock, or a deadlock
    is being diagnosed
```

When a topic accumulates enough entries to deserve its own routing, give it
a directory and a topic file carrying its own index, and let the parent index
point at the topic rather than its contents. The shape is fractal: a reader
should always be one routing decision away from the right file, at every
depth, and should never scan a flat list of everything.

## What never enters

- **Anything the code already states plainly.** A base that restates
  signatures and file names competes with the source and loses; the source
  is regenerated by every edit, the restatement is not.
- **Craft advice.** It fails the transplant test above and belongs to the
  other layer.
- **Unverified claims.** An entry that no one observed — inferred from a
  design doc, assumed from a name, remembered without a source — reads
  exactly like an observed one once it is in the file. Write what was seen,
  and name what it rests on.
- **Secrets, credentials, and personal data.** No redaction step downstream
  can be relied on to catch what was written here first.

## The always-loaded budget

Some part of the base is usually loaded unconditionally — a preamble, a
digest, a "critical" tier every reader gets. That tier is paid for by every
single reader, every single time, whether or not it turns out to be relevant.
Treat its size as a hard budget rather than a preference.

**A tier that admits most candidates has stopped classifying.** If nearly
everything is marked critical, the mark no longer separates anything, and its
only remaining effect is the cost. The test is not "is this entry valuable?"
— entries in a knowledge base are valuable by construction — but "is this
worth *every* reader paying for it, on the reads where it does not apply?"
Almost always the honest answer is no, and the entry belongs behind a
routing trigger where the readers who need it will find it.

Prefer an executable check to a prose warning wherever the rule can be
mechanized. A rule that a linter, a guard, or a test enforces costs nothing
to read and cannot be skimmed past; prose is the fallback for judgment,
taste, and intent, which is a much smaller set than it first appears.

## Read permissively, degrade rather than fail

A knowledge layer is consumed under pressure, often by something that
cannot ask a follow-up question. Whatever validates it on the way in, the
read path must never refuse an entry for a missing optional field, an
unfamiliar category, an extra key it does not recognize, or a link that no
longer resolves. An entry carrying only its subject is still worth reading;
a reader that rejects it has converted a partial answer into no answer.

The rule cuts the other way on write, and the asymmetry is the point:
**strict where the layer is authored, forgiving where it is read.** Enforce
shape with a check that runs against the layer, so authors learn
immediately; keep the consumer tolerant, so one malformed entry never
darkens the rest.

## Freshness is a date, not a feeling

Every entry that can expire carries **the date it expires on** — an
absolute date, not a duration and not a vibe. An absolute date makes
staleness a plain comparison that anything can run without knowing when the
entry was written or when it was last read; a relative "review every six
months" needs both, and so nobody ever computes it.

Pair it with a coarse state — *draft* (not reviewed, may be incomplete),
*current*, *deprecated* (kept only for links and history) — and default the
missing state to *current* so silence is never ambiguous.

The payoff is that rot becomes reportable. A layer where every entry
carries an expiry can be swept in one pass and answer "what have we stopped
believing?" — which is the question nobody asks in time, because until it
is a date comparison, asking it means re-reading everything.

## Migrations are when the layer starts lying

Renames, rewrites, and migrations are the moment a knowledge base becomes
actively harmful, and the failure is structurally quiet: the code moved, the
tests moved with it, and nothing whatsoever tests the prose. The orientation
file and any entry naming implementation now describe a system that no longer
exists — and they keep teaching it, confidently, to every reader who arrives
after the change.

Treat the knowledge layer as part of the migration's scope, not as follow-up.
When it genuinely cannot be swept in the same pass, say so *in the affected
files*: mark them historical for the implementation they name, and state what
is known to still hold. An entry openly labeled stale costs a reader one
sentence. An entry silently stale costs them the afternoon.

## Retire out loud

Entries expire. When a fact stops being true, replace it and say what
replaced it; when a practice is abandoned, record that it was abandoned and
why, because the next person will otherwise rediscover the idea and repeat
the experiment. Do not rewrite the record silently — a reader who remembers
the old entry needs to see that it was superseded, not to wonder whether
they misremembered.

The base is measured by what a reader can trust, not by how much it holds.
An entry nobody can act on, nobody has verified, or nobody can date is a
liability wearing the costume of an asset: delete it, and the base gets
stronger.

## Patterns

Reusable patterns live in `knowledge/patterns/` as individual files. Each
line carries its load trigger — read a pattern only when its trigger
applies, never the whole directory.

- [attribution-that-survives-rewrites](knowledge/patterns/attribution-that-survives-rewrites.md)
  — read when an entry cites a source, or references another entry, in a
  layer that gets rewritten by something other than a careful human.
- [rules-that-ship-as-a-check](knowledge/patterns/rules-that-ship-as-a-check.md)
  — read when an entry states a rule a machine could enforce: a sanctioned
  computation, a required shape, a command that decides done.

The frontmatter families this guide describes — provenance, trust,
lifecycle, and the checkable-rule contract — are the shape the Open
Knowledge Format settled on in its v0.2 specification. The rules above are
stated independently of it: adopt the format if you want the interop, but
the discipline is what earns the value.

## Related guides

- [documentation.md](documentation.md) — read when writing a single spec: what
  belongs in it, precision, currency, honest gaps.
- [decisions.md](decisions.md) — read when a choice settles and needs a
  record, or a recorded decision is being contested or superseded.
- [tests.md](tests.md) — read when the rule you are about to write down could
  be enforced by a test instead of remembered by a reader.
