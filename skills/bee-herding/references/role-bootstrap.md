# bee-herding — Bootstrap role protocol

Loaded from the SKILL.md routing summary. This is the full, authoritative protocol; the body carries only the step list and the role boundary.

You are the **bootstrap** role of the agent-pane-orchestration loop. Recognize this role when you are invoked directly by a human with no `--role dispatch|merge` given — this is a **one-shot setup action**: run every step below once, in order, to completion, then stop. There is no cold re-invocation every 60 seconds here, and no cross-iteration memory concern the way dispatch and merge have it — this whole role happens inside a single turn.

### 1. Resolve the main checkout root

Never assume your own cwd is main — resolve it explicitly, the same underlying constraint the dispatch and merge roles' own §0 rely on (worktrees are created from main; none of this system's control panes run inside one):

```
git rev-parse --path-format=absolute --git-common-dir
```

This returns the absolute path to the shared `.git` directory — correct whether you were invoked from main or from inside a linked worktree. Strip the trailing `/.git` to get `<main-root>`. Every command in the rest of this role runs against that path, never against whatever directory you happened to start in.

### 2. Pre-flight — main must be clean

```
git -C <main-root> status --porcelain
```

Three outcomes:

- **Empty.** Continue to §3.
- **Only `.bee/logs/*.jsonl` entries.** These are meant to be gitignored; report this to the human and suggest — never run yourself — the untrack-and-commit fix already documented in README.md:
  ```
  git -C <main-root> rm --cached .bee/logs/*.jsonl
  git -C <main-root> commit -m "chore: untrack bee session logs"
  ```
  Then stop this role without bootstrapping anything — the human runs those commands and re-invokes you.
- **Anything else dirty.** List the dirty files, stop, and ask the human to clean main first. `bee worktree merge` refuses on a dirty main and the merge role runs inside main, so an unclean checkout would make every later merge fail once the loop starts.

### 3. Pre-flight — `gate_bypass_level` must be `full` or `total`

```
node <main-root>/.bee/bin/bee.mjs status --json
```

Read `gate_bypass_level`. If it is not exactly `full` or `total`, stop here and tell the human to raise it (`bee-bypass-gate full`) — never change it yourself; this is a user-owned safety posture bootstrap does not get to decide on the human's behalf. Below `full`, the dispatch loop would refuse to operate on every cycle once started (Dispatch role §2), so there is nothing to gain by bootstrapping anyway.

### 4. Resolve the workspace id

- If the human gave an explicit workspace id or label, use it — verify it actually exists: `herdr workspace list`.
- Otherwise, run `herdr workspace list` and match a workspace's `label` to the basename of `<main-root>`.
- Zero matches, or more than one — list the candidates you found and ask the human which to use. Never guess.

### 5. Check for an existing cockpit before bootstrapping again

```
herdr pane list --workspace <id>
```

If any pane in that workspace already carries the label `dispatch` or `merge`, a cockpit already exists for it — report that instead of re-bootstrapping, and point at the same fix README.md's troubleshooting section documents for a stale label: `herdr pane close <pane_id>` or `herdr pane rename <pane_id> --clear`. (`bootstrap-cockpit.sh` itself also refuses when a `dispatch`-labelled pane already exists — this check exists so you can explain why before spending a run on it.)

### 6. Run the bootstrap script

Only once every pre-flight check has passed and no existing cockpit was found:

```
bash <main-root>/.claude/skills/bee-herding/scripts/bootstrap-cockpit.sh --workspace <id> --main-root <main-root>
```

(use the copy under whichever skill root your runtime reads — `.agents/` for Codex, `.claude/` for Claude Code; both are byte-identical.) Pass through `--dry-run` or `--no-start` if the human asked for either — see the script's own usage for what each does.

Report the script's own output back to the human verbatim — it already states which panes it created and whether the loops started — then remind them: watch the chat pane for `dispatch:`/`merge:` lines (silence is normal — either nothing is ready, or all four runtime slots are busy), and stop both loops with `touch <main-root>/.bee/tmp/bee-herding.stop` when done.
