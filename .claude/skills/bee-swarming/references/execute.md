# Execute — the worker contract

Read this when you are the worker for one assigned cell id. The orchestrator side lives in `SKILL.md`.

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
   `[BLOCKED]`. An open choice (`must_haves.open_choices`) may be questioned
   with evidence through that same `bee cells dissent`; a conflict with a
   hard constraint (a prohibition or a locked decision) stays a
   locked-decision `[BLOCKED]`. An unexpected red or an unfamiliar mechanism
   mid-cell is a pull moment: `bee knowledge search --text "<symptom>"`
   surfaces matching patterns and area concepts before you guess. A
   product question only the human can answer parks your cell, never the
   lane: `bee cells park --id <cell> --question "<question>"`, then return
   `[PARKED]`.
4. Write your knowledge delta in the same commit as your code, per the
   contract in `packages/bee/prompts/worker-cell.md`: the worker brief
   prints the knowledge rules (the same list `bee knowledge consolidate`
   prints). Update the owning area concept, add a pitfall pattern if you
   hit one, and update `bee.anchors` for files you changed — thin and
   anchored, no review step. The cap answers which concepts changed a
   decision with `--knowledge-used '<path> — <what it changed>'` or
   `--no-knowledge-used`, never refused. A loaded concept that misled the
   cell and that the commit did not fix goes in with `--knowledge-wrong
   '<path> — <what was wrong>'`, omitted when there is none. Before the commit, run `bee
   knowledge check --json`. For every suspect concept anchored by a file
   you changed, fix the concept or run `bee knowledge confirm --id <id>`,
   then reserve the concept file and commit it with the cell — the cap door
   refuses otherwise.
5. Revert any edit the proof did not need. Commit
   once: subject describes the change in imperative mood; the last line of
   the body is the literal trailer `cell: <id>` — a bare id alone fails
   the cap.
6. Run the finish command in your dispatch prompt — cap and release in
   one verb, `--report` REQUIRED and carrying the Result form you return
   (the dispatch prompt holds that form). Its `tests`
   proof line is yours to pick (code → related tests green; docs →
   parity/pointer checks; behavior → judge verdict; user-facing surface →
   drive its mapped feature and inspect the result, the evidence locator
   named in the scope reason, `green:live`), run and record — a `red`
   result refuses the cap. For
   `green:live` on a `behavior_change` cell, the scope reason names the
   before-state you saw on main. `bee close` and `bee worktree merge`
   check that recorded proof at the boundary; they run nothing
   themselves. CI runs the full declared command on every push — the
   one deterministic net. When the dispatch prompt carries a Principles
   block, the cap names each principle you applied and the decision it
   changed (`--principle-used '<slug> — <decision>'`), or says plainly it
   changed none (`--no-principle-used`).
7. Return exactly one token, first thing in your final message, and the
   Result form beside it — never in place of it:
   `[DONE]` (outcome, files, commit) · `[BLOCKED]` (what, why, your
   diagnosis) · `[HANDOFF]` (the 150k-token or 65% handoff — handoff file
   written before the token; rule: agents-context-handoff-65) · `[PARKED]` (question, cell id) · `[NOOP]` (cell missing or already capped). Never wait
   silently; never ask a blocking question — you run headless.

## Tester contract

A code cell has two workers: the implementer writes the code and owns the
existing test edits its change forces; a separate tester, dispatched after
the implementer caps and sharing no context with it, verifies the result
independently and adds the new tests. The tester is a `role: test` cell
that depends on the implementer's cell.

- A tester never edits product code. It writes tests, runs them, and caps
  at or above the proof floor of its cell; the floor table is the cap's
  own (`bee cells finish --help`), never copied into a brief.
- A red result is a finding, not a fix: the tester returns `[BLOCKED]`
  with the red output and its diagnosis.
- The leader answers a red tester by reopening the implementer cell and
  dispatching a fresh implementer. That prompt carries the tester's
  `[BLOCKED]` block as a prior round. The tester re-runs after the new
  implementer caps.
- The tester's cap answers the principle question like any other cell's.
