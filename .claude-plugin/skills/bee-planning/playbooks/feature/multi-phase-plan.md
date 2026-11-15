# multi-phase-plan

You own the plan, not the code. The plan is a checklist an owner runs box by
box and the operator audits from the evidence. This is for work that spans
phases or stacked PRs. The plan is the deliverable. Do not implement.

1. When the change is one or two files with an obvious approach, skip the plan.
   Say so and stop, because a plan for a change that small costs more than the
   change.
2. Settle open questions by prototype before you write. For a question about
   layout, timing, behavior, or whether an API works, run
   `playbooks/spike/prototype.md`. Keep the branch, the SHA and the screenshots
   for Appendix A. Ask the operator only about a product or preference call that no run can settle
   (`bee-principle-never-block-on-the-human`), and give options, because a question with options
   costs the operator a pick and a bare question costs a round trip.
3. Explore in gathers started with `bee dispatch prepare --kind gather`, on the read role's configured model. Each returns file pointers, conventions,
   test commands and entry points. No inlined dumps, because the plan's lead keeps its context for the plan
   (`bee-principle-guard-the-context-window`).
4. Choose the program's grouping by the rule in
   `bee-planning/references/planning-reference.md` under "Phase plan vs epic
   map". Use phases only when the work has observable milestones a user could
   demo in order, never technical buckets. Use an epic map when capability or
   risk areas explain the work more honestly than a timeline. Never force 2 to
   4 phases onto work that is really one slice. The rule has one home there, so
   a copy here cannot drift from it.
5. Copy the skeleton below into the plan file and fill every placeholder. Unless
   the operator names a path, write the file under `docs/history/<feature>/`,
   for example `program-plan.md`, beside the feature's own `plan.md`. Keep every
   heading and every sub-block in the order shown. One section per PR. One PR is
   one change with its own evidence, built, verified and committed before the next
   (`bee-principle-sequence-verifiable-units`), so a failure points at one PR. Name the execution playbook in **How to
   read this**. Pick between `playbooks/session/autopilot-full.md` and
   `playbooks/session/autopilot-stack.md` per the rule at the end of
   `playbooks/session/autopilot-stack.md`. A standing program takes
   `playbooks/session/orchestrate.md`.
6. Write under `bee-technical-writing` in full, then `bee-unslop`. The body is
   one Diátaxis mode, how-to. Appendices hold explanation and reference. Two
   rules apply verbatim. "i dont want any abstract metaphors" and "write like
   hemingway". Each heading states the task or the finding. No long dashes. No
   mid-sentence colons.
7. Check the file against the skeleton by hand, heading by heading, in the order
   shown, and fix every drift: the verification rule opens every verification
   block, and the punctuation rules hold. A plan whose shape drifts cannot be
   audited box by box, and a rule left to memory is skipped under load
   (`bee-principle-encode-lessons-in-structure`).
8. Hand back. Post the plan path and the result of the shape check, then stop.
   Execution starts on the operator's explicit go, under the execution playbook
   the plan names, because a plan handed over is not a go.

**Verification.** Tests alone are not sufficient verification. A PR is verified
only when its unit, live and perf boxes are all checked
(`bee-principle-prove-it-works`). That sentence is the
verification rule. Every verification block opens with it. The live block is
mandatory. Ten lanes at the PR head drive the real surface, fanned out per
`bee-fanout`, each started through `bee dispatch prepare --kind reviewer` on
the review role's configured model. Each lane is one box with a concrete scenario, the screenshot it saves,
and its pass predicate. The perf block names the metric, the probe, the trunk
baseline measured first, and the rule with the number that fails. A PR that
changes an interaction is review-gated. The operator reviews it in chat with
screenshots and a video before merge. A PR that changes no interaction writes
`**Review gate.** None. <PR id> is not review-gated.` and no boxes under it.

**Control skill.** Pick it by surface. Drive each surface through the project's
`verify-app` skill, or through the real interface itself when the project has
none. A PR that touches two surfaces gets lanes on both. A surface that nothing
can drive is a risk in Appendix C, and its live block still names how each lane
drives it.

````markdown
# <Program> plan

<Under ten lines. What changes, for whom, the rule the program enforces, and the PR ids in order.>

## How to read this

One box is one unit of work. Every box names the evidence that checks it. A nested box is a sub-step of the box above it. Check a box only when its evidence exists, a file, a log line, a screenshot, a test run, or a SHA. The body is a how-to. The appendices explain and record.

The program runs `skills/bee-planning/playbooks/session/<execution playbook>.md`. <Who merges, and which PR ids are the operator's items that stop at merge-ready.>

Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

## Program checklist

### Arm the program

- [ ] State the protocol and this plan to the operator, then stop. Start execution only on the operator's explicit go.
- [ ] On the operator's go, write the exit condition as a checkable predicate and drive it as `playbooks/session/autonomous-run.md` does. Use this exact text. "<The plan path, the PR ids in order, the verification rule, who merges, and the done condition.>"
- [ ] Read these from trunk at program start. Re-read them at every tick.
  - [ ] `git show main:skills/bee-planning/playbooks/session/<execution playbook>.md`
  - [ ] `git show main:<verify-app skill path, or the real interface's doc>`
  - [ ] `git show main:skills/bee-planning/playbooks/session/opening-a-pr.md`
  - [ ] `git show main:skills/<each other leaf skill the program uses>/SKILL.md`
- [ ] Arm the 30-minute audit tick under `bee-herding` (`bee herding control-loop`). Never leave the cadence to memory.
- [ ] Use this tick prompt, verbatim. "Re-read the execution playbook from trunk and the exit condition. Audit the operation against both and fix drift in this tick. Probe every active lane and judge progress by side effects only. Stand down a stuck lane and dispatch its replacement now. Then send the operator a status message, whether or not anything changed, with the queue table of PR, owner, state, and head SHA, the verdicts since the last tick, what merged, open operator gates, and blockers."
- [ ] On the operator's hold or stand-down, send every owner a zero-writes order at once.

### Spawn owners

- [ ] Spawn one owner per PR with the full lifecycle the execution playbook names, through `bee dispatch prepare --runtime claude --kind cell --json`.
- [ ] Follow this dependency graph. Start dependent work only after its parent merges, or base it on the parent branch when the execution playbook stacks.
  - [ ] <PR id> and <PR id> are independent and first. Both branch from `main`.
  - [ ] <PR id> after <PR id>.
- [ ] Hold the file boundaries. <PR id or class> touches only `<glob>`.
- [ ] Hold the review gate. <PR ids> change an interaction. They wait for the operator's review in chat with screenshots and a video before merge.

### PR mechanics, for every PR

- [ ] Land each PR through `playbooks/session/opening-a-pr.md`. The `ship_visibility` rule opens it as a draft and the draft stays a draft until the operator says otherwise.
- [ ] Run the repo's lint and typecheck once before the PR-facing push. Push with hooks on.
- [ ] Run `bee-comment-review` over the diff before review, because bee writes no comment in code and a comment the diff added is the first thing a reviewer reads.
- [ ] Triage every review comment per `playbooks/session/babysit.md`.
- [ ] Send drift against current trunk to the merger as a cell before babysit and again before the merge-ready report. The owner never rebases, per `playbooks/session/orchestrate.md` § Merge safety.

### Verdict and merge, for every PR

- [ ] At the merge-ready head SHA, fan out the lanes per `bee-fanout`, each through `bee dispatch prepare --runtime claude --kind reviewer --json`. One gates lane. The ten live lanes from the PR's **Verify, live** block. The perf lane from its **Verify, perf** block. One audit lane that reads the diff and the receipts and distrusts the PR body.
- [ ] Clean only when every lane is `PASS`. Findings go back to the owner. A new head gets fresh lanes and a fresh verdict.
- [ ] <The merge or append rule from the execution playbook, with the patch-id rule from `playbooks/session/shipping.md`.>

### Boot recipe, for every live lane

Each live lane runs in its own worktree at the PR head. Drive through the `verify-app` skill, or the real interface when the project has none.

- [ ] `git fetch origin <head-branch> && git checkout <head SHA>`.
- [ ] <Start the backend and the surface. Wait for ready.>
- [ ] <Deliver input only through the driving skill's commands. Name the read-only diagnostics.>
- [ ] Save every screenshot to `/tmp/lanes-<pr-id>/worker-<n>/<slug>.png` and return the paths with the report.

## <Task as a verb phrase> (<PR id>)

**Depends on.** <PR id, or None.>

**Files.**

- [ ] Edit `<path>`.
- [ ] Create `<path>`.
- [ ] Delete `<path>`.

**Build.**

- [ ] <One change. Name the symbol and the file.>

**You see.**

- [ ] <One observable result, with the exact log line or screen state.>

**Verify, unit.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] <Test file and the case it gains.> Run `<command>`.

**Verify, live.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked. Ten lanes at the PR head, per the boot recipe.

- [ ] Lane 1. <Scenario.> Save `<slug>.png`. Pass when <predicate>.
- [ ] Lane 2. <Scenario.> Save `<slug>.png`. Pass when <predicate>.
- [ ] Lane 3. <Scenario.> Save `<slug>.png`. Pass when <predicate>.
- [ ] Lane 4. <Scenario.> Save `<slug>.png`. Pass when <predicate>.
- [ ] Lane 5. <Scenario.> Save `<slug>.png`. Pass when <predicate>.
- [ ] Lane 6. <Scenario.> Save `<slug>.png`. Pass when <predicate>.
- [ ] Lane 7. <Scenario.> Save `<slug>.png`. Pass when <predicate>.
- [ ] Lane 8. <Scenario.> Save `<slug>.png`. Pass when <predicate>.
- [ ] Lane 9. <Scenario.> Save `<slug>.png`. Pass when <predicate>.
- [ ] Lane 10. <Scenario.> Save `<slug>.png`. Pass when <predicate>.

**Verify, perf.** Tests alone are not sufficient verification. A PR is verified only when its unit, live, and perf boxes are all checked.

- [ ] Metric. <What is measured.>
- [ ] Probe. <The command or procedure, run at trunk and at the head, interleaved.>
- [ ] Baseline. Record the trunk <value> first.
- [ ] Rule. <Head against trunk, with the number that fails.>

**Review gate.** The operator reviews before merge.

- [ ] Copy lane <n> screenshots into `<media path>/<pr-id>-review-<slug>.png`.
- [ ] Record a 30 to 60 second video of the change running on the real surface, when the host can capture one, and save it as `<media path>/<pr-id>-review.mp4`. When it cannot, say so and post the screenshots alone.
- [ ] Post the screenshots and the video in chat. Stop at merge-ready. Wait for the operator's click.

**Merge.**

- [ ] The clean verdict at the exact head SHA.
- [ ] Review comments triaged.
- [ ] The head's patch-id is unchanged since the verdict. Drift against trunk went to the merger as a cell, never an owner's rebase.
- [ ] <The root runs `bee worktree merge` per the execution playbook, or the PR stops at merge-ready for the operator. The `uat` door of `bee close` stays the operator's.>

## Close the program

- [ ] Every box above is checked with its evidence.
- [ ] Reply to the operator with the report the execution playbook names.

## Appendix A. Prototype evidence

<Each open question a prototype answered, with the branch, the SHA, and the artifact links. Each question that stays unproven.>

## Appendix B. Alternatives rejected

<Each approach weighed and why it lost.>

## Appendix C. Risks

<Each risk with the PR it lands in and what the owner watches.>

## Appendix D. Links and reading list

<Docs to read before editing. Which PRs get a `bee-how` gather first and a `bee-interrogate` pass. The decision trail per `bee-show-me-your-work`, under `docs/history/<feature>/`.>
````

Reply: the plan path, the PR ids with their dependencies and the review-gated
set, what the prototypes proved and what stays unproven, and the result of the
shape check.

Proof line: the plan path and its headings in order, verbatim from
`rg -n '^#{2,3} ' <plan path>`.
