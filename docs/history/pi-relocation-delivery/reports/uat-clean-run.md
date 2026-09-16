# Live acceptance, clean run (2026-09-16, attempt 4)

The first live attempt proved nothing — its "delivery" came from a marker the
operator had copied, and the relocated session was running the worktree's
pre-fix belt (`reports/uat-correction.md`). This run fixed both conditions
before it started:

- The sandbox carries the FIXED belt in BOTH roots (90837 bytes in `repo` and in
  `repo--wt--uat-carry`), and its own `doctor --runtime pi` reports
  `wiring_matches_binary ok`, so the runtime under test really holds the fix.
- `.bee/result-inbox/` was emptied and `relocation-carry.json` deleted first, and
  the operator placed nothing at any point during the run.

## The chain, as recorded

| UTC | Fact |
|---|---|
| 02:33:47 | Detached job `job-1789526027218-2587254-1` launched with `inbox_session: 01a0a80f-6e64-7611-8255-20743cf3eacf` — the OLD session; its marker lands under that token |
| 02:33:58 | The marker is confirmed present under the old token folder, and nowhere else |
| 02:34:08 | `worktree enter --id repo--wt--uat-carry` emits the transition |
| 02:34:13 | The replaced session starts as `01a0a810-9303-7611-8255-2076aaf3770e` — a different id |
| 02:34:18 | The job finishes (`result-1.json` in its mailbox) |
| 02:34:23 | The OLD token's marker becomes `job-1789526027218-2587254-1.json.processing` — the NEW session's drain claimed the CARRIED marker |
| — | The injected `bee-result` block in the new session names that same job id; the session transcript holds exactly ONE injection header, so nothing was delivered twice |
| — | `relocation-carry.json` holds `{"01a0a810…": ["01a0a80f…"]}` |

## The one FAIL in the leader's verdict is the task's wording, not the product

The leader marked step 5c FAIL because `ls -la .bee/result-inbox/` returned
"No such file or directory". It ran that from its cwd, which after the move is
the worktree — and the inbox lives under the MAIN checkout's `.bee`, by design
(the dispatch side always writes there). Checked from the main root, the folder
is present and holds the carried marker as `.processing`. The step's instruction
should have named an absolute path; the behavior it was meant to observe is the
one the rest of the table already proves.

## What this proves, and what it does not

PROVEN live: a detached result addressed to the pre-relocation session reaches
the session after the move, once, without the operator touching anything.

NOT covered by this run: the claim rebind. The short task deliberately created
no cell, so nothing here exercises `cells rebind-session` end to end. That half
rests on the contract case `relocation_with_a_job_in_flight_carries_inbox_and_rebinds_claims`,
which failed on the rebound-claim assertion before prd-5 and passes after it,
plus the verb's four unit tests.
