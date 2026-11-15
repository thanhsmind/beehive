# refactoring

You own the contract. The structure changes; the behavior does not. For
"refactor", "rename", "extract", "inline", "dedupe", "restructure", "move this
module", "tidy up this area". Distinct from a feature, which adds behavior,
and a bug fix, which corrects it.

A refactor that smuggles in a behavior change loses its safety net, and a red
found later has many suspects. A behavior change is not a refactor — it is a
separate cell. If the cleanup reveals a missing feature or a real bug, split
it out and ship the structural change first against the pinned contract. A
redesign is allowed, but name it and route it to the feature class. Large or
cross-cutting structural work, such as a migration across many call sites or a
coordinated reshape of many subsystems, is not this playbook: it belongs to
`bee-figure-it-out`, and this playbook is the focused-to-medium change.

1. Pin the behavior contract first. Run `bee-how` over the affected subsystem
   as a gather (`bee dispatch prepare --kind gather`) to learn the contract,
   then record existing behavior — a characterization test, a snapshot or an
   equivalence harness — before any structure moves. The record makes
   "refactor" a checkable claim, because without one nothing can say the
   behavior held. If the area has no coverage, write the pin before touching
   structure. A type check or a lint is not a record.
2. Prove that record green on the UNCHANGED tree, because a red base hides new
   reds.
3. Name the structure the code is missing: a state machine over scattered
   booleans, a table or registry over spread-out branching, a typed model over
   repeated shape assumptions, a reducer over ad hoc mutations. Boring code
   stays when the shape is already clear and local, because the reshape must
   delete branches or invalid states, not add indirection.
4. Name the target shape: what the module layout, types and call graph should
   be if built today, because a move with no target is churn. If the target
   crosses a function boundary, run `bee-architect` for parallel design
   exploration of the shape before the move, and write down the contract it
   keeps before dispatching, so the workers share one definition of
   "unchanged".
5. Subtract first (`bee-principle-subtract-before-you-add`) — every later step
   then moves less: an early cell deletes dead weight, collapses one-caller
   wrappers, drops redundant validators and removes orphan references before
   the new shape is introduced. The smallest change that reaches the target
   shape ships, and a speculative cleanup that "might help" gets reverted, not
   left to ride, because every line kept is a line the next reader holds.
6. Move in small behavior-preserving steps, each keeping the record green, so
   that a red points at one step. For an API move, migrate every caller and
   delete the old API in the same slice
   (`bee-principle-migrate-callers-then-delete`) — two live APIs drift apart —
   with no compatibility shims and no parallel old-and-new paths. Then search
   strings, prose and back-references for the old name, because a rename
   silently misses usages there; spot-check every rename against the actual
   files. Dispatch the mechanical edits as a cell
   (`bee dispatch prepare --kind cell`) with a specific scope — the file
   paths, the names being moved, the behavior to hold — and review the diff
   yourself.
7. Never edit the record, the harness or the baseline to make a step green — a
   record bent to pass no longer records anything. If the pin looks wrong,
   stop and fix it as its own change, on its own evidence.
8. Prove behavior is unchanged on the real artifact, not "it compiles". For
   larger reshapes, run an equivalence check: a script that diffs old and new
   outputs, a recorded baseline replayed against the new code, or a smoke run
   on the matching surface, driven through the project's `verify-app` skill
   when it has one, otherwise through the real interface itself. Own the
   verification yourself: a delegate's "looks good" summary is a claim, not
   evidence.
9. Keep the diff only if it lowers what a reader must hold in mind — that is
   the whole payoff of a refactor. The measure is reduced reader load: fewer
   layers between question and answer, less hidden state, fewer indirections
   without a second consumer. If the diff does not lower reader load
   somewhere, revert it.
10. Order the commits so they tell the story: a subtraction commit, then the
    reshape, then any follow-on cleanup, so a single revert undoes one slice
    and each behavior-preserving slice stays green before the next. Then land
    it through the bee close and worktree merge path: the work is capped,
    merged by `bee worktree merge`, and the `uat` acceptance at `bee close`
    stays the owner's.

Reply: the structure that changed, the pin you held it against, the
equivalence proof, the reader-load delta, what shipped and what got reverted.
No new behavior.

Proof line: the record, green before the change and green after it.
Proof floor: the cap refuses a result weaker than this class's floor and names it; the table is the cap's own (`bee cells finish --help`).
