# Port notes — session group, cell ppb-12

Source: pstack poteto-mode at commit `b9ddc83`, MIT, copyright 2026 Lauren Tan
(notice in `skills/bee-planning/playbooks/LICENSE.pstack`). This file is the
only place a pstack-only tool is named; no file under `skills/` names one.
`skills/bee-planning/playbooks/session/index.md` gains five situation lines:
`babysit`, `shipping`, `autopilot-full`, `autopilot-stack`, `opening-a-pr`.

Added to every ported text: a reason on each step (decision `61b52e96`
requires one of class playbooks; pstack gave one only where it is kept in the
body), a Proof line, and the trigger line of the index. No Proof floor line:
the session group is no route class.

Recurring across the five files, handled the same way each time:

- The `show-me-your-work` skill (the `decisions.tsv` trail, never committed)
  became the decision trail under `docs/history/<feature>/`. The skill pointer
  is dropped (D5, slice 3 re-adds it). The "never committed" clause is dropped:
  bee's program folder is kept intact as the postmortem.
- The `swarm` skill is the fan-out of verifiers in autopilot-full step 4 and
  autopilot-stack step 4. The pointer is dropped (D5, slice 3 re-adds it). The
  fan-out itself stayed, as independent reviewers started with
  `bee dispatch prepare --kind reviewer`, aggregated to one verdict.
- The `no-comments` skill pointer is dropped (D5, slice 3 re-adds it). Its
  effect stays as a sentence: strip the comments the diff added, because bee
  writes no comment in code.
- The `deslop` skill from the team-kit plugin is dropped everywhere. It is a
  separate plugin's command with no bee counterpart and no D5 entry; the
  comment strip and `bee-unslop` over the prose cover what bee has.
- The `prove-it-works` principle skill is dropped as a pointer in
  autopilot-full step 2 (D5, slice 3). Its method stayed: self-proof on the
  real artifact.
- The control skills for the live lane (`control-cli`, `control-ui`) became the
  project's `verify-app` skill, or the real interface when the project has
  none.
- Graphite stack metadata, registration and submit commands have no bee
  counterpart. A pstack "PR in a stack" became a unit: one worktree branch,
  usually with a draft PR, merged by `bee worktree merge`.
- The Cursor `/goal` command and its `/loop` cadence became a checkable exit
  condition driven as `autonomous-run` does, and the audit tick armed under
  `bee-herding` (`bee herding control-loop`).
- The bug-triage reference file (`../references/...`) does not exist in bee;
  its rubric is dropped from every step that cites it, and the triage method
  that the playbook text itself states stays inline.

## babysit

Source `playbooks/babysit.md`, written to
`skills/bee-planning/playbooks/session/babysit.md`.

Tool swaps:

- The `watch-pr` script and its JSON verdicts became
  `gh pr view <n> --json mergeStateStatus,reviewDecision,statusCheckRollup` and
  `gh pr checks <n> --watch`, with the same rule: trust GitHub's merge state,
  not a hand-assembled check list. The review-comment text stays untrusted
  data.
- "Run `drive` and `background` under `/loop` in dynamic mode" became "work that
  must outlive the session hands the wait to `bee-herding`"; the "never add a
  second sleep loop" rule stays.
- The bot-triage reference became inline triage in step 8: verify each claim
  against the code, fix real findings red-first through
  `bee-principle-red-before-green`, dismiss noise with a disproof, lean toward
  dismissal from the third pass except security, auth, billing, data and
  migrations. The pass count came from the watcher; it now comes from the thread
  history.
- The "offer a dismissal pattern to the shared rubric" clause became recording
  it through `bee-capturing`.
- "Stack" and "frontier PR" became "unit" and the next unit to merge into main.
  "Never mutate stack topology" became "never rewrite history from inside a
  babysit": no rebase, no force-push, no merge of one unit's branch into
  another. The sanctioned follow-up when the owning unit has merged became a
  new unit branched from current main.
- "One babysitter per stack" became "per frontier", with the check named: an
  open owner session, a herding worker on that worktree.
- `git merge-base --is-ancestor main HEAD` kept as the stale-base test.

Dropped steps and clauses:

- The queued-mode terminals (`WAITING` with reason `merge-queue`, `ADVANCE`,
  `COMPLETE`): they are the watcher script's output states and bee has no
  merge queue. The frozen queue list and its one sanctioned revision stayed.
- "Never a job retry, because a retry reuses the original ref snapshot" in
  step 7: it describes the pstack CI vendor's retry. The one-retry limit and the
  reclassify-on-second-failure rule stayed.
- The four-column stack-status table in the Reply.
- The warning about a stacked PR merging into a parent with no required checks
  when merge-when-ready is armed: Graphite and merge-when-ready have no bee
  counterpart.

Changed (D4): pstack's babysit never lands. Decision `b749c98d` lists babysit
among the playbooks that merge on their own, so `drive` ends with
`bee worktree merge` on green and stops at the `uat` door of `bee close`. The
modes `check`, `threads-only` and `background` never land. Departure recorded:
the pstack line "babysitting never authorizes merging" is now "only `drive`
lands, and `shipping` adds an independent verdict for a run of several units".
The Proof line is new.

## shipping

Source `playbooks/shipping.md`, written to
`skills/bee-planning/playbooks/session/shipping.md`.

Tool swaps:

- "One subagent per PR, each a cloud agent, each exercising the control skill"
  became one reviewer per unit through `bee dispatch prepare --kind reviewer`,
  exercising the real surface through `verify-app` or the real interface.
- "Posts the verdict on its own PR" kept, and a row in the decision trail under
  `docs/history/<feature>/` added.
- Step 4, arming merge-when-ready through Graphite with `--always`, became
  `bee worktree merge --id <unit> --cleanup` from the main checkout, bottom unit
  first, without asking, with exactly one merger and a stop at the first
  refusal (`MERGE_CONFLICT` goes to a cell).
- Step 5, never enable GitHub auto-merge on a stack, kept the rule and swapped
  the reason: a self-merging PR skips the merge verb's proof-line and uat checks.
  The disarm command `gh pr merge <n> --disable-auto` stayed.
- Step 6, "do not read `autoMergeRequest` as proof that merge-when-ready is
  armed", became "confirm each landing from git": `git merge-base --is-ancestor
  <head> main` and `git log` on main.
- Step 7, never run Graphite from a worktree whose parentage is unchecked,
  became "while a merge runs, touch nothing else".
- Step 8, the queued watcher, became reporting each merge and the new ceiling as
  it lands; the diagnose-before-mutating rule stayed.

Dropped steps and clauses:

- The `graphite-base/*` refs and retargeting paragraph: a Graphite artifact.
- The `gt submit --stack` and `gt sync` prohibitions: replaced by the
  touch-nothing rule, since bee's merge is a synchronous verb and has no queue
  to drain.

Kept: the contiguous-run-from-the-bottom rule, the ceiling report, the
patch-id re-check and its twenty-one-stale-verdicts reason. Added (D4): the
final stop at the `uat` door.

## autopilot-full

Source `playbooks/autopilot-full.md`, written to
`skills/bee-planning/playbooks/session/autopilot-full.md`.

Tool swaps:

- "One cloud agent per PR" became one worker per unit through
  `bee dispatch prepare --kind cell` in its own worktree.
- The self-proof step kept its method and lost the `prove-it-works` pointer
  (see the recurring list).
- "Restack onto current trunk" became a rebase of the owner's own branch onto
  main; the rule that it precedes babysit and never waits stayed.
- "Squash-merges its own PR" became the root running
  `bee worktree merge --id <unit> --cleanup`. Departure recorded: bee allows
  exactly one merger and says workers never merge (see the orchestrate
  playbook, Merge safety), so the owner reports merge-ready and the root merges
  on the clean verdict. The verdict stays the merge authorization, as in pstack.
- "Merge only from a freshly restacked head" became
  `git merge-base --is-ancestor main <head>`; the patch-id rule in `shipping`
  step 3 is cited unchanged.
- "Arm a `/goal`" became a checkable exit condition driven as `autonomous-run`.
- The 30-minute audit tick on a terminal `/loop`, and the cloud sleeper wake
  chain, became a fixed-interval tick armed under `bee-herding`. The interval
  number is dropped: bee sizes it per run, as `autonomous-run` step 2 says.
- Re-reading the playbook from trunk became
  `git show main:skills/bee-planning/playbooks/session/autopilot-full.md`.
- "Probe each owner with a generic liveness check" became a read-only probe of
  the cell records, `git log` and the worktree list, with the no-resume rule
  from `orchestrate`.
- Every operator pronoun became the owner, with plain "they".

Dropped steps and clauses:

- "Every PR is still gt-registered; the Graphite-metadata rule is about the UI":
  Graphite has no bee counterpart.
- "Self-contained PRs branch straight off main" kept; "never stack" became
  "never chain", with the private chain exception kept.

Added (D4): the merge runs on a clean verdict without asking, and the `uat`
door of `bee close` stays the owner's.

## autopilot-stack

Source `playbooks/autopilot-stack.md`, written to
`skills/bee-planning/playbooks/session/autopilot-stack.md`.

Tool swaps:

- The owner loop, the audit tick, the `/goal` and the STACK-READY verification
  swapped exactly as in autopilot-full above.
- "Append the PR to the one linear Graphite stack" became appending the unit to
  one verified chain, with the order recorded in the decision trail.
- Step 6, `gt track` and `gt submit --stack` from the tip, became "single
  writer on order": the root owns the chain's order, an owner pushes only its own
  branch with `--force-with-lease` after an `ls-remote` check, and reports its
  tip and the unit it follows.
- Step 7, `gt restack` and `gt sync` by the root, became each owner rebasing its
  own branch; conflicts in an owner's files stay with that owner. The patch-id
  comparison and the countersign rule stayed.
- "The operator reviews and lands it with her own clicks, or with
  merge-when-ready she arms herself" became step 8: the root lands the verified
  chain bottom first with `bee worktree merge`, in one pass, and stops at the
  `uat` door of `bee close`, which is where the owner reviews.

Dropped steps and clauses:

- The cloud-environment rule that a cloud agent must never pull branches below
  its own into a Graphite submit, and the option to set a bottom PR's base
  directly: Graphite and cloud agents have no bee counterpart.
- "Reviewable bottom-up in the Graphite UI": the chain is reviewed from the
  per-link verdicts and at the uat door.

Changed (D4), a departure to check: pstack's terminal is "nothing auto-ships,
the operator lands". Decision `b749c98d` lists autopilot-stack among the
playbooks that merge on their own, as `uat_stop = "close"` does, so the
terminal became one ordered landing pass followed by the owner's uat. The
`uat_stop = "merge"` setting keeps pstack's behavior: the chain is presented for
the owner's yes before any merge, which the playbook states and does not
override. The "choosing between the autopilots" paragraph changed to match: full
lands each unit as its verdict arrives, stack lands the chain whole.

## opening-a-pr

Source `playbooks/opening-a-pr.md`, written to
`skills/bee-planning/playbooks/session/opening-a-pr.md`. The playbook is
invoked at the end of every other playbook; the trigger line of the index keeps
the situation reachable on its own.

Tool swaps:

- "Opening a PR" became the bee path: the `ship_visibility` draft-PR rule
  (`skills/bee-hive/references/scout-and-ticks.md`), `bee worktree merge`,
  `bee feature finish` and `bee close`.
- Writing the title, description and commit body with `/technical-writing` and
  `/unslop` became `bee-technical-writing` then `bee-unslop`. The Conventional
  Commits title rule, the five description sections and the no-boilerplate rule
  are not copied: `bee-technical-writing` already holds the title shape and
  section order, so one fact keeps one home. The "one word for each action, keep
  articles, avoid `-ing`" line stayed.
- "Commit liberally, rebase into small commits" kept, with the cap's
  `cell: <id>` trailer and the rule that after the cap a fix is a new commit.
- "Stack follow-ups with Graphite" became follow-ups branching from current
  main after the first merge.
- "A subagent that opens a PR runs interrogate, `/deslop` and `/no-comments`"
  became a worker that strips the comments its diff added, returns the URL and
  does not babysit.

Dropped steps and clauses:

- `/deslop` from the team-kit plugin, the `interrogate` skill (D5, slice 3
  re-adds it) and the `no-comments` skill pointer (D5, slice 3 re-adds it).
- The "Cloud-agent PR tools default to draft, so set `draft: false`" paragraph.

Changed to keep a locked bee rule: pstack says open every PR ready, never as a
draft. The `ship_visibility` rule says the draft stays a draft, "a window, not a
ship decision". Step 5 keeps the bee rule, and marking ready or merging is the
owner's call or the verb's. Departure recorded here.

Pstack's "Babysit" paragraph (opening a PR does not start a babysit) is kept as
step 7, with its reason.
