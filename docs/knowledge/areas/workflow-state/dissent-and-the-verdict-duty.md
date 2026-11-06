---
type: bee.area
title: "Workflow State — the worker's dissent, the orchestrator's obligated verdict, and the two debt doors that make it real"
description: "How a dispatched worker records a structured disagreement with the cell it was handed instead of complying or arguing in prose, why a blocker-severity dissent parks only the related work through the existing blocked-status machinery, why the orchestrator's answer is one of exactly three logged verdicts rather than free text, and why both the close door and the merge door refuse while any dissent is unanswered — the same enforcement shape judge-debt already established."
timestamp: 2026-11-04
bee:
  id: workflow-state-dissent-and-the-verdict-duty
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md, areas/workflow-state/cells-completion-judge-and-archive.md]
  decisions: ["slp-dissent-stop-and-ask 4b7aa303 (dissent has FULL TEETH: a blocker-severity dissent pauses the RELATED part of the work while other parts continue, and the orchestrator is OBLIGATED to answer one of three — accept and log / reject with reasoning / escalate a rung — recorded in the decision log before the related work resumes)", "slp-role-gaps b2baf5e0 (dissent once per point: after a verdict on a cell's dissent, a further bee cells dissent on that cell is REFUSED unless it carries --new-evidence <text>; every dissent still appends and nothing is overwritten)", "slp-dissent-stop-and-ask a2affcba (mechanism: a cells-level dissent record {target, claim, alternative, severity} written through the CLI; bee close and bee worktree merge REFUSE while any dissent lacks a recorded verdict, the same enforcement shape as judge-debt; blocker severity rides the existing blocked-status machinery so the related work stays unclaimable; the SLP escalate verb NEVER reuses bee cells escalate, which means model tier)", "slp-dissent-stop-and-ask 787a9eb0 (dissent adds an obligation on the orchestrator, never a relaxation of a gate)", "slp-dissent-stop-and-ask a020319d (this is the SECOND of four slp clusters; blind lanes and contract/original-request are separate features and out of this boundary)", "slp-dissent-stop-and-ask e29918f7 (the merge half of the obligation is NEW code, not a copied arm — merge carried no judge-debt door, only proof debt, so the door had to be built rather than mirrored)", "slp-dissent-stop-and-ask 6a6b9975 (herding-lane dissent is OUT of this boundary: one writer owns the dissent record, and a herding worker is instructed never to run a bee command, so it cannot reach that writer; carried as backlog item p-05d2a4f4)", "slp-followup-gaps 7db30738 (2026-08-29 — the herding half of 6a6b9975 is closed, and closed without a second writer: the dissent rides the herding mailbox result as data and the control loop transcribes it through the same record-dissent function; touches 6a6b9975)", "slp-article-gaps D1 (the brief names goal, hard constraints and open choices apart; must_haves.open_choices)", "slp-article-gaps D2 (dissent is a right used on evidence, never a duty)", "slp-article-gaps D3 (bee report outcomes: per-layer counts of results changed, not recorded never 0, no advice)", "agent-anti-patterns D1 (a cell action locks the contract, not the implementation; a simpler route is a found-a-better-route departure)", "agent-anti-patterns D3 (the plan departure row of bee report outcomes)", "cell-question-park D1 (a worker parks only its own cell on a product question; dependents wait, siblings stay schedulable)", "cell-question-park D2 (one answer verb logs the answer as a decision cited by the cell and reopens it with the answer in its brief)", "cell-question-park D3 (the leader sets the question waiting-on mark only when no ready cell remains)", "cell-question-park D4 (bee close and bee worktree merge refuse a feature with a parked cell)"]
  sources: ["docs/history/slp-dissent-stop-and-ask/CONTEXT.md (locked decisions, terms, boundary)", "docs/history/slp-role-gaps/CONTEXT.md (D3, store b2baf5e0)", "docs/history/slp-role-gaps/plan.md (cell rig-1)", "slp-role-gaps cell rig-1 (the evidence-taking sibling writer, the after-verdict refusal, and the --new-evidence flag; packages/bee-rs/crates/bee/src/verbs/cells/dissent.rs)", "docs/history/slp-dissent-stop-and-ask/plan.md (the four phases)", "slp-dissent-stop-and-ask cell sd-1 (the dissent record, its closed severity set, the secret scan, the claim release, and the blocker tooth; trace .bee/cells/archive/slp-dissent-stop-and-ask/sd-1.json, commit 156b9566, 2026-08-28)", "slp-dissent-stop-and-ask cell sd-2 (the obligated verdict written to the decision log fail-closed, releasing the cell the blocker dissent parked; trace .bee/cells/archive/slp-dissent-stop-and-ask/sd-2.json, commit d7726926)", "slp-dissent-stop-and-ask cell sd-3 (the cells dispatch table folded into the served-but-undeclared law, proven red-first; trace .bee/cells/archive/slp-dissent-stop-and-ask/sd-3.json, commit 79f05db3)", "slp-dissent-stop-and-ask cell sd-4 (bee close refuses while any dissent lacks a verdict, in every lane; trace .bee/cells/archive/slp-dissent-stop-and-ask/sd-4.json, commit 6966d85c)", "slp-dissent-stop-and-ask cell sd-5 (bee worktree merge refuses WORKTREE_MERGE_DISSENT_DEBT, reading the close door's own two helpers so one deferral clears both doors; trace .bee/cells/archive/slp-dissent-stop-and-ask/sd-5.json, commit 18aa0e09)", "slp-dissent-stop-and-ask cell sd-6 (options[] and leaning on the worker result across three code surfaces, plus the swarming worker contract; trace .bee/cells/archive/slp-dissent-stop-and-ask/sd-6.json, commit ecdb89ea)", "slp-followup-gaps cell sfg-2 (commit 29fd6fbe, 2026-08-29 — the carried dissent on the herding mailbox result, and its transcription through the one record-dissent writer)"]
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
and citable from then on, instead of a line in a transcript nobody can find. Recording
the verdict is also what releases the cell a blocker dissent parked — the pause
and the release are two ends of one mechanism, so there is no state where the
answer exists and the work is still stuck.

The verdict payload is a **small closed form**, deliberately not the judge-verdict
shape. A judge verdict carries per-check evidence because a judge checks many
things; a dissent verdict answers one question, so it takes one answer and one
reason.

## Dissent once per point

A point is dissented once. Once a verdict is recorded on a cell's dissent, the
ordinary `bee cells dissent` on that same cell is **refused** — the ruling has
been made, and following it is the worker's part of the bargain. The refusal
quotes the newest answered verdict and its reason, so the reader sees exactly
which ruling they are breaking, and it names the one fix that reopens the
question:

    bee cells dissent --id <cell> --reason "<claim>" --alternative "<plan>" \
      --severity consider --new-evidence "<what is new>"

`--new-evidence` is the difference measured against the ruling: what the verdict
did not cover, or what has changed since it was made. It is free text, it takes
the same ONE secret scan as the claim and the alternative, and it is stored on
the new entry as `new_evidence`. With it the dissent appends as always; without
it nothing at all is written, and the cell file is byte-identical after the
refusal.

The rule is **one dissent per ANSWERED point**, never one per cell. A first
dissent on a cell with no verdict records as it always did, and several
dissents may stand unanswered side by side — each of those still owes its own
verdict, exactly as the oldest-unanswered rule above says.

This is the piggery rule the feature copied: question the premise once, with
evidence, then follow the ruling. Two consequences are deliberate. The refusal
reads the cell *before* the store lock is taken, so a refused repeat dissent
never disturbs the file or the claim. And the record stays append-only: the
earlier entries, the verdict among them, are never overwritten by the new one.

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

## Open choices: what a worker may question

A right to dissent is only usable when the worker can see which parts of the
brief are open. slp-article-gaps D1 splits the brief into three parts, and the
cell prompt names them apart right after the inlined cell:

- **Goal** — `must_haves.truths`: what must be true at the finish.
- **Hard constraints** — `must_haves.prohibitions` plus the locked D-IDs: a
  conflict is a locked-decision `[BLOCKED]`, never a workaround.
- **Open choices** — `must_haves.open_choices`, optional: design choices the
  plan made that the user did not ask for. A worker may question one with
  evidence through `bee cells dissent`; without evidence it follows the plan.
  The goal-check never reads an open choice as a truth.

slp-article-gaps D2 holds the other side: no prompt, skill or template orders a
worker to look for faults, doubt the lead, or dissent a set number of times.
Dissent is a right used on evidence, never a duty — a duty pays tokens for
argument.

## Measuring whether a layer changes results

`bee report outcomes [--feature <slug>] [--json]` (slp-article-gaps D3) reads
the cell ledger — active and archived cells in the main store, also from inside
a granted worktree — and prints one row per layer: worker reaction, dissent,
semantic judge, leader check, rework (judge-driven reopen or `bee cells
reopen`), role reroute, and plan departure (agent-anti-patterns D3: cells with
recorded departures and the count per departure kind, plus `followed the plan`
and `other` for older free-form lines). Each row gives the cells carrying a record and how many
records changed a result. A layer with no record prints `not recorded`, never 0.
The report gives data only; it never advises adding or removing a layer — that
call stays with the human.

## A parked question: one cell waits, the lane keeps working

Dissent questions the plan. A **parked cell** asks the human a product question
the worker cannot answer (cell-question-park D1-D4). Before this, one open
question ended the leader's turn and stalled the whole lane.

- **Park (D1).** `bee cells park --id <cell> --question "<text>"` sets the cell to
  status `parked` and appends `{question, parked_at}` to the append-only
  `trace.park` array. It releases the claim and every file reservation the cell
  holds, and it adds no attempts row: a question is not a failed try. The cell
  leaves the ready set, its dependents wait, and every sibling stays
  schedulable. The worker returns the token `[PARKED]`.
- **Why a new status, not `blocked`.** `blocked` already means "cannot proceed",
  with its own release paths (a swept claim, `bee cells block`, a blocker
  dissent). A parked cell has exactly one exit: the answer. So `bee cells
  reopen` refuses a parked cell, and every refusal names the remedy
  `bee cells answer --id <cell> --answer "<text>"`.
- **Answer (D2).** `bee cells answer` logs the answer as a decision first (tags
  `cells` and `slp`), then cites it on the cell, stamps the park entry and sets
  the cell `open`. A failed log leaves the cell unchanged. The next dispatch
  brief carries every answered park entry as an `Answered question:` line.
- **The leader keeps working (D3).** The leader sets the `question` waiting-on
  mark only when no ready cell remains in the lane, and lists every open parked
  question in its reply. The supervisor what-needs-you view lists one row per
  parked cell. Rule home: `skills/bee-swarming/SKILL.md`.
- **Two doors (D4).** `bee close` shows a `parked-cell` door, and
  `bee worktree merge` refuses `WORKTREE_MERGE_PARKED_CELL` before any write.
  The close door appears only while a parked cell exists, so a feature with
  none keeps its door list unchanged. Status, orient, merge-ready and
  start-feature all count a parked cell as unfinished work, never as idle.

This is still asynchronous, as "What this is not" requires: the worker parks and
exits; the human answers later, through one verb.

## Open Gaps

- ~~**Herding-lane dissent has no carrier.**~~ Closed 2026-08-29 by decision
  `7db30738` (cell sfg-2), along exactly the line the gap named: the dissent
  fields ride the herding mailbox result and the control loop transcribes them
  through the same one verb, so one record type still keeps one writer. Backlog
  item `p-05d2a4f4` is done.
- **A herding worker's prompt does not name the open-choice dissent route.**
  The "question an open choice through `bee cells dissent`" line renders for
  native workers only, because the herding prompt may hold no bee command. A
  herding worker can still dissent as data on its mailbox result (above), but
  its prompt does not say so next to the open choices.
- **Consider-grade dissent has no dedicated reader.** It is recorded and it owes a
  verdict like any other, but nothing surfaces it separately from blocker dissent.

## Pointers (implementation)

- Verbs: `bee cells dissent` (record, with `--new-evidence` as the one way
  past an existing verdict on the same cell) and `bee cells dissent-verdict`
  (answer).
  The claim is spelled `--reason`, not `--claim`: `claim` is a CLI-wide
  flag-alone boolean, so `--claim <text>` would swallow its own value token and
  the whole argument list would decline. The stored field is still `claim`.
- Rust: `packages/bee-rs/crates/bee/src/verbs/cells/dissent.rs`;
  the shared blocked-status write lifted into `apply_block_mutation` in
  `verbs/cells/util.rs` so the block path has one implementation, not two;
  the close door's dissent helpers in `verbs/drivers/close.rs`, read by
  `verbs/worktree/phases.rs` for the merge door.
- The three-part brief: `packages/bee/prompts/worker-cell.md` (the rendered
  cell prompt), `skills/bee-swarming/references/swarming-reference.md` (the
  template), `skills/bee-planning/references/planning-reference.md` (the
  `open_choices` key).
- The outcomes report: `packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs`
  (`ledger_root` reads `StoreRoots::main_root`).
- Parked cells: `packages/bee-rs/crates/bee/src/verbs/cells/park.rs` (park,
  answer, `feature_parked_cells`, `park_answers`); `verbs/cells/schedule.rs`
  (`classify` parked arm); `verbs/drivers/close.rs` (`parked-cell` door);
  `verbs/worktree/phases.rs` (`WORKTREE_MERGE_PARKED_CELL`);
  `verbs/supervisor.rs` (`collect_needs_you` rows); `verbs/drivers/prepare.rs`
  (answered lines in the brief).
- Both sub-verbs are declared in the cells dispatch table, which the
  served-but-undeclared law now sweeps — a served verb missing its registry row
  fails a contract test rather than shipping invisible.
