# ppb-9c — feature port checked against its sources

Scope: what ppb-9 wrote in commit `3ba27a8d1`. That is the class index
`skills/bee-planning/playbooks/feature.md`, the situation files
`feature/feature.md` and `feature/multi-phase-plan.md`, and the port notes
`docs/history/pstack-playbook-port/port-notes/feature.md`. The files are
checked against the pstack sources `pstack/skills/poteto-mode/playbooks/feature.md`
and `playbooks/multi-phase-plan.md`, and against the old bee `feature.md` from
`git show 3ba27a8d1^:skills/bee-planning/playbooks/feature.md`.

Verdict: no method step is missing. Every old enforced line is present, and
the Proof floor line in the index is byte-identical to the old one. Two
source clauses are gone with no port-notes entry, and both are small. The
pstack-only tool fence is clean. Every skill the files name exists in this
repo. The index lines match the contract grammar. There are four findings,
and one of them (finding 1) is worth fixing before slice 3. None of them
blocks the cap.

Labels: **present** means the step is in the new file, in bee's terms where a
tool was swapped. **dropped (noted)** means it is gone and the port notes name
it. **dropped (not noted)** means it is gone and the notes say nothing, which
D2 counts as missing. **missing** means a method step is gone with no reason.

## Old enforced lines (old `feature.md`)

| Old line | New home | Status |
|---|---|---|
| 1 Name the DATA shape first, before any verb, flag or screen reads it | `feature/feature.md` step 2, verbatim | present |
| 2 Walking skeleton end to end, no stub on that path, with its reason | step 5, first bullet, verbatim reason | present |
| 3 Build the CURRENT slice only; headline the rest, leave it out of the cells | step 5, second bullet | present |
| 4 Prove the user-visible path, not only the unit | step 6, verbatim reason | present |
| 5 Sync `docs/knowledge/` in the SAME change; state layer, not a write-up | step 7 | present |
| 6 `statechart` model of an event-order flow, before the flow's code, break is a guess until a real test fails, skip-and-say-so | step 3, every clause kept, reason added | present (moved ahead of the code, noted) |
| 7 `create-formal-verification-skill`, `.bee/verify/<name>/`, `maintain-formal-verification-skill`, skip-and-say-so | step 8, verbatim | present |
| "A green unit behind a path nobody can walk is not a result." | line 91, verbatim | present |
| Proof line: the user-visible path driven, with its evidence (`green:live`) | line 97, verbatim | present |
| Proof floor line | index `feature.md` line 10 | present, byte-identical (string equality of the old and new `^Proof floor` lines); absent from `feature/feature.md`, see finding 1 |

## pstack `feature.md`

| Source step | New home | Status |
|---|---|---|
| Intro: you own the design; plan, review, verify; delegate, stay in the lead | intro, with `bee dispatch prepare` and a reason | present (swapped, noted) |
| 1 `how` over the affected subsystem | step 1, `bee-how` as a gather | present (swapped, noted) |
| 2 `architect` for parallel design exploration, `architect skipped: <reason>` | gone | dropped (noted; slice 3) |
| 3 Throughput checkpoint as four todo items, `n/a: <reason>` | step 4, verbatim, reason added | present |
| 3 Blocking first steps | step 4 bullet | present |
| 3 Independent workstreams | step 4 bullet | present |
| 3 Shared mutable state, split the target per the principle skill | step 4 bullet, `bee-principle-separate-before-serializing-shared-state` | present (swapped, noted) |
| 3 Smallest safe decomposition | step 4 bullet | present |
| 4 Delegate code-writing to a subagent on the configured feature model | step 5, a cell on the code role's model | present (swapped, noted) |
| 4 Specific scope: file paths, named data shape and organizing structure (state machine, table, typed model) chosen first, success criteria | step 5 scope plus step 2 structure list | present |
| 4 `principle-model-the-domain` pointer | rule kept in step 2, name gone | dropped (noted; slice 3) |
| 4 Review its diff yourself | step 5 | present |
| 4 `arena` when the implementation admits several shapes | gone | dropped (noted; slice 3) |
| 4 Mandatory, no skip-with-reason escape, Laziness Protocol does not override, gain is review separation | step 5 third bullet, Laziness Protocol name gone | present; the name dropped (noted) |
| 4 "You can spawn a subagent even though you are one"; "the app is small" and "a subagent cannot spawn one" are both wrong | step 5: "A lead that cannot dispatch owns the diff directly" | present (swapped, noted); the rebuttal of "the app is small" is gone, see finding 3 |
| 4 Forbidden-to-spawn owns the diff with the same review separation; no "standing by" reply | step 5 third bullet | present |
| 4 Comments per **Comments** | step 5 fourth bullet, the bee no-comment rule | present (swapped, noted) |
| 4 Surgical edits; re-ground against the source for upstream-derived files | step 5 fifth bullet, reason added | present |
| 4 Port shared-primitive improvements to all consumers, verify each | step 5 fifth bullet | present |
| 4 Commit liberally | step 5 fifth bullet | present |
| 5 Verify on the matching surface; inconclusive or wrong surface is not a pass; flag it | step 6, through `verify-app` or the real interface | present (swapped, noted) |
| 6 Rebase into small, ordered commits | step 9: "Leave small, ordered commits" | present; the word "rebase" is gone, not noted, see finding 3 |
| 6 Stack follow-ups | step 9 | present |
| 6 `sequence-verifiable-units`: build, verify, commit each unit before the next | step 9 sentence | present (swapped, noted) |
| 7 `interrogate` before shipping a contested design | gone | dropped (noted; slice 3) |
| 8 Run **Opening a PR** | step 10, `playbooks/session/opening-a-pr.md`, `bee worktree merge`, `uat` door | present (swapped, noted) |
| Closing: code-coupled work to one owner with the checkpoint inline; owner fans out after the blocking phase | closing paragraph, verbatim | present |
| Closing: parent-level fan-out for independent artifacts | closing paragraph | present |
| Closing: rewrite the checkpoint at phase boundaries; fresh owner, no chained interrupts | closing paragraph, reason added | present |
| Reply: what you built, what you chose and why, open decisions; tables for alternatives | Reply paragraph | present |

## pstack `multi-phase-plan.md`

| Source step | New home | Status |
|---|---|---|
| Intro: own the plan not the code; checklist the operator audits; plan is the deliverable; do not implement | intro, verbatim | present |
| 1 Skip the plan for a one- or two-file obvious change; say so and stop | step 1, reason added | present |
| 2 Settle open questions by prototype, run the prototype playbook | step 2, `playbooks/spike/prototype.md` | present (swapped, noted) |
| 2 Keep branch, SHA, screenshots for Appendix A | step 2 | present |
| 2 Ask only about a product or preference call; give options (`never-block-on-the-human`) | step 2, sentence inline | present (swapped, noted) |
| 3 Explore in `poteto-agent` subagents with an explicit model (`guard-the-context-window`) | step 3, gathers via `bee dispatch prepare --kind gather` | present (swapped, noted); the role name is wrong, see finding 2 |
| 3 Each returns pointers, conventions, test commands, entry points; no inlined dumps | step 3 | present |
| 4 Copy the skeleton, fill every placeholder | step 5 | present |
| 4 Write under the agent store's `docs/` | step 5, `docs/history/<feature>/` | present (swapped, noted) |
| 4 Keep every heading and sub-block in order; one section per PR; one PR one change (`sequence-verifiable-units`) | step 5 | present (swapped, noted) |
| 4 Name the execution playbook; pick autopilot-full or autopilot-stack per the rule at the end of autopilot-stack; standing program takes orchestrate | step 5, `playbooks/session/...`; the rule exists at `session/autopilot-stack.md:77` | present (swapped, noted) |
| 5 `/technical-writing` then `/unslop`; Diátaxis how-to; the two verbatim rules; headings; no long dashes; no mid-sentence colons | step 6, `bee-technical-writing`, `bee-unslop` | present (swapped, noted) |
| 6 Run the `check-plan.mjs` script, fix every line | step 7, a by-hand shape check | dropped (noted), replaced |
| 6 `encode-lessons-in-structure` pointer | gone | dropped (noted); see finding 4 |
| 7 Hand back: plan path and the check output, then stop; execution on the operator's explicit go | step 8 | present |
| Verification: the verification rule sentence, every block opens with it, live block mandatory | Verification paragraph | present |
| Verification: `prove-it-works` pointer | the rule sentence itself | present (swapped, noted) |
| Verification: ten lanes on a named model via the `swarm` skill | ten lanes via `bee dispatch prepare --kind reviewer` | present (swapped, noted) |
| Verification: lane box, scenario, screenshot, predicate; perf block; review gate; the not-review-gated line | Verification paragraph, verbatim | present |
| Control skill by surface (`control-ui`, `control-cli`, simulator skill) | `verify-app` or the real interface | present (swapped, noted) |
| Control skill: two surfaces get lanes on both; no control skill is an Appendix C risk | Control skill paragraph | present |
| Skeleton: title, summary, How to read this | skeleton, execution path under `skills/bee-planning/playbooks/session/` | present (swapped, noted) |
| Skeleton Arm: state the protocol, wait for the go | present, pronoun swapped | present (swapped, noted) |
| Skeleton Arm: `/goal` | a checkable exit condition per `session/autonomous-run.md` | present (swapped, noted) |
| Skeleton Arm: read list from `origin/main` (playbook, swarm, control skill, opening-a-pr, leaf skills) | `git show main:` list; the swarm line is gone | present (swapped, noted); the swarm read dropped (noted with `swarm`) |
| Skeleton Arm: 30-minute tick on `/loop` or a cloud wake chain | `bee-herding` (`bee herding control-loop`) | present (swapped, noted) |
| Skeleton Arm: tick prompt verbatim | present, `/goal` became the exit condition | present |
| Skeleton Arm: zero-writes order on hold | present | present |
| Skeleton Spawn owners: one owner per PR, dependency graph, file boundaries, review gate | present, owner via `--kind cell` | present |
| Skeleton PR mechanics: open ready, never draft, `gh` or Graphite | `session/opening-a-pr.md`, draft per `ship_visibility` | present (swapped, noted; bee wins on draft) |
| Skeleton PR mechanics: lint and typecheck, hooks on | present | present |
| Skeleton PR mechanics: `/deslop` | gone | dropped (noted) |
| Skeleton PR mechanics: `/no-comments` | strip added comments before review | present (swapped, noted) |
| Skeleton PR mechanics: Bugbot triage per the triage reference | review comments per `session/babysit.md` | present (swapped, noted) |
| Skeleton PR mechanics: rebase before babysit and before merge-ready | present | present |
| Skeleton Verdict: swarm at the merge-ready SHA, gates, live, perf, audit lanes | lanes via `--kind reviewer`, same four lane kinds | present (swapped, noted) |
| Skeleton Verdict: clean only when every lane passes; new head, fresh verdict | present | present |
| Skeleton Verdict: merge rule with patch-id from `shipping.md` | `session/shipping.md` | present (swapped, noted) |
| Skeleton Boot recipe: own cloud VM, `control-ui`/`control-cli` | own worktree, `verify-app` | present (swapped, noted) |
| Skeleton Boot recipe: fetch and checkout, start, deliver input, save screenshots to `/tmp/swarm-...` | present, `/tmp/lanes-...` | present (swapped, noted) |
| Skeleton PR section: Depends on, Files, Build, You see, Verify unit | verbatim | present |
| Skeleton Verify live: ten lanes on a named model | ten lanes, model name gone | present (swapped, noted) |
| Skeleton Verify perf: metric, probe, baseline, rule | verbatim | present |
| Skeleton Review gate: screenshots; video on a lane VM; post and wait | video "when the host can capture one" | present (swapped, noted) |
| Skeleton Merge: root's clean verdict, Bugbot triage, rebase with patch-id, squash-merge or Graphite append | clean verdict, review comments, rebase, `bee worktree merge` or merge-ready with the `uat` door | present (swapped, noted) |
| Skeleton Close the program | verbatim | present |
| Skeleton Appendix A, B, C | verbatim | present |
| Skeleton Appendix D: `how`, `interrogate`, `show-me-your-work` reading list | a `bee-how` gather and the decision trail | present (swapped, noted) |
| Reply: plan path, PR ids, dependencies, review-gated set, prototypes, check output | Reply, "the result of the shape check" | present |

Added beyond pstack, all noted: step 4 (the phase-plan versus epic-map rule,
by pointer to `planning-reference.md:220`, the heading exists, and its three
clauses match the reference), a reason on each step, and a Proof line.

## Fences

- pstack-only tool names under `skills/bee-planning/playbooks/`: none.
  `rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/`
  exits 1 with no output.
- Skills named: `bee-how`, `bee-herding`, `bee-technical-writing`,
  `bee-unslop`, `bee-planning` and
  `bee-principle-separate-before-serializing-shared-state` exist in `skills/`.
  `statechart`, `create-formal-verification-skill`,
  `maintain-formal-verification-skill` and `verify-app` are not in `skills/`.
  They are project skills under `.claude/skills/`, and each is named with a
  "when the host has that skill" or "when the project has one" clause, the
  same form the old bee playbook and `spike/prototype.md` use. `architect`,
  `arena`, `interrogate`, `swarm` and the absent principles are not named.
- Playbook paths named: `session/opening-a-pr.md`, `session/autonomous-run.md`,
  `session/autopilot-full.md`, `session/autopilot-stack.md`,
  `session/orchestrate.md`, `session/babysit.md`, `session/shipping.md` and
  `spike/prototype.md` all exist.
- Bee verbs named: `bee dispatch prepare --kind cell|gather|reviewer` are in
  `DISPATCH_KINDS` (`packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:152`).
  `bee herding control-loop` and `ship_visibility` exist in the crate.
- Index grammar (contract `913115b3`): `feature.md` has one `## Situations`
  heading and two lines, `feature` and `multi-phase-plan`. Both match
  `^- \`[a-z0-9-]+\` — .+ — delivers: .+$`, and both carry English and
  Vietnamese triggers. Each has its file under `feature/`, and no file there
  lacks a line. The section runs to end of file and holds the Proof floor
  line, which `parse_line` skips (`situations.rs:36-37`).
  `cargo test --release --test class_playbook_parity`: 9 passed, 0 failed.

## Findings

1. `feature/feature.md` has no Proof floor line. D3 says bee's enforced lines,
   the proof floor named first, "stay in" the merged situation file. Every
   other merged situation file carries the line: `bugfix/bug-fix.md:67`,
   `spike/prototype.md:67`, `refactor/refactoring.md`, `perf/perf-issue.md`,
   `content/content.md`, `release/release.md` and `docs/docs.md`. The port
   notes say the line is "now in the index, byte-identical", and that is
   true. The cap still holds the floor, so nothing breaks. But a reader who
   opens the situation file, which is the file the plan check makes a plan
   cite, does not see it. Fix: add the line after the Proof line, as the
   siblings do. `multi-phase-plan.md` has no floor line either. It is not a
   D3 merge, so this is a consistency point only.
2. `multi-phase-plan.md` step 3 says gathers run "on the explore role's
   configured model". bee has no `explore` role. A gather runs on the `read`
   role (`prepare.rs:175`, `"gather" => Some("read")`), and the port notes
   themselves say "the read role's configured model". Fix: change "explore"
   to "read".
3. Two pstack clauses are gone with no port-notes entry. (a) Step 6 "Rebase
   into small, ordered commits" became "Leave small, ordered commits", so the
   rebase instruction is gone. (b) Step 4's rebuttal "'the app is small' ...
   is wrong" is gone, while its sibling excuse is kept as the swap. Neither is
   a method step, and the mandatory-delegation rule still holds without (b).
   D2 asks the notes to name each drop, so each needs a one-line note or a
   restore.
4. The port notes drop the `encode-lessons-in-structure` pointer together
   with the `check-plan.mjs` script. bee has the principle at
   `skills/bee-principle-encode-lessons-in-structure`. The script has no
   counterpart, but the pointer does, so step 7 could cite the bee principle.
   This is a candidate for the slice 3 re-add list. It is not a defect in
   this slice.

Also confirmed: the stale-pointer note is accurate. `bugfix/bug-fix.md:57-59`
cites "`feature.md` step 6" and "`feature.md` step 7". Those steps are now
`feature/feature.md` steps 3 and 8. The cell that owns bugfix should fix
this, so it is not edited here.

Outside this cell: the rendered tree `.claude-plugin/skills/bee-planning/playbooks/`
still holds the old class playbooks for every ported class, not only
`feature`, and it has none of the situation subdirectories. Plan claim 16
("the rendered playbooks are byte copies") held when the plan was written. It
no longer holds. This is presumably left for a render step later in the
plan. It is not a ppb-9 defect.
