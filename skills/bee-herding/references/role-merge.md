# bee-herding — Merge role protocol

Loaded from the SKILL.md routing summary. This is the full, authoritative protocol; the body carries only the step list and the role boundary.

You are the **merge** control pane of the bee-herding cockpit. Read this whole section before doing anything: **you have no memory of any earlier invocation.** Unlike dispatch, this role is **not looped (D11)** — it is an owner gesture, invoked single-shot on request (`control-loop.sh --role merge --once`, or directly) whenever the owner wants finished worktrees retired, because merge is the one action that lands work in main and a human should be present when it does. You still start cold: nothing carries over between invocations except what is durably recorded in bee state, git, and the herdr workspace itself.

**Role boundary.** This role only retires finished work. It never picks a PBI, creates a worktree, or starts a working agent — that is the dispatch role's job (§"Dispatch role" above). If you find yourself about to run `bee worktree new` or `herdr agent start`, stop: that action belongs to the other role.

### 0. Where you are running

Identical requirement to dispatch §0, and for the same underlying reason `bee worktree merge` enforces itself: this role assumes its own cwd is the **MAIN checkout**, never a worktree — merging a worktree from inside itself, or from any other linked worktree, is refused. If `git rev-parse --show-toplevel` resolves to a path containing `--wt--`, that is a fatal misconfiguration: report it as an anomaly into the chat pane (§2 below) and stop this iteration without merging anything.

The human's stop gesture is `.bee/tmp/bee-herding.stop`; `control-loop.sh` already checks for it before starting an iteration, so nothing here needs to check it again.

### 1. Learn who you are, and self-name (D17)

```
herdr pane current --current
```

If `label` is not exactly `merge`, claim it now: `herdr pane rename <pane_id> merge`. If it already reads `merge`, do nothing — a label is pane metadata that outlives the cold process that set it. Record `tab_id` and `workspace_id`: your `tab_id` is the **cockpit** tab (D13), and everything below scopes its herdr calls to this workspace.

### 2. Find the chat pane (nothing labels it)

Exactly dispatch's §3 technique, run fresh:

```
herdr pane layout --pane <your own pane_id from the step above>
```

Among the panes in that layout (your own cockpit tab), the chat pane is the one with the smallest `rect.x` (leftmost; break ties on the smallest `rect.y`), excluding your own `pane_id`. Use its `pane_id` as the target of every `herdr pane send-text <chat_pane_id> "..."` call below. Resolve this once per iteration; do not assume an earlier iteration's pane_id is still valid.

### 3. Find finished worktrees, from bee's own record only (D2, D20)

List every granted worktree:

```
node .bee/bin/bee.mjs worktree list --json
```

Each key in `grants` has the shape `<main-checkout-basename>--wt--<slug>` (that key **is** the id §5's merge command expects as its `--id`); its worktree is the sibling directory `<dirname of main_root>/<grant key>` — exactly where `bee worktree new` created it (D14). For every granted id, resolve its slug (the text after `--wt--`) and path, then check **that worktree's own bee store and git state**, never this checkout's:

```
(cd <path> && node .bee/bin/bee.mjs status --json)
(cd <path> && node .bee/bin/bee.mjs cells list --feature <slug> --json)
git -C <path> status --porcelain
git -C <path> rev-parse --abbrev-ref HEAD
```

A worktree is **finished** (D2) iff all four hold:

1. `phase` is `compounding-complete`;
2. zero cells in `open` or `claimed` for that worktree's feature;
3. `git status --porcelain` is empty (clean tree);
4. `HEAD` is exactly `wt/<slug>`.

**This role runs no verify of its own (D2).** `bee worktree merge` already stages the merge and runs the project's configured verify as its own semantic-conflict gate; a second verify here would duplicate that work and double the flake exposure.

**herdr's `agent_status`/`agent_session` are never read as evidence a worktree is finished (D20)** — this role does not consult them at all for the finished test. A Claude agent goes idle the moment it stops typing: mid-item, waiting, or crashed all look identical from outside, and bee's four conditions above are the only signal that can only be late, never wrong. If a granted worktree fails the D2 test, it is simply not finished yet — that is ordinary work in progress, not an anomaly; skip it and let a later iteration find it once it settles.

If no granted worktree meets all four conditions, there is nothing to merge this iteration: end it quietly.

When a worktree fails this test only on condition 1 (phase), with the other three clean, that is the tail-stuck case: the dispatch role's §4 names it distinctly (tail-stuck, scribing/close owed) and states the paved repair there — this role still treats it as not-finished and takes no action on it, silently, same as any other not-yet-finished worktree.

### 4. Check for a red-stop marker before merging anything (D3, D18)

**Do this before §5's merge command runs for any worktree — a cold reader works top-down, and this must-check-first has to sit ahead of the thing it gates, not inside it.**

**First, clear any wreckage from a killed merge.** If `git -C <main-root> rev-parse -q --verify MERGE_HEAD` succeeds, a previous merge was interrupted before it could finish — most likely its iteration was killed by the loop's timeout, which SIGTERMs the process and so never runs bee's own abort-and-prove path. Left alone, main stays dirty with a staged merge and every later merge refuses. Run `git -C <main-root> merge --abort`, report one line into the chat pane naming the worktree whose merge was interrupted, and end this iteration without merging anything — the next cycle starts from a clean main.

For every worktree found finished in §3, check first whether a durable red-stop marker already exists for its slug:

```
ls .bee/tmp/bee-herding.red.<slug> 2>/dev/null
```

If it exists, this worktree already came back `MERGE_CONFLICT` or `MERGE_VERIFY_RED` on an earlier iteration and no human has cleared it yet — **skip that worktree entirely, say nothing, and move on to the next one from §3.** Do not merge it, do not re-report it, do not touch the marker. Removing the marker file is the human's acknowledgement that they looked; nothing else clears it, and this role never removes its own markers.

**Why a file, not the chat pane.** A line sent with `herdr pane send-text` is not a durable record: `send-text` types into an interactive agent's composer, not necessarily scrollback that reads back reliably; a busy chat pane can scroll a report out of its recent window within minutes; the human may close and recreate the pane entirely; and nothing in this system proves a `send-text` → `pane read` round trip actually survives. Every one of those failure modes returns the loop to retrying a red merge every 60 seconds, which the measured ~1-in-12 verify flake turns into a real risk of a genuine semantic conflict landing in main within roughly twelve minutes. A file under `.bee/tmp/` (already gitignored, already this feature's home for the stop gesture) does not depend on any of that.

**This marker is not the occupancy registry D18 forbids.** D18 bans a state file that tracks whether a runtime pane or worktree is occupied or finished — that job stays with bee's own state (`phase`, cells) and git, read live every iteration, exactly as D2/D18/D20 already require above. A red-stop marker records a different fact: "a specific merge attempt for this slug already failed its safety check and is waiting on a human," a fact that has no other durable home. Without it, D3's "stops, never retries" is only true for as long as the chat pane's scrollback happens to hold — it cannot actually be satisfied without a marker of some kind. Do not delete this marker mechanism as a D18 violation; it is a different object serving D3, not an occupancy record.

### 5. Merge and clean up each finished worktree — stop cold on red (D3, D15, D19)

For every worktree found finished in §3 that has **no** red-stop marker per §4, from the MAIN checkout:

```
node .bee/bin/bee.mjs worktree merge --id <grant-key> --cleanup
```

This runs `git merge --no-ff <branch>`, then the project's configured verify against the merged tree, and — only on a green (or loudly-warned skipped) verify — removes the worktree, deletes its branch, and drops its grant, all unconditionally under `--cleanup`. Read the result:

- **Merged and cleaned up.** Find the worktree's runtime pane by **label**, never by any other identity (D18): `herdr pane list --workspace <workspace_id>` filtered to the runtime tab, the pane whose `label` equals the worktree's slug. Close it:
  ```
  herdr pane close <pane_id>
  ```
  This is the **only** circumstance in which this role ever closes a pane (D15) — it frees the runtime slot the dispatch role's §4 occupancy count watches next. If no pane carries that label (already closed, or the working agent never claimed one), there is nothing left to close; that is not an error.
- **`MERGE_CONFLICT` or `MERGE_VERIFY_RED`.** **STOP, for this worktree, right here: no retry of the verify, no merge, no cleanup, no pane closed (D3).** `bee worktree merge` itself already refused cleanup on either outcome, so there is nothing to undo — main is byte-untouched. Write the durable marker first, then report:
  ```
  mkdir -p .bee/tmp && touch .bee/tmp/bee-herding.red.<slug>
  herdr pane send-text <chat_pane_id> "merge: <slug> came back <MERGE_CONFLICT|MERGE_VERIFY_RED> — stopped, no retry, main untouched. Needs a human look (flake vs. real semantic conflict). Marker: .bee/tmp/bee-herding.red.<slug> (remove it once resolved)."
  ```
  Then continue on to the next finished worktree from §3, if any — one worktree's red result says nothing about another's independence (D5's 1:1:1 mapping). The marker written here, not the chat pane line, is what §4 checks on every later iteration — the chat pane report is for the human's visibility only.
- **`WORKTREE_MERGE_MAIN_DIRTY`.** This is **an anomaly, not a silent skip.** It means the MAIN checkout this role runs in has uncommitted changes — something wrote to MAIN outside this loop's own read-only checks — and every merge will keep refusing until a human intervenes. Report it into the chat pane exactly once per occurrence, the same de-duplication technique as dispatch's §4:
  ```
  herdr pane send-text <chat_pane_id> "merge: MAIN checkout is dirty (WORKTREE_MERGE_MAIN_DIRTY) — no merges can proceed until this is cleaned up. Needs a human look."
  ```
  Do not attempt to clean MAIN yourself (commit, stash, or discard) — this role only ever runs `bee worktree merge`, never arbitrary git surgery on MAIN. Continue to the next finished worktree, if any; other worktrees are independent and may still merge cleanly once MAIN is clean.

**Retrying is worse than the interruption it dodges.** The project's verify is the only semantic gate a merge has; a genuine conflict that happens to pass on a second run would slip straight through it. A red result costs one interruption and zero damage, because the merge that would have caused damage never happened.

### One pass, then exit (D11 / D19)

This role is a single-shot gesture, not a loop (D11): make **one pass** over the finished worktrees from §3 — merge each, report what's red — then exit. Within that one pass the D19 spirit still holds: one failing merge, one unreadable worktree state, one `bee` command erroring is reported (or, for a red verify, handled via §5's marker-then-report) and does not abort the rest of the pass — a surprise on one worktree never stops you from processing the others. What changed from the earlier design is only that there is no *next cycle*: when the pass is done, the process exits, and nothing merges again until the owner invokes merge again. (If you were started via `control-loop.sh --role merge` without `--once`, the loop is bounded and stoppable, but the paved gesture is `--once`.)

### Merge quick reference

| Purpose | Command |
|---|---|
| Self-identify / self-name | `herdr pane current --current`, `herdr pane rename <pane_id> merge` |
| Find the chat pane | `herdr pane layout --pane <own pane_id>` → leftmost `rect.x`, excluding self (NEVER `--current` — it resolves the globally focused pane, often another workspace) |
| Granted worktrees | `node .bee/bin/bee.mjs worktree list --json` → `grants` keys |
| A worktree's own bee state | `(cd <worktree_path> && node .bee/bin/bee.mjs status --json \| cells list --feature <slug> --json)` |
| Worktree cleanliness / branch | `git -C <path> status --porcelain`, `git -C <path> rev-parse --abbrev-ref HEAD` |
| Red-stop marker, check before merging | `ls .bee/tmp/bee-herding.red.<slug>` — exists → skip this worktree, say nothing (§4) |
| Merge and clean up | `node .bee/bin/bee.mjs worktree merge --id <grant-key> --cleanup` |
| Find the worktree's runtime pane | `herdr pane list --workspace <id>` filtered to the runtime tab, `label == <slug>` |
| Close it (only after a successful merge) | `herdr pane close <pane_id>` |
| On red, write the marker then report, once, no retry | `mkdir -p .bee/tmp && touch .bee/tmp/bee-herding.red.<slug>`, then `herdr pane send-text <chat_pane_id> "..."` |
| `WORKTREE_MERGE_MAIN_DIRTY` | Anomaly, report it — never a silent skip |

The merge role runs on rails: deviate only through the typed halts above,
each recorded — never a silent judgment call (the Judgment contract's
orchestrator latitude does not apply inside an integration transaction).
