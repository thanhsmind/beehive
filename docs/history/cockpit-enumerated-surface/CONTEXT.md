# Cockpit Enumerated Surface — Context

**Feature slug:** cockpit-enumerated-surface
**Date:** 2026-09-01
**Shaping session:** complete
**Scope:** Quick — the target is already fixed by a locked decision
**Domain types:** RUN

## Feature Boundary

The two cockpit control panes — dispatch and merge — stop carrying a wildcard
over the whole bee CLI and carry an enumerated command surface instead, the way
the observer already does. It ends at the tool strings and their tests; it
changes no role prompt, no loop mechanics and no other privilege.

## Why this is a defect, not a proposal

`herding/control_loop.rs:277-286` grants both panes
`Bash(.bee/bin/bee:*)` and `Bash(git -C:*)` — the entire CLI including
`worktree merge` and `state gate`, plus arbitrary git against any repository on
the machine. The rules that keep those panes in their lane are prose, read by a
cold model every sixty seconds.

The locked owner decision says otherwise, verbatim
(`.bee/decisions.jsonl`, herding-adopt D7-FINAL):

> the four working agents keep `--permission-mode bypassPermissions` as an
> explicitly accepted risk with its blast radius recorded; **the two CONTROL
> panes are narrowed to an enumerated command surface**, never to "read-only".

The pattern to copy already sits twenty lines below in the same file:
`SUPERVISOR_ALLOWED_TOOLS` enumerates verb by verb, and a test asserts the
forbidden tokens can never reappear. Its own doc comment names the gap — "why
the `Bash(.bee/bin/bee:*)` wildcard the cockpit roles carry is deliberately
absent here."

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Both cockpit panes get an enumerated `Bash(.bee/bin/bee <verb>:*)` surface, never a CLI wildcard. Neither is narrowed to read-only — they still start work and land merges respectively. | The owner's D7-FINAL, applied. |
| D2 | The two surfaces are DIFFERENT, and the difference is the point: dispatch may not merge, may not approve a gate, and may not write with git; merge may. | Dispatch's own role prompt never invokes `worktree merge`, `state gate` or a writing git command; merge's does. Splitting the surface is what makes "merge is an owner gesture" mechanical instead of prose. |
| D3 | The surface is derived from what the role prompts actually invoke, and every entry must be traceable to a call site in `role-dispatch.md` or `role-merge.md`. A string that merely appears inside a message the pane sends, or inside advice addressed to a worker, is NOT a call site. | Two false positives were already found this way: `bee gate` in role-merge is text inside a chat message, and `bee cells finish` there is advice to a worker. |
| D4 | A forbidden-token test guards each surface, mirroring the observer's. Narrowing without that test is a rule with no owner. | bee's own pattern: a guard that tests one state is a law with a hole. |
| D5 | `herding enable` (and `disable`) are EXCLUDED from both cockpit surfaces, and the forbidden-token test names them. A dispatch pane must not be able to arm its own interlock. | Two locked records already say so in prose and nothing enforced it: `herding-dispatch-lock-toggle D4` — "CLI verbs stay owner-typed only, never called by bee automation" — and area rule R8, "the enable marker has two equivalent human-typed forms, never an automated one". Excluding the verb turns both into a mechanical fact. This is a second violation found while fixing the first. |

### Agent's Discretion

- The exact verb list, provided every entry traces to a call site per D3.
- Whether `git -C` stays for dispatch as read-only subcommands or is dropped.

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/herding/control_loop.rs` — `allowed_tools_for`
  (the four cockpit arms) and `SUPERVISOR_ALLOWED_TOOLS` plus its
  forbidden-token constant and test, which is the shape to copy.

### Established Patterns

- Enumerate a privileged surface verb by verb and assert the prohibition by
  test, never by comment — the observer's existing treatment.

## Evidence gathered before shaping

Every `bee` invocation in each role prompt, counted:

- **dispatch** — `herding pane` (36), `herding agent-start` (5), `worktree new`
  (4), `worktree list` (3), `herding result` (3), `cells list` (3), `status`
  (2), `orient` (2), `herding record-worker` (2), `herding occupancy` (2),
  `herding interlock` (2), `herding classify-lane` (2), `backlog pbi` (2),
  `reservations list` (1), `herding enable` (1). It never invokes
  `worktree merge`, `state gate`, or a writing git command.
- **merge** — `herding pane` (15), `worktree merge` (6), `worktree new` (3),
  `worktree list` (2), `orient` (2), `staging rebuild` (1),
  `herding agent-start` (1). Its `bee gate` and `cells finish` hits are a chat
  message and worker advice, not invocations.
- git — dispatch reads only (`rev-parse`, `status`); merge writes
  (`git -C <main-root> merge`).

## Outstanding Questions

### Resolve Before Planning

None.

### Deferred To Planning

None. The one open question — whether `herding enable` belongs on dispatch's
surface — was answered during shaping and became D5: it does not, and two
locked records already said so with nothing enforcing them.

## Deferred Ideas

None.

## Handoff Note

CONTEXT.md is the source of truth. The risk to design against is not that the
surface is too wide — it is that a too-narrow surface refuses the dispatch pane
in an unattended run with no human present to widen it.
