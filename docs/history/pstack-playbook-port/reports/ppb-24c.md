# ppb-24c: check of the steps restored in ppb-24

Checked against commit `d28cf9a9e` (ppb-24). Playbook paths are under
`skills/bee-planning/playbooks/`. No file other than this report was edited.

## Verdict

33 of 34 restored steps are present and name a skill that exists in `skills/`.
One step is present in substance but does not name the skill by its bee name:
`session/worktree-cleanup.md` step 3 says "a herded worker spawns arena and
repro trees". It never writes `bee-arena`, and there is no `skills/arena/`.
restored-steps.md and session-ppb-11.md list the step as restored with
`bee-arena`. It is a one-word fix that belongs to ppb-24's files, not this cell.

All 10 indexes carry the fallback line, once each, after the Situations
heading. No source step that waited for one of the eight new skills is still
missing.

## Skills named

Each skill a restored step names has a `SKILL.md`: `bee-architect`,
`bee-arena`, `bee-interrogate`, `bee-fanout`, `bee-tdd`,
`bee-show-me-your-work`, `bee-figure-it-out`, `bee-comment-review`.

## Every source step, by status

"Present" means the situation file names the skill at the place
restored-steps.md gives. The line is where `grep -n` found the skill.

| # | Source step (pstack) | Situation file | Skill | Status | Evidence |
|---|----------------------|----------------|-------|--------|----------|
| 1 | `architect` before a fix that crosses a boundary | `bugfix/bug-fix.md` | `bee-architect` | present | lines 39, 44, step 4 |
| 2 | `interrogate` over the fix plan | `bugfix/bug-fix.md` | `bee-interrogate` | present | lines 39, 46, step 4 |
| 3 | `tdd` in the commit-order step | `bugfix/bug-fix.md` | `bee-tdd` | present | line 56, step 6 |
| 4 | `architect` shape handoff | `spike/prototype.md` | `bee-architect` | present | line 57, step 7 |
| 5 | `architect` before a boundary-crossing fix | `perf/perf-issue.md` | `bee-architect` | present | line 61, step 4 |
| 6 | show-me-your-work trail | `perf/hillclimb.md` | `bee-show-me-your-work` | present | line 34, step 3 |
| 7 | `architect` for the target shape | `refactor/refactoring.md` | `bee-architect` | present | line 33, step 4 |
| 8 | figure-it-out for large structural work | `refactor/refactoring.md` | `bee-figure-it-out` | present | line 15, opening paragraph |
| 9 | `arena` Phase B candidates | `research/eval.md` | `bee-arena` | present | line 46, step 4 |
| 10 | `arena` Phase C judge | `research/eval.md` | `bee-arena` | present | line 54, step 5 |
| 11 | the "no `architect`" clause | `research/investigation.md` | `bee-architect` | present | line 40, closing paragraph |
| 12 | `architect` with `architect skipped: <reason>` | `feature/feature.md` | `bee-architect` | present | line 19, step 2 |
| 13 | `arena` for several valid shapes | `feature/feature.md` | `bee-arena` | present | line 48, step 5 |
| 14 | `interrogate` before shipping a contested design | `feature/feature.md` | `bee-interrogate` | present | line 86, new step 10 |
| 15 | `swarm` ten-lane fan-out | `feature/multi-phase-plan.md` | `bee-fanout` | present | lines 54, 115 |
| 16 | no-comments pre-review pass | `feature/multi-phase-plan.md` | `bee-comment-review` | present | line 109 |
| 17 | Appendix D reading list | `feature/multi-phase-plan.md` | `bee-interrogate`, `bee-show-me-your-work` | present | line 202 |
| 18 | show-me-your-work trail | `session/pause-safely.md` | `bee-show-me-your-work` | present | line 30 |
| 19 | show-me-your-work trail | `session/autonomous-run.md` | `bee-show-me-your-work` | present | line 30 |
| 20 | show-me-your-work trail | `session/shipping.md` | `bee-show-me-your-work` | present | line 21 |
| 21 | show-me-your-work trail | `session/orchestrate.md` | `bee-show-me-your-work` | present | line 80 |
| 22 | show-me-your-work trail | `session/autopilot-full.md` | `bee-show-me-your-work` | present | line 28 |
| 23 | show-me-your-work trail | `session/autopilot-stack.md` | `bee-show-me-your-work` | present | line 20 |
| 24 | `arena` for a contested decomposition | `session/orchestrate.md` | `bee-arena` | present | line 135, Frame step |
| 25 | `arena` trees in the in-use check | `session/worktree-cleanup.md` | `bee-arena` | present, skill not named | step 3, line 22 says "arena", with no `bee-arena` anywhere in the file |
| 26 | cross-model trail audit | `session/orchestrate.md` | `bee-show-me-your-work` | present | line 170, Close step |
| 27 | figure-it-out for a bespoke run | `session/orchestrate.md` | `bee-figure-it-out` | present | line 10, intro |
| 28 | `swarm` verifier fan-out | `session/autopilot-full.md` | `bee-fanout` | present | line 40 |
| 29 | `swarm` verifier fan-out | `session/autopilot-stack.md` | `bee-fanout` | present | line 41 |
| 30 | no-comments pass | `session/autopilot-full.md` | `bee-comment-review` | present | line 24 |
| 31 | no-comments pass | `session/autopilot-stack.md` | `bee-comment-review` | present | line 17 |
| 32 | no-comments pass | `session/opening-a-pr.md` | `bee-comment-review` | present | lines 34, 62 |
| 33 | `interrogate` by the pushing worker | `session/opening-a-pr.md` | `bee-interrogate` | present | line 62, step 8 |
| 34 | per-candidate model spread | `research/eval.md` | `bee-arena` lane roles | present | step 4 names `lane-1` to `lane-3` |

### Dropped on purpose, and still dropped

Each of these is stated in the class's port notes:

- `deslop`: no bee counterpart and no decision brings one (`feature.md`,
  `session-ppb-12.md`).
- `prove-it-works`, `guard-the-context-window`, `sequence-verifiable-units`,
  `laziness-protocol`, `model-the-domain`, `never-block-on-the-human`,
  `exhaust-the-design-space`: no skill of that name exists in `skills/`. Each
  keeps its method as a plain sentence and is marked "Still waiting".
- `/loop`, `/goal`, the throughput checkpoint, the plan-shape check script and
  the Cursor and cloud machinery: no bee counterpart. Not D5 skills, so
  ppb-24 did not need to restore them.

### Missing

None. Every drop in the port notes that cites D5 slice 3 for one of the eight
skills now carries a "Restored (ppb-24)" mark, and its step is in the
situation file.

## Fallback line

"No situation fits, or the work is large and cross-cutting: use
bee-figure-it-out." appears exactly once in each index, after `## Situations`
at line 5:

| Index | Fallback line |
|-------|---------------|
| `bugfix.md` | 9 |
| `content.md` | 9 |
| `docs.md` | 10 |
| `feature.md` | 10 |
| `perf.md` | 10 |
| `refactor.md` | 10 |
| `release.md` | 9 |
| `research.md` | 12 |
| `spike.md` | 9 |
| `session/index.md` | 18 |

## Verify

`rg -c 'bee-figure-it-out' skills/bee-planning/playbooks/bugfix.md skills/bee-planning/playbooks/session/index.md`

```
skills/bee-planning/playbooks/bugfix.md:1
skills/bee-planning/playbooks/session/index.md:1
```
