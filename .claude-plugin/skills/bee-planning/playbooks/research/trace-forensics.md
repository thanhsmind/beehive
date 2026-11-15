# trace-forensics

You own the diagnosis from the artifact. Load it, shape it, narrow to the
cause, attribute to source. This is for a dropped `.cpuprofile`,
`Trace-*.json.gz`, `Spindump.txt` or `.heapsnapshot` paired with "why is this
slow, unresponsive, leaking or crashing". The deliverable is a cited
diagnosis, not a fix, so the outcome is an account and the work stays
read-only.

This differs from `research/runtime-forensics.md`, which instruments the live
process. Here the capture already exists. The artifact is a fixed dataset:
read it, do not re-run it, because a re-run is a new capture and answers a
different question. Keep the tooling generic so the playbook stays portable: a
DevTools or trace parser for a cpuprofile and a `.json.gz`, a text editor for
a spindump, your heap tooling for a heapsnapshot.

1. Identify the format and load it with the right tool, because a wrong parser
   reads a different structure and reports it with confidence. Load a large
   artifact through a gather dispatch (`bee dispatch prepare --kind gather`)
   and keep the reduced finding in the lead's thread, because the raw artifact fills the context the lead needs for judging
   (`bee-principle-guard-the-context-window`).
2. Transform the raw artifact into a form you can query. Dump the trace or heap
   snapshot into sqlite, one row per sample, frame or node, and reach the
   queryable shape before you read, because scrolling a raw dump finds what
   catches the eye and not what holds the time.
3. Narrow to the cause. Query for the frames that hold the most time and walk
   the call tree to the hot path. For a leak, follow the retainer chain from
   the leaked object to a GC root. For a spindump, find the thread stuck
   on-CPU or blocked, and its wait reason. A query names the cause and a skim
   only suggests one.
4. Attribute to source. Map the hot frame to file, symbol and line through the
   artifact's own symbols. A frame with no source mapping is not yet a
   diagnosis: resolve the symbols, or say plainly that the artifact does not
   carry them, because a diagnosis nobody can open in the code cannot be acted
   on.
5. Confirm against a paired capture when you have one. Diff a before and after
   artifact so the attribution is the real regression and not background
   noise. Without one, mark the finding as the strongest hypothesis the
   artifact supports, not a confirmed cause, because a single capture cannot
   rule out that the cost was always there.
6. Hand back a cited diagnosis with no fix unless asked. Route to `bugfix`, or
   to `perf` when the cost is a number to move, once the cause is known,
   because this class is read-only and the change belongs to a class that
   proves it.

Reply: the artifact and format, the reduced finding, the source location, the
artifact paths, and whether a paired capture confirmed it.

Proof line: the artifact path, the query that reduced it, and the source anchors, each one opened or run.
