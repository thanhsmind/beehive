# bee-herding — Bootstrap role protocol

The full protocol. The body carries only the step list and the role boundary.

You are the **bootstrap** role — recognized when a human invokes the skill
directly with no `--role`. This is a **one-shot setup action**: run every step
below once, in order, to completion, then stop. There is no cold re-invocation
here and no cross-invocation memory concern — the whole role happens inside a
single turn.

### 1. Resolve the main checkout root

Never assume your cwd is main — resolve it explicitly:

```
git rev-parse --path-format=absolute --git-common-dir
```

This returns the absolute path to the shared `.git` directory, correct whether
you were invoked from main or from inside a linked worktree. Strip the trailing
`/.git` for `<main-root>`. Every command below runs against that path, never
against whatever directory you happened to start in.

### 2. Pre-flight — main must be clean

```
git -C <main-root> status --porcelain
```

- **Empty.** Continue to §3.
- **Only `.bee/logs/*.jsonl` entries.** These are meant to be gitignored.
  Report it and suggest — never run yourself — the untrack-and-commit fix in
  README.md, then stop; the human runs it and re-invokes you.
- **Anything else dirty.** List the dirty files, stop, and ask the human to
  clean main first. `bee worktree merge` refuses on a dirty main and the merge
  role runs inside main, so an unclean checkout makes every later merge fail
  once the loop starts.

### 3. Pre-flight — `gate_bypass_level` must be `full` or `total`

```
bee status --json
```

Not `full` or `total` → stop and tell the human to raise it — never change it
yourself; this is a user-owned safety posture bootstrap does not decide on
their behalf. Below `full`, the dispatch loop would refuse on every cycle
anyway (Dispatch role §2), so there is nothing to gain by bootstrapping.

### 4. Resolve the workspace id

- The human gave an explicit workspace id or label → verify it exists:
  `herdr workspace list`.
- Otherwise run `herdr workspace list` and match a workspace's `label` to the
  basename of `<main-root>`.
- Zero matches, or more than one → list the candidates you found and ask which
  to use. Never guess.

### 5. Check for an existing cockpit before bootstrapping again

```
herdr pane list --workspace <id>
```

Any pane already labelled `dispatch` or `merge` → a cockpit exists for this
workspace: report that instead of re-bootstrapping, and point at README.md's
stale-label fix (`herdr pane close <pane_id>` or
`herdr pane rename <pane_id> --clear`). The script refuses this case too; the
check exists so you can explain why before spending a run on it.

### 6. Run the bootstrap script

Only once every pre-flight has passed and no cockpit was found:

```
bash <main-root>/.claude/skills/bee-herding/scripts/bootstrap-cockpit.sh --workspace <id> --main-root <main-root>
```

(use the copy under whichever skill root your runtime reads — `.agents/` for
Codex; both are byte-identical.) Pass through `--dry-run` or `--no-start` if
the human asked for either — see the script's own usage.

Report the script's output back verbatim — it already states which panes it
created and whether the loops started — then remind the human: watch the chat
pane for `dispatch:`/`merge:` lines (silence is normal — either nothing is
ready, or all four runtime slots are busy), and stop the dispatch loop with
`touch <main-root>/.bee/tmp/bee-herding.stop` when done.
