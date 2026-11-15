---
name: bee-principle-exhaust-the-design-space
description: "Apply when facing a novel UI interaction or architectural decision with no precedent in the codebase. Build 2-3 competing prototypes or sketches and compare them side by side before committing."
---

# Exhaust the Design Space

When the right answer is not obvious, build two or three competing prototypes
or sketches, compare them side by side, and only then commit. Design it twice
is this rule by another name. A second flavor of the first shape does not
count: the candidates must differ in shape, not in names or polish.

It applies to a novel interaction with no prior art in the codebase, an
architectural choice with several viable approaches, and a product decision
whose answer depends on feel and not on logic.

The candidates go out as blind lanes, each as
`bee dispatch prepare --runtime claude --kind advisor --role lane-1 --json`,
then `--role lane-2` and `--role lane-3` for a third, and the judge as
`--kind reviewer`. `bee blind check` checks the blind brief before the lanes
run. With no lanes available, write the second candidate yourself as a
deliberate break from the first.

**Check:** which two or three alternatives did you compare, and what makes each
a different shape rather than a variation of the first?

**Not when:** the pattern is already established, the work is a bug fix or a
refactor with a clear target state, or the constraints leave one viable
approach. Running three candidates to confirm what is already fixed only
spends the fan-out.

**Why:** building the wrong thing costs more than exploring three options.

**Depth:** `.bee/expertise/thinking.md` § Exhaust the Design Space.
