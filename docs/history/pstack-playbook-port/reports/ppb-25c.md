# ppb-25c — slice 2 fixes checked

Scope: commit `ffc6c41ff` (cell ppb-25), checked against every finding in
`reports/ppb-9c.md`, `reports/ppb-11c.md` and `reports/ppb-12c.md`. ppb-25
worked under decision `b25486e3`, which names its scope: "ppb-9c finding 1,
ppb-11c findings 1-3, ppb-12c findings 1-5". Findings outside that list are
still checked, and each one is marked out of scope where ppb-25 was not asked
to fix it.

Verdict: every finding inside the `b25486e3` scope is fixed or noted. Three
ppb-9c findings and one optional ppb-11c finding were outside that scope and
are still open. The fix opened two new gaps: the `multi-phase-plan` skeleton
still has owners rebase, and one restored note in `session-ppb-12.md` sits in
the middle of a bullet list, so the bullets under it read as restored items.
Neither blocks the cap.

Labels: **fixed** means the playbook text now does what the finding asked.
**noted** means the port notes now name the drop or change, which is what D2
asks. **still open** means neither.

## ppb-9c (feature port)

| # | Finding | Status | Evidence |
|---|---|---|---|
| 1 | `feature/feature.md` has no Proof floor line | fixed | `feature/feature.md:98`. All 21 `^Proof floor` lines under `skills/bee-planning/playbooks/` are byte-identical (`rg -I '^Proof floor' … \| sort \| uniq -c` gives one line, count 21) |
| 1b | `multi-phase-plan.md` has no floor line (consistency only, not a D3 merge) | still open, out of scope | not in `b25486e3`. The checker said it was optional |
| 2 | `multi-phase-plan.md` step 3 says "explore role"; bee's gather runs on the `read` role | still open, out of scope | `feature/multi-phase-plan.md:17` still reads "explore role's configured model" |
| 3a | "Rebase into small, ordered commits" became "Leave small, ordered commits" with no note | still open, out of scope | `port-notes/feature.md` has no `rebase` match, and the file is unchanged since `3ba27a8d1` |
| 3b | the rebuttal of "the app is small" is dropped with no note | still open, out of scope | `port-notes/feature.md` has no "app is small" match |
| 4 | the `encode-lessons-in-structure` pointer could cite `bee-principle-encode-lessons-in-structure` | still open, deferred | `port-notes/feature.md:141` still lists it as dropped. The checker filed it for the slice 3 re-add list, not as a defect |
| — | stale pointer `bugfix/bug-fix.md` cites `feature.md` steps 6 and 7 | fixed | `bugfix/bug-fix.md:57-59` now cites `feature/feature.md` steps 3 and 8 |
| — | rendered trees hold the old class playbooks | fixed | `diff -r` of `skills/bee-planning/playbooks` against each of `.claude-plugin`, `.codex-plugin`, `.claude`, `.agents` and `.opencode` under `skills/bee-planning/playbooks` reports no difference |

## ppb-11c (session port, first half)

| # | Finding | Status | Evidence |
|---|---|---|---|
| 1 | `port-notes/session-ppb-11.md` has no `worktree-cleanup` section | noted | new `## worktree-cleanup` section. Every row the checker marked **not noted** is covered: the prune swap, the narrower classification (PR state and newest chat), the transcript-scan note, the `.cursor/worktrees/` example, `prove-it-works`, the gather swap, `verify-recent-chat`, `guard-the-context-window`, the arena drop, the `wip:N`/`scratch:N`/Autonomy labels, the `bee worktree prune` swap and the Cursor reclaimer |
| 2 | the `figure-it-out` intro sentence is dropped with no note | noted | orchestrate drop list, cited to D5 and slice 3 |
| 2b | nesting to depth 3 and the Task schema are dropped with no note (table row only) | noted | orchestrate drop list |
| 3 | `inbox/` is gone from the store list, but three references remain | fixed | `orchestrate.md:73` restores `inbox/` in the store list. Roles (`:33`), Close (`:163`) and Liveness (`:235`) still say inbox, and Queue (`:172`) now names `inbox/` instead of "the program folder". Recorded as decision `b25486e3` in the notes |
| 4 | optional: `session-pickup.md` step 1 should lead with the command, not `.bee/HANDOFF.json` | still open, out of scope | `session-pickup.md:9-10` unchanged |
| — | table-only rows marked "not noted": the `gh` liveness probes, "pushes its branch" changed to "commits in its worktree", the `units.tsv` PR column, "merge order" in `preferences.md` | still open, minor | the checker did not list these as findings to act on, and `session-ppb-11.md` has no match for them |

## ppb-12c (session port, second half)

| # | Finding | Status | Evidence |
|---|---|---|---|
| 1 | `babysit` waits for `CLEAN`, which a draft PR never reads | fixed | step 6 gates on `statusCheckRollup` and `reviewDecision` and says the merge state is no gate. Step 6's stop, step 9 and the Proof line drop `mergeStateStatus`, and step 9 adds "never run `gh pr merge`". The only `CLEAN` left (`babysit.md:63`) explains why it is no gate. `shipping.md` never gated on `CLEAN`, and its step 5 already forbids `gh pr merge`. The swap is noted in `session-ppb-12.md` |
| 2 | autopilot owners rebase, while `orchestrate` § Merge safety says workers never rebase | fixed in the autopilots | `autopilot-full.md:25` and `:53`, `autopilot-stack.md:56-57`, and `babysit.md` steps 4, 5 and 7 now send a conflict, drift or a stale base to the merger as a cell. Noted in `session-ppb-12.md`. See new finding A for one place this missed |
| 3 | the CI-retry note's reason is doubtful | fixed | `babysit.md` step 7: "one fresh run on a fresh ref… Never retry the failed jobs, because a retry reuses the original ref snapshot". The note is now a "Restored" entry. See new finding B for where it sits |
| 4a | babysit's Cursor routing sentence | noted | babysit drop list |
| 4b | shipping's "enable merge when ready" trigger | noted | shipping drop list |
| 4c | shipping's re-parent clause | noted | shipping drop list |
| 4d | autopilot-full step 6 "collect the decision trails" | fixed | `autopilot-full.md:72-73` restores it. Noted as "Restored" |
| 4e | opening-a-pr's `git fetch && git reset --hard` alternative | noted | opening-a-pr drop list |
| 4f | opening-a-pr's screenshot and no-boilerplate clauses | fixed | `opening-a-pr.md:28-29` restores both. The notes stop claiming `bee-technical-writing` holds them |
| 5 | "Self-contained PRs branch straight off main kept" listed under Dropped | fixed | that note now reads "is kept, not dropped" |
| — | table-only: "rearm with the corrected snapshot; the follow-up merges last" dropped with no note | noted | babysit drop list |
| — | table-only: autopilot-stack self-proof wording drift | noted | autopilot-stack swaps |

## New findings from the fix

A. **`multi-phase-plan` still has owners rebase (consider).** The plan
skeleton has the checklist line "Rebase onto current trunk before babysit and
again before the merge-ready report" (`feature/multi-phase-plan.md:111`) and
the line "Rebased onto current trunk after the verdict, patch-id unchanged"
(`:180`). The skeleton spawns one owner per PR and executes through the
autopilots. Under `b25486e3` and `orchestrate` § Merge safety, those owners
never rebase. ppb-12c only checked the session files, so this is not a ppb-25
miss, but it now contradicts the playbooks the plan names as its execution
path. The fix is to send drift to the merger as a cell in both lines, the same
way the autopilots now do.

B. **The babysit "Restored" paragraph splits the drop list (cosmetic).** In
`port-notes/session-ppb-12.md`, the paragraph "Restored (`b25486e3`): never a
job retry…" sits between two drop bullets. The two bullets after it (the
four-column table in the Reply, and the merge-when-ready warning) are still
drops, but they now read as part of the restored entry. The fix is to move the
paragraph below the last bullet.

C. **Unwrapped lines (cosmetic).** The edits left lines past 80 columns in
`babysit.md` (lines 46, 75, 87, 90, 110 and 126), `opening-a-pr.md` (28 and
29) and `autopilot-full.md` (34). The rest of each file wraps at 80. This does
not change what the text says.

## Fences

- pstack-only tool names: `rg -n -i 'cursor|graphite|bugbot|poteto-agent'
  skills/bee-planning/playbooks/` prints nothing.
- Decision `b25486e3` exists in the store, and its four clauses match the
  commit: CLEAN, owner rebase, CI retry and the floor lines.
- The cell's verify command prints a count of 1 for each of the six merged
  situation files (output in the proof line of the result).
