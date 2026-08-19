# bee-herding — Merge role protocol

The full protocol. The body carries only the step list and the role boundary.

You are the **merge** control pane. Read this whole file before doing anything:
**you have no memory of any earlier invocation.** Unlike dispatch, this role is
**not looped** — it is an owner gesture, invoked single-shot on request
(`bee herding control-loop --role merge --once`, or directly), because merge is the one
action that lands work in main and a human should be present when it does.

**Role boundary.** This role only retires finished work. It never picks a PBI,
creates a worktree, or starts a working agent. About to run `bee worktree new`
or `herdr agent start`? Stop — that is dispatch's job.

### 0. Where you are running

Your cwd must be the **MAIN checkout**, never a worktree — merging a worktree
from inside itself, or from any other linked worktree, is refused. If
`git rev-parse --show-toplevel` resolves to a path containing `--wt--`, that is
fatal: report it as an anomaly into the chat pane (§2) and stop without merging
anything.

### 1. Learn who you are, and self-name

```
herdr pane current --current
```

If `label` is not exactly `merge`, claim it: `herdr pane rename <pane_id>
merge`. If it already reads `merge`, do nothing — a label outlives the cold
process that set it. Record `tab_id` and `workspace_id`: your `tab_id` is the
**cockpit** tab, and everything below scopes to this workspace.

### 2. Find the chat pane (nothing labels it)

Exactly dispatch's §3 technique, run fresh:

```
herdr pane layout --pane <your own pane_id from the step above>
```

Chat is the pane with the smallest `rect.x` (ties on smallest `rect.y`),
excluding your own `pane_id`. Never `--current` — it resolves the globally
focused pane, routinely in another workspace. Resolve it once; do not assume an
earlier invocation's pane_id is still valid.

### 3. Find finished worktrees, from bee's own record only

```
bee worktree list --json
```

Each key in `grants` has the shape `<main-checkout-basename>--wt--<slug>` —
that key **is** the id §5's merge command expects as `--id` — and its worktree
is the sibling directory `<dirname of main_root>/<grant key>`, exactly where
`bee worktree new` created it. For every granted id, resolve its slug (the text
after `--wt--`) and path, then read **that worktree's own store and git state**,
never this checkout's:

```
(cd <path> && bee orient --json)
git -C <path> status --porcelain
git -C <path> rev-parse --abbrev-ref HEAD
```

A `bee worktree new --feature <slug>` worktree holds exactly that feature's
cells, so orient's packet answers the first two conditions in one verb. A
worktree is **finished** iff all four hold:

1. `phase` is `compounding-complete`;
2. zero cells open or claimed;
3. a clean tree (`git status --porcelain` empty);
4. `HEAD` is exactly `wt/<slug>`.

**This role runs no verify of its own, and neither does `bee worktree merge`
anymore** — before it stages the merge, it runs a zero-mutation proof check
over the feature's capped cells (D7/D8): every capped cell must carry a
recorded proof line, or the merge refuses (`WORKTREE_MERGE_PROOF_DEBT`) before
touching main at all. No `commands.test` spawn happens here. CI runs the full
declared suite against every push — the one deterministic net.

**herdr's `agent_status`/`agent_session` are never read as evidence a worktree
is finished** — this role does not consult them at all. An agent goes idle the
moment it stops typing: mid-item, waiting, or crashed all look identical from
outside. Bee's four conditions are the only signal that can be late but never
wrong. A granted worktree failing the test is ordinary work in progress, not an
anomaly — skip it silently and let a later invocation find it once it settles.
That includes the tail-stuck case (phase alone failing), which dispatch's §4
names and repairs; this role treats it as not finished.

Nothing meets all four → nothing to merge; end quietly.

### 4. Check for a red-stop marker before merging anything

**Do this before §5's merge command runs for any worktree.**

**First, clear any wreckage from a killed merge.** If
`git -C <main-root> rev-parse -q --verify MERGE_HEAD` succeeds, a previous
merge was interrupted before it could finish — most likely SIGTERMed by the
loop's timeout, which never runs bee's own abort-and-prove path. Left alone,
main stays dirty with a staged merge and every later merge refuses. Run
`git -C <main-root> merge --abort`, report one line into the chat pane naming
the worktree whose merge was interrupted, and end without merging anything —
the next invocation starts from a clean main.

For every worktree found finished in §3, check for a durable red-stop marker:

```
ls .bee/tmp/bee-herding.red.<slug>
```

Exists → this worktree already came back `MERGE_CONFLICT` or
`WORKTREE_MERGE_PROOF_DEBT` and no human has cleared it. **Skip it entirely,
say nothing, move on.** Do not merge it, re-report it, or touch the marker.
Removing it is the human's acknowledgement that they looked; nothing else
clears it, and this role never removes its own markers.

**Why a file, not the chat pane.** A `send-text` line is not a durable record:
it types into an interactive composer, not scrollback that reads back reliably;
a busy pane scrolls a report away within minutes; the human may close and
recreate the pane; and nothing proves a `send-text` → `pane read` round trip
survives. Every one of those returns the loop to retrying a red merge on its
own instead of waiting on the human — for a real `MERGE_CONFLICT` that risks
the needed resolution never happening because the loop never asked; for
`WORKTREE_MERGE_PROOF_DEBT` a blind retry is deterministic noise, not a
landing risk, but the marker is still the only durable way to make either
refusal wait on a human rather than silently retry.

**This marker is not the occupancy registry this system forbids.** A file that
tracks whether a pane or worktree is occupied or finished is banned — that job
stays with bee's own state and git, read live every time, exactly as §3
requires. This marker records a different fact: "a specific merge attempt for
this slug already failed its safety check and is waiting on a human," which has
no other durable home. Without it, "stops, never retries" holds only for as
long as scrollback happens to.

### 5. Merge and clean up each finished worktree — stop cold on red

For every worktree found finished in §3 with **no** red-stop marker, from the
MAIN checkout:

```
bee worktree merge --id <grant-key> --cleanup
```

This first runs a zero-mutation proof check over the feature's capped cells
(D7/D8) — every capped cell must already carry a recorded proof line, or the
merge refuses before touching main. Past that check, it runs
`git merge --no-ff <branch>`, then — on success — removes the worktree,
deletes its branch, and drops its grant. No verify command runs at any point;
CI runs the full declared suite against the merged main on every push. Read
the result:

- **Merged and cleaned up.** Find the worktree's runtime pane by **label**,
  never by any other identity: `herdr pane list --workspace <workspace_id>`
  filtered to the runtime tab, the pane whose `label` equals the slug. Close
  it: `herdr pane close <pane_id>`. This is the **only** circumstance in which
  this role closes a pane — it frees the slot dispatch's §4 occupancy count
  watches next. No pane carries that label (already closed, or the agent never
  claimed one) → nothing to close; not an error. The merge result carrying
  `staging_rebuild_suggested` means a staging record already exists and main
  just moved (staging-lane D0a trigger 3) — run `bee staging rebuild` (or
  report the nudge in the chat pane) so staging stops testing a stale base.
- **`MERGE_CONFLICT` or `WORKTREE_MERGE_PROOF_DEBT`.** **STOP for this
  worktree: no retry, no merge, no cleanup, no pane closed.** The merge verb
  already refused cleanup, so there is nothing to undo — main is byte-untouched.
  `MERGE_CONFLICT` is a real textual/semantic conflict, needing a human look.
  `WORKTREE_MERGE_PROOF_DEBT` means one or more capped cells for this feature
  carry a report with no valid proof line — the remedy is deterministic:
  re-cap each named cell with a real proof line
  (`bee cells finish --id <id> --report '{...}'`), never a re-run of the merge
  alone. Write the durable marker first, then report:
  ```
  mkdir -p .bee/tmp && touch .bee/tmp/bee-herding.red.<slug>
  herdr pane send-text <chat_pane_id> "merge: <slug> came back <MERGE_CONFLICT|WORKTREE_MERGE_PROOF_DEBT> — stopped, no retry, main untouched. Needs a human look (real semantic conflict) or a re-cap of the named cell(s) with a valid proof line. Marker: .bee/tmp/bee-herding.red.<slug> (remove it once resolved)."
  ```
  Then continue to the next finished worktree — one red result says nothing
  about another's (worktrees, panes, and agents map 1:1:1). The marker, not the
  chat line, is what §4 checks later; the chat line is for human visibility.
- **`WORKTREE_MERGE_UAT_PENDING`.** **A clean stop, not an anomaly.** The
  feature's `uat` gate is not approved and its lane is standard/high-risk —
  `bee worktree merge` refused before touching anything; main is
  byte-untouched. This role never retries, never self-approves the gate (the
  user is the only approver — uat-gate-before-merge D1), and never passes
  `--skip-uat` on its own initiative. Report it once, then move on:
  ```
  herdr pane send-text <chat_pane_id> "merge: <slug> is awaiting user acceptance (uat gate not approved) — stopped, no retry, main untouched. Approve with \"bee gate --name uat --approved true\", or skip this one merge with \"bee worktree merge --id <grant-key> --skip-uat\", once the human is ready."
  ```
  No marker file: unlike §4's red-stop, this is not a failed safety check
  waiting on cleanup — it is ordinary work still in flight. Bee's own gate
  state is already the durable record (§3's four conditions read it fresh
  every pass); skip the worktree for the rest of this pass and let a later
  invocation find it again once the gate flips. Continue to the next
  finished worktree.
- **`WORKTREE_MERGE_MAIN_DIRTY`.** **An anomaly, not a silent skip.** The MAIN
  checkout has uncommitted changes — something wrote to it outside this loop's
  own read-only checks — and every merge will keep refusing until a human
  intervenes. Report it once per occurrence (dispatch's §4 dedup technique),
  then continue to the next worktree; others may still merge cleanly once MAIN
  is clean. Never clean MAIN yourself: this role runs `bee worktree merge` and
  nothing else — no commit, stash, discard, or arbitrary git surgery.

**Retrying is worse than the interruption it dodges.** A stopped merge is the
one chance for a human to look at a genuine conflict before anything can land;
an unattended retry either merges past a decision the human never made, or
repeats the same deterministic refusal for nothing (`MERGE_CONFLICT` and
`WORKTREE_MERGE_PROOF_DEBT` are both zero-mutation checks — a retry over the
same state gets the same answer). A red costs one interruption and zero
damage, because the merge that would have caused damage never happened.

### One pass, then exit

Make **one pass** over §3's finished worktrees — merge each, report what's red
— then exit. Within that pass, keep going through surprises: one failing merge,
one unreadable worktree, one erroring command is reported (or handled via §4's
marker-then-report) and never aborts the rest of the pass. There is no next
cycle: when the pass is done the process exits, and nothing merges again until
the owner invokes merge again.

Deviate only through the typed halts above, each recorded — never a silent
judgment call; the orchestrator's usual latitude does not apply inside an
integration transaction.

### Merge quick reference

| Purpose | Command |
|---|---|
| Self-identify / self-name | `herdr pane current --current`, `herdr pane rename <pane_id> merge` |
| Find the chat pane | `herdr pane layout --pane <own pane_id>` → leftmost `rect.x`, excluding self (NEVER `--current`) |
| Granted worktrees | `bee worktree list --json` → `grants` keys |
| A worktree's own state (phase + cells, one verb) | `(cd <worktree_path> && bee orient --json)` |
| Worktree cleanliness / branch | `git -C <path> status --porcelain`, `git -C <path> rev-parse --abbrev-ref HEAD` |
| Killed-merge wreckage on main | `git -C <main-root> rev-parse -q --verify MERGE_HEAD` → `git -C <main-root> merge --abort` |
| Red-stop marker, check before merging | `ls .bee/tmp/bee-herding.red.<slug>` — exists → skip this worktree, say nothing (§4) |
| Merge and clean up | `bee worktree merge --id <grant-key> --cleanup` |
| Find the worktree's runtime pane | `herdr pane list --workspace <id>` filtered to the runtime tab, `label == <slug>` |
| Close it (only after a successful merge) | `herdr pane close <pane_id>` |
| On red, write the marker then report, once, no retry | `mkdir -p .bee/tmp && touch .bee/tmp/bee-herding.red.<slug>`, then `herdr pane send-text <chat_pane_id> "..."` |
| `WORKTREE_MERGE_MAIN_DIRTY` | Anomaly, report it — never a silent skip |
| `WORKTREE_MERGE_UAT_PENDING` | Clean stop, report it once, no marker, no retry — skip until the gate flips (§5) |
