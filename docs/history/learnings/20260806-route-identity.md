---
date: 2026-08-06
feature: route-identity
categories: [workflow-state]
severity: medium
tags: [record-identity, monotonic-rules, carry-forward, triage, ratchets]
---

# route-identity — a monotonic rule needs to know whose history it is guarding

## What Happened

`bee state start-feature` resets the gates, the phase, the mode, the summary
and the next action — and carries everything else on the record forward,
including the previous feature's route. The route object had no feature of its
own, so the never-demote rule then read a finished feature's `high-risk` lane
as *this* feature's history and refused to record an honest `small`. Two
one-file bugfixes ran the day this was found under a label they never earned,
and there was no override flag.

The route now carries the feature it was triaged for; a feature start drops
the one it inherited; and the demote check only fires against a route stamped
with the same feature.

## What Was Learned

**A monotonic rule is a ratchet, and a ratchet with no scope ratchets the
world.** "Never demote" is exactly right inside one feature: it stops a team
from quietly re-labelling risky work as cheap once the ceremony gets annoying.
Across a feature boundary it means the highest lane anything ever reached
becomes the permanent floor for everything after it. Every never-goes-back
rule needs an explicit answer to *"never goes back for whom, and until when?"*

**A record without identity inherits whatever it is stored beside.** The route
lived under a key on the active record and travelled with the record, not with
the feature. The sibling code path had it right by accident: `start_lane`
builds a fresh record, so lanes never inherited a stale route, while
`start_default` mutates the record in place. When two paths do the same job
and only one builds fresh, the other is carrying something — the question is
only what.

**A reset that enumerates what it clears will miss the next field.**
`start_default` clears six named keys. Every future key added to the record is
opt-in to the reset, and silence is the default. Clearing the whole record and
re-adding what survives inverts that default; where that is too big a change,
the enumeration deserves a test that fails when a new key is added, not a
comment asking people to remember.

**Fix the old data, not just the new writes.** Stamping the feature at write
time would have left every already-recorded route unstamped and still
ambiguous. Treating an unstamped route as *no route* makes the pre-fix records
correct too, and costs one comparison.

## Evidence

- Cell `rti-1`, commit `68beab21` — `start_default` drops the carried route,
  `validate_route_set_flags` stamps its feature, `run_route` gates the
  transition check by it, and `validate_route_lane_transition` keeps its
  signature so its seven existing tests stand unchanged.
- Measured: after `start-feature --feature feature-swap-door`, `route --show`
  still returned worktree-reclaim's route (`lane=high-risk`, `files=7`,
  `updated_at 2026-08-05T10:07:35Z`).
- Behavior captured in
  `docs/knowledge/areas/workflow-state/sessions-lanes-and-identity.md` (R80a).
