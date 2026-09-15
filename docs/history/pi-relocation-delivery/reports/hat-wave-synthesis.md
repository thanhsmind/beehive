# Hat wave synthesis — pi-relocation-delivery (plan step)

Seats: `hat-facts-gaps` (opus), `hat-alternatives` (opus), `hat-user-impact`
(agy-flash, herding job `job-1789512170756-1908863-1`). Three seats, standard
lane. None dropped; the wave finished inside its budget.

Every finding below was re-checked against the bytes by the leader before it was
accepted — the seats' reports are navigation aids, not evidence.

## Accepted, and what changed in plan.md

1. **The destination root was unstated, and the two sides disagree** (facts-gaps
   1, alternatives 1, user-impact 4). Verified: the writer always writes under
   the main checkout (`herding/run.rs:2243`) while the reader probes the current
   directory first (`bee-guard.ts:652-658`). A folder created under the worktree
   would win the probe and hide every job dispatched after the move.
   → The approach now states the main root explicitly, claims 3 and 4 carry the
   bytes, and prd-2 must prove a post-move dispatch is still delivered.

2. **Carry the token instead of moving the files** (alternatives 2). Verified
   the cheaper shape is sound: `resultInboxDir` already resolves per token, so a
   second token costs a loop, not a migration. It also removes the duplicate
   injection hazard and the abort hazard below.
   → Approach step 2 rewritten; a SMALLER PATH line now states the trade and the
   one thing the move bought (restart durability), which the pointer file buys
   for one write.

3. **Order inside `withSession`** (user-impact 1, alternatives 3). Verified:
   the exit branch calls `handlePostExitMerge` first (`bee-guard.ts:1531-1533`),
   which shells a merge that can refuse and on success deletes the worktree.
   → The carry and the rebind now run FIRST in the closure, and the test asserts
   the order on the exit direction.

4. **An abort after the closure would strand the claims** (facts-gaps 4).
   Verified: `switchSession` can still report cancelled or failed after
   `withSession` ran (`:1552-1560`, `:1571`).
   → The rebind is now compensating: undone on a cancelled or failed switch.

5. **Duplicate injection through `.processing` reclaim** (user-impact 3).
   Verified the lifecycle: injected markers live as `.processing` until
   `agent_settled` deletes them (`:1838-1839`), and orphan reclaim renames them
   back (`:710-719`).
   → A carried folder is read for unclaimed `<job-id>.json` only; orphan reclaim
   never runs over it.

6. **Claims-table rows were digest prose, not re-openable bytes** (facts-gaps 2).
   → Rows 3, 4 and 6 replaced with byte-exact anchors; the table is now twelve
   verified rows.

7. **The test harness cannot tell the two session ids apart** (facts-gaps 3).
   Verified at `pi_plugin_contracts.rs:701`.
   → The harness change is part of prd-2, and the new case is watched red before
   the belt edit.

8. **The rewriter named was the wrong one, and the epoch bump has consumers**
   (facts-gaps 5). Verified: `verbs/cells/claims.rs:339` is the cells-module
   port; `CLAIM_FENCE_STALE` (`:296-325`) refuses a stale epoch afterwards.
   → prd-1 now names that writer, scopes the rewrite to claims whose `session`
   equals `--from`, and owes the epoch statement.

9. **Warn on a failed rebind, and name what was carried** (user-impact 2 and 5).
   → Approach steps 3 and 4.

10. **prd-4's parity proof was impossible as written** (facts-gaps, minor note).
    Verified: the Pi block's premise forbids a native `Subagent type` row.
    → The proof is now "all seven row names present, with an explicit n/a where
    the mechanism does not exist on Pi".

## Dismissed

- **Split the harness change into its own cell** (facts-gaps 3, second option) —
  dismissed: the harness change is three lines in the same test file prd-2
  already owns, and splitting it would put a red test in one cell and its fix in
  another, which no cap can carry.
- **Drop the prd-2 → prd-1 dependency and run all three cells at once**
  (alternatives 4) — dismissed: correct that the belt only calls the verb by
  name, but prd-2's own test asserts the claim rebind end to end, so it needs the
  verb to exist. The wave change that survives is prd-4 in parallel, which the
  plan now states.
- **Land prd-3's failing case as a separate cell before the belt edit**
  (alternatives 5) — dismissed for the same reason as the first item: a cell
  that ends red cannot cap. Red-before-green now happens inside prd-2, where the
  worker writes the case, watches it fail, then fixes the belt.

## Open questions

<!-- bee:not-a-deferral: this section records that the wave deferred nothing — it promises no later work -->

None. The mechanism questions the wave raised are answered in the plan; nothing
was deferred to the user.
<!-- /bee:not-a-deferral -->
