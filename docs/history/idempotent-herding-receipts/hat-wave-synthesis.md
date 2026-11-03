# Hat wave synthesis — idempotent-herding-receipts, plan step

Date: 2026-09-20. Five seats, dispatched through `bee dispatch prepare
--kind advisor --role <seat>`: three as Agent workers, two as herding panes.
This synthesis is the plan check AND the high-risk gate's advisor consult.

## Verdict

The wave rejected two of the plan's three slices and corrected four of its
fourteen load-bearing claims. The plan was rewritten to slice 1 only.

## What each seat changed

**hat-facts-gaps** — audited 11 of 14 claim rows against bytes. Eight held.
Three failed, and every failure was confirmed by the leader before acting:

- Claim 4's stated reason was false. `execute_no_pane` writes the inbox marker
  itself (`run.rs:2892-2896`), so "the native path returns before the marker"
  was wrong. The true fact is one level up: `run()` branches on `no_pane`
  itself and the detached path returns earlier still (`run.rs:3944-3963`).
- Claim 13 said two tests pin the header row set with `assert_eq!`. One does
  (`pi_plugin_contracts.rs:3273`); the other asserts by containment
  (`:2786-2794`) and is additive-safe.
- Claim 10's `ran` evidence did not reproduce: `341` came from the main
  checkout and named none; this worktree holds 0, main now holds 344.
- Claim 2's anchor was off by four lines (`:280` → `:284`).

It also named four spec gaps in slices 2 and 3 that no claim row covered:
what a replay returns to its caller, what round a non-`--continue` re-run is,
which bytes the digest covers, and what marks a job "finished" for prune.

<!-- bee:not-a-deferral: a historical record of what each hat seat reported. The words "deferred" and "DEFER" below quote the seats' verdicts; each verdict was acted on the same day (D5-D8) and none is an open promise. -->
**hat-risks** — retracted the plan's one escalation and found the delete
hazard. The usage-limit resume writes no brief (`run.rs:3176-3240` contains no
`brief_path` write; the only two are `:2635` and `:3374`), so the
same-key-different-payload conflict the plan was about to put to the user does
not exist. Claim 14 is withdrawn, not deferred. On prune: `.bee/mailbox/` is
gitignored, so a wrong delete has no undo, and the plan's "no terminal result"
test is wrong for three live shapes — finished-but-undelivered, multi-round in
flight (`result-1.json` + `brief-2.txt` is a live worker), and usage-limit
paused. Live count: 344 dirs, 310 with `result-1.json`, **0 with
`result-2.json`**, 34 with no result.

**hat-value** — materiality. Slice 1 PASS: silent data loss, two files.
Slice 2 FAIL/DEFER: high risk on the critical path for a rare condition.
Slice 3 FAIL: 14 MB of gitignored disk, and the single reason the feature
routed high-risk.

**hat-alternatives** — agreed the slicing was already minimal but said the
question was only half asked: the mechanism choices inside slice 2 were not
the cheapest. It independently found the `execute()` placement error and
proposed hashing `brief-N.txt` on demand instead of storing a digest artifact.
Both findings now sit in § Out of scope as inputs to the second shaping pass.

**hat-user-impact** — wrote the byte-for-byte SEE mock of the round-1 and
round-2 injections. The mock is what confirmed the `round` row belongs
directly after `job_id` so the two read as one key.

<!-- /bee:not-a-deferral -->

## What the leader kept

The wave did not get to decide scope. D1, D3 and D4 remain locked and
unreduced; the plan carries a SPLIT RECOMMENDED to the user with each gap
named, per AGENTS.md scope integrity. The `hat-value` seat's "drop slice 3"
was taken as "do not plan it yet", not as a decision to drop it.

## Deviation recorded

`bee route --set` refused to demote the lane from `high-risk` to `standard`
after the deleting slice left the plan ("high-risk lanes never demote"). The
plan therefore carries high-risk ceremony for standard-sized work. Named in
the plan's Summary rather than worked around.
