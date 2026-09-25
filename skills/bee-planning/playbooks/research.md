# research

1. Read-only: the outcome is an account, never a diff.
2. Trace the runtime path, not just the file list.
3. Name every source searched that came up EMPTY.
4. End with anchors a reader can open — `path:line`, or the command that ran.
5. A question that chooses between alternatives ends in a recommendation and
   a tradeoffs table. An answer that leads to a change re-routes to `bugfix`
   or `feature`.

Both flows have ONE home: `bee-researching/references/trace-and-provenance.md`
— § "Trace" for step 2, § "Provenance sweep" for step 3.

**When the symptom is live at runtime.** Capture a real profile or trace.
Reduce it through a gather dispatch (`bee dispatch prepare --kind gather`).
Confirm the reduction with one instrumented run, and map it to `path:line`.
Without a before-and-after pair, label the cause a hypothesis.

This is the investigation route (per D3, decision `f1ffa7bd`): the existing
`research` class, no new route and no new lane. Nothing yet ENFORCES step 1 —
read-only is craft here, not a guard (backlog `p-69bee217`).

Proof line: the account's anchors, each one opened or run.
