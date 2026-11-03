---
name: bee-principle-attack-the-premise
description: "Apply when a second fix fails the same check the first fix failed. Stop patching: write the premise both fixes share in one sentence and attack it before you try a third fix."
---

# Attack the Premise

Two fixes failed the same gate — the same test, the same reviewer objection,
the same refusal. If both fixes rest on one shared assumption, a third fix
built on that assumption will fail for the same reason.

Stop before the third attempt. Write the premise both fixes share in one
sentence: "both assume the cache owns the expiry time." Then count which
parts of the system hold the imbalance that sentence describes — two writers
of one value, a duty assigned to the wrong component, a check living on the
wrong side of a boundary. Remove that assignment instead of compensating for
it. A retry that fails twice, then a longer timeout that also fails, share
the premise "the call is slow"; the real fault may be that the call is never
made.

Only after the premise is named and attacked does the next fix start.

**Why:** each failed fix built on a wrong premise reads as a near miss, so
the next attempt looks reasonable too. Naming the premise turns a loop of
local patches into one question that can be checked.

**Depth:** `.bee/expertise/debugging.md` § Attack the premise.
