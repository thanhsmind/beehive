---
type: bee.area
title: "Bee Herding — the three-role cockpit, its safety boundaries, and adoption"
description: "A herdr-driven cockpit that runs several Claude Code sessions in parallel worktrees: a dispatch loop that starts work behind an owner interlock, a merge gesture the owner runs by hand, and the safety boundaries that make unattended dispatch acceptable while keeping every landing in main a human act."
timestamp: 2026-07-24
bee:
  id: bee-herding-overview
  lifecycle: active
  areas: [bee-herding]
  required_context: [areas/worktree-parallelism/overview.md]
  decisions: [herding-adopt D1 (rename mandatory), herding-adopt D7 (posture split), herding-adopt D10 (dispatch interlock), herding-adopt D11 (merge is a gesture), herding-adopt D12 (supervised acceptance cycle), "herding-dispatch-lock-toggle D1-D3 (bee herding enable/disable/status CLI verb group, byte-identical to the manual marker gesture)", "herding-dispatch-lock-toggle D4 (CLI verbs stay owner-typed only, never called by bee automation)", herding-dispatch-lock-toggle D5 (no runtime guard added — explicit user decision), i54-closeout D4]
  sources: ["PR #50 (external contribution, vantt — the design)", "herding-adopt cells h-2, h-3 (adoption: rename, hardening, merge demotion, interlock, shipping switch; traces in `.bee/cells/`, 2026-07-23)", docs/history/herding-adopt/CONTEXT.md, docs/history/herding-adopt/reports/advisor-digest.md, docs/history/herding-dispatch-lock-toggle/CONTEXT.md, "hdlt-1 (cell: bee herding enable/disable/status CLI verb group; trace in .bee/cells/hdlt-1.json, 2026-07-23)", "i54-closeout cell i54-closeout-4 (herding spawn command config-driven templates; trace in .bee/cells/, 2026-07-24)"]
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

**The working-agent and control-pane spawn commands are config-driven templates,
byte-equivalent to the hardcoded default (i54-closeout D4).** `bee herding
control-loop` reads an optional `.bee/config.json` `herding.control_command` — a JSON array of
argv-token strings — and, when present, substitutes `{PROMPT}` / `{MODEL}` /
`{MAX_TURNS}` / `{ALLOWED_TOOLS}` per token and runs the result verbatim: tokens
are never joined into one string and re-split or shell-`eval`'d, so a
config-supplied command cannot smuggle shell injection through a placeholder
value. The working agent's spawn tail has the matching `herding.agent_command`
seam. When the key is absent, invalid, or empty, the command built is
byte-equivalent to the pre-existing hardcoded `claude -p ... --model sonnet
--max-turns ... --allowedTools ...` invocation — a project with no config
change sees no behavior change at all. A codex adapter example is documented
purely as an illustration of the seam; full codex-native herding (its own event
loop and pane protocol) stays out of scope (D4). None of enable/disable/status,
the dispatch interlock, or the merge owner-gesture change.

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
  Business Rules). It is confined to its own worktree and branch until a merge.

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
- **A control pane narrowed too far** stalls silently every interval — the exact failure the whole
  cockpit exists to end. This is why the control surface is enumerated against measured actions, and
  why it is documented to grow when a role gains a command, rather than being set to "read-only".
- **A worktree finished by bee but never merged** is the merge gesture's normal input; nothing
  retires it automatically, by design.

## Open Gaps

- **The four-slot concurrency cap is not yet mechanical** (see Edge Cases). Until it is, a spawned
  agent that fails to self-name can lead the loop to over-spawn.
- **The classifier reads the backlog row, not the work.** It vouches for an item from its one-line
  description, never opening the feature's own context. Reading the real work is the honest form of
  the safety check and is not yet built — the interlock (R3) is the compensating control meanwhile.
- **The dependency on herdr's JSON shapes is unpinned** — no capability probe, so an upstream shape
  change degrades to the silent-stall class.
- **The supervised acceptance cycle (R7) is owner-run and outstanding** for this repo.

## Pointers (implementation)

- The skill and its three roles: `skills/bee-herding/SKILL.md`; the loop driver
  `bee herding control-loop`
  (`packages/bee-rs/crates/bee/src/herding/control_loop.rs`); the one-shot
  `skills/bee-herding/scripts/bootstrap-cockpit.sh`.
- The `herding` command group — `classify-lane`, `interlock`, `command-template`,
  `herdr-result`, `herdr-pane-id`, `wave`, `occupancy`, `record-worker` and
  `control-loop`, the nine verbs the current binary actually
  serves — is implemented in `packages/bee-rs/crates/bee/src/herding.rs`, dispatched
  from `packages/bee-rs/crates/bee/src/router.rs`, and listed (with `enable`,
  `disable` and `status` marked `unavailable`) in the command catalog
  `packages/bee-rs/crates/bee/src/catalog.rs`. `enable`, `disable` and `status` are
  not among the nine live verbs and refuse by name; the manual `touch`/`rm` marker
  gesture is their only live form (see Data Dictionary). Test coverage is inline:
  the `#[cfg(test)] mod tests` block in `herding.rs`.
- The isolation the working agents depend on is `worktree-parallelism`; the guarded landing is that
  area's merge gate.
