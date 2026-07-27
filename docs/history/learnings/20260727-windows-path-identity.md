---
date: 2026-07-27
feature: windows-path-identity
categories: [cross-platform, proof-discipline, review-layering, orchestration]
severity: high
tags: [path-identity, ci-red, false-equal, exclusion-list, unprovable-locally]
---

# windows-path-identity — a four-day CI red, one real bug, and two claims that did not survive contact

Windows CI had been red for four days while main CI was green on the same commits. The diagnosis said one mechanism explained everything. Most of that held; two central claims did not, and the difference is the whole lesson.

## What Happened

**The diagnosis was right about the mechanism and wrong about its reach.** Path-string identity genuinely explains the three newly-failing suites: a version-control tool always emits one separator form while the runtime emits the platform's, and one platform is additionally case-insensitive. One of the three was a real product bug — a worktree failing to resolve because two strings naming the same directory were compared byte-for-byte, which cascaded into a merge never reaching its verify child and a suite timing out.

But the plan then claimed the four *standing* exclusions were the same cause, citing the workflow comment that lists them. Validation read that comment and refuted the claim from its own text: one of the four carries four failure classes, of which exactly one is path identity. A second has an entirely independent cause — its write-failure simulation skips when running as root, a condition that never fires on the platform in question, where the permission call it relies on does not prevent the write at all. Two more have no diagnosed cause on record; attributing them to path identity was assertion.

**The gate meant to protect the exclusion removal was vacuous.** The plan required each suite to pass locally before its exclusion came off. Validation measured all four passing locally — of course they did; they were excluded *because* the other platform differs. A local pass is definitionally not evidence about them. Applied literally, that gate would have shipped both undiagnosed causes into CI as a surprise red.

**The fix's first version created the bug it was preventing.** Normalizing both sides folded one separator character into the other on every platform, on the belief that the character cannot appear inside a filename. Where it can, a directory literally named with it compared **equal** to a genuinely different nested directory, and the identity check then examined the wrong location. Three review passes and the worker's own red-first proof missed it. A judge asked one specific question — construct a case where two genuinely different things compare equal — and found it in minutes.

**And one thing could not be proven here at all.** The corrected assertions use a canonical comparison where they used exact equality. On this platform those two forms agree by construction, so reverting an assertion is a semantic no-op and no local test can see it. The judge said so and marked it a decision rather than a fix. I overrode that and spent a round adding a second assertion pass per site; the worker implemented it faithfully; I then measured that reverting the original assertion still passes, because the second pass is a separate line. The judge was right.

## Root Cause

1. **A citation was treated as evidence for more than it says.** The workflow comment was quoted as proof that four suites share one cause, while the same comment enumerates additional causes for one of them. Reading a source for the claim you want is how a plan inherits an error and lends it authority.
2. **A gate can be shaped so it cannot fail.** "Passes locally" was applied to suites whose entire reason for exclusion is that local is not where they fail.
3. **Every proof written for a loosening points the same direction.** Red-first evidence shows the fix does something; coverage shows the new path runs; neither asks whether the change now accepts something it must not. The inverse question is not a regression of old behaviour — it is a new acceptance, and nothing in the diff looks like one.

## Recommendation

1. **When a document is your evidence, quote the part that contradicts you too.** If it does not contradict you, say that explicitly. The refutation here came from the first paragraph of the source the plan already cited.
2. **Test your gate against the state it is meant to catch before you rely on it.** Run the check on the failing case: if it passes there, it is not a gate. "Green locally" gates nothing about a platform-specific red.
3. **For every comparison you loosen, write the negative control first** — two things that must stay distinct which the new implementation might conflate. If you cannot name one, the normalization is not yet understood. Interrogate each step for what it destroys: case folding destroys case, separator folding destroys a character that may be data, link-following destroys the alias/target distinction.
4. **Separate what the platform decides from what the medium decides.** Separator meaning is a platform property; case behaviour belongs to the volume. Conflating them produced errors in both directions here — a fold applied where it should not be, and an assumption made where the answer should have been asked for.
5. **Ask reviewers the inverse question by name.** "Find a case where this says equal and must not" is what three prior passes were not asked and the fourth was.
6. **When a property cannot be proven on the machine you have, name the limit and stop.** Do not spend a round manufacturing a proof shape that cannot exist. Keep the coverage that is genuinely real — here, that the canonical comparison accepts a value exact equality rejects — and let the platform's own run establish the rest.

## What shipped

The one diagnosed product bug is fixed, with the comparison's separator handling delegated to the platform's own resolver, case decided per volume with both sides agreeing, zero identity treated as absent, and every ambiguity resolving to "different". Three test assertions no longer encode a single-platform assumption. Two workflow steps that pointed at files which do not exist — masked for as long as the job failed earlier — now resolve, and a new check validates that every workflow step's script path exists, proven by replaying it against the pre-fix workflow where it found both. **No exclusion was removed**, and each remaining cause is filed with what is actually known about it, including the two for which the honest entry is that nobody has diagnosed them.
