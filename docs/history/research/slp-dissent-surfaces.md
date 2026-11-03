# Research digest — Dissent / StopAndAsk surfaces in bee (SLP ticket 005)

- Date: 2026-08-26 · Tier: advisor (fable) — supersedes the same-day cheap-tier draft
- Context: `docs/discovery/slp-supervisor-lead-peer/tickets/005-dissent-stop-and-ask.md`

## 1. Per surface: what it carries, who reads it, whether reading is forced

### a. `bee cells escalate` — NOT a dissent channel (name collision with the spec)

- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1376-1455` (`set_escalation`, `run_escalate`): "escalate" in bee means **model-tier escalation** — set a flag so the cell runs on the session model. It carries the boolean flag plus an optional `--reason` persisted as `trace.escalation_reason` (:1463), under a 40% per-feature budget (refusal text :1422). No claim, no alternative, no severity, no addressee.
- Read by `bee dispatch prepare` (routing) and `bee status --json` (`role_mix`) — `skills/bee-swarming/references/swarming-reference.md:168-176`, :368 ("Escalation is not a role").
- It is an **orchestrator** verb. The worker contract never grants it; rescue-ladder rung 2 (`skills/bee-swarming/SKILL.md:72-75`) is the orchestrator escalating a blocked cell to itself. Nothing forces anyone to answer the `escalation_reason`.

### b. `bee cells block` + the `[BLOCKED]` result — the only objection path a worker has

- `handlers_close.rs:1102-1143` (`run_block`): writes `status: "blocked"`, `trace.blocked_reason` (one free-text string), an attempt record. No structured fields.
- Worker side, `swarming-reference.md:519-525`, the `[BLOCKED]` result form is the richest carrier today: `Blocker: <conflict | failing verification | ambiguity | locked-decision conflict>` (a closed taxonomy already naming the two SLP triggers), `What happened:`, `What I need next:` — one action, not options.
- Reading IS forced at the transport level (the worker's final message lands in the orchestrator's context) and `SKILL.md:57-59` tells the orchestrator to read the Result form ("never its prose" — where any consider-grade concern would live, so prose concerns are read-banned). Responding is NOT forced per-event: the rescue ladder is skill prose. The one enforced tooth is coarse and late: a blocked cell is non-terminal, so `bee close` refuses/reports it (`verbs/drivers/close.rs:2248-2251`).
- `[BLOCKED]` is **terminal for the worker** — the whole cell aborts. No "hold the related part, continue the rest"; `SKILL.md:152-155` is the explicit anti-StopAndAsk rule: "Never wait silently; never ask a blocking question — you run headless."

### c. The cap report and departures — dissent after the fact

- Cap `--report` is exactly five keys `["outcome","commit","files","tests","deviations"]` (`verbs/cells/finish_support.rs:55-56`). **No concerns field exists.**
- `deviations` are departures `{what, why, kind}` with a CLOSED four-kind set (`verbs/mailbox.rs:226-231`): obstacle / better route / plan wrong about a fact / had to fix something first. `handlers_close.rs:698+` (`departure_door`) refuses an armed cap that states neither a departure nor "followed the plan" — disclosure is forced, but only **after** the worker already deviated unilaterally. "Found a better route" is the spec's `alternative`, executed first and reported later.

### d. `bee state waiting-on set --kind question` — human-facing, session-level

- `verbs/state_group/waiting_on.rs:227-289`; kinds validated in `verbs/workflow_store/record.rs:388-395`; requires a session id (record.rs:402-406; refusal at waiting_on.rs:219-223). A worker could physically run it, but it targets the workflow/default state record — one mark per record, aimed at the **human**, cleared by the user's next prompt. It would collide with the session's own mark; nothing pauses the orchestrator on it. Not a worker→lead channel.

### e. The two mailboxes

- `.bee/human-mailbox/` (`verbs/mailbox.rs:1-70`): per-run letters **to the human**; entries append at cap / feature-close / blocker only (`ENTRY_KINDS` :508-512); letters carry `needs_you[]` `{id, what, blocks}` (:440-448). Caps write `needs_you: Vec::new()` by design (`handlers_close.rs:829-832`). Reading is not forced; the NeedsYou reply surface "does not exist yet" (:441-443).
- `.bee/mailbox/<job-id>/` (`herding/mailbox.rs:20, 95-100, 328-331`): the herding cli-worker back-channel — `brief-N.txt`/`ack-N.json`/`result-N.json` with `status: done|blocked`, and `bee herding run --continue <job-id>` sends the next round (`swarming-reference.md:460-466`; a `blocked` result "is never force-capped"). The only genuine two-way worker↔orchestrator channel, but round-based: the worker exits after writing its result.

### f. Is anyone obligated to respond before work continues?

No, not per-event. Gates obligate the human through hooks plus close doors. The nearest dissent analog is the **judge-debt pattern**: at `standard`/`high-risk`, every `behavior_change` cell owes a `bee cells judge-record` verdict or `bee close` refuses (`SKILL.md:62-64`). Nothing analogous exists for a worker's objection.

## 2. Gap table vs SLP §4.3/§4.4

| SLP field | Nearest bee surface today | Gap |
|---|---|---|
| Dissent.target | `[BLOCKED] <cell-id>`; `trace.blocked_reason` | Cell-granular only; cannot target one instruction/decision inside the order |
| Dissent.claim + reasoning | `What happened:` (swarming-reference.md:521-523); departure `{what, why, kind}` | Blocked = full abort; departure = post-hoc confession, not pre-act objection |
| Dissent.alternative | `What I need next:` (one parent action); departure kind "better route" | No structured alternative offered **before** acting |
| Dissent.severity `blocker\|consider` | Only blocker exists | `consider` has NO carrier: report has no concerns key, prose is read-banned — soft dissent is structurally dropped |
| Lead MUST answer one-of-three, logged | Rescue ladder is prose; blocked cell blocks `bee close` (close.rs:2248-2251) | No verdict record, no enforced answer-before-continue; obligation is coarse and reason-free |
| Escalate to design council | Rung 3 "surface it to the user" | No structure; `bee cells escalate` name already means model-tier — a trap |
| StopAndAsk.boundary_hit | Worker rules: architectural/package changes → `[BLOCKED] with the proposal` (SKILL.md:126-133) | Exists only as whole-cell abort |
| Options 2-3 + trade-offs + leaning | None — `[BLOCKED]` asks one action | Missing entirely |
| MUST NOT continue related part until answered | Worker exits; blocked cell stays non-terminal/unclaimable | Partial-continue semantics missing; SKILL.md:155 is the explicit opposite rule |

## 3. Advisor opinion

The cheapest honest design puts dissent **on the cell, through the CLI, answered at a door** — the only place bee already knows how to make an obligation real. Add `bee cells dissent` writing a structured `{target, claim, alternative, severity}` record to the cell trace, and enforce the answer the way judge-debt already works: `bee close` and `bee worktree merge` refuse while any dissent lacks a recorded `accept|reject|escalate + reason` verdict, and a blocker-severity dissent additionally rides the existing blocked-status machinery so the related work stays unclaimable. Do not build a live mid-flight Q&A channel: native subagents exit when they speak — the herding round-mailbox (`--continue <job-id>`) is the honest StopAndAsk shape, and the `[BLOCKED]` form just needs `options[] + leaning` fields grafted on. Trade-off accepted: the lead can keep running *other* cells while a consider-grade dissent sits unanswered; "MUST NOT continue the related part" is approximated by cell-state, not a paused process — truer to bee's headless-worker reality than the spec's synchronous framing. Naming warning: the SLP "escalate" verb must not touch `bee cells escalate`, which already means model-tier.
