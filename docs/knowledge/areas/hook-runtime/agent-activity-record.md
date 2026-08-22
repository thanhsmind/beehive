---
type: bee.area
title: Hook Runtime — the agent activity record and its read-time signal
description: "The one durable object that says what an agent session is doing right now: what the lifecycle checkpoints write into it, the five states it can hold, which of those states refuse to age, the bounded transition log beside it, the liveness answer nobody stores, and the waiting-on mark it may set but never steal."
timestamp: 2026-08-22
bee:
  id: hook-runtime-agent-activity-record
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: [b17bfa89, 2f782f51, 40c707ba, 2d4e3900, b4f21f29]
  sources: [docs/history/agent-activity-hook/CONTEXT.md, docs/history/agent-activity-hook/plan.md, docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md]
  authoritative_for: "hook-runtime: the agent activity record, its states, and the read-time signal"
---

# Hook Runtime — the agent activity record and its read-time signal

A heartbeat answers "is this session's process still there". It cannot answer
"is the agent working, or has it been waiting on a human for ten minutes" —
and that second question is the one an operator, a dashboard, or a sibling
session actually asks. So the lifecycle checkpoints write one small object
into the session record saying what just happened, and every reader turns
that object into an answer at the moment it reads. Nothing here is a new
authority: the checkpoints observe, they never block, and a reader that
disagrees with the record is reading a record that went quiet, not a session
that died.

## Entry Points & Triggers

- One checkpoint handler receives every observed lifecycle event and switches
  on the event name (decision b17bfa89). The events it observes are the
  prompt submission, the before-tool and after-tool events (including the
  after-tool failure), the permission request, the turn stop, the harness
  notification, and the session end. The child-stop event is deliberately
  **not** among them — a child's stop says nothing about what its parent
  session is doing.
- The handler always exits successfully, whatever it was handed. A malformed,
  empty, or oversized payload writes nothing and leaves one line in the
  capped activity gap log. This is the fail-open frame every checkpoint here
  works under (see `overview.md`); an activity record is evidence, and no
  amount of missing evidence may end a turn.
- Readers trigger on their own: the session listing and the status projection
  each derive liveness when they render, from the record they already hold.

## Data Dictionary

| Element | Meaning |
|---|---|
| activity record | The `activity` object on a session record: the single durable statement of what that session was last observed doing. One per session, overwritten in place, never appended to. |
| state | The activity record's headline value — one of five: `working`, `waiting_input`, `blocked`, `idle`, `exited`. |
| turn boundary | A prompt submission or a turn stop: the two events that mean the conversation itself moved, and therefore the only events that may clear a sticky state. |
| sticky state | `waiting_input` or `blocked` — a state that does not expire with time and is cleared only by a specific, named event. |
| transition log | The bounded `.activity.jsonl` file beside the session record, holding the recent state transitions in order. |
| signal | The read-time liveness answer — `live` or `no_signal` — computed by each reader from the activity record's own timestamp. Never a stored field. |
| waiting-on mark | The session's existing "this session is waiting on the human" mark, which the checkpoints may set and clear under one narrow rule, and which an agent may always set for itself. |

The activity record's fields (decision 2f782f51):

| Field | Meaning |
|---|---|
| `state` | One of the five states below. |
| `event` | The lifecycle event name that produced this state — so a reader can tell `working` from a before-tool event apart from `working` from a prompt. |
| `tool_name` | Present when the event named a tool. Also the fallback identity for clearing `blocked`. |
| `tool_use_id` | Present when the event carried one. The primary identity for clearing `blocked`. |
| `at` | The ISO-8601 UTC instant the checkpoint observed the event. The only input the signal rule reads. |
| `pane` | The terminal pane the session runs in, when known — so an operator can go look at it. |
| `cwd` | The working directory the session runs in — which, for a worktree session, is how a reader tells two sessions of the same project apart. |
| `waiting_on_set_by_hook` | The marker that makes the waiting-on rule below safe: it records that the current mark came from a checkpoint, not from the agent. |

## Behaviors & Operations

**B18 — Every observed lifecycle event writes one activity record.** The
handler maps the event to a state, writes the whole `activity` object onto
`.bee/sessions/<session_id>.json`, and appends the transition beside it. A
session record that does not exist yet is created minimally rather than
skipped: an agent whose very first observed event is a permission prompt is
still a session somebody needs to see (decision 2f782f51).

**B19 — Five states, and each one is a claim about the agent, not about the
process.**

| State | What it claims |
|---|---|
| `working` | The agent is running: it submitted a prompt, or it is about to run a tool, or a tool just returned. |
| `waiting_input` | The agent asked the human something and stopped for the answer. |
| `blocked` | The agent is stopped at a permission prompt — it wants to do something and needs a human to allow it. |
| `idle` | The turn ended. The agent is present and owes nothing. |
| `exited` | The session ended. |

`waiting_input` and `blocked` are both "stopped, waiting on a person", and
they are kept apart on purpose: one is answered by typing, the other by
approving. A reader that collapses them into one "needs you" state throws
away exactly the distinction that tells an operator which button to reach for
(decision 40c707ba).

**B20 — The event-to-state mapping, and what each event may clear
(decision 40c707ba).**

| Event | State it writes | What it clears |
|---|---|---|
| prompt submission | `working` | Turn boundary: clears any sticky state and any checkpoint-set waiting-on mark. |
| before-tool | `working` | Nothing. |
| after-tool / after-tool failure | `working` | Clears `blocked` only when it names the same `tool_use_id` — or, with no id on either side, the same `tool_name`. |
| permission request | `blocked` | Nothing. Sets the waiting-on mark (gate). |
| harness notification saying the agent needs input | `waiting_input` | Nothing. Sets the waiting-on mark (question). |
| turn stop | `idle` | Turn boundary: clears any sticky state and any checkpoint-set waiting-on mark. |
| session end | `exited` | — but not for an end reason that is a restart of the same conversation rather than its finish. |

**B21 — A sticky state does not age.** `waiting_input` and `blocked` stay put
until the event that clears them arrives. Time alone never moves them, and
neither does an unrelated tool event: an after-tool event for a *different*
tool leaves `blocked` exactly where it was. This is the whole point of the
pair — an agent that has been waiting on a human for an hour must still read
as waiting, not decay into something softer (decision 40c707ba).

**B22 — The transition log is bounded at fifty.** Each transition is appended
to `.bee/sessions/<session_id>.activity.jsonl`, and the file is trimmed
atomically to the most recent fifty lines. It is history for reading a
session back — "it was blocked, then a tool ran, then it went quiet" — and no
decision is ever taken from it. Fifty is a cap, not a promise of retention.
The file is invisible to every session enumerator in bee, all of which match
on `.json` (decision 2f782f51).

**B23 — Liveness is computed at read time and never stored.** Each reader
turns the activity record into a `signal` when it renders (decision
2d4e3900):

- `live` — the record's `at` is inside the ninety-second window.
- `no_signal` — there is no activity record, or its `at` is at least ninety
  seconds old, or its `at` cannot be read at all.
- *no signal at all* — the session's own status says it is finished
  (`dead` or `closed`). There is nothing left to be live about, so readers
  emit an empty value rather than the misleading `no_signal`.

`no_signal` means "this record says nothing about right now". It does not
mean the agent is gone; the heartbeat is what answers that, and the two ages
are read independently. The ninety-second window is this rule's own constant
and touches no heartbeat constant.

**B24 — The waiting-on mark may be set by a checkpoint and is never stolen
from the agent (decision b4f21f29).** Entering `waiting_input` or `blocked`
sets the session's waiting-on mark — question for the first, gate for the
second — so that a dashboard or a sibling session reads "waiting on you"
instead of "idle" without the agent having to remember to say so. Two limits
make that safe: a turn boundary clears a mark the checkpoint set, and a mark
the *agent* set is never overwritten or cleared by a checkpoint. The agent's
own statement about what it is waiting for always outranks the harness's
guess.

## Business Rules

- R18 — The activity record is written only by bee's own checkpoint handler.
  One writer into `.bee/`, so a reader never has to reconcile two products'
  ideas of the same session (decision b17bfa89).
- R19 — `signal` is derived, never persisted. No writer puts it on a session
  record; every reader computes it from `at` against its own clock. A stored
  liveness value would be a stale claim about the present the moment it
  landed (decision 2d4e3900).
- R20 — Both readers derive `signal` from the same activity object with the
  same rule. `bee state session list --json` carries `signal` on every
  record; the worker rows of `bee status --json` carry the recorded
  `activity` object verbatim plus the `signal` derived from it. Two readers,
  one rule — a reader that computed its own window would be a second
  definition of "live" (decision 2d4e3900).
- R21 — The activity record holds observation, never content. Event names,
  tool names, identifiers, timestamps, a pane and a working directory —
  never prompt text, tool input, tool output, or credentials. It inherits the
  same content-free discipline the passive usage log runs under (see
  `child-agent-attribution-and-audit.md`).

## Edge Cases Settled

- **An after-tool event for the wrong tool does not unblock.** Clearing
  `blocked` requires the same `tool_use_id`, or the same `tool_name` when
  neither side carries an id. Anything else leaves the state alone — the
  agent really is still stopped at that prompt.
- **A session end that is not an ending.** Some end reasons mean the same
  conversation is restarting rather than finishing. Those do not write
  `exited`; treating them as the session's death would retire a session that
  is about to keep working.
- **The session file does not exist yet.** It is created minimally instead of
  the event being dropped, so the first thing a fresh session does is still
  visible.
- **`at` is unreadable.** It reads as `no_signal`. The field is an ISO-8601
  stamp bee's own checkpoint wrote, so an unreadable one is a damaged record
  — and a damaged record is never evidence of life.
- **A finished session with a fresh activity record.** Status decides first:
  `dead` or `closed` yields no signal at all, whatever the timestamp says.
- **The fifty-first transition.** The log is trimmed atomically, so a
  concurrent reader sees either the pre-trim or the post-trim file, never a
  half-written one.

## Open Gaps

- Only the first runtime's lifecycle events are observed today. The other
  runtimes have no equivalent event set wired, so their sessions carry no
  activity record and read as `no_signal` — correct by this concept's own
  rule, but it is an absence of coverage rather than an absence of activity.
- Nothing sweeps a `waiting_input` or `blocked` state left behind by a
  session that vanished without a turn boundary. Its heartbeat going stale,
  and then the session being swept, is what retires it.
