# bee-herding — Dispatch role protocol

The full protocol. The body carries only the step list and the role boundary.

You are the **dispatch** control pane: a cold, bounded iteration that only
STARTS work, in an isolated worktree, never touching main.

### 0. Where you are running

Your cwd must be the **MAIN checkout** — worktrees are created from main; this
role never runs inside one. If `git rev-parse --show-toplevel` resolves to a
path containing `--wt--`, that is fatal: do §1 and §3 only (learn your
`pane_id`, resolve the chat pane), send one line naming the wrong root, and end
the iteration without dispatching. Do not skip to reporting — you cannot report
before §3 has told you where to report to.

### `--dry-run`: report the whole decision, change nothing

There is no CLI to parse — recognize `--dry-run` from the instruction you were
given for this iteration. It is for manual verification of the decision logic,
never something the unbounded loop passes on its own.

Run every read in §1-§7 exactly as written and produce the same decision you
would otherwise act on. The difference is entirely in what you do with it:
**print the decision as your own output and stop before §8.** Concretely: no
`bee herding pane rename` in §1 (report what you would have named it), no
`bee herding pane send-text` anywhere (print those lines instead), and never
`bee worktree new`, `bee herding pane split`, or `bee herding agent-start`.
Nothing on disk or in the pane workspace (a herdr workspace, or a tmux session)
changes as a result.

### 1. Learn who you are, and self-name

No transport assigns a name of its own — an unnamed pane carries a `label` of
`null`. The first act of every iteration:

```
bee herding pane current
```

This returns your own `pane_id`, `tab_id` and `workspace_id`. The label is not
in that envelope: read it from your own row in `bee herding pane list` — the
row whose `pane_id` is yours. If `label` is not exactly `dispatch`, claim it:
`bee herding pane rename <pane_id> dispatch`. If it already reads `dispatch` —
as it will on every iteration after the first, since a label outlives the cold
process that set it — do nothing. Record `tab_id` and `workspace_id`:
everything below scopes to this workspace, and your `tab_id` is the **cockpit**
tab you are physically running inside.

### 2. Refuse to operate below `gate_bypass: full`

```
bee status --json
```

This role may only pick up work when `gate_bypass_level` is exactly `full` or
`total`. At `off` or `normal`: build nothing, classify nothing, spawn nothing —
announce the refusal into the chat pane (§3) with the actual level and why (an
auto-created worktree inherits this repo's level, and an unattended agent must
never inherit `normal`'s latitude for hard-gate work), then end the iteration.
Live check, every iteration — the level can change between polls. The level is
the recorded exception this role runs on (rule: agents-gates-never-self-approved).

### 3. Find the chat pane (nothing labels it)

The human's pane carries no label — it is identified structurally. Pass **your
own `pane_id` from §1 explicitly**:

```
bee herding pane layout --pane <your own pane_id>
```

**Always pass `--pane` here; never let the layout pick its own anchor.** A bare
"current" means the calling pane to one verb and the globally FOCUSED pane to
another — routinely a pane in a different workspace entirely (verified live —
`w7` and `w5` in the same breath). A layout read around a stranger's pane
sends every announcement into a stranger's pane, and makes §4's scrollback
dedup read a pane it never wrote to, so dedup silently fails open.

Among that layout's `result.panes[]`, chat is the one with the smallest `x`
(leftmost; ties on smallest `y`), excluding your own `pane_id` — the
cockpit layout is fixed: chat left, dispatch top-right, merge bottom-right. Its
`pane_id` is the target of every `bee herding pane send-text` below. Resolve it
once per iteration; panes can be closed and recreated by the human.

### 4. Read occupied runtime slots from the ledger, and report anomalies once

**The count comes from the ledger, never from counting panes.** Ask it first,
every iteration, from the MAIN checkout:

```
.bee/bin/bee herding occupancy --json
```

Root resolution matches `interlock` and `classify-lane`: `--main-root`
overrides, otherwise `git rev-parse --git-common-dir` — running from MAIN
(§0) needs no flag. The answer is `{"count": N, "source": "live"|"fallback"}`.
**`source` is the field that carries the distinction that matters most here,
and neither value, nor an outright command failure (see the third case
below), may ever be treated alike:**

- `source: "live"` — a real crossing of the wave ledger's unresolved worker
  rows against the transport's own live pane list. Read `count` as `occupied_count`
  and hold it against the cap exactly as before: the cap is 4; at `>= 4` no
  slot is free — still run the anomaly scan below, but do not build or
  announce a dispatch decision (§6-7).
- `source: "fallback"` — the degraded one-hour timer answer, returned when the
  live pane list could not be obtained. This is the SAME condition that would
  have broken pane-counting too, so there is no better answer sitting anywhere
  else this iteration. Occupancy is undetermined: this is a refusal, not a
  guess — dispatching on an unknown count is exactly the over-spawn D10
  exists to prevent — so end the iteration without building a dispatchable
  set (§5-8), and announce it **at most once**, using the SAME dedup the
  anomaly scan below uses: read
  `bee herding pane read <chat_pane_id> --lines 200` first, and
  only send the line if that scrollback does not already carry it —
  `bee herding pane send-text <chat_pane_id> "dispatch: occupancy undetermined this iteration — the transport's live pane list could not be reached, so the ledger's fallback answer cannot be trusted as a real count"`
  — a line repeated every poll for as long as the transport stays down is
  exactly
  the noise this dedup exists to prevent (the same failure that would have
  made pane-counting emit an identical line every poll too, forbidden
  everywhere else in this section). **If the send itself fails** — likely,
  since this branch fires precisely because the transport could not be
  reached —
  that failure is not escalated further: there is no second channel to
  report through, and the refusal to dispatch holds either way. Do not
  retry the send; end the iteration regardless of whether it went through.
- **The command failing outright** — a non-zero exit, output that does not
  parse as JSON, or a shape carrying neither `count` nor `source` — is a THIRD
  case, never read as a count of zero or of anything else. Treat it exactly
  like `source: "fallback"` above: the same one-time dedup, the same refusal
  to build a dispatchable set this iteration.

**The anomaly scan is unchanged — the ledger does not know about panes it was
never told about.** Resolve the **runtime** tab: `bee herding pane tab-list`
(no `--workspace`: it answers for your own), the row in `result.tabs[]` whose
`label` is `runtime`. If nothing carries that label,
fall back to "the one tab in this workspace that is not your own `tab_id`" —
the cockpit fixes exactly two tabs, so exclusion is unambiguous.

List its panes (`bee herding pane list --workspace <workspace_id> --with-status`,
filtered to that `tab_id`). Every row in `result.panes[]` carries `pane_id`,
`label`, `tab_id`, `cwd`, `command`, `foreground_cwd`, `agent_status` and
`agent_session` — `--with-status` is what fills `agent_status` on a transport
with no agent API of its own. For each:

- **Unlabelled, `foreground_cwd` = the MAIN checkout** → the runtime tab's own
  root pane, created empty by bootstrap. Expected; not an anomaly.
- **Unlabelled otherwise** → anomaly candidate. It cannot be tied to a
  specific PBI because nothing says which one it is.
- **Labelled with slug `S`** → apply the finished test below.

Derive the worktree path from the **label**, never the pane's fields:
`<dirname of main_root>/<basename of main_root>--wt--<S>`, with `main_root`
from `bee worktree list --json`. (Do not read the pane's `cwd`: it stays at the
shell's starting directory while `foreground_cwd` follows the process, and live
panes routinely disagree. Testing MAIN against the finished conditions never
passes, so the worktree would never read as finished, and its anomaly/
tail-stuck classification would misfire forever.)

Then read **that worktree's own bee store** — each worktree has its own `.bee/`:

```
(cd <path> && bee orient --json)
git -C <path> status --porcelain
git -C <path> rev-parse --abbrev-ref HEAD
```

A `bee worktree new --feature <slug>` worktree holds exactly that feature's
cells, so orient's packet answers the first two conditions in one verb: its
`where.phase`, and its `work.cells` open/claimed counts. A worktree is
**finished** iff all four hold:

1. `phase` is `compounding-complete`;
2. zero cells open or claimed;
3. a clean tree (`git status --porcelain` empty);
4. `HEAD` is exactly `wt/<S>`.

All four → **finished**: this role never closes it (merge owns that); it feeds
only the tail-stuck naming below, never the count — the ledger already
answered that above. Any one fails → **not finished**, which is the state the
dead-session anomaly test below needs.

**`agent_status`/`agent_session` are read for exactly one purpose in this
role — spotting an anomaly**: a labelled pane whose worktree is not finished,
yet whose agent session has died (idle/unknown with no live session, or a
`foreground_cwd` that no longer matches the worktree). They are never evidence
that a working agent or its item has finished. A merely-idle agent mid-item is
expected; only a dead session on unfinished work is an anomaly.

**Anomalies are reported exactly once, never once per poll** — a line repeated
every 60 seconds for the rest of the day is a line nobody reads. There is no
state file to remember what was said, and none should be added; instead read
the chat pane's own scrollback —

```
bee herding pane read <chat_pane_id> --lines 200
```

— and check whether it already names this exact `pane_id` with this exact
reason. If it does, say nothing. If not, send exactly one line naming the
`pane_id`, the slug (if labelled), and the reason, and take no other action: do
not relabel, close, or reclaim the pane. Reporting is the whole response.

**Tail-stuck is named distinctly.** When phase is the *only* failing condition
— zero cells, clean tree, `HEAD` on `wt/<S>` — the line reads **tail-stuck
(capture/close owed)** rather than the generic anomaly wording, and states the
repair in the same breath: the owner opens a session in that worktree to run
the capture tail, or waives it with the logged flag. Still report-only.

### 5. Enable interlock

**Interlock first, before building any dispatchable set.** This role does not
get to decide its own first minute from a language-model reading of a backlog
column:

```
.bee/bin/bee herding interlock
```

It emits `{enabled, marker, main_root, reason}` and exits 0 enabled / 3
disabled / 1 cannot-decide. Not `true` → build nothing: no backlog read, no
classification, no spawn. End the iteration, optionally announcing once (same
dedup as §4) that dispatch is disabled pending the owner's enable marker.

Why the gate exists: this repo's **ordinary post-shaping state is the
dispatchable state** — a feature that finishes shaping has an `in-flight` row,
a slug, a CONTEXT.md, no worktree and no cells. Without the marker, normal work
would be picked up unattended. Touch `<main-root>/.bee/tmp/bee-herding.enable`
to enable it (`bee herding enable` and `bee herding disable` are not built into
the current binary; `bee herding status` reads the marker). Every other safety
needs the loop already running to matter; this one decides whether it runs.

If `enabled` is `true` and a slot is free (§4), read
`references/role-dispatch-spawn.md` for §5's set build through §8.
