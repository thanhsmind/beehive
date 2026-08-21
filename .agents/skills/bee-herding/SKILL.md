---
name: bee-herding
description: >-
  Drive the bee-herding cockpit's three roles — bootstrap (one-shot human setup that pre-flights and turns the cockpit on), dispatch (one cold control-loop iteration that starts safe backlog work in a fresh worktree), and merge (an owner gesture, single-shot, that lands finished worktrees in main). Use when a human invokes bootstrap directly (no --role given), or when bee herding control-loop runs exactly one iteration as --role dispatch|merge — each invocation is fresh, with no memory of any earlier one. Not for feature work inside a worktree — that belongs to the working agent's own bee chain.
metadata:
  version: '0.2'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: unavailable
      reason: The dispatchable-set build, lane classification, the enable interlock, and the merge role's worktree-finished checks all run through the vendored bee binary (`bee herding classify-lane`, `bee herding interlock`, `bee worktree ...`). The binary is vendored into the repo by onboarding; no Node runtime is involved.
    herdr-cli:
      kind: command
      command: herdr
      missing_effect: unavailable
      reason: Every pane/tab/agent action in either role goes through the herdr binary directly — there is no other way to reach a pane.
---

# Herding — the unattended cockpit

## The three roles

Read your role's reference in full before acting, then act once,
report into the chat pane, and exit; the loop (or the owner's next
gesture) starts the next invocation cold.

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

**Merge** — an owner gesture, single-shot, never looped; the one
action that lands work in main, so a human is present: find finished
worktrees from bee's own record only, honor
red-stop markers, merge and clean up each finished worktree, close its
pane — and STOP COLD on `MERGE_CONFLICT` or `WORKTREE_MERGE_PROOF_DEBT`,
never retry (the proof check runs before `git merge`; this role runs no
verify command). Protocol: `references/role-merge.md`.

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
- **Carry nothing over.** Never act on an assumption from an earlier
  iteration — only bee state, git, and the herdr workspace persist.

## Runtime adapter

Both spawn points — the working agent's argv and the control pane's
invocation — read an optional `.bee/config.json` seam
(`herding.agent_command` / `herding.control_command`, argv-token
arrays). Absent, the commands are byte-equivalent to the defaults —
zero behavior change. Substitution is per-token, never join-then-split,
never `eval`. Shape and examples:
`references/operational-invariants.md` ("Runtime adapter").

## Waves — briefing several workers at once

`bee herding wave` is a SEPARATE shape from the three roles, and no role
calls it. Dispatch starts one worker per iteration and never speaks to
it again; a wave briefs N already-running panes in one act, waits on
all of them at the same time, and records the run as one ledger row.
It has no interlock, no classifier and no gate, so it is never the way
to start the cockpit's ordinary backlog work — only a fan-out over
panes that already exist. Three things surprise every first caller: it
creates no panes and no worktrees (splitting is yours), one
unresolvable target stops the whole run before any brief is sent, and
its `success` is `false` even on a perfect run because no completion
signal exists — read the ledger row and the panes instead. Protocol:
`references/wave-runs.md`.

## References

| File | When to load |
|---|---|
| `references/role-bootstrap.md` | You are the bootstrap role — read the full protocol before any pre-flight action |
| `references/role-dispatch.md` | You are the dispatch role — read the full protocol (plus quick reference) before building the dispatchable set |
| `references/role-merge.md` | You are the merge role — read the full protocol (plus quick reference) before touching any worktree |
| `references/wave-runs.md` | Running `bee herding wave` — what it does not do, the input shape, and why `success` is not the thing to read |
| `references/operational-invariants.md` | A safety boundary needs its full record — permission posture, runtime adapter, containment, stop/resume |
| `references/dispatch-dry-run.md` | Auditing what a dispatch iteration decides — the recorded dry-run proof |
| `references/spawn-proof.md` | Auditing a live spawn end to end — the recorded proof |
| `references/dispatch-prompt.md`, `references/merge-prompt.md` | Opening a control pane and needing its exact opening prompt |
