# herding-prompt-stall — locked context

## The report

Handed over from a sibling session, recorded in `.bee/backlog.jsonl` as a P2
`finding` under layer `herding`: three concurrent `bee herding run` calls
against one herd agent, one pane went active and finished, the other two
stalled and were killed. The reporter stacked two suspected effects — a
per-workspace trust prompt the herd entry's auto-approve flag does not cover,
and a possible serialization of the parallel dispatch path — and asked for a
third thing outright: `herding run` cannot tell a pane waiting on a prompt
from a slow worker until a timeout expires.

## What the reproduction actually showed

Live, 2026-08-21, in this repo.

- **Probe B** — ONE `bee herding run --agent agy-flash` into a brand-new,
  untrusted worktree. It succeeded, with no trust prompt at all. The claim
  "a first-time workspace always stalls" does not hold as written.
- **Probe C** — THREE concurrent `bee herding run --agent agy-flash` into the
  same untrusted worktree, with the workspace trust entry removed first.
  Two runs completed. The third, job `trust-par-2`, stalled exactly as
  reported — and again with no trust prompt anywhere in the pane.
- The stalled pane `w4:p1W` sat at an empty agy input prompt, agent banner
  rendered, `agent_status=done`, `interactive_ready=true`, `focused=false`.
  Its `.bee/mailbox/trust-par-2/job.json` nonetheless carried a stamped
  `pane_id` and `kind` — fields written only AFTER `deliver_pointer` returns
  Ok and `record_dispatch` runs. **bee had declared the pointer delivered to a
  pane that never rendered it.**

So the trust prompt is not the mechanism. The mechanism is that bee reads
herdr's agent lifecycle state during the agent's boot window, where it is not
yet stable, and both of bee's gates there misread it.

## The external contract bee was misreading

From `herdr --skill`, the agent lifecycle section, verbatim:

> `idle` means the agent is ready for input and its tab has been seen in the
> focused Herdr UI. `done` is the same underlying idle state after unseen
> background work finishes. Focusing the tab or targeting the pane or agent
> with a focus command marks it seen. CLI reads do not mark it seen.
> `blocked` means Herdr recognized an approval or question UI. `unknown`
> means an agent is present but Herdr cannot classify it confidently.

And on `agent prompt`:

> `agent prompt` atomically submits text and encoded Enter while honoring the
> pane's live bracketed-paste mode. … A prompt sent from a non-working state
> must produce an observed lifecycle change within five seconds. Otherwise
> Herdr returns `agent_prompt_stalled` instead of waiting indefinitely.

bee splits every worker pane with `--no-focus` and only ever reads over the
CLI. By that contract its panes normally report `done`, never `idle`. bee
calls `herdr agent prompt` without `--wait`, so herdr's own five-second
stall detector is switched off. And bee checks `blocked` nowhere.

## Locked decisions

**D1** (`9391e9e8`, supersedes `herding-pointer-delivery D1` / `57a22bfd`) —
bee stops hand-rolling the pointer-delivery receipt. The send becomes herdr's
own atomic submit-and-observe, `herdr agent prompt <job> <text> --wait --until
working --timeout <ms>`, and herdr's `agent_prompt_stalled` IS the delivery
failure, surfaced at once. bee's baseline/transition poll is retired: sampled
right after `agent start` it reads the boot window, where an agy pane flaps
through unknown/working/idle/done, so a boot flap satisfies the transition
test and receipts a pointer the booting TUI discarded.

**D2** (`herding-prompt-stall D2`, narrows `herding-run-ready-wait D1`) —
the ready gate accepts `idle` OR `done`. `done` is the same underlying
ready-for-input state for a tab nobody has looked at, and it is the NORMAL
resting state of a bee worker pane. Prefer `herdr agent wait <job> --until
idle --until done --timeout <ms>` over bee's hand-rolled poll.

**D3** (`herding-prompt-stall D3`) — herdr's `blocked` state is a fast, loud
failure at every bee wait point: the ready gate, pointer delivery, and the
round poll. A blocked pane ends the wait immediately with a typed error
naming the pane id, the tail of its text, and the remedy. bee never burns the
60s ready wait, the 30 pointer resends, or the 900s idle timeout on a question
nobody is going to answer. This is how a per-workspace trust prompt is
covered without bee carrying any agent-specific pattern table.

**D4** (`herding-prompt-stall D4`, narrows D1, decided by the user mid-flight) —
the delivery receipt becomes an ACK the worker WRITES, not a state bee infers.
The rendered brief's first instruction is: before any other step, write
`<mailbox>/ack-<round>.json` atomically — tmp then rename, the gesture the
result file already uses — carrying who took the job: worker nickname, cell id
when there is one, job id, round, the agent's own name, and a `received_at`
timestamp. `deliver_pointer`'s receipt is that ack appearing, or the round's
result appearing for an ultra-fast round. herdr lifecycle state stops being
the receipt entirely; D1's `agent_prompt_stalled` and D3's `blocked` stay as
the fast FAILURE detectors, never as the success signal.

A file the worker wrote is unambiguous: it cannot be faked by a boot flap, it
does not depend on which tab a human last looked at, and it names WHO took the
job — something no lifecycle state carries. It also gives the run its first
heartbeat and makes "was this work ever picked up" answerable from the mailbox
alone.

### Shape addendum for D4

`plan.md` was already frozen by its approved shape gate when D4 settled, and a
`plan-rev` bump would unapprove the execution gate under two in-flight
workers. Recorded here instead, deliberately: the slice gains

**hps-3 — the worker writes an ack file and that ack is the receipt.**
`herding/mailbox.rs` + `herding/run.rs`, deps `hps-1` (same file). `render_brief`
gains a first-step ack block; `deliver_pointer` takes its receipt from the ack
(or the result file); the ack also counts as a heartbeat. D1's stall and D3's
`blocked` stay exactly as hps-1 builds them — failure detectors, not success
signals.

## Explicitly out of scope

- **A config-driven workspace-trust pre-flight.** D3 makes a trust prompt fail
  in seconds with a named remedy, which is bee's house style for a guard.
  Pre-seeding another tool's trust store from bee is a separate call, and the
  reproduction says it is not what breaks parallel dispatch. The remedy line
  is the deliverable here.
- **The job-id collision.** `bee herding run` defaults its job id to
  `job-<epoch-millis>` with no collision check, so two runs starting in the
  same millisecond share a mailbox, a `job.json`, and a herdr agent name.
  It did not cause this incident (the incident's ids were 1.6s apart) but it
  is a real latent defect; recorded to the backlog, not fixed here.
