# Spawn Proof — a real herdr 0.8.0 round trip (start, prompt, read)

This is the authoritative worked example for §8's spawn sequence, run once
for real against the installed `herdr 0.8.0`, with a throwaway feature slug
(`ho2-spawn-rt`), torn down completely afterward. All values below are
observed, not descriptions. This document replaces the pre-0.8.0 recording
that forbade splitting first (D12) — that recording is now the wrong order
and is not amended, only replaced.

Run in workspace `w4` (label `beehive`), a live production workspace with
several other real bee workers running concurrently — this is not an idle
sandbox. The session running this proof owns pane `w4:p1` (tab `w4:t1`,
label `dispatch`); that pane was resolved first via `herdr pane current
--current` and was never split, renamed, closed, or otherwise touched.
Everything below happens in a new tab created for this experiment.

## Step 0 — create the worktree (MAIN checkout only)

`bee worktree new` refuses when the caller's cwd is inside a linked
worktree. This step therefore runs with cwd set to the MAIN checkout, while
the rest of this proof (and the cell that produced it) runs from a feature
worktree:

```
.bee/bin/bee worktree new --feature ho2-spawn-rt --json
```

Result:
- `id`: `beehive--wt--ho2-spawn-rt`
- `worktreeRoot`: `/home/thanhsmind/projects/goglbe/beehive--wt--ho2-spawn-rt`
- `branch`: `wt/ho2-spawn-rt`

## Step 1 — open a dedicated experiment tab

Rather than splitting into the cockpit's own runtime tab (none exists in
this workspace; the cockpit layout here is one-tab), a fresh tab is opened
so nothing created by this proof can be mistaken for a live working agent:

```
herdr tab create --workspace w4 --cwd /home/thanhsmind/projects/goglbe/beehive \
  --label ho2-spawn-rt-experiment --no-focus
```

Result: new tab `w4:tA`, root pane `w4:pA`. This root pane stands in for
"the runtime pane" that §8 splits.

## Step 2 — split a pane into the worktree

Geometry check first, per §8's tie-break rule:

```
herdr pane layout --pane w4:pA
```

`rect`: `width 173, height 50` — wider than tall → `--direction right`.

```
herdr pane split w4:pA --direction right --ratio 0.5 \
  --cwd /home/thanhsmind/projects/goglbe/beehive--wt--ho2-spawn-rt --no-focus
```

Result: new pane `w4:pB`, `tab_id w4:tA`, `cwd` and `foreground_cwd` both
the new worktree path, `agent_status: "unknown"`, no `label`. `--ratio 0.5`
produced a normal, readable split (86 columns each side of a 173-wide
pane) — nothing here suggested a different ratio was needed.

**Settle check.** `pane split` returns as soon as the pane exists, before
its shell is necessarily at an interactive prompt. Reading the pane
immediately after (`herdr pane read w4:pB --source detection --lines 20`)
already showed a plain `$` prompt with no retry needed in this run — on a
slower shell start (the documented Windows/ConPTY risk) this read should be
polled until the prompt appears before issuing `agent start`.

## Step 3 — start the working agent (the previously unproven wiring)

```
herdr agent start ho2-spawn-rt --kind claude --pane w4:pB --timeout 60000 \
  -- --model sonnet --permission-mode bypassPermissions \
  "Print exactly one short sentence and nothing else: the word DONE followed by a period. Do not read any files, do not run any commands, do not do anything else."
```

This is the split-then-start form D12 fixes: the pane is created by
`pane split` first, and `agent start` is handed that pane's id via
`--pane` — the inverse of the pre-0.8.0 form, which passed `--cwd` /
`--workspace` / `--tab` and let `agent start` open its own pane (now a
hard `unknown option: --cwd` on this binary).

**Observed exact argv, echoed back verbatim by `agent start`'s own JSON
result (`result.argv`):**

```json
["claude","--model","sonnet","--permission-mode","bypassPermissions",
 "Print exactly one short sentence and nothing else: the word DONE followed by a period. Do not read any files, do not run any commands, do not do anything else."]
```

`result.agent`: `pane_id: "w4:pB"` (the exact pane passed via `--pane` —
`agent start` did not open a second pane; the step-2 split pane IS the
agent's pane under `--kind`/`--pane`), `tab_id: "w4:tA"`,
`workspace_id: "w4"`, `name: "ho2-spawn-rt"`, `agent_status: "idle"`,
`interactive_ready: true`.

**Observed side effect — `agent start` moved global tab focus, unlike
`pane split`/`tab create`.** `agent start` carries no `--no-focus` flag.
Before this call the workspace's `active_tab_id` was `w4:t1` (this
session's own tab); immediately after, `herdr workspace list` showed
`active_tab_id: "w4:tA"` — the experiment tab. This is why the very first
status read came back `idle` rather than `done`: the tab had already been
"seen" by the UI as a side effect of starting the agent, not because any
explicit focus command was issued.

## Step 4 — prompt the agent and wait

```
herdr agent prompt ho2-spawn-rt "Reply with exactly one word: PONG" --wait --timeout 60000
```

Result: `agent_status: "done"`, `focused: false` on the target pane. This
was observed **without issuing any focus command** in between — the tab
(`w4:tA`) had remained the workspace's active tab from step 3 onward, but
the individual pane's own `focused` field (distinct from tab-level
activeness, and tracked per-pane when a tab holds more than one pane —
`w4:tA` held both the root pane `w4:pA` and the agent pane `w4:pB`) read
`false`. `done` is documented as "the same underlying idle state after
unseen background work finishes" — this second turn's completion was
genuinely unseen at the pane level even though its tab was still active,
confirming the D7 hazard that `idle` vs `done` tracks per-pane UI
attention, not just tab activeness or an explicit reader's intent.

## Step 5 — read the reply back

```
herdr agent read ho2-spawn-rt --source recent-unwrapped --lines 60
```

Observed transcript (trimmed to the relevant lines):

```
❯ Print exactly one short sentence and nothing else: the word DONE followed by a
  period. Do not read any files, do not run any commands, do not do anything else.

● DONE.

✻ Sautéed for 3s

❯ Reply with exactly one word: PONG

● PONG

✻ Crunched for 2s

  /home/thanhsmind/projects/goglbe/beehive--wt--ho2-spawn-rt | wt/ho2-spawn-rt | …
  sonnet-5 103k new/26k cached
  sonnet-5 $0.42 billed
  ⏵⏵ bypass permissions on (shift+tab to cycle) · ← 1 agent
```

Both replies (`DONE.` and `PONG`) are present, in order, matching the two
prompts sent — no evidence of a stale or misattributed turn. The pane's own
status bar reads **`⏵⏵ bypass permissions on (shift+tab to cycle)`** —
confirming `--permission-mode bypassPermissions` took effect — and the cwd
row confirms the pane's branch is `wt/ho2-spawn-rt`, the throwaway
worktree, not the calling session's own checkout.

## Teardown (back to starting state)

```
herdr pane close w4:pB
herdr tab close w4:tA
herdr tab focus w4:t1
git worktree remove --force /home/thanhsmind/projects/goglbe/beehive--wt--ho2-spawn-rt
git branch -d wt/ho2-spawn-rt
.bee/bin/bee worktree unregister --id beehive--wt--ho2-spawn-rt
```

Observed results:
- `herdr pane close w4:pB` → `{"type":"ok"}`.
- `herdr tab close w4:tA` → `{"type":"ok"}` — removes the root pane
  (`w4:pA`) along with the tab.
- `herdr tab focus w4:t1` → restores the workspace's active tab to this
  session's own tab (a courtesy, not a requirement — focus is not one of
  the destructive actions this session's own pane is protected from).
- `herdr pane list --workspace w4` afterward: exactly the same five panes
  present before this proof started (`w4:p1`, `w4:p4`, `w4:p5`, `w4:p6`,
  `w4:p7`) — no `w4:pA`/`w4:pB` remnant, no `w4:tA` remnant.
- `git worktree remove --force …` → exit 0. (Run from the feature
  worktree's own cwd, not the MAIN checkout — `git worktree` metadata is
  shared across all worktrees of one repo, so no `cd`/`-C` to MAIN is
  needed for this particular git subcommand, unlike step 0's
  `bee worktree new`.)
- `git branch -d wt/ho2-spawn-rt` → deleted cleanly (plain `-d`, not `-D` —
  the branch had zero commits beyond the base it forked from).
- `.bee/bin/bee worktree unregister --id beehive--wt--ho2-spawn-rt` →
  `Removed worktree grant for id beehive--wt--ho2-spawn-rt.`
- `git worktree list` afterward: no `ho2-spawn-rt` row.
- `git branch --list 'wt/ho2-spawn-rt'`: empty.
- `.bee/bin/bee worktree list --json` afterward: no key containing
  `ho2-spawn-rt` (other concurrent worktrees' grants in this live repo are
  unrelated and untouched).

## Takeaways for §8

1. **`agent start` no longer opens its own pane, and splitting first no
   longer leaks a pane.** Under `--kind`/`--pane`, `agent start` occupies
   exactly the pane it is handed — the step-2 split pane IS the agent's
   pane. There is no second, independent pane and nothing stray to close on
   the success path. (A start *failure* still leaves the step-2 pane
   behind, unlabelled, at the worktree cwd — §8's own cleanup rule for that
   case is unchanged by this proof.)
2. **`agent start` has no `--no-focus`.** Unlike `pane split` and
   `tab create`, starting an agent moves the workspace's active tab to the
   agent's own tab. A caller that wants to preserve its own tab's focus
   (as this proof did, via `herdr tab focus <own-tab>` in teardown) must
   restore it explicitly afterward; nothing in `agent start` offers to skip
   the focus move.
3. **`idle` vs `done` is a per-pane UI-attention signal, not a per-tab
   one.** The first turn read `idle` because starting the agent itself
   focused its tab; the second turn — sent and completed with no further
   focus command — read `done`, because the individual pane's own
   `focused` field had gone false even though its tab remained the
   workspace's active tab. A caller must not treat "the tab is active" as
   "every pane in it has been seen."
4. **Never pass `-p`/`--print`.** Not re-tested in this proof (the previous
   recording already established that a headless one-shot argv closes its
   own pane the instant the process exits); the working agent must stay a
   plain interactive `claude` for exactly that reason.
5. **Self-naming still needs an explicit instruction.** This proof's
   opening prompt did not include a self-naming step (it was a trivial,
   bounded print-only instruction, deliberately not real repository work),
   so the agent's pane was never renamed and stayed labelled only by the
   `--pane`/`--kind` plumbing (`name: "ho2-spawn-rt"` at the agent level,
   no pane `label`). §8's production argv still needs its own explicit
   self-naming instruction folded in, as the pre-0.8.0 recording already
   established.
