---
name: bee-herdr
description: >-
  Drive the herdr terminal-pane transport correctly: delivering a prompt to an
  agent pane, proving it was submitted, recovering a stall, and reading the
  worker's outcome. Use when sending a task to a tmux/agent pane, when a pane
  shows text typed into the input box but never submitted, when a dispatched
  worker's outcome must be reported, when a pane id from a note or an earlier
  session is about to be targeted, or when tempted to run tmux send-keys,
  paste-buffer, or hand-typed text against an agent pane. Not for cockpit
  roles (bootstrap/dispatch/merge) — that is bee-herding.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: unavailable
      reason: Every rule below routes through the vendored bee binary (`bee herding run`, `bee herding herdr-result`, `bee herding occupancy`); the skill has no raw-tmux path to fall back to.
---

# herdr transport — deliver, prove, recover, read

The pane is a screen. A screen can show your text and still have submitted
nothing. Every rule below exists because "it looks sent" and "it was sent"
are different facts, and only one of them is written down.

## Deliver — one door

- Deliver a prompt to a pane ONLY through `bee herding run --task-file <f>`
  (dispatched via `.bee/bin/bee dispatch prepare`, AGENTS.md's one door).
  `--dry-run` first when the config or agent kind is unproven.
- NEVER `tmux send-keys "$(cat file)"`, `paste-buffer`, or hand-typed text
  into an agent pane: send-keys fires every newline as Enter — line 1
  becomes the prompt, the rest lands as junk prompts — and a composer can
  hold typed text forever without submitting it.
- The run verb owns ready-wait (sends only to `idle`/`done`; a `blocked`
  pane ends the wait — a dialog is answered by a human, never by a
  prompt's first character), submission verification, and bounded resends.
  Never hand-roll any of those.

## Prove — the receipt is a file

- Delivery is proven ONLY by the worker's `ack-N.json` or `result-N.json`
  in `.bee/mailbox/<job-id>/`. Screen echo, boot flap, or a `working`
  status prove typing, never submission.
- Start proof: `job.json` plus `ack-N.json` in the mailbox (and, where the
  host writes one, a `dispatch.jsonl` row under `.bee/logs/`). Absent
  artifacts = not started, whatever the pane shows; a log file the host
  never writes names a gap, never a verdict (AGENTS.md, environment-fact
  rule).

## Recover — a stall is retryable, blind Enter is not

- Text sitting unsubmitted in a composer: never press Enter into it.
  Salvage the text read-only through the pane vocabulary
  (`bee herding pane read <pane> --lines 200`), then re-deliver through
  `bee herding run` — it always opens a fresh pane, so the stalled one is
  never reused. Do not try to clear the composer by keystrokes: the pane
  vocabulary has no key-send verb on purpose. A bee-spawned stalled pane
  is closed (`bee herding pane close <pane>`) once salvaged; a pane a
  human owns is left for the human. The run verb's own resend logic is
  bounded and ack-gated; a hand-sent Enter is fire-and-forget into an
  unverified state.
- Never resend on a timer while the pane's activity record says `working`.

## Read — outcomes, not scrollback

- Read outcomes from `result-N.json` / `report-N.md` in the mailbox, or
  through `bee herding herdr-result <field>`. Scrollback has no `done`
  state to give.
- ENVELOPE TRAP (observed live): `herdr-result` reads
  `{ok, transport, result:{...}}` — piping a bare `result-N.json` in fails
  with "missing result.status". The guard-safe wrap and the working
  invocation, verbatim:
  `jq '{ok:true,transport:"tmux",result:.}' result-1.json | bee herding herdr-result status`
  — the field path is relative to `result` (the verb prefixes `result.`
  itself): ask for `status`, never `result.status`.
- A failed or timed-out run leaves its pane open as forensics; a clean
  result closes it. Test it, don't eyeball it: `bee herding occupancy` —
  an open pane the morning after means not-clean.

## Address — panes move overnight

- A pane id is a position, not a name. Resolve at dispatch time:
  `bee herding pane-id --label <l>` / `bee herding occupancy`; never trust
  yesterday's `w1:p7`, whoever wrote it down.
- Transport is declared, never sniffed: `HERDR_ENV=1` plus a non-empty
  `HERDR_PANE_ID` mark a live herdr transport; `$TMUX` proves nothing.

## Headless

All rules bind unattended. A pane that cannot be verified as submitted is
reported as not-delivered, never assumed; an unreadable result record is a
typed refusal, never a guessed outcome.

## Red Flags — stop and re-read the rule you are about to break

"just hit Enter, the text is already there" · "send-keys is one command,
the verb needs --help first" · "the pane shows my prompt, so it started" ·
"the pane is idle and the output looks fine, report done" · "the manager's
note says w1:p7, checking looks distrustful" · "pipe result-1.json
straight into herdr-result".

Cockpit roles (bootstrap/dispatch/merge), agent registry, and wave
occupancy live in bee-herding — invoke bee-herding, never restate it here.
Mechanism depth: docs/knowledge/areas/bee-herding/
(the-run-verb-and-worker-outcomes.md, handing-a-foreign-agent-its-brief.md).

Transport handled. Invoke bee-hive skill.
