# bee-herding — operational invariants

Full text of the four cross-cutting rules the body only summarizes: permission
posture, the runtime adapter seam, what actually contains this system, and
stop/resume semantics.

## Permission posture

The two halves of this system run under deliberately different permission
postures. The split is coupled and recorded here rather than decided silently.

**Working agents — `bypassPermissions`, no allowlist. This is an accepted
risk, owned by the operator.**

> Accepted risk (owner decision): every working agent this loop
> spawns runs `claude --permission-mode bypassPermissions` with no tool
> allowlist. It can run any command, edit any file, and reach anything the
> machine's user can, unattended and unsupervised. This posture is accepted
> knowingly, because a narrowed working agent stalls forever the first time it
> hits a permission prompt with no TTY, which defeats the whole point of
> unattended dispatch. **Blast radius:** each working agent is confined to its
> own git worktree and its own branch (`wt/<slug>`), so its edits do not touch
> main or any other agent's worktree until a merge — but "confined to a
> worktree" is a filesystem-and-git boundary, not a security sandbox: the
> agent shares the machine, the network, the user's credentials, and every
> ambient tool. The lane filter chooses *which item* is picked up; it does
> **not** constrain *what commands* the agent may run. What actually bounds
> the damage is the containment list below, not the filter — and none of it
> is a sandbox.

**Control panes — enumerated command surface, never `bypassPermissions`,
never "read-only".** `control-loop.sh` starts each control pane
under an enumerated `--allowedTools` list sized to exactly what that role
measurably does. It is not read-only, because both control roles genuinely
write: **dispatch** runs `bee worktree new` (creates a worktree and registers
a grant); **merge** runs `git merge --abort` on main, writes `.bee/tmp/`
markers, and runs `bee worktree merge --cleanup` (deletes a branch, removes a
worktree). Taken literally, "read-only" would give a dispatch pane that
cannot dispatch and a merge pane that cannot merge — a silent stall every
interval, the exact failure this feature exists to kill. The two halves of
the posture are **coupled, not separable**: the merge pane runs the project's
verify against the just-merged tree, so it **executes code the unsandboxed
working agents wrote**. Narrowing the control panes buys one thing honestly —
it stops a cold control model at thousands of iterations a day from
"helpfully" improvising a command outside its job (e.g. cleaning a dirty
main); it does **not** sandbox the agent-authored code that verify runs. The
exact allowlist per role, and the note that it must grow if a role gains a
command, live in `control-loop.sh`.

## Runtime adapter

Config-driven spawn commands. Both spawn points — the
working agent's trailing argv (Dispatch role §8), and the control pane's real
invocation inside `control-loop.sh` — read from an optional `.bee/config.json`
command-template seam instead of a hardcoded string. **With no `herding`
config keys at all, every spawned command is BYTE-EQUIVALENT to what this
skill has always run — zero behavior change.** This is an adapter seam, not a
new runtime: full codex-native herding (its own event loop, its own pane
protocol) stays out of scope.

Two independent keys, each a JSON array of argv-token strings:

- **`herding.agent_command`** — the WORKING agent's spawn argv (the tail of
  `herdr agent start ... --`, Dispatch role §8 step 2). Placeholder:
  `{MODEL}` (the fixed model, `sonnet`). Default when absent:
  `["claude", "--model", "sonnet", "--permission-mode", "bypassPermissions"]`
  — exactly today's string.
- **`herding.control_command`** — the CONTROL pane's real invocation inside
  `control-loop.sh`'s `run_iteration`. Placeholders: `{PROMPT}`, `{MODEL}`,
  `{MAX_TURNS}`, `{ALLOWED_TOOLS}`. Default when absent:
  `["claude", "-p", "{PROMPT}", "--model", "sonnet", "--max-turns",
  "{MAX_TURNS}", "--allowedTools", "{ALLOWED_TOOLS}"]` — exactly today's
  invocation.

Example `.bee/config.json` fragment (both keys are optional and independent —
set either, both, or neither):

```json
{
  "herding": {
    "agent_command": ["claude", "--model", "{MODEL}", "--permission-mode", "bypassPermissions"],
    "control_command": ["claude", "-p", "{PROMPT}", "--model", "{MODEL}", "--max-turns", "{MAX_TURNS}", "--allowedTools", "{ALLOWED_TOOLS}"]
  }
}
```

**Substitution is per-token, never a join-then-re-split and never `eval`** —
this is the shell-injection-safe shape the design requires. Each array
element is substituted and passed as one discrete argv element; a value
containing spaces, quotes, or shell metacharacters (the free-form `{PROMPT}`
text, in particular) lands as the literal content of that one argument and
can never spill into another argument or be reinterpreted as a shell
operator. `control-loop.sh`'s `read_command_template`/`substitute_placeholders`
functions are the reference implementation for `control_command`; a
dispatch-role agent applies the identical per-token substitution itself when
building `agent_command` for §8 (there is no script to call — the
working-agent spawn line is issued live by whichever agent is running the
dispatch role).

**Codex adapter example — illustrative only, not a supported native herding
mode:**

```json
{
  "herding": {
    "control_command": ["codex", "exec", "-m", "{MODEL}", "-s", "workspace-write", "{PROMPT}"]
  }
}
```

This shows the shape a codex-backed control pane's command COULD take under
the adapter seam. It is not wired into, or validated against, an actual codex
control-loop run in this repo — the event loop and pane protocol both still
assume a `claude` session underneath (Merge role / Dispatch role in
SKILL.md). Treat it as a documented starting point for a future adapter, not
a claim that codex control panes work today.

## What actually contains this

Do not assume the loop "will not pick up hard-gate work" —
that was measured false: the lane-safety classifier
passed **8 of 8** real backlog rows in an adversarial review, including one
whose story was "delete the entire JS runtime," because it matches an English
keyword list against a row that judges work by its title, and most real rows
are not in English. Do not rely on the lane filter as a containment. What
actually contains this system, in descending order of load:

1. **The enable interlock (Dispatch role §5)** — dispatch builds nothing
   at all without the owner's `bee-herding.enable` marker. This is the gate
   that decides whether the loop does anything; everything else only matters
   once it is running.
2. **Merge is an owner gesture, not a loop** — nothing lands in main
   unattended. The single highest-authority action in the system requires a
   human present.
3. **Worktree isolation** — each working agent's edits are confined to its
   own worktree and branch until a merge (a git boundary, not a security
   sandbox — see "Permission posture" above).
4. **The four-slot cap and the stop file** — bound concurrency and give the
   human an off switch (which stops the *control* loop, not agents already
   running — see "Stop and resume" below).
5. **Key-2 reading (Dispatch role §6)** — the dispatcher's own reading of
   each row, refusing when unsure. Advisory, fail-closed on refusal, but not
   a mechanical guarantee.

The lane classifier script is one advisory input to item 5, not a
containment in its own right. Treat every `lane_safe:true` from it as "no
obvious English keyword hit," never as "safe."

## Stop and resume

`touch <main-root>/.bee/tmp/bee-herding.stop` stops the **control loop**
(dispatch) at the next iteration boundary — `control-loop.sh` checks the file
both before and after every iteration, so a stop created mid-iteration takes
effect at that boundary rather than a full interval later. Removing the file
lets the loop be started again (it does not restart on its own — re-run
bootstrap).

**The stop file does not stop working agents already running.** Each working
agent is an independent `claude` session in its own runtime pane and
worktree; the stop file is never read by them. To stop those, close their
panes (`herdr pane close <pane_id>`) or open a pane and talk to the agent
directly. Its worktree survives either way (`bee worktree list` shows it).
Stopping the dispatch loop only guarantees no *new* agents are spawned — not
that in-flight ones halt.
