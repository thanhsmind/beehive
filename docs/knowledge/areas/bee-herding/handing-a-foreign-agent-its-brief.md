---
type: bee.area
title: "Bee Herding — handing a foreign agent its brief, and knowing it arrived"
description: "The mailbox channel a bee-ignorant worker is briefed over, the standalone-executor contract that keeps it bee-ignorant, and the delivery receipt rule: only a state transition the agent itself caused counts as evidence it received anything."
timestamp: 2026-08-20
bee:
  id: bee-herding-handing-a-foreign-agent-its-brief
  lifecycle: active
  areas: [bee-herding]
  required_context: [areas/bee-herding/the-run-verb-and-worker-outcomes.md]
  decisions: ["herding-executor D3 (file mailbox is the completion signal)", "herding-executor D4 (worker stays bee-ignorant, orchestrator owns bee bookkeeping)", "herding-brief-file D1 (the brief persists as brief-N.txt behind a one-line pointer)", "herding-run-ready-wait D1 (readiness is observed before the send)", "herding-start-retry D1 (agent start retries through a booting shell)", "herding-prompt-verify D1 (bounded resends, never a silent proceed)", "herding-receipt-source D1 (superseded: the receipt no longer reads pane text)", "herding-worker-standalone D1-D3 (standalone-executor contract, the worker env marker, hooks silent under it)"]
  sources: [docs/history/herding-executor/CONTEXT.md, docs/history/herding-brief-file/CONTEXT.md, docs/history/herding-worker-standalone/CONTEXT.md, "live smoke smoke-agy-delivery-1/-2/-3", "live case job hws-1-r1"]
  authoritative_for: "bee-herding: the brief mailbox, the standalone-worker contract, and delivery receipts"
---

# Bee Herding — handing a foreign agent its brief, and knowing it arrived

The worker on the other end of a herded pane has never seen this tool. Everything
it needs arrives in one written brief, and every step of handing it over verifies
rather than trusts a flag (herding-executor arc, live-proven).

## The mailbox is the channel

**Mailbox** — `.bee/mailbox/<job-id>/`, the file channel `bee herding run` uses
to talk to a bee-ignorant worker: `job.json` (the written brief), round-numbered
`brief-N.txt` (the brief as the worker reads it, persisted rather than injected),
round-numbered `result-N.json`, and `log.txt`. Every write is staged
tmp-then-rename, so a result file's appearance under its final name IS the
completion signal — never the pane's screen (herding-executor D3).

The brief travels only as a mailbox file behind a ONE-LINE POINTER — never raw
argv, never a multi-line injected prompt. Two independent reasons force this: a
long brief cannot be encoded as a start argument at all, and at least one agent
kind silently drops a multi-line injected prompt even when idle.

The pane start retries through a booting shell, and readiness is observed before
the send.

## The Expertise section — briefed like a leader, still outside the workflow

The dispatcher may hand the worker an **Expertise section** in the brief
(worker-brief-expertise D1–D3): `bee herding run --expertise` takes one entry
per line, `<path> :: <purpose> :: <read-to>`, and `render_brief` renders each as
"read this file, here is why" between the Task and Working-directory sections.
Zero entries render nothing — the brief stays byte-identical. Entries point at
bee's own skill references and knowledge files, picked by the dispatcher's
judgment per task, never auto-derived (D2). `job.json` persists the entries so a
`--continue` round keeps them; a fresh flag wins.

This forced a rescope of the opening clause: the worker ignores **workflow
participation** (gates, cells, claims, state) — it still never runs a bee
command and never writes under `.bee/` beyond its result file — but the
Expertise-listed files are explicitly its to read (D3). Workflow-ignorance is
not expertise-denial. The same entry shape reaches Task-tool workers through
`bee dispatch prepare --expertise` and the worker-cell prompt's conditional
Expertise block (D4).

## A delivery receipt is a state transition the agent itself caused

This is the "state-receipt delivery", and it replaced an earlier pane-text check.

A pane ECHOES the send's own keystrokes while the agent is still booting, so any
receipt that looks for the sent text coming back confirms nothing but its own
typing — it passes exactly when delivery failed. The receipt is therefore the
agent's own reported state moving into working — as a TRANSITION off a per-send
baseline, never a level read — or the round's result file appearing.

A status LEVEL is not a transition (herding-pointer-delivery D1, narrowing
herding-receipt-state D1, tripled live 2026-08-21): a booting agent flaps
`working` before it accepts input, so a receipt that merely reads
`working` — or a stale `done` from a prior round — receipts a swallowed
pointer, the bounded re-send loop concludes falsely, and the run sits to
ceiling on a brief no agent ever saw. The ready gate is idle-only for the same
reason. The fix samples the status immediately before EACH send and counts only
`not-working → working` (or the result file) as delivery.

Proven the hard way in live smoke: two runs whose brief was silently lost still
satisfied the text check, and only the run that watched for a state change
completed end to end.

The general shape — an echo, a mirror, a write-through cache — is that any
confirmation an actor can produce by itself is not evidence the other side
received anything.

Waiting for that receipt is bounded, not hopeful: the pointer is re-sent a fixed
number of times, each attempt polling the agent's state a few times before the
next, and the pointer is idempotent so a duplicate send costs nothing. Running
out of attempts is a typed failure that says the prompt was never accepted — it
never becomes a silent decision to wait anyway.

Failing to get that receipt splits by WHEN it failed, and the two halves treat
the pane oppositely. A failure BEFORE the agent starts closes the pane — there is
nothing to look at. A ready wait that runs out AFTER the agent started keeps it:
something was running and did not answer, and that screen is the only record of
why.

## The worker is kept bee-ignorant, not merely asked to be

A working agent runs with its permissions fully open, confined to its own
worktree and branch until a merge. Since herding-worker-standalone D1-D3
(2026-08-20) its bee-ignorance is ENFORCED:

- The brief opens with a standalone-executor contract — do the task only; ignore
  the repo's own workflow instructions; never run a `bee` command; the mailbox
  result file is the one permitted write into the tool's own state directory.
- Every fresh spawn exports a worker marker into the pane before `agent start`,
  and that marker wins over any per-agent env value.
- Every hook exits 0 silently under that marker, checked once in the hook
  dispatcher, so a worker session gets zero preamble, zero guards, zero nudges.

The live case that forced it (job hws-1-r1): a worker in this repo read the
repo's own agent instructions, activated the full workflow, and then the write
guard denied its own mailbox result write — the one write its contract exists to
permit.

## Pointers (implementation)

- The mailbox contract it writes and reads is
  `packages/bee-rs/crates/bee/src/herding/mailbox.rs`; the delivery path, receipt
  and resend ceiling are in `packages/bee-rs/crates/bee/src/herding/run.rs`.
- The worker marker is the environment variable `BEE_HERDING_WORKER=1`, exported
  into the pane before `agent start`; the mailbox directory is
  `.bee/mailbox/<job-id>/`. The repo instructions the contract tells the worker to
  ignore are this project's `AGENTS.md` and `CLAUDE.md`; the live case that forced
  the rule was a herded Claude Code worker in this repo (job hws-1-r1).
- The hook-side half of the worker marker is the dispatch-guard rule in
  `areas/hook-runtime/dispatch-guard.md`; the mailbox's write-guard exemption is
  in `areas/hook-runtime/guard-precision-exemptions-and-remedies.md`.
- Operator-facing detail:
  `skills/bee-herding/references/operational-invariants.md` ("Spawn resilience").
