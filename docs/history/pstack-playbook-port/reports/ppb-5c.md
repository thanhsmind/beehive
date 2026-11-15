# ppb-5c — perf port checked against its sources

Scope: what ppb-5 wrote in commit `800d8342e`. That is the class index
`skills/bee-planning/playbooks/perf.md`, the situation files
`perf/perf-issue.md` and `perf/hillclimb.md`, and the port notes
`docs/history/pstack-playbook-port/port-notes/perf.md`. The files are checked
against the pstack sources `pstack/skills/poteto-mode/playbooks/perf-issue.md`
and `hillclimb.md` (commit `b9ddc83`), and against the old bee `perf.md` from
`git show 800d8342e^:skills/bee-planning/playbooks/perf.md`.

Verdict: every source step is either present or dropped with a port-notes
entry, and none is missing. Every old enforced line is present, and the Proof
floor line is byte-identical. The pstack-only tool fence is clean. Every skill
the files name exists in `skills/`, except `verify-app`, which is named on a
condition the cell action prescribes. Both index lines match the contract
grammar. There are three small findings: one is a likely loss of meaning in
hillclimb, and two are inaccuracies in the notes. None blocks the cap.

Labels: **present** means the step is in the new file, in bee's terms where a
tool was swapped. **dropped (noted)** means it is gone and the port notes name
it. **dropped (not noted)** means it is gone and the notes say nothing, which
D2 counts as missing. **missing** means a method step is gone with no reason.

## Old enforced lines (old `perf.md`)

| Old line | New home | Status |
|---|---|---|
| 1 Decision and noise floor written before the first run; stop when spread > effect | `perf-issue.md` step 1, verbatim | present |
| 2 Baseline with the real command or trace; record the number | `perf-issue.md` step 2 (adds the `verify-app` or real-interface surface) | present |
| 3 Hypothesis is a mechanism the baseline shows | `perf-issue.md` step 3 ("State the hypothesis as a mechanism the baseline shows") | present |
| 3 "First ask whether the slow path must exist at all: a deleted path beats a faster one" | `perf-issue.md` step 3, the Elimination family ("Deleting the work beats every other family when it applies") | present, reworded into the pstack text |
| 4 Change ONE thing so the delta has one cause | `perf-issue.md` step 4 | present |
| 5 Re-measure the same way, same command | `perf-issue.md` step 5 (adds "on the same surface") | present |
| 6 Keep the win, discard the loss, record BOTH numbers | `perf-issue.md` step 7 (adds the trail under `docs/history/<feature>/`) | present |
| Sustained target: prove the harness separates the target case, then freeze it | `hillclimb.md` step 2 | present |
| Sustained target: report the median of several runs | `hillclimb.md` step 2 | present |
| Sustained target: stop rule before the first attempt = target + minimum attempts | `hillclimb.md` step 7 (and step 1) | present |
| Sustained target: one row per attempt | `hillclimb.md` step 3 | present |
| Sustained target: revert an attempt that does not clear the noise | `hillclimb.md` step 5, verbatim | present |
| Sustained target: never loosen the stop rule | `hillclimb.md` step 7, verbatim | present |
| "A plateau is not a stop — it means pivot the hypothesis, not give up." | `hillclimb.md` step 6, verbatim | present |
| "It feels faster" is not a result (per D2, decision `1593e365`) | both situation files | present |
| Proof line (baseline, after, delta, command, artifact path) | both situation files | present |
| Proof floor line | `perf.md` line 10, and both situation files | present; the index copy is byte-identical (checked with `diff` of the `^Proof floor` line, old against new) |

The perf class has no routed-principle, statechart or formal-verify lines in
the old text, so D3 had nothing more to carry.

## perf-issue (pstack `perf-issue.md`)

| Source | Status |
|---|---|
| Lead line "You own the measurement story. Plan, review, verify the numbers." | present |
| "Tie every fix to a measurement, don't read source instead of measuring" | present, with a reason added |
| 1 Baseline trace via the matching control skill | present: swapped to the project's `verify-app` or the real interface (noted) |
| 2 `how` to ground hypotheses | present: swapped to `bee-how` as a `bee dispatch prepare --kind gather` (noted) |
| 2 Do not claim a perf ceiling without running it first | present |
| 2 Eight strategy families are hypothesis generators, not a checklist; a family earns an attempt only on its signal; a focused fix beats all eight | present |
| 2 Elimination (incl. "needs the `how` pass, not the profiler") | present, `bee-how` (noted) |
| 2 Divide and conquer | present |
| 2 Caching (name what invalidates it) | present, with a reason added |
| 2 Indirection | present |
| 2 Batching | present |
| 2 Redundancy | present |
| 2 Lazy evaluation | present |
| 2 Scheduling | present |
| 3 Plan the fix from the trace | present (step 4) |
| 3 `architect` first when the fix crosses a function boundary | dropped (noted, slice 3); the rule stays as "write down the contract it keeps before dispatching" |
| 3 Delegate implementation to a subagent on the configured perf-issue model; review the diff | present: `bee dispatch prepare --kind cell` on the code role's model (noted) |
| 3 Capture a post-fix trace | present (step 5) |
| 3 `sequence-verifiable-units` principle; verify each attempt before the next | the citation is dropped (noted); the sentence is present |
| 4 Parse and compare the artifacts (JSON to sqlite, diff); inconclusive or wrong-surface is not a pass; flag it | present (step 6) |
| 5 Cite the measurement in the PR | present: "in the capped result" (noted) |
| 6 Run Opening a PR | present: the bee close and `bee worktree merge` path, with `uat` the owner's (noted) |
| Pointer to Hillclimb for a sustained metric | dropped (noted): the `hillclimb` line on the class index carries it now |
| Reply (baseline, post-fix, delta, artifact path) | present |

## hillclimb (pstack `hillclimb.md`)

| Source | Status |
|---|---|
| Lead line "You own the metric and the experiment's integrity. Supervise and review; delegate the attempts." | present, delegation as `bee dispatch prepare` |
| Trigger examples and "A one-off fix is Bug fix or Perf issue; this is the loop" | present (`bug-fix` and `perf-issue` both exist as situation files) |
| Core discipline: one change, one measurement, keep or revert; never stack untested changes; never claim a win from inspection | present |
| Core discipline: `prove-it-works` principle | the citation is dropped (noted); the reason "the data decides" stays |
| 1 Ground workload and architecture with `how`; name the workload dimensions; pick a case that reproduces the complaint | present, `bee-how` as a gather (noted) |
| 1 If no case reproduces it, fix the repro first | present |
| 1 One metric, its direction, a stop predicate pairing a target with an attempt floor; the 50%/10-iterations example; use the user's numbers or agree them | present |
| 2 Build the harness, prove its sensitivity, freeze it (`build-the-lever`) | present, `bee-principle-build-the-lever` (noted) |
| 2 Contrasting workloads separate; revise if they do not | present |
| 2 One repeatable command; median of N; changing it invalidates earlier numbers | present |
| 2 Baseline metric and green regression gate before any change | present |
| 3 Decision log via `show-me-your-work`: `decision.tsv` with the nine columns | present: the decision trail under `docs/history/<feature>/`, same columns (noted) |
| 3 Read it before each attempt so the search accumulates | present |
| 3 Keep it out of the tree (gitignored) so it survives reverts | present, swapped to "outside the files an attempt touches" (noted); see Finding 1 |
| 4 Ground each hypothesis in a named mechanism | present |
| 5a Hand the change to a subagent on the configured hillclimb model; supervise and review the diff | present: `bee dispatch prepare --kind cell` on the code role's model (noted) |
| 5a `guard-the-context-window` principle | the citation is dropped (noted); the reason "the lead's context stays on the metric" stays |
| 5a Parallel hypotheses fan out to parallel workers, one worktree each (`separate-before-serializing-shared-state`) | present, `bee-principle-separate-before-serializing-shared-state` (noted) |
| 5b Measure before and after with the frozen harness; run the regression gate | present |
| 5c Accept only past noise and with a green gate; otherwise revert in full; no "might help" ride-alongs | present |
| 5d One commit per accepted fix; `git add <files>`, never `-A`; log the row either way | present |
| 5 Each iteration ends in a check (`sequence-verifiable-units`) | the citation is dropped (noted); the sentence is present |
| 5 Unattended: borrow only the wake mechanism from Autonomous run, not its stop rule; a plateau means pivot | present, pointer to `playbooks/session/autonomous-run.md`, which exists (noted) |
| 6 Push past the first plateau: pivot the category, combine near-misses, re-read the source, go radical | present |
| 6 Correctness and simplicity outrank the number; revert a win that breaks behavior; keep a simplification that holds the number (`laziness-protocol`) | present; the citation is dropped (noted) |
| 7 Stop when the predicate is met or the ideas left are marginal; never relax the predicate; do not quit while cheap ideas remain; surface being stuck | present |
| 8 Opening a PR with commits stacked in landing order | present: the bee close and `bee worktree merge` path, in landing order (noted) |
| Reply (metric and target, baseline to final with %, kept vs reverted, one line per fix, `decision.tsv` path, next idea) | present, the trail path in place of `decision.tsv` |

## Tool fence and skill names

- `rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/`
  finds nothing (exit 1).
- A wider search of `perf.md` and `perf/` for arena, architect, interrogate,
  swarm, tdd, figure-it-out, show-me-your-work, `/loop`, `gt`,
  `decision.tsv`, `gpt-`, subagent and the four unported principle names
  finds only the ordinary word "architecture" (`hillclimb.md:14`, `:39`). No
  pstack tool is there.
- Skills named: `bee-how`, `bee-principle-build-the-lever` and
  `bee-principle-separate-before-serializing-shared-state` all exist under
  `skills/`. `verify-app` is not under `skills/`. It lives in
  `.claude/skills/verify-app`, `.agents/skills/verify-app` and
  `.opencode/skills/verify-app`. The new file names it only as "the project's
  `verify-app` skill when it has one, otherwise the real interface". That is
  the swap the cell action prescribes, so it is not a broken step. The
  ppb-12c report rules the same way.
- The MIT notice is in place: `skills/bee-planning/playbooks/LICENSE.pstack`
  exists, and the port notes point to it.

## Index grammar (contract `913115b3`)

Both lines under `## Situations` in `perf.md` match
`` ^- `[a-z0-9-]+` — .+ — delivers: .+$ ``. Each named file exists:

- `` `perf-issue` `` exists as `perf/perf-issue.md`. Its triggers are in English
  and Vietnamese, and its deliverable is set.
- `` `hillclimb` `` exists as `perf/hillclimb.md`. Its triggers are in English
  and Vietnamese, and its deliverable is set.

No `## ` heading follows the section, so it runs to the end of the file. It
holds two blank lines and the Proof floor line. The fence reads only lines
that start with `- `, so the Proof floor line is not a situation line. This
is the same shape as `bugfix.md`. The parity fence passes:
`cargo test --release -p bee --test class_playbook_parity` reports 9 passed,
0 failed.

## Findings

1. **The hillclimb trail can lose rows on a working-tree revert. Likely, not
   run.** pstack kept `decision.tsv` gitignored "so it survives reverts". The
   port moves the trail to `docs/history/<feature>/`, a tracked path, with the
   rule "keep the trail file outside the files an attempt touches". That rule
   protects the trail from `git revert <commit>`. It does not protect rows not
   yet committed from a whole-tree discard such as `git checkout -- .` or
   `git reset --hard`. Step 5 says "revert the change in full", and a
   whole-tree discard is a natural way to do that. This comes from how git
   treats tracked files. I tried to confirm it in a scratch repo under
   `/tmp`, but the bee concurrent-worker git guard refused the `git add`
   there. Suggested fix for the leader: make step 5 say "revert only the
   attempt's paths" (path-scoped), or commit the trail row before reverting.
2. **One port-notes line is inaccurate. Verified.**
   `port-notes/perf.md:29` says 'The "decision trail" of the old step 6'. Old
   step 6 has no decision trail. The trail path is new text added in the port.
   It is harmless, but the notes should call it an addition, not a rename.
3. **Hillclimb repeats the stop rule. Verified, cosmetic.** Hillclimb states
   the stop predicate twice. Step 1 has the pstack form: target plus attempt
   floor, agreed with the user. Step 7 has the old bee form, which opens "When
   the ask is a sustained metric target", even though the whole situation is
   that case. The two do not conflict.

Checked and not filed: the old step 3's "a deleted path beats a faster one"
looked dropped, but the Elimination family carries it. The `bug-fix` and
`autonomous-run` cross-references both resolve to existing files. The Proof
floor line inside the Situations section does not break the fence.
