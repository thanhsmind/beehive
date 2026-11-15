# Port notes — session group, cell ppb-11

Source: pstack poteto-mode at commit `b9ddc83`, MIT, copyright 2026 Lauren Tan
(notice in `skills/bee-planning/playbooks/LICENSE.pstack`). This file is the
only place a pstack-only tool is named; no ported file names one.
The session group has no route class: `skills/bee-planning/playbooks/session/index.md`
is its index, and ppb-12 appends its five situations to it.

Added to every ported text: a reason on each step (decision `61b52e96` requires
one of class playbooks; pstack gave one only where it is quoted in the body), a
Proof line, and the trigger lines of the index. The Proof floor line is not
added: the session group is no route class and the cap's floor table has no row
for it.

Three pstack tools recur across the five playbooks. Each was swapped or dropped
the same way in every file, and the sections below list the occurrences.

- The `principle-guard-the-context-window`, `principle-prove-it-works` and
  `principle-sequence-verifiable-units` skills have no bee skill yet (D5 brings
  them in slice 3). Each pointer is dropped and its method sentence stays inline.
  Still waiting (ppb-24): no such skill exists in `skills/`, so the sentences stay and the three names stay here.
  Restored (ppb-37): the three principles exist now as
  `bee-principle-guard-the-context-window`, `bee-principle-prove-it-works` and
  `bee-principle-sequence-verifiable-units`, and each pointer is back in the
  file named in its own entry below.
- The show-me-your-work skill has no bee skill yet (D5, slice 3). Its trail
  became a decision trail under `docs/history/<feature>/`; the skill pointer is
  dropped. Restored (ppb-24): `pause-safely`, `autonomous-run` and
  `orchestrate` name `bee-show-me-your-work` for the trail.
- The Cursor-only wake, transcript and dashboard machinery has no bee
  counterpart and is dropped where listed.

## session-pickup

Source `playbooks/session-pickup.md`, written to
`skills/bee-planning/playbooks/session/session-pickup.md`.

Tool swaps:

- The trail locations (a transcript under the editor's `agent-transcripts/`
  directory, a cloud-agent URL, a pushed branch) became `.bee/HANDOFF.json` read
  with `bee state handoff show --json`, with the kind rule from the bee-hive
  resume logic: `pause` waits for the owner, `planned-next` is adopted with
  `bee state handoff adopt` only in a fresh session. A transcript or pushed
  branch the owner names stays as a source.
- Parsing a long transcript in a subagent became a gather,
  `bee dispatch prepare --kind gather`.
- Routing the remaining work to the matching playbook became `bee orient`.
- Verifying inherited claims on the real artifact gained the project's
  `verify-app` skill, otherwise the real interface.

Dropped steps and clauses:

- The cloud-agent URL as a handoff source: bee has no counterpart.
- The instruction not to glob `~/.cursor/projects/*/`: the editor's store, not
  bee's. The reason it gave survives as "a transcript from another project is a
  private chat".
- The `principle-guard-the-context-window` pointer in step 1 and the
  `principle-prove-it-works` pointer in step 5: dropped, method text kept inline
  (slice 3 re-adds the pointers).
  Restored (ppb-37): step 1 names `bee-principle-guard-the-context-window` and
  step 5 names `bee-principle-prove-it-works`.

## pause-safely

Source `playbooks/pause-safely.md`, written to
`skills/bee-planning/playbooks/session/pause-safely.md`.

Tool swaps:

- The resume note written to a file became `bee state handoff write --kind
  pause` with `--cell`, `--files`, `--done`, `--remaining` and `--next-action`.
  The `/tmp/<slug>-resume.md` file stays for the compaction trigger only.
- "Cancel any nested subagents" became "let any dispatched worker finish or
  cancel it".
- "No PR and no push unless you already had one out" became "no new PR and no
  push beyond what `ship_visibility` already pushes on a cap".
- "Point at a show-me-your-work trail" became "point at the feature's decision
  trail under `docs/history/<feature>/`".

Dropped steps and clauses:

- The editor restart as a trigger ("restart Cursor"): bee has no counterpart.
  The trigger line keeps going offline and boarding a flight.

Changed to keep a locked bee rule: pstack lists "context is about to compact" as
a pause trigger. The bee handoff rule (`bee-hive`, rule `agents-context-handoff-65`)
says a context mark never stops work and a handoff happens only on the owner's
word or at feature close. The compaction case therefore writes the file note and
keeps working, with no handoff record. Departure recorded here; the playbook
states it in its intro and step 4.

## autonomous-run

Source `playbooks/autonomous-run.md`, written to
`skills/bee-planning/playbooks/session/autonomous-run.md`.

Tool swaps:

- The editor's `/loop` command, a watcher subagent and the time-based heartbeat
  became `bee-herding` and `bee herding control-loop`: one cold iteration per
  wake, state read from disk, interval sized to when the result is worth
  re-checking.
- "Address ... yourself via poteto-mode" became "routed through `bee orient`
  like any task".
- "Put out-of-band fixes in their own PR" became "in their own cell".
- The `AskQuestion` tool became "do not stop to ask".
- The `principle-sequence-verifiable-units` skill: pointer dropped, the sentence
  "verify each unit before the next instead of batching checks at the end" kept inline.
  Restored (ppb-37): the sentence names `bee-principle-sequence-verifiable-units`.
- The show-me-your-work checkpoint became a row in the feature's decision trail
  under `docs/history/<feature>/`.

Dropped steps and clauses:

- The event-watcher subagent that wakes the run on CI, a merge or a ref
  advancing: bee has no event wake. Only the interval remains.

Added (D4): the landing clause at the end of step 6. When the work lives in a
worktree, it merges on green with `bee worktree merge` from the main checkout
without asking, and stops at the `uat` door of `bee close`.

## orchestrate

Source `playbooks/orchestrate.md`, written to
`skills/bee-planning/playbooks/session/orchestrate.md`. The structure and the
three rules, the brief template, the queue discipline, the verification verdict
scale, the liveness rules and the escalation lists are kept.

Tool swaps:

- Spawning, resuming and draining agents through the Task tool became
  `bee dispatch prepare`; the unattended loop is `bee-herding`.
- Units became bee cells. The cell record stays the source of truth for the
  claim and the cap, and the cap's proof line is the worker's verdict.
- Landing a verified unit became `bee worktree merge` from the main checkout on
  green without asking, stopping at the `uat` door of `bee close` (D4).
- The restart recovery ("local agents are dead, cloud work is not") became
  "dispatched local workers are dead, committed work is not".
- The terminal handoff after repeated tool aborts became `bee state handoff
  write --kind pause`.
- The store under the agent's own folder became a program folder under
  `docs/history/<feature>/`; the file names `preferences.md`, `units.tsv`,
  `gates.md`, `decisions.tsv` and `status.md` are kept.
- `principle-separate-before-serializing-shared-state` and
  `principle-encode-lessons-in-structure` became `bee-principle-` skills that
  already exist.
- The show-me-your-work trail (`decisions.tsv`) became the decision trail.

Dropped steps and clauses:

- The `orch` CLI (`scripts/orch/orch.ts`) and its commands: `orch init`,
  `orch frontier set`, `orch inbox push` and `drain`, `orch unit add` and `set`,
  `orch ledger record` and `check`, `orch status`. bee has no counterpart. The
  rows they wrote are written by hand to the program folder, and `status.md`
  stays derived from `units.tsv`.
- `overview.md` (the PR and issue DB), `frontier.json` and `ledger.tsv`: the
  PR database and the computed stack frontier belong to a stacked-PR tool bee
  does not have; the verdict scale moved to a per-unit record keyed by cell and
  head SHA.
- The Stack safety section (the stacker, restacks in the cloud, the frontier
  recomputed from the stack tool, the babysitter scoped to one frontier
  generation, PR closes and retargets): replaced by a short Merge safety section
  that keeps what holds without stacks (recompute from git, one merger, workers
  never rebase, one retro watcher). The babysitter pointer is dropped here; ppb-12
  brings `babysit`.
- The cloud-environment rule for workers and its exception list (control-ui,
  control-cli, local transcripts, simulators, local auth): bee workers run in
  their own worktree, so there is no placement choice.
- The cloud agents' dashboard as a liveness probe, and the editor-restart
  paragraph about cloud agents.
- The `arena` skill for a contested decomposition or one-way door (Frame step):
  no bee skill yet (D5, slice 3). Restored (ppb-24): the Frame step runs
  `bee-arena` before the pilot.
- The cross-model review audit of the trail in the Close step: the show-me-your-work
  skill that defines it has no bee skill yet; the per-unit verifier on another
  model family stays. Restored (ppb-24): the Close step audits the trail per
  `bee-show-me-your-work`, including its cross-model review.
- The loop skill that arms a frontier watcher: the herding loop replaces it with
  an interval only.
- The PR links in the closing Reply: a program lands units, not necessarily PRs.
- The sentence "One ambitious run needing a bespoke workflow is figure-it-out"
  in the intro: the `figure-it-out` skill has no bee skill yet (D5, slice 3
  re-adds it with the fallback line in every index). Restored (ppb-24): the
  intro names `bee-figure-it-out`.
- Nesting to depth 3 and the full Task schema on a nested spawn: Task-tool details. `bee dispatch prepare` replaces the Task tool, and depth stays at coordinator, track, worker.
- The `gh` probes in Liveness: bee reads liveness from `units.tsv`, the cell
  records, `git log` and the worktree list, which are local and need no host
  CLI.
- "Pushes its branch" in the Verification paragraph became "commits in its
  worktree": a unit lands through `bee worktree merge`, so no push stands
  between the worker and the merger.
- The PR column of `units.tsv`: a unit is a cell in a worktree and not always a
  PR, so the row keeps the branch and head SHA only.
- "Stack shape and count" in `preferences.md` became "merge order", because bee
  has no stack to shape or count and the order of merges is the standing
  constraint left.

Decision `b25486e3` on the `inbox/` store entry: `inbox/` stays in the program
record list as one completion-pointer file per finished unit, written by the
unit's owner and read and cleared only by the coordinator at a drain. The first
port dropped the entry but kept three references to "the inbox" (Roles, Close,
Liveness), and the Queue step recorded the pointer in "the program folder"
without naming a file; all four now name `inbox/`. The `orch inbox push` and
`drain` commands stay dropped, since the files are written by hand.

## worktree-cleanup

Source `playbooks/worktree-cleanup.md` ("Worktree and simulator cleanup"),
written to `skills/bee-planning/playbooks/session/worktree-cleanup.md`. The
steps and the closing "no code review" sentence are kept.

Tool swaps:

- Step 1, `df -h /` then `scripts/worktree-audit.sh`, became `df -h /` then
  `bee worktree prune --dry-run` from the main checkout. The script's
  classification by size, age, merge state, uncommitted work, PR state and the
  newest chat is narrower here: prune reports a size or the condition that keeps
  a worktree, and the PR-state and newest-chat columns have no bee counterpart.
- Step 3, verifying usage through subagents that read transcripts, became a
  gather through `bee dispatch prepare --kind gather` over session records.
- Step 3, "a pinned chat spawns arena and repro trees", became "a herded worker
  spawns repro trees", with `arena` dropped (D5, slice 3). Restored (ppb-24):
  the step reads "a herded worker spawns arena and repro trees", because a
  `bee-arena` run spawns candidate trees.
- Step 5, `git worktree remove --force`, `rm -rf` and `git worktree prune`,
  became `bee worktree prune`, with `--older-than-days` for the age cut and a
  `git worktree lock` as the opt-out the verb's help names.
- "Chat" became "session" throughout, including the Reply.
- The `principle-build-the-lever` and `principle-encode-lessons-in-structure`
  pointers became the `bee-principle-` skills that already exist.

Dropped steps and clauses:

- The transcript scan note ("slow, background it") in step 1: bee has no
  transcript scan.
- The `.cursor/worktrees/` hand-typed-path example in step 1: an editor path.
  The generic reason it illustrated stays.
- The `principle-prove-it-works` pointer in step 2 and the
  `principle-guard-the-context-window` pointer in step 3: dropped, method kept inline (D5, slice 3 re-adds them). Restored (ppb-37): step 2 names
  `bee-principle-prove-it-works` and step 3 names
  `bee-principle-guard-the-context-window`.
- The `verify-recent-chat` bucket name in step 3, and the `wip:N` and `scratch:N`
  labels and the "Per Autonomy" pointer in step 4: bucket names of the pstack
  audit script. The pause-on-irreversible-loss method stays.
- The `~/Library/Application Support/Cursor` reclaimer (`state.vscdb.backup`,
  `snapshots/roots`) in step 6: an editor cache. The three `xcrun` simulator
  commands, DerivedData, DeviceSupport and the package caches stay.
