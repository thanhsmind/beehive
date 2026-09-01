# Trace And Provenance — Two Named Procedures

Load only after `bee-researching` is active. `Trace` answers "how does this
actually run?". `Provenance sweep` answers "why is this the way it is?".
Both are read-only: the outcome is an account, never a diff.

## Trace

The outcome is a runtime path — the ordered steps a call actually takes, each
anchored `path:line`. A file list is not a trace.

1. State the question as a path: what enters, what runs, what comes out.
2. Pick the entry points from evidence — a CLI verb, an HTTP route, a hook, a
   test that drives the behavior. Two is the FLOOR and four is the CEILING.
   Entry points must be DISJOINT: two names that reach the same first frame
   are one entry point, not two.
3. Fan one read-only worker at each entry point. The ONE door is
   `.bee/bin/bee dispatch prepare --runtime <rt> --kind gather --json`. Run
   that door first, then run exactly the tool and payload it returns. Never
   hand-pick a `subagent_type`, a `model`, or a tier marker — the door
   returns them.
4. One honest entry point means NO fan-out: the trace runs inline, and the
   account says so in one line. Splitting one path into two workers to reach
   the floor is theater.
5. Fold every return into ONE account. The leader writes the path; a worker's
   return is evidence, never pasted output.
6. Name every step the trace could not follow as UNFOLLOWED, with the reason
   — dynamic dispatch, a generated file, a binary, a network call. A gap
   named is data; a gap hidden is a wrong map.

Close with anchors a reader can open: `path:line` per step, or the command
that ran.

## Provenance sweep

The outcome is the reason a thing is the way it is, carried by evidence and
never by memory. Sweep all seven categories, in this order.

1. **Decision log** — `bee decisions search --text "<term>"`. The locked
   agreements, with their ids.
2. **Git history** — `git log -S "<string>"` for the line's birth,
   `git log --follow <path>` for the file's. Read the commit bodies.
3. **Feature history** — `docs/history/<feature>/`: `CONTEXT.md` for the
   locked decisions, `plan.md` for the approach, `reports/` for what shipped.
4. **Knowledge bundle** — `bee knowledge search --text "<term>"`, then open
   the matching files under `docs/knowledge/`.
5. **Code comments** — `rg -n "<symbol>"` to find the OWNING source file, then
   read its comments and doc-comments at the site, not from the snippet.
6. **Tests** — `rg -n "<behavior>" $(fd -t d '^tests$')`. A test name is often the
   plainest statement of intent this repo holds.
7. **External tracker** — `gh issue list --search "<term>"` and `gh pr list
   --search "<term>"`. Often absent here; report it as absent rather than
   dropping the row.

Report rules, which are the point of the procedure:

- A category that returned NOTHING is reported BY NAME as empty.
- A category you did not sweep is reported as UNSWEPT, with the reason.
- An omitted category is the defect this procedure exists to stop. Seven
  rows go in, seven rows come out.

Close with anchors: `path:line`, a commit sha, a decision id, or the command
that ran.
