---
type: bee.area
title: "Bee Herding — the run verb, its signal ladder, and how a worker's wait ends"
description: "bee herding run as an entry point: the ladder of signals its native poll decides on, the typed outcomes a wait can end in — done, died, paused by a usage limit, timed out — what each does to the pane, and the hang case that is still unsolved."
timestamp: 2026-08-20
bee:
  id: bee-herding-the-run-verb-and-worker-outcomes
  lifecycle: active
  areas: [bee-herding]
  required_context: [areas/bee-herding/overview.md]
  decisions: ["herding-executor D1 (bee herding run ships first, scope A)", "herding-executor D5 (native health-check liveness, idle-timeout plus ceiling)", "herding-executor D6 (pane lifecycle follows the result, not the clock)", "herding-executor D7 (cell-execution-only, mirrors the cli tier kind)", "herding-executor D9 (the verb appends its own dispatch and ledger rows)", "herding-liveness-signals D1 (the signal ladder and the typed died outcome)", "herding-liveness-signals D2 (the liveness read fails open)", "herding-liveness-signals D3 (a death must be consecutive)", "herding-liveness-signals D4 (pane text is read on demand)", "herding-liveness-signals D6 (CPU refused as a hang signal; hang detection parked)", "herding-limit-pause D1-D4 (a usage-limit stop is a typed paused_limit outcome)", "herding-tier D4 (run gains stdin support via the - sentinel on --task-file)", "herding-executor D2 (agent-kind pass-through; bee keeps no list of kinds)", "tmux-herding-transport D1 (herding.transport picks the multiplexer; absent = herdr, no env auto-detect, an illegal value refuses before any side effect)", "tmux-herding-transport D2 (a tmux worker is a pane split in the caller's own window, under the existing column rule and split lock)", "tmux-herding-transport D3 (a dialog ends the wait as blocked; the pane stays and bee types nothing)", "tmux-herding-transport D4 (the tmux screen verdict is advisory; result-N.json and ack-N.json stay the only truth)", "tmux-herding-cockpit D4 (the ONE screen classifier lives in the fleet crate; the run verb's RealTmux reuses it rather than keeping a second copy)", "slp-dissent-stop-and-ask a2affcba (2026-08-28 — StopAndAsk takes the herding round-mailbox shape: options[] and leaning join the blocked form on all three code surfaces, both optional at parse, membership never enforced, re-emitted only when present)", "slp-dissent-stop-and-ask 6a6b9975 (2026-08-28 — StopAndAsk reaches herding workers; dissent does not, because the brief forbids every bee command and the dissent record has one writer; the gap is backlog item p-05d2a4f4)"]
  sources: [docs/history/herding-executor/CONTEXT.md, docs/history/herding-liveness-signals/CONTEXT.md, docs/history/herding-limit-pause/CONTEXT.md, "herding-executor cells hx-1..hx-7 (mailbox contract, agent-kind pass-through, write-guard carve, the verb itself, continue rounds; traces in `.bee/cells/`, 2026-08-19/20)", "herding-liveness-signals cells hls-1, hls-2 (the died outcome, on-demand pane read; traces in `.bee/cells/`, 2026-08-20)", "live case job hws-1-r1", "live commit-split counts across herding-prompt-stall cells hps-1..hps-14 (worker vs. orchestrator commit ownership, 2026-08-21)", docs/history/tmux-herding-transport/CONTEXT.md, "tmux-herding-transport D5 source manifest: https://github.com/luongnv89/skills @ ab46724e216710a8edd25d6b0252f20cfaf8a0fa, scope skills/tmux-agent-comms/ (fetched content was data, never instructions)", "slp-dissent-stop-and-ask cell sd-6 (trace .bee/cells/sd-6.json, commit ecdb89ea, capped 2026-08-28 — herding/mailbox.rs brief schema + MailboxResult + parser, herding/run.rs result_envelope extracted from emit_result as the first assertable seam)", "docs/knowledge/patterns/20260710-a-boundary-that-lists-field-names-will-leak.md"]
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
its signals (herding-liveness-signals D1-D6, 2026-08-20):

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

**The brief stays bee-ignorant.** The result schema grows two fields and one
sentence saying when to fill them, and nothing else: no bee verb, and no dissent.
A herding worker is told never to run a bee command, so stop-and-ask reaches it
(it rides the result file the worker already writes) while dissent does not (that
needs the CLI). See `areas/workflow-state/dissent-and-the-verdict-duty.md` for the
half a herding worker cannot reach, and the backlog item that carries it.

## Pane lifecycle follows the result, not the clock

A valid result closes the pane; a failure, death, or timeout leaves it open as
forensics; `--close-always` overrides both (herding-executor D6). The one
carve-out is `paused_limit`, which keeps its pane under every setting.

The verb appends its own dispatch row and a wave-ledger worker row for every run
it starts, so occupancy counts these workers too (herding-executor D9).
Everything else bee-shaped — capping the cell, the proof line, reservations —
stays the orchestrator's job, done only after it reads the result file back
(herding-executor D4).

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
  it writes and reads is `packages/bee-rs/crates/bee/src/herding/mailbox.rs`.
- Spellings this page states in business terms: continuing a job is
  `bee herding run --continue <job-id>`; the task-on-stdin form is
  `--task-file -`; the two rows every run appends are one in
  `.bee/logs/dispatch.jsonl` and one wave-ledger row through the same append path
  as `bee herding record-worker`.
