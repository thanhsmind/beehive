# slp-lead-recovery — hat wave synthesis (plan-step consult)

Five seats, high-risk. Ran at the REVIEW tier (opus), not the configured advisor
tier (fable) — all five fable seats died on an HTTP 429 account limit; deviation
recorded as a decision. All five seats returned; no seat dropped.

## Verdict

The drafted auto-spawn shape does not survive its own plan check. The user chose
the cheaper shape after being shown this synthesis. Every spawn-specific locked
decision (D1 executor split, D3 one-successor cap, D6 ACK, D8 executor guard) is
superseded; the observer-only half (D2 evidence standard, D4 durable bundle,
D5 never touch the old lead, D7 default off, D9 provenance) survives.

## What each seat found

**hat-value — RED FLAG.** The triggering event has never happened here: 155
observation ticks carry 153 `silence`, one `big-decision`, one `struggling-loop`;
218 session records show 207 clean closes and zero `dead_at` marks (the only file
carrying one is the live session that wrote it). The two records that would match
the trigger are both false positives — `guide-pages` is `run_state:
awaiting-approval` waiting on a human UAT gate, `prompt-work-record` is
`run_state: done`, merged. Both sit at phase `swarming`, which is non-terminal, so
D8's lane check passes for both. Guard precision on the only real data: 0/2.
Verified independently by the leader.

**hat-facts-gaps — 6 BLOCKERS.** Chief among them: D8's "non-empty evidence field"
has no field to live in (`Intervention` carries no evidence/bundle member, and the
only carrier `question` is capped at 2 sentences / 500 chars with an injection
screen); nothing in the repo ever runs `recovery scan`, whose sole non-test caller
sets the `dead` mark the whole trigger depends on; a row addressed to a dead
session is never delivered, because `pending_for` filters strictly on
`target_session` at a turn boundary a corpse never reaches; the `control_command`
config template can replace the whole argv and bypass `SUPERVISOR_ALLOWED_TOOLS`,
so "the supervisor can never spawn" is not enforced by the cited bytes.

**hat-risks — 12 risks, 3 irreversible paths.** The cap keys on
`(target_session, point_key)`, so a dead successor mints a new id and the cap has
never seen it — D3's "one successor, ever" is unenforced and a spawn CHAIN is
reachable. "Dead" means "silent for 900s", and this repo holds real single tool
calls of 16.7 and 12.5 minutes, so a live lead mid-`cargo test` qualifies. The
crash-candidate row carries no `cwd`, so the successor would spawn into the
dispatch loop's own directory — main — not the dead lead's worktree: certain, not
probable. Nothing bounds overnight spend.

**hat-user-impact — 12 gaps.** The one machine action that spends money renders as
a pending question, never as "this already happened"; the single next-action line
prints a literal `<id>` placeholder; feature-off is indistinguishable from
bee-not-noticing; a user who simply closes the laptop without `bee supervisor away`
gets no report, no notification, and an unexplained new pane.

**hat-alternatives — cheaper shape found.** Adding a signal to `KNOWN_SIGNALS`
turns the currently-green `the_shipped_prompt_pins_the_record_verbs_own_closed_sets`
red unless the prompt lands in the same cell, so the vocabulary and prompt halves
are one cell, not two. Confirmed by the leader at
`herding/control_loop.rs:1064-1075`.

## The shape that replaced it

Detection already works with zero new code: `bee status --json` carries
`recovery.candidates` with every field the note needs, and `bee status` is already
inside `SUPERVISOR_ALLOWED_TOOLS`. The supervisor has in fact already written about
a stale candidate in prose. What is missing is a name for the signal, a rank so it
is not truncated below a struggling-loop, and a resume line the human can act on.

That is one cell, plus one fix-first cell for a pre-existing red on the base.
