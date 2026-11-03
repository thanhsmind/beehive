---
artifact_contract: bee-research/v1
topic: pstack-craft-study
depth: deep
date: 2026-09-11
---

# pstack craft study: what pstack does better, and what bee adopts

## Bottom Line

- Recommendation (ladder rung): adapt-upstream. Carry pstack's better craft into bee's own skills. This is not a port of pstack's skills.
- bee has the stronger method in almost every area. It counts risk flags, labels its claims, freezes plans, lets the CLI enforce proof, keeps review user-invoked, and runs blind lanes that can be checked. pstack has the stronger surface. Its documents keep one fixed skeleton that a script checks. Every unit says what you will see and what proves it. Every playbook ends on an explicit evidence line. Its steps carry checks you can run in the moment.
- The gap is real in bee's own records. A real plan, `docs/history/pstack-gaps/plan.md`, broke four rules of bee's own template, and nothing caught it. bee-planning also contradicts itself: its hard rule says "no cells before the gate", but its hat wave "reviews drafted cells before the gate".
- Why this beats the next-best rung: bee's existing machinery already does the enforcement. Adopting only the surface practices, and the few new checks, gives the gain without a second system.
- Confidence: 85%. Six lanes read both sides file by file. One lane (explain and research) is marked separately below.
- Suggested next step: bee-planning for `pstack-craft-adoption`. Slice 1 holds the shortcut skills and the doctrine edits. Slice 2 holds the Rust checks. Slice 3 holds the new principles.

## Question and method

The user asked for a craft study, not a feature check. For every pstack skill or playbook, the study compares how well it does the job against bee's counterpart. It judges four things: method, structure, readability and implementability. It then adopts the better practices into bee's skills. A feature match is not a reason to skip a practice (decision `1e77b136`). The user also asked for thin shortcut skills, `bee-how` and `bee-why`, to sit beside `bee-teach`.

Source manifest (decision `729f6e82`): cursor/plugins, ref main, commit `7366ac128bdf95f45e6734f412b49a4031800169`, scope `pstack/skills/**`.

Seven lanes ran in parallel through `bee dispatch prepare --kind advisor --role plan`. The Fable quota was spent, so they used the plan role instead (decision `5c6777d0`). Every lane read both sides file by file and cited `Upstream` for pstack and `Local` for bee at the line. The leader then checked each defect a lane reported before it went into this study.

## Findings by lane

### L1 planning

bee's method is stronger: lanes set by counted flags, the claims table, the reality touch, the frozen plan, and cells that carry `rg` hits. pstack's document is stronger:

- One skeleton, checked by a script (Upstream `check-plan.mjs:8-18,63-107`).
- An outcome summary of at most ten lines (Upstream `multi-phase-plan.md:20`).
- A "You see" line per unit (Upstream `multi-phase-plan.md:93-95`).
- A literal pass predicate per verification scenario (Upstream `multi-phase-plan.md:103-119`).
- Each risk tied to the unit it lands in (Upstream `multi-phase-plan.md:147-149`).

bee's plan skeleton lives only in a template, and the leader checks it by hand (Local `skills/bee-planning/SKILL.md:106-114`).

### L2 class playbooks

bee's classes are shorter and cite rather than copy, and that must stay. pstack gives each class a method:

- **Bug fix.** Hypothesis elimination: test the split that removes the most candidates (Upstream `bug-fix.md:8`). Revert every edit that a refuted hypothesis motivated (:5). An inconclusive run, or a run on the wrong surface, is not a pass (:10).
- **Hillclimb.** Discipline for a sustained metric: a harness proven to separate cases, the median of N, and a stop rule of target plus a minimum attempt count (Upstream `hillclimb.md:7-18`).
- **Refactor.** Delete dead code first, migrate every caller then delete the old path, and change nothing that would make a check pass (Upstream `refactoring.md:7-13`, `visual-parity.md:6`).
- **Every class.** An explicit evidence line at the end (Upstream `bug-fix.md:17`, `perf-issue.md:24`).

### L3 execution

bee's enforcement is stronger: claim guards, reservations, finish validation, and the rule that red never moves to another model. pstack is better in five places:

- A pause must be safe. It ends at a safe boundary and makes a `wip:` commit (Upstream `pause-safely.md:5-10`).
- In-flight work is reconciled on resume (Upstream `orchestrate.md:98,101`).
- A dependency passes context, not only order (Upstream `orchestrate.md:41,56`).
- The worker pool refills as each cell caps, not at wave barriers (Upstream `orchestrate.md:16,63`).
- "Stuck" means no side effect past an expected runtime (Upstream `orchestrate.md:95`).

### L4 explain and research

bee is stronger here. Trace's disjoint-entry-point rule can be checked. UNFOLLOWED steps carry a named reason. The sweep takes seven rows and returns seven. Its sources are native to the repo, with the decision log first. The leader writes the answer, and the dispatch door picks the model. Confidence tiers take 28 lines, where pstack spreads them over about 280. Session state is recorded rather than rebuilt from transcripts.

pstack is better in seven small ways:

- An anchor step before the sweep: a `path:line` range, the symbols, `git blame -L`, and PR numbers (Upstream `why/SKILL.md:23-54`).
- A fixed return shape for trace workers, plus a rule that resolves disagreement by opening the anchor (Upstream `explorer-prompt.md:32-52`, `explainer-prompt.md:19`).
- A self-check before sending, with the counterfactual question and no neighbor substitution (Upstream `epistemics.md:137-144`, `investigator-prompt.md:19,63`).
- "Question and anchor" at the top of the answer, and "who would know" on each gap (Upstream `synthesizer-prompt.md:53-59`, `epistemics.md:123-129`).
- Read back to the first commit, not the last, and add incident terms for defensive code (Upstream `why/SKILL.md:146-148`, `incident-postmortem.md`).
- When the target is vague, state your reading and continue (Upstream `how/SKILL.md:13`).
- Close a research question as settled, refuted or inconclusive (Upstream `figure-it-out:17-21,43`).

For catch-me-up, pstack's reply shape reads better: a scope line, a short capsule, one status tag per thread, what was tried and reverted, and one next move (Upstream `recall/SKILL.md:24-33`). bee has no defined reply shape for that question.

### L5 design

bee's blind lanes are stronger. Their briefs are byte-identical and linted, and `bee blind check` can check them. pstack is better in three places:

- A rubric is written before the fan-out (Upstream `arena:26-27,47`).
- The leader picks a base, then grafts one or two parts from each rejected candidate. If candidates converge, it ships the consensus. If they diverge, it reframes the question and never averages (Upstream `arena:43-67`).
- A five-rung safety-fact evidence ladder replaces a confidence word (Upstream `blast-radius:19-35`).

### L6 quality

bee is stronger here: proof enforced at cap, a user-invoked review with a frozen scope, gated self-evolution, and deep test craft. pstack is better in three narrow places:

- A bug fix records red-before and green-after, not only the green run (Upstream `tdd/SKILL.md:17-20,38-44`).
- A live proof names the feature, the entry point and the evidence path (Upstream `feature-map-example/README.md:23-31`).
- Proof pointers must resolve (Upstream `show-me-your-work:54-63`).

### L7 principles

bee's triggering is stronger: orient names candidates per class, and every pstack principle sets `disable-model-invocation`. pstack is better at "what do I do, and am I done?". Its principles carry three lines that bee's lack:

- A Check line you can run in the moment (Upstream `test-behavior-not-implementation:11`, `make-operations-idempotent:19-24`).
- A "Not when" line (Upstream `exhaust-the-design-space:18-21`).
- An evidence line (Upstream `build-the-lever:18`).

Eight pstack principles fill real gaps in bee. They are ranked below.

## Defects in bee that the study found (verified by the leader)

1. bee-planning's hard rule says "No cells, no prep artifacts, before the gate is approved" (Local `skills/bee-planning/SKILL.md:164`). But its hat wave applies findings "directly to the cells (legal — cells are mutable before the gate)" and reviews "the drafted cells (`bee cells list`)" (Local `skills/bee-planning/references/planning-reference.md:533-543`). The CLI refuses `cells add` before the gate ("D3: no cells before the gate").
2. `skills/bee-principle-reproduce-first/SKILL.md:17` names `principle-red-before-green` without the load-bearing `bee-` prefix, so the name does not resolve.
3. `skills/bee-swarming/references/worker-details.md:182` writes the commit trailer as `-m "Cell: <cell-id>"`. AGENTS.md and the worker contract require the literal `cell: <id>`.
4. The Result form has three homes (Local `skills/bee-swarming/SKILL.md:170-174`, `swarming-reference.md:548-557`, `worker-details.md:185-203`). The departure rule has two (`swarming-reference.md:592-603`, `worker-details.md:94-109`).
5. `docs/history/pstack-gaps/plan.md` has no Shape, Test matrix, Open Questions or Out of scope section. Some of its claim rows have no line number, and some paraphrase their evidence instead of quoting it.

## Adoption backlog, ranked

### Slice 1: doctrine edits and shortcut skills (docs only, runs in parallel by file)

| # | Adoption | Target | Lane | Effort |
|---|---|---|---|---|
| 1 | `bee-how` and `bee-why`: thin shortcut skills that route to Trace and Provenance sweep | `skills/bee-how/`, `skills/bee-why/` (new) | L4 | S |
| 2 | Plan skeleton: `## Summary` of at most 5 lines; a `## Cells — current slice (preview)` table with a "You see" and a proof column; a `Waves:` line; `Pass when <literal>` on each test row, plus a main-vs-head row for behavior changes; a `Lands in` column on the risk map. The hat wave reviews the preview table, which removes defect 1 | `skills/bee-planning/references/planning-reference.md`, `skills/bee-planning/SKILL.md` | L1 | M |
| 3 | Class playbooks. **bugfix:** hypothesis elimination, revert what a refuted hypothesis motivated, and "inconclusive is not a pass". **Every class:** a "Proof line:". **perf:** a sustained-target block. **refactor:** subtract first, migrate then delete, anti-tamper. **research:** a tradeoffs table, and forensics on a captured artifact. **spike:** one unrequested variant | `planning-reference.md` § Class playbooks | L2 | S |
| 4 | Proof honesty: a bug fix carries two proof lines (red-before, then green-after); `green:live` names `<feature>@<entry-point>` and the evidence path; red-before-green gains a delegation clause | `.bee/expertise/tests.md`, `skills/bee-verifying/SKILL.md`, `skills/bee-principle-red-before-green/SKILL.md` | L6, L7 | S |
| 5 | Swarming. Handoff: a safe boundary, a `wip:` commit, and the fields `verified`, `tree` and `gotchas`. Resume: triage every in-flight cell. Also: a definition of stuck; a failure taxonomy with a re-dispatch budget; a stop line for systemic failure; a pilot before a same-shape fan-out; a triage when CI goes red; a Reply line. Fix defects 3 and 4 | `skills/bee-swarming/SKILL.md`, `references/swarming-reference.md`, `references/worker-details.md` | L3 | M |
| 6 | Blind lanes: a move 0 that writes `## Rubric` into the dossier; Converge picks a base, then grafts; `## Grafts` names its source; the rule is diverge → reframe, never average. The hat-wave digest carries a Dismissed list | `skills/bee-hive/references/gates-and-delegation.md`, `skills/bee-planning/references/design-sketch.md` | L5 | S |
| 7 | Review: the five-rung safety-fact ladder replaces confidence words; a lead filter at synthesis; smell entries for Swelling File, Dual Path and bolted-on test; one bounded retry for broken reviewer output | `.bee/expertise/review.md`, `skills/bee-reviewing/SKILL.md`, `references/reviewing-reference.md` | L5, L6 | S |
| 8 | Principles: a `**Check:**` line in six skills and `**Not when:**` lines on the broad ones; the vacuity check; the pattern sweep in crash-site; a second question in the deletion test; the "Not the same as" links. Fix defect 2 | `skills/bee-principle-*/SKILL.md` | L7 | M |

| 9 | Trace and Provenance sweep. Add an anchor step, a worker return shape and a disagreement rule, and a self-check. Open each answer with the question and its anchor. On each gap, name who would know. Add the first-commit rule and incident terms. bee-researching gains the settled/refuted/inconclusive close and answers Trace and Provenance questions in the reply | `skills/bee-researching/references/trace-and-provenance.md`, `skills/bee-researching/SKILL.md` | L4 | S |
| 10 | Capture and verify upkeep. A decision's `why` names its evidence pointer. Check that each proof pointer resolves before the harvest. Harvest through two named lenses (learnings, divergent). A lesson that exists but did not fire gets a wording or placement fix, never a second copy. A Catch-me-up reply shape. Verify upkeep lands through a worktree and a cell, not a PR, and stops when no verify skill exists | `skills/bee-capturing/SKILL.md`, `skills/bee-verify-upkeep/SKILL.md`, `skills/bee-hive/references/routing-and-contracts.md` | L4, L6 | S |

Rows 9 and 10 were in the lane digests but missing from this table's first draft.
The plan-step hat wave found the gap, and they are added here so the plan's scope matches the study.

### Slice 2: checks in the Rust binary

| # | Adoption | Target | Lane | Effort |
|---|---|---|---|---|
| 9 | A `bee plan check` verb (or a gate dry run). It checks that the plan's sections exist and are in order, that a class playbook is cited, and that every D-id is covered. It also checks that the evidence in each `read` claim row is a byte substring of the anchored lines. It reuses `plan_claims.rs` | `packages/bee-rs/crates/bee/src/verbs/state_group/` | L1 | M |
| 10 | Upstream trace relay: `dispatch prepare --kind cell` renders each capped dependency's outcome, commit, files and deviations | the dispatch renderer | L3 | M |
| 11 | Patch-id proof currency: `cells finish` stores the patch-id and `worktree merge` recomputes it | cells and worktree verbs | L3 | M |
| 12 | `bee close` warns when a proof line points at a path that does not exist | close verb | L6 | S |
| 13 | Rolling refill, if `bee dispatch wave --limit` cannot already do it | dispatch wave | L3 | S-M |

### Slice 3: new principles

Each new principle needs the `bee-principle-` prefix, an index row with spoken and classes lines, and a craft-guide home. In order: attack-the-premise, encode-lessons-in-structure, build-the-lever, migrate-callers-then-delete, subtract-before-you-add, boundary-discipline, make-illegal-states-unrepresentable, separate-before-serializing-shared-state (L7).

## The shortcut skills

`bee-how` and `bee-why` are thin doors. Each scopes the question, invokes bee-researching, loads `trace-and-provenance.md`, and runs "Trace" or "Provenance sweep". The procedure owns every step; the skill never restates one.

**Scoping rules for both skills:**
- A vague target gets its reading as the first line of the reply, and the skill never stops to ask.
- A question that is part how and part why sends the other part to the one next action.
- Research on an outside library goes to bee-researching.

**Reply for both skills:**
- The first line is the answer in one sentence. For `bee-why` it also carries the tier.
- The answer shape follows the procedure, and tier words and empty rows stay exactly as the procedure writes them.
- The reply closes on exactly one next action. At a gate, the gate wins.

The texts that L4 proposed are the starting candidates. The Iron Law applies: RED pressure tests run before either skill ships. The tests cover four risks: a vague `/bee-how` asks back or pastes a file list; `/bee-why` agrees with the guess in the question; `/bee-why` drops empty rows on a thin history; the shortcut copies Trace's steps.

One source fix comes first. bee-researching always runs its four-step order and writes a brief file. A Trace or Provenance question must skip that order and answer in the reply. Add this line to `bee-researching/SKILL.md` § Output.

Once the shortcuts ship, bee-teach's one next action can be a ready `/bee-how <question>` or `/bee-why <question>`.

## What bee does better and keeps

- Lanes set by counted risk flags.
- The load-bearing claims table, and a gate that refuses `guessed` rows.
- The reality touch.
- The frozen plan.
- Proof at the cap, enforced by the CLI.
- Cells that carry `rg` hits.
- One home for each rule.
- The split between the plan and the cell.
- Claim guards, reservations and finish validation.
- A red result never moves to another model.
- One dispatch door.
- The user's request passed verbatim to workers.
- Blind lanes that can be checked, and lanes kept separate from hats.
- Review that is user-invoked, with a frozen scope, a finding schema, and Dismissed reasons that name what was checked.
- Gated bee-evolving.
- Deep test craft.
- Principles that fire on their own through orient, each with one named example and a Depth pointer.

## Risks, unknowns, follow-ups

- **Seven lanes, one model family.** All lanes ran on Opus through the plan role. A second family did not cross-check them.
- **Rust code not read.** The lanes read doctrine text, not the Rust gate code. Slice 2 needs a reality touch on each verb before its plan.
- A suspected defect was cleared. `bee finish` is a real verb, the "Flow spelling of `bee cells finish`" (`bee --help --names`). A lane that reports a defect can be wrong, so the leader checks every one before it reaches this study.
- **Slice 1 row 9: Trace and Provenance sweep improvements** (L4, S, target `skills/bee-researching/references/trace-and-provenance.md`). This row adds an anchor step 0, a fixed worker return shape with a rule for disagreements, a self-check before sending, "Question and anchor" at the top of the answer, "who would know" on each gap, the first-commit rule, incident search terms, and the "complex, say why" line. The same row adds the settled/refuted/inconclusive close to `bee-researching/SKILL.md` and a catch-me-up reply shape to the Communication contract.
- **Doctrine size.** Every Slice 1 edit adds text to always-read doctrine. The plan check must keep one home per rule, and new detail goes into `references/` or expertise guides, not into skill bodies.

## Source pack

- **Upstream:** `pstack/skills/**` at `7366ac12`. The lane digests list each file they read.
- **Local:** each digest lists its bee files, and the leader checked every defect above with `sed` or `rg` on the line.
- **Lane digests** (session scratchpad, used for this synthesis): `L1-digest.md`, `L2-digest.md`, `L3-digest.md`, `L5-digest.md`, `L6-digest.md`, `L7-digest.md`.
