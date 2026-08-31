---
type: bee.area
title: "Human Mailbox — the letter an unattended run leaves behind"
description: "One plain-language letter per run, appended entry by entry as the run works and composed when it ends: what a letter is, when one is filed and when none is, the authorship ban that keeps it truthful, the departure contract that makes its most-read section trustworthy, the single command a consuming inbox may call, and how a run that died in the night still gets its letter."
timestamp: 2026-08-26
bee:
  id: human-mailbox-overview
  lifecycle: active
  areas: [human-mailbox]
  required_context: []
  decisions: ["LR1 5dbdb0e2 (the letter gains a Mistakes & reflection section, after Broken-or-unfinished, dropped when empty — amends D7's section set)", "LR2 b8291876 (its only source is the reflection entry kind, appended by the agent through bee mailbox reflect; the composing pass never authors one)", "LR3 ba9f06a4 (a reflection entry has two required parts: what went wrong, and what would have been better)", "LR4 bb73e821 (lesson mining sources unchanged; reflection entries are not mined)", "D1 303488de (bee owns the mailbox data and the code that emits it, and nothing above it)", "D2 e3618e3b (a readable subject is a validity rule, not a formatting preference)", "D3 1b079912 (one markdown file per letter with typed frontmatter; no JSON twin, no index stream)", "D4 1d56c1d2 (append at every clean stop; compose when the run ends)", "D5 e9cb4c15 (a departure has three required parts and a closed kind set)", "D6 2009bc71 (read state is a field bee owns; the inbox flips it by calling a bee command)", "D7 b94381e5 (five nightly sections, empty ones dropped; architecture/behaviour/usage only in the close letter)", "D8 1c7a9d87 (sentences written at the moment of the event; the composing pass may never author a fact)", "D9 d970d6fc (every session appends; only an unattended run files)", "D10 1fb69f4b (the explicit no-departure statement is enforced only while armed)", "D11 349f25d8 (one letter maps to one run, never one night)", "D12 05b5f964 (a run that died gets its letter from the next session; no scheduler)", "D13 c3ece144 (each needs-your-call item carries a stable id and names what it blocks)", "D14 a6475e2c (the feature-close letter is in scope; a weekly digest is not — superseded in effect by letter-digest LD1/LD3)", "D17 1660158a (bee and the consuming project are separate; bee never writes another project's tree)", "LD1 b610a1dc (the mailbox stays files; daily and weekly digests are files beside the letters)", "LD2 aedb5be9 (every bee close files its close letter at the moment of close, attended included)", "LD3 dbbe0778 (the next session composes a finished period's missing digest; no scheduler)", "LD4 b343870b (the weekly fold auto-logs repeat error shapes as decisions tagged lesson)", "RBL D1 db562f26 (a run may not end its work without an explicit mistakes answer; the close door refuses without one)", "RBL D2 a240362a (the explicit clean-run answer is its own stored entry kind, never rendered as a mistake and never mined)", "RBL D3 0872f328 (lesson mining reads reflection entries as a trouble source, by their what alone; the clean-run answer is never mined — supersedes LR4 bb73e821)", "RBL D5 20fe96d3 (the answer is recorded per cell and the close door walks the feature's capped cells, never the closing session's run)", "RBL D6 7760339d (the feedback digest reports the clean-run-to-reflection ratio; a witness, never a guard)"]
  sources: ["docs/history/human-mailbox/CONTEXT.md (the locked decisions)", "docs/history/human-mailbox/plan.md (four phases, strictly ordered)", "docs/discovery/human-mailbox/MAP.md (the discovery map and its 14 tickets)", "cells hm-1..hm-10 (four slices, merged 2026-08-25/26)"]
  authoritative_for: "human-mailbox: the letter record, when it is filed, and the one command a consumer may call"
  owns.code: [packages/bee-rs/crates/bee/src/verbs/mailbox.rs]
---

# Human Mailbox — the letter an unattended run leaves behind

## Purpose

A long unattended run used to leave the person who started it nothing readable.
Reconstructing the night meant replaying a transcript. This area is the record
that replaces that: **one letter per run**, written in the run's own words as it
works, so that eight hours of unwatched activity can be understood in two
minutes of reading.

The letter's value rests entirely on being **trustworthy rather than
comprehensive**. A summary that reads well but quietly invents a reassuring
detail is worse than no letter, because it will be believed. Everything below
follows from that.

## Entry Points & Triggers

- **A clean stop** — a unit of work is capped, a feature is closed, or a blocker
  is hit. Each appends its own entry at the moment it happens.
- **A feature close** — first demands the mistakes answer: it refuses while
  any capped cell of the feature has neither a reflection nor the clean-run
  statement (RBL D1, D5). Past that door it files the run's letter
  immediately, attended or not (LD2); the run end later re-composes that same
  letter in place.
- **The end of a run** — an armed run composes its entries into one letter and
  files it.
- **The start of a session** — a run that went silent without ever reaching its
  end gets its letter here, from the next session that starts; the same moment
  composes the digest of every finished day or week that has letters and no
  digest yet (LD3), and the weekly fold logs its mined lessons (LD4).
- **A consuming inbox** — flips a filed letter's read state by calling one
  command, and never by writing the file.

## Data Dictionary

| Term | Meaning |
|---|---|
| **letter** | One filed record covering exactly one run: a single markdown file whose typed frontmatter is the machine contract and whose body is human prose. There is no second artifact — no JSON twin, no separate index stream — because one artifact cannot drift against itself. |
| **entry** | One raw append written at a clean stop, before any letter exists. Entries are the only source a letter may draw on. |
| **run** | A session's span — not a night, and not a dispatched job. One night may hold many runs, and each earns its own letter. |
| **armed** | The mailbox is on for this run, so it will file a letter and the explicit-departure rule is enforced. Arming needs both the checkout's own configuration and the owner's enable marker: the configuration alone only says a checkout *can* run unattended. |
| **departure** | A recorded difference between what the plan said and what was done, in three required parts — what was done differently, why, and which kind — with kind drawn from a closed set of four. |
| **plan-followed statement** | The explicit declaration that a unit followed its plan. Recorded separately from departures, so that a statement meaning *nothing happened* can never be mined as though it were a lesson. |
| **reflection** | An entry kind the agent appends the moment it notices a mistake, or at the run-end look-back, via `bee mailbox reflect --wrong <what went wrong> --better <what would have been better>` (LR2). Both parts are required; a missing part refuses the append (LR3). It renders as the letter's "Mistakes & reflection" section, between Broken-or-unfinished and Needs-your-call, dropped when empty (LR1); a reflection never appears in Done and never becomes the subject — a mistake is not a thing the run did. Lesson mining reads it as a trouble source, by its `what` alone (RBL D3 `0872f328`, superseding LR4). |
| **unfinished letter** | A letter filed by a later session for a run that went silent, marked plainly as such and naming the moment the run last recorded anything. |
| **read state** | Whether the human has read a letter. It lives inside the letter file, and bee is its only writer. |
| **digest** | One markdown file folding one finished period — a UTC day (`digest-YYYY-MM-DD.md`) or an ISO week (`digest-YYYY-Www.md`) — from that period's letters and stored usage records, filed beside the letters with frontmatter `type: digest`. A renderer, never a summarizer: it groups and transcribes, it computes nothing (LD1, LD3). |
| **lesson** | A decision tagged `lesson`, auto-logged by the weekly fold when the same normalized error shape appears in letters of two or more distinct runs. It cites the letters and carries a stable `shape:<sha-12>` token; a token already logged — active or superseded — is never re-logged, so a retired lesson stays retired (LD4). |

## Behaviors & Operations

**Appending at a clean stop.** Trigger: a unit of work caps, a feature closes,
or a blocker is hit. One entry is appended the moment it happens, carrying the
plain-language sentence for that event, the files touched, the proof, and any
departure. What the reader eventually observes: a letter that is complete up to
the moment the run stopped, even if it stopped abruptly. What they never
observe: a run that died mid-flight leaving nothing, which is what end-only
composition would produce.

**Composing and filing at the end of a run.** Trigger: an armed run reaches its
end. The stored entries are composed into one letter and filed. The composing
pass may reorder, group and drop — and may never state a fact no stored entry
carries. What the reader observes: six sections (D7 as amended by LR1), each present only if it has
something to report. What they never observe: an empty section printed for
completeness, or a sentence the run did not actually record.

**Recording without filing.** Trigger: an attended session works normally. Its
entries are appended exactly as an unattended run's are, and no letter is filed.
This is what lets a session that began attended and became an overnight run keep
a complete record of its whole span.

**Flipping read state.** Trigger: a consuming inbox marks a letter read or
unread. It calls the one command bee exposes; the flip changes that field and
nothing else, a repeat is a no-op rather than an error, and a letter that does
not exist is refused by name. What the consumer never does: write into bee's
store itself.

**Recovering a run that went silent.** Trigger: a session starts while a
previous run's entries exist with no letter. Detection reads directory names
only — no file is opened to decide — and calls a run silent only on positive
evidence that its session ended or its heartbeat went stale. An unfinished
letter is then filed, marked plainly, naming the moment the run last recorded
anything. What never happens: a live run being swept as dead, or a background
scheduler being introduced to watch for this.

## Actors & Access

| Actor | What they do |
|---|---|
| The person who started the run | Reads the letter. Never edits it, and never needs tooling to open it. |
| The acting side (the assistant) | Writes entries as it works; composes and files at the end. It is the only writer of this store. |
| A consuming inbox in another project | Reads letters and calls the one command to flip read state. It never writes the files, and bee never writes into its tree. |

## Business Rules

1. **The composing pass is a renderer, not a summarizer.** It may reorder, group
   and drop; it may never state a fact no stored entry carries (D8). This is the
   rule the whole area exists to protect, and it is enforced by a test that
   walks every word of a letter body against the words its entries carry.
2. **The sentences are written at the moment of the event** (D8). Composition
   assembles sentences; it does not author them.
3. **A subject is a validity rule.** One plain sentence, in the human's own
   vocabulary, answering *what happened* on its own. A record without a readable
   subject is not valid (D2).
4. **One letter maps to one run, never one night** (D11). Folding a night would
   hide the fact that one run died.
5. **Every session appends; only an armed run files the run-end letter of a
   letterless run** (D9, narrowed by LD2): a feature close files immediately,
   armed or not, and an existing letter re-composes at run end whatever the
   arming says — arming still gates only the first filing of a run that
   never closed a feature.
6. **A departure carries three parts and a kind from a closed set of four**
   (D5): hit an unforeseen obstacle, found a better route, the plan was wrong
   about a fact, or something else had to be fixed first. A unit that followed
   its plan says so explicitly — silence and nothing-happened must not read
   alike.
7. **That requirement is enforced only while armed** (D10), so a run that files
   no letter keeps the behaviour its callers already relied on.
8. **The plan-followed statement is recorded apart from departures**, so a line
   meaning *nothing happened* cannot teach the pattern miner a lesson out of
   silence.
9. **Read state lives in the letter and bee is its only writer** (D6).
10. **A dead run's letter comes from the next session, never from a scheduler**
    (D12) — a scheduler shares the failure mode it would exist to cover.
11. **Detection of a silent run is bounded and fails closed.** Directory names
    only, no file opened to decide, and silence declared only on positive
    evidence. Absent or unreadable evidence means *not silent*.
12. **bee owns the data and nothing above it** (D1, D17). No listing, no
    rendering, no viewer ships from here, and nothing is ever written into
    another project's tree.
13. **Digests are files beside the letters, composed by the next session**
    (LD1, LD3) — no email, no scheduler, idempotent by file existence, and
    letter-only surfaces never see a `digest-*` name, so a digest can never
    enter the lettered set or be folded as a letter.
14. **A close-lettered run that later goes silent is still recovered** — a
    lettered run whose entries file is newer than its letter is a D12
    candidate (two stats, no opens), and recovery re-composes that one
    letter in place.
15. **Lesson mining reads only trouble** (LD4, widened by RBL D3): the
    broken-or-unfinished bullets, the obstacle / plan-was-wrong / fix-first
    departure kinds, and the run's own reflections — never better-route
    departures, never plan-followed statements, never the clean-run answer. A
    mined decision carries `source: "agent"` and only what its cited letters
    say (D8). A digest or lesson failure never refuses the work that
    triggered it.
16. **A reflection is mined by its `what`, never by its rendered bullet**
    (RBL D3). The bullet joins the two stored parts as
    `<what> — better: <better>`; tokenizing that join would make the same
    mistake with a differently worded counterfactual fail to match. The item
    carries its parts apart, so the miner reads the part that recurs and
    leaves the run's own idea of the fix out of the shape.
17. **The clean-run answer is excluded structurally, never by its wording**
    (RBL D2/D3). It is the one mistakes-answer entry that carries no
    `better`, and that absence is what keeps it out of the miner — the same
    shape test the letter composer uses, so a re-worded sentence cannot slip
    past a string match that never existed.
18. **The collapse of the answer is measured, not assumed** (RBL D6):
    `bee feedback digest` counts the caps that answered clean against the
    caps that recorded a real reflection, from the cell traces the cap
    already writes, and reports both numbers and their ratio in its object
    and on its printed line. It is a WITNESS, never a guard — it refuses
    nothing, drops nothing, and scores nothing.
19. **A feature close demands the mistakes answer, and asks the CELLS, not
    the run** (RBL D1, D5). The close refuses while any capped cell of the
    feature carries neither a reflection nor the clean-run answer, and names
    those cells. It reads the feature's capped cell traces, never the closing
    session's own run entries: the session that closes a feature is routinely
    a different run from the workers that built it, so a run-scoped door
    would refuse every close and leave the clean-run answer as the only
    reachable reply — the exact silence the rule exists to end. The door is
    NOT armed-gated, because a feature close files its letter
    unconditionally (LD2), and it runs with the other blocking doors, before
    any write, so a refused close appends no entry and files no letter.
20. **The cap is the one writer, and it fills two sinks** (RBL D5). One
    reading of the reported values lands both the cell trace the door reads
    and the mailbox entry the letter renders, so the two can never disagree
    about what a run answered. The cap itself adds no refusal of its own.

## Edge Cases Settled

- **A run with no entries** produces no invalid letter.
- **A section with nothing to report is absent**, never printed empty (D7).
- **Two runs in one night file two letters** (D11).
- **An already-filed run is not filed twice**; a run reaching its end again
  re-composes its existing letter in place, preserving the filename and the
  human's read state.
- **A run killed mid-append** can leave a torn last line; reading tolerates it
  rather than losing the whole run's record.
- **An unfinished letter states only that the run went silent and when.** It
  does not say the run crashed, failed or died — those are facts no entry
  carries.
- **Architecture, behaviour and usage appear only in the feature-close letter**
  (D7, D14), and drop like any other section when the entries carry nothing for
  them.

## Open Gaps

- **The `Next` section can never print.** D7 names five nightly sections, but no
  entry field carries a next step, and composing one would breach the authorship
  ban. It is correctly dropped today, which means a locked decision promises a
  section that cannot appear. Settling it means either giving an entry a
  next-step field or amending D7 — both change something locked, so it is the
  owner's call and is filed rather than decided.
- **"Usage" is a chosen mapping.** It is sourced from the skills and specs a
  feature's units declare they change. That is stored and factual, but nothing
  in the store literally says *here is how to use this*, so the mapping is a
  judgement rather than a transcription.

## Pointers (implementation)

- The store, the record and every composing rule: `packages/bee-rs/crates/bee/src/verbs/mailbox.rs`.
- The digest composer, period detection and lesson miner: `packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs`; the work-set hook: `packages/bee-rs/crates/bee/src/verbs/work.rs`; the agent-sourced decision append: `packages/bee-rs/crates/bee/src/verbs/cells/audit.rs` (`log_decision_from`).
- The cap's append and the departure door: `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs`.
- The run-end hook and silent-run recovery: `packages/bee-rs/crates/bee/src/verbs/work.rs`.
- The feature-close letter: `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs`.
- The two exposed commands: `bee mailbox mark --id <letter> --status read|unread`
  and `bee mailbox reflect --wrong <text> --better <text> [--session-id <run>]`
  (letter-reflection, 2026-08-30).
- Letters and entries live under `.bee/human-mailbox/`, git-ignored as runtime state.
- The locked decisions: `docs/history/human-mailbox/CONTEXT.md`. The plan and its four phases: `docs/history/human-mailbox/plan.md`.
