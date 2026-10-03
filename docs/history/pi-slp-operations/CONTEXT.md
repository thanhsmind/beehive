# Context: pi-slp-operations

Slice 2 of `docs/history/research/seatworks-slp-pi-small-models.md`. The
user said "tiếp tục đi" (continue) on 2026-10-03, after slice 1
(`pi-slp-dispatch-return`) merged.

The research report asked for the target small model and a failing
transcript before slice 2. Both came from the repo, not from the user: the Pi
leader runs `deepseek-flash` (`~/.pi/agent/settings.json`), herded workers run
`gemini-3.8-flash`, and five Pi sessions of this repo were mined (findings in
the research report, "Transcript evidence"). The mining changed the slice: the
models met almost every guard, and the damage came after refusals whose fix
named no command, or named an act an agent cannot do.

## Locked decisions

- **D1 — One runnable fix per refusal** (store `b9a6dfe6`). Each refusal
  family below ends its fix with one runnable bee command or one concrete
  path, built from values the refusing code already holds. A placeholder is
  allowed only for a value the code does not hold.
  - `role_plan_required` (dispatch prepare, stage deployment)
  - `deploy_authorization_consumed` and `deploy_authorization_wrong_feature`
    (lane missing)
  - `startFeature` with a live workflow
  - `CLAIMED`
  - the write-guard containment denies for a main-checkout path and for
    another worktree's path
  - the `addCells` preview mismatch
- **D2 — Text only** (store `d361a53f`). Only the fix wording and the JSON
  `fix` field change. Which calls refuse, every reason or code key, and every
  exit code stay as they are.
- **D3 — Write-guard fixes an agent can act on** (store `aa4db1c5`, amended
  after the hat wave; supersedes `cb57963d`). A main-checkout path denied in
  a worktree session names the same relative path inside the current
  worktree; a `.bee/` path names `bee --help --json` run from main instead,
  never a path. A path inside another granted worktree drops "open a
  session with cwd=", says the file belongs to that worktree, and names the
  caller's own worktree root as the place to write. It never suggests
  merging the other worktree.
- **D4 — CLAIMED names who holds it** (store `b1be5bf9`). Held by the calling
  session: the text says so and names the next step without `--claim`. Held by
  another session: the text names `bee cells claim-next`.

Test contract: `contract:refusal-runnable-fix` (store `66f4fd2d`).

## Out of scope

The per-role allowed-operations packet, identity-derived actor and record,
and run-from directory (research implications 1 and 2) are slice 3. Guard
accuracy for paths outside every checkout (user config under `~/.pi`) is a
policy question, not wording; it is filed separately.
