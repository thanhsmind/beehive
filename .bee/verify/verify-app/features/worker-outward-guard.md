# Worker outward guard

The write guard refuses three outward-facing shell commands from any call whose working
directory is a linked git worktree: `git push`, a `gh` command that is not on the read-only
list, and a nested `claude`/`codex`/`pi`/`opencode` launch. It holds in every phase, and the
location decides rather than the caller — a human in their own worktree is refused too. The
main checkout is unchanged. One key, `guards.worker_outward` read from the MAIN checkout's merged config layers (`.bee/config.json` / `config.local.json`), turns the three refusals off. The full read-only list lives in
`docs/config-reference.md` § `guards.worker_outward`.

## Sub-features

- `outward-push` refuses `git push` from a linked worktree, in every spelling.
- `outward-gh-write` refuses every `gh` form outside the read-only list, an alias included.
- `outward-gh-read` lets the read-only list through with no change.
- `outward-launch` refuses a nested agent launch by command word, through `env`, `npx`,
  `bunx`, `sudo` and `timeout` too.
- `outward-launch-exempt` allows the command `bee dispatch prepare` returns: a configured
  `models.<runtime>.<name>.command` prefix, or `codex exec` with a read-only sandbox.
- `outward-main-unchanged` leaves every verdict from the main checkout as it was.
- `outward-opt-out` stays silent when the MAIN checkout sets `guards.worker_outward: false`.

## How to get to it (user POV)

- Run any Bash tool call from a session whose cwd is a worktree `bee worktree new` created —
  an execution worker, a herding pane, or the user's own shell.
- Run `git push origin main` there. It stops with exit 2 and names `bee worktree merge` from
  the main checkout.
- Run `gh pr create --fill` there. It stops and names the main checkout and
  `scripts/release.sh` as the paths for a GitHub write.
- Run `claude -p "hi"` there. It stops and names `bee dispatch prepare`.
- Run `bee config set --key guards.worker_outward --value false --json` in the
  MAIN checkout to turn all three off for the repo (read from the merged config
  layers).

## Driving it with control-bee

Preconditions:

- A launched sandbox, `control-bee doctor` fully `ok`.
- An active feature and an approved execution gate.
- A worktree: `control-bee cli -- worktree new --feature demo --json`. Read its path from the
  payload; it is the `repo--wt--demo` sibling of the sandbox, the tree
  `VERIFY_CWD=repo--wt--demo` names.

- **Refuse a push from the worktree.** The guard reads one JSON object on stdin. Run
  `printf '{"tool_name":"Bash","tool_input":{"command":"git push origin main"},"cwd":"<worktree path>"}' | control-bee cli -- hook write-guard`.
  The `.exit` file holds `2` and stderr carries `bee worktree merge`.
- **Let a GitHub read through.** Send the same payload with `"command":"gh pr view 1"`. The
  `.exit` file holds `0`.
- **Refuse a nested agent.** Send the same payload with `"command":"claude -p x"`. The `.exit`
  file holds `2` and stderr carries `bee dispatch prepare`.
- **Turn the key off in the MAIN root.** Run
  `control-bee cli -- config set --key guards.worker_outward --value false --json`
  in the sandbox's main checkout (never `put` over `.bee/config.json`, which
  replaces the whole config, and never set it only in the worktree). Re-send the
  push payload: the `.exit` file holds `0`.
- **Proof.** Run `control-bee snapshot worker-outward`. The evidence dir holds the two
  refusals at `2`, the read at `0`, the opted-out push at `0`, and the `.bee/config.json`
  that decided.

## Gotchas

- **`--dry-run` is still a push.** `git push --dry-run` is refused. The arm reads the
  subcommand, never the flags.
- **A heredoc body is not judged.** `cat <<EOF` … `git push` … `EOF` passes, because the
  guard fences heredoc text.
- **A quoted push is refused anyway.** `echo git push` is refused: the git finder reads every
  token of the line.
- **The worktree's own config is not read.** The same key inside the worktree's
  `.bee/config.json` changes nothing, and the refusal says so.
- **An unknown `gh` verb is an alias, and an alias is refused.** `gh v` cannot be proved
  read-only.
- **`bee` is never judged.** `bee herding run` still opens its pane agent from the worktree.
- **Leader and herded-pane boundaries.** A leader holding the worktree grant is
  also exempt from the launch refusal (`outward-launch-exempt`), while a herded
  pane (`BEE_HERDING_WORKER=1`) has a separate worker guard that ignores the
  `guards.worker_outward: false` opt-out.
