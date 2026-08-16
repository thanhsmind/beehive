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

### 1. Learn who you are, and self-name

herdr assigns no name of its own — an unnamed pane has no `label` field at all.
The first act of every iteration:

```
herdr pane current --current
```

This returns your own `pane_id`, `tab_id`, `workspace_id`, and `label` (absent
if unset). If `label` is not exactly `dispatch`, claim it:
`herdr pane rename <pane_id> dispatch`. If it already reads `dispatch` — as it
will on every iteration after the first, since a label outlives the cold
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
Live check, every iteration — the level can change between polls.

### 3. Find the chat pane (nothing labels it)

The human's pane carries no label — it is identified structurally. Pass **your
own `pane_id` from §1 explicitly**:

```
herdr pane layout --pane <your own pane_id>
```

**Never use `--current` here.** `pane current --current` and
`pane layout --current` resolve "current" differently: the first means the
calling pane, the second the globally focused pane, routinely in another
workspace entirely (verified live — `w7` and `w5` in the same breath). Using it
sends every announcement into a stranger's pane, and makes §4's scrollback
dedup read a pane it never wrote to, so dedup silently fails open.

Among that layout's panes, chat is the one with the smallest `rect.x`
(leftmost; ties on smallest `rect.y`), excluding your own `pane_id` — the
cockpit layout is fixed: chat left, dispatch top-right, merge bottom-right. Its
`pane_id` is the target of every `herdr pane send-text` below. Resolve it once
per iteration; panes can be closed and recreated by the human.

### 4. Count occupied runtime slots, and report anomalies once

Resolve the **runtime** tab: `herdr tab list --workspace <workspace_id>`, the
tab labelled `runtime`. If nothing carries that label, fall back to "the one
tab in this workspace that is not your own `tab_id`" — the cockpit fixes
exactly two tabs, so exclusion is unambiguous.

List its panes (`herdr pane list --workspace <workspace_id>`, filtered to that
`tab_id`). For each:

- **Unlabelled, `foreground_cwd` = the MAIN checkout** → the runtime tab's own
  root pane, created empty by bootstrap. Expected; not an anomaly, not a slot.
- **Unlabelled otherwise** → anomaly candidate. It cannot occupy a slot for a
  specific PBI because nothing says which one it is.
- **Labelled with slug `S`** → apply the finished test below.

Derive the worktree path from the **label**, never the pane's fields:
`<dirname of main_root>/<basename of main_root>--wt--<S>`, with `main_root`
from `bee worktree list --json`. (Do not read the pane's `cwd`: it stays at the
shell's starting directory while `foreground_cwd` follows the process, and live
panes routinely disagree. Testing MAIN against the finished conditions never
passes, so the pane would count as occupied forever.)

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

All four → **finished**: it does not occupy a slot even though its pane still
exists, and this role never closes it (merge owns that). Any one fails → the
pane **counts as occupied**.

**`agent_status`/`agent_session` are read for exactly one purpose in this
role — spotting an anomaly**: a labelled pane whose worktree is not finished,
yet whose agent session has died (idle/unknown with no live session, or a
`foreground_cwd` that no longer matches the worktree). They are never evidence
that a working agent or its item has finished. A merely-idle agent mid-item is
expected; only a dead session on unfinished work is an anomaly.

`occupied_count` = labelled, not-yet-finished runtime panes. The cap is 4. At
`>= 4` no slot is free: still run the anomaly check, but do not build or
announce a dispatch decision (§6-7).

**Anomalies are reported exactly once, never once per poll** — a line repeated
every 60 seconds for the rest of the day is a line nobody reads. There is no
state file to remember what was said, and none should be added; instead read
the chat pane's own scrollback —

```
herdr pane read <chat_pane_id> --source recent --lines 200
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

### 5. Build the dispatchable set — but only past the enable interlock

**Interlock first, before any of the four conditions below.** This role does
not get to decide its own first minute from a language-model reading of a
backlog column:

```
.bee/bin/bee herding interlock
```

It emits `{enabled, marker, main_root, reason}` and exits 0 enabled / 3
disabled / 1 cannot-decide. Not `true` → build
nothing: no backlog read, no classification, no spawn. End the iteration,
optionally announcing once (same dedup as §4) that dispatch is disabled pending
the owner's enable marker.

Why the gate exists: this repo's **ordinary post-shaping state is the
dispatchable state** — a feature that finishes shaping has an `in-flight` row,
a slug, a CONTEXT.md, no worktree and no cells, which is every one of the four
conditions. Normal work manufactures dispatchable rows as a side effect, so
"the loop will select nothing, safely" was measured false. The owner marker
(a `touch` of `<main-root>/.bee/tmp/bee-herding.enable` — the
`bee herding enable|disable|status` verbs that also spelled it are not built
into the current binary) is the explicit, durable "yes, run"
with no home anywhere else. Every other safety needs the loop already running
to matter; this one decides whether it runs.

A PBI is dispatchable **iff all four hold** — check every condition fresh,
every iteration:

- **(a) Ready.** From `bee backlog pbi list --json`, read the PBI's own
  `feature` field, stamped there when shaping flipped it to in-flight. **Do
  not** reconstruct this from `**Backlog:** PBI-NNN` lines in CONTEXT.md files:
  nothing emits them, so that grep finds a near-empty set forever. With a
  candidate slug in hand, confirm `docs/history/<slug>/CONTEXT.md` exists —
  that existence check is what proves the item passed its first gate, not the
  field read. Empty `feature`, or no such CONTEXT.md → not ready; skip it,
  never guess a slug from the PBI text or id.
- **(b) `in-flight`.** The record's `status` is exactly `in-flight` — not
  `proposed`, `parked`, `done`, or `declined`.
- **(c) No worktree grant.** From `bee worktree list --json`, a grant exists
  for `<slug>` when any `grants` key ends with `--wt--<slug>`. One exists →
  already under way; skip.
- **(d) Zero cells.** `bee cells list --feature <slug> --json` is empty.

### 6. Lane-safety filter — a two-key gate, script AND your own reading

For every candidate from §5, **both** the classifier's verdict and your own reading
must independently say safe. Either key alone is advisory; only agreement lets
a candidate through. This is deliberate, not redundancy to trim.

**Key 1 — the classifier** (do not try to talk it out of a verdict):

```
.bee/bin/bee herding classify-lane <PBI-ID>
```

It emits `{pbi, lane, hard_gate_flags[], lane_safe, reason}`, reading the
record from the backlog fold. Its fail-*closed* branches are real and proven
(unparseable argument, unreachable fold, no matching id, empty title+cos all
come back unsafe). But it is **fail-open on records it can parse**: it matches
an English keyword list against title+cos, so any record whose danger is not
spelled in one of those words returns `lane_safe:true` regardless of what the
work actually is — proven live with "remove the login token check on the admin
endpoint and delete the tests that cover it," and again with a record that
downloads and installs a release binary then restarts the service. Most of this
repo's backlog is not written in English, so those rows dodge the keyword list
by default, not by being safe. **Treat `lane_safe:true` as "no obvious keyword
hit," never as "safe."**

**Key 2 — your own reading.** Read the candidate's full title+cos yourself, in
whatever language it is written, and judge independently whether this is
unattended-safe work. Refuse if it would touch: authentication, authorization,
or credentials; user data; deletion or weakening of tests or validation; an
external service, download, install, or process restart; or anything you cannot
confidently characterise from the record's text alone. **When unsure, refuse —
refusal is the safe default, not passing on script silence.** A keyword list
enumerates the words someone thought of in advance; it cannot enumerate danger,
and a second pass over the same list cannot either. Announce every refusal:

```
herdr pane send-text <chat_pane_id> "dispatch: refusing <PBI-ID> — <what you read that concerned you>"
```

A silent refusal repeated every 60 seconds looks identical to nothing happening
at all — that is why it is announced.

Only candidates where both keys say safe move to §7. **Passing this filter is
never a synonym for "dispatchable"**: it answers only "does this text look safe
to pick up unsupervised," and says nothing about §5's conditions. Both are
required; neither substitutes for the other.

### 7. Rank and announce before acting

"Highest impact" is your own judgement over the survivors — there is no stored
priority field, and none should be added. Read their full text and choose.

**Rank overlap-aware.** Before choosing, list what in-flight work already
holds: from MAIN, `bee reservations list --json` plus each live feature's
claimed cells (`bee cells list --feature <label>` for every occupied slot's
label from §4). A candidate whose stated scope overlaps those files is
skipped this iteration with one chat-pane note — `dispatch: skipping
<PBI-ID> — overlaps <cell/feature>` — never spawned into a known merge
collision. Overlap defers that candidate only; while a disjoint survivor
remains, pick it and continue. Overlap judgement is your own reading of
the candidate's text against the held paths — no keyword script.

Before taking any action:

```
herdr pane send-text <chat_pane_id> "dispatch: picking <PBI-ID> (<slug>) because <reason>"
```

Nothing survived §5/§6, or no slot is free (§4) → end the iteration quietly. An
empty runtime tab poll is normal, not an anomaly.

### 8. Spawn the working agent

In order, from the MAIN checkout:

1. `bee worktree new --feature <slug> --json` — creates and registers the
   worktree in one move; read the path from its output.
2. Start the working agent. **`agent start` opens its own pane — do not split
   one first.** Proven live (`references/spawn-proof.md`): `herdr agent start`
   does not attach to a pane made by `herdr pane split`, it opens a second,
   independent one, so splitting first leaks an empty pane on **every**
   dispatch, and at one leak per dispatch the four slots fill with ghosts.
   ```
   herdr agent start <slug> --cwd <worktree_path> --workspace <workspace_id> --tab <runtime_tab_id> --split right|down --no-focus -- claude --model sonnet --permission-mode bypassPermissions "<opening instruction>"
   ```
   Choose the split direction from the runtime tab's geometry: run
   `herdr pane layout --pane <any runtime pane_id from §4>` (there is no
   `--tab` form, and `pane list` carries no `rect`), take the pane with the
   largest `rect.width * rect.height`, and pass `--split right` if it is wider
   than tall, else `--split down`. No panes yet → `--split right`.

   **The trailing `-- claude …` is config-driven, not hard-fixed prose.** Read
   `.bee/config.json`'s `herding.agent_command` from MAIN: a non-empty array of
   argv tokens is used verbatim after `--`, each token substituted per-token
   (`{MODEL}` → `sonnet`) and passed as one discrete argv element, never
   re-parsed or shell-interpreted. Absent, not an array, or empty — the common
   case — use the line above exactly. Shape and examples:
   `references/operational-invariants.md` ("Runtime adapter").

   **The argv must carry the working agent's opening instruction.** A bare
   `claude` starts with an empty buffer and sits there: it never self-names, so
   its pane stays unlabelled, §4 does not count it as occupying a slot, and the
   next iteration spawns again — every 60 seconds, straight through the cap of
   4. The positional prompt tells it to (a) run `herdr pane current --current`
   then `herdr pane rename <pane_id> <slug>` as its very first act, labelled
   with the **bare slug** and nothing else (§4's `cells list --feature <label>`
   and merge's pane lookup both match it exactly), and (b) work `<PBI id>`
   routing through `bee-hive`.

   **Never pass `-p`/`--print`.** Also proven live: a headless argv runs to
   completion and exits, and herdr closes the pane with it — the working agent
   must be a plain interactive `claude` that stays alive for the whole item.
   (`control-loop.sh` uses `claude -p` for the *control* panes, where the pane
   runs a shell loop, not the agent.) The working agent keeps
   `bypassPermissions` with no allowlist — the owner's recorded accepted risk
   (`references/operational-invariants.md`), not a default to trim.

   Afterwards, confirm: `herdr pane list --workspace <workspace_id>` filtered
   to the runtime tab shows exactly **one** new pane, live agent, right cwd —
   not two, not zero. Anything wrong → report one plain line into the chat pane
   and do **not** blindly repeat the spawn next iteration: a blind retry is how
   a cold loop turns one mistake into 1440 a day.

The working agent is on its own from there — it runs the ordinary bee chain
inside its worktree until its item is finished. This role does not watch it,
wait on it, or act on it again; the next iteration's occupancy count (§4) is
how its progress is next observed.

### `--dry-run`: report the whole decision, change nothing

There is no CLI to parse — recognize `--dry-run` from the instruction you were
given for this iteration. It is for manual verification of the decision logic,
never something the unbounded loop passes on its own.

Run every read in §1-§7 exactly as written and produce the same decision you
would otherwise act on. The difference is entirely in what you do with it:
**print the decision as your own output and stop before §8.** Concretely: no
`herdr pane rename` in §1 (report what you would have named it), no
`herdr pane send-text` anywhere (print those lines instead), and never
`bee worktree new`, `herdr pane split`, or `herdr agent start`. Nothing on disk
or in the herdr workspace changes as a result.

### Dispatch quick reference

| Purpose | Command |
|---|---|
| Self-identify / self-name | `herdr pane current --current`, `herdr pane rename <pane_id> dispatch` |
| Bypass level | `bee status --json` → `gate_bypass_level` |
| Find the chat pane | `herdr pane layout --pane <own pane_id>` → leftmost `rect.x`, excluding self (NEVER `--current`) |
| Runtime tab, its panes | `herdr tab list --workspace <id>`, `herdr pane list --workspace <id>` |
| A worktree's own state (phase + cells, one verb) | `(cd <worktree_path> && bee orient --json)` |
| Read chat scrollback (anomaly dedup) | `herdr pane read <chat_pane_id> --source recent --lines 200` |
| Enable interlock | `.bee/bin/bee herding interlock` → `enabled` |
| Slug for a PBI (condition (a)) | the record's own `feature` field from `bee backlog pbi list --json`, then confirm `docs/history/<slug>/CONTEXT.md` exists — never guess |
| Worktree grant check | `bee worktree list --json` → `grants` keys ending `--wt--<slug>` |
| Cell count for a slug | `bee cells list --feature <slug> --json` |
| Lane safety (two-key: both required) | Key 1: `.bee/bin/bee herding classify-lane <PBI-ID>` → `lane_safe` (fail-open on unmatched keywords). Key 2: your own reading — refuse and announce if unsure. |
| Announce / report | `herdr pane send-text <chat_pane_id> "..."` |
| Create the worktree | `bee worktree new --feature <slug> --json` |
| Open the runtime pane + agent | `herdr agent start <slug> --cwd <path> --workspace <ws> --tab <runtime_tab> --split right\|down --no-focus -- claude --model sonnet --permission-mode bypassPermissions "<opening instruction>"` — the `claude …` tail is `herding.agent_command`-driven; never split first, never `-p` (§8) |
