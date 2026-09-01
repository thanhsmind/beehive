---
name: principle-verify-before-reporting
description: "Apply before you file a suspected defect. Reproduce it, or walk the exact code path by hand — and record the suspicions that failed verification instead of dropping them silently."
---

# Verify Before Reporting

Reproduce or trace every suspected defect before it becomes a finding. Run the
failing input when you can. When you cannot run it, walk the exact path by
hand, line by line, and confirm each step of the failure actually follows.

A dropped suspicion is still a result. When one fails verification, record the
drop: the suspicion in one line, the `path:line` that raised it, and what you
checked — a reason, never a verdict word. "Traced `retry()`; the guard at
`src/net/retry.ts:118` already catches it" is a reason. "Not a problem" is not.

**Why:** a plausible-but-wrong finding makes the author do your verification
for you, and it buries the real findings in the same report. Three verified
defects outrank three verified defects plus seven maybes — the maybes cost the
three their credibility. A silent drop is an invisible filter: the author never
sees the judgement, so nobody can overturn a wrong one.

**Depth:** `.bee/expertise/review.md` § Verify before reporting.
