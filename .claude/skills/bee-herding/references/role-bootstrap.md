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
against your starting cwd.

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
yourself; the bypass level is a user-owned safety posture. Below `full`, the
dispatch loop refuses every cycle anyway (Dispatch role §2), so there is
nothing to gain by bootstrapping.

### 4. Resolve the workspace id

On tmux there is nothing to resolve — the workspace IS the tmux session you
are already in (D3). Skip to §5, and run every later step from the pane you
want to become the chat pane: the cockpit and runtime windows appear in that
same session, and the dispatch and merge panes are split off your own pane.

Everywhere else:

- The human gave an explicit workspace id or label → verify it exists:
  `bee herding pane tab-list --workspace <id>`.
- Otherwise run `bee herding pane current` and read the `workspace_id` it
  reports — that is the workspace your own pane sits in.
- No id comes back, or the human's id does not resolve → list what you found
  and ask which to use. Never guess.

### 5. Check for an existing cockpit before bootstrapping again

```
bee herding pane-id --label dispatch
bee herding pane-id --label merge
```

Either one answers with a `pane_id` (exit 0) → a cockpit exists: report that
instead of re-bootstrapping, and point at README.md's stale-label fix
(`bee herding pane close <pane_id>` or
`bee herding pane rename <pane_id> --clear`). Exit 1 with `not_found` means
no such pane. The script refuses the `dispatch` case too; the check exists so
you can explain why before spending a run on it.

### 6. Run the bootstrap script

Only once every pre-flight has passed and no cockpit was found:

```
bash <main-root>/.claude/skills/bee-herding/scripts/bootstrap-cockpit.sh --main-root <main-root> [--workspace <id>]
```

(use the copy under whichever skill root your runtime reads — `.agents/` for
Codex; both are byte-identical.) `--workspace` is required on a transport that
has workspace objects and ignored on tmux. Pass through `--dry-run` or
`--no-start` if the human asked for either — see the script's own usage.

Report the script's output back verbatim — it already states which panes it
created and whether the loops started — then remind the human: watch the chat
pane for `dispatch:`/`merge:` lines (silence is normal — either nothing is
ready, or all four runtime slots are busy), and stop the dispatch loop with
`touch <main-root>/.bee/tmp/bee-herding.stop` when done.
