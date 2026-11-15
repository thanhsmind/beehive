# orchestrate

You own the program, never the code. Author briefs, drain the queue, keep the
frontier green, decide.

This is for a whole project handed to one standing coordinator: multi-day,
many cells, dozens to hundreds of workers, the owner checking in twice a day
instead of every five minutes. One task driven to a predicate is
`autonomous-run`. Route here when the work outlives any single agent. Work one
agent could finish inside the session's budget is not a program; measured
head-to-head, this playbook's ceremony turned a half-hour 12-unit job into 1
landed unit while a plain agent landed all 12. Below that line, route to
`autonomous-run`.

Ceremony must scale with the program. Every gate below prices in coordinator
minutes; on cheap near-identical units, collapse it as each section directs
rather than paying list price.

Three rules carry the rest.

- Completions are queue events, not interrupts.
- Every spawn and every resume carries the standing orders verbatim.
- The brief is the product. A vague brief fails quietly, because a worker
  cannot ask you a question.

Open a todolist with the steps below copied in verbatim. A step you skip stays
listed with `skip: <reason>`, so a skipped gate is a decision on the page and
not a silent omission.

#### Roles and placement

- **Coordinator (the leader session).** Frames, authors briefs, drains the
  inbox, owns the owner's report and makes judgment calls. It never authors or
  edits code: conflicted merges and code changes are always cells. Mechanically
  landing a verified unit with `bee worktree merge` is bookkeeping the
  coordinator does itself, because queueing finished work behind an idle merger
  is how a deadline harvests nothing. Workers are started, resumed and drained
  only through `bee dispatch prepare`, and the unattended loop is `bee-herding`
  (`bee herding control-loop`).
- **Sub-coordinator.** Durable, one per track, and only when the program
  exceeds what one coordinator's drains can manage. A track the coordinator can
  drain itself needs no middle layer: each nested layer re-pays a full
  orientation preamble, and a blocking sub-coordinator hides its children while
  the parent idles. It owns its track's units, authors its workers' briefs and
  spawns its own workers and verifiers. It rolls up aggregates at wave
  boundaries and never forwards raw child reports. Cap in-flight children at
  what one drain can process, roughly ten, as a rolling window; never as
  blocking batches, which cost the slowest child of every batch.
- **Worker / verifier.** One worker per cell, in its own worktree. Prefer
  fewer, broader workers, and one writer per worktree or branch
  (`bee-principle-separate-before-serializing-shared-state`), so two workers
  never write the same file. Run a unit's verifier on a different model family
  from its worker, so the same blind spot does not grade its own work.

Depth stays at coordinator, track, worker. Author the track decomposition per
project (build, landing and verification are common cuts, not a required
shape); hard-coded worker trees were tried and parked as too rigid.

#### Program record

Create the program's folder under `docs/history/<feature>/`. Every file has
exactly one writer; owners publish facts, readers aggregate at read time.

- `preferences.md` is the standing-orders register: numbered lines, one
  constraint each (model policy, merge order, verification bar, forbidden
  paths, escalation policy). Paste it verbatim into every spawn and every
  resume, because directives decay across resumes and each dropped one costs
  the owner a turn. When you catch yourself restating an instruction, append
  the line before you act (`bee-principle-encode-lessons-in-structure`).
- `units.tsv` has one row per unit: id, track, state, branch, head SHA, brief
  path. Update rows in place. A unit is a bee cell, and the cell record stays
  the source of truth for its claim and cap.
- `gates.md` parks owner gates (question, options, default on no answer), so a
  completion flood cannot wipe a pending question.
- `decisions.tsv` is the decision trail.
- `status.md` is derived from `units.tsv` at each drain, never hand-maintained;
  regenerate it from the table instead of narrating events into it, because
  hand-churned boards get rewritten on every event and go unreadable.

#### The brief

Your prompts to agents are your only product, and a sloppy brief compounds into
slop across the whole tree. Every spawn carries all of it; a field you cannot
fill is a unit you have not scoped yet.

```
GOAL         one sentence, the outcome, executable by a stranger with no chat access
SCOPE        paths this unit may write; paths it may not; its exclusive worktree or branch
CONTEXT      pointers to files and decisions; upstream reports pasted in full when this
             unit depends on them, because workers cannot see siblings
ACCEPTANCE   checkable criteria, one per line
VERIFY       exact commands or the verify-app path, plus known gotchas
TIMEBOX      rough cap on runtime; on expiry, return partial findings and stop rather than run on
FORBIDDEN    no rebase, no force-push, no fixes outside scope, plus unit-specific bans
REPORT       status, branch, head SHA, verdict, what you actually ran, deviations,
             suggested follow-ups
STANDING     <preferences.md pasted verbatim>
```

Size the brief to the unit. A one-command unit gets the template collapsed to a
paragraph that still names goal, scope, the verify command and the report
shape; a 4KB scaffold around a two-line edit costs more to write and obey than
the edit. A local spawn may reference the standing-orders file by path;
verbatim paste is for every resume.

A sub-coordinator brief adds its track boundary and unit list, its spawn
budget, the drain protocol and the rollup format (per child: name, status,
head SHA, verdict, one line; plus track status and merge delta).

A dependency is a context relay, not just ordering: undeclared upstream context
makes the worker guess. Missing fields are a refuse-to-spawn condition. Audit
one sampled worker brief per sub-coordinator per wave, concurrently with the
wave it samples, never as a gate in front of it; a failing brief stops that
track and fixes the sub-coordinator's instructions, not just the worker,
because brief quality decays late in a run. Never resume-chain a brief; respawn
fresh with consolidated scope.

#### Steps

1. **Frame.** State the done predicate as something countable ("all 126 units
   merged, each verified `unit-test-verified` or better"). Quantify scope:
   units, rough effort, expected merge order and the wall-clock budget. If one
   agent could finish inside that budget, stop here and run `autonomous-run`
   instead. Collapsing must not depend on another document being present: it
   means do the work directly in this session, plain workers where they help,
   verification inline, landing as you go, and none of the register or pilot
   machinery below. Schedule landing against the budget: by roughly 70% of it,
   stop spawning and land what is verified, because finished-but-unlanded work
   counts as zero. Name the tracks per project. Present the framing once;
   reversible prep proceeds without waiting.
2. **Install the program record.** Create the folder above, write the standing
   orders before any spawn, and seed `units.tsv` from the cells and worktrees
   that already exist. The orders come first because the first spawn is the
   first place a missing one costs a turn.
3. **Pilot.** Push one unit through the whole path: brief, worker,
   verification, merge candidate, verdict, merge. The pilot exists to falsify
   the brief template, the verify recipe and the unit size while that costs one
   agent instead of fifty. Fix the contract from pilot evidence before any
   fan-out. Scale the pilot to the unit: on programs of near-identical cheap
   units, the first unit is the pilot, run as a normal unit with its verify
   command inline, and fan-out starts the moment it lands. The dedicated pilot
   pipeline (separate verifier agent, audit gate) is for expensive or novel
   unit shapes, not for clone-units where a serialized pilot has nothing to
   falsify.
4. **Scale.** Spawn a rolling window of workers up to the in-flight cap,
   refilling as children finish; blocking batches pay the slowest child of every
   batch. Spawn track sub-coordinators only past the one-drain threshold in
   Roles. Recompute ready work after each drain; relay upstream reports into
   downstream briefs; keep sibling communication upward only. The sampled brief
   audit runs alongside the wave it samples and stops the next refill on
   failure, not the current one.
5. **Drain.** Run the queue discipline below at every drain point.
6. **Land.** Landing is continuous, never a terminal phase: integration starts
   with the first verified unit and runs alongside the remaining waves. Land a
   verified unit with `bee worktree merge` from the main checkout on green,
   without asking, as `uat_stop = "close"` already does, and stop at the `uat`
   door of `bee close`, which stays the owner's: uat is never auto-approved at
   any bypass level. Keep main green before more work lands on it; Merge safety
   governs. Advance `units.tsv` only on a merge or a reported new head SHA.
7. **Close.** Drain the final inbox, reconcile every spawned agent to a
   terminal row (done, abandoned, zombie-reconciled), confirm the predicate on
   the real artifact, confirm every landed unit has a verdict for its current
   head SHA, audit the decision trail, and encode recurring corrections into
   `preferences.md` or the brief template. Leave the folder intact; it is the
   postmortem.

#### Queue and drain

- On a completion, record the pointer in the program folder (agent, unit,
  status, report path) and return to what you were doing. Never deep-review
  inline; a completion that needs review becomes a verifier unit. Never review
  a diff inside a drain.
- Drain in batches at four points: the end of a critical section, a track
  rollup, a wake of the herding loop (`bee herding control-loop`, with a long
  interval as the fallback), and before an owner report. Arrivals during a drain
  wait for the next one.
- Critical sections you finish first: authoring a brief, a merge, a conflict
  decision, writing a gate, updating `units.tsv`.
- Each drain classifies every pointer (landed, needs-verify, failed, zombie,
  noise), writes the resulting rows to `units.tsv`, regenerates `status.md`,
  then spawns the next wave in one message.
- Account for every spawned child at its track's rollup: arrived, respawned, or
  its scope explicitly absorbed. Silently redoing a missing child's work hides
  both the wasted spend and the coverage gap its result existed to close.
- A drain turn ends with three lines: counts against the states, what changed,
  gates open. Detail lives in `status.md`; the full reply contract applies at
  checkpoints and close.

#### Merge safety

- Main is the frontier: recompute which units are merged from git after every
  merge, never from narrative, because a unit's row can say landed while its
  commit never reached main.
- Exactly one merger runs `bee worktree merge` at a time, recorded in the
  standing orders, because two merges racing for main lose a commit.
- Workers never rebase and never merge. A worktree's conflict goes to the
  merger as a cell, and `MERGE_CONFLICT` stops the run rather than retrying.
- Closing or retargeting someone else's branch goes through the merger only.
  Merges and conflict repair are units with briefs like any other.
- One retro watcher follows merged units for reverts, post-merge CI breaks and
  orphaned follow-ups.

#### Verification

Scale verification to the unit. When VERIFY is a single cheap command, the
worker runs it and reports the output, and the coordinator spot-checks
receipts; a dedicated verifier agent (on a different model family than the
worker) is for units whose verification is expensive, judgment-laden or
high-blast-radius. A verifier agent whose entire product would be rerunning one
command is ceremony, not verification.

Record a verdict per unit, keyed by cell and head SHA: `live-ui-verified |
unit-test-verified | type-check-only | verifier-blocked | verifier-failed`. The
cap's proof line is the worker's own verdict. CI green is an input to a
verdict, not a verdict. Behavioral work needs better than `type-check-only`.
`verifier-blocked` is not a pass; respawn when the environment heals.
`verifier-failed` gets a fix unit, not a re-verify. A worker may self-report; a
verifier overrides it on the same key. A new head SHA voids the verdict, so
re-verify after a rebase. The record answers "was this verified", not memory
and not the transcript.

A unit is not done until its output is externalized the moment it lands, never
batched to the end of the run: a worker commits in its worktree, a verifier
writes its verdict, receipts land in the program folder. Work that exists only
on one machine when it dies was never done.

#### Liveness and failure

- Never resume an agent to check on it; a resume restarts an idle agent. Probe
  read-only: `units.tsv`, the cell records, `git log` and the worktree list.
  Transcript mtime is not liveness.
- A silent death gets a synthetic postmortem row in the inbox (unit, failure
  mode, last evidence, options). Replan on evidence as it arrives; never wait
  for full quiescence.
- Retry by mode: cap-hit or oom, respawn with smaller scope; network-drop,
  retry as-is; tool-error, retry on a different model; unknown, retry once. Two
  retries, then abandon the unit and replan around it.
- A zombie that returns hours late reconciles against the current main and
  `units.tsv` before anything is accepted; the world moved while it slept.
  Salvage unique findings through a fresh unit, never a blind merge.
- When continued spawning would produce garbage tree-wide (bad upstream output,
  broken acceptance, dead infra), write a stop line at the top of the standing
  orders, let in-flight work finish, fix the cause, clear it.
- Bound your own infra retries the same way you bound a child's. After a few
  consecutive tool aborts, stop retrying: write a terminal handoff
  (`bee state handoff write --kind pause`, naming what is done, where it lives
  and the exact command to resume) and end the run. Hours of retry loops against
  a dead executor produce nothing a handoff would not.
- After a restart: dispatched local workers are dead, committed work is not.
  Re-read the standing orders and `units.tsv`, recompute main's state, respawn
  one sub-coordinator per track from its stored brief plus current state,
  drain, resume.

#### Escalation

Reaches the owner, batched into the status page rather than per item:
irreversible actions (force-push to shared branches, deploys, deletions,
closing someone else's branch), genuine product or preference calls no
experiment settles, a standing order that contradicts observed reality, and a
program-level dead end that survived a replan. Park each as a `gates.md` entry
before asking, and route work around it.

Never reaches the owner: merge-order nudges, retries, CI flake triage,
review-thread triage, format fixes, scope the brief already forbids (refuse and
continue), and "should I keep going". When in doubt, act and log; deferring is
the measured failure mode.

Mid-run discoveries fix only what blocks the merge frontier. Everything else
parks in follow-ups; at this fan-out a small scope leak multiplies into work
nobody asked for.

Reply: at checkpoints and close: the predicate and the count against it from
`units.tsv`, tracks and what each landed, the main state (landed units plus
SHAs), verdicts summary, what was abandoned and why, gates awaiting the owner
(the only asks), the program folder path, and the trail path. Numbers from the
tables, not narrative.

Proof line: the predicate's count from `units.tsv` checked against `git log` on
main, verbatim.
