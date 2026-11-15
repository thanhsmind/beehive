# feature

You own the design. Plan, review, verify. Delegate implementation through
`bee dispatch prepare` and stay in the lead, because the lead that writes the
code has no one left to review it.

This is for new or changed behavior, built from a named data shape.

1. Map the affected subsystem with `bee-how`, run as a gather
   (`bee dispatch prepare --kind gather`), before you design anything. A design
   drawn on memory of the code meets the real code at the diff review, where a
   wrong turn costs the most.
2. Name the DATA shape first: the record the feature turns on, before any
   verb, flag or screen reads it. Pick its organizing structure before the
   delegate writes logic: a state machine over scattered booleans, a table or
   registry over branching, a typed model over repeated shape assumptions. Core
   types and the state concurrent actors share are the part that is expensive
   to change later, so they are decided first and decided on purpose.
3. Model an event-order flow with the `statechart` skill, so that TLC walks
   every step order and finds the stuck state a unit test never reaches:
   payment hold, capture and refund with retry, a message queue, a re-run or
   idempotent job, a multi-state workflow, a lock or lease. Do it before the
   flow's code, when the host has that skill. Each break it reports stays a
   guess until a test on the real code fails the same way, because a model can
   disagree with the code for reasons the code does not share. With no
   `statechart` skill on the host, skip this step and say so.
4. Write the throughput checkpoint as four todo items, because the
   decomposition decides whether a fan-out helps or collides. A dimension that
   genuinely does not apply (single file, no fan-out) keeps its item with
   `n/a: <reason>` rather than being dropped, so the skip stays visible.
   - **Blocking first steps.** Gates run before fan-out.
   - **Independent workstreams.** Disjoint files, services or layers
     parallelize. Shared writes serialize.
   - **Shared mutable state.** Default to splitting the target
     (`bee-principle-separate-before-serializing-shared-state`). Serialize only
     for real invariants.
   - **Smallest safe decomposition.** If one worker is best, name why.
5. Delegate the code to a worker started as a cell
   (`bee dispatch prepare --kind cell`) on the code role's configured model,
   with a specific scope: the file paths, the named data shape from step 2 and
   its organizing structure, and success criteria. Review its diff yourself.
   - Build a walking skeleton end to end first, with no stub left on that path,
     because wiring breaks then show up first, while they are still cheap.
   - Build the CURRENT slice only. A cell for a later slice is planned on
     guesses, so headline the rest and leave it out of the cells.
   - Delegation is mandatory, with no skip-with-reason escape, and a wish to
     save lines does not override it: the gain is review separation, not lines
     saved. A lead that cannot dispatch owns the diff directly with the same
     review separation, and never replies that it is standing by for a nested
     agent.
   - Write no comment in the code, in any language. The why goes to a
     `docs/knowledge/` concept or to `bee decisions log`, because a comment
     goes stale beside the code it describes.
   - Edits are surgical. Re-ground against the source for upstream-derived
     files, because a rewrite from memory drifts from what it copies. Port a
     shared-primitive improvement to all its consumers and verify each, because
     a fix in one caller leaves the siblings broken. Commit liberally.
6. Verify on the matching surface, and prove the user-visible path, not only
   the unit sitting under it: the user walks the path, never the unit. Drive
   that surface through the project's `verify-app` skill when it has one,
   otherwise through the real interface itself. An inconclusive run, or a run
   on a different surface, is not a pass; flag it.
7. Sync `docs/knowledge/` in the SAME change when behavior moves. It is the
   state layer, not a write-up, and a concept that lags the code misleads the
   next reader.
8. Keep a model that must stay true as the code moves with the
   `create-formal-verification-skill` skill: it picks the code where a break
   costs most, proves the property can fail, and leaves a project-local verify
   skill under `.bee/verify/<name>/` with the files it covers and its sync
   commit. A change to covered code runs that verify skill; drift that piled up
   across merges takes `maintain-formal-verification-skill`. With neither skill
   on the host, skip this step and say so.
9. Leave small, ordered commits and stack follow-ups: build, verify and commit
   each small unit before the next, so a failure points at one unit and a
   reviewer reads the story bottom-up. Each cap commit ends with the
   `cell: <id>` trailer.
10. Land through the bee path in `playbooks/session/opening-a-pr.md`: the draft
    PR the `ship_visibility` rule opens on the first cap, `bee worktree merge`
    on green, and the `uat` door of `bee close`, which stays the owner's.
    Nothing here marks the PR ready or merges it by hand, because a PR that
    lands itself skips the proof-line and uat checks.

Code-coupled work (one feature, one migration) goes to a single owner with the
checkpoint inline; that owner fans out internally after the blocking phase.
Parent-level fan-out is for slices that produce independent artifacts (audits,
cross-subsystem investigations, competing experiments). Rewrite the checkpoint
at phase boundaries and start a fresh owner rather than chaining interrupts,
because an owner chained across phases carries context the new phase no longer
needs.

A green unit behind a path nobody can walk is not a result.

Reply: what you built, what you chose and why, open decisions. Tables for
design alternatives, because a table lets the reader compare the options on
the same axes.

Proof line: the user-visible path driven, with its evidence (`green:live`).
