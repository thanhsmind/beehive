# Port notes — feature class

Source: pstack poteto-mode at commit `b9ddc83`, MIT, copyright 2026 Lauren Tan
(notice in `skills/bee-planning/playbooks/LICENSE.pstack`). Cell ppb-9. This
file is the only place a pstack-only tool is named; no file under `skills/`
names one. `skills/bee-planning/playbooks/feature.md` is now the class index
with two situations.

## feature

Source `playbooks/feature.md`, merged with the bee `feature` class playbook
(D3) into `skills/bee-planning/playbooks/feature/feature.md`.

Tool swaps:

- The `how` skill (step 1) became `bee-how`, run as a gather
  (`bee dispatch prepare --kind gather`).
- Delegating code-writing to a subagent on the named default feature model
  became a cell (`bee dispatch prepare --kind cell`) on the code role's
  configured model.
- "You can spawn a subagent even though you are one" and the owner that is
  forbidden to spawn became "a lead that cannot dispatch owns the diff
  directly with the same review separation".
- The principle skill `separate-before-serializing-shared-state` became
  `bee-principle-separate-before-serializing-shared-state`, the bee principle
  of the same rule.
- The **Comments** section of the router became the bee rule: no comment in
  code, the why goes to a `docs/knowledge/` concept or `bee decisions log`.
- The `sequence-verifiable-units` principle skill (step 6) became the
  build-verify-commit-each-unit sentence in step 9, with the `cell: <id>`
  trailer rule from `session/opening-a-pr.md`.
- The control skill (step 5, "Verify on the matching surface") became the
  project's `verify-app` skill, or the real interface when the project has
  none.
- Step 8, run Opening a PR, became step 10, a pointer to
  `playbooks/session/opening-a-pr.md`: the draft PR `ship_visibility` opens,
  `bee worktree merge` on green, and the `uat` door of `bee close`, which stays
  the owner's.

Dropped steps and clauses:

- Step 2, `architect` for parallel design exploration, and its
  `architect skipped: <reason>` line: the skill does not exist in bee yet (D5,
  slice 3). Restored (ppb-24): step 2 runs `bee-architect` and keeps the
  `architect skipped: <reason>` line.
- The `arena` delegation when the implementation admits several valid shapes:
  the skill does not exist in bee yet (D5, slice 3). The single-cell delegation
  stays. Restored (ppb-24): step 5 runs `bee-arena` when the implementation
  admits several valid shapes.
- Step 7, `interrogate` before shipping a contested design: the skill does not
  exist in bee yet (D5, slice 3). No replacement step was written. Restored
  (ppb-24): a new step 10 runs `bee-interrogate` before shipping a contested
  design, so landing is now step 11 and `opening-a-pr` is pointed at from it.
- The `principle-model-the-domain` pointer in the delegation step: no bee
  principle of that name yet (D5, slice 3). Its rule stays in plain words in
  step 2, where it joins the bee "name the data shape first" step.
- The `Laziness Protocol` clause ("does not override it"): no bee principle of
  that name yet (D5, slice 3). The sentence keeps the override rule without the
  name.

Kept from the bee `feature` playbook (D3), each as a step with its reason: the
data shape first (step 2), the walking skeleton with no stub (step 5), the
current slice only (step 5), the user-visible path proved, not the unit (step
6), the `docs/knowledge/` sync in the same change (step 7), the `statechart`
model with its skip-and-say-so clause (step 3, moved ahead of the code because
its own text says "before the flow's code"), the
`create-formal-verification-skill` and `maintain-formal-verification-skill`
step with its skip-and-say-so clause (step 8), the closing line "A green unit
behind a path nobody can walk is not a result.", the Proof line, and the Proof
floor line (now in the index, byte-identical).

Added to the pstack text: a reason on every step where pstack gave none
(decision `61b52e96`).

Stale pointer in a sibling class, not edited here: `bugfix/bug-fix.md` cites
"`feature.md` step 6" (the `statechart` model) and "`feature.md` step 7" (the
formal verify skill). `feature.md` is now the class index with no steps, and
those steps are now steps 3 and 8 of `feature/feature.md`.

## multi-phase-plan

Source `playbooks/multi-phase-plan.md`, written to
`skills/bee-planning/playbooks/feature/multi-phase-plan.md`. The pstack
playbook has no bee counterpart (it was not in the D3 overlap list), so the
pstack text is the body.

Tool swaps:

- The prototype playbook reference became `playbooks/spike/prototype.md`.
- Exploring in subagents of the named agent type with an explicit model per the
  Subagents section became gathers (`bee dispatch prepare --kind gather`) on
  the read role's configured model.
- The `guard-the-context-window` principle pointer became its method sentence
  inline ("the plan's lead keeps its context for the plan"); the skill does not
  exist in bee yet (D5, slice 3).
- The `never-block-on-the-human` principle pointer became its method sentence
  inline ("give options"); same reason.
- The `sequence-verifiable-units` principle pointer became the sentence "built,
  verified and committed before the next, so a failure points at one PR".
- The agent store's `docs/` as the plan location became
  `docs/history/<feature>/`, for example `program-plan.md`, beside the
  feature's own `plan.md`.
- `/technical-writing` and `/unslop` became `bee-technical-writing` and
  `bee-unslop`.
- The `prove-it-works` principle pointer became the verification rule sentence
  itself, which was already the principle's content in the source.
- The live lanes' named model and the ten-lane fan-out "per the swarm skill"
  became lanes started through `bee dispatch prepare --kind reviewer` on the
  review role's configured model. The lane count, one box per lane, the saved
  screenshot and the pass predicate stay. Restored (ppb-24): the lanes are
  fanned out per `bee-fanout`, in the plan's Verification paragraph and in the
  merge-ready verdict box.
- The control skill by surface (browser, CLI and TUI skills of the editor team
  kit, a native mobile skill) became the project's `verify-app` skill, or the
  real interface when the project has none.
- The skeleton's `/goal` became a checkable exit condition driven as
  `playbooks/session/autonomous-run.md` does, the same swap the session group
  made.
- The 30-minute audit tick on a terminal `/loop` or a cloud wake chain became
  the tick armed under `bee-herding` (`bee herding control-loop`).
- The `git show origin/main:` reads became `git show main:` on the bee playbook
  paths under `skills/bee-planning/playbooks/session/`, matching
  `autopilot-full`.
- The PR mechanics (open ready with the host CLI, or with the stacking tool)
  became `playbooks/session/opening-a-pr.md`. Bee wins on draft: the
  `ship_visibility` rule opens the PR as a draft and the draft stays a draft
  until the operator says otherwise.
- Triage of every bot and security-reviewer comment per the router's triage
  reference became triage of every review comment per
  `playbooks/session/babysit.md`.
- The pre-review no-comments pass became "strip every comment the diff added
  before review, because bee writes no comment in code" (restored in ppb-24:
  the box runs `bee-comment-review` over the diff).
- Each lane's own cloud VM became its own worktree at the PR head.
- The lane screenshot path `/tmp/swarm-<pr-id>/` became
  `/tmp/lanes-<pr-id>/`.
- The merge box (the owner squash-merges, or the root appends to the stack and
  the operator lands it) became `bee worktree merge` per the execution
  playbook, or a stop at merge-ready, with the `uat` door of `bee close` staying
  the operator's. The patch-id rule now points at `playbooks/session/shipping.md`.
- The skill reading list in Appendix D (`how`, `interrogate`,
  `show-me-your-work`) became a `bee-how` gather and the decision trail under
  `docs/history/<feature>/` (restored in ppb-24: Appendix D names
  `bee-interrogate` and the trail per `bee-show-me-your-work`).
- The pronoun for the operator became "the operator" throughout.

Dropped steps and clauses:

- The plan-shape check script and its `encode-lessons-in-structure` pointer
  (step 6): bee has no script for the skeleton's shape. `bee plan check`
  checks the `bee-plan/v2` sections of a feature's own `plan.md`, and a
  program plan carries other headings, so it cannot stand in. Step 7 checks the
  skeleton by hand instead and says why. A bee shape check is a candidate for
  a later slice.
- The deslop pass before each commit: bee has no counterpart skill.
- The `/loop` and `/goal` Cursor commands as commands: replaced as listed above,
  not kept.
- The video recorded "on a lane VM": kept as a conditional step, "when the host
  can capture one", because bee has no capture tool.
- The named default lane model: bee names no model in a playbook; the role's
  configured model applies.
- The `swarm` skill's own lane protocol: the skill does not exist in bee yet
  (D5, slice 3, where it returns under a bee name that does not collide with
  `bee-swarming`). Restored (ppb-24) as `bee-fanout`.

Kept from bee (D3 pointer rule): the phase-plan-versus-epic-map rule stays
in `skills/bee-planning/references/planning-reference.md` under "Phase plan vs
epic map". Step 4 points at it and states its three clauses so the reader can
act without opening it; the pointer is the single home.

Added to the pstack text: a reason on every step where pstack gave none
(decision `61b52e96`), step 4 (the grouping rule), and a Proof line (the plan
headings in order, from `rg -n '^#{2,3} ' <plan path>`).

## Steps re-added in ppb-24

Restored: `architect` (feature step 2, now `bee-architect`), `arena` (feature
step 5, now `bee-arena`), `interrogate` (feature step 10, now
`bee-interrogate`, which moved landing to step 11), the `swarm` fan-out
(multi-phase-plan lanes, now `bee-fanout`), the pre-review no-comments pass
(now `bee-comment-review`) and the Appendix D reading list (`bee-interrogate`,
`bee-show-me-your-work`).

Still waiting, because no skill of that name exists in `skills/`:
`principle-model-the-domain` (feature step 2), `laziness-protocol` (feature step
5), `guard-the-context-window`, `never-block-on-the-human` and `prove-it-works`
pointers (multi-phase-plan). Each keeps its method as a plain sentence. The
deslop pass has no bee counterpart and stays dropped.
