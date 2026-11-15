---
name: bee-principle-prove-it-works
description: "Apply after you finish a task, before you declare it done. Verify against the real artifact — run the feature, read the actual value, inspect the diff — not a proxy, a self-report or 'it compiles'."
---

# Prove It Works

Before a task is called done, ask how you would prove it works, then check the
real thing directly. Do not infer from a proxy: a build that passes, a file's
age, an output that looks fresh, a cached screenshot, or an agent's account of
what it did.

For code, building it is necessary and never sufficient. Run it, exercise the
actual feature path, and follow the data from input to output; for an
integration, test the whole path end to end. When a check fails, suspect the
way you observed before you suspect the system. When you delegated the work,
open the artifact — the diff, the file, the running behavior — not the
delegate's summary.

The strongest proof is a script that re-runs the same comparison: write it,
run it, and keep its output where a reviewer can re-run it.

**Check:** what did you run or read to see this work, and is it the artifact
itself or something derived from it?

**Not when:** the real artifact cannot be reached from here. Then name the
closest check you did run, say it is a proxy, and write no "done" beside it.

**Why:** unverified work has unknown correctness, and the indirect check feels
cheaper than the direct one; acting on a wrong inference costs far more than
looking at the source.

**Depth:** `.bee/expertise/tests.md` § Prove it works.
