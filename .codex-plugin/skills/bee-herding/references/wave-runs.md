# Wave runs — briefing several workers at once

A **wave** is one coordinated run over several workers: brief each one, wait on
all of them at the same time, collect what came back, and record the whole run
as a single row. It is a different shape from the dispatch loop. Dispatch
starts ONE worker per iteration and never speaks to it again; a wave speaks to
N workers in one act and waits for all of them.

Reach for a wave when you have several already-running (or ready-to-run) panes
and one question or task for each — a fan-out. Do NOT reach for it to start the
cockpit's ordinary backlog work: that is dispatch's job, behind the owner's
enable marker, and a wave has no interlock, no classifier, and no gate.

```
.bee/bin/bee herding wave --main-root <main-root> --wave-id <id> \
  --worker-settle-ms 15000 --json < workers.json
```

## What a wave does NOT do

**It does not create panes, and it does not create worktrees.** This is the
first thing that surprises a caller. Every worker's `name` must already be a
herdr pane id, or the name of an agent herdr already knows. Splitting a pane
into a worktree is the CALLER's act, exactly as §8 of `role-dispatch.md` does
it:

```
herdr pane split <anchor-pane-id> --direction right --ratio 0.5 \
  --cwd <worktree_path> --no-focus
```

Read `.result.pane.pane_id` from that reply and use it as the worker's `name`.
The `worktree` field in a worker's input is recorded in the ledger row and
nothing more — it does not place the worker anywhere.

## The input

Worker specs arrive on stdin, as a bare JSON array or as `{"workers": [...]}`:

```json
{"workers":[
 {"name":"w4:pG","worktree":"/path/to/worktree-a","task":"the brief for A"},
 {"name":"w4:pH","worktree":"/path/to/worktree-b","task":"the brief for B"}
]}
```

`name` is the pane id (or a known agent name). `task` is the brief. `worktree`
is optional and is ledger bookkeeping only.

## One bad name kills the whole wave

Resolution and start happen for EVERY target before ANY brief is sent. If even
one target fails — a name that is not a pane id and names no known agent, a
pane that cannot be started into — the run stops there, reports every failure
under `resolution_failed`, and sends nothing to anyone. This is deliberate: a
half-briefed fan-out is worse than none. The practical rule is that a wave's
input must be clean; do not mix a speculative name in with good ones to "see
what happens".

Failures AFTER that point are isolated per worker, and each lands in its own
named bucket: refused at pre-flight, changed under us before the send, send
failed, timed out, or unverifiable afterwards.

## Do not read `success` — read the ledger and the panes

**A wave cannot confirm that a worker finished.** The only completion signal
available tracks a pane's attention, not its work, so a worker that took its
brief and answered correctly is still classified `unverifiable_after_send` —
and the run's overall `success` is therefore `false` even when every worker did
its job. That verdict is honest, not broken: claiming success from a signal
that does not mean success is the failure this refuses to commit.

So read two things instead:

```
tail -1 <main-root>/.bee/wave-ledger.jsonl
herdr pane read <pane-id> --lines 30
```

The ledger row carries one entry per worker — name, pane, worktree, brief and
outcome bucket — and is written once per wave. The pane carries what the worker
actually said.

## Which agent a wave starts

The command reads `herding.agent_command` from `<main-root>/.bee/config.json`:
token 0 is the agent kind, the rest are the agent's own arguments, and `{MODEL}`
is substituted per token. Tokens are never joined into one string and re-split,
so a configured command cannot smuggle shell metacharacters through a
placeholder. With the key absent the built command is the byte-equivalent
default. An unrecognised token 0 is a typed error naming that config key — never
a generic start failure.

## Two live facts worth knowing before you run one

- **Starting an agent moves the workspace's focus.** herdr's `agent start`
  carries no do-not-focus flag, unlike `pane split` and `tab create`. Every
  worker a wave starts pulls the view away from wherever you were. Restore it
  afterwards with `herdr tab focus <your own tab_id>`.
- **The ledger only grows.** Nothing sweeps it and no row is marked resolved
  when its wave ends. Occupancy is unaffected — it crosses the ledger's
  unresolved pane ids against the live pane list, so a row whose pane is gone
  stops counting on its own — but the file itself grows without bound.
