# ppb-12c — session port (second half) checked against its sources

Scope: the five session playbooks ppb-12 wrote (commit `051eda386`):
`babysit`, `shipping`, `autopilot-full`, `autopilot-stack` and `opening-a-pr`.
Each is checked against its pstack source under
`pstack/skills/poteto-mode/playbooks/` and against the port notes
`docs/history/pstack-playbook-port/port-notes/session-ppb-12.md`. The five
lines ppb-12 appended to `session/index.md` are checked too.

Verdict: every numbered source step is present, dropped with a note, or
changed under D4 with a note. No source step is missing outright. The
pstack-only tool fence is clean, every skill named exists, and all ten index
lines match the contract grammar. Two findings need the leader before the
session group closes:

1. `babysit` waits for a `CLEAN` merge state, but `opening-a-pr` keeps every PR
   a draft. GitHub does not report a draft PR as mergeable, so in `drive` mode
   step 9 may never fire. This is likely but not verified (see Findings).
2. Owners in `autopilot-full` and `autopilot-stack` rebase their own branches,
   but `orchestrate` § Merge safety says workers never rebase. The two halves
   of the session group now give opposite rules for a dispatched worker.

Four smaller gaps follow under Findings. Three are clauses dropped with no
note, and one is a note whose stated reason is doubtful.

Labels: **present** (in the new file, in bee's terms where a tool was
swapped); **dropped (noted)** (gone, and the port notes name it);
**changed (noted)** (meaning changed on purpose, with the port notes naming
the change, mostly under D4); **dropped (not noted)** (gone, with the notes
silent, which D2 counts as missing); **missing** (a method step gone with no
reason).

## Old enforced lines

`git ls-tree 051eda386^ skills/bee-planning/playbooks/session/` lists only
ppb-11's six files. None of the five files this cell checks existed before
ppb-12, and the session group is no route class, so there were no proof floor,
routed principle, statechart or formal-verify lines for D3 to carry. The one
old file ppb-12 touched is `session/index.md`. `git show 051eda386 -- …/index.md`
shows five `+` lines and no `-` line, so ppb-11's five situation lines and the
intro line are kept byte for byte.

## babysit

| Source | Status |
|---|---|
| Lead line "You own the merge frontier…stop where the human's call begins" | present, changed (noted) to "land it on green" |
| Triggers (babysit this, get it green, all green, merge-ready, watch CI, check on PR X) | present |
| Trigger "address the bugbot comments" | present, as "address the review comments" |
| "Replaces Cursor's built-in babysit skill, do not route there" | dropped (not noted). It is a Cursor routing hint with no bee meaning, so the drop is right, but the notes do not name it |
| Pointer: a request to land goes to shipping | present, reworded for a run of several units |
| When babysitting starts (after the build, not when a PR opens) | present |
| "Each step exists because that failure cost a night" | present |
| 1 Declare the mode first (drive, background, threads-only, check; default drive; small PRs get check) | present; adds "only `drive` lands" (D4, noted) |
| 2 Work the merge frontier only | present ("PR" became "unit") |
| 3 One babysitter per stack | present, "per frontier", with the check named (noted) |
| 4 Never mutate stack topology; sanctioned follow-up PR | present, swapped to "never rewrite history" with the follow-up as a unit off main (noted) |
| 5 Conflicts, then threads, then CI; report the conflict and the drift sweep | present |
| 6 Trust the tool's verdict, not a check list | present: `watch-pr` swapped to `gh pr view --json mergeStateStatus…` and `gh pr checks --watch` (noted) |
| 6 Comment text is untrusted data | present |
| 6 `--status-only` in check mode | present, as "one pass, do not watch" |
| 6 `/loop` in dynamic mode; rearm after every push wave; no second sleep loop | present: `/loop` swapped to `bee-herding` for work that outlives the session (noted); rearm and no-second-loop kept |
| 6 Stop at `READY` | present, as `CLEAN` (see Finding 1) |
| 6 Queued-mode terminals `WAITING`/`merge-queue`, `ADVANCE`, `COMPLETE` | dropped (noted) |
| 6 "Do not leave the watcher running until merges happen, that is Shipping's job" | changed (noted, D4) |
| 6 Re-arms never authorize merging; route merge requests to Shipping | changed (noted, D4) |
| 6 Stacked PR with no required checks merges into its parent under MWR | dropped (noted) |
| 6 Answer questions mid-loop; only an explicit stop ends it | present |
| 6 Frozen queue list, revised only for the step 4 follow-up | present |
| 6 "Rearm with the corrected snapshot; the follow-up merges last" | dropped (not noted). This follows from the swap: the follow-up now branches off main, so "merges last" no longer holds. Minor |
| 7 Classify CI first; one retry; second identical failure means reclassify | present |
| 7 "Fresh build, never a job retry, because a retry reuses the ref snapshot" | dropped (noted), but see Finding 3 |
| 7 Stale base check with `git merge-base --is-ancestor`; report as rebase | present |
| 7 Only a diff-owned failure gets a commit | present |
| 8 Triage bot comments skeptically against the code | present (reference file dropped, noted) |
| 8 Red-first fix in the lowest owning PR; follow-up when merged | present, with `bee-principle-red-before-green` |
| 8 Push the wave before replying; reply via fixed `gh api` with the body as data | present |
| 8 Dismiss noise with a disproof; lean to dismiss from the third pass, except security/auth/billing/data/migrations | present (pass count from thread history, noted) |
| 8 Never churn code to quiet a bot | present |
| 9 Stop at the human's line; owner approval is a wait | present in step 9 |
| 9 Babysitting never authorizes merging | changed (noted, D4): `drive` runs `bee worktree merge` on green and stops at the uat door |
| 9 Surface the escalation and keep working the rest | present |
| 9 Sweep triage decisions once; offer dismissal patterns to the shared rubric | present as step 10, through `bee-capturing` (noted) |
| Closing paragraph "drive ends at merge-ready; landing is Shipping" | changed (noted, D4) |
| Reply | present; four-column table dropped (noted); "what landed" added |

## shipping

| Source | Status |
|---|---|
| Lead line | present, with "stop at the uat door" added (D4, noted) |
| Triggers (land the stack, ship it, second half after Babysit) | present |
| Trigger "enable merge when ready" | dropped (not noted). A merge-when-ready trigger has no bee meaning, but the notes are silent |
| "Babysit makes it mergeable, Shipping decides what is safe; green is not safe" | present ("lets Graphite drain it" became "lands it") |
| 1 Verify every PR independently; one agent per PR, not batched; real surface parent vs head; PASS/PASS+NOTES/FAIL posted on its PR; non-author verdict; CI and bot approval are not verdicts | present: cloud agent swapped to `bee dispatch prepare --kind reviewer`, control skill swapped to `verify-app` or the real interface; a decision-trail row added (noted) |
| 2 Contiguous verified run from the bottom; report the ceiling | present |
| 3 Patch-id re-check after a rewrite; the twenty-one stale verdicts | present ("restack" became "rebase") |
| 4 Arm MWR through Graphite with `--always` | changed (noted): `bee worktree merge --id <unit> --cleanup`, one merger, stop at the first refusal |
| 5 Never enable GitHub auto-merge; disarm with `gh pr merge <n> --disable-auto` | present, with a new reason (noted) |
| 6 Do not read `autoMergeRequest` as proof | changed (noted): confirm each landing from git |
| 6 "If you cannot confirm, say so rather than infer" | present |
| 7 Once draining, stop touching the stack; no `gt` from an unchecked worktree | changed (noted): "while a merge runs, touch nothing else" |
| 7 "Independent work gets re-parented onto trunk and shipped on its own" | changed (not noted): now "waits for its own pass through step 1". The meaning is close but the notes do not list it |
| 8 Watch the drain, do not drive it; ADVANCE is progress; report each merge; diagnose a stall before mutating | changed (noted): the root drives merges one at a time; "progress, not termination" and diagnose-before-mutating kept |
| 8 `graphite-base/*` refs and retargeting | dropped (noted) |
| 9 Stop at the ceiling; report what landed and the next gap; extending is a new step 1 | present, with the uat door added (D4) |
| Reply | present ("armed" became "merged") |

## autopilot-full

| Source | Status |
|---|---|
| Lead line and the contrast with Orchestrate | present ("to merge" became "to merge-ready", since the root merges, noted) |
| Triggers | present |
| 1 Operator's items stay theirs; state-then-wait; start only on go | present |
| 1 Arm a `/goal` with the program objective | changed (noted): a checkable exit condition driven as `autonomous-run` |
| 2 One owner per PR with the full lifecycle | present: `bee dispatch prepare --kind cell` (noted) |
| 2 lifecycle: build | present |
| 2 lifecycle: gt registration | dropped (noted) |
| 2 lifecycle: self-proof on the real artifact (prove-it-works) | present, pointer dropped (noted) |
| 2 lifecycle: skeptical Bugbot triage per the reference | present, "per `babysit`" (reference dropped, noted) |
| 2 lifecycle: deslop | dropped (noted) |
| 2 lifecycle: no-comments | present as a comment strip, pointer dropped (noted) |
| 2 lifecycle: restack onto trunk, before babysit, never waits | present as a rebase onto main (noted); see Finding 2 |
| 2 lifecycle: babysit to green | present |
| 2 lifecycle: the merge itself, gated by step 4 | changed (noted): owners never merge, the root does |
| 2 decisions.tsv trail, returned with reports | present as a decision trail under `docs/history/<feature>/` (noted) |
| 3 True parallel; one writer per branch; disjoint files; drift absorbed by rebase; only overlapping work serializes | present, with `bee-principle-separate-before-serializing-shared-state` |
| 3 Self-contained branches off main; merge-then-branch | present (the notes list this under "Dropped" by mistake; it is kept) |
| 3 Every PR gt-registered; the Graphite-metadata rule | dropped (noted) |
| 3 Short private stack exception | present, as a private chain |
| 4 Swarm-verify at the merge-ready SHA, aggregate to one verdict | present: reviewers via `bee dispatch prepare --kind reviewer`; swarm pointer dropped (noted) |
| 4 Lanes: re-run gates; live proof (control skill); receipts-and-diff audit | present (control skill swapped, noted) |
| 4 Live lane is the floor; no merge without a clean verdict; fix-forward and re-verify | present |
| 5 Owner merges only from a trunk-current head; the verdict pins that SHA | present, `git merge-base --is-ancestor main <head>` (noted) |
| 5 Patch-id rule from shipping | present |
| 5 Squash-merge own PR | changed (noted): the root runs `bee worktree merge`. "Squash" is gone, not named; bee's merge verb picks the merge shape |
| 5 Owner takes the next item; full autonomy plus a clean verdict authorizes the merge; operator items stop at merge-ready | present |
| 6 Countersign a new raise after verifier proof; absorbed drift is not a raise | present |
| 6 Audit tick about every 30 minutes on `/loop`, cloud sleeper chain; never left to memory | changed (noted): a fixed-interval tick under `bee-herding`, interval number dropped |
| 6 Re-read the playbook from trunk, then the `/goal`; audit against both; fix drift as urgent | present (path and exit condition swapped, noted) |
| 6 Probe each owner with a liveness check | changed (noted): a read-only probe, no resume |
| 6 "and collect the decision trails" at each tick | dropped (not noted). Step 2 has owners return trails with their report and the Reply asks where they live, but the per-tick collection is gone |
| 6 Only side effects count; stuck lanes replaced at once | present ("store reports" became "reports") |
| 6 Retro pass and post-merge bot-comment sweep when merges batch | present |
| 7 Instant stand-down on the operator's stop; zero writes; hold briefs | present |
| Reply | present |

## autopilot-stack

| Source | Status |
|---|---|
| Lead line "never the landing…she reviews and lands herself" | changed (noted, D4): the root lands the verified chain, then stops at the uat door |
| Triggers; sibling of Autopilot-full; same owner loop and gate, different terminal | present |
| 1 Owner loop: one agent per PR; build; gt registration; self-proof; Bugbot triage; deslop; no-comments; babysit to green; parallel when self-contained; decisions.tsv | present with the same swaps and drops as autopilot-full (noted). One wording drift: pstack's self-proof here is "gates, CI, receipts", while the port says "on the real artifact" (not noted, minor) |
| 2 Audit tick about every 30 minutes on `/loop`; re-read playbook and `/goal`; liveness probe; side effects only; replace stuck lanes | present, swapped as in autopilot-full (noted) |
| 3 State-then-wait; arm `/goal` on go; zero-writes hold on stop | present (the `/goal` swap is noted) |
| 4 Verify at STACK-READY with a swarm; gates, live floor, receipts audit; one verdict; nothing enters unverified | present (swarm pointer dropped, noted) |
| 5 Append on a clean verdict; no owner merges, arms auto-merge or closes | present |
| 6 Single writer on topology; owner pushes only its branch with `--force-with-lease` after `ls-remote`; reports tip and parent | present |
| 6 Root registers appends with `gt track` / `gt submit --stack`; the cloud agent rule about the submit walk; setting a bottom base directly | changed or dropped (noted) |
| 7 Root absorbs drift by restacking; owner fixes conflicts in its slice and the root pushes | changed (noted): each owner rebases its own branch (see Finding 2) |
| 7 Patch-id re-check; drifted links go back through step 4 | present |
| 7 Countersign rule unchanged from Autopilot-full | present |
| 8 Deliver one linear chain, each link carrying its verdict; operator reviews and lands it | changed (noted, D4): new step 8 lands bottom first with `bee worktree merge`; new step 9 delivers the chain for review at the uat door. `uat_stop = "merge"` keeps pstack's review-first order (stated, cited to `bee-swarming`, which documents it) |
| "Reviewable bottom-up in the Graphite UI" | dropped (noted) |
| Choosing between the autopilots | changed (noted) |
| Reply | present ("stack root and tip" became "first and last unit") |

## opening-a-pr

| Source | Status |
|---|---|
| "Invoked at the end of every other playbook" | present |
| Worktree: work off main; subagents inherit it | present (step 1) |
| Worktree: parallel `Task` calls each get their own worktree | present, as "two workers never share a branch" |
| Worktree: or `git fetch && git reset --hard origin/<branch>` between them | dropped (not noted). bee's one-worktree-per-worker rule makes the reset alternative unneeded, but the notes are silent |
| Worktree: dirty branch, patch out and reapply; snarled, reset and redo | present |
| Commits: commit liberally; rebase into small ordered commits; each a future PR; amend vs new commit | present, with the cap's `cell:` trailer and the after-cap rule (noted) |
| PRs: deslop over the diff | dropped (noted) |
| PRs: no-comments before review | present as "write no comment in the code", pointer dropped (noted) |
| PRs: technical-writing then unslop for title, description and commit body; all layers except Diátaxis | present: `bee-technical-writing` then `bee-unslop` (noted). "Except Diátaxis" lives in that skill (SKILL.md line 111) |
| PRs: one word per action, keep articles, avoid `-ing` | present |
| Titles: Conventional Commits `type(scope): subject`, imperative, real symbol, no period | moved (noted) to `bee-technical-writing`, which holds it (SKILL.md line 112) |
| Descriptions: Why, Scope, Tradeoffs, Blast Radius, Verification | moved (noted) to `bee-technical-writing` (SKILL.md line 113, which adds "What changed") |
| "Attach videos or screenshots when they prove a claim"; "no `## Summary` or `## Test plan` boilerplate" | dropped (not noted correctly). The notes say these live in `bee-technical-writing`, but `rg -i 'boilerplate\|test plan\|screenshot\|video'` over that skill finds nothing. The one-home argument does not cover these two clauses |
| Descriptions: a commit body does not restate its subject | present |
| Size and stacks: prefer five narrow PRs; keep the order visible; branch off main only for independent work; rebase on main first | present (step 4; Graphite stacking swapped, noted) |
| Readiness: open every PR ready, never draft | changed (noted): bee's `ship_visibility` rule keeps it a draft (see Finding 1) |
| Readiness: cloud-agent tools default to draft, set `draft: false`, `gh pr ready` | dropped (noted) |
| Readiness: `gh pr view` before citing status | present |
| Babysit paragraph (opening does not start a babysit; push back on drifting feedback) | present (step 7) |
| Subagent that opens a PR runs interrogate, deslop, no-comments; returns the URL; does not babysit | present (step 8); interrogate and deslop dropped (noted) |

Steps 6 (land through the bee path) and the Proof line are bee additions
the notes name.

## Fence: pstack-only tool names

`rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/`
prints nothing and exits 1. A wider sweep of the five files for `gt `, `/loop`,
`/goal`, `deslop`, `no-comments`, `watch-pr`, `swarm`, `interrogate`,
`show-me-your-work`, `prove-it-works`, `control-ui`, `control-cli`, `arena`,
`architect`, `figure-it-out` and `tdd` finds one hit. It is
`autopilot-stack.md:69`, which names `bee-swarming`, a real bee skill and not
pstack's `swarm`.

## Skills named

Every backticked skill in the five files:

| Name | Exists |
|---|---|
| `bee-capturing` | `skills/bee-capturing` |
| `bee-herding` | `skills/bee-herding` |
| `bee-principle-red-before-green` | `skills/bee-principle-red-before-green` |
| `bee-principle-separate-before-serializing-shared-state` | `skills/bee-principle-separate-before-serializing-shared-state` |
| `bee-swarming` | `skills/bee-swarming` |
| `bee-technical-writing` | `skills/bee-technical-writing` |
| `bee-unslop` | `skills/bee-unslop` |
| `verify-app` | not under `skills/`; it lives in `.claude/skills/verify-app`. Every use reads "the project's `verify-app` skill, when it has one, otherwise the real interface", which is the swap the cell action prescribes, so it names a host's skill on a condition and is not a broken step |

The other backticked names (`babysit`, `shipping`, `autonomous-run`,
`orchestrate`, `autopilot-full`) are situations, and each has a file in
`session/`.

bee verbs and values the files cite, checked in source because a worker may
not run them: `bee dispatch prepare --kind cell|reviewer` (`DISPATCH_KINDS`,
`verbs/drivers/prepare.rs:152`); `bee worktree merge --id … --cleanup`
(`parse_merge_flags`, `verbs/worktree/handlers.rs:1182`); `MERGE_CONFLICT`
(`verbs/worktree/phases.rs:718`); `bee herding control-loop`
(`verbs/herding.rs:153`); `bee feature finish` (`verbs/feature_group.rs:29`);
the `uat_stop = "merge"` path (`skills/bee-swarming/SKILL.md:145`); and the
"draft stays draft, a window" rule
(`skills/bee-hive/references/scout-and-ticks.md:157`). All exist.

## Index grammar

The contract (913115b3): a `## Situations` heading, then one line per
situation of the form `` - `<name>` — <triggers> — delivers: <deliverable> ``,
with `<name>` matching `[a-z0-9-]+`, ending at the next `## `. Every non-blank
line in the section was tested against
`^- \`[a-z0-9-]+\` — .+ — delivers: .+$`:

| Line | Grammar | File |
|---|---|---|
| `session-pickup` | match | exists |
| `pause-safely` | match | exists |
| `autonomous-run` | match | exists |
| `orchestrate` | match | exists |
| `worktree-cleanup` | match | exists |
| `babysit` | match | exists |
| `shipping` | match | exists |
| `autopilot-full` | match | exists |
| `autopilot-stack` | match | exists |
| `opening-a-pr` | match | exists |

The reverse also holds: `session/` holds exactly these ten files plus
`index.md`. Every new line carries English and Vietnamese triggers.

## Findings

1. **`babysit` lands on `CLEAN`, but bee keeps PRs as drafts (consider; likely,
   unverified).** `babysit.md:73` stops at `CLEAN` and step 9 merges only "with
   the frontier `CLEAN`". `opening-a-pr.md:39` and the `ship_visibility` rule
   keep the PR a draft until the owner says otherwise. GitHub's
   `MergeStateStatus` enum, read live with
   `gh api graphql … __type(name:"MergeStateStatus")`, describes `CLEAN` as
   "Mergeable and passing commit status", and GitHub does not merge a draft.
   I expect a draft to read `BLOCKED` or `UNKNOWN`, never `CLEAN`, so a
   `drive` babysit over a bee draft PR would wait forever. I could not prove
   it: the only open PR in this repo (#63) is not a draft. A fix would gate
   step 9 on the checks rollup and review decision, plus `isDraft`, instead of
   the bare merge state. Or `bee worktree merge` could be named as the
   landing, with no GitHub merge state needed.
2. **Owner rebase in the autopilots against "workers never rebase" (consider).**
   `autopilot-full.md:26` has each owner rebase onto main, and
   `autopilot-stack.md:55` has each owner rebase its own branch to absorb
   drift. `orchestrate.md:196` (Merge safety, ppb-11) says "Workers never
   rebase and never merge. A worktree's conflict goes to the merger as a
   cell". The ppb-12 notes cite that same section to justify "owners never
   merge" but keep the pstack rebase. Both rules are bee's own, so one of them
   should be chosen and stated for both playbooks.
3. **The CI-retry note's reason is doubtful (consider).** The notes drop "fresh
   build, never a job retry, because a retry reuses the original ref snapshot"
   as specific to the pstack CI vendor. The port then says "one fresh run of
   the failed jobs" (`babysit.md:82`). That is a job retry, and a GitHub
   Actions re-run also runs against the original commit and merge ref. So the
   pstack reason likely still applies, and the port inverts the rule. This is
   unverified against a live re-run.
4. **Clauses dropped with no note (minor):** babysit's Cursor routing sentence;
   shipping's "enable merge when ready" trigger and its re-parent clause;
   autopilot-full step 6 "collect the decision trails"; opening-a-pr's
   `git fetch && git reset --hard` alternative; and opening-a-pr's screenshot
   and no-boilerplate clauses, which the notes say `bee-technical-writing`
   holds but it does not.
5. **Port-notes slip (cosmetic):** `session-ppb-12.md:173` lists "Self-contained
   PRs branch straight off main kept" under "Dropped".

Suspicions checked and not filed: `verify-app` outside `skills/` is a
prescribed conditional, not a broken step. `bee-swarming` at
`autopilot-stack.md:69` is a real skill and not the pstack `swarm`. The
`uat_stop = "merge"` claim matches `bee-swarming/SKILL.md:145`. `--id <unit>`
takes a worktree id, which matches the files' own definition of a unit as one
worktree branch.
