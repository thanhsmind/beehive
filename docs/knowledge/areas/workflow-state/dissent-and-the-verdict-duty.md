---
type: bee.area
title: "Workflow State — the worker's dissent, the orchestrator's obligated verdict, and the two debt doors that make it real"
description: "How a dispatched worker records a structured disagreement with the cell it was handed instead of complying or arguing in prose, why a blocker-severity dissent parks only the related work through the existing blocked-status machinery, why the orchestrator's answer is one of exactly three logged verdicts rather than free text, and why both the close door and the merge door refuse while any dissent is unanswered — the same enforcement shape judge-debt already established."
timestamp: 2026-08-28
bee:
  id: workflow-state-dissent-and-the-verdict-duty
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md, areas/workflow-state/cells-completion-judge-and-archive.md]
  decisions: ["slp-dissent-stop-and-ask 4b7aa303 (dissent has FULL TEETH: a blocker-severity dissent pauses the RELATED part of the work while other parts continue, and the orchestrator is OBLIGATED to answer one of three — accept and log / reject with reasoning / escalate a rung — recorded in the decision log before the related work resumes)", "slp-dissent-stop-and-ask a2affcba (mechanism: a cells-level dissent record {target, claim, alternative, severity} written through the CLI; bee close and bee worktree merge REFUSE while any dissent lacks a recorded verdict, the same enforcement shape as judge-debt; blocker severity rides the existing blocked-status machinery so the related work stays unclaimable; the SLP escalate verb NEVER reuses bee cells escalate, which means model tier)", "slp-dissent-stop-and-ask 787a9eb0 (dissent adds an obligation on the orchestrator, never a relaxation of a gate)", "slp-dissent-stop-and-ask a020319d (this is the SECOND of four slp clusters; blind lanes and contract/original-request are separate features and out of this boundary)", "slp-dissent-stop-and-ask e29918f7 (the merge half of the obligation is NEW code, not a copied arm — merge carried no judge-debt door, only proof debt, so the door had to be built rather than mirrored)", "slp-dissent-stop-and-ask 6a6b9975 (herding-lane dissent is OUT of this boundary: one writer owns the dissent record, and a herding worker is instructed never to run a bee command, so it cannot reach that writer; carried as backlog item p-05d2a4f4)", "slp-followup-gaps 7db30738 (2026-08-29 — the herding half of 6a6b9975 is closed, and closed without a second writer: the dissent rides the herding mailbox result as data and the control loop transcribes it through the same record-dissent function; touches 6a6b9975)"]
  sources: ["docs/history/slp-dissent-stop-and-ask/CONTEXT.md (locked decisions, terms, boundary)", "docs/history/slp-dissent-stop-and-ask/plan.md (the four phases)", "slp-dissent-stop-and-ask cell sd-1 (the dissent record, its closed severity set, the secret scan, the claim release, and the blocker tooth; trace .bee/cells/sd-1.json, commit 156b9566, 2026-08-28)", "slp-dissent-stop-and-ask cell sd-2 (the obligated verdict written to the decision log fail-closed, releasing the cell the blocker dissent parked; trace .bee/cells/sd-2.json, commit d7726926)", "slp-dissent-stop-and-ask cell sd-3 (the cells dispatch table folded into the served-but-undeclared law, proven red-first; trace .bee/cells/sd-3.json, commit 79f05db3)", "slp-dissent-stop-and-ask cell sd-4 (bee close refuses while any dissent lacks a verdict, in every lane; trace .bee/cells/sd-4.json, commit 6966d85c)", "slp-dissent-stop-and-ask cell sd-5 (bee worktree merge refuses WORKTREE_MERGE_DISSENT_DEBT, reading the close door's own two helpers so one deferral clears both doors; trace .bee/cells/sd-5.json, commit 18aa0e09)", "slp-dissent-stop-and-ask cell sd-6 (options[] and leaning on the worker result across three code surfaces, plus the swarming worker contract; trace .bee/cells/sd-6.json, commit ecdb89ea)", "slp-followup-gaps cell sfg-2 (commit 29fd6fbe, 2026-08-29 — the carried dissent on the herding mailbox result, and its transcription through the one record-dissent writer)"]
  authoritative_for: "workflow-state: the dissent record, the obligated verdict, the close and merge dissent-debt doors, and the three boundary signals that oblige a worker to stop and ask"
---

# Workflow State — Dissent and the Verdict Duty

A dispatched worker used to have two ways to disagree with the cell it was handed:
comply anyway, or return prose nobody is obliged to read. Neither is a voice. A
**dissent** is the third way — a structured record with teeth, and an obligation
on the orchestrator that a door enforces.

The rule the whole design follows: **an obligation is only real where a door refuses.**
Dissent does not invent a second enforcement style; it copies the one judge-debt
already proved.

## The dissent record

A worker records a dissent against the cell it holds. The record carries four
fields, and it is a record — never prose inside a report:

| Field | Meaning |
|---|---|
| target | The cell the dissent is against. This is also how "the related part" is resolved. |
| claim | What the worker says is wrong with the cell as handed. |
| alternative | What the worker would do instead. |
| severity | How hard the dissent bites. The set is CLOSED. |

Two severities matter:

- **Blocker** — the dissent pauses the related work and obligates a verdict.
- **Consider** — the dissent is recorded and still owes an answer, but it pauses
  nothing. Other work runs on.

The record is scanned for secrets before it is written, like every other
worker-authored text bee stores. Writing a blocker dissent also releases the
worker's claim on the target: the worker has said its piece and stops, rather
than holding a cell it refuses to build.

**The blocker tooth reuses the existing blocked-status machinery** rather than
inventing a parallel parking mechanism. A blocker dissent puts its target into
the same blocked state the scheduler already refuses to hand out, so the related
work is unclaimable by anyone — the dissenting worker included — until the
verdict lands. Everything not related keeps running: the pause is scoped to the
target, never to the feature.

## The verdict is one of exactly three, and it is logged

The orchestrator is **obligated** to answer. The answer is not free text and not
a mood; the set is closed at three:

- **Accept and log** — the dissent is right; the recorded answer says so.
- **Reject with reasoning** — the cell stands; the reason is recorded, not implied.
- **Escalate a rung** — the question goes to the next authority up.

"Escalate a rung" is authority, not model tier. It is deliberately NOT the
existing escalate verb on a cell, which means model tier and keeps that meaning.
Two verbs, two meanings, no overload.

The verdict is written to the **decision log**, fail-closed: if the log write
does not land, the verdict does not count. That is what makes the answer durable
and citable later instead of a line in a transcript nobody can find. Recording
the verdict is also what releases the cell a blocker dissent parked — the pause
and the release are two ends of one mechanism, so there is no state where the
answer exists and the work is still stuck.

The verdict payload is a **small closed form**, deliberately not the judge-verdict
shape. A judge verdict carries per-check evidence because a judge checks many
things; a dissent verdict answers one question, so it takes one answer and one
reason.

## The two debt doors

An unanswered dissent is **debt**, and debt is what doors are for. Both exits from
a piece of work refuse while any dissent lacks a verdict:

- **`bee close`** refuses, in every lane. There is no tiny-lane exemption: a
  dissent is a person disagreeing, and lane size does not make disagreement
  cheaper to ignore.
- **`bee worktree merge`** refuses with a typed `WORKTREE_MERGE_DISSENT_DEBT`,
  as a zero-mutation precondition — before `git merge` is ever attempted, so
  dissent debt never touches main.

The merge half was **new code, not a copied arm**. The close driver already had
a judge-debt door to imitate; the merge path had only one cell-debt precondition
(proof debt), so there was no dissent-shaped arm sitting there to extend. The
merge door instead **calls the close door's own two helpers**, so both doors read
one implementation of "is this dissent answered?" A single recorded deferral
therefore clears both doors at once, and the two can never drift into
disagreeing about the same dissent — the failure mode a rule checked at two
points always invites.

## Three signals that oblige a worker to stop and ask

A worker's contract now names three moments where guessing is the wrong move and
`[BLOCKED]` with options is the right one:

- a contract or API change,
- trading data quality or user experience for a technical target,
- a new dependency.

The third is not a new rule. Package installs were already not a worker's to
make, so "a new dependency" **extends that existing clause** rather than being
restated beside it — the same rule spelled twice in two places is the drift that
makes a contract unreadable.

The blocked form itself carries the options and the leaning (see
`areas/bee-herding/the-run-verb-and-worker-outcomes.md` for the machine-readable
half of the same shape), and the orchestrator's per-result duty names the verdict
verb: a blocked result carrying a dissent owes its answer before the related work
resumes. The duty sits beside the per-result instruction, where the orchestrator
already reads one result at a time — not in the rescue ladder, and not in both.

## What this is not

- It is not a live question-and-answer channel. A worker cannot ask mid-flight and
  wait for a reply. Native subagents exit when they speak, so a synchronous wait
  would need a transport bee does not have. Dissent is asynchronous by
  construction: the worker records and stops; the orchestrator answers at a door.
- It is not a gate relaxation. Dissent only ever ADDS an obligation on the
  orchestrator. No gate becomes skippable because a worker objected.
- **It is not a second writer for herding-lane workers.** The original boundary
  (6a6b9975) held that dissent could not reach a herding worker at all: one
  writer owns the record, and a herding worker is instructed never to run a bee
  command. That half is now closed, and closed WITHOUT a second writer — see the
  section below.

## A herding-lane worker reaches it too, as data

Decision `6a6b9975` scoped herding-lane dissent out of the original feature: one
writer owns the dissent record, and a herding worker is told never to run a bee
command, so it could not reach that writer. Decision `7db30738`
(slp-followup-gaps, cell sfg-2, 2026-08-29) moved that boundary without moving
the writer.

A herding worker's dissent now travels as **data on the mailbox result file it
already writes** — the same carrier stop-and-ask uses — carrying the three
fields the record needs: claim, alternative, severity. The control loop reads it
back and writes it through the SAME function `bee cells dissent` routes to, so
the record shape, the closed severity set, the secret scan, the blocker tooth
and the claim release still have exactly one implementation. Nothing about the
record, the verdict duty, or the two debt doors changes: a transcribed dissent
is an ordinary dissent and owes its verdict like any other. The mechanism — the
three surfaces, the lenient parse, the severity left for the writer to check,
and how a transcription that fails is reported rather than swallowed — is in
`areas/bee-herding/the-run-verb-and-worker-outcomes.md`.

## Open Gaps

- ~~**Herding-lane dissent has no carrier.**~~ Closed 2026-08-29 by decision
  `7db30738` (cell sfg-2), along exactly the line the gap named: the dissent
  fields ride the herding mailbox result and the control loop transcribes them
  through the same one verb, so one record type still keeps one writer. Backlog
  item `p-05d2a4f4` is done.
- **Consider-grade dissent has no dedicated reader.** It is recorded and it owes a
  verdict like any other, but nothing surfaces it separately from blocker dissent.

## Pointers (implementation)

- Verbs: `bee cells dissent` (record) and `bee cells dissent-verdict` (answer).
  The claim is spelled `--reason`, not `--claim`: `claim` is a CLI-wide
  flag-alone boolean, so `--claim <text>` would swallow its own value token and
  the whole argument list would decline. The stored field is still `claim`.
- Rust: `packages/bee-rs/crates/bee/src/verbs/cells/dissent.rs`;
  the shared blocked-status write lifted into `apply_block_mutation` in
  `verbs/cells/util.rs` so the block path has one implementation, not two;
  the close door's dissent helpers in `verbs/drivers/close.rs`, read by
  `verbs/worktree/phases.rs` for the merge door.
- Both sub-verbs are declared in the cells dispatch table, which the
  served-but-undeclared law now sweeps — a served verb missing its registry row
  fails a contract test rather than shipping invisible.
