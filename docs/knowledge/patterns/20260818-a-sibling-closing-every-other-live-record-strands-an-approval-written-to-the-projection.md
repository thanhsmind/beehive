---
type: bee.pattern
title: A sibling closing every other live record strands an approval that only reached the projection
description: "A gate approved for a lane writes the lane projection, but the merge door reads the live workflow record and falls back only to the default record when it tracks the same feature. Once a sibling session closes every record but its own, a lane has no live record left — so the approval lands somewhere no reader looks, the door keeps refusing, and neither message names the split. Two sessions hit it independently within twenty minutes on 2026-08-18."
timestamp: 2026-08-18
bee:
  id: pattern-20260818-sibling-close-strands-projection-approval
  lifecycle: active
  areas: [workflow-state, worktree-parallelism]
  sources: ["start-feature-reservation-scope: bee gate --name uat --approved true --lane start-feature-reservation-scope reported success and wrote approved_gates.uat true to .bee/lanes/start-feature-reservation-scope.json, while bee worktree merge kept refusing WORKTREE_MERGE_UAT_PENDING; the feature's workflow record wf-a39091bc read status closed with gates.uat.approved false (2026-08-18)", "uat-stop-placement: the same wall reached about twenty minutes earlier on a different feature and a different session, resolved with the documented --skip-uat escape plus a recorded deviation", "uat_merge_precheck, packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs (prefers the live workflow record's gates.uat.approved, falls back to the default record's approved_gates.uat ONLY when that record is presently tracking this feature)", "bee state workflows close --all-but-active — closes every live record whose feature differs from the calling context's own active feature"]
  polarity: pitfall
---

An owner approved a merge. The gate command reported success. The merge
still refused, naming the gate as unapproved. Nothing in either message
was false, and nothing in either message explained the contradiction.

The split: a **lane** feature keeps its state in two places — the durable
per-feature record, and the lane file that projects it. Approving a gate
for a lane writes whichever of the two is reachable; the merge door reads
the durable record first and only consults a second record when that
record happens to be tracking this same feature. While a live record
exists the two agree, so the split is invisible. It becomes visible the
moment the record stops being live.

Any session may close every live record except its own active one. That
is deliberate housekeeping and it does exactly what it says — but its
blast radius includes **other sessions' unfinished features**. A feature
whose record a sibling closed still has a lane file, still routes, still
answers `status`, and still accepts a gate approval. The approval simply
lands where no reader looks. Waiting longer never helps: nothing
reopens a closed record on its own.

**Recognize it by the shape, not the verb:** a write reports success, a
reader keeps reporting the pre-write state, and the two disagree about
*which* copy of the state is authoritative. Whenever a durable record and
a projection of it can be approved through the same command, ask which
one the blocking reader consults before believing the success line.

**Recovery, and why this order:** wind the lane back to a terminal phase,
start it again so a live record exists, then re-record every gate that
mattered — including the human's approval, with the human named as the
actor, because a re-record is bookkeeping and must never quietly become a
self-approval. Only then merge. The escape hatch that skips the door is
legitimate when a human really did approve, but it merges without ever
repairing the split, so the next merge on that feature walks into it
again.

**The rule:** housekeeping scoped to "everything that is not mine" is
scoped to other people's work. Before a verb closes, prunes, or releases
by that rule, it owes the question of what a live sibling still needs from
what it is about to close — and any state a sibling can invalidate must
not be the only place an approval can land.
