# Stage: executing (`bee-swarming` — "Execute")

**Purpose** — Implement, test, and finish *exactly one* parent-assigned cell as a
worker, then return a structured status token. This is the only stage that edits
source.

**When it runs** — Inside a dispatched worker that received an assigned cell id.
`small` and up always runs this way; a `tiny` cell may instead run inline in the
orchestrator's own session.

## Inputs
- The dispatch prompt — the assignment itself: one cell, **already claimed** for
  you, its listed files **already reserved** under your nickname. Everything else
  comes from CLI outputs; when a verb refuses, its message names the fix.
- `AGENTS.md`, then the cell's `CONTEXT.md` and plan (paths in the prompt).
- The assigned [cell](../register.md#beecellsfeature-njson) (`bee cells show --id`)
  and its `read_first` files.

## Outputs
- File edits *within the reserved `files`* of the cell.
- A capped cell whose `trace` carries `{outcome, files_changed, deviations,
  tests}` — `tests: "boundary"` (the declared suite proves later, at `bee
  close`/`bee worktree merge`), or `"undeclared"` in a repo with no
  `commands.test`.
- **One commit per cell**: the subject describes the change in imperative mood, and
  the cell id rides the last line of the body.
- Exactly one status token, first thing in the final message.

## Gate
None — workers never approve gates.

## State touched
[`reservations reserve`](../register.md#beereservationsjson) for any *additional*
path discovered mid-work,
[`bee finish`](../register.md#beecellsfeature-njson) (`cells finish` —
commit-only proof, caps and releases the cell's reservations, all in one
verb; tests prove at the boundary, `bee close`/`bee worktree merge`),
the cell's `trace`, one git commit.

## Key rules
- **Never choose your own cell** or browse the ready list — validate the
  assignment, don't claim it.
- **Conform before you code.** Scout adjacent patterns, reuse existing helpers,
  match the codebase's idiom. Authoring tests? Judge existing coverage first
  (`.bee/expertise/tests.md`).
- **`bee finish` is the completion door, and it is commit-only proof.** It caps
  and releases; tests prove at the boundary — `bee close` runs `commands.test`
  when the feature has no worktree, `bee worktree merge` runs it when it does
  — and a red there refuses that door, carries the failing excerpt, and that
  red is now the next work. There is no proof tier to satisfy, no red-first
  evidence flag, and no `cells verify` step: the cell's own `verify` field is
  plan text MAIN runs once at feature close (`verify_owner`), never the
  worker.
- **Never build on a red base** — a red at close or merge is its own fix-first
  cell. The claim door refuses against a recorded red run unless it carries
  `--fix-first "<reason>"`.
- **Commit before you cap.** A cap on a cell that changed files refuses unless a
  commit in the last 50 carries the trailer `cell: <id>`; `--commit-pending
  "<reason>"` is the one recorded exception. On a `small`+ lane the cap also refuses
  when `trace.worker` names no registered execution worker, unless it carries
  `--inline-reason`.
- **`bee finish` may run from the feature's worktree.** It resolves the cell and its
  claim at the main store while recording commit-only proof in the worktree's own
  directory — every other mutating `cells` verb still refuses there and names the
  main checkout.
- **No stubs, TODO-only, or dead code.**
- **Deviation has four answers**: a bug in touched code → fix it and record the
  deviation; a missing piece the outcome depends on → add it and record; blocking
  breakage in your path → fix and record; anything architectural → `[BLOCKED]`
  with the proposal. Never reinterpret a locked decision to make the cell fit.
  Package installs always checkpoint (`[BLOCKED]`).
- **Return exactly one token**: `[DONE]` (outcome, files, commit) · `[BLOCKED]`
  (what, why, your diagnosis) · `[HANDOFF]` (at ~65% context — write
  `.bee/HANDOFF.json` first) · `[NOOP]` (cell missing or already capped). Never
  wait silently; never ask a blocking question — you run headless.

## Source
`skills/bee-swarming/SKILL.md` ("Execute") + `references/worker-details.md`;
craft in `.bee/expertise/tests.md` and `.bee/expertise/debugging.md`
