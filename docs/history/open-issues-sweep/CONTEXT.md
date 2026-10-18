# Context: open-issues-sweep

The user asked to fix every open GitHub issue on thanhsmind/beehive
(2026-10-18). Twenty-one issues are open. This feature fixes the bee
defects among them. Each decision below names the issue it closes.

## Locked decisions

- D1 (#109, #111, #113): `bee dispatch prepare --kind cell` honours the
  cell's `escalate` flag also when the approved plan assigns the cell a
  role. An escalated cell resolves to the session model (no herding
  command), with or without a v2 role plan. Only an explicit `--role` on
  the command line still outranks the flag.
- D2 (#108): `bee cells unclaim` and `bee cells reopen` remove the claim
  store row for the cell, the same way cap already does. After an
  unclaim, `cells claim` and `dispatch prepare --claim` both succeed.
- D3 (#103, #112): a herding child started with a dispatch permit
  (`BEE_DISPATCH_ID` in the pane passthrough) keeps the passthrough
  `BEE_SESSION_ID`. Every other leader session variable
  (`CLAUDE_CODE_SESSION_ID`, `PI_SESSION_ID`, and `BEE_SESSION_ID`
  without a permit) is still stripped.
- D4 (#98): `herding run` adds the "start the daemon" FIX only when the
  Paseo daemon does not answer. When the daemon answered with an error,
  the FIX names the agent's provider and model configuration instead.
- D5 (#96): the Windows build compiles again — the unix-only test
  helper in `herding/run.rs` and its callers are `cfg(unix)`.
- D6 (#97, #105, #110, #114): inside a granted feature worktree,
  `bee decisions *` and `bee backlog *` read and write the control-root
  (main checkout) store, the same store the claim door, dispatch and
  `cells add` use. This supersedes the WIDE-door rule that gave a
  granted worktree its own decisions store.
- D7 (#99): `bee triggers add --decision <id>` refuses an id that names
  no decision in the store, and writes nothing.
- D8 (#101): the plan-freeze guard's fallback (no lane record) reads the
  control root's `.bee/state.json`, not the worktree's own copy.
- D9 (#115): the close doc-deferral door does not scan a frozen
  `docs/history/<feature>/plan.md` (shape gate approved), and does not
  scan files that `bee knowledge index` generates.
- D10 (#107): a closed feature's promote-proposal reminder can be
  cleared. The reminder and the `scribing-run` refusal from phase idle
  both name the one command that clears it, and that command works from
  phase idle.
- D11 (#100): `bee onboard --apply` never overwrites a verify-app
  projection file that is newer than its canonical source in
  `.bee/verify/`; it reports that item blocked, with a FIX.
- D12 (#102): `bee cells finish` and `bee cells cap` accept
  `--report-file <path>` as well as `--report`, so a report whose proof
  names `bash` can come from a file instead of a shell argument.

## Already fixed at HEAD (closed with evidence, no cell)

- #104, #106: `worktree prune` keeps a queue only when it holds a stub
  main lacks (commit e40edc403). `bee worktree prune --dry-run
  --older-than-days 0` now reports merged worktrees as removable.

## Out of scope

- #69 (a read-only multi-project web dashboard, "bee-hive") is a new
  product, not a defect. It needs its own discovery and shaping. It is
  filed to the backlog and stays open.
- Clearing the 72 existing promote-proposal reminders is proposal
  review work, not part of D10.
