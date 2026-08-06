---
date: 2026-08-05
feature: feature-swap-door
categories: [workflow-state]
severity: high
tags: [runtime-cutover, delegation-stubs, doors, scribing-debt, git-archaeology]
---

# feature-swap-door — a delegation stub outlives the runtime it delegates to

## What Happened

`bee state set --feature <other>` refused every live-feature swap with
"unsupported command shape". The cause was not a broken door but a missing
one: two branches in `set_gate.rs` detected a swap and returned the
"not-mine" signal so the JS runtime would answer it. That runtime was deleted;
`main.rs` turns the same signal into a flat refusal. The door was restored
natively — same shared debt counter as the close door, both escapes, a waiver
that names the abandoned feature.

## What Was Learned

**A delegation stub is a dangling pointer, and deleting the target does not
delete the pointer.** The stub kept compiling, kept its comment explaining
where the work happens, and read as intentional. Nothing in the build could
notice that "hand this to the other runtime" had become "refuse". When a
runtime, service, or module is retired, the stubs pointing at it are the
migration's real inventory — and they are found by grepping for the *handoff
signal*, never by reading what still compiles.

**The old defect report was stale; the code under it was not.** The open cell
here (`p3-1`) said the swap door asked "two of three debt questions". By the
time anyone returned to it, the other two doors had been deliberately deleted
(decision `412e9b3a`, test-simple), so the fix as written would have restored
questions the project no longer wants. The finding was worth keeping; its
prescription was worth re-deriving. **Age a finding's remedy, not its
evidence.**

**Recover deleted behavior from git, never from memory.** Every message string
and the after-write waiver discipline came from
`git show 5c62cad0^:packages/bee/bee.mjs`. Reconstructing a refusal message
from a description produces text that reads right and matches nothing users
have seen. The port cost one `git show`; inventing it would have cost the
wording.

**A waiver record must name the subject that was harmed, not the subject the
record now points at.** After a swap the routing record holds the NEW feature,
so reusing the close door's waiver — which reads the feature off the record —
would have logged the wrong name every time. The two doors share a counter and
share escapes, but they cannot share the sentence.

## Evidence

- Cell `fsd-1`, commit `41d8b0e6` — six tests over `run_set_body` covering the
  refusal, the three non-swap shapes, the waiver decision, the shared
  deferral escape, and the two lane-path regressions.
- Recovered original: `bee.mjs` at `5c62cad0^` — `featureSwapGuardScribingDebt`
  (:3215), its caller (:3051-3060), the post-write waiver (:3143-3154).
- Behavior captured in
  `docs/knowledge/areas/workflow-state/gates.md` ("The swap wall asks the same
  question the close wall asks").
