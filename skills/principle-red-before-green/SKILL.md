---
name: principle-red-before-green
description: "Apply when you fix a reported bug or a failing behavior, before you write the fix. Reproduce the bug as a test, watch it fail for the reported reason, then fix — and write 'green' only beside fresh output."
---

# Red Before Green

A bug fix starts with the test, not the fix. Write a test that reproduces the
report, run it, and watch it fail for the reported reason — the same wrong
value, the same error. Only then write the fix, and re-run for the green.

The same honesty covers the report. Write "green", "passing", or "fixed" only
beside output from a run you just made. "Tests should pass now" is a
prediction, not a result. If you have not run it, say so.

**Why:** a test written after the fix has never failed, so it proves nothing
about the bug. It can assert the wrong thing, exercise the wrong path, or pass
vacuously. The observed red is the evidence that this test detects this bug;
the green that follows is the evidence the fix works.

**Depth:** `.bee/expertise/tests.md` § Red before green.
