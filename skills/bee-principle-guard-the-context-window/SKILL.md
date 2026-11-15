---
name: bee-principle-guard-the-context-window
description: "Apply when context is filling up: large outputs, long files, repeated reads, fan-out planning. Route bulk to a read-only gather and keep summaries in the main thread, not raw payloads."
---

# Guard the Context Window

The context window is finite and does not refill inside a session. Every token
that enters should earn its place: bulk goes to a delegate, and the main
thread keeps the summary.

Route verbose output, large documents and multi-file hunts to a read-only
gather that returns paths, facts with anchors and quotes only where asked; a
gather goes out as `bee dispatch prepare --runtime claude --kind gather --json`.
Read by relevance: a file the current task does not need stays closed. Keep
what every invocation uses inline in the skill body, not in a file that costs a
read each time. Size the units: cap the files each one touches, and count what
the mechanism itself costs, such as a fan-out whose reports all land back in
the main thread.

**Check:** what is the largest thing about to enter the main thread, and does
the task need it whole or only its summary?

**Not when:** the exact bytes decide the answer — a diff under review, a value
you must prove, a quote you will cite. Read the artifact itself; a summary is
not proof (see prove-it-works). One or two files cost less to read than to
delegate.

**Why:** overflow degrades reasoning, leaves compression artifacts and halts
progress, and context spent inside a session cannot be reclaimed.

**Depth:** `.bee/expertise/thinking.md` § Guard the Context Window.
