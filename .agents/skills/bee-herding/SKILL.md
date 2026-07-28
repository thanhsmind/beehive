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
      reason: Every pane/tab/agent action in either role goes through the herdr binary directly — there is no other way to reach a pane.
---

# bee-herding — dispatch and merge roles

Managed `bee-*` plugin skill: source `skills/bee-herding/`, rendered to
`.claude/skills/bee-herding/` and `.agents/skills/bee-herding/` at install.
The `bee-` prefix is mandatory — distribution preflight hard-refuses any
non-matching skill directory, and the plugin render copies only `bee-*`
dirs, so a differently-named skill would install for nobody. Rules below
are stated bare — provenance: `references/provenance.md`.

It drives three roles. A human invokes **bootstrap** directly — no
`--role` — to pre-flight and turn the cockpit on; one-shot, run once to
completion, not a repeating iteration. Once up, the cockpit's two control
panes run **asymmetrically**: **dispatch** — starts work in isolated
worktrees, never touches main — runs as a bounded `control-loop.sh` loop,
a brand-new cold `claude -p` process every interval; **merge** — alone
lands work in main — is NOT looped: an **owner gesture**, single-shot on
request. `control-loop.sh` picks the role via `--role dispatch|merge`;
nothing carries over between invocations except what bee state, git, and
the herdr workspace durably record. Read the whole section for your role
before doing anything: **you have no memory of any earlier iteration**
(bootstrap excepted — it runs once, start to finish, in one invocation).

**Role boundary.** Bootstrap only builds the cockpit and starts the
dispatch and merge loops — never picks a PBI, creates a worktree beyond
layout, or merges one. Dispatch only starts work — never merges, deletes a
worktree, or closes a pane. Merge only retires finished work — never picks
a PBI, creates a worktree, or starts an agent. About to take another
role's action? Stop — wrong section.

## Bootstrap role

One-shot, human-invoked, start to finish in one turn. Resolve main root ->
pre-flight (main clean; `gate_bypass_level` `full`/`total`) -> resolve
workspace id -> refuse if a cockpit already exists -> run the bootstrap
script (starts ONLY the dispatch loop; merge stays an owner gesture). Full
protocol: `references/role-bootstrap.md`.

## Dispatch role

A cold, bounded loop iteration: only STARTS work, in isolated worktrees,
never touching main. Locate and self-name -> refuse below
`gate_bypass: full` -> find the chat pane -> count occupied runtime
slots, report anomalies once -> build the dispatchable set
past the enable interlock -> two-key lane-safety filter, script
AND your own reading -> rank and announce before acting ->
spawn the working agent via herdr. `--dry-run` reports the
whole decision and changes nothing. Full protocol + quick reference:
`references/role-dispatch.md`.

## Merge role

An owner GESTURE, single-shot, never looped — alone lands work in main,
never picks a PBI, creates a worktree, or starts an agent. Locate and
self-name -> find the chat pane -> find finished worktrees from bee's own
record only -> check the red-stop marker before merging anything
-> merge and clean up each finished worktree, closing its pane —
STOP COLD on a red verify, never retry. Full protocol:
`references/role-merge.md`.

## Operational invariants

**Permission posture:** working agents run
`bypassPermissions`, no allowlist — an accepted, owner-owned risk, bounded
only by worktree/branch isolation (a git boundary, not a sandbox). Control
panes run an enumerated `--allowedTools` surface, never
`bypassPermissions`, never "read-only" — both genuinely write (dispatch:
`bee worktree new`; merge: `git merge --abort`, `.bee/tmp/` markers,
`bee worktree merge --cleanup`). Full accepted-risk record:
`references/operational-invariants.md` ("Permission posture").

**Runtime adapter:** both spawn points — the working
agent's argv and the control pane's invocation — read an optional
`.bee/config.json` seam (`herding.agent_command` / `herding.control_command`,
each an argv-token array). Absent -> byte-equivalent to today's hardcoded
strings, zero behavior change. Substitution is per-token, never
join-then-split, never `eval`. Full config shape + example:
`references/operational-invariants.md` ("Runtime adapter").

**What actually contains this:** the lane-safety classifier is
advisory, not containment — measured false: 8/8 adversarial backlog rows
passed, English-keyword matching a mostly non-English backlog. Real
containment, descending load: (1) the enable interlock — dispatch builds
nothing without the owner's marker; (2) merge is an owner gesture, not a
loop; (3) worktree isolation (a git boundary, not a sandbox); (4) the
four-slot cap + stop file; (5) Key-2 reading — the dispatcher's own
judgement, fail-closed on refusal, advisory not mechanical. Full record:
`references/operational-invariants.md` ("What actually contains this").

**Stop and resume:** `touch <main-root>/.bee/tmp/bee-herding.stop` stops
the CONTROL loop (dispatch) at the next iteration boundary; removing it
allows a restart (it never self-restarts — re-run bootstrap). **Does NOT
stop working agents already running** — each is its own `claude` session
that never reads the stop file; close their panes to stop them. Full
detail: `references/operational-invariants.md` ("Stop and resume").

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
| `references/provenance.md` | Body rule -> decision tag map |
