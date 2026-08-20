---
type: bee.area
title: "Bee Herding — the three-role cockpit, its safety boundaries, and adoption"
description: "A herdr-driven cockpit that runs several Claude Code sessions in parallel worktrees: a dispatch loop that starts work behind an owner interlock, a merge gesture the owner runs by hand, and the safety boundaries that make unattended dispatch acceptable while keeping every landing in main a human act."
timestamp: 2026-08-20
bee:
  id: bee-herding-overview
  lifecycle: active
  areas: [bee-herding]
  required_context: [areas/worktree-parallelism/overview.md]
  decisions: ["herding-executor D1-D9 (bee herding run: mailbox executor, health-check liveness, pane lifecycle)", "herding-tier D1-D6 (the config tier route)", "herd-registry D1-D2 (the named-agent registry)", "herding-bare-agent D1-D5 (four-step bare-run agent resolution order: --agent > tier slot > string agent_command > array fallback)", herding-adopt D1 (rename mandatory), herding-adopt D7 (posture split), herding-adopt D10 (dispatch interlock), herding-adopt D11 (merge is a gesture), herding-adopt D12 (supervised acceptance cycle), "herding-dispatch-lock-toggle D1-D3 (bee herding enable/disable/status CLI verb group, byte-identical to the manual marker gesture)", "herding-dispatch-lock-toggle D4 (CLI verbs stay owner-typed only, never called by bee automation)", herding-dispatch-lock-toggle D5 (no runtime guard added — explicit user decision), i54-closeout D4, "herding-executor D1 (bee herding run ships first, scope A)", "herding-executor D3 (file mailbox is the completion signal)", "herding-executor D4 (worker stays bee-ignorant, orchestrator owns bee bookkeeping)", "herding-executor D5 (native health-check liveness, idle-timeout plus ceiling)", "herding-executor D6 (pane lifecycle follows the result, not the clock)", "herding-executor D7 (cell-execution-only, mirrors the cli tier kind)", herding-executor D9 (the verb appends its own dispatch and ledger rows)]
  sources: ["PR #50 (external contribution, vantt — the design)", "herding-adopt cells h-2, h-3 (adoption: rename, hardening, merge demotion, interlock, shipping switch; traces in `.bee/cells/`, 2026-07-23)", docs/history/herding-adopt/CONTEXT.md, docs/history/herding-adopt/reports/advisor-digest.md, docs/history/herding-dispatch-lock-toggle/CONTEXT.md, "hdlt-1 (cell: bee herding enable/disable/status CLI verb group; trace in .bee/cells/hdlt-1.json, 2026-07-23)", "i54-closeout cell i54-closeout-4 (herding spawn command config-driven templates; trace in .bee/cells/, 2026-07-24)", docs/history/herding-executor/CONTEXT.md, "herding-executor cells hx-1..hx-5 (bee herding run: mailbox contract, agent-kind pass-through, write-guard carve, the verb itself, this doc sync; traces in `.bee/cells/`, 2026-08-19/20)", docs/history/herding-bare-agent/CONTEXT.md]
  authoritative_for: "bee-herding: the three-role cockpit, its safety boundaries, and adoption"
---

# Bee Herding — The Three-Role Cockpit, Its Safety Boundaries, and Adoption

Bee herding runs several working sessions at once and retires them as they finish. It is one
cockpit with three roles, and the whole design turns on a single principle: **the dangerous act —
landing work in the shared trunk — stays a human gesture, while the cheap act — starting work in an
isolated copy — is what runs unattended.**

## Entry Points & Triggers

- **Bootstrap** is a one-shot the human runs directly. It pre-flights, then turns the cockpit on —
  starting **only** the dispatch loop.
- **Dispatch** is a cold process re-invoked on a fixed interval. It has no memory of any earlier
  iteration; every fact it needs is read live from state, the trunk, and the pane workspace.
- **Merge** is **not a loop.** It is a single-shot the owner runs by hand when they want finished
  work retired.
- **The config tier route**: setting `{"kind": "herding"}` on a `models.<runtime>.generation`
  slot (or any configurable slot) now routes EVERY purpose dispatched against it — cell,
  gather, reviewer, advisor, extraction — through `bee herding run` automatically, no
  per-purpose request needed; the old gather/review/advisor default-model fallback is gone,
  so the operator who sets the slot owns the pane cost for every purpose it serves
  (herding-tier D1-D6, widened by herding-review-slots D1). An optional `"fallback":
  "default"` on the same shape lets a failed herding run (spawn failure, timeout, invalid
  result) re-dispatch through the runtime's own default model path for that slot instead;
  absent, a failed run stays loud and keeps its pane open as forensics
  (herding-review-slots D3).
- **A wave is a fourth entry point, and no role calls it.** Dispatch starts one worker per iteration
  and never speaks to it again; a wave briefs several already-running workers in one act and waits
  on all of them together. It carries none of dispatch's guards — no arming marker, no classifier —
  so it is a fan-out over workers that already exist, never the way ordinary backlog work is
  started. It is invoked directly, by a human or by an agent that was told to.
- **A herding run is a fifth entry point, and it starts a worker rather than briefing one that
  already exists.** `bee herding run` starts ONE bee-ignorant external agent (any herdr-supported
  kind) in a fresh pane, hands it a fully self-contained brief over a file mailbox, and waits on
  that mailbox with native health-check liveness — no LLM call anywhere on the wait path
  (herding-executor D1, D3, D5). It carries none of dispatch's guards either. It is
  **cell-execution-only** (D7) — the mirror of the `cli` tier kind's gather/review/advisor-only
  boundary: a gather never dispatches through a herding pane.

## Data Dictionary

- **Cockpit** — the pane layout bootstrap builds: one control pane per running role, plus the
  working panes.
- **Working agent** — a session started in its own isolated worktree to do one unit of work. Up to
  four run at once.
- **Enable marker** — an owner-created file. Without it, dispatch selects nothing. It is the switch
  that arms the loop, and only the human sets it — by hand, `touch`/`rm` on the marker file. The
  equivalent `bee herding enable`/`disable`/`status` CLI verbs performed the identical file
  operation and existed purely as a human-typed convenience; they are **not built into the current
  binary** (never ported off Node, and they now refuse by name), so the manual gesture is the only
  live form. No bee automation ever called them.
- **Stop gesture** — an owner-created file that halts the control loops at the next iteration
  boundary. It does **not** halt working agents already running.
- **Dispatchable** — a backlog item that is ready, unclaimed, has no worktree yet, and passes the
  work classifier. This is a *candidate* state, not a licence — the interlock still governs whether
  any candidate is acted on.
- **Wave** — one coordinated run over several workers, described as a single value rather than a
  sequence of calls: the worker list, the timeouts, and the failure policy (wait-for-all,
  first-success-cancel-rest, best-effort) all sit in that one value, so a scenario is something you
  hand over rather than something you perform (herding-orchestration D11).
- **Wave ledger** — the append-only record of what each wave did: one row per wave, one entry per
  worker carrying its name, its pane, its worktree, its brief and its outcome. It is the cockpit's
  memory of who was started, and it is written at the moment of the spawn rather than at the end
  (herding-orchestration D10).
- **Occupancy** — how many working slots are actually taken. It is answered by crossing the ledger's
  unresolved workers against the live pane list, and it carries the SOURCE of its own answer: a real
  crossing, or a degraded timer fallback used when the live list cannot be obtained.
- **Mailbox** — `.bee/mailbox/<job-id>/`, the file channel `bee herding run` uses to talk to a
  bee-ignorant worker: `job.json` (the written brief), round-numbered `result-N.json`, and
  `log.txt`. Every write is staged tmp-then-rename, so a result file's appearance under its final
  name IS the completion signal — never the pane's screen (herding-executor D3).

## Behaviors & Operations

**Bootstrap starts one loop, never two.** It builds the full layout, including the merge pane, but
starts only dispatch. The merge pane is left idle for the owner's gesture. Starting merge as a loop
was considered and refused: unattended merge is where every serious risk concentrates.

**Dispatch is armed by the owner, not by readiness.** Before it builds any set of candidate work it
checks for the enable marker; absent, it does nothing and says so, naming the exact gesture that
arms it. This exists because the alternative — trusting that "nothing is ready" is a safe resting
state — was measured false: ready work is the trunk's *ordinary* condition, manufactured as a normal
side effect of finishing the planning of any feature. An unarmed loop that merely spins is the
intended resting state.

**Dispatch, once armed, starts work in isolation and never lands it.** It picks the highest-impact
dispatchable item, refuses anything its classifier cannot vouch for, and starts a working agent in a
fresh worktree. The worst an errant dispatch can do is start work in a throwaway copy — nothing it
does reaches the trunk.

**Merge is the human's act.** Run single-shot, it finds a worktree that bee's own state records as
finished, merges it behind the configured verify gate, cleans it up, closes its pane, and stops. On
a red verify it stops cold and never retries — a failed landing is a signal to a person, not a
condition to loop on.

**The stop gesture stops the controllers, not the workers.** It is honoured at the next iteration
boundary of the control loops. Working agents already running are independent sessions; stopping the
loop leaves them running, and retiring them is a separate act (close their pane, or unset the enable
marker so dispatch stops feeding new ones). This is stated plainly rather than implied, because a
stop that silently leaves agents running is worse than none.

**The loop is bounded.** A consecutive-failure ceiling and a default iteration cap ensure a missing
binary or a transient error cannot produce an infinite retry, and the control invocations carry a
turn ceiling — iterations were bounded in the original design, spend was not.

**The control panes and the working panes do not share a permission posture
(herding-orchestration D13).** A control agent runs headless under an enumerated command surface; a working agent runs with
its permissions open inside its own worktree. Keeping the two argv forms separate is what stops the
narrow one from silently widening.

**The control loop is a native command, not a script.** The loop that re-invokes the dispatch role
on its interval is part of the tool itself. This is what made the cockpit portable: the previous
form was a shell script that depended on GNU utilities and a modern shell, so it could not run on
Windows at all. The one-shot cockpit setup is still a shell script and is a recorded gap
(herding-orchestration D8).

**A wave is run once and recorded once.** The coordination that drives it is deliberately generic —
it knows nothing about this tool's own vocabulary, and lives behind a boundary a compiler enforces
rather than a promise (herding-orchestration D2/D5). Workers run beside each other on ordinary
threads rather than on an event runtime, because a wave is a handful of workers and each waiter is a
blocking poll (herding-orchestration D9). The entry point takes the worker list on its input, runs
the whole choreography — resolve and de-duplicate the targets, refuse any target that is not safe to
disturb, take a baseline, re-check each target immediately before handing it its brief, then wait on
all of them at the same time and aggregate what came back — and appends exactly ONE ledger row for
the whole wave. Each worker's outcome is classified into a named bucket (finished, refused at
pre-flight, changed under us before the send, send failed, timed out, or unverifiable afterwards)
rather than into a bare pass/fail, because partial failure is the normal case and the caller needs
to know which kind it got. A worker that fails does not stop the others.

**Occupancy is read, and an unverifiable read refuses.** The dispatch role asks for the occupancy
count instead of counting panes itself, and it reads WHICH answer it got. On a real crossing it
compares the count against the four-slot cap as before. On the degraded fallback it cannot know
occupancy, so it reports one plain line saying so and dispatches nothing that iteration. The
fallback fires exactly when the live pane list could not be obtained — which is also when counting
panes would have failed — so refusing is not a lost opportunity, and dispatching on a count nobody
can verify is the over-spawn the ledger exists to prevent.

**A herding run starts one bee-ignorant worker and waits on a file, not a screen.**
`bee herding run` is a native verb, not a script: it splits a pane off the caller's own
runtime pane, starts the agent through the same `herding.agent_command` seam the
working-agent spawn uses, and writes it a fully self-contained brief — task, absolute
paths, file constraints, the result schema, the tmp-rename write gesture — over the
mailbox described above, so a worker that has never seen bee can complete it
(herding-executor D4). Its poll loop is native and health-check based, at zero token
cost: `result-N.json` presence, `log.txt` mtime, worktree diff activity, and `herdr
agent list` status all extend the wait; a stale heartbeat past `--idle-timeout` ends
it, and an absolute `--ceiling` caps it regardless of activity as the busy-loop
backstop (D5) — there is no fixed short wall-clock timeout, because wall-clock alone
cannot tell a long cell from a stuck agent. Pane lifecycle follows the result, not the
clock: a valid result closes the pane, a failure or timeout leaves it open as
forensics, and `--close-always` overrides both (D6) — with one carve-out: a worker
stopped by a USAGE LIMIT is a typed `paused_limit` outcome, never `timed_out_idle`
(herding-limit-pause D1-D4, 2026-08-20). A stale heartbeat whose pane text matches a
limit pattern ("hit your session limit" / "usage limit", case-insensitive,
extensible) ends the wait as `paused_limit`; that pane is NEVER closed, even under
`--close-always`, and `job.json` is stamped `paused_limit_at` plus
`limit_reset_hint` (the matched line). `bee herding run --continue <job-id>` on a
stamped job with a live pane resumes the SAME round — a resume pointer through the
state-receipt delivery, stamp cleared, wait re-entered; a gone pane refuses typed.
The control loop's occupancy (unresolved ledger rows × live panes) already counts
the paused job as occupying its slot, so its work is never re-dispatched — a limit
stop is a pause, not a death (live case hws-1-r1). The verb appends its own
`dispatch.jsonl` row and a wave-ledger `record-worker` row for every run it starts, so
occupancy counts these workers too (D9) — everything else bee-shaped (`cells finish`,
the proof line, reservations) stays the orchestrator's job, done only after it reads
the result file back (D4). Unlike wave and dispatch, this entry point is
**cell-execution-only** (D7): the mirror of the `cli` tier kind's
gather/review/advisor-only boundary — a gather never dispatches through a herding
pane.

**The working-agent and control-pane spawn commands are config-driven templates,
byte-equivalent to the hardcoded default (i54-closeout D4).** `bee herding
control-loop` reads an optional `.bee/config.json` `herding.control_command` — a JSON array of
argv-token strings — and, when present, substitutes `{PROMPT}` / `{MODEL}` /
`{MAX_TURNS}` / `{ALLOWED_TOOLS}` per token and runs the result verbatim: tokens
are never joined into one string and re-split or shell-`eval`'d, so a
config-supplied command cannot smuggle shell injection through a placeholder
value. The working agent's spawn tail has the matching `herding.agent_command`
seam. `.bee/config.json`'s `herding.agents` (herd-registry D1/D2) names
several agents once — a map of name → argv tokens with the same validation.
When picking which external agent a herded pane runs as (`bee herding run` and
`bee herding wave` use the same resolver, herding-bare-agent D1-D5), resolution
follows a strict four-step precedence:
1. An explicit `--agent <name>`, resolved through `herding.agents`;
2. The cell-execution tier slot `models.<runtime>.generation`, but ONLY when it
   is an object with `kind: "herding"` and a non-empty `agent` string — that
   name resolves through `herding.agents`. This is the configured role-to-agent
   mapping that a bare `bee herding run` obeys;
3. `herding.agent_command` as a plain string, resolved through `herding.agents`;
4. `herding.agent_command` as an array (token 0 is the herdr `--kind`, rest
   are args), or the built-in default array when absent or malformed.

Any other slot shape (`kind: "herding"` with no agent, a plain model name like
`"sonnet"`, `{"kind":"cli",...}`, null, absent) is skipped and falls through.
A slot naming an agent not declared in `herding.agents` fails closed with a typed
`UnknownAgent` error listing every known key — never a silent fallback.
`<runtime>` resolves to `BEE_RUNTIME` when it names `claude`, `codex`, or
`opencode`, defaulting to `claude`. Since defaults-and-agent-env D3 (2026-08-20)
the registry starts from two BUILT-IN entries — `claude-sonnet` and `agy-flash` —
so those names resolve on a repo with no herding block at all; a same-name config
entry overrides its built-in, and the unknown-name listing includes the
built-ins. defaults-and-agent-env D4 (same day) adds a second entry shape beside
the argv array: `{"argv": [...], "env": {"KEY": "value"}}` — the env map is
exported into the freshly split pane as one `export K='v'` line BEFORE
`agent start` (keys `[A-Za-z_][A-Za-z0-9_]*`, values newline-free; any
violation drops that entry only, the registry's standing fail-open-per-entry
rule), and a failed env send is a typed spawn failure that closes the pane.
Only the `bee herding run` spawn path applies env; the wave/control-loop
caller resolves it but cannot apply it (its `agent start` lives in
`fleet::backend::herdr`, another crate — noted at the call site). A herd name
always means the pane transport; the cli tier kind is unrelated. When the key is absent, invalid, or empty, the command built is
byte-equivalent to the pre-existing hardcoded `claude -p ... --model sonnet
--max-turns ... --allowedTools ...` invocation — a project with no config
change sees no behavior change at all. A codex adapter example is documented
purely as an illustration of the seam; full codex-native herding (its own event
loop and pane protocol) stays out of scope (D4). None of enable/disable/status,
the dispatch interlock, or the merge owner-gesture change.

**Every step of handing a foreign agent its brief verifies rather than
trusts a flag (herding-executor arc, live-proven).** The pane start retries
through a booting shell; the brief travels only as a mailbox file
(`brief-N.txt`) behind a one-line pointer — never raw argv, never a
multi-line injected prompt; readiness is observed before the send and
delivery is confirmed against the pane's own text before the wait begins.
The operating detail lives in
`skills/bee-herding/references/operational-invariants.md` ("Spawn
resilience").

## Actors & Access

- **The owner** performs three acts and only three: bootstrap once, set the enable marker to arm
  dispatch (by hand — `touch`/`rm` the marker file), and run the merge gesture to land
  finished work. Everything else is the cockpit's.
- **The dispatch controller** reads state and the backlog and starts working agents; it is confined
  to an enumerated command surface, because a cold model re-invoked ~1,440 times a day will
  eventually improvise if left unconstrained.
- **The merge controller** reads finished-worktree state and runs the guarded merge; its command
  surface includes the writes that landing requires, and it runs the project's verify over the
  just-merged tree — so it executes whatever the working agents wrote.
- **A working agent** runs with its permissions fully open, as a deliberately accepted risk (see
  Business Rules). It is confined to its own worktree and branch until a merge. Since
  herding-worker-standalone D1-D3 (2026-08-20) that bee-ignorance is enforced, not just asked for:
  the brief opens with a standalone-executor contract (do the task only; ignore the repo's
  AGENTS.md/CLAUDE.md workflow instructions; never run a `bee` command; the mailbox result file is
  the one permitted `.bee` write), every fresh spawn exports `BEE_HERDING_WORKER=1` into the pane
  before `agent start` (the marker wins over per-agent env), and every `bee hook <name>` exits 0
  silently under that marker — checked once in the hook dispatcher, so a worker session gets zero
  bee preamble, zero guards, zero nudges. Live case that forced it (job hws-1-r1): a worker
  Claude Code in the repo activated the bee flow via AGENTS.md, and the write-guard denied its
  own mailbox result write.

## Business Rules

- R1 — **The name must match the managed-skill shape.** The distribution refuses any other at
  install time for every user, and the render ships only matching skills; the name is not a
  preference (D1).
- R2 — **Merge is a gesture, not a loop** (D11). Unattended merge alone carries the merge-authority
  risk, the long stop-latency window, and the exposure of running verify over unsandboxed agent
  code. Making it a keystroke removes all three and costs only the owner's presence.
- R3 — **Dispatch is interlocked behind an owner marker** (D10). The dispatchable state is the
  trunk's ordinary post-planning condition, so "nothing is ready" is not a safe resting state; the
  marker is.
- R4 — **The permission posture is split, and the split is a decision, not an oversight** (D7). The
  working agents keep full permissions as a recorded accepted risk — narrowing them makes an agent
  that hits a permission prompt with no terminal stall forever, defeating unattended dispatch. The
  control panes are narrowed to an enumerated command surface; "read-only" was measured to stall
  them, because both control roles genuinely write.
- R5 — **A red verify stops cold.** Merge never retries a failed landing; it is a signal to a
  person.
- R6 — **The loop is bounded** in iterations, consecutive failures, and control-invocation turns
  (D4/D12).
- R7 — **Adoption is not complete until one supervised end-to-end cycle has run** (D12). Every
  hardened defect was found by running things; the assembled system's first real run is a watched
  acceptance cycle the owner performs, not a headless claim.
- R8 — **The enable marker has two equivalent human-typed forms, never an automated one**
  (herding-dispatch-lock-toggle D1-D5). `bee herding enable`/`disable`/`status` performed byte-identical
  operations to the manual `touch`/`rm` gesture — same file, same resolution logic as the interlock —
  and deliberately carried no runtime guard (no TTY check, not hidden from `bee --help --json`): an
  explicit, considered trade-off that keeps the safety property exactly where R3 already put it
  (convention, not enforcement) rather than adding a new one. No bee automation, skill, or agent code
  ever calls these verbs itself.

## Edge Cases Settled

- **A working agent that fails to name its own pane** used to leave a slot looking free, because the
  four-slot cap was enforced by the control model counting panes. **That hole is closed** — the cap
  now rests on the wave ledger, not on a pane count (herding-orchestration D10, D18). §8 records a
  row the moment it spawns, carrying the worker's pane id, so an agent that never names its own pane
  is still visible to the next iteration: the ledger knows the pane even when the pane does not know
  itself. Occupancy is a liveness question — the ledger's unresolved pane ids crossed against herdr's
  own live pane list — and a one-hour timer survives only as an explicitly tagged FALLBACK for when
  that list cannot be obtained. §4 refuses to dispatch on a fallback answer rather than guessing,
  because a count it cannot verify is exactly the over-spawn the ledger exists to remove. The case is
  still worth knowing, because it names the class: a cap enforced by counting what you can see is
  only as good as the naming discipline of the things being counted.
- **Starting an agent steals the owner's view.** Splitting a pane and creating a tab both honour a
  do-not-focus request; starting an agent has no such option and moves the workspace's focus to the
  new agent. For a loop that dispatches unattended on a fixed interval, every single spawn yanks the
  owner away from whatever they were reading — the one thing the do-not-focus option exists to
  prevent everywhere else. Found only by running the spawn for real; no documentation states it.
- **"Idle" tracks the pane's own focus, not the work.** A worker's runtime status flips to idle or
  done according to whether that individual pane has been seen, not according to whether the work
  finished — a pane reported done while never being focused, and the multiplexer's own
  documentation states a coarser tab-level rule than its behavior actually follows. Any reading of a
  worker's status must therefore treat "done" as a fact about attention, never as evidence that the
  work is complete; that is why an explicitly UNVERIFIABLE outcome is a first-class answer rather
  than an error (herding-orchestration D7, which makes unverifiable one of the five worker states a
  backend must map its own vocabulary onto).
- **Starting a worker is two acts, not one.** The pane is created first, and the agent is started
  INTO that pane; a single call that both creates and starts no longer exists
  (herding-orchestration D12). What the agent itself is — which runtime, and the arguments it gets —
  is configuration, read as separate tokens and never re-joined into a string a shell could
  reinterpret (herding-orchestration D14).
- **A worker's agent name is derived from its pane, and the multiplexer will not take it raw.**
  Panes are numbered 1 to 9 and then A, B, C…, so most panes in a busy workspace carry an uppercase
  letter — and an agent name may only be lowercase letters, digits, dash and underscore, must begin
  with a lowercase letter, and may not exceed 32 characters. The derived name is therefore made
  legal by construction before it is used; the cost is that two panes whose ids differ only by case
  would collapse onto one name, which is accepted because no such pair exists. This was found by the
  first live run, not by any test: before the repair, every pane with an uppercase letter was
  refused and the whole wave aborted before sending anything.
- **A control pane narrowed too far** stalls silently every interval — the exact failure the whole
  cockpit exists to end. This is why the control surface is enumerated against measured actions, and
  why it is documented to grow when a role gains a command, rather than being set to "read-only".
- **A worktree finished by bee but never merged** is the merge gesture's normal input; nothing
  retires it automatically, by design.

## Open Gaps

- **A wave cannot confirm that a worker finished.** Proven by the first live run on Linux: two
  workers were started in their own worktrees, took their briefs and answered correctly, and the
  wave still reported both as UNVERIFIABLE — which is the honest answer, because the only completion
  signal available tracks the pane's attention rather than the work (see Edge Cases). The
  consequence is that a wave over ordinary agent sessions reports overall failure even when every
  worker did its job, so today the ledger row and the pane's own output are what an owner reads, not
  the verdict. Closing this needs a completion signal the worker itself emits.
- **The live D6 scenario has not been run end to end on Windows.** The mechanism is proven there —
  the whole suite runs unexcluded on a Windows CI lane and every behavior that matters is pinned by
  platform-portable tests — but a live run needs a running herdr server, real panes and real agents,
  which CI cannot stand up. That run is an owner-run supervised cycle, the same shape as R7, not an
  agent-run step (herding-orchestration D19, narrowing D4).
- **The classifier reads the backlog row, not the work.** It vouches for an item from its one-line
  description, never opening the feature's own context. Reading the real work is the honest form of
  the safety check and is not yet built — the interlock (R3) is the compensating control meanwhile.
- **The dependency on the multiplexer's JSON shapes is still unpinned** — there is no capability or
  version probe anywhere on the path. What changed is the failure DIRECTION, not the gap: an
  unrecognised status string now maps to unverifiable, and a live-pane list that cannot be read now
  returns the tagged fallback that makes dispatch refuse. So an upstream shape change degrades to a
  loud refusal rather than to a silent stall — but it is still not detected, and nothing names the
  version this cockpit was proven against.
- **The supervised acceptance cycle (R7) is owner-run and outstanding** for this repo.

## Pointers (implementation)

- The skill and its three roles: `skills/bee-herding/SKILL.md`; the loop driver
  `bee herding control-loop`
  (`packages/bee-rs/crates/bee/src/herding/control_loop.rs`); the one-shot
  `skills/bee-herding/scripts/bootstrap-cockpit.sh`.
- The `herding` command group — `classify-lane`, `interlock`, `command-template`,
  `herdr-result`, `herdr-pane-id`, `wave`, `occupancy`, `record-worker`, `run` and
  `control-loop`, the ten verbs the current binary actually
  serves — is implemented in `packages/bee-rs/crates/bee/src/herding.rs`, dispatched
  from `packages/bee-rs/crates/bee/src/router.rs`, and listed (with `enable`,
  `disable` and `status` marked `unavailable`) in the command catalog
  `packages/bee-rs/crates/bee/src/catalog.rs`. `enable`, `disable` and `status` are
  not among the ten live verbs and refuse by name; the manual `touch`/`rm` marker
  gesture is their only live form (see Data Dictionary). Test coverage is inline:
  the `#[cfg(test)] mod tests` block in `herding.rs`.
- `run`'s own module — pane split/start, the native poll loop, and pane-lifecycle
  decisions, each seam-tested with a fake so no test needs a real `herdr` on PATH —
  is `packages/bee-rs/crates/bee/src/herding/run.rs`; the mailbox contract it writes
  and reads (`job.json`, `result-N.json`, `log.txt`) is
  `packages/bee-rs/crates/bee/src/herding/mailbox.rs`.
- The isolation the working agents depend on is `worktree-parallelism`; the guarded landing is that
  area's merge gate.
