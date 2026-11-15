# Port notes — steps restored in ppb-24

Each step below was dropped by an earlier port cell because its skill did not
exist in bee yet. The skill exists now, so the step is back in its situation
file, naming the bee skill, with its reason. Every restored step is marked
"Restored (ppb-24)" in the port notes of its class, beside the line that dropped
it. No situation was added or removed, and no skill was edited.

Playbook paths are under `skills/bee-planning/playbooks/`.

## Restored, by skill

| Skill | Situation file | Where it came back |
|-------|----------------|--------------------|
| `bee-architect` | `bugfix/bug-fix.md` | Step 4, before the fix when it crosses a function boundary; step 3 confirms the mechanism before this fan-out |
| `bee-interrogate` | `bugfix/bug-fix.md` | Step 4, over a contested plan |
| `bee-tdd` | `bugfix/bug-fix.md` | Step 6, the failing-test-first cadence |
| `bee-architect` | `spike/prototype.md` | Step 7, the shape handoff beside `playbooks/feature.md` |
| `bee-architect` | `perf/perf-issue.md` | Step 4, before a fix that crosses a function boundary |
| `bee-show-me-your-work` | `perf/hillclimb.md` | Step 3, the decision trail |
| `bee-architect` | `refactor/refactoring.md` | Step 4, the target shape |
| `bee-figure-it-out` | `refactor/refactoring.md` | The opening paragraph, for large or cross-cutting structural work |
| `bee-arena` | `research/eval.md` | Steps 4 and 5, Phase B candidates through `lane-1` to `lane-3` and the Phase C judge; the per-candidate model spread comes back as a team setting of those roles |
| `bee-architect` | `research/investigation.md` | The closing paragraph, only for the code change that follows |
| `bee-architect` | `feature/feature.md` | Step 2, with the `architect skipped: <reason>` line |
| `bee-arena` | `feature/feature.md` | Step 5, when the implementation admits several valid shapes |
| `bee-interrogate` | `feature/feature.md` | New step 10, a contested design before shipping; landing moved from step 10 to step 11, and no other file cites that number |
| `bee-fanout` | `feature/multi-phase-plan.md` | The Verification paragraph and the merge-ready verdict box, the ten lanes |
| `bee-comment-review` | `feature/multi-phase-plan.md` | The pre-review box |
| `bee-interrogate`, `bee-show-me-your-work` | `feature/multi-phase-plan.md` | Appendix D, the reading list |
| `bee-show-me-your-work` | `session/pause-safely.md`, `session/autonomous-run.md`, `session/shipping.md`, `session/orchestrate.md`, `session/autopilot-full.md`, `session/autopilot-stack.md` | The decision trail |
| `bee-arena` | `session/orchestrate.md` | Step 1, a contested decomposition or one-way door before the pilot |
| `bee-arena` | `session/worktree-cleanup.md` | Step 3, the sibling trees an arena run spawns |
| `bee-show-me-your-work` | `session/orchestrate.md` | Step 7, the trail audit with its cross-model review |
| `bee-figure-it-out` | `session/orchestrate.md` | The intro, one ambitious run that needs a bespoke workflow |
| `bee-fanout` | `session/autopilot-full.md`, `session/autopilot-stack.md` | The verification fan-out of reviewers |
| `bee-comment-review` | `session/autopilot-full.md`, `session/autopilot-stack.md`, `session/opening-a-pr.md` | The comment pass over the diff |
| `bee-interrogate` | `session/opening-a-pr.md` | Step 8, the pushing worker's pass |

## The fallback line

Every class index (`bugfix.md`, `content.md`, `docs.md`, `feature.md`,
`perf.md`, `refactor.md`, `release.md`, `research.md`, `spike.md`) and
`session/index.md` carries, after its Situations section: "No situation fits, or
the work is large and cross-cutting: use bee-figure-it-out."

## Skills with no step that waited

`bee-reflect`, `bee-recall` and `bee-blast-radius` were never cited by a pstack
playbook, so no earlier port dropped a step for them and none came back here.

## Still waiting, named here

No skill or principle of these names exists in `skills/` yet, so each keeps its
method as a plain sentence in the playbook, and its port note says so:
`prove-it-works`, `guard-the-context-window`, `sequence-verifiable-units`,
`laziness-protocol`, `model-the-domain`, `foundational-thinking`,
`redesign-from-first-principles`, `minimize-reader-load`,
`never-block-on-the-human` and `exhaust-the-design-space`. `deslop` has no bee
counterpart and no decision brings one, so it stays dropped.
