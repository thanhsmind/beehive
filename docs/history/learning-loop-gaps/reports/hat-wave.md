# Hat wave — learning-loop-gaps (plan step)

Three seats: `hat-facts-gaps` (opus), `hat-alternatives` (opus),
`hat-user-impact` (sonnet). Opened 2026-09-02 at the plan step. Synthesis by the
leader. The wave **rejected the first draft** and the shape was rebuilt before
plan.md reached its gate-ready bytes.

## The draft the hats critiqued

Three cells: (1) build `bee recovery window --session <id> [--purpose
recover|reflect]`, superseding `sweep-recovery-door` D3; (2) widen
`scout-and-ticks.md` to crashed-OR-asked-for; (3) add a fourth promotion bar to
bee-capturing.

## What the wave changed

**Cell 1 is deleted.** Both hats rejected it, from opposite directions, and a
direct check settled it.

- *alternatives*: every fact the verb would compute already ships — the session
  record's `transcript_path` (`hooks/session_init.rs:477`,
  `hooks/activity.rs:458`), emitted whole by `bee state session list --json`
  (`sessions.rs:204`); `transcript` + `since` on `bee status --json`
  `recovery.candidates[]`; and `DEFAULT_TAIL_MAX_BYTES` as the only bound the
  binary applies. The "hard event cap" the registry describes has no constant in
  the shipped binary — `maintenance/recovery.md:218` says so outright; it went
  with the deleted Node verb (`DEFAULT_MINING_WINDOW_MAX_EVENTS = 500`, found by
  facts-gaps at `5c62cad0^:packages/bee/lib/recovery.mjs:45`).
- *alternatives*, the finding that decided it: `last_durable_settlement`
  (`recovery.rs:216-218`) scans every decision in the repo with **no lane
  filter**, while the cell and capture scans are lane-filtered.
- **Leader verification** (not taken on trust): `bee status --json` on the live
  store returns `since: 2026-09-02T06:16:58.963Z` for candidates whose last
  heartbeats were `2026-08-25T05:41` and `2026-08-31T05:16`. That value is
  `tail -1 .bee/decisions.jsonl` — the reflect-xia decision logged an hour
  before. Those sessions' mining windows fall entirely after their own death.
  Building the verb would have blessed that computation.
- *facts-gaps*: 7 BLOCKERs on cell 1 independently — the refusal is held in
  **eleven** places, not one (registry payload + description + an undeclared
  `purpose` flag, `try_native`, an in-crate tripwire at `tests.rs:1781-1796`,
  `registry_dispatch.rs`, and six shipped doc surfaces); the canonical registry
  example would run in a scratch repo with no `.bee/sessions/` and must not emit
  any of the five refusal markers; the Node prompt was **already broken** for the
  digest-only contract (it says "read ONLY the events supplied below" while the
  payload carries a path); transcript resolution unspecified and the obvious
  copy wrong for every non-Claude runtime; and two truncation axes behind one
  `window_truncated` flag.

**Result: no code, no supersede** (D1). `sweep-recovery-door` D3 stands whole and
every doc asserting the verb is unbuilt stays true. The `since` defect is filed
as its own P2 backlog row — a shipped bug, not something to fix silently inside
a widening.

**The prompts move to the skill** (D4). *alternatives*: a prompt in Rust cannot
be tuned without a rebuild, and instruction text is the skill layer's to own
(the Regrowth law). This also answers the open question at
`maintenance/recovery.md:215` — the skill's dangling `recovery window` citation
at `scout-and-ticks.md:95` is the stale half, and it is fixed in the cheap
direction.

**The fourth bar becomes a Q3 routing qualifier** (D8). *alternatives*: the three
existing bars are filters that can reject; "route to a skill the run opened,
else `tune description:`" never rejects — it decides where a promotion lands.
Shelving a router among filters mis-teaches all four. Q3 ("not mechanizable →
promote as prose") is the slot, and `SKILL.md:123` already points at the tree, so
the body costs nothing — cleaner under the Regrowth law than the draft's split.

**D9 is added.** *alternatives*: "was this skill opened" is not mechanically
checkable — nothing in `hooks/` records a Skill invocation, and the cell field
`affects_skills` records which skills the work EDITS, a different fact. Q4
already requires a one-line recorded reason when prose survives instead of a
mechanism; that reason is written and the durable owner is filed.

**D7 is added**, from *user-impact*: the offer must disclose two things the user
cannot otherwise know — that the transcript is read by the configured `read`
slot (an external pane on this host) and that the capture queue is already past
its blocker threshold (`bee orient`: 50 pending, B50/R101 escalates at 10+).

## Kept as drafted, with the seat's reason

- Two prompts, not one (*alternatives* Q2): the union is one prompt in content
  terms, but the recover digest's shape is pinned by `recovery.md` B33 and fusing
  it costs a second supersede for no gain.
- Cells file-disjoint and concurrent (*alternatives* Q5): `bee-hive/**` +
  `docs/knowledge/**` vs `bee-capturing/**`; only the `bee dev regen` invocation
  serializes.

## Recorded as Known gaps rather than solved

- **The trigger is a sentence, not a command** (*user-impact* Q1): nothing in bee
  names the words that start an asked-for mining, and no existing gesture fits.
  Named in CONTEXT.md (trigger `a-user-s-asked-for-mining-request-is-mis__81d46f80`);
  a gesture is a later decision if the sentence proves soft — trigger `a-user-s-asked-for-mining-request-is-mis__81d46f80`.
- **No dedupe** (*user-impact* Q4): `capture.rs` stores a `source: "mined"`
  marker and no content hash, so two asks before a flush can file
  near-duplicates — trigger `two-asked-for-minings-before-a-capture-f__81d46f80`.
- **The pane trust boundary** (*user-impact* Q3): D7 discloses it; no transport
  rule is added — trigger `transcript-text-reaches-an-external-cli-b__81d46f80`.
- **The fourth-bar change is invisible at runtime** (*user-impact* Q6): it only
  changes where a Compound promotion lands, and Compound runs at the owner's own
  pace by design (`bee-capturing` § Compound). Stated plainly rather than oversold.
- **Partial duplication** (*alternatives* Q4): D5 already tells the miner to
  scope routing candidates, so D8 only adds value on the non-mined Compound path.
- **`recovery` has no verify-app feature file** (*facts-gaps* GAP 13): this
  feature changes no runtime surface, so the obligation does not fire here.

## Second-order: nil

*facts-gaps* GAP 15, stated so no cell spends time on it: `build_recovery_block`
and `run_scan` neither call nor are called by window math, so `bee status`'s
recovery block, `recovery scan`, `doctor` and the session-close path are
untouched — and with cell 1 deleted, nothing in `packages/` moves at all.
