# Context: pi-slp-next-ops

Slice 3 of `docs/history/research/seatworks-slp-pi-small-models.md`
("derive each role's permitted next operations"). The user said "làm toàn
bộ cho tới hết" (do everything to the end) on 2026-10-03.

A read-only map of the next-step paths (herding gather, 2026-10-03) found:
`bee orient` reads only `.bee/state.json`, so a lane-bound session sees
another feature; `next.command` is either a browse (`bee cells ready`) or an
act an agent cannot do ("open your session at …"); the per-turn hint that Pi
shows every turn (`prompt-context`) carries prose only. Workers get no
preamble by design (herding-worker-standalone D3) and one order by
pi-slp-dispatch-return D1, so "per role" here means the leader.

## Locked decisions

- **D1 — orient reads the session's own record** (store `b3f14c45`).
- **D2 — one runnable next operation, one function** (store `fbde3d6f`):
  `bee worktree enter --id <id>` from main with a grant; `bee dispatch wave
  --runtime <runtime> --feature <feature> --json` for ready cells under an
  approved execution gate; `bee state handoff show --json` for a handoff;
  else null. `next.run_from` names the directory to run it in.
- **D3 — the per-turn hint carries the command** (store `42a3429b`): one
  `run: <command> (from <run_from>)` line beside `next:`.
- **D4 — no "open your session at" anywhere** (store `dfd7ae10`): the
  worktree-first denies and route notices name `bee worktree enter --id`;
  the generic containment deny says a path outside the project belongs to
  the user. Writes outside the project stay denied.

Test contract: `contract:next-op-runnable` (store `80987a3f`).

## Out of scope

Binding the deploy permit to a worker identity (this repo's own release flow
runs `scripts/release.sh` from the leader by design), and the `--actor`
label on gate writes (bee cannot tell a relayed user "yes" from an agent's).
