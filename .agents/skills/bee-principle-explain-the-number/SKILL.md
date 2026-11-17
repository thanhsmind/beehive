---
name: bee-principle-explain-the-number
description: "Apply when measuring or reporting a number — speed, throughput, test counts, token counts, or eval results — before trusting or acting on it."
---

# Explain the Number

Before running a measurement, write down the threshold that would change your
decision and the baseline noise floor. Afterward, ask why the number is not
double and name what bounds it from an actual run's profile or counters, not
from reading code.

List what else the figure could reflect — skipped or cached work, uncounted
errors, an untuned baseline, background noise, or an unrepresentative slice —
and rule each out with evidence. Keep the run count, spread, and limiter
attached to every reported benchmark, test count, token count, or eval result.

**Check:** what bounds this number, what else could have produced it, and do
the run count and spread support the claim?

**Not when:** a rough ballpark was explicitly asked for; even then, verify the
work ran without errors and label it as a single run.

**Why:** an unexplained number invites fitting the story to whatever appeared,
hiding regressions, untuned baselines, or work that never happened.

**Depth:** `.bee/expertise/performance.md` § Measuring without lying to yourself.
