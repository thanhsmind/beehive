---
name: bee-herding
description: >-
  Drives the bee-herding cockpit's three roles: bootstrap is a one-shot setup a human invokes directly (no `--role` given) to pre-flight and turn the cockpit on, starting ONLY the dispatch loop; dispatch — enabled only once the owner creates an enable marker — picks the highest-impact ready backlog item, refuses unsafe or unclassifiable work, and starts a working agent in a fresh worktree via the herdr CLI; merge is an owner GESTURE run single-shot (not looped), finding worktrees finished by bee's own state, merging and cleaning them up, closing their runtime pane, and stopping cold — never retrying — on a red verify. Use bootstrap for that one direct human invocation; use dispatch/merge for exactly one control iteration at a time, in the role named by `--role dispatch|merge` — each invocation is fresh, with no memory of any earlier one.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: unavailable
      reason: The dispatchable-set build, lane classification, and the merge role's worktree-finished checks all run through the vendored .bee/bin helpers and scripts/classify-lane.mjs.
    herdr-cli:
      kind: command
      command: herdr
      missing_effect: unavailable
      reason: Every pane/tab/agent action in either role goes through the herdr binary directly (D8) — there is no other way to reach a pane.
---

# bee-herding — dispatch and merge roles

This skill ships as a managed `bee-*` plugin skill: its source is `skills/bee-herding/`, and it is rendered into the per-runtime skill roots (`.claude/skills/bee-herding/`, `.agents/skills/bee-herding/`) at install. The `bee-` prefix is mandatory — the distribution preflight hard-refuses any skill directory that does not match `^bee-[a-z0-9-]+$`, and the plugin render only copies `bee-*` directories, so a differently-named skill would refuse at install for every user and reach nobody. The `(D…)` tags below are this skill's own internal design-decision shorthand; they do not resolve to any file in this repo.

It drives three roles. A human invokes **bootstrap** directly — no `--role` given — to run the pre-flight checks and turn the cockpit on; this is a one-shot setup action, run once to completion, not a repeating iteration. Once the cockpit is up, the two control panes of the cockpit tab (D13) run **asymmetrically (D11)**: **dispatch** — which only starts work in isolated worktrees, never touching main — runs as a bounded `control-loop.sh` loop, invoked as a brand-new, cold `claude -p` process every interval; **merge** — which alone lands work in main — is NOT looped. Bootstrap starts only the dispatch loop; merge is an **owner gesture**, run single-shot on request (`control-loop.sh --role merge --once`, or this skill's merge section for one pass), so a human is present whenever anything merges. `control-loop.sh` picks which role's section below to follow via `--role dispatch|merge` — nothing carries over between invocations except what is durably recorded in bee state, git, and the herdr workspace itself. Read the whole section for your role before doing anything: **you have no memory of any earlier iteration** (bootstrap is the exception — it runs once, start to finish, in a single invocation, so this concern does not apply to it). Every fact any role needs is either written in this file or read live, right now, from bee/herdr/git. Never assume "I already checked that" — you didn't; a different process did, or nobody did.

**Role boundary.** Bootstrap only builds the cockpit/runtime layout and starts the dispatch and merge loops: it never picks a PBI, creates a worktree beyond what the layout needs, or merges one — those are the dispatch and merge loops' own job, running afterward as their own cold iterations. Dispatch only starts work: it never merges a branch back into main, deletes a worktree, or closes a pane. Merge only retires finished work: it never picks a PBI, creates a worktree, or starts a working agent. If you find yourself about to take another role's action, stop — you are following the wrong section.

## Bootstrap role

One-shot, human-invoked, run start to finish in a single invocation. Steps: resolve the main root → pre-flight (main clean; `gate_bypass_level` `full`/`total`, D6) → resolve the workspace id → refuse if a cockpit already exists → run the bootstrap script (starts ONLY the dispatch loop; merge stays an owner gesture). Full protocol: `references/role-bootstrap.md`.

## Dispatch role

A cold, bounded loop iteration: it only STARTS work, in isolated worktrees, never touching main. Steps: locate yourself and self-name (D17) → refuse below `gate_bypass: full` (D6) → find the chat pane → count occupied runtime slots and report anomalies once (D5/D18/D20) → build the dispatchable set past the enable interlock (D1/D10) → the two-key lane-safety filter, script AND your own reading (D6) → rank and announce before acting (D16) → spawn the working agent via herdr (D14/D9/D22/D4). `--dry-run` reports the whole decision and changes nothing. Full protocol and quick reference: `references/role-dispatch.md`.

## Merge role

An owner GESTURE, single-shot, never looped — it alone lands work in main and never picks a PBI, creates a worktree, or starts an agent. Steps: locate yourself and self-name → find the chat pane → find finished worktrees from bee's own record only (D2/D20) → check the red-stop marker before merging anything (D3/D18) → merge and clean up each finished worktree, closing its pane — and STOP COLD on a red verify, never retry (D3/D15/D19). Full protocol: `references/role-merge.md`.

## Permission posture — the accepted risk, on the record (D7-FINAL / D22)

The two halves of this system run under deliberately different permission postures. The split is coupled and recorded here rather than decided silently.

**Working agents — `bypassPermissions`, no allowlist. This is an accepted risk, owned by the operator.**

> Accepted risk (owner decision D7-FINAL): every working agent this loop spawns runs `claude --permission-mode bypassPermissions` with no tool allowlist. It can run any command, edit any file, and reach anything the machine's user can, unattended and unsupervised. This posture is accepted knowingly, because a narrowed working agent stalls forever the first time it hits a permission prompt with no TTY, which defeats the whole point of unattended dispatch. **Blast radius:** each working agent is confined to its own git worktree and its own branch (`wt/<slug>`), so its edits do not touch main or any other agent's worktree until a merge — but "confined to a worktree" is a filesystem-and-git boundary, not a security sandbox: the agent shares the machine, the network, the user's credentials, and every ambient tool. The lane filter chooses *which item* is picked up; it does **not** constrain *what commands* the agent may run. What actually bounds the damage is the set below, not the filter — and none of it is a sandbox.

**Control panes — enumerated command surface, never `bypassPermissions`, never "read-only" (D7-FINAL).** `control-loop.sh` starts each control pane under an enumerated `--allowedTools` list sized to exactly what that role measurably does. It is not read-only, because both control roles genuinely write: **dispatch** runs `bee worktree new` (creates a worktree and registers a grant); **merge** runs `git merge --abort` on main, writes `.bee/tmp/` markers, and runs `bee worktree merge --cleanup` (deletes a branch, removes a worktree). Taken literally, "read-only" would give a dispatch pane that cannot dispatch and a merge pane that cannot merge — a silent stall every interval, the exact failure this feature exists to kill. The two halves of the posture are **coupled, not separable**: the merge pane runs the project's verify against the just-merged tree, so it **executes code the unsandboxed working agents wrote**. Narrowing the control panes buys one thing honestly — it stops a cold control model at thousands of iterations a day from "helpfully" improvising a command outside its job (e.g. cleaning a dirty main); it does **not** sandbox the agent-authored code that verify runs. The exact allowlist per role, and the note that it must grow if a role gains a command, live in `control-loop.sh`.

## Herding runtime adapter — config-driven spawn commands (D4, i54-closeout-4)

Both spawn points documented above — the working agent's trailing argv in §8, and the control pane's real invocation inside `control-loop.sh` — read from an optional `.bee/config.json` command-template seam instead of a hardcoded string. **With no `herding` config keys at all, every spawned command is BYTE-EQUIVALENT to what this skill has always run — zero behavior change.** This is an adapter seam, not a new runtime: full codex-native herding (its own event loop, its own pane protocol) stays out of scope (CONTEXT.md D4, out of scope).

Two independent keys, each a JSON array of argv-token strings:

- **`herding.agent_command`** — the WORKING agent's spawn argv (the tail of `herdr agent start ... --`, §8 step 2). Placeholder: `{MODEL}` (D4's fixed model, `sonnet`). Default when absent: `["claude", "--model", "sonnet", "--permission-mode", "bypassPermissions"]` — exactly today's string.
- **`herding.control_command`** — the CONTROL pane's real invocation inside `control-loop.sh`'s `run_iteration`. Placeholders: `{PROMPT}`, `{MODEL}`, `{MAX_TURNS}`, `{ALLOWED_TOOLS}`. Default when absent: `["claude", "-p", "{PROMPT}", "--model", "sonnet", "--max-turns", "{MAX_TURNS}", "--allowedTools", "{ALLOWED_TOOLS}"]` — exactly today's invocation.

Example `.bee/config.json` fragment (both keys are optional and independent — set either, both, or neither):

```json
{
  "herding": {
    "agent_command": ["claude", "--model", "{MODEL}", "--permission-mode", "bypassPermissions"],
    "control_command": ["claude", "-p", "{PROMPT}", "--model", "{MODEL}", "--max-turns", "{MAX_TURNS}", "--allowedTools", "{ALLOWED_TOOLS}"]
  }
}
```

**Substitution is per-token, never a join-then-re-split and never `eval`** — this is the shell-injection-safe shape the design requires. Each array element is substituted and passed as one discrete argv element; a value containing spaces, quotes, or shell metacharacters (the free-form `{PROMPT}` text, in particular) lands as the literal content of that one argument and can never spill into another argument or be reinterpreted as a shell operator. `control-loop.sh`'s `read_command_template`/`substitute_placeholders` functions are the reference implementation for `control_command`; a dispatch-role agent applies the identical per-token substitution itself when building `agent_command` for §8 (there is no script to call — the working-agent spawn line is issued live by whichever agent is running the dispatch role).

**Codex adapter example — illustrative only, not a supported native herding mode:**

```json
{
  "herding": {
    "control_command": ["codex", "exec", "-m", "{MODEL}", "-s", "workspace-write", "{PROMPT}"]
  }
}
```

This shows the shape a codex-backed control pane's command COULD take under the adapter seam. It is not wired into, or validated against, an actual codex control-loop run in this repo — the event loop and pane protocol both still assume a `claude` session underneath (Merge role / Dispatch role sections above). Treat it as a documented starting point for a future adapter, not a claim that codex control panes work today.

## What actually contains this — and what does not (D6, corrected)

An earlier version of this skill claimed the loop "will not pick up hard-gate work." That was measured false: the lane-safety classifier passed **8 of 8** real backlog rows in an adversarial review, including one whose story was "delete the entire JS runtime," because it matches an English keyword list against a row that judges work by its title, and most real rows are not in English. Do not rely on the lane filter as a containment. What actually contains this system, in descending order of load:

1. **The enable interlock (§5, D10)** — dispatch builds nothing at all without the owner's `bee-herding.enable` marker. This is the gate that decides whether the loop does anything; everything else only matters once it is running.
2. **Merge is an owner gesture, not a loop (D11)** — nothing lands in main unattended. The single highest-authority action in the system requires a human present.
3. **Worktree isolation** — each working agent's edits are confined to its own worktree and branch until a merge (a git boundary, not a security sandbox — see the accepted-risk record above).
4. **The four-slot cap and the stop file** — bound concurrency and give the human an off switch (which stops the *control* loop, not agents already running — see "Stop and resume").
5. **Key-2 reading (§6)** — the dispatcher's own reading of each row, refusing when unsure. Advisory, fail-closed on refusal, but not a mechanical guarantee.

The lane classifier script is one advisory input to item 5, not a containment in its own right. Treat every `lane_safe:true` from it as "no obvious English keyword hit," never as "safe."

## Stop and resume — and what the stop file does NOT stop

`touch <main-root>/.bee/tmp/bee-herding.stop` stops the **control loop** (dispatch) at the next iteration boundary — `control-loop.sh` checks the file both before and after every iteration, so a stop created mid-iteration takes effect at that boundary rather than a full interval later. Removing the file lets the loop be started again (it does not restart on its own — re-run bootstrap).

**The stop file does not stop working agents already running.** Each working agent is an independent `claude` session in its own runtime pane and worktree; the stop file is never read by them. To stop those, close their panes (`herdr pane close <pane_id>`) or open a pane and talk to the agent directly. Its worktree survives either way (`bee worktree list` shows it). Stopping the dispatch loop only guarantees no *new* agents are spawned — not that in-flight ones halt.
