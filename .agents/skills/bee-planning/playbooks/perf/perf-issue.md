# perf-issue

You own the measurement story. Plan, review, verify the numbers. Dispatch the
trace and the fix through `bee dispatch prepare` and stay in the lead. Tie
every fix to a measurement and do not read source instead of measuring,
because source shows what the code does, never what it costs.

1. Before the first run, write down the result that would change the
   decision and the noise floor — the run-to-run spread on an unchanged
   build. Stop when the spread is larger than the effect: a number with no
   decision written before it gets fitted to whatever it shows.
2. Capture a baseline trace with the real command, on the surface where the
   cost was felt. Drive that surface through the project's `verify-app` skill
   when it has one, otherwise through the real interface itself. Record the
   number: without it, no later number means anything.
3. Ground the hypotheses with `bee-how` over the slow path, run as a gather
   (`bee dispatch prepare --kind gather`), because the trace says what is slow
   and only a read of the design says why. Do not claim a perf ceiling without
   running it first, because an unrun ceiling is a guess. State the
   hypothesis as a mechanism the baseline shows. Most fixes come from eight
   strategy families. Use them as hypothesis generators, not a checklist: a
   family earns an attempt only when the trace shows the signal it names, and
   a focused fix for the dominant cost beats applying all eight.
   - **Elimination.** The cheapest work is work that does not run. Before
     optimizing the hot path, ask whether it needs to exist: a computation
     nobody consumes, a feature gate that is always off for this user, a sync
     that redundantly mirrors state, a legacy path kept "just in case". The
     trace shows what is slow, never that it is deletable, so this family
     needs the `bee-how` pass, not the profiler. Deleting the work beats every
     other family when it applies.
   - **Divide and conquer.** The dominant cost scales with input size. Split
     the work so each piece touches less (chunk, shard, prune the search
     space) or so independent pieces run in parallel.
   - **Caching.** The same computation or fetch repeats on identical inputs.
     Store and reuse the result; name what invalidates it before claiming the
     win, because a cache with no invalidation rule serves stale answers.
   - **Indirection.** The hot path does expensive work a cheaper intermediate
     could absorb: an index instead of a scan, a queue that shifts work off
     the interactive thread, a handle that lets a cheaper implementation swap
     in. Add the hop only when it removes more from the critical path than it
     adds; a layer that sits on the hot path without removing work is pure
     cost.
   - **Batching.** Many small operations each pay a fixed overhead (RPC,
     query, syscall, draw call). Coalesce them to pay the overhead once per
     batch.
   - **Redundancy.** The wait hangs on one slow instance or attempt.
     Duplicate the work (replicas, hedged requests, speculative execution) and
     take the fastest result. This trades extra load for lower tail latency,
     so the trace has to show the wait dominates and the system has headroom;
     duplication without that tradeoff only adds load.
   - **Lazy evaluation.** Cost lands on results that are never used or not
     needed yet (eager init on the boot path, rendering offscreen items).
     Defer the work until first use.
   - **Scheduling.** The work must happen, but not during the interactive
     moment. Move it to where nobody is waiting: idle callbacks, a background
     warmup after boot, precompute before the user arrives, cleanup after the
     frame commits. Distinct from Lazy (later when needed): Scheduling often
     runs the work earlier than the hot moment, or in its shadow. The win is
     perceived latency, so measure the interactive path, not total work done.
4. Plan the fix from the trace, and change ONE thing, so that the delta has
   one cause. When the fix crosses a function boundary, run `bee-architect`
   first and write down the contract the fix keeps, because a boundary change
   decided in the diff alone is decided without a second design to compare.
   Dispatch the implementation as a cell
   (`bee dispatch prepare --kind cell`) with a specific scope, and review the
   diff. Verify each attempt before trying the next, so a regression points at
   one change.
5. Re-measure the same way, with the same command on the same surface, so that
   the delta measures the change and not the method. Capture a post-fix trace.
6. Parse and compare the artifacts (JSON to sqlite, diff). An inconclusive run,
   or a run on a different surface, is not a pass; flag it, because only the
   same-surface comparison shows this cost is gone.
7. Keep the win, discard the loss, and record BOTH numbers in the feature's
   decision trail under `docs/history/<feature>/` — a loss on record stops the
   next attempt from trying it again.
8. Cite the measurement in the capped result, then land it through the bee
   close and worktree merge path: the work is capped, merged by
   `bee worktree merge`, and the `uat` acceptance at `bee close` stays the
   owner's, because the measurement is what the owner accepts.

"It feels faster" is not a result (per D2, decision `1593e365`).

Reply: baseline number, post-fix number, delta, artifact path.

Proof line: the baseline, the after, the delta, the command, and the artifact path.
Proof floor: the cap refuses a result weaker than this class's floor and names it; the table is the cap's own (`bee cells finish --help`).
