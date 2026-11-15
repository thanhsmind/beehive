# ppb-11c — session port checked against its sources

Scope: the five session playbooks and `session/index.md` written by ppb-11
(commit `0adfdf223`), checked against the pstack sources under
`pstack/skills/poteto-mode/playbooks/` and the port notes
`docs/history/pstack-playbook-port/port-notes/session-ppb-11.md`.

Verdict: every numbered source step is present in all five files. The
pstack-only tool fence is clean and the index grammar holds. Two port-notes
gaps need a fix before the session group closes. The `worktree-cleanup`
section is missing from the port notes entirely, and the orchestrate port
dropped `figure-it-out` and the `inbox/` store entry without a note. The
`inbox/` drop also leaves dangling references to "the inbox".

Labels used below: **present** (the step or clause is in the new file, in
bee's terms where a tool was swapped); **dropped (noted)** (the step or clause
is gone and the port notes name it); **dropped (not noted)** (it is gone and
the port notes are silent, which D2 counts as missing); **missing** (a step
whose method is gone with no reason given).

## Old enforced lines

`git ls-tree 0adfdf223^ skills/bee-planning/playbooks/` shows that no session
playbook existed before ppb-11. The session group is no route class. It had no
class playbook, so it had no enforced lines (proof floor, routed principles,
statechart or formal-verify steps) for D3 to carry. Nothing was lost. The port
notes say why no Proof floor line was added: the cap's floor table has no
session row.

## session-pickup

| Source | Status |
|---|---|
| Lead line "You own the resume point…" | present |
| Triggers (take over, resume, continue from transcript, you're taking over, pick up where X left off, pushed branch) | present (index line) |
| Trigger: cloud-agent URL handoff | dropped (noted) |
| Inheritance paragraph | present |
| 1 Locate the prior trail | present: swapped to `.bee/HANDOFF.json` / `bee state handoff show --json`, with the pause/planned-next rule; transcript or branch kept |
| 1 clause: do not glob `~/.cursor/projects/*/` | dropped (noted); its reason kept as "a transcript from another project is a private chat" |
| 1 clause: parse a long transcript in a subagent | present: swapped to `bee dispatch prepare --kind gather` (kind `gather` exists) |
| 1 pointer: principle-guard-the-context-window | dropped (noted); method kept inline |
| 2 Reconstruct operational state | present ("open todos" became "open cells") |
| 3 Diff done vs pending | present |
| 4 Route remaining work, pick the verdict | present: swapped to `bee orient` |
| 5 Verify inherited claims on the real artifact | present: `verify-app` or the real interface |
| 5 pointer: principle-prove-it-works | dropped (noted) |
| Reply | present |

One accuracy note: step 1 names `.bee/HANDOFF.json` as the location.
`bee state handoff show --help` says a resolved workflow reads its own mailbox
under `.bee/runtime/handoffs/<workflow-id>/` instead. The command is right but
the file name is only true for repos without workflow records. This is minor.
Naming the command first would be enough.

## pause-safely

| Source | Status |
|---|---|
| Lead line "You own a clean stop…" | present |
| Triggers: pause safely, go offline, board my flight | present (index line) |
| Trigger: restart Cursor | dropped (noted) |
| Trigger: context about to compact | changed (noted): now writes a file note and keeps working, following the bee rule `agents-context-handoff-65` |
| Explicit-only rule; keep going / going to bed / don't stop mean continue | present |
| 1 Stop at a safe boundary; cancel nested subagents | present: "let any dispatched worker finish or cancel it" |
| 2 Don't cross an irreversible line | present: swapped to `ship_visibility` |
| 3 One `wip:` commit; broken tree named in body | present (adds: no `cell:` trailer) |
| 4 Resume note off-context (intent, doing, progress, verified, state, next steps, key files, gotchas) | present: `bee state handoff write --kind pause` with `--cell/--files/--done/--remaining/--next-action` (all real flags); `/tmp/<slug>-resume.md` kept for compaction |
| 4 show-me-your-work trail pointer | present: swapped to the decision trail under `docs/history/<feature>/` (noted) |
| Reply; resume is Session pickup | present |

## autonomous-run

| Source | Status |
|---|---|
| Lead line | present |
| Triggers: going to bed, run until done, `/loop until X` | present ("keep going until X"; `/loop` swap noted) |
| 1 Exit condition as a predicate | present ("all N PRs merged" became "all N cells capped"; not noted, but a plain PR-to-cell swap) |
| 2 Wake mechanism via `/loop` | present: swapped to `bee-herding` / `bee herding control-loop` (noted) |
| 2 event-watcher subagent with heartbeat fallback | dropped (noted) |
| 2 fixed interval sized to re-check value | present |
| 3 Smallest change per iteration; revert "might help" | present |
| 3 pointer: sequence-verifiable-units | dropped (noted); method kept inline |
| 4 Mid-run discoveries are yours | present: "via poteto-mode" became `bee orient`; own PR became own cell; `AskQuestion` became "do not stop to ask" (all noted) |
| 5 Checkpoint each iteration (show-me-your-work) | present: swapped to a decision-trail row (noted) |
| 6 Stop on predicate; plateau is not a stop; never relax | present |
| D4 landing clause | added (noted): `bee worktree merge` on green, stop at the `uat` door |
| Reply | present |

## orchestrate

| Source | Status |
|---|---|
| Lead line; whole-project scope; head-to-head 12-unit measurement; route below the line to autonomous-run | present |
| Intro: "One ambitious run needing a bespoke workflow is figure-it-out." | **dropped (not noted)**: the D5 slice-3 skill `figure-it-out` is named in the notes' drop list for other cells but not here |
| Ceremony scales with the program | present |
| Three rules | present |
| Todolist with `skip: <reason>` | present |
| Roles: Coordinator never edits code; mechanical landing | present: `bee worktree merge` |
| Roles: Task tool spawns | present: `bee dispatch prepare` (noted) |
| Roles: `scripts/orch/orch.ts` state reads at drains | dropped (noted) |
| Roles: "Local" / "Always local" placement; cloud default for workers and its exception list; cloud briefs inline what they need | dropped (noted as the cloud-environment rule) |
| Roles: nesting to depth 3; nested spawn carries the full Task schema | dropped (not noted): a Task-tool detail, covered in spirit by the Task swap |
| Roles: sub-coordinator rolling window cap (~10) | present |
| Roles: verifier on a different model family | present |
| Depth stays coordinator/track/worker; hard-coded trees parked | present |
| Store layout (agent store) | present: swapped to the program folder under `docs/history/<feature>/` (noted) |
| `preferences.md` standing orders | present ("stack shape and count" became "merge order"; not noted, falls under the stack drop) |
| `overview.md`, `frontier.json`, `ledger.tsv` | dropped (noted) |
| `units.tsv` | present (PR column dropped, not noted; falls under the stack drop) |
| `inbox/` completion pointers | **dropped (not noted)**: the store list no longer names it, but the text still says "drain the inbox" (Roles), "Drain the final inbox" (step 7) and "a synthetic postmortem row in the inbox" (Liveness), and "record the pointer in the program folder" names no file. The fix is to keep `inbox/` in the program record list or to rename the three references |
| `gates.md` | present ("AskQuestion state" became "a pending question") |
| `decisions.tsv` trail | present (noted) |
| `status.md` derived, never hand-kept | present (now from `units.tsv` only) |
| The brief template (all nine fields) | present: `gt` gone from FORBIDDEN, PRs gone from CONTEXT/REPORT, control-skill became verify-app |
| Size the brief to the unit | present (cloud-spawn clause dropped, under the cloud note) |
| Sub-coordinator brief additions | present (cloud default and frontier delta became merge delta) |
| Dependency is a context relay; refuse-to-spawn; sampled audit; never resume-chain | present |
| Step 1 Frame (countable predicate, quantify, collapse to autonomous-run, 70% landing cut, tracks, present once) | present |
| Step 1 arena for a contested decomposition | dropped (noted) |
| Step 2 Install runtime (`orch init`, trail, standing orders, seed frontier) | present: program record, orders before spawn, seed `units.tsv` from cells and worktrees (`orch` noted) |
| Step 3 Pilot | present |
| Step 4 Scale | present |
| Step 5 Drain | present |
| Step 6 Land (continuous landing, frontier green) | present: `bee worktree merge` plus D4 `uat` door; the standing-stacker clause falls under the Stack safety drop (noted) |
| Step 7 Close | present; the cross-model trail audit was dropped (noted) |
| Queue: `orch inbox push/drain`, `orch unit/ledger/status` | dropped (noted); prose replaces them |
| Queue: frontier-watcher wake via the loop skill | present: swapped to the herding loop wake (noted) |
| Queue: critical sections, classify pointers, account for every child, three-line drain turn | present |
| Stack safety section | replaced by Merge safety (noted): recompute from git, one merger, workers never rebase or merge, retro watcher kept; stacker, restack in cloud and babysit pointer dropped (noted) |
| Verification: scale to unit; verdict scale; CI green is an input; new SHA voids; externalize on landing | present ("pushes its branch" became "commits in its worktree"; not noted) |
| Verification: `orch ledger record/check` | dropped (noted) |
| Liveness: never resume to check; silent death; retry by mode; zombie; stop line; bound own retries | present; the terminal handoff swapped to `bee state handoff write --kind pause` (noted) |
| Liveness: Cursor dashboard and `gh` probes | dropped (dashboard noted, `gh` not noted) |
| Liveness: Cursor restart, reattach cloud work, `orch` lock | dropped or swapped (noted) |
| Escalation: reaches the human / never reaches / mid-run discoveries | present ("restack mechanics" gone with the stack drop) |
| Reply | present; PR links dropped (noted) |

## worktree-cleanup

**The port notes have no `worktree-cleanup` section.** Every swap and drop
below is undocumented, which D2 requires. The steps themselves are all
present.

| Source | Status |
|---|---|
| Title "Worktree and simulator cleanup"; lead line | present |
| 1 Snapshot and audit: `df -h /`, then `scripts/worktree-audit.sh` | present: swapped to `bee worktree prune --dry-run` from the main checkout (real flag). **not noted** |
| 1 Lever classifies by size, age, merge state, uncommitted work, PR state, newest chat; suggests a bucket | present in part: prune reports a size or the keep-condition. PR state and newest-chat classification dropped, **not noted** |
| 1 Transcript scan is slow, background it | dropped, **not noted** |
| 1 `.cursor/worktrees/` hand-typed-path example | dropped (the generic reason kept), **not noted** |
| 1 principle pointers build-the-lever, encode-lessons-in-structure | present as `bee-principle-*` (both exist) |
| 2 Bucket is advice; pinned chats are the artifact | present: "open sessions" and the owner's set |
| 2 pointer principle-prove-it-works | dropped, **not noted** |
| 3 Verify usage via subagents reading transcripts | present: gather via `bee dispatch prepare --kind gather` over session records. **not noted** |
| 3 `verify-recent-chat` bucket name | dropped, **not noted** |
| 3 pointer principle-guard-the-context-window | dropped (reason kept inline), **not noted** |
| 3 "A pinned chat spawns arena and repro trees" | present without arena ("a herded worker spawns repro trees"). **not noted** |
| 4 Pause on irreversible loss (`wip:N`, `scratch:N`, "Per Autonomy") | present: bucket labels and the Autonomy pointer dropped, method kept. **not noted** |
| 5 Prune: `git worktree remove --force`, `rm -rf`, `git worktree prune` | present: swapped to `bee worktree prune` (`--older-than-days` is real; `git worktree lock` opt-out matches the help). **not noted** |
| 6 Simulators: the three `xcrun` commands, DerivedData, DeviceSupport, package caches | present |
| 6 `~/Library/Application Support/Cursor` reclaimer (`state.vscdb.backup`, `snapshots/roots`) | dropped, **not noted** |
| Closing "no code review" sentence | present |
| Reply | present ("chat" became "session") |

## pstack-only tool names under `skills/bee-planning/playbooks/`

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/`
returns no match (exit 1). A wider scan of `session/` for
`gt`, `orch`, `AskQuestion`, `arena`, `figure-it-out`, `show-me-your-work`,
`poteto`, `/loop`, `Task tool` and the `principle-guard|prove|sequence`
pointers also returns no match (exit 1).

## Skills named that do not exist in `skills/`

Named and present: `bee-herding`, `bee-principle-build-the-lever`,
`bee-principle-encode-lessons-in-structure`,
`bee-principle-separate-before-serializing-shared-state`.

Not in `skills/`: `verify-app` (`session-pickup.md:36`, `orchestrate.md:92`).
It is named as the host project's own skill, "when it has one, otherwise the
real interface". The plan's port rule asks for exactly that, and
`bugfix/bug-fix.md:15` uses the same wording. I accept it as sanctioned, not a
break. It does conflict with the literal rule "name no skill that does not
exist in skills/", so the plan or the rule should say which wins.

## Index grammar (contract `913115b3`)

<!-- bee:not-a-deferral: a record or description of work already done, never a promise to act -->
`session/index.md` has a `## Situations` heading and five lines. All five
match `^- \`[a-z0-9-]+\` — .+ — delivers: .+$` (5 of 5). Each name matches a
file in `session/`, each trigger list has English and Vietnamese phrases, and
no later `## ` heading follows, so the section runs to the end of the file. The
grammar holds.
<!-- /bee:not-a-deferral -->

## Findings to act on

1. Add a `## worktree-cleanup` section to `port-notes/session-ppb-11.md`,
   listing the swaps and drops in the table above.
2. Note the dropped `figure-it-out` sentence in the orchestrate section (D5,
   slice 3 re-adds it).
3. Fix the orchestrate `inbox/` gap: either keep `inbox/` in the program
   record list or reword the three "inbox" references, and note the decision.
4. Optional: in `session-pickup.md` step 1, lead with the command rather than
   the `.bee/HANDOFF.json` path, since workflow repos keep the handoff in their
   mailbox.
