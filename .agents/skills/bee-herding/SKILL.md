---
name: bee-herding
description: >-
  Drive the bee-herding cockpit's three roles — bootstrap (one-shot human setup that pre-flights and turns the cockpit on), dispatch (one cold control-loop iteration that starts safe backlog work in a fresh worktree), and merge (an owner gesture, single-shot, that lands finished worktrees in main). Use when a human invokes bootstrap directly (no --role given), or when control-loop.sh runs exactly one iteration as --role dispatch|merge — each invocation is fresh, with no memory of any earlier one. Not for feature work inside a worktree — that belongs to the working agent's own bee chain.
metadata:
  version: '0.2'
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
      reason: Every pane/tab/agent action in either role goes through the herdr binary directly — there is no other way to reach a pane.
---

# Herding — the unattended cockpit

Three roles drive a herdr cockpit over one repo. A human invokes
**bootstrap** directly — no `--role` — to pre-flight and turn the
cockpit on, once. After that, every control action is a **fresh,
memoryless invocation**: `control-loop.sh --role dispatch` runs one
cold iteration per interval, and **merge** is an owner gesture,
single-shot, never looped. Nothing carries over between invocations
except what bee state, git, and the herdr workspace durably record —
read your role's reference in full before acting, then act once,
report into the chat pane, and exit; the loop (or the owner's next
gesture) starts the next invocation cold.

## The three roles

**Bootstrap** — one-shot human setup, start to finish in one turn:
resolve the main root, pre-flight (main clean; gate-bypass level at
full/total — never raise it yourself), resolve the workspace, refuse
if a cockpit already exists, run the bootstrap script. It starts ONLY
the dispatch loop; merge stays an owner gesture. Protocol:
`references/role-bootstrap.md`.

**Dispatch** — one cold iteration that only STARTS work, in an
isolated worktree, never touching main: self-name, check the bypass
level, find the chat pane, count occupied runtime slots and report
anomalies once, build the dispatchable set only past the enable
interlock, apply the two-key lane-safety filter (script AND your own
reading), rank, announce before acting, spawn the working agent.
`--dry-run` reports the whole decision and changes nothing. Protocol:
`references/role-dispatch.md`.

**Merge** — the one action that lands work in main, so a human is
present: find finished worktrees from bee's own record only, honor
red-stop markers, merge and clean up each finished worktree, close its
pane — and STOP COLD on a red verify, never retry. Protocol:
`references/role-merge.md`.

## Role boundary

Bootstrap only builds the cockpit and starts the loops — never picks a
PBI, creates a worktree beyond layout, or merges one. Dispatch only
starts work — never merges, deletes a worktree, or closes a pane.
Merge only retires finished work — never picks a PBI, creates a
worktree, or starts an agent. About to take another role's action?
Stop — wrong section.

## Safety boundaries

Full record for each: `references/operational-invariants.md`.

- **The enable interlock contains dispatch.** Dispatch builds nothing
  without the owner's durable enable marker — every other safety only
  matters once the loop is allowed to run at all.
- **Merge stays human.** Nothing lands in main unattended; the
  highest-authority action in the system requires an owner present.
- **Worktree isolation is a git boundary, not a sandbox.** Working
  agents run `bypassPermissions` with no allowlist — an accepted,
  owner-owned risk, bounded by worktree/branch isolation. Control
  panes run an enumerated `--allowedTools` surface, never
  `bypassPermissions` and never "read-only" — both roles genuinely
  write.
- **Lane safety is advisory, not containment.** Treat the classifier's
  safe verdict as "no keyword hit," never as "safe" — your own
  fail-closed reading of the record is the key the refusal actually
  depends on.
- **Stop is for the loop, not the agents.** The stop file halts the
  control loop at the next iteration boundary; agents already running
  never read it — close their panes to stop them. Removing the file
  allows a restart (re-run bootstrap; it never self-restarts).
- **Carry nothing over.** Assumptions from "an earlier iteration" are
  the defect this design kills — only bee state, git, and the herdr
  workspace persist.

## Runtime adapter

Both spawn points — the working agent's argv and the control pane's
invocation — read an optional `.bee/config.json` seam
(`herding.agent_command` / `herding.control_command`, argv-token
arrays). Absent, the commands are byte-equivalent to the defaults —
zero behavior change. Substitution is per-token, never join-then-split,
never `eval`. Shape and examples:
`references/operational-invariants.md` ("Runtime adapter").

## References

| File | Contents |
|---|---|
| `references/role-bootstrap.md` | Bootstrap protocol in full |
| `references/role-dispatch.md` | Dispatch protocol + quick reference |
| `references/role-merge.md` | Merge protocol + quick reference |
| `references/operational-invariants.md` | Permission posture, runtime adapter, containment, stop/resume in full |
| `references/dispatch-dry-run.md` | Recorded dry-run proof |
| `references/spawn-proof.md` | Recorded live spawn proof |
| `references/dispatch-prompt.md`, `references/merge-prompt.md` | Control pane opening prompts |
