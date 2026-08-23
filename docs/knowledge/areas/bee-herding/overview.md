---
type: bee.area
title: "Bee Herding — the three-role cockpit, its safety boundaries, and adoption"
description: "A cockpit that runs several Claude Code sessions in parallel worktrees, over whichever pane transport one config key names (herdr or tmux): a dispatch loop that starts work behind an owner interlock, a merge gesture the owner runs by hand, and the safety boundaries that make unattended dispatch acceptable while keeping every landing in main a human act."
timestamp: 2026-08-20
bee:
  id: bee-herding-overview
  lifecycle: active
  areas: [bee-herding]
  required_context: [areas/worktree-parallelism/overview.md]
  decisions: [herding-adopt D1 (rename mandatory), herding-adopt D7 (posture split), herding-adopt D10 (dispatch interlock), herding-adopt D11 (merge is a gesture), herding-adopt D12 (supervised acceptance cycle), "herding-dispatch-lock-toggle D1-D3 (bee herding enable/disable/status CLI verb group, byte-identical to the manual marker gesture)", "herding-dispatch-lock-toggle D4 (CLI verbs stay owner-typed only, never called by bee automation)", herding-dispatch-lock-toggle D5 (no runtime guard added — explicit user decision), "herding-orchestration D8 (the control loop is a native command, not a script)", "herding-orchestration D13 (the control and working panes do not share a permission posture)", "herding-orchestration D19 (the live Windows run is an owner-run gap, not a blocker)", "tmux-herding-cockpit D1 (one herding.transport key selects the transport for the WHOLE cockpit)", "tmux-herding-cockpit D2 (the roles and bootstrap act on panes only through transport-neutral bee pane verbs)", "tmux-herding-cockpit D3 (the tmux mapping: session = workspace, window = tab, pane title = label, the bootstrap pane = chat)", "tmux-herding-cockpit D5 (the pane-verb pre-send guard fails open on an unreadable screen and always refuses a blocked one)"]
  sources: ["PR #50 (external contribution, vantt — the design)", "herding-adopt cells h-2, h-3 (adoption: rename, hardening, merge demotion, interlock, shipping switch; traces in `.bee/cells/`, 2026-07-23)", docs/history/herding-adopt/CONTEXT.md, docs/history/herding-adopt/reports/advisor-digest.md, docs/history/herding-dispatch-lock-toggle/CONTEXT.md, "hdlt-1 (cell: bee herding enable/disable/status CLI verb group; trace in .bee/cells/hdlt-1.json, 2026-07-23)", docs/history/herding-orchestration/CONTEXT.md, docs/history/tmux-herding-cockpit/CONTEXT.md]
  authoritative_for: "bee-herding: the three-role cockpit, its safety boundaries, and adoption"
  owns.code: [packages/bee-rs/crates/bee/src/herding.rs, "packages/bee-rs/crates/bee/src/herding/*", packages/bee-rs/crates/bee/src/herding/tmux.rs, packages/bee-rs/crates/fleet/src/backend/tmux.rs, packages/bee-rs/crates/fleet/src/screen.rs]
  owns.skills: ["skills/bee-herding/*"]
  owns.tests: [packages/bee-rs/crates/fleet/tests/choreography.rs, packages/bee-rs/crates/fleet/tests/herdr_backend.rs, packages/bee-rs/crates/fleet/tests/tmux_backend.rs, packages/bee-rs/crates/fleet/tests/manifest_boundary.rs]
---

# Bee Herding — the three-role cockpit, its safety boundaries, and adoption

This page is the cockpit itself: who may act, what arms it, and what bounds it.
The machinery it starts is documented beside it:

| Concept | What it covers |
|---|---|
| [Agent resolution and spawn commands](agent-resolution-and-spawn-commands.md) | Which agent a pane runs as, the named-agent registry, and how its command is built |
| [Handing a foreign agent its brief](handing-a-foreign-agent-its-brief.md) | The mailbox channel, the standalone-worker contract, and delivery receipts |
| [The run verb and worker outcomes](the-run-verb-and-worker-outcomes.md) | The poll's signal ladder, the typed outcomes, and pane lifecycle |
| [Waves and occupancy](waves-and-occupancy.md) | Fan-out over running workers, the ledger, and slot counting |

## Entry Points & Triggers

- **Bootstrap** is a one-shot the human runs directly. It pre-flights, then turns the cockpit on —
  starting **only** the dispatch loop.
- **Dispatch** is a cold process re-invoked on a fixed interval. It has no memory of any earlier
  iteration; every fact it needs is read live from state, the trunk, and the pane workspace.
- **Merge** is **not a loop.** It is a single-shot the owner runs by hand when they want finished
  work retired.
- **The config tier route** sends every purpose dispatched against a herding-kind slot through a
  pane automatically — see
  [agent resolution and spawn commands](agent-resolution-and-spawn-commands.md).
- **A wave** is a fourth entry point that briefs several already-running workers at once, and no
  role calls it — see [waves and occupancy](waves-and-occupancy.md).
- **A herding run** is a fifth entry point that starts a worker rather than briefing one that
  already exists — see [the run verb and worker outcomes](the-run-verb-and-worker-outcomes.md).

## Data Dictionary

- **Cockpit** — the pane layout bootstrap builds: one control pane per running role, plus the
  working panes.
- **Working agent** — a session started in its own isolated worktree to do one unit of work. Up to
  four run at once.
- **Enable marker** — an owner-created file. Without it, dispatch selects nothing. It is the switch
  that arms the loop, and only the human sets it — by hand, `touch`/`rm` on the marker file. The
  equivalent `bee herding enable`/`disable` CLI verbs performed the identical file
  operation and existed purely as a human-typed convenience; they are **not built into the current
  binary** (never ported off Node, and they now refuse by name), so the manual gesture is the only
  live form for arming. The enable state is readable as a command again: `bee herding status`
  reports whether the marker arms dispatch and, beside it, whether the pane transport is
  reachable at all and why (herding-reach hrc-2, 2026-08-22). It only reads — the marker is
  still set and cleared by the owner's own hand. No bee automation ever called any of them.
- **Stop gesture** — an owner-created file that halts the control loops at the next iteration
  boundary. It does **not** halt working agents already running.
- **Dispatchable** — a backlog item that is ready, unclaimed, has no worktree yet, and passes the
  work classifier. This is a *candidate* state, not a licence — the interlock still governs whether
  any candidate is acted on.
- **Transport** — which terminal multiplexer the cockpit reaches a pane through: `herdr` or `tmux`.
  It is one configuration key, `herding.transport`, absent meaning `herdr`, and it selects for the
  WHOLE cockpit — occupancy, waves, the control-pane allowlist, bootstrap, and both control roles
  (tmux-herding-cockpit D1). bee never guesses it from the environment. The two reachability
  probes — the status report's and the dispatch door's — read the configured transport's own
  environment and nobody else's: on `herdr` the herdr pane variables, on `tmux` the tmux ones, and
  each answer names the transport it probed (`kind`). A repo that sets nothing keeps the herdr
  answer it always got (tmux-herding-transport cell tht-1).
- **Pane verb** — a `bee herding pane …` command that performs one pane action on whichever
  transport the key names. The cockpit's whole vocabulary is
  `pane current|list|split|run|send-text|read|rename|close|layout|tab-create|tab-list|tab-focus`,
  plus `bee herding agent-start`, `bee herding pane-id --label` and `bee herding result`. A role
  document, a wave brief and the bootstrap script use ONLY these — never a raw `herdr` or `tmux`
  line (D2) — so a cold control agent learns one vocabulary rather than two. Every verb prints one
  envelope of the same shape on both transports (`ok`, `transport`, and either `result` or a typed
  `error.code`), and `bee herding result <dotted.path>` reads one field back out of it.
  A pane listing row carries eight facts, and every transport answers all eight: the pane's id,
  its label, its tab, the directory its shell started in, the directory its foreground process is
  in now, the command it runs, and the transport's own word on the agent — that agent's status and
  its session. A fact the transport cannot answer renders as an explicit empty value rather than a
  missing key, so a role reads one shape whichever transport replied. A layout row carries the
  pane's origin as well as its size, because the roles pick the chat pane by position
  (tmux-herding-cockpit cell thc-7).
- **Label**, **workspace**, **tab**, **chat pane** — the cockpit's four pane nouns, and on tmux each
  lands on a carrier that survives a reattach (D3): the workspace is the caller's current tmux
  SESSION, a tab is a WINDOW (`cockpit`, `runtime`), a pane's label is its pane TITLE
  (`select-pane -T`, and label lookup reads `list-panes`' `pane_title`), and the chat pane is the
  pane bootstrap was run from. On herdr each noun is that tool's own object of the same name.
  Creating a tab answers with the id of that tab's ROOT PANE, never a tab id, on both transports —
  a role names a fresh tab by the pane it can act on. The chat pane is the one noun that differs:
  on tmux it is the pane the bootstrap ran in, on herdr the cockpit tab's root pane
  (tmux-herding-cockpit cell thc-4).
- **Blocked pane** — a pane showing a trust, permission, or auth dialog. bee never types into one:
  the pane-verb send pre-reads the screen and refuses. That guard fails OPEN when the screen cannot
  be READ at all — an unreadable capture does not stop the send, matching the run verb's own
  posture — while a screen that classifies blocked always refuses (D5). A human answers the dialog;
  the pane stays open.

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
(herding-orchestration D13).** A control agent runs headless under an enumerated command surface; a
working agent runs with its permissions open inside its own worktree. Keeping the two argv forms
separate is what stops the narrow one from silently widening.

**The control loop is a native command, not a script.** The loop that re-invokes the dispatch role
on its interval is part of the tool itself. This is what made the cockpit portable: the previous
form was a shell script that depended on GNU utilities and a modern shell, so it could not run on
Windows at all. The one-shot cockpit setup is still a shell script and is a recorded gap
(herding-orchestration D8).

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
  R4). What it is confined to, and how its bee-ignorance is enforced rather than requested, is
  [handing a foreign agent its brief](handing-a-foreign-agent-its-brief.md).

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
  (herding-dispatch-lock-toggle D1-D5). `bee herding enable`/`disable` performed byte-identical
  operations to the manual `touch`/`rm` gesture (`status` is live again as a read-only report,
  herding-reach hrc-2) — same file, same resolution logic as the interlock —
  and deliberately carried no runtime guard (no TTY check, not hidden from `bee --help --json`): an
  explicit, considered trade-off that keeps the safety property exactly where R3 already put it
  (convention, not enforcement) rather than adding a new one. No bee automation, skill, or agent code
  ever calls these verbs itself.

## Edge Cases Settled

- **Starting an agent steals the owner's view.** Splitting a pane and creating a tab both honour a
  do-not-focus request; starting an agent has no such option and moves the workspace's focus to the
  new agent. For a loop that dispatches unattended on a fixed interval, every single spawn yanks the
  owner away from whatever they were reading — the one thing the do-not-focus option exists to
  prevent everywhere else. Found only by running the spawn for real; no documentation states it.
- **A control pane narrowed too far** stalls silently every interval — the exact failure the whole
  cockpit exists to end. This is why the control surface is enumerated against measured actions, and
  why it is documented to grow when a role gains a command, rather than being set to "read-only".
- **A worktree finished by bee but never merged** is the merge gesture's normal input; nothing
  retires it automatically, by design.

## Open Gaps

- **The live D6 scenario has not been run end to end on Windows.** The mechanism is proven there —
  the whole suite runs unexcluded on a Windows CI lane and every behavior that matters is pinned by
  platform-portable tests — but a live run needs a running herdr server, real panes and real agents,
  which CI cannot stand up. That run is an owner-run supervised cycle, the same shape as R7, not an
  agent-run step (herding-orchestration D19, narrowing D4).
- **The classifier reads the backlog row, not the work.** It vouches for an item from its one-line
  description, never opening the feature's own context. Reading the real work is the honest form of
  the safety check and is not yet built — the interlock (R3) is the compensating control meanwhile.
- **The supervised acceptance cycle (R7) is owner-run and outstanding** for this repo.

## Pointers (implementation)

- The skill and its three roles: `skills/bee-herding/SKILL.md`; the loop driver
  `bee herding control-loop`
  (`packages/bee-rs/crates/bee/src/herding/control_loop.rs`); the one-shot
  `skills/bee-herding/scripts/bootstrap-cockpit.sh`.
- The `herding` command group — `classify-lane`, `interlock`, `status`, `command-template`,
  `herdr-result`, `herdr-pane-id`, `pane`, `agent-start`, `pane-id`, `result`, `wave`,
  `occupancy`, `record-worker`, `run` and `control-loop`, the fifteen verbs the current binary
  actually serves — is implemented in `packages/bee-rs/crates/bee/src/herding.rs`, dispatched
  from `packages/bee-rs/crates/bee/src/router.rs`, and listed (with `enable` and
  `disable` marked `unavailable`) in the command catalog
  `packages/bee-rs/crates/bee/src/catalog.rs`. `enable` and `disable` are
  not among the live verbs and refuse by name; the manual `touch`/`rm` marker
  gesture is their only live form (see Data Dictionary). `status` is live
  (`herding.rs`, the `"status"` arm): enable state plus transport
  `{ready, reason, pane_id, kind}`. Test coverage is inline:
  the `#[cfg(test)] mod tests` block in `herding.rs`.
- The transport-neutral pane verbs are
  `packages/bee-rs/crates/bee/src/herding/pane_verbs.rs`: a `CockpitTransport` trait on top of the
  run verb's `PaneTransport`, implemented for both `RealHerdr` and `RealTmux`, plus the envelope
  every verb prints. The tmux wave backend is
  `packages/bee-rs/crates/fleet/src/backend/tmux.rs` and the one shared screen classifier is
  `packages/bee-rs/crates/fleet/src/screen.rs` — see
  [waves and occupancy](waves-and-occupancy.md) for why it lives there.
- The isolation the working agents depend on is `worktree-parallelism`; the guarded landing is that
  area's merge gate.
