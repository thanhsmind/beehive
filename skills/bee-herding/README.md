# bee-herding — operator's guide

`SKILL.md` is written for the agent. This is written for you.

## What you get

One herdr workspace, two tabs:

```
cockpit ─┬─ chat      (yours)
         ├─ dispatch  (loops: starts work in isolated worktrees)
         └─ merge     (idle: an owner gesture you run by hand)

runtime ─── up to 4 working agents, one per backlog item, each in its own worktree
```

Every interval, **dispatch** looks for a free runtime slot and a ready backlog item, and starts an agent on it — but only if you have turned dispatch on (see step 3 below), and only in an isolated worktree that never touches main. **Merge is not a loop.** It is a gesture you run by hand when you want finished worktrees retired: it merges them into main, runs verify, and closes their panes. That asymmetry is deliberate — merge is the one action that lands work in main, so a human is present whenever it happens.

## Before you turn it on

**1. Main must be clean.** `bee worktree merge` refuses on a dirty main, and the merge role runs *in* main. Check:

```
git -C <main-root> status --porcelain
```

If bee's session logs show up there, untrack them once (they are meant to be ignored):

```
git -C <main-root> rm --cached .bee/logs/*.jsonl
git -C <main-root> commit -m "chore: untrack bee session logs"
```

**2. `gate_bypass` must be `full` or `total`.** Dispatch refuses to operate below that and will tell you so, every cycle. Check with `bee status --json`. This is deliberate: an agent working unattended must not inherit `normal`'s latitude for hard-gate work.

**3. You must explicitly enable dispatch — it will not run on its own.** Dispatch refuses to build *any* dispatchable set until you create an owner enable marker:

```
touch <main-root>/.bee/tmp/bee-herding.enable
```

The CLI verbs that used to spell this — `bee herding enable`, `bee herding disable`, `bee herding status` — are **not built into the current binary** (they were never ported off Node and now refuse by name). The `touch`/`rm` above IS the gesture, byte for byte; it was always owner-typed and never called by bee automation, so nothing is lost.

Remove the file to disable dispatch again (it takes effect at the next interval). This interlock is deliberate and load-bearing: this repo's **ordinary post-exploring state is already the dispatchable state** — the moment a feature finishes exploring, its row is `in-flight` with a slug, a CONTEXT.md, no worktree and no cells, which is every condition below. Without the marker, dispatch would start picking up whatever exploring last produced, unattended. The marker is your explicit "yes, run this now." Nothing else creates it; no agent creates it; only you.

**4. There must be something ready.** The loop does not invent work — it picks up items *you* have already taken through exploring. Once dispatch is enabled, an item is dispatchable only when **all four** hold:

- its backlog row is `in-flight` and carries a `` Feature `<slug>` `` annotation
- `docs/history/<slug>/CONTEXT.md` exists (i.e. it passed Gate 1)
- no worktree exists for it yet
- it has no cells yet

If nothing satisfies that, the loop runs correctly and does nothing. That is not a bug — it means the queue is empty, and filling it is your job.

## Turn it on

```
herdr workspace list                     # find your workspace id
bash .claude/skills/bee-herding/scripts/bootstrap-cockpit.sh \
    --workspace <id> --main-root <absolute path to the MAIN checkout>
```

`--main-root` is required and must be the main checkout, never a worktree — `bee worktree new` and `bee worktree merge` both refuse from inside a linked worktree, and a misrooted dispatcher would fail every cycle while dutifully continuing.

This starts **only the dispatch loop.** The merge pane is created but left idle — merge is a gesture you run by hand (see "Retiring finished work" below). And remember: dispatch does nothing until you create the enable marker (step 3 above).

Useful first: `--dry-run` prints the herdr commands and executes nothing. `--no-start` builds the layout without launching the loop.

## Retiring finished work (the merge gesture)

When you want finished worktrees merged into main, run merge yourself, in the merge pane (or any shell rooted at the main checkout):

```
bash .claude/skills/bee-herding/scripts/control-loop.sh \
    --role merge --main-root <main-root> --timeout 5400 --once
```

It makes one pass: merges each finished worktree, runs verify, closes the merged pane, and stops cold — no retry — on a red verify. Then it exits. Nothing merges again until you run it again. This is the point: the highest-authority action in the system never happens while you are away.

**Or invoke the skill directly.** Instead of running the script yourself, you can just invoke `bee-herding` with no `--role` given — the agent resolves `<main-root>` and the workspace id itself, runs the same two pre-flight checks above (main clean, `gate_bypass_level` full/total), checks for an already-running cockpit, and then runs `bootstrap-cockpit.sh` for you, passing through `--dry-run`/`--no-start` if you ask for either. The manual invocation above is still there and still useful for scripting or testing — this is just an alternative path for the common case.

## Is it working?

Watch your chat pane. Working looks like:

- `dispatch: picking <PBI-ID> — <reason>` before anything is created
- `dispatch: refusing <PBI-ID> — <what it saw>` when it declines something
- `merge: <slug> merged and cleaned up`

**Silence is ambiguous** and worth understanding: it means either nothing is ready, or all four slots are busy. Both are normal. `herdr pane list --workspace <id>` shows you which — a runtime pane per live item, labelled with its worktree.

## Stop and resume

```
touch <main-root>/.bee/tmp/bee-herding.stop     # dispatch loop exits at the next boundary
rm    <main-root>/.bee/tmp/bee-herding.stop     # it can be started again
```

Nothing needs killing. The dispatch loop checks that file both before and after every iteration, so a stop created mid-iteration takes effect at that boundary, not a full interval later. The path is under the **main checkout** — that is the file the loop and the bootstrap both look at.

**The stop file does NOT stop working agents already running.** Each working agent is its own `claude` session in its own runtime pane and worktree; the stop file is never read by them. Stopping the loop only guarantees no *new* agents are spawned. To stop one already running, close its pane (`herdr pane close <pane_id>`) or open it and talk to the agent — its worktree survives either way (`bee worktree list` shows it). To disable *dispatch* without stopping the loop process, remove the enable marker (`rm <main-root>/.bee/tmp/bee-herding.enable`); it will then poll and do nothing. `ls <main-root>/.bee/tmp/bee-herding.enable` reports whether the marker is currently present — the `bee herding enable|disable|status` verbs are not built into the current binary.

Removing the stop file does not restart the loop: it only lets it be started again. Re-run the bootstrap.

## When something happens

**`merge: <slug> came back MERGE_VERIFY_RED` — needs you.** The merge was abandoned before any commit existed; main is untouched. It is either a real semantic conflict or a flaky test. Investigate, then clear the marker so a later merge gesture will consider that worktree again:

```
rm <main-root>/.bee/tmp/bee-herding.red.<slug>
```

Until you remove it, that worktree is skipped by the merge gesture. **This is on purpose.** Merge stops cold on red and never re-runs verify on its own — a genuine conflict that happened to pass on a second run would slip through the only gate the merge has. (Since merge is a hand-run gesture anyway, "retry" only happens if you run it again — and the marker makes even that a deliberate, marker-clearing act.)

**An anomaly is reported once, not once per cycle.** An unlabelled runtime pane, or one whose agent died mid-item, is reported and then left alone — never silently reclaimed. Its slot stays held until you deal with it. Four of those deadlock the runtime.

**Bootstrap refuses: "a pane labelled `dispatch` already exists".** Either a loop is running (stop it first), or the label is left over from a dead one — a label outlives the process that set it. Clear it:

```
herdr pane close <pane_id>          # or
herdr pane rename <pane_id> --clear
```

Stopping the loop alone does not clear the label.

**A dispatched agent is doing something you don't want.** It is a normal Claude session in its own worktree — open its pane and talk to it, or close the pane. Its worktree survives either way; `bee worktree list` shows it.

## What it will not do

- **It will not start at all until you enable it**, and it will not merge into main without you present. Those two — the enable marker and merge-as-a-gesture — are the real containment, not the lane filter.
- **It will not merge past a red verify**, ever, on its own.
- **It will not exceed four concurrent working agents.**
- **It will not decide an item is finished because its agent went quiet.** Only bee's own record — cells capped with recorded evidence — counts as finished. An agent reports idle the moment it stops typing, whether it is thinking, waiting, or dead.

## What it does NOT reliably do — read this

- **The lane filter is NOT a hard-gate containment.** Do not assume the loop "will not pick up hard-gate work" — that was measured false: in an adversarial review the lane classifier passed **8 of 8** real backlog rows, including one whose story was "delete the entire JS runtime." It matches an English keyword list against a row and judges work by its title; most real rows are not in English. Treat a lane-safe verdict as "no obvious keyword hit," never as "safe." The dispatcher's own reading of each row (and its bias to refuse when unsure) is the real filter — and even that is advisory. What actually contains this system is the enable marker, merge being a gesture, worktree isolation, the slot cap, and the stop file — **not** the lane filter.

## Things worth knowing

- **The working agents run with permission checks bypassed and no allowlist — this is a knowingly accepted risk (owner decision).** That, not the lane filter, is the real limit on what damage is possible: the filter decides *which item* is picked up, not *what commands* the agent may run. **Blast radius:** each agent is confined to its own git worktree and branch until a merge — a git boundary, not a security sandbox: it shares the machine, network, credentials and every ambient tool. What bounds it: the enable marker, merge-as-a-gesture, the stop file, the four-slot cap, and worktree isolation. See SKILL.md "Accepted risk".
- **The two control panes are NOT bypassed — they run an enumerated command surface.** Dispatch and merge each carry an `--allowedTools` allowlist sized to exactly what they do (dispatch creates worktrees; merge aborts/merges/cleans on main), never "read-only" (which would stall them) and never `bypassPermissions`. Note the coupling: the merge pane runs verify over the just-merged tree, so it executes code the working agents wrote. See SKILL.md "Permission posture".
- **Verify runs one at a time**, behind a lock shared by main and every worktree. Four concurrent verifies on a normal laptop produce red results caused by memory pressure, which look exactly like real failures. If you adopt this skill elsewhere, set your own lock path in `commands.test`.
- **The impact ranking is judgement, not a stored field.** Two cycles over the same backlog can choose differently. The reason announced in chat is the only audit trail.
