# bee verification map

This directory is the maintained source for verifying the user-facing behavior of
the `bee` CLI. Read this index before driving, then use the matching feature file
as the recipe.

## Baseline preconditions

- Build the binary once: `control-bee build`. Never hardcode
  `packages/bee-rs/target/release/bee` — `CARGO_TARGET_DIR` may move it. Ask
  `control-bee bin`.
- Create a disposable sandbox: `control-bee launch`. It prints
  `$VERIFY_HOME/run/<run-id>/repo`.
- Set `VERIFY_HOME` to give concurrent runs separate roots. Each run already gets
  its own id, sandbox and evidence dir.
- Run `control-bee doctor` and require every line to read `ok`.
- Never drive a checkout this run did not create. `control-bee` refuses any tree
  outside `$VERIFY_HOME/run`.

## Driving conventions

- `control-bee` below is short for `bash .bee/verify/verify-app/control-bee`.
  Invoke it by that literal path, with the `bash` prefix, from the repo root:
  bee renders `0644` copies of this tree into every runtime skill home, so a
  bare path fails there with `Permission denied`. A shell variable in command
  position is refused by bee's write guard.
- Every recipe starts from a freshly launched sandbox unless it says otherwise.
- Run bee commands through `control-bee cli -- <args>`; run git and other tools
  through `control-bee sh -- <cmd>`; create files with `control-bee put <path>`.
- `bee onboard` is the exception: drive it with `control-bee host -- <binary>
  onboard --repo-root <target> ...`, because it needs cwd set to the real
  checkout. Make its target with `control-bee newrepo <name>`.
- Drive a worktree bee created by setting `VERIFY_CWD=repo--wt--<feature>` on the
  same call.
- Pass `--json` to every bee command and assert on payload keys, on `error`, and
  on `kind` — never on human wording, and never on the `[bee] <verb> Nms` timing
  line bee writes to stderr.
- Treat every command as literal. Keep quoted flag values unchanged.
- Do not hand-edit `.bee/*.json` to reach a state. Reaching it through the verbs
  is the thing being verified.

## Proof and skip reporting

- Capture the user action and the resulting state, not only the last command.
- Success proof is the `--json` payload plus a `control-bee snapshot <label>`
  showing the state bee wrote (`state.json` gates, `cells/<id>.json` status, a
  git log entry, a file on disk).
- Refusal proof is the `error` and `kind` strings **and** a non-zero `.exit`
  file. bee prints its refusal to stdout when `--json` is passed, so the exit
  code is the only half the JSON does not carry.
- Record the feature ID and the entry point used with every artifact.
- Report an unreachable path with the attempted command and the unmet
  precondition. Do not report a skipped entry point as verified through another.

## Feature entry contract

Each feature file starts with an H1 title and one paragraph describing the
user-visible behavior. It then uses exactly four H2 sections in this order.

1. `Sub-features` lists short IDs with one line for each behavior.
2. `How to get to it (user POV)` lists every user entry point.
3. `Driving it with control-bee` starts with `Preconditions:` and uses labeled
   bullets that pair each user action with an exact command and observable
   result.
4. `Gotchas` lists traps that can waste or invalidate a verification run.

Keep implementation details out of the map. Name only user paths, stable handles,
required state, commands, and observable proof.

## Features

- [Onboard a project](./onboard.md) covers the read-only plan, apply, idempotent
  re-run, and what lands on disk.
- [Orient and status](./orient-and-status.md) covers the two "where am I" verbs
  and the version answer that needs no repo.
- [Feature start and gates](./feature-gates.md) covers starting a feature,
  recording its route, and approving the merged and named gates.
- [Cells and the proof line](./cells-and-proof.md) covers adding, claiming and
  capping a cell, and the red-proof refusal that is bee's core promise.
- [Worktree and close](./worktree-and-close.md) covers creating a feature
  worktree, merging it back, and the close doors.

## Not yet mapped

Real user surfaces with no feature file yet. Add one before claiming coverage of
them: `decisions` (log, search, supersede), `capture` and `knowledge`,
`backlog` / `pbi`, `reservations`, `reviews`, `staging`, `dispatch prepare`,
`mailbox` (reflect and the run digests — used by the close recipe but not mapped
on its own), the `hook` handlers, and the `herding` cockpit (which needs live
agent panes — see the `bee-herdr` skill instead).
