---
name: bee-principle-reproduce-first
description: "Apply when a bug is reported and you do not yet have it happening on demand. Get the exact command, input and wrong output first — before you read any implementation or form a theory."
---

# Reproduce First

A bug you cannot trigger is a bug you cannot verify fixed. Before you open the
implementation, make the failure happen on purpose: the exact command, the
exact input, the exact wrong output. "Sometimes fails on save" is a report, not
a repro — your first deliverable is a way to make it fail every time.

The repro is also the finish line. "I changed something and the report stopped
coming in" is not a verified fix. "The repro failed before the change and
passes after it" is.

**Not the same as `principle-red-before-green`.** This one is about getting the
bug to happen at all, by any means — a shell command, a curl, a click path.
That one is about the shape of the proof once you have it: a test that fails
for the reported reason before the fix exists. Reproduce first, then red before
green.

**Why:** without a repro every later step is guesswork. You cannot narrow the
search, you cannot tell a fix from a coincidence, and you cannot tell the
report's bug from a different one you happened to find.

**Depth:** `.bee/expertise/debugging.md` § Reproduce first.
