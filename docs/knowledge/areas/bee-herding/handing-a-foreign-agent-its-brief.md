---
type: bee.area
title: "Bee Herding — handing a foreign agent its brief, and knowing it arrived"
description: "The mailbox channel a bee-ignorant worker is briefed over, the standalone-executor contract that keeps it bee-ignorant, and the delivery receipt rule: the worker's own ack file is the only evidence the brief was ever received — herdr lifecycle state is a failure detector, never the receipt."
timestamp: 2026-08-20
bee:
  id: bee-herding-handing-a-foreign-agent-its-brief
  lifecycle: active
  areas: [bee-herding]
  required_context: [areas/bee-herding/the-run-verb-and-worker-outcomes.md]
  decisions: ["herding-executor D3 (file mailbox is the completion signal)", "herding-executor D4 (worker stays bee-ignorant, orchestrator owns bee bookkeeping)", "herding-brief-file D1 (the brief persists as brief-N.txt behind a one-line pointer)", "herding-run-ready-wait D1 (readiness is observed before the send; narrowed by herding-prompt-stall D2 — the gate accepts idle OR done, not idle alone)", "herding-start-retry D1 (agent start retries through a booting shell)", "herding-prompt-verify D1 (bounded resends, never a silent proceed; narrowed by herding-prompt-stall D1/D4 — the receipt is now the worker's ack file, and a resend fires only when the agent returns to idle/done with still no ack)", "herding-receipt-source D1 (superseded: the receipt no longer reads pane text)", "herding-worker-standalone D1-D3 (standalone-executor contract, the worker env marker, hooks silent under it)", "herding-prompt-stall D1 (retires the lifecycle-transition receipt; the send is herdr's own atomic submit-and-observe)", "herding-prompt-stall D2 (the ready gate accepts idle or done)", "herding-prompt-stall D3 (a blocked pane ends the wait immediately, at every wait point)", "herding-prompt-stall D4 (the receipt is the worker's own ack file, or the round's result file for an ultra-fast round; a resend fires only on ready-with-no-ack, bounded separately from the ack-wait budget)"]
  sources: [docs/history/herding-executor/CONTEXT.md, docs/history/herding-brief-file/CONTEXT.md, docs/history/herding-worker-standalone/CONTEXT.md, docs/history/herding-prompt-stall/CONTEXT.md, "live smoke smoke-agy-delivery-1/-2/-3", "live case job hws-1-r1"]
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

## A delivery receipt is an artifact the receiver wrote

This is the "receipt-as-artifact" rule. It retired two earlier detection
strategies in turn: first a pane-text echo check, then a herdr lifecycle-state
transition.

A pane ECHOES the send's own keystrokes while the agent is still booting, so any
receipt that looks for the sent text coming back confirms nothing but its own
typing — it passes exactly when delivery failed. The fix that followed chased
the same self-confirmation trap one layer down: it took the receipt as the
agent's own reported state moving into `working`, as a TRANSITION off a
per-send baseline. That claim is now retired (herding-prompt-stall D1):
sampled right after `agent start`, the baseline reads the agent's boot window,
where a pane flaps through unknown/working/idle/done before it is actually
ready to accept input, so the boot flap itself satisfies the transition test.
Proven live on 2026-08-21, job `trust-par-2`: bee stamped `pane_id` into
`job.json` — a field written only after the pointer was declared delivered —
while the pane still sat at an empty, unrendered prompt.

The receipt is now an ACK FILE THE WORKER WRITES (herding-prompt-stall D4):
the rendered brief's first instruction is to write
`<mailbox>/ack-<round>.json` atomically — tmp
then rename, the same gesture the result file already uses — before any other
step, carrying who took the job (worker nickname, cell id when there is one,
job id, round, the agent's own name, a `received_at` timestamp). Delivery is
that ack file appearing, or the round's result file appearing for an
ultra-fast round. herdr lifecycle state is no longer the success signal at
all: a file the worker wrote cannot be faked by a boot flap, and it names WHO
took the job — something no lifecycle state carries.

The ready gate is not idle-only either — that claim is also retired
(herding-prompt-stall D2). herdr defines `idle` as ready-for-input AND the
tab has been seen in the focused Herdr UI, and `done` as the same underlying
ready state for a tab nobody has looked at. CLI reads never mark a tab seen,
so a `--no-focus` bee
worker pane normally reports `done`, never `idle` — and `done` is that pane's
NORMAL resting state, not a failure. The gate accepts `idle` OR `done`.

What ends the wait EARLY, before any ceiling, is now two herdr-native failure
detectors — never success signals: `agent_prompt_stalled` (D1), when a
submission from a non-working state produces no observed lifecycle change
within five seconds, and `blocked` (herding-prompt-stall D3), herdr
recognizing a stuck approval or question UI, checked at every wait point —
the ready gate, pointer delivery, and the round poll. Both end the wait with
a typed failure the moment they
fire; neither one confirms the brief arrived.

The general shape survives, and reads sharper for the retirement: any
confirmation an actor can produce BY ITSELF — an echoed keystroke, its own
boot transition, a status level nobody else wrote — is not evidence the other
side received anything. Only an artifact the receiver wrote is.

Waiting for that receipt is bounded, not hopeful — but it is not a resend
loop either. The pointer goes out ONCE through herdr's own atomic
submit-and-observe, `herdr agent prompt <job> <text> --wait --until working
--timeout <ms>` (herding-prompt-stall D1), and delivery is confirmed by the
worker's ack file appearing, or by the round's result file appearing for an
ultra-fast round that finishes before an ack is ever observed. A `blocked`
pane or herdr's own `agent_prompt_stalled` ends the wait at once, ahead of
any resend. The pointer is idempotent, so a RESEND fires only once the agent
has gone back to a ready state (`idle` or `done`) with STILL no ack — never
on a timer while the agent is still `working` (herding-prompt-stall D4). That
resend path is itself bounded two ways — a fixed count of ready-with-no-ack
resends, and a separate wall-clock ack-wait budget — and exhausting either is
a typed failure that says the prompt was never accepted; it never becomes a
silent decision to wait anyway.

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
  `packages/bee-rs/crates/bee/src/herding/mailbox.rs`; the delivery path
  (`deliver_pointer`), the ack-wait budget, and the stall/blocked errors are in
  `packages/bee-rs/crates/bee/src/herding/run.rs`.
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
