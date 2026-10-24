# Advisor consult — issue-sweep-1024

High-risk gate consult on plan.md, cells iss-1 and iss-4. Advisor role
(fable), read-only, confidence 90.

## Findings

- iss-1 file list was too narrow: the intake gate's git logic is in
  `hooks/write_guard/checks.rs` (`evaluate_git_invocation` ~793-934, `READONLY`
  ~860, `idle_gate_safe_form` ~1263, push deny ~873-880); `paths.rs:592` holds
  only the message. Applied: checks.rs added to iss-1.
- `write_guard/tests.rs` ~1825-1827 and ~2859-2861 assert that `git push origin
  main` on idle main is refused. D1 reverses exactly that. Applied: the
  prohibition names those two assertions as the only allowed edit.
- #135: `git add .bee/config.json` becomes a target at `guards.rs` ~841-868 and
  `main.rs` ~633-650 denies it; `git commit` adds no targets and may already
  pass. Applied: the cell reproduces first.
- Neighbours to pin refused (iss-1): `main:other`, `HEAD~1:main`, `+main`,
  `--force-with-lease`, `:main`, `-o`, `-c` before push, `git pull`,
  `git checkout -- / restore / rm / mv` of config.json, the compound
  `git add … && echo x > .bee/config.json`, and the linked-worktree outward arm
  (checks.rs ~1033). Applied to the iss-1 action.
- iss-4 (1): accept `ceiling` only when `cell_is_escalated(cell)`; pin a
  non-escalated `ceiling`, a `role: ceiling` without the flag, and a third role.
  Applied.
- iss-4 (2): the fault is `off` missing from `FLAG_ALONE_BOOLEANS`
  (`verbs/reservations/flags.rs` ~31-38) and `bool_flag` mapping `"true"` to
  false (`verbs/cells/util.rs` ~127). Applied: both files added to iss-4.

## Recommendation

Add checks.rs to iss-1 and flags.rs to iss-4, relax the iss-1 assertion
prohibition to the two named assertions, pin the neighbours. All applied
before the gate.

Falsifier (not run): `bee cells escalate --id <any> --off --json` clears the
flag with no shape refusal.
