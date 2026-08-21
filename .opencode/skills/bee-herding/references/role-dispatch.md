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
  rows against herdr's own live pane list. Read `count` as `occupied_count`
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
  `herdr pane read <chat_pane_id> --source recent --lines 200` first, and
  only send the line if that scrollback does not already carry it —
  `herdr pane send-text <chat_pane_id> "dispatch: occupancy undetermined this iteration — herdr's live pane list could not be reached, so the ledger's fallback answer cannot be trusted as a real count"`
  — a line repeated every poll for as long as herdr stays down is exactly
  the noise this dedup exists to prevent (the same failure that would have
  made pane-counting emit an identical line every poll too, forbidden
  everywhere else in this section). **If the send itself fails** — likely,
  since this branch fires precisely because herdr could not be reached —
  that failure is not escalated further: there is no second channel to
  report through, and the refusal to dispatch holds either way. Do not
  retry the send; end the iteration regardless of whether it went through.
- **The command failing outright** — a non-zero exit, output that does not
  parse as JSON, or a shape carrying neither `count` nor `source` (the error
  envelope a stale or pre-D18 `bee` binary returns, since it predates this
  verb) — is a THIRD case, never read as a count of zero or of anything
  else. Treat it exactly like `source: "fallback"` above: the same one-time
  dedup, the same refusal to build a dispatchable set this iteration.

(The plain, non-`--json` form prints `occupancy: {count} worker(s) live
({source})` — the parenthetical word carries the identical live/fallback
distinction for a caller reading text rather than JSON. This role reads the
`--json` form's `source` field: matching one field beats matching a word
inside a sentence.)

**The anomaly scan is unchanged — the ledger does not know about panes it was
never told about.** Resolve the **runtime** tab: `herdr tab list --workspace
<workspace_id>`, the tab labelled `runtime`. If nothing carries that label,
fall back to "the one tab in this workspace that is not your own `tab_id`" —
the cockpit fixes exactly two tabs, so exclusion is unambiguous.

List its panes (`herdr pane list --workspace <workspace_id>`, filtered to that
`tab_id`). For each:

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
2. Split a runtime pane for the working agent. On herdr 0.8.0, `agent start`
   requires `--pane` and "never creates, splits, or moves layout" — splitting
   first is now MANDATORY, not forbidden. `references/spawn-proof.md` is the
   re-recorded live evidence for this split-then-start order: a real
   `herdr agent start … --kind … --pane …` round trip against the installed
   0.8.0 binary.
   ```
   herdr pane split <runtime-pane-id> --direction right|down --ratio <r> --cwd <worktree_path> --no-focus
   ```
   Choose the runtime pane to split, and the direction, from the runtime
   tab's geometry: run `herdr pane layout --pane <any runtime pane_id from
   §4>` (there is no `--tab` form, and `pane list` carries no `rect`), take
   the pane with the largest `rect.width * rect.height`, and pass
   `--direction right` if it is wider than tall, else `--direction down`.
   There is always a runtime root pane to split — `bootstrap-cockpit.sh`
   creates it — so there is no "no panes yet" case. The geometry rule above
   computes only the split *direction* (right vs down); the ratio to pass is
   `--ratio 0.5`, unconditionally — confirmed live in
   `references/spawn-proof.md`, where 0.5 produced a normal, readable split.
   Read the new pane's id from the response's `.result.pane.pane_id`.

   **Settle before starting the agent.** `agent start` requires its target
   pane to already be at its interactive shell prompt; `pane split` returns
   as soon as the pane exists, before its shell has necessarily finished
   settling — the next command's `--timeout` covers agent *detection*, not
   shell *readiness*. Confirm (poll, or wait briefly and retry) that the new
   pane is at its shell prompt before issuing `agent start`. This is the
   failure most likely to fire on Windows, where ConPTY starts slower.
3. Start the working agent into that pane.
   ```
   herdr agent start <slug> --kind <kind> --pane <new_pane_id> --timeout 60000 -- <agent args>
   ```
   `<kind>` and `<agent args>` come from `herding.agent_command`, per the next
   paragraph. **On any `agent start` failure, close the pane step 2 of this
   section just created (`herdr pane close <new_pane_id>`) before reporting**
   — an unlabelled pane
   whose `foreground_cwd` is the worktree is exactly what §4 classifies as an
   anomaly candidate, and this section created that pane, so this section is
   the one that cleans it up.

   **The trailing agent argv is config-driven, not hard-fixed prose.** Read
   `.bee/config.json`'s `herding.agent_command` from MAIN: a non-empty array
   of argv tokens has its first token supply `--kind` (the herdr-recognized
   agent kind, e.g. `claude`) and its remaining tokens, substituted per-token
   (`{MODEL}` → `sonnet`) and each passed as one discrete argv element, go
   after `--` as the agent's own arguments — never re-parsed or
   shell-interpreted, and never re-used as `--kind`. An unrecognised token 0
   (not one of herdr's supported kinds) is a typed error naming the
   `herding.agent_command` key, never a generic `agent start` failure. Absent,
   not an array, or empty — the common case — use `--kind claude -- --model
   sonnet --permission-mode bypassPermissions "<opening instruction>"`. Shape
   and examples: `references/operational-invariants.md` ("Runtime adapter").

   **The argv must carry the working agent's opening instruction.** A bare
   `claude` starts with an empty buffer and sits there: it never self-names, so
   its pane stays unlabelled. §4 no longer counts panes at all — the ledger
   row step 4 below records already carries its pane id regardless of any
   label — but the anomaly scan (§4) and merge's own pane lookup both still
   key off the label, so an unlabelled pane still cannot be tied to a
   specific PBI there, and merge cannot find it. The positional prompt tells
   it to (a) run `herdr pane current --current`
   then `herdr pane rename <pane_id> <slug>` as its very first act, labelled
   with the **bare slug** and nothing else (§4's `cells list --feature <label>`
   and merge's pane lookup both match it exactly), and (b) work `<PBI id>`
   routing through `bee-hive`.

   **Never pass `-p`/`--print`.** Also proven live: a headless argv runs to
   completion and exits, and herdr closes the pane with it — the working agent
   must be a plain interactive `claude` that stays alive for the whole item.
   (`bee herding control-loop` uses `claude -p` for the *control* panes, where the pane
   runs the control loop, not the agent.) The working agent keeps
   `bypassPermissions` with no allowlist — the owner's recorded accepted risk
   (`references/operational-invariants.md`), not a default to trim.

   Afterwards, confirm: `herdr pane list --workspace <workspace_id>` filtered
   to the runtime tab shows exactly **one** new pane — the one step 2 of this
   section split — live agent, right cwd, not two, not zero. Anything wrong →
   apply the pane-close rule above if `agent start` itself failed, report one
   plain line into the chat pane, and do **not** blindly repeat the spawn next
   iteration: a blind retry is how a cold loop turns one mistake into 1440 a
   day.
4. **Record the spawn in the wave ledger — only after that confirm step
   passed.** This is what closes the loop (herding-orchestration D18): **a
   spawn that is not recorded here is invisible to the next iteration's
   occupancy read (§4)** — §4 no longer counts panes, it reads the wave
   ledger, and a spawn with no row in it simply is not in what it reads. From
   the MAIN checkout:
   ```
   .bee/bin/bee herding record-worker --name <slug> --pane-id <new_pane_id> \
     --path <worktree_path> --task <PBI-ID>
   ```
   Root resolution matches every other verb in this section: `--main-root`
   overrides, otherwise `git rev-parse --git-common-dir`. **On any failure of
   this call** — non-zero exit, or any error — do not treat it as a minor
   bookkeeping miss: an unrecorded spawn is WORSE than no spawn at all, since
   it is a live agent the next iteration's occupancy read cannot see, which
   lets the four-slot cap be walked past silently. Report it loudly —
   `herdr pane send-text <chat_pane_id> "dispatch: recording <slug> (pane <new_pane_id>) in the wave ledger FAILED — occupancy will undercount until this is repaired"`
   — and still end the iteration without spawning again this poll: do not
   retry the recording call, and do not spawn a second worker to compensate.

5. **Give the human their view back.** `agent start` carries no `--no-focus`
   flag, unlike `pane split` and `tab create`, and it MOVES the workspace's
   focus onto the new agent's tab (recorded live in
   `references/spawn-proof.md`). Left alone, a loop polling on a fixed interval
   yanks the owner away from whatever they were reading, every single spawn —
   the one thing `--no-focus` exists everywhere else to prevent. Read your own
   tab from the pane id you already hold (§1), then focus it back:
   ```
   herdr pane current --pane <your own pane_id>     # read .result.pane.tab_id
   herdr tab focus <that tab_id>
   ```
   **Never `--current` here**, for the reason §3 already gives: it resolves to
   the globally focused pane, which after `agent start` is the WORKER's — so
   `--current` would read the worker's tab and focus the thing you are trying
   to move away from. A failure here is cosmetic, not structural: report it in
   one line and end the iteration normally, never retry the spawn over it.

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
| Occupied slot count (ledger, never pane-counting) | `.bee/bin/bee herding occupancy --json` → `count`/`source` (`live` = real crossing, use against the cap; `fallback` = undetermined, refuse to dispatch this iteration) |
| Runtime tab, its panes (anomaly scan only) | `herdr tab list --workspace <id>`, `herdr pane list --workspace <id>` |
| A worktree's own state (phase + cells, one verb) | `(cd <worktree_path> && bee orient --json)` |
| Read chat scrollback (anomaly dedup) | `herdr pane read <chat_pane_id> --source recent --lines 200` |
| Enable interlock | `.bee/bin/bee herding interlock` → `enabled` |
| Slug for a PBI (condition (a)) | the record's own `feature` field from `bee backlog pbi list --json`, then confirm `docs/history/<slug>/CONTEXT.md` exists — never guess |
| Worktree grant check | `bee worktree list --json` → `grants` keys ending `--wt--<slug>` |
| Cell count for a slug | `bee cells list --feature <slug> --json` |
| Lane safety (two-key: both required) | Key 1: `.bee/bin/bee herding classify-lane <PBI-ID>` → `lane_safe` (fail-open on unmatched keywords). Key 2: your own reading — refuse and announce if unsure. |
| Announce / report | `herdr pane send-text <chat_pane_id> "..."` |
| Create the worktree | `bee worktree new --feature <slug> --json` |
| Split the runtime pane | `herdr pane split <runtime-pane-id> --direction right\|down --ratio <r> --cwd <path> --no-focus` → read `.result.pane.pane_id` (§8) |
| Start the working agent | `herdr agent start <slug> --kind <kind> --pane <new_pane_id> --timeout 60000 -- <agent args>` — `<kind>` and `<agent args>` are `herding.agent_command`-driven; pane must exist first (split, then start), never `-p` (§8) |
| Record the spawn (closes the occupancy loop) | `.bee/bin/bee herding record-worker --name <slug> --pane-id <new_pane_id> --path <worktree_path> --task <PBI-ID>` — only after the confirm step; failure is reported loudly, never silently passed over (§8) |
| Give the human their view back after a spawn | `herdr pane current --pane <your own pane_id>` for its `tab_id`, then `herdr tab focus <tab_id>` — `agent start` has no `--no-focus`; never `--current` (§3) |
