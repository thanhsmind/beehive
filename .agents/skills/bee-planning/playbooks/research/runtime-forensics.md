# runtime-forensics

You own the diagnosis. Instrument the live process; do not theorize from
source, because source shows what the code does and never what the running
process is doing. This is for "why is X leaking, spinning or slow at runtime":
heap growth, an idle-but-busy process, an intermittent glitch. The deliverable
is a cited diagnosis, not a fix, so the outcome is an account and the work
stays read-only.

1. Capture the live signal on the matching surface, because the symptom lives
   there and not in a unit. Drive that surface through the project's
   `verify-app` skill when it has one, otherwise through the real interface
   itself. Capture a real artifact, not a guess: a CPU profile for a spinning
   process, a heap snapshot for a leak, a trace for a visual glitch. A
   diagnosis with no artifact behind it is an opinion.
2. Reduce the artifact to the smoking gun: the function on the hot path, the
   retainer chain from the leaked object to a GC root, the loop firing without
   input. Reduce a large artifact through a gather dispatch
   (`bee dispatch prepare --kind gather`) and keep only the reduced finding in
   the lead's thread, because the raw artifact fills the context the lead needs for judging
   (`bee-principle-guard-the-context-window`).
3. Prove the mechanism before believing it. Inject instrumentation into the
   running process, or hotfix the live code without reloading, to confirm the
   hypothesis cheaply. A plausible but unconfirmed cause can be wrong while the
   real one sits one layer over. Without a before-and-after pair, label the
   cause a hypothesis.
4. Map the finding back to source: file, symbol, and the line that allocates
   or schedules, as `path:line`, because a diagnosis a reader cannot open in
   the code cannot be acted on.

Hand back the diagnosis with no fix unless asked. Once the cause is known, the
fix re-routes to `bugfix`, or to `perf` when the cost is a number to move,
because this class is read-only and the change belongs to a class that proves
it.

Reply: the signal captured, the reduced finding, how you proved the mechanism,
the source location, artifact paths.

Proof line: the artifact path, the reduction's command, and the source anchors, each one opened or run.
