# Context: issues-sweep-1023

The user asked to fix the open GitHub issues on thanhsmind/beehive
(2026-10-23). Fifteen issues are open. This feature fixes the bee
defects among them. Each decision below names the issue it closes and
its store id.

## Locked decisions

- D1 (#127, `01708244`): `bee cells add` checks each batch cell against
  the approved gate preview packet. Every packet cell must be in the
  batch or already in the cell store. The check that the batch length
  equals the packet length goes.
- D2 (#125, `b364bcbe`): a claim that refuses after its lease was
  written unwinds that lease under the session the claim file records.
  No lease outlives a refused claim, so `cells show` never reports an
  open cell with a held claim.
- D3 (#121, `cf4781e7`): `bee cells leader-check` counts a directory
  entry in `files_changed` as touched when the commit touched a file
  under that directory.
- D4 (#96, `d8e689c8`): the Windows suite goes green. `gate_pid_dead`
  gets a Windows arm. The paseo workspace path match and the worker-guard
  mailbox check compare canonical paths with no `\\?\` prefix and no 8.3
  short names.
- D5 (#126, `e0da7c92`): `bee dispatch prepare` never adds `--read-only`
  to a deployment-stage herding payload.
- D6 (#117, `3b60bbef`): `scripts/release.sh` spends the deploy permit
  only after every zero-mutation precondition passed.
  `scripts/release-dirt.sh` allowlists
  `.bee/supervisor/broker-heartbeat.json`.
- D7 (#123, `438886cc`): `bee herding run` stops the paseo agent when the
  ceiling times out. The agent record stays for inspection.
- D8 (#118, `3656a936`): every test in `tests/pi_plugin_contracts.rs`
  and `tests/session_release.rs` that runs the real bee binary removes
  `BEE_HERDING_WORKER`, except a test that sets it on purpose.
- D9 (#120, `5e66f40a`, supersedes `faa75d94`): onboarding writes a
  bee-managed `.gitattributes` block (the same `# BEE:START` /
  `# BEE:END` markers as the gitignore block) that sets `merge=union` on
  `.bee/decisions.jsonl`, `.bee/decisions-archive.jsonl`,
  `.bee/backlog.jsonl`, `.bee/review-candidates.jsonl`,
  `.bee/deferred-queue.jsonl` and `.bee/wave-ledger.jsonl`. This repo
  gets the block from its own onboarding (`bee dev regen`), with no hand
  edit. Existing hosts get it on their next onboarding.
- D10 (#122, `17294ba4`, supersedes `f4a92008`): `bee worktree merge`
  never stages or commits untracked `docs/history/<feature>` files on
  main that the merging branch also writes. It refuses up front with
  `WORKTREE_MERGE_MAIN_DIRTY` naming them, with zero mutations — the same
  refusal other colliding untracked files get. (Git itself refuses such a
  merge for identical and for different bytes alike.)
- D11 (#124, `32712b8e`): `bee state route --set` takes
  `--lane-record <feature>` to pick the lane record, through the selector
  the other lane-aware state verbs use. `--lane` stays the triage lane.
- D12 (`50c3b5e3`): #104, #106 and #119 close as already fixed
  (`e40edc403` in v2.54.0; `7983338d3` in v2.56.0). #69 is a dashboard
  feature request and is out of this sweep.
