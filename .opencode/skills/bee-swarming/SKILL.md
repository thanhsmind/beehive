---
name: bee-swarming
description: >-
  Run approved cells to done — orchestrate bounded workers over gate-approved cells, or execute exactly one assigned cell inside a dispatched worker. Use when the merged shape+execution gate is approved and current-slice cells are open, or when running as a worker that received an assigned cell id.
metadata:
  version: '0.2'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: unavailable
      reason: Both roles drive the work through the vendored bee binary. The binary is vendored into the repo by onboarding; no Node runtime is involved.
---

# Swarming — run the approved work

A **worker** has an assigned cell id in its dispatch prompt; everyone else
is the **orchestrator**. `bee orient` shows where the work stands either way.

## Orchestrate

You launch workers and tend results; you do not implement. The wave runs
inside the feature's worktree (rule: agents-worktree-first). Claiming from
main is control-plane and fine; execution depends on transport:

- **Native subagents** (Agent tool, Task tool) inherit the session's OS
  cwd. From main they cannot write into the worktree — the write guard
  refuses. Enter the worktree first (EnterWorktree, or a session/pane
  opened at the worktree path) and dispatch from there.
- **External herding workers** (bee herding run with --cwd) receive an
  explicit cwd at process start. The leader can stay in main while the
  worker writes in the worktree — no manual entry required. A `tiny` cell may run inline in this
session; `small` and up always dispatches — one worker per cell
(`references/swarming-reference.md` ("Single execution worker in full")),
parallel by default: disjoint cells fan out concurrently (reservations
prove it, 3-4 live workers cap it), serial needs a named file conflict.
From two cells up, state the one-line concurrency plan before dispatching.

1. `bee cells schedule --json` sets dispatch order — override only with a
   stated reason. Overlapping-file cells are fixed by scope or
   reservations, never by "spawn both carefully".
2. `bee dispatch wave --runtime <rt> --feature <f>` prepares the current
   wave of one feature in one call — claim, reserve, and payload per
   ready cell, refusals landing in `skipped` with a typed reason instead
   of aborting the batch (`--feature` omitted only resolves from a bound
   session lane or the default record's own feature; nothing resolving is
   a typed refusal, never an every-feature grab — pass `--limit <n>` to
   bound how many cells of the wave are claimed). One cell, or a cell
   needing its own worker name, takes
   `bee dispatch prepare --cell <id> --worker <name> --runtime <rt> --claim`
   instead. The dispatcher may compose an Expertise section for the worker leader-style via `--expertise` (one entry per line, `<path> :: <purpose> :: <read-to>`), choosing from bee's own skill references and knowledge files; optional and judgment-driven, never auto-derived. A user-facing cell carries its mapped feature file that way — `.bee/verify/verify-app/features/<feature>.md` with its last-modified date, so the worker can judge a stale map. The cell already declares the job that
   selects its model (`role`, required at `cells add`) — escalate onto the
   session model only where the work earns it
   (`bee cells escalate --id <id>`; rubric:
   `references/swarming-reference.md`).
3. Spawn with exactly that payload — a whole wave goes out in ONE message,
   one tool call per cell. Never paste session history; never hand a
   worker two cells.
4. Tend: read each worker's Result form (the fenced
   `{outcome, commit, files, tests, deviations}` block its prompt
   requires), never its prose. Silence is not failure — inspect
   `bee cells list` and `bee reservations list` before assuming stuck.
5. On `[DONE]`: the worker's word is never the evidence. Goal-check on
   smell; `bee cells judge` for undeclared-file hits. At
   `standard`/`high-risk`, every `behavior_change` cell owes a
   `bee cells judge-record` verdict or `bee close` refuses (`judge-debt`) —
   run the slice judge before you reach for close.
6. Slice clean: `bee close --feature <slug> --dry-run` names every
   remaining door with the command that settles it; the final slice runs
   `bee close --feature <slug>`, which checks every capped cell's recorded
   proof line — same check `bee worktree merge` runs when the feature has
   a worktree. Doors check proof, they never run tests themselves; a cell
   with no valid proof is the remaining door. Doors are never waived.

**`[BLOCKED]` rescue ladder:** (1) re-dispatch the same cell with the
missing context; (2) `bee cells escalate --id <id>` and re-dispatch — the
session model is this session, so that rung hands the blocker to you;
(3) surface it to the user with the worker's diagnosis. If it invalidates the plan, return to bee-planning.

**Completion:** slice done with more approved work remaining → return to
bee-planning for the next batch IN THE SAME TURN — the slice boundary is
never a user question, never a "say go" (an approved plan stays frozen;
planning shapes the next batch, never reopens it). Final slice green → the road
splits on `uat_stop`:

- Default `uat_stop: "close"` (absent means this): merge FIRST, without
  asking — `bee worktree merge` from main on the FEATURE's own branch is
  the publish-for-testing step; green caps are its precondition, not a
  user question. The worktree is kept. Then hand the user the reloaded
  product on main — what changed, how to run or see it — and ask for uat
  there. Mark the wait:
  `bee state waiting-on set --kind gate --subject "uat: <feature>"`.
  A failed uat is fixed in the worktree and merged again; the approval
  (`bee gate --name uat --approved true`) unlocks `bee close`.
- `uat_stop: "merge"`: put the work where the user tests it BEFORE any
  merge: `bee staging add --feature <slug>` plus the staging build, when
  the host repo records `commands.staging_build`; fall back to presenting
  the feature worktree itself when it does not, or when
  `"staging_before_merge": false` makes `bee staging add` refuse
  `STAGING_DISABLED`. Present what changed, how to run or see it, and the
  fixed question "Ready to merge?" — never merge on your own read of
  green tests. Mark the wait the same way. After the user approves uat
  (`bee gate --name uat --approved true`), land with `bee worktree merge`
  from main on the FEATURE's own branch — never staging's — which refuses
  `WORKTREE_MERGE_UAT_PENDING` for `standard`/`high-risk` features until
  that approval.

Either placement: a green merge that finds a staging record carries the
trigger-3 nudge `staging_rebuild_suggested: "bee staging rebuild"`, run
or suggested next. Capture is recorded as pending (bee-capturing runs
later, at the owner's pace).
Before declaring done: no active reservations, no in-flight workers
recorded.

The 65%-context handoff holds mid-wave (rule: agents-context-handoff-65). When a unit finishes and approved work remains, continue
in-session; finishing a unit is never a reason to stop.

## Execute (worker)

Your dispatch prompt is the assignment: one cell, claimed for you, its
listed files reserved under your nickname. Everything else comes from CLI
outputs — when a verb refuses, its message names the fix.

Two contract guards can refuse the cell before you run it — at the claim
door and again at `dispatch prepare --kind cell` — both typed, both
mutating nothing, both naming their own remedy: `CONTRACT_RETIRED` /
`CONTRACT_UNSETTLED` when a decision the cell cites has left the active set
or carries a `waiting`/`due` trigger, and `CONTRACT_UNCITED` — the mint
trap — when a test-writing cell cites no `contract:<name>` decision at all.
A `cell.decisions` entry that resolves to no store decision (a local `D1`
pointing into a CONTEXT.md table) is passed over silently, never refused
(`docs/knowledge/areas/workflow-state/dispatch.md`).

The prompt carries the user's original request VERBATIM under a
DO-NOT-PARAPHRASE header. No layer may replace or paraphrase it, yours
included: add guidance beside those words, never over them.

1. Read `AGENTS.md`, then the cell's `CONTEXT.md` and plan (paths in the
   prompt). Conform before you code: scout adjacent patterns, reuse
   existing helpers, match the codebase's idiom. Authoring tests? Judge
   existing coverage first — `.bee/expertise/tests.md`.
2. Implement exactly the assigned cell. Reserve any additional path before
   writing (`bee reservations reserve`). Package installs, any new
   dependency (vendored or declared), and architectural changes are not
   yours to make — `[BLOCKED]` with the proposal. A
   contract or API change the cell did not name is the orchestrator's
   call — `[BLOCKED]` with the options. Trading data quality or the
   user's experience for a technical target (speed, a green test, a
   smaller diff) is never a worker's trade — `[BLOCKED]` with the
   options.
3. When reality disagrees with the cell: a bug in touched code → fix it,
   record the deviation; a missing piece the outcome depends on → add it,
   record; blocking breakage in your path → fix, record; anything
   architectural → `[BLOCKED]`. Never reinterpret a locked decision to
   make the cell fit. Disagreeing with the cell is a RECORD, never a
   silent workaround — `bee cells dissent --id <cell> --reason "<claim>"
   --alternative "<instead>" --severity blocker|consider`, then
   `[BLOCKED]`. An unexpected red or an unfamiliar mechanism
   mid-cell is a pull moment: `bee knowledge search --text "<symptom>"`
   surfaces matching patterns and area concepts before you guess.
4. Commit once: subject describes the change in imperative mood; the last
   line of the body is the literal trailer `cell: <id>` — a bare id alone
   fails the cap.
5. `bee finish --id <cell> --outcome "<one line>" --files <a,b>
   --report '<json>'` — cap and release in one verb, `--report` REQUIRED
   and carrying the same Result form you return (`{outcome, commit,
   files, tests, deviations}`), which finish validates key-for-key onto
   the trace. `tests` is a proof line `<command> — <result> — <scope
   reason>`: pick the proof your change type needs (code → related tests
   green; docs → parity/pointer checks; behavior → judge verdict;
   user-facing surface → drive its mapped feature and inspect the result,
   evidence attached, `green:live`), run it yourself, and record it — a
   `red` result refuses the cap. `bee close` and `bee worktree merge`
   check that recorded proof at the boundary; they run nothing
   themselves. CI runs the full declared command on every push — the
   one deterministic net.
6. Return exactly one token, first thing in your final message, and the
   Result form beside it — never in place of it:
   `[DONE]` (outcome, files, commit) · `[BLOCKED]` (what, why, your
   diagnosis) · `[HANDOFF]` (the 65% handoff — handoff file
   written before the token; rule: agents-context-handoff-65) · `[NOOP]` (cell missing or already capped). Never wait
   silently; never ask a blocking question — you run headless.

## Hard rules (both roles)

- Never spawn before the execution gate is approved; the orchestrator
  never edits source in a `standard`/`high-risk` wave.
- One cell per worker; the claim guard refuses a worker that claims, browses, or self-selects.
- Conflicts are fixed in scope or reservations, never by being careful.
- Never build on a red base — a red becomes its own fix-first cell (rule: agents-never-build-on-red).

## Headless

`bee-hive` ("Headless") governs; waves run without check-ins, and an
unrescuable blocker becomes an `Outstanding Questions` entry.

## References

| File | When to load |
|---|---|
| `references/swarming-reference.md` | Role rubric and escalation, worktree dispatch transaction, prompt template details, result formats |
| `references/worker-details.md` | Deep worker mechanics: finish and its refusals, advisor consult, friction triggers |
| `.bee/expertise/tests.md`, `.bee/expertise/debugging.md` | Authoring tests; hunting a red |
| `.bee/expertise/INDEX.md` | The cell is domain work — stored data, a caller-facing contract, a trust boundary, a rollout, a speed budget, a surface people use: route from the index, load exactly one |
