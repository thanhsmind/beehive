# bee-herding — Dispatch role spawn protocol (§5 set build – §8)

Read this file only after `role-dispatch.md` §5's interlock returns `enabled: true` and §4 found a free slot.

### 5. Build the dispatchable set

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
bee herding pane send-text <chat_pane_id> "dispatch: refusing <PBI-ID> — <what you read that concerned you>"
```

A silent refusal repeated every 60 seconds looks identical to nothing happening
at all — that is why it is announced.

**Key 2 asks a second question of the same text: is the CoS *checkable*?** You
already hold the candidate's `title+cos` from the read above — this is one more
question over those words, not a new pass over the backlog and not a third key.
A Condition of Satisfaction is checkable when it points at something an agent
can evaluate: a command it can run, a `file:line` it can open, or a repository
state it can inspect. Point at none of those and nothing can ever tell the
working agent it is finished — bee has no run-until-X loop, so the CoS *is* the
finish condition. Skip such a candidate, and make the skip line name why. Judge
this by reading, the way you judge danger: there is no word list here, and none
should be added — a list of vague-sounding words would miss the vague CoS
written in words nobody listed.

- **Not checkable** — `cos: "make the parser better"`. Nothing to run, nothing
  to open, no state to compare. Announce the skip on the chat pane, the same
  way a danger refusal is announced:

  ```
  SKIPPED p-xxxxxxxx — cos is not checkable: it names no command, path, or state an agent can evaluate.
  ```

- **Checkable** — `cos: "zero old callers remain and every parser fixture
  passes"`. Two evaluable facts: a repository state to inspect, and a command
  to run.

This fires at DISPATCH time only — nothing checks a CoS when it is written.

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
bee herding pane send-text <chat_pane_id> "dispatch: picking <PBI-ID> (<slug>) because <reason>"
```

Nothing survived §5/§6, or no slot is free (§4) → end the iteration quietly. An
empty runtime tab poll is normal, not an anomaly.

### 8. Spawn the working agent

In order, from the MAIN checkout:

1. `bee worktree new --feature <slug> --json` — creates and registers the
   worktree in one move; read the path from its output.
2. Split a runtime pane for the working agent. `bee herding agent-start`
   requires `--pane` and never creates, splits, or moves layout — splitting
   first is MANDATORY, not forbidden. `references/spawn-proof.md` is the
   recorded live evidence for this split-then-start order.
   ```
   bee herding pane split <runtime-pane-id> --direction right|down --ratio <r> --cwd <worktree_path>
   ```
   The split never takes focus and there is no flag either way — the verb
   applies the no-focus behavior itself, on every transport.

   Run `bee herding pane layout --pane <any runtime pane_id from §4>` (there is
   no `--tab` form, and `pane list` carries no geometry). Take the pane with the
   largest `width * height` **excluding the caller's own pane** as the parent,
   falling back to the caller's own pane only when the layout lists it alone.
   Pass `--direction right` only when the parent IS the caller's own pane (the
   first spawn, creating the worker column), and `--direction down` for any
   other parent (stacking subsequent workers inside that column). The ratio is
   the share the **parent** keeps: `0.5` for `down`, and
   `(parent_width - child_columns) / parent_width` for `right` where
   `child_columns` is `parent_width / 3` floored at 60 and capped at half.
   Read the new pane's id out of the reply with `bee herding result pane_id`.

   **Settle before starting the agent.** `bee herding agent-start` requires
   its target pane to already be at its interactive shell prompt; `pane split`
   returns as soon as the pane exists, before its shell has necessarily finished
   settling — the detection timeout the next verb carries covers agent
   *detection*, not shell *readiness*. Confirm (poll, or wait briefly and retry)
   that the new pane is at its shell prompt before issuing `agent-start`. This
   is the failure most likely to fire on Windows, where ConPTY starts slower.
3. Start the working agent into that pane.
   ```
   bee herding agent-start <slug> --kind <kind> --pane <new_pane_id> -- <agent args>
   ```
   The verb carries the 60-second agent-detection timeout itself; there is no
   `--timeout` to pass.
   `<kind>` and `<agent args>` come from `herding.agent_command`, per the next
   paragraph. **On any `agent-start` failure, close the pane step 2 of this
   section just created (`bee herding pane close <new_pane_id>`) before
   reporting** — an unlabelled pane
   whose `foreground_cwd` is the worktree is exactly what §4 classifies as an
   anomaly candidate, and this section created that pane, so this section is
   the one that cleans it up.

   **The trailing agent argv is config-driven, not hard-fixed prose.** Read
   `.bee/config.json`'s `herding.agent_command` from MAIN: a non-empty array
   of argv tokens has its first token supply `--kind` (the agent kind the
   transport recognizes, e.g. `claude`) and its remaining tokens, substituted
   per-token (`{MODEL}` → `sonnet`) and each passed as one discrete argv
   element, go after `--` as the agent's own arguments — never re-parsed or
   shell-interpreted, and never re-used as `--kind`. An unrecognised token 0
   is a typed error naming `herding.agent_command`. Absent, not an array, or
   empty — the common case — use `--kind claude -- --model sonnet
   --permission-mode bypassPermissions "<opening instruction>"`. Shape and
   examples: `references/operational-invariants.md` ("Runtime adapter").

   **The argv must carry the working agent's opening instruction.** A bare
   `claude` starts with an empty buffer and never self-names; the positional
   prompt tells it to (a) run `bee herding pane current` then
   `bee herding pane rename <pane_id> <slug>` as its very first act, labelled
   with the **bare slug** and nothing else (the anomaly scan in §4 and merge's
   pane lookup both key off it), and (b) work `<PBI id>` routing through
   `bee-hive`.

   **Never pass `-p`/`--print`.** A headless argv exits on completion and the
   transport closes the pane with it; the working agent must be an interactive
   `claude` that stays alive for the whole item. (`bee herding control-loop`
   uses `claude -p` for control panes.) The working agent keeps
   `bypassPermissions` with no allowlist — the owner's recorded accepted risk
   (`references/operational-invariants.md`), not a default to trim.

   Afterwards, confirm: `bee herding pane list --workspace <workspace_id>`
   filtered to the runtime tab shows exactly **one** new pane — the one step 2
   of this section split — live agent, right cwd, not two, not zero. Anything
   wrong → apply the pane-close rule above if `agent-start` itself failed,
   report one plain line into the chat pane, and do **not** blindly repeat the
   spawn next iteration: a blind retry is how a cold loop turns one mistake
   into 1440 a day.
4. **Record the spawn in the wave ledger — only after that confirm step
   passed.** This is what closes the loop: **a spawn that is not recorded here
   is invisible to the next iteration's occupancy read (§4)** — §4 reads the
   wave ledger, and a spawn with no row in it simply is not in what it reads.
   From the MAIN checkout:
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
   `bee herding pane send-text <chat_pane_id> "dispatch: recording <slug> (pane <new_pane_id>) in the wave ledger FAILED — occupancy will undercount until this is repaired"`
   — and still end the iteration without spawning again this poll: do not
   retry the recording call, and do not spawn a second worker to compensate.

5. **Give the human their view back.** `agent-start` cannot decline focus and
   moves the workspace's focus onto the new agent's tab. Read your own tab from
   your own pane context, then focus it back:
   ```
   bee herding pane current                # its tab_id: bee herding result tab_id
   bee herding pane tab-focus <that tab_id>
   ```
   `bee herding pane current` always answers the CALLING pane, never the
   globally focused one — after `agent-start` the focused pane is the worker's.
   A failure here is cosmetic, not structural: report it in one line and end
   the iteration normally, never retry the spawn over it.

The working agent is on its own from there — it runs the ordinary bee chain
inside its worktree until its item is finished. This role does not watch it,
wait on it, or act on it again; the next iteration's occupancy count (§4) is
how its progress is next observed.
