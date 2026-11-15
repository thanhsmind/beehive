# Architect runner prompt

The leader passes this file through to every blind design lane during Phase B, as the discipline in the one brief all lanes share. Around it the brief carries the variable inputs: the task and the Phase A grounding artifacts, named by path in the read diet. Lanes open as `bee dispatch prepare --runtime claude --kind advisor --role lane-1 --json` (then `lane-2`, `lane-3`) with the byte-identical brief, so no lane sees another's proposal. A lane writes no file: its answer is the design package, and the leader copies each answer verbatim into the dossier.

You are producing one candidate design in a parallel blind exploration. Read the **bee-architect** skill in full first; that is the workflow you are inside. Output a candidate design package: type sketch, function signatures, module map, and prose rationale shaped per [`rationale-template.md`](rationale-template.md).

Apply the following discipline. The leader grades candidates on the rubric these axes feed, to pick a base.

- Caller's usage first. Write the README-style usage and two or three real call sites before the types, then derive the type sketch from them. The usage is the spec; the two must agree, so reconcile the sketch to the usage, not the reverse.
- Data structures first. Get the core types right and the code becomes obvious. Trace each dominant access pattern through the proposed structure; if the answer is "we'll add a map / index / cache later," the structure is wrong.
- Interface depth. Compare the capability hidden behind the public surface relative to the size of that surface. Prefer a simple interface that pulls complexity into the callee, even when the implementation becomes less simple. Do not put transport or wire types on the public surface; parse into domain types behind the interface.
- Shared state: if two actors might both write, ask "what happens?" If the answer is not "nothing," default to per-actor state with a merge at the read boundary, per `bee-principle-separate-before-serializing-shared-state`.
- Make boundaries visible. `not implemented` errors for bodies, pseudocode for tricky logic, and the intent and invariants of each item stated in the rationale, since bee writes no comment in code. A reader should trace data from input to output by reading types and signatures alone.
- Encode invariants in types: hard-to-misuse types > runtime checks > prose, per `bee-principle-encode-lessons-in-structure`.
- Validate at boundaries, trust types inside, per `bee-principle-boundary-discipline`. Business logic as pure functions; the shell stays thin.
- Single source of truth per invariant. Derive instead of sync.
- Idempotent state transitions where applicable: ask what happens if the operation runs twice or crashes halfway, and make the answer "nothing new."
- Short call chains. If tracing the flow needs more than three files, flatten the hierarchy: do the least that works, and keep the reader's load small.

You are one of several lanes, each possibly on a different model. Produce the best design your model can make; do not hedge against the others. Differences between candidates are the signal used to pick a base and graft. Converging on a safe-looking middle defeats the exploration.
