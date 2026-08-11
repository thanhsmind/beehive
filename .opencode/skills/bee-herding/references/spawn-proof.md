# Spawn Proof — one real worktree, one real pane, one real agent

> Note: this is a recording made in a different repo.
> Pane ids (`w7:pN`), PBI ids (`PBI-043`), the workspace name
> and the `--wt--` slugs below are that repo's, not this one's; they are kept
> verbatim because the *method* is what this document proves.

This is the authoritative worked example for §8's spawn sequence,
run once for real with the throwaway slug
`apo-throwaway-spawn`, torn down completely afterward. All values below are
observed, not descriptions.

Run from workspace `w7`, tab `w7:t1` (a live production herdr workspace with
several other real bee workers running concurrently — this is not an idle
sandbox; pane counts include those other panes throughout).

## Step 1 — create the worktree (MAIN checkout only)

```
.bee/bin/bee worktree new --feature apo-throwaway-spawn --json
```
Run from `/home/you/proj/repo` (never from inside a linked
worktree — it refuses). Result:
- `id`: `herdr-gateway--wt--apo-throwaway-spawn`
- `worktreeRoot`: `/home/you/proj/repo--wt--apo-throwaway-spawn`
- `branch`: `wt/apo-throwaway-spawn`

## Step 2 — split a pane into that worktree

Pane count in workspace `w7` immediately before the split: **5**
(`w7:pP`, `w7:pM`, `w7:pJ`, `w7:pN` in tab `w7:t1`; `w7:pQ` in tab `w7:t5`).

Picked split target `w7:pP` (tied for the largest rect, `117x36`, wider than
tall → `--direction right`, per §8's tie-break rule):

```
herdr pane split w7:pP --direction right --ratio 0.5 \
  --cwd /home/you/proj/repo--wt--apo-throwaway-spawn --no-focus
```

Result: new empty pane `w7:pR`, `tab_id w7:t1`, `cwd` and `foreground_cwd`
both the new worktree path, `agent_status: "unknown"`, no `label`.

## Step 3 — start the working agent (the previously unproven wiring)

```
herdr agent start apo-throwaway-spawn \
  --cwd /home/you/proj/repo--wt--apo-throwaway-spawn \
  --workspace w7 --tab w7:t1 \
  -- claude --model sonnet --permission-mode bypassPermissions
```

**Observed finding — the step-2 pane is not reused.** `agent start` does not
attach to the pane opened by `pane split`; it opens its own **new** pane.
First attempt (using a headless `-p` one-shot argv, see "Rejected first
attempt" below) produced pane `w7:pS`, distinct from `w7:pR`. Left as
written, §8's sequence therefore leaves the step-2 split pane (`w7:pR`)
permanently empty and stray — it must be closed explicitly, or step 2 should
be dropped and step 3 relied on alone to open the pane. This proof run
closed the stray pane (`herdr pane close w7:pR`) before continuing; this
document only records what was observed.

**Rejected first attempt — headless `-p` self-terminates the pane before
anything can be observed.** Passing `-p "<prompt>"` in the argv runs the
agent to completion and exits; herdr then closes the pane along with the
exited process. Result: pane `w7:pS` was returned by `agent start`'s own
response, but a `pane list` taken immediately after already showed it gone
(`herdr pane get w7:pS` → `pane_not_found`). This is why §8's production
argv is a plain interactive `claude --model sonnet --permission-mode
bypassPermissions` with no `-p`: only an interactive session keeps the pane
alive long enough to be observed (or to do real unattended work).

**Working attempt — plain interactive argv, seeded with a bounded initial
prompt as a positional argument (not a flag, so `--permission-mode
bypassPermissions` is unnarrowed):**

```
herdr agent start apo-throwaway-spawn \
  --cwd /home/you/proj/repo--wt--apo-throwaway-spawn \
  --workspace w7 --tab w7:t1 \
  -- claude --model sonnet --permission-mode bypassPermissions \
  "First run exactly: herdr pane current --current -- to learn your own \
  pane_id. Then run exactly: herdr pane rename <that pane_id> \
  apo-throwaway-spawn -- substituting the real pane_id you just learned. \
  Then stop and do nothing else: do not read AGENTS.md or CLAUDE.md, do not \
  run any bee command, do not edit, create, or delete any file, do not \
  start any other task."
```

Observed exact argv, echoed back verbatim by `agent start`'s own JSON
result (`result.argv`):

```json
["claude","--model","sonnet","--permission-mode","bypassPermissions",
 "First run exactly: herdr pane current --current -- to learn your own pane_id. Then run exactly: herdr pane rename <that pane_id> apo-throwaway-spawn -- substituting the real pane_id you just learned. Then stop and do nothing else: do not read AGENTS.md or CLAUDE.md, do not run any bee command, do not edit, create, or delete any file, do not start any other task."]
```

`result.agent`: new pane `pane_id: "w7:pT"`, `tab_id: "w7:t1"`,
`workspace_id: "w7"`, `name: "apo-throwaway-spawn"`,
`cwd`/`foreground_cwd`: `/home/you/proj/repo--wt--apo-throwaway-spawn`.

## Confirmed: self-naming and live session

Polling `herdr pane get w7:pT` ~8s later:

```json
{
  "pane_id": "w7:pT",
  "label": "apo-throwaway-spawn",
  "agent": "claude",
  "agent_status": "idle",
  "agent_session": {
    "agent": "claude",
    "kind": "id",
    "source": "herdr:claude",
    "value": "d43cd24b-d6b4-4e8a-839d-6b9e617fba69"
  },
  "cwd": "/home/you/proj/repo--wt--apo-throwaway-spawn",
  "foreground_cwd": "/home/you/proj/repo--wt--apo-throwaway-spawn",
  "tab_id": "w7:t1",
  "workspace_id": "w7"
}
```

- **Pane self-named to exactly the worktree name, nothing else appended**:
  `label` = `apo-throwaway-spawn`, matching the rule that a working agent
  labels its pane with its worktree name.
- **Live agent session confirmed**: real `agent_session.value`
  (`d43cd24b-d6b4-4e8a-839d-6b9e617fba69`), not a placeholder.
- `herdr pane read w7:pT --source recent --lines 60` showed the agent ran
  exactly the two instructed shell commands, reported "Done. Pane w7:pT
  renamed to apo-throwaway-spawn. Stopping here as instructed.", and the
  pane's own status bar read `⏵⏵ bypass permissions on` and
  `🌿 wt/apo-throwaway-spawn` — confirming both `--permission-mode
  bypassPermissions` took effect and the pane's cwd is the throwaway branch.
- Pane count in workspace `w7` right after this step: **6** (one net new
  pane vs. the pre-split baseline of 5 — the transient `w7:pR`/`w7:pS`
  detours were both already closed by this point).
- `git status --porcelain` inside the throwaway worktree showed only
  `M .bee/logs/tools.jsonl` (the harness's own tool-call log, an expected
  side effect of any tool call, uncommitted) — no other file was touched,
  confirming the agent did no work beyond the bounded instruction.

## Teardown (back to starting state)

```
herdr pane close w7:pT
git worktree remove --force /home/you/proj/repo--wt--apo-throwaway-spawn
git branch -d wt/apo-throwaway-spawn
.bee/bin/bee worktree unregister --id herdr-gateway--wt--apo-throwaway-spawn
```

Observed results:
- `herdr pane close w7:pT` → `{"type":"ok"}`; `herdr pane get w7:pT` →
  `pane_not_found` immediately after.
- Pane count in workspace `w7` after close: **5** — back to the pre-split
  baseline.
- `git worktree remove --force` → exit 0.
- `git branch -d wt/apo-throwaway-spawn` → `Deleted branch
  wt/apo-throwaway-spawn (was 35732b6).` (plain `-d`, not `-D`, succeeded —
  the branch had zero commits beyond the base it forked from, so it was
  trivially merged).
- `.bee/bin/bee worktree unregister --id
  herdr-gateway--wt--apo-throwaway-spawn` → `Removed worktree grant for id
  herdr-gateway--wt--apo-throwaway-spawn.`
- `git worktree list` afterward: no `apo-throwaway-spawn` row.
- `git branch --list 'wt/apo-throwaway-spawn'`: empty.
- `.bee/bin/bee worktree list --json` afterward: `grants` has no
  key containing `apo-throwaway-spawn` (other concurrent worktrees' grants
  in this live workspace are unrelated and untouched).

## Takeaways for §8

1. `herdr pane split` followed by `herdr agent start` with matching
   `--cwd`/`--workspace`/`--tab` does **not** reuse the split pane — it
   opens a second, independent pane. As written, §8 leaves a stray empty
   pane behind on every dispatch unless it is closed. Either drop step 2
   (let `agent start` open the pane on its own — it already places the pane
   in the requested workspace/tab at the requested cwd) or explicitly close
   the step-2 pane after step 3 confirms the agent's own pane is live.
2. Never spawn the working agent with `-p`/headless mode for anything meant
   to stay observable or unattended-long-running — the pane closes the
   instant the process exits, which is also why it would be the wrong shape
   for the real dispatch loop's runtime agents (they are meant to keep
   running, not exit after one turn).
3. Self-naming is not automatic from `AGENTS.md`/`CLAUDE.md` alone in
   this proof run (no unprompted self-naming instruction fired before the
   bounded prompt drove it) — it happened here because the initial prompt
   explicitly told the agent to run `herdr pane current --current` then
   `herdr pane rename`. A real runtime agent (that reads `AGENTS.md` and
   picks up bee work on its own) still needs an explicit self-naming step
   folded into its own onboarding instructions to self-name unattended.
