---
artifact_contract: bee-research/v1
topic: pstack-guide-distill
depth: standard
date: 2026-11-17
---

## Bottom Line

- Recommendation (ladder rung): reuse for most chapters, adapt-upstream for five gaps.
- Mode: `xia`. This brief reports and discusses. It builds nothing.
- Why this is the lightest credible path: earlier ports already carry most of the guide into bee. The 22 playbooks, the eval observer-effect rule, the principle index, sharpen-the-ask, `--fix-at`, the decision log, pause and pickup, autopilot, babysit and shipping all exist. The open work is five narrow gaps, not a new port.
- Why the next-best rung lost: a fresh full port would repeat `pstack-playbook-port`, `pstack-principles` and `pstack-0-15-36-port`.
- Confidence: 80%. Gaps G4 and G5 are Inference: no history search for a recorded rejection was run.
- Suggested next step: bee-shaping on G1 to G3 as one feature. G4 and G5 go to the backlog as proposed PBIs.

## Source manifest

| Field | Value |
|---|---|
| Repo | `cursor/plugins` |
| Ref | `main` |
| Resolved commit SHA | `e1cc03121a0e71129a4fb730d7c5528a0a3e43c8` |
| Narrowed scope | `pstack/docs/guide/` README and chapters 01 to 10 |

## Chapter matrix

| Ch | Teaches | beehive today | Status |
|---|---|---|---|
| 01 Setup | per-role models, verify-skill and formal-skill offers at setup | `bee team show` and `team.claude` roles. The verify offer is `bee-verifying`. No formal offer. | EXISTS, except the formal offer (G1) |
| 02 Router | goal plus done check, "new task" re-match, a worktree per task, no skill lists in the prompt | `bee orient` classes, situations and playbooks; `bee worktree new`; shaping playback | EXISTS (Local) |
| 03 Understand | `/how` `/why` `/teach` `/recall`, investigation, session pickup | `bee-how`, `bee-why`, `bee-teach`, `bee-recall`, `research/investigation.md`, `session/session-pickup.md` | EXISTS (Local) |
| 04 Design | architect, arena, swarm, interrogate, prototype, multi-phase plan, README first | `bee-architect`, `bee-arena`, `bee-fanout`, `bee-interrogate`, `spike/prototype.md`, `feature/multi-phase-plan.md`. README first has no home. | EXISTS, except README first (G4) |
| 05 Build and clean | per-class prompts, tdd, unslop, no-comments, forensics | `bee-tdd`, `bee-unslop`, `bee-comment-review`, `research/runtime-forensics.md`, `research/trace-forensics.md`, `perf/hillclimb.md` | EXISTS (Local) |
| 06 Verify and ship | prove it works, benchmark-checklist, verify skill and its upkeep, formal verify skill and its upkeep, statechart, PR, babysit, shipping | `bee-verifying`, `bee-verify-upkeep`, `verify-app`, `bee-blast-radius`, `session/opening-a-pr.md`, `babysit.md`, `shipping.md`. `skills/bee-planning/playbooks/feature/feature.md:78` names `create-formal-verification-skill`, which bee does not ship. Perf has only a noise floor (`perf/perf-issue.md:9`) and a median (`hillclimb.md:30`). | NEW: G1, G2, G3 |
| 07 Overnight | trust before the loop, overnight contract, decision log, autopilot, automations | `session/autonomous-run.md:5` (checkable predicate), `bee-show-me-your-work`, `autopilot-full.md`, `autopilot-stack.md`, `bee-herding`. The four trust preconditions have no gate. | EXISTS, except G5 |
| 08 Principles | 24 names that steer | 22 `bee-principle-*` skills plus about 15 of bee's own. Explain the Number is a perf playbook rule only. | G2 |
| 09 Make it yours | automate-me, reflect, correct, skill authoring, blind eval | `bee-reflect`, `bee mailbox reflect --fix-at` (`packages/bee/LEADER.md:367`), `bee-writing-skills`, `docs/authoring-a-skill.md`, `research/eval.md`. `automate-me` was skipped on purpose (`docs/history/pstack-playbook-port/CONTEXT.md:97`). | EXISTS |
| 10 Recipes | one-line steer prompts, pitfalls | spread across the playbooks; no single recipe page for the user | covered by G4 |

## Gaps and proposals

- **G1. The formal-verification and statechart doors are dead on hosts.** `feature.md:78` and `bugfix/bug-fix.md:60` route to `create-formal-verification-skill`, `maintain-formal-verification-skill` and `statechart`. bee ships none of them, so every host takes the "skip this step and say so" branch. beehive has one model built by hand: `formal-cell-lifecycle`. Proposal: ship `bee-formal-verifying` (create) and `bee-formal-upkeep` (maintain), modelled on the `bee-verifying` and `bee-verify-upkeep` pair. Each output lives under `.bee/verify/<name>/` with its covered files and sync commit. It must prove the property fails on a broken variant. `formal-cell-lifecycle` is the seed and the acceptance example. Statechart becomes a reference inside it, not a separate skill. (Local, Upstream)
- **G2. Explain the Number has no principle and no checklist.** Proposal: add `bee-principle-explain-the-number` with an index row covering every class. Add the seven benchmark questions (limiter, tuned like production, physical limit, errors, alternating runs with median and range, end to end, work inside the timed region) as a reference that `perf-issue.md` and `hillclimb.md` cite. The verdict is one of: faster, slower, no difference, inconclusive. This also covers test counts and token numbers. (Local, Upstream)
- **G3. Cut the principle list per task.** From the earlier session: `bee orient --class refactor` prints 29 principles. Proposal: show 5 to 7 ranked by lane, risk flags and touched files, with the rest as one name-only line. Check `principles_used` at cap. (Local)
- **G4. README first, and a recipe page for the human.** Add one step to `feature/feature.md`: for a package or API other code calls, write the caller's tutorial first and check the result against it. Add a short `docs/guide/recipes.md` of one-line steer prompts mapped to bee situations and principles, so the user can steer by name. (Inference: no recorded rejection was found by the probe.)
- **G5. A trust gate before an unattended loop.** chapter 07 lists four preconditions: done once by hand, the agent has the tools, every stage can stop the line, repeated failures already turned into checks. `autonomous-run.md` checks the predicate only. Proposal: add the four as a checklist in `autonomous-run.md` and in the herding bootstrap. (Inference)

## Deliberately not adopted (recorded earlier)

`automate-me`, `bro`, `setup-pstack`, `typescript-best-practices`, `make-bot-ui` (`docs/history/pstack-playbook-port/CONTEXT.md:97`). `ketchup` and `poteto-help` overlap with `bee-recall`, `bee status` and `bee-hive`. No new evidence argues for reopening them.

## Source Pack

- Local: `skills/bee-planning/playbooks/**`, `skills/bee-verify-upkeep/SKILL.md`, `skills/bee-shaping/references/shaping-reference.md:8`, `packages/bee/LEADER.md:367`, `docs/history/pstack-playbook-port/CONTEXT.md:97`, `docs/history/research/pstack-distill.md:79`, output of `bee orient --class refactor`.
- Upstream: `cursor/plugins@e1cc031` `pstack/docs/guide/*.md`. The sparse clone is at `.bee/tmp/pstack-guide/src`.
