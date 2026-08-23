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
  decisions: ["herding-executor D3 (file mailbox is the completion signal)", "herding-executor D4 (worker stays bee-ignorant, orchestrator owns bee bookkeeping)", "herding-brief-file D1 (the brief persists as brief-N.txt behind a one-line pointer)", "herding-run-ready-wait D1 (readiness is observed before the send; narrowed by herding-prompt-stall D2 — the gate accepts idle OR done, not idle alone)", "herding-start-retry D1 (agent start retries through a booting shell)", "herding-prompt-verify D1 (bounded resends, never a silent proceed; narrowed by herding-prompt-stall D1/D4 — the receipt is now the worker's ack file, and a resend fires only when the agent returns to idle/done with still no ack)", "herding-receipt-source D1 (superseded: the receipt no longer reads pane text)", "herding-worker-standalone D1-D3 (standalone-executor contract, the worker env marker, hooks silent under it)", "herding-prompt-stall D1 (retires the lifecycle-transition receipt; the send is herdr's own atomic submit-and-observe)", "herding-prompt-stall D2 (the ready gate accepts idle or done)", "herding-prompt-stall D3 (a blocked pane ends the wait immediately, at every wait point)", "herding-prompt-stall D4 (the receipt is the worker's own ack file, or the round's result file for an ultra-fast round; a resend fires only on ready-with-no-ack, bounded separately from the ack-wait budget)", "herding-prompt-stall D5 (corrects D3: blocked does not cover a trust dialog; a give-up wait reads the pane for a confirmation cue instead)", "herding-prompt-stall D6 (narrows D1: a stalled submission is retryable, not an immediate delivery failure)"]
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

## The worker's own hook reports its state into the mailbox

A herded pane silences every bee hook — except one. Since herding-activity-hook
D1 (2026-08-23) the `activity` hook still runs under `BEE_HERDING_WORKER=1`;
every guard, preamble and nudge hook keeps exiting 0 before stdin is read.
`activity` never denies and never prints, so letting it through widens nothing
the worker can touch.

When the pane also carries `BEE_HERDING_JOB_ID` (exported with the worker
marker at fresh spawn), the hook writes **`<mailbox>/activity.json`** —
tmp-then-rename, the same gesture as ack and result — instead of
`.bee/sessions/` (D2). The record is the sessions-record shape plus `job_id`
and `round`: `{state, event, tool_name?, tool_use_id?, at (RFC3339), job_id,
round}`. The hook reads `round` as the highest `brief-N.txt` present in the
mailbox, never from an env var — the export fires once at spawn and would go
stale across a `--continue` round. The state vocabulary and the
same-`tool_use_id` unblock rule are `hooks/activity.rs`'s, unchanged.

The run verb reads that record BEFORE the screen classifier at all three wait
points — the ready gate, pointer delivery, and the round poll (D3): `blocked`
or `waiting_input` ends the wait as blocked; `working` satisfies the
submit-observed check so a stalled-looking submission is not resent. Two
fences keep a stale record from steering: a `round` below the current round is
ignored (the round is the launch-id fence), and an `at` older than
`ACTIVITY_FRESHNESS_SECS` (120 s) is ignored; either way the answer falls back
to the screen classifier, exactly today's behavior. Agent kinds that install no
hooks never write the record and keep the screen path — the hook is an upgrade,
never a requirement. `ack-N.json` and `result-N.json` remain the only truth
for delivered and done.

Why this exists: the screen read missed a trust dialog live (the `blocked`
detector saw `idle`, the full ready-wait burned). The agent's own
`PermissionRequest` hook is exact where a screen regex guesses. Distilled from
agent-orchestrator (D4, `docs/history/research/agent-orchestrator-mailbox-distill.md`);
only its hook return channel was adopted.

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
pane ends the wait at once, ahead of any resend; a STALLED submission does
not (herding-prompt-stall D6, narrowing D1). The pointer is idempotent, so a RESEND fires only once the agent
has gone back to a ready state (`idle` or `done`) with STILL no ack — never
on a timer while the agent is still `working` (herding-prompt-stall D4). That
resend path is itself bounded two ways — a fixed count of ready-with-no-ack
resends, and a separate wall-clock ack-wait budget — and exhausting either is
a typed failure that says the prompt was never accepted; it never becomes a
silent decision to wait anyway.

Two facts about that single submission are easy to get wrong, and both were
measured live rather than reasoned about.

bee's own deadline on the submit-and-observe send must comfortably outlast the
herd tool's internal stall detector. Set to the same instant, the two race,
bee's client-side deadline wins, and the caller sees a bare transport timeout
before the detector ever fires or the agent ever settles. On one healthy pane a
five-second window returned a timeout while a twenty-second window on the SAME
pane returned a working observation and the brief landed. A timeout reply is
also not a stall: a timeout means the submission WAS made, so it falls through
to the ack wait exactly as a successful send does — never aborting delivery,
never resending blind (herding-prompt-stall, cell hps-11).

A stalled submission is a RETRYABLE outcome, not an immediate delivery
failure. A stall only means the herd tool observed no state change from that
one submission; for an agent it already reports as ready but whose interface
has not finished drawing, that is the expected transient — measured live, the
identical submission typed by hand seconds later on the same pane returned
working at once. A stall therefore feeds the same bounded retry the
ready-with-no-ack path uses, under the same two bounds, and becomes terminal
only when a bound runs out, with a distinct failure saying the agent never
took the text at all — kept apart from the failure that says the agent took it
and never confirmed it. A blocked pane and a transport error stay immediate
(herding-prompt-stall D6, cell hps-14).

Failing to get that receipt splits by WHEN it failed, and the two halves treat
the pane oppositely. A failure BEFORE the agent starts closes the pane — there is
nothing to look at. A ready wait that runs out AFTER the agent started keeps it:
something was running and did not answer, and that screen is the only record of
why.

## A give-up diagnosis reads the pane, and the trust-prompt theory is retired

A first working theory for a 2026-08-21 parallel-dispatch stall was a
per-workspace trust question the herd entry's auto-approve flag does not
reach. **That diagnosis was WRONG for the incident it was raised against.**
Reproduced live: the stall happened again with no trust prompt anywhere on
the pane — the actual mechanism was bee reading the herd agent's lifecycle
state during its BOOT window, where it is not yet stable, covered above (the
receipt-as-artifact rule and the `idle`-or-`done` ready gate). A later reader
should not re-derive the trust-prompt theory; it does not explain that
stall.

A related fact turned out true anyway, on a separate probe: a trust dialog
is a real stall shape, and `blocked` — the failure detector that was
supposed to catch any approval-or-question UI — does NOT reliably cover it.
Live proof: three concurrent runs into a genuinely untrusted workspace all
sat at a trust dialog while the herd tool reported the agent `idle`, never
`blocked`, and bee burned its full ready-wait ceiling before failing with a
generic timeout message that named nothing on screen.

Two consequences followed. First, a herd entry may declare and pre-seed the
foreign tool's own trust store so the question never appears at all — the
declaration lives in `agent-resolution-and-spawn-commands.md` ("A herd
entry may declare the foreign tool's own trust store"). Second, every wait
that gives up — the ready gate, the ack budget, the delivery bound — now
reads the pane on its way out, and when the text shows a confirmation cue (a
short yes/no hint, an arrow-key nav footer, a selection caret, or a line
ending in a question mark) it names the matched line and the remedy in its
error instead of generic timeout wording. This diagnosis pass runs only on
an already-failed wait, so a false positive costs nothing and it is never
load-bearing for whether a wait keeps going (herding-prompt-stall D5,
corrects D3's reach). A wait that gives up shows the end of the pane even
when it recognizes nothing on it, so a person always sees the screen the
failure happened on; and a spawn that fails names its own cleanup — how to
look at the pane it left, and the two commands that give back the claim and
the reserved files (herding-reach hrc-3, 2026-08-22).

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
- The two measured submission facts live in the same file: the send deadline is
  `AGENT_PROMPT_TIMEOUT_MS` (20s) with `is_agent_prompt_timeout` telling a
  timeout apart from a stall, pinned against the captured live reply body; the
  retryable stall runs through `is_agent_prompt_stalled` into
  `DeliveryError::NeverDelivered`, kept distinct from
  `DeliveryError::NeverAcked` (cells hps-11, hps-14).
- The activity record: writer is the herded sink in
  `packages/bee-rs/crates/bee/src/hooks/activity.rs` (the marker pass-through
  is in `hooks/mod.rs`); reader is `activity_path` / `parse_activity_text` /
  `ACTIVITY_FRESHNESS_SECS` in `herding/mailbox.rs`, wired through
  `status_with_activity` in `herding/run.rs`.
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
