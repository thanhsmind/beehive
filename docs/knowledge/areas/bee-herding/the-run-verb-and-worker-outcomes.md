---
type: bee.area
title: "Bee Herding — the run verb, its signal ladder, and how a worker's wait ends"
description: "bee herding run as an entry point: the ladder of signals its native poll decides on, the typed outcomes a wait can end in — done, died, paused by a usage limit, timed out, interrupted, cancelled — what each does to the pane, the retryable envelope bit, the git handoff block, the interrupt and cancel control verbs, the hang case that is still unsolved, and how a blocked worker hands back options, a leaning, and now a structured dissent the verb transcribes through the one dissent writer."
timestamp: 2026-08-20
bee:
  id: bee-herding-the-run-verb-and-worker-outcomes
  lifecycle: active
  areas: [bee-herding]
  required_context: [areas/bee-herding/overview.md]
  decisions: ["herding-cockpit-completeness D1-D8 (interrupted and cancelled outcomes, the stalled/recovered projection, the interrupt and cancel verbs, the orphan sweep, the retryable bit, the git block, the six new signals, the docs sync)", "herding-executor D1 (bee herding run ships first, scope A)", "herding-executor D5 (native health-check liveness, idle-timeout plus ceiling)", "herding-executor D6 (pane lifecycle follows the result, not the clock)", "herding-executor D7 (cell-execution-only, mirrors the cli tier kind)", "herding-executor D9 (the verb appends its own dispatch and ledger rows)", "herding-liveness-signals D1 (the signal ladder and the typed died outcome)", "herding-liveness-signals D2 (the liveness read fails open)", "herding-liveness-signals D3 (a death must be consecutive)", "herding-liveness-signals D4 (pane text is read on demand)", "herding-liveness-signals D6 (CPU refused as a hang signal; hang detection parked)", "herding-limit-pause D1-D4 (a usage-limit stop is a typed paused_limit outcome)", "herding-tier D4 (run gains stdin support via the - sentinel on --task-file)", "herding-executor D2 (agent-kind pass-through; bee keeps no list of kinds)", "tmux-herding-transport D1 (herding.transport picks the multiplexer; absent = herdr, no env auto-detect, an illegal value refuses before any side effect)", "tmux-herding-transport D2 (a tmux worker is a pane split in the caller's own window, under the existing column rule and split lock)", "tmux-herding-transport D3 (a dialog ends the wait as blocked; the pane stays and bee types nothing)", "tmux-herding-transport D4 (the tmux screen verdict is advisory; result-N.json and ack-N.json stay the only truth)", "tmux-herding-cockpit D4 (the ONE screen classifier lives in the fleet crate; the run verb's RealTmux reuses it rather than keeping a second copy)", "slp-dissent-stop-and-ask a2affcba (2026-08-28 — StopAndAsk takes the herding round-mailbox shape: options[] and leaning join the blocked form on all three code surfaces, both optional at parse, membership never enforced, re-emitted only when present)", "slp-dissent-stop-and-ask 6a6b9975 (2026-08-28 — StopAndAsk reaches herding workers; dissent does not, because the brief forbids every bee command and the dissent record has one writer; the gap is backlog item p-05d2a4f4)", "slp-followup-gaps 7db30738 (2026-08-29 — a herding worker's dissent travels as DATA on the mailbox result across the same three surfaces, read as leniently as options/leaning and with the severity passed through unchecked; the run verb transcribes it through the one record-dissent writer and reports the outcome as dissent_recorded plus dissent_error)", "pi-result-mailbox D1 (the worker's full report rides the mailbox as round-numbered report-N.md; a result without one stays legal)", "pi-result-mailbox D2 with a recorded deviation (the report travels as report_path — never inline at any size, because a truncated envelope is unparseable; report_note names an expected-but-unusable report)", "pi-result-mailbox D6 (one delivery path per job, decided structurally at dispatch: --inbox-session is the detached fact and writes the pending marker before the pane spawns; no flag writes none)", "herding-cockpit-completeness c943feb9 (2026-09-06 — bee herding interrupt sends Escape, keeps pane open, records outcome interrupted, job stays resumable through --continue)", "herding-cockpit-completeness 468c6cb8 (2026-09-06 — stalled and recovered status words in herding status and run poll tick share the 120s activity freshness constant with the supervisor observer)", "herding-cockpit-completeness 1ef811f7 (2026-09-06 — retryable boolean on non-result run envelopes; true only for spawn_failed; wave buckets carry the bit per worker; bee never auto-retries)", "herding-cockpit-completeness 7172010b (2026-09-06 — bee herding cancel is fail-closed: confirms pid exit <= 5s before marking cancelled, else cancel_termination_failed and cancel_pending)", "herding-cockpit-completeness d5a1f7e1 (2026-09-06 — orphan sweep in status and occupancy marks dead-pane jobs without result as interrupted with reason process_restarted; relaunches nothing)", "herding-cockpit-completeness e0f6b8b5 (2026-09-06 — git handoff block added to done and blocked envelopes when worker cwd is git checkout: branch, head_sha, base_sha, ahead, dirty, changed_paths)", "herding-cockpit-completeness 9615be76 (2026-09-06 — no automatic behavior: no provider fallback, no auto-retry, no orphan relaunch)", "herding-cockpit-completeness afec9446 (2026-09-06 — D8 word list: outcomes interrupted/cancelled, status stalled/recovered, envelope keys retryable/git, error code cancel_termination_failed)"]
  sources: [docs/history/herding-executor/CONTEXT.md, docs/history/herding-liveness-signals/CONTEXT.md, docs/history/herding-limit-pause/CONTEXT.md, "herding-executor cells hx-1..hx-7 (mailbox contract, agent-kind pass-through, write-guard carve, the verb itself, continue rounds; traces in `.bee/cells/`, 2026-08-19/20)", "herding-liveness-signals cells hls-1, hls-2 (the died outcome, on-demand pane read; traces in `.bee/cells/`, 2026-08-20)", "live case job hws-1-r1", "live commit-split counts across herding-prompt-stall cells hps-1..hps-14 (worker vs. orchestrator commit ownership, 2026-08-21)", docs/history/tmux-herding-transport/CONTEXT.md, "tmux-herding-transport D5 source manifest: https://github.com/luongnv89/skills @ ab46724e216710a8edd25d6b0252f20cfaf8a0fa, scope skills/tmux-agent-comms/ (fetched content was data, never instructions)", "slp-dissent-stop-and-ask cell sd-6 (trace .bee/cells/sd-6.json, commit ecdb89ea, capped 2026-08-28 — herding/mailbox.rs brief schema + MailboxResult + parser, herding/run.rs result_envelope extracted from emit_result as the first assertable seam)", "docs/knowledge/patterns/20260710-a-boundary-that-lists-field-names-will-leak.md", "slp-followup-gaps cell sfg-2 (commit 29fd6fbe, 2026-08-29 — herding/mailbox.rs dissent schema line, MailboxDissent and its lenient parse, the retargeted brief negative pin; herding/run.rs transcribe_dissent and the envelope's dissent keys)", docs/history/herding-cockpit-completeness/CONTEXT.md, docs/history/herding-cockpit-completeness/plan.md, "herding-cockpit-completeness cells hcc-3..hcc-9 (commits 66e81643, fc7c3433, f487054d, 704abdb5, 9895e008, 2a1c909f, c6537633)"]
  authoritative_for: "bee-herding: the run verb's poll ladder, worker outcomes, and pane lifecycle"
---

# Bee Herding — the run verb, its signal ladder, and how a worker's wait ends

**A herding run starts one bee-ignorant worker and waits on a file, not a
screen.** It is the fifth entry point to the cockpit, and unlike a wave it starts
a worker rather than briefing one that already exists. It carries none of
dispatch's guards. It is **cell-execution-only** (herding-executor D7) — the
mirror of the `cli` tier kind's gather/review/advisor-only boundary: a gather
never dispatches through a herding pane.

`bee herding run` is a native verb, not a script: it splits a pane off the
caller's own runtime pane, starts the agent through the same spawn seam the
working-agent spawn uses, and writes it a fully self-contained brief — task,
absolute paths, file constraints, the result schema, the tmp-rename write gesture
— over the mailbox, so a worker that has never seen bee can complete it
(herding-executor D4). The task itself may arrive on standard input rather than
as an argument, and an empty standard input refuses exactly as an empty task
argument does — a caller piping a generated task never gets a worker started on
nothing.

**The caller's pane is the main pane, and it is split exactly once.** Workers
live in one column beside it and never take space from it again. The first
spawn splits the caller's pane to the RIGHT and takes a column one third of the
tab wide — floored at the sixty-column worker minimum, capped at half — so the
main pane always keeps the larger share and its full height. Every spawn after
that splits DOWN inside that worker column, stacking under the previous worker.
The parent is therefore the roomiest pane in the tab EXCLUDING the caller's
own, and the direction follows from which pane that picked, not from any
measurement of the rectangle.

Two earlier rules were tried and retired against live evidence. Reading the
aspect ratio answered `right` again and again on a wide tab: a 120-column tab
went 60/30/15, and both the 30- and 15-column children died mid-submission.
Taking the roomiest pane overall then ate the human's own pane instead: five
spawns on a 173-by-50 tab cut it from 50 rows to 13 while every worker kept 25.
The human needs their own pane readable at all times; a worker only needs
enough width to accept a submission (herding-split-serialize D2).

**The share passed to the terminal tool is what the PARENT keeps, not what the
child gets.** Measured live: asking for a quarter left the parent a quarter and
handed the child the rest. A stacking split inside the worker column halves it;
the one split that creates the column asks for whatever leaves the worker its
computed width.

**Counting the panes and splitting one are a single indivisible step.** Every
spawn runs as its own process, so the rule above is only correct over a count
that already includes the sibling a concurrent spawn just made. Spawns
therefore queue: one at a time counts the tab and takes its pane, and the next
one counts again afterwards. Without that queue, five simultaneous spawns from
one tab each saw an untouched root, each answered `right`, and the fifth worker
died waiting for an acknowledgement it could never send from a sliver — the
same stale count also hides the width floor, since every one of them measures
the untouched root. A spawn waits up to two minutes for its turn, well inside
the acknowledgement budget, and a turn held by a process that has since died,
or held far past any plausible split, is taken over. If the wait runs out
anyway, or the queue itself cannot be used, the spawn warns and proceeds
unqueued: a worker that never starts is a worse outcome than one that lands in
the wrong place (herding-split-serialize D1).

## The poll decides on a ladder of signals, not one signal

The poll loop is native and health-check based, at zero token cost, and it ranks
its signals (herding-liveness-signals D1-D6, 2026-08-20; herding-cockpit-completeness c943feb9, 468c6cb8, 7172010b, 2026-09-06):

0. **Job mark** — a `mark` field stamped into `job.json`. If marked `interrupted`,
   the wait ends immediately as typed outcome `interrupted`, leaving the pane
   open (c943feb9). If marked `cancelled`, the wait ends as typed outcome
   `cancelled` (7172010b).
1. **Truth** — a `result-N.json` for the round outranks every other signal.
2. **Agent liveness** — the pane's foreground process list, where the agent
   counts as present when any foreground process is not the pane's own shell.
   Pane liveness is not agent liveness: an agent that exits leaves its pane alive
   at a shell prompt, so a pane-existence check cannot see the death. No result
   plus no agent process is a typed `died` outcome, reported in seconds rather
   than after the whole idle window.
3. **Progress** — `log.txt` mtime advancing, or the reported agent status being
   `working`. A stale heartbeat past `--idle-timeout` ends the wait.
4. **Classification** — reached only when progress has gone stale; see the
   usage-limit carve-out below.

An absolute `--ceiling` caps the wait regardless of activity as the busy-loop
backstop (herding-executor D5) and outranks the `died` rung, so a ceiling and a
death arriving together still report the ceiling. There is no fixed short
wall-clock timeout, because wall-clock alone cannot tell a long cell from a stuck
agent.

Two rules keep the liveness rung from becoming a hazard. **The liveness read
fails OPEN**: an unreachable or unreadable process list reports "unknown", never
"absent" — the opposite direction from the pane check that guards a continued
job, which fails closed on purpose. A refusal gate may safely refuse on bad
information; a kill decision may not, because the job it would end may be hours
deep. **A death must be consecutive**: several successive absent readings are
required before `died` is declared, and a single "unknown" reading RESETS that
count rather than counting toward it, so an absent/unknown/absent flicker never
ends a healthy job.

**Stalled and recovered status projections.** During the poll loop, a job whose
activity file (`activity.json`) is older than the single 120 s freshness constant
(`mailbox::ACTIVITY_FRESHNESS_SECS = 120`, shared with the supervisor observer;
herding-cockpit-completeness 468c6cb8) while its pane process is alive projects
status word `stalled`. The run stream emits exactly one progress line:
`herding: job <id> stalled`. When activity resumes, it projects `recovered` once
and emits `herding: job <id> recovered`, transitioning back to `working` with no
extra line. This execution stall is distinct from herdr's prompt stall (below
sixty columns, where an unsubmitted prompt fails to reach an agent); status
output prefixes every line with the job id (`herding: job <id> stalled`) so the
two never share a line.

Pane text is read only at the moment it is needed — when the heartbeat has
already gone stale and the stall must be classified — not on every poll tick. The
classification fires on exactly the tick it always did; what changed is that a
quiet stall no longer pays for thousands of discarded screen captures.

## A usage-limit stop is a pause, not a death

A worker stopped by a USAGE LIMIT is a typed `paused_limit` outcome, never
`timed_out_idle` (herding-limit-pause D1-D4, 2026-08-20). A stale heartbeat whose
pane text matches a limit pattern ("hit your session limit" / "usage limit",
case-insensitive, extensible) ends the wait as `paused_limit`; that pane is NEVER
closed, even under `--close-always`, and `job.json` is stamped `paused_limit_at`
plus `limit_reset_hint` (the matched line).

Continuing a stamped job with a live pane resumes the SAME round — a resume
pointer through the pointer-delivery path (herding-prompt-stall D1/D4), stamp
cleared, wait re-entered; a gone pane refuses typed. The control loop's
occupancy already counts the paused job as occupying its slot, so its work
is never re-dispatched (live case
hws-1-r1).

## A job is not always one round

Reusing a finished job continues it: the same mailbox is kept, the follow-up
brief reaches the agent ALREADY RUNNING in the pane rather than starting a second
one beside it, and the wait then targets the next round's result file. A missing
job, a missing prior result, or a pane that is gone all refuse with a typed
reason — continuing is only meaningful against a job that actually got
somewhere.

`bee herding run --continue <job-id>` reads `mark` in `job.json` before
proceeding (herding-cockpit-completeness c943feb9, 7172010b):
- If mark is `cancelled` or `cancel_pending`, continue refuses with typed refusal
  `ContinueRefusal::Cancelled` and a FIX line (`FIX: a cancelled job cannot be
  continued` or `FIX: run bee herding cancel <job-id> to complete cancellation`).
- If mark is `interrupted`, continue succeeds: it resumes the job in the open pane
  and clears the `interrupted` mark from `job.json` upon resumption.

## A blocked worker can hand back options and a leaning

A worker that can only say "blocked, here is prose" hands the orchestrator a
problem. Since slp-dissent-stop-and-ask (cell sd-6, commit ecdb89ea, 2026-08-28)
it can hand back a **choice** instead: a blocked result may carry `options` — one
self-contained sentence per element — and a `leaning`, the worker's own pick
written out as a verbatim repeat of one option.

Three properties hold this together, and all three are deliberate:

- **Both fields are optional at parse, and membership is never enforced.** A
  leaning that matches no option still parses. Strict validation of a foreign
  agent's output would turn a useful blocked answer into a malformed one and cost
  a whole round — the expensive failure, traded away.
- **`leaning` is free text, not an index.** An index is an off-by-one waiting to
  happen across a foreign-model boundary, and it is unreadable to a human opening
  the mailbox file.
- **They are re-emitted only when present**, so a result carrying neither parses
  and re-emits exactly as it did before the fields existed.

The two fields spell the same names on all three surfaces the round crosses — the
brief's result schema handed to the worker, the parsed result the verb reads back,
and the JSON envelope the verb re-emits — because a boundary that lists field
names in three places is a boundary that leaks the one you forgot. The envelope
gains no `status` key: done-versus-blocked has always ridden `outcome` there, and
still does. The plain-text output path is untouched; the orchestrator's own door
always asks for JSON.

**The brief stays bee-ignorant.** The result schema grows fields and one
sentence per group saying when to fill them, and nothing else: no bee command
anywhere in it. That constraint has never moved. What moved is how much a worker
can say through it — see the next section.

## A worker's dissent rides the same result file, as data

The brief forbids every bee command and the dissent record has exactly one
writer, so for a while a herding worker could hand back a choice but never a
disagreement with the TASK ITSELF. Since slp-followup-gaps (cell sfg-2, commit
29fd6fbe, 2026-08-29) it can do both, and it does it the way stop-and-ask
already worked: the dissent travels as **data on the result file the worker
already writes**, never as a command it runs.

The shape is one optional `dissent` object carrying exactly three fields —
`claim` (what is wrong with the task as it was handed over), `alternative` (what
the worker would do instead), and `severity` (`blocker` or `consider`) — plus
one sentence in the brief: fill it only when you disagree with the task itself,
and leave it out entirely when you agree. Those three names are spelled on the
same three surfaces `options`/`leaning` cross — the brief's result schema, the
parsed result the verb reads back, and the JSON envelope the verb re-emits — for
the same reason: a boundary that lists field names in three places leaks the one
you forgot.

**The parse is exactly as lenient as the pair above it.** Absent, not an object,
missing any one of the three fields, or a field that is not a string all read as
NO dissent, and none of them turns a usable round into a malformed one. The
severity string is passed straight through **unchecked**: the closed set is the
dissent writer's own single check, and a second copy of a closed set is the
drift a boundary listed twice always earns.

**The run verb transcribes a carried dissent through that one writer.** When the
wait ends with a parsed result carrying a dissent, the verb calls the same
record-dissent function `bee cells dissent` routes to, against the run's own
cell id, so the record shape, the closed severity set, the secret scan, the
blocker tooth and the claim release all keep exactly one implementation — a
`blocker` dissent parks its cell here because that writer parks it, not because
the verb re-implemented the tooth. Ownership is deliberately NOT forced: the
force-ownership override is audited, so a cell another session holds refuses,
and the refusal is reported.

**A lost voice is reported, never swallowed.** The envelope always re-emits the
raw dissent object beside a `dissent_recorded` flag, and stamps `dissent_error`
with the reason whenever the write did not land — either the run carries no cell
id, so there is no cell to record against and the orchestrator is told to record
it by hand, or the writer itself refused. Because the raw object rides the
envelope either way, a failed transcription stays recoverable. A result carrying
no dissent adds no key at all and keeps the envelope's exact prior key set, and
the exit code is untouched: a blocked result already exits non-zero, and the
transcription never votes on it.

The brief's own negative pin moved with the boundary instead of being deleted.
It still forbids naming a bee command — now by scanning for any standalone
`bee <word>` rather than one literal spelling, so a newly added command line
fails on its own — while the `dissent` FIELD NAME it used to ban is now required
in the result schema. See `areas/workflow-state/dissent-and-the-verdict-duty.md`
for the record itself, its severities, the obligated verdict, and the two doors
that refuse while a dissent is unanswered.

## The worker's report rides the mailbox as a PATH, under the same additive law

Locked in pi-result-mailbox D1/D2/D6 (2026-08-30). The gap it closes was
measured, not imagined: a finished job's `result-1.json` carried only
`{status, summary, files_changed, proof}`, so the DIGEST — the thing the
dispatcher actually asked for — died with the pane it was printed into.

The worker now writes **`report-N.md` beside `result-N.json`**, round-numbered
and atomically like every other mailbox file, and the result may name it in
`report_path`. The envelope grows exactly two keys, and both obey the
no-new-key law stated above: **`report_path`** appears only when a report is
on disk, **`report_note`** only when a report was expected and is not usable.
A result with neither is byte-identical to the envelope this verb printed
before the feature existed — a legacy worker stays legal, and a missing report
is never a parse error.

The report travels as a PATH and **never as a body, at any size**. That is a
recorded deviation from D2's letter, which permitted an inline report under a
size cap: one long line read back through an orchestrator's tool call
truncates, and a truncated envelope is unparseable — strictly worse than the
loss it was meant to fix. One extra file read buys a digest that always parses.

Resolution happens at the ONE site with filesystem access (the parse), so the
envelope builder stays pure, and it prefers evidence over guesswork in a fixed
order: a **declared** `report_path` wins, and a declared path with no file
behind it becomes an explicit note rather than a silent fall back to a guess;
nothing declared falls to a convention probe for `report-N.md`, which is
accepted only when it is at least as new as that round's own brief delivery.
An older file is the same-round resume case — the worker rewrote its result and
left the previous attempt's report behind — and it is reported as a stale-report
note, never attached. A **malformed** result surfaces its error and any report
path together: a good report does not die with a broken result JSON.

**The detached fact is a flag, because nothing else in here can see it.** A
shell that backgrounds this verb is invisible from inside it, so
`bee herding run --inbox-session <token>` carries the fact structurally: its
PRESENCE means the caller is not synchronously waiting. The flag writes one
pending marker, `.bee/result-inbox/<token>/<job-id>.json` — job id, mailbox
path, cell id — **before** the pane is split, so no finished result can ever
exist without a marker pointing at it, and the marker is a POINTER: the
envelope is never copied into it, so there is exactly one copy of the truth.
The write is advisory in every failure (an unusable token or an unwritable
marker is a stderr note, never a refusal to dispatch) and `--dry-run` leaves
none, because it promises nothing. No flag, no marker, and the caller reads
the result off this verb's own output — one delivery path per job, decided at
dispatch. Who drains those markers, and the at-least-once guarantee that comes
with it, is Pi's half: `areas/hook-runtime/catalog-projections-and-activation.md`
and `docs/config-reference.md` (§ Pi).

**Pi result drain reads only result file headers, never run envelope keys.** The
Pi extension's result drain (`renderResultInjection` in
`.pi/extensions/bee-guard.ts:729-743`) parses only `{job_id, cell_id, status,
summary, proof, report_path}` from the worker's mailbox `result-N.json`. The new
envelope keys added by `bee herding run` (`retryable`, `git`) belong to the CLI
output envelope, not `result-N.json`. They never reach the Pi injection header,
and Pi ignores unknown properties without requiring code changes (afec9446, D8
reader list).

## Pane lifecycle follows the result, not the clock

A valid result closes the pane; a failure, death, timeout, or interrupt leaves
it open as forensics (herding-executor D6, herding-cockpit-completeness c943feb9);
`--close-always` overrides them. Cancel closes the pane and confirms process exit
(7172010b). The other carve-out is `paused_limit`, which keeps its pane under
every setting.

The verb appends its own dispatch row and a wave-ledger worker row for every run
it starts, so occupancy counts these workers too (herding-executor D9).
Everything else bee-shaped — capping the cell, the proof line, reservations —
stays the orchestrator's job, done only after it reads the result file back
(herding-executor D4).

## Two control verbs: interrupt and cancel

Two CLI verbs provide deterministic control over running workers (herding-cockpit-completeness c943feb9, 7172010b, 9615be76, afec9446):

- **`bee herding interrupt <job-id>`** (c943feb9):
  Sends an Escape key to the job's recorded pane (`"esc"` via herdr `pane send-keys`,
  `"Escape"` via tmux `send-keys` without literal mode `-l`). The pane **stays open**.
  The verb marks the job by setting `mark: "interrupted"` and `mark_reason: "user"`
  in `job.json`. A waiting `bee herding run` poll loop detects the mark and returns
  typed outcome `interrupted`. The command outputs:
  `herding: interrupted job <id> (pane kept open)`.
  Resuming an interrupted job is supported via `bee herding run --continue <job-id>`,
  which clears the mark on resume.
  If the job is unknown, it refuses with error code `job_not_found` and a `FIX:` line.
  If the job is already marked `cancelled` or `cancel_pending`, it refuses with error
  code `already_cancelled` and a `FIX:` line.

- **`bee herding cancel <job-id>`** (7172010b):
  A fail-closed termination sequence. The verb reads the pane's foreground process ID
  (`process_info`), writes `mark: "cancel_pending"` and `cancel_pid` into `job.json`,
  closes the pane (`pane_close`), and polls process liveness (`is_pid_alive`) every
  100 ms up to a 5-second deadline.
  - If process exit is confirmed within 5 s, it writes `mark: "cancelled"` and outputs:
    `herding: cancelled job <id> (pane closed, pid <pid> exited)`.
    A waiting `bee herding run` returns terminal outcome `cancelled`.
  - If process exit is not confirmed within 5 s, the command exits non-zero with error
    code `cancel_termination_failed`. The mark remains `cancel_pending` in `job.json`.
    The error message includes a typed refusal and FIX line:
    `FIX: kill pid <pid> by hand, then run bee herding cancel <id> again`.
  - A subsequent invocation of `bee herding cancel` against a `cancel_pending` job
    skips pane closure and re-probes the recorded `cancel_pid`, finalizing `cancelled`
    once the process has terminated.
  - The wave failure policy `FirstSuccessCancelRest` routes through this exact
    fail-closed cancel path.

### Job marks in `job.json`

Job state in `.bee/mailbox/<job-id>/job.json` tracks marks and lifecycle transitions
with five dedicated fields:
- `mark`: `"interrupted"` | `"cancelled"` | `"cancel_pending"`.
- `mark_reason`: `"user"` (direct verb invocation) or `"process_restarted"` (set by orphan sweep).
- `mark_at`: ISO-8601 timestamp when the mark was recorded.
- `cancel_pid`: Foreground process PID captured during cancellation confirmation.
- `last_status`: Tracks the most recent projected status word (`stalled`, `recovered`, `working`)
  to prevent redundant transition progress lines.

## The retryable envelope bit

Every non-result run envelope emitted by `bee herding run` carries the boolean key
**`retryable`** (herding-cockpit-completeness 1ef811f7, afec9446).

- **`true`** only when the worker has proven to have performed zero work:
  - `spawn_failed`: the agent process failed to launch.
  - (In wave bucket rows: `send_failed` and `flipped_before_send`.)
- **`false`** for any outcome where the worker started and may have modified files
  or system state:
  - `died`, `interrupted`, `cancelled`, `timed_out_idle`, `paused_limit`, `blocked`,
    `unverifiable_after_send`.
- **Omitted** entirely on `done` and `dry_run` envelopes, respecting the envelope
  no-new-key law.

**No automatic retry.** In adherence to decision 9615be76 (no automatic behavior),
bee never retries or relaunches a job on its own. `retryable` is an advisory signal
for the outer orchestrator or human loop to decide whether re-dispatching is safe.

## The git handoff block

When a job ends in a result (`done` or `blocked`), `bee herding run` inspects the
worker's working directory (`cwd`) and adds the **`git`** envelope key
(herding-cockpit-completeness e0f6b8b5, afec9446).

The key appears **only when `cwd` is inside a git checkout** (verified via
`git rev-parse --is-inside-work-tree`). If `cwd` is not a git worktree, or on
non-result outcomes (`died`, `interrupted`, `cancelled`, etc.), the `git` key is
omitted.

The `git` object contains six fields read directly by bee from the worker checkout:
1. `branch`: Current branch name (`git rev-parse --abbrev-ref HEAD`).
2. `head_sha`: Commit SHA of `HEAD` (`git rev-parse HEAD`).
3. `base_sha`: Merge-base between `HEAD` and the main checkout's `HEAD`
   (`git merge-base HEAD <main HEAD>`).
4. `ahead`: Number of commits `HEAD` is ahead of `base_sha`
   (`git rev-list --count <base>..HEAD`).
5. `dirty`: Boolean indicating whether uncommitted modifications exist
   (`git status --porcelain`).
6. `changed_paths`: Array of relative paths changed in the worktree (the deduplicated
   union of committed differences against base via `git diff --name-only <base>..HEAD`
   and uncommitted modifications from porcelain status).

The worker-reported `files_changed` array remains untouched beside `git`. While
`files_changed` reflects the worker's self-reported claims, `git` provides verified,
git-backed evidence computed independently by the host tooling.

## A worker pane must be wide enough to take a submission at all

Below sixty columns a submitted prompt does not merely render badly — the herd
tool reports it stalled before the agent ever processes it. Proven live from
one 120-column tab: the first split produced a sixty-column child that carried
a full round to a written ack and result, while two thirty-column children both
died mid-submission.

So the run verb always measures the width the CHILD will land at, never the
parent's, and the worker column's width is floored at that sixty-column minimum
before the main pane's share is worked out. When no pane in the tab can yield a
workable child, the worker gets a FRESH TAB's root pane at full width — never a
sliver, and never a refusal — and it never takes the human's focus. A geometry
read that fails at all falls open to the caller's own pane (herding-prompt-stall,
cells hps-12 and hps-13).

## Transport: herdr or tmux

**The run verb reaches a pane through one of two multiplexers, and a single
config key picks which** (tmux-herding-transport D1-D4, 2026-08-22; the
marker defaults trace to the source manifest in tmux-herding-transport D5).
`herding.transport` in `.bee/config.json` is the string `herdr` or the
string `tmux`. Absent is `herdr` — the unchanged default, and a missing or
unparseable config reads the same way. bee **never sniffs the environment**
for it: `$TMUX` and `$HERDR_ENV` are both ignored as selectors, because a
session nested in both tools would otherwise pick by accident. Any other
value is a typed refusal naming both legal spellings, and the refusal lands
**before** the job file, the mailbox, or any pane split — a typo'd
transport never half-starts a worker. A dry run names the transport it would
have reached for, beside the brief it would have sent; the key is added on the
dry-run answer alone, and a real run's answer keeps every field it had before
the transport choice existed (tmux-herding-transport cell tht-4).

Everything this page describes above survives the switch. On tmux a worker
is still a pane split inside the CALLER's current window, under the same
one-column rule and the same cross-process split lock — never a detached
session per worker (D2). The mailbox contract, the signal ladder, the typed
outcomes, and the pane lifecycle are the herdr ones, untouched.

Three differences are real, and each follows from tmux having no agent API.

- **Status is a screen read, and it is advisory only (D4).** There is no
  `agent list` and no lifecycle state to ask for, so worker status is a
  classifier over a bounded `capture-pane` read: content stability plus two
  marker lists (busy, blocked) held as config data under `herding.tmux.*`
  with upstream defaults, because marker strings are another tool's UI
  chrome and rot with its releases. The classifier has no "done" answer at
  all — `result-N.json` and `ack-N.json` stay the ONLY truth for done and
  delivered, exactly as under herdr. **That classifier is now shared, and
  there is exactly one of it** (tmux-herding-cockpit D4): the body moved
  down into `fleet::screen` — one `ScreenSettings`, one `Screen`, one
  `classify`, with the marker literals and both tail windows unchanged —
  and this verb's `RealTmux` reuses it, as do waves and the cockpit's own
  `pane list --with-status`. `fleet` still never reads `.bee/config.json`;
  bee's `TmuxSettings::from_config` resolves `herding.tmux.*` and hands the
  settings over already decided.
- **Whether a worker's screen has gone quiet is the transport's memory, not
  one call's** (tmux-ready-wait D1). The quiet window — a run of identical
  screen reads that is both long enough in count and long enough in time — is
  held per pane and survives across calls, so a caller that polls in short
  bursts reaches idle after about one window instead of never. A read that
  fails, a pane that closes, and a new worker started into a pane each drop
  the window. A dialog still ends the wait immediately, ahead of the window
  (pattern `pattern-20260823-a-settling-window-rebuilt-per-call-never-closes`).
- **A dialog ends the wait as `blocked`, and bee types nothing.** A pane
  showing a trust, permission, or auth prompt stops the wait; the pane
  STAYS OPEN and the human answers it (D3). bee never sends a key into a
  dialog — a wrong marker match would answer on the human's behalf. The
  same rule guards the send path: the prompt gesture pre-reads the pane and
  refuses rather than typing into a dialog.
- **Every send is two calls, and every read is filtered by pane id.** Text
  reaches a tmux pane only by being typed, so a submission is `send-keys -l
  <text>` followed by a separate `send-keys Enter`; one call cannot express
  "these bytes literally, then submit". And `list-panes -t <pane>` lists
  the whole WINDOW the pane belongs to, not that pane alone — so geometry
  and liveness rows are matched on the pane id before they are read, never
  taken as the only row returned.

Implementation: `packages/bee-rs/crates/bee/src/herding/tmux.rs` — a
`PaneTransport` peer of the herdr one, selected at a single construction
site from `herding.rs`'s `transport_kind`, and keeping only the half that
is bee's (`TmuxSettings::from_config`) while re-exporting
`classify`/`Screen` from `packages/bee-rs/crates/fleet/src/screen.rs` at
its own path, so every sibling call site reads as it did. It never runs `new-session`,
`attach-session`, or `switch-client`: the first would put the worker where
the human is not looking, and the last two need a TTY a tool shell does not
have.

## The commit split, as observed

The worker owns the edit and the result file; the orchestrator owns the cell
commit and the cap. That split is now backed by observation, not just
design: across this feature's fourteen dispatches the worker committed with
the required commit-trailer form exactly ONCE, used a bare (id-only) trailer
FIVE times, and made no commit at all THREE times — and every time, the
orchestrator made the path-scoped cell commit itself. A bee-ignorant worker
does not reliably carry bee's commit conventions, trailer form included,
even when the brief states the rule.

A second run (dispatch-door-upfront, 2026-08-22) repeated the split on the
same runtime: two of three workers committed with a bare id line instead of
the required trailer and never ran the finish step; the orchestrator rewrote
each commit and capped by hand. The third worker got it right only because
its brief spelled the exact trailer text and named the finish step — which
is the second arm of the open gap below, now with evidence that it works.

The same split covers the proof. A foreign worker's result text names a
command but often not its outcome (herding-reach hrc-2: the proof line
carried no result), and the completion door records what it is handed
rather than running anything. So the orchestrator runs the proof itself
before it caps, and treats the worker's proof text as a claim, never as
evidence.


## The run verb's outcome token says nothing about the cap door

A worker dispatched through `bee herding run` **does not reliably cap its own
cell**, and the run's own token cannot tell you whether it did. Observed twice
consecutively on `nudge-consult`: nc-1 returned `outcome=done` with a valid proof
string while leaving six files uncommitted *and* the cell still claimed; nc-2
committed cleanly and still returned without ever running `bee cells finish`.
Both looked identical from the envelope.

Two consequences follow. An orchestrator checks cell status and `git log` after
every herding run rather than trusting the token. And the herding worker prompt
(or its close path) owes a fix that makes the cap an action taken rather than
text echoed. A Task-tool worker is a different case: it caps reliably today, so
this is a transport-specific hole, not a worker-contract hole.

**A busy but useless pane is capped by the clock, not by the heartbeat.**
`--idle-timeout` only counts a dead heartbeat, so a worker that reads for fifty
minutes without editing is never stopped — it keeps its own heartbeat alive by
working. The `herding.ceiling_seconds` config key, read by `bee dispatch
prepare`, appends `--ceiling` to the herding command and bounds that case; the
verb's own default ceiling of 21600 s is far too loose to serve as one.

## Open Gaps

- **Whether the brief should even ask the worker to commit is unresolved.**
  Given the observed split above, either the brief drops the commit step
  for a herding worker entirely and the orchestrator always makes the cell
  commit, or the trailer rule gets restated in a form a bee-ignorant agent
  can follow verbatim instead of assumed known. The counts above (1 correct,
  5 bare, 3 none, out of 14; then 1 correct, 2 bare, out of 3) are the
  evidence. The worker brief today says only "cell id as the last body line"
  while the cap checker demands the literal `cell: <id>` form — a bare id
  satisfied the brief's letter and failed the cap. Applied 2026-08-22: the
  brief and the swarming skill now spell the literal `cell: <id>` trailer and
  say a bare id fails the cap. Whether that closes the split is the next
  run's evidence to record here.
- **Hang detection remains an open gap.** A worker that is stuck but still
  emitting output satisfies every progress source there is. Accumulated CPU time
  was the intended discriminator and was REFUSED on measurement
  (herding-liveness-signals D6): an interactive agent's event loop burns CPU
  while it sits blocked, so any-delta never goes stale and catches nothing; and
  treating flat CPU as an override kills an agent legitimately waiting minutes on
  a remote call. Pane output counters fail identically, for the same reason a
  spinner advances the log. Picking a real discriminator needs calibration traces
  from healthy-but-blocked workers against genuinely hung ones, and the question
  is parked against a registered trigger until those exist.
- **A caller that waits in the foreground and times out loses the worker's
  cleanup.** Seen live 2026-09-06: the caller's own tool timeout (120 s) ended
  the wait first, the worker later finished with a valid result, and the pane
  sat idle with nothing to retire it — closed by hand. The correction on record
  is orchestration, not code: the foreground wait now runs without a premature
  caller timeout, and a rerun returned a done outcome with the pane closed.
  Whether the verb should refuse a foreground wait it cannot outlast, and
  recovery from abrupt process death, are both untested and open.
- **A valid result closes the pane before a promised report is checked.**
  Completion is read from the result file, never console text; a declared
  report that is missing, empty, unreadable, a directory, or stale becomes a
  note on the envelope while the result stays eligible to close the pane.
  "Report first, result last" is a worker instruction, not a hard
  report-completeness check. The repair requested 2026-09-06 — a promised
  report is validated before the result authorizes pane closure, with the
  legacy no-report worker still legal — is scoped in the planning lane
  herding-late-cleanup and not built.

## Pointers (implementation)

- The pane-width floor is `MIN_PANE_WIDTH` (60) in the same module, with
  `resolve_split_parent` picking the roomiest parent excluding `own_pane`,
  `split_direction` answering from whether that parent is the caller's own,
  `first_split_geometry` computing the worker column's columns and the
  `--ratio` that leaves them, `narrow_pane_refusal` measuring the child's
  resulting width, and the `tab_create` fallback on the herd seam (cells
  hps-12, hps-13, hss-3).
- The spawn queue is a lock file at `.bee/locks/herding-pane-split.lock` under
  the main checkout, taken and released by
  `packages/bee-rs/crates/bee/src/herding/split_lock.rs`; `run.rs`'s
  `split_worker_pane` holds it across the layout read and the split, waiting
  `SPLIT_LOCK_WAIT` (120s) and failing open past it (cells hss-1, hss-2).
  Release is identity-checked — a guard removes the lock file only while the
  on-disk holder still carries its own pid AND token, so a process that already
  lost its turn to a stale takeover cannot delete the winner's lock.
- `run`'s own module — pane split/start, the native poll loop, and pane-lifecycle
  decisions, each seam-tested with a fake so no test needs a real multiplexer on
  PATH — is `packages/bee-rs/crates/bee/src/herding/run.rs`; the mailbox contract
  it writes and reads is `packages/bee-rs/crates/bee/src/herding/mailbox.rs`
  (including marks via `read_mark`/`write_mark`, orphan sweep via `mark_orphans`,
  `transition_status`, and `ACTIVITY_FRESHNESS_SECS = 120`).
- Control verbs `bee herding interrupt` and `bee herding cancel` are implemented in
  `packages/bee-rs/crates/bee/src/herding/job_verbs.rs`.
- Spellings this page states in business terms: continuing a job is
  `bee herding run --continue <job-id>`; interrupting a job is
  `bee herding interrupt <job-id>`; cancelling a job is
  `bee herding cancel <job-id>`; the task-on-stdin form is
  `--task-file -`; the two rows every run appends are one in
  `.bee/logs/dispatch.jsonl` and one wave-ledger row through the same append path
  as `bee herding record-worker`.
