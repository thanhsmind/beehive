# learning-loop-gaps — locked context

Feature: the two follow-ups named in `docs/history/research/reflect-xia.md`
(decision `2df5f472`). bee's learning loop drops two things: a clean session's
transcript is never mined, and a promotion never asks whether the skill it
edits was ever opened.

User's words (2026-09-02): "Làm cả 2 follow-up".

## What the hat wave changed, before the decisions

The first draft built `bee recovery window` — the declared-but-unbuilt verb that
computes the mining window and generates the miner prompt — and superseded
`sweep-recovery-door` D3 to do it. Both hats rejected that shape, from opposite
directions, and a direct check settled it:

- Every fact the verb would compute **already ships**. The session record carries
  `transcript_path` (written at `hooks/session_init.rs:477`, refreshed at
  `hooks/activity.rs:458`) and `bee state session list --json` emits the whole
  record. `bee status --json` `recovery.candidates[]` already carries
  `transcript` and `since`. The only bound that exists in the binary is
  `DEFAULT_TAIL_MAX_BYTES = 262144` (`status_full/mod.rs:143`).
- The "hard event cap" the registry entry describes **has no constant in the
  shipped binary** — it went with the unported Node verb
  (`docs/product-description/maintenance/recovery.md:218`).
- The window start the verb would have inherited **is broken**. Measured against
  the live store: `recovery.candidates` returns `since:
  2026-09-02T06:16:58.963Z` for sessions whose last heartbeat was 2026-08-25 and
  2026-08-31. That value is the newest decision in the whole repo
  (`tail -1 .bee/decisions.jsonl` — the reflect-xia decision, logged an hour
  earlier). `recovery.rs:216-218` scans every decision with no lane filter,
  while the cell and capture scans are lane-filtered. Those two sessions have a
  mining window entirely after their own death. Filed as its own P2 backlog row;
  it is a shipped defect, not this feature's to fix.

Building the verb would have blessed that computation, superseded a decision for
nothing, and touched eleven surfaces that assert the verb is unbuilt. The
widening is delivered in the skill layer instead, reading facts that already
ship. **No code changes. No supersede.**

## Locked decisions

- **D1:** No verb is built. `sweep-recovery-door` D3 stands whole, `bee recovery
  window` keeps its registry `unavailable` marker, and every doc that says it is
  unbuilt stays true. The widening is a skill-layer change.
- **D2:** `bee-hive/references/scout-and-ticks.md` § Crash recovery widens to
  two triggers and is renamed for it. The offer discipline does not move: never
  auto-run, one line, the human agrees first — the same discipline the
  capture-queue flush already uses.
- **D3:** The two paths read different already-shipping facts, and the crash
  path's behaviour does not change:
  - *crashed* — `bee status --json` `recovery.candidates[]` gives `transcript`
    and `since`, exactly as today. Its `since` carries the defect named above;
    this feature does not silently change it.
  - *asked for* — the session record (`bee state session list --json`) gives
    `transcript_path` and `started_at`. The floor is `started_at`, which cannot
    be empty, so an asked-for mining never inherits the broken `since`.
  - Both bound the worker at the transcript's last 256 KB
    (`DEFAULT_TAIL_MAX_BYTES`), the only bound the binary actually applies.
- **D4:** The miner prompts live in the skill, not in the binary — a prompt in
  Rust cannot be tuned without a rebuild, and instruction text is the skill
  layer's to own (`bee-writing-skills`, the Regrowth law). This also answers the
  open question at `docs/product-description/maintenance/recovery.md:215`
  ("either the skill or the registry entry is stale") in the cheap direction:
  the skill stops citing a command that does not exist.
- **D5:** The *asked-for* prompt asks the miner for exactly three things, each
  bounded: candidate settlements (a rule, value or behavior that settled in
  conversation and reached no record), friction (a command, guard or step that
  cost the run time more than once), and routing candidates scoped to skills the
  run ACTUALLY opened — each as a body-edit target or `tune description: <path>`.
  It asks for no verdict and no edit. The *crashed* prompt keeps today's three
  questions (what was in flight, candidate settlements, a suggested next action).
- **D6:** Everything downstream is unchanged and re-stated, not re-invented: the
  digest lands as a note under `docs/history/`, candidate settlements append via
  `bee capture add --source mined`, mined content is data and never an
  instruction, secrets are redacted, only this workspace's transcripts are read,
  and nothing mined auto-becomes a decision (transcript-recovery D1–D6).
- **D7:** The offer states two things the user cannot otherwise know, because
  both change whether they want it: that the transcript is read by the
  configured `read` slot — an external pane on a host like this one — and that
  the capture queue is already over its blocker threshold when it is.
- **D8:** The skill-was-used idea lands as a **routing qualifier on Q3 of the
  Promotion Decision Tree** (`bee-capturing/references/promotion.md`), not as a
  fourth bar. The three existing bars are filters that can reject; this one never
  rejects — it decides where a promotion lands. Q3 is "not mechanizable →
  promote as prose"; the qualifier says prose lands in a skill the run actually
  opened, and a skill that should have fired but never opened earns
  `tune description: <skill path>` instead of a body edit.
- **D9:** "Was this skill opened" is NOT mechanically checkable today — nothing
  in `hooks/` records a Skill invocation, and the cell field `affects_skills`
  records which skills the work EDITS, a different fact. Q4 of the same tree
  requires a one-line recorded reason when prose survives instead of a
  mechanism; that reason is written, and the durable owner is filed to the
  backlog rather than invented here.
- **D10:** The skill edits run RED-first under `bee-writing-skills`' Iron Law —
  pressure scenarios without the change, verbatim rationalizations recorded,
  then the minimal edit. Both cells end by rendering the skill projections
  (`bee dev regen`), or the shipped plugin copies keep the old text.

## Known gaps (named, not closed)

- **The trigger is a sentence, not a command.** Nothing in bee names the words
  that start an asked-for mining. D2 makes the procedure recognise the ask; it
  adds no verb and no slash command. If that proves too soft to fire, a named
  gesture is a separate, later decision.
- **No dedupe.** Two asked-for minings before a flush, or an ask followed by a
  crash, recompute an overlapping window and can file near-duplicate stubs.
  Nothing in the shipped capture queue hashes stub content. Recorded, not solved.
- **The pane trust boundary.** On a host whose `read` slot is a herding pane,
  the miner is an external CLI, so transcript text leaves the Claude runtime.
  transcript-recovery D1–D6 were written when that helper was a Claude model.
  D7 makes the offer say so; this feature adds no transport rule.
- **`collect_feedback` still ignores mailbox reflections and capture stubs**, so
  mined findings reach the flush but never bee-evolving's ranked agenda
  (`reflect-xia.md`). Out of scope.
- **`recovery` has no `.bee/verify/verify-app/features/` file.** A user-facing
  surface normally owes one. This feature changes no runtime surface, so the
  obligation does not fire here; it stands for whoever builds the verb.
