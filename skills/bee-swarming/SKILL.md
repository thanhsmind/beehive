---
name: bee-swarming
description: >-
  Run approved cells to done with bounded workers. Use when the gate is approved and cells are open ("run them to done", "fan the workers out"), or when you are a worker given one assigned cell id. Not for the unattended backlog loop (bee-herding) or pane delivery (bee-herdr).
metadata:
  version: '0.2'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: unavailable
      reason: Both roles drive the work through the vendored bee binary. The binary is vendored into the repo by onboarding; no Node runtime is involved.
---

# Swarming — run the approved work

A **worker** has an assigned cell id in its dispatch prompt; everyone else
is the **orchestrator**. `bee orient` shows where the work stands either way.

## Orchestrate

You launch workers and tend results; you do not implement. The wave runs
inside the feature's worktree (rule: agents-worktree-first). Claiming from
main is control-plane and fine; execution depends on transport:

- **Native subagents** (Agent tool, Task tool) inherit the session's OS
  cwd. From main they cannot write into the worktree — the write guard
  refuses. Enter the worktree first (EnterWorktree, or a session/pane
  opened at the worktree path) and dispatch from there; use `bee worktree exit`
  as the way back.
<!-- bee:only pi -->
- On Pi: there are no native subagents. Every worker dispatches through
  herding only, and its full answer is the file at `report_path`.
<!-- bee:end -->
- **External herding workers** (bee herding run with --cwd) receive an
  explicit cwd at process start. The leader can stay in main while the
  worker writes in the worktree — no manual entry required. Every code cell, `tiny` included, is dispatched
through `bee dispatch prepare` because each role runs its own configured model; inline execution of a code cell is refused at cap. `bee route` refuses `--lane tiny` with `--files` above 1. Dispatches run one worker per cell
(`references/swarming-reference.md` ("Single execution worker in full")),
parallel by default: disjoint cells fan out concurrently (reservations
prove it, the machine-aware limit caps it (`herding.limits.concurrency`; unset, bee sizes it from the CPU count — `docs/knowledge/areas/bee-herding/waves-and-occupancy.md`), the same number for native workers, never a fixed count), serial needs a named file conflict.
From two cells up, state the one-line concurrency plan before dispatching.

1. `bee cells schedule --json` sets dispatch order — override only with a
   stated reason. Overlapping-file cells are fixed by scope or
   reservations (Hard rules).
2. `bee dispatch wave --runtime <rt> --feature <f>` prepares the current
   wave of one feature in one call — claim, reserve, and payload per
   ready cell, refusals landing in `skipped` with a typed reason instead
   of aborting the batch (`--feature` omitted only resolves from a bound
   session lane or the default record's own feature; nothing resolving is
   a typed refusal, never an every-feature grab — pass `--limit <n>` to
   bound how many cells of the wave are claimed). Add `--run` to have bee
   run every herding payload of that wave in parallel and wait: you get one
   `{ran, native, skipped}` value — one short row per herding cell, full
   envelopes saved under `.bee/workers/<id>-wave-run.json` — and you launch
   only the `native` items yourself, then cap each cell. `bee herding run`
   caps a herding cell itself when its worker reports done with a
   `cell: <id>` commit and a green proof line; any other done row stays
   uncapped — settle it with `bee cells finish --id <cell> --from-job
   <job_id>`. One cell, or a cell
   needing its own worker name, takes
   `bee dispatch prepare --cell <id> --worker <name> --runtime <rt> --claim`
   instead. The dispatcher may compose an Expertise section for the worker leader-style via `--expertise` (one entry per line, `<path> :: <purpose> :: <read-to>`), choosing from bee's own skill references and knowledge files; optional and judgment-driven, never auto-derived. A user-facing cell carries its mapped feature file that way — `.bee/verify/verify-app/features/<feature>.md` with its last-modified date, so the worker can judge a stale map. The cell already declares the job that
   selects its model (`role`, required at `cells add`) — escalate onto the
   session model only where the work earns it
   (`bee cells escalate --id <id>`; rubric:
   `references/swarming-reference.md`).
3. Spawn with exactly that payload — a whole wave goes out in ONE message,
   one tool call per cell. A herding payload carries `transport_ready` and
   `transport_reason`: read them before you spawn. The payload's `fallback`
   model applies only when it says `transport_ready: false`; `bee herding
   status` reports the calling session's own pane, never worker availability.
   Never paste session history; never hand a worker two cells.
4. Tend: learn a running worker's progress only from
   `bee herding status --job <id>` — one line — and never read a worker's
   transcript, pane log, log.txt or activity stream for progress. Read each worker's Result form
   (`references/swarming-reference.md`, "Result Formats"), never its
   prose. A `partial` result's `stopped` part becomes a new cell or a
   dissent verdict before close; your own final report states the four
   layers (same section). Silence is not failure — inspect `bee cells list` and
   `bee reservations list` before assuming stuck. What stuck means, the
   pilot for a same-pattern wave, and the user's stop word:
   `references/swarming-reference.md` ("Operating Contract in full",
   steps 3 and 6).
5. On `[DONE]`: the worker's word is never the evidence. Run the leader
   completeness check before accepting (`bee cells leader-check`) — the cap
   is not the end of the cell; the recorded mark is. Compare every approved
   requirement (the cell's `must_haves`, the plan's acceptance criteria)
   against the actual artifacts, wiring, and recorded proof; the worker's
   report only tells you where to look
   (`bee-hive/references/routing-and-contracts.md`, "Leader completeness check").
   A recorded proof is never re-run; `bee cells judge` for undeclared-file hits. At
   `standard`/`high-risk`, every `behavior_change` cell owes a
   `bee cells judge-record` verdict or `bee close` refuses (`judge-debt`) —
   except at `standard`, where a cell whose cap commit changed fewer than 30
   lines owes none. A code cell's tester returns the judge block for the cell
   it tested, and you record it with `bee cells judge-record`; the tier table
   is `bee-hive/references/gates-and-delegation.md`. Run the slice judge
   before you reach for close.
6. Slice clean: `bee close --feature <slug> --dry-run` names every
   remaining door with the command that settles it; the final slice lands
   through `bee feature finish` (Completion, below), whose `bee close`
   checks every capped cell's recorded
   proof line — same check `bee worktree merge` runs when the feature has
   a worktree. Doors check proof, they never run tests themselves; a cell
   with no valid proof is the remaining door. Doors are never waived.

**`[BLOCKED]` rescue ladder:** (1) re-dispatch the same cell with the
missing context; (2) `bee cells escalate --id <id>` and re-dispatch — the
session model is this session, so that rung hands the blocker to you;
(3) surface it to the user with the worker's diagnosis. If it invalidates the plan, return to bee-planning.
A `[PARKED]` cell is never re-dispatched or escalated; it waits for the answer.

**`[PARKED]`:** the lane keeps running — keep dispatching ready cells. Set
`bee state waiting-on set --kind question --subject "<question>"` only when no ready cell remains
in the lane, and list every open parked question in the reply. Record the
human's answer with `bee cells answer --id <cell> --answer "<text>"`; the
cell reopens and runs again with the answer in its brief.

**Other failures** — a handoff, a provider error, a red, a silent death —
each take one fixed response, and a second failed re-dispatch goes to the
user: `references/swarming-reference.md` ("Failure responses").

**Completion:** slice done with more approved work remaining → return to
bee-planning for the next batch IN THE SAME TURN — the slice boundary is
never a user question, never a "say go" (an approved plan stays frozen;
planning shapes the next batch, never reopens it). Final slice green → the road
splits on `uat_stop`. Either way the feature lands in one verb from main:
`bee feature finish --feature <slug> --areas <a,b>` runs `bee worktree
merge` on the FEATURE's own branch, then the scribing stamp (`bee state
scribing-run`), then `bee close`, and stops at the first refusal with
that verb's own message. It never approves a gate or the uat door; run it
again once the refusal is settled — a merged branch is already up to
date, so the run moves on.

- Default `uat_stop: "close"` (absent means this): run it FIRST, without
  asking — the merge is the publish-for-testing step; green caps are its
  precondition, not a user question. The worktree is kept, and `bee close`
  stops at the uat door. Then hand the user the reloaded
  product on main — what changed, how to run or see it — and ask for uat
  there. Mark the wait:
  `bee state waiting-on set --kind gate --subject "uat: <feature>"`.
  A failed uat is fixed in the worktree and finished again; the approval
  (`bee gate --name uat --approved true`) unlocks `bee close`.
- `uat_stop: "merge"`: put the work where the user tests it BEFORE any
  merge: `bee staging add --feature <slug>` plus the staging build, when
  the host repo records `commands.staging_build`; fall back to presenting
  the feature worktree itself when it does not, or when
  `"staging_before_merge": false` makes `bee staging add` refuse
  `STAGING_DISABLED`. Present what changed, how to run or see it, and the
  fixed question "Ready to merge?" — never merge on your own read of
  green tests. Mark the wait the same way. After the user approves uat
  (`bee gate --name uat --approved true`), land with `bee feature finish`
  — its merge takes the FEATURE's own branch, never staging's, and refuses
  `WORKTREE_MERGE_UAT_PENDING` for `standard`/`high-risk` features until
  that approval.

Either placement: a green merge that finds a staging record carries the
trigger-3 nudge `staging_rebuild_suggested: "bee staging rebuild"`, run
or suggested next. Capture is recorded as pending (bee-capturing runs
later, at the owner's pace).
Before declaring done: no active reservations, no in-flight workers
recorded.

A context mark never stops a cell: at or past it, hand off at the next clean stop, and after every green worktree merge write the handoff and continue in a fresh leader session (rule: agents-context-handoff-65; see `.bee/playbooks/handoff.md`). Below the mark, when a unit finishes and approved work remains, continue
in-session; finishing a unit is never a reason to stop.

**Reply:** capped cells over total, the proof line per cell, the merge or
uat state, blockers, and the one next action.

## Execute (worker)

A worker reads `references/execute.md`: the Execute loop and the Tester contract.

## Hard rules (both roles)

- Never spawn before the execution gate is approved; the orchestrator
  never edits source in a `standard`/`high-risk` wave.
- One cell per worker; the claim guard refuses a worker that claims, browses, or self-selects.
- Conflicts are fixed in scope or reservations, never by being careful.
- Never build on a red base — a red becomes its own fix-first cell (rule: agents-never-build-on-red).
- Two workers failing on one root cause stop the wave: dispatch nothing new, let in-flight work finish, open a fix-first cell, and resume after it caps.

## Headless

`bee-hive` ("Headless") governs; waves run without check-ins, and an
unrescuable blocker becomes an `Outstanding Questions` entry.

## References

| File | When to load |
|---|---|
| `references/swarming-reference.md` | Role rubric and escalation, worktree dispatch transaction, prompt template details, result formats |
| `references/execute.md` | You are the worker for one cell id: the Execute loop and the Tester contract |
| `references/worker-details.md` | Deep worker mechanics: finish and its refusals, advisor consult, friction triggers |
| `.bee/expertise/tests.md`, `.bee/expertise/debugging.md` | Authoring tests; hunting a red |
| `.bee/expertise/INDEX.md` | The cell is domain work — stored data, a caller-facing contract, a trust boundary, a rollout, a speed budget, a surface people use: route from the index, load exactly one |
