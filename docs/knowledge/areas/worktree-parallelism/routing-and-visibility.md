---
type: bee.area
title: "Worktree Parallelism — when to take a worktree at all, and what an occupied checkout says out loud"
description: "The prose routing rule, now worktree-first: code-touching feature work branches into its own worktree at feature start, the main checkout takes only integration, docs-lane, release, and solo-tiny work, the old lane-first grant deferral is superseded — plus the notices an ungranted worktree and a denied write print so isolation is never silently absent."
timestamp: 2026-07-22
bee:
  id: worktree-parallelism-routing-and-visibility
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [areas/worktree-parallelism/entering-creating-and-registering.md]
  decisions: ["worktree-first (docs/specs/worktree-first.md, 2026-07-31, owner-approved)", "worktree-session-routing D9 (the routing rule is prose, not a hook)", D9a (a live cross-session heartbeat plus a non-idle phase in the shared store), "cross-worktree-holds D7 (lane-first: the grant is taken at Gate 2's execution component — the old standalone execution gate, folded in by validation-diet D2 — and only on genuine file overlap; superseded by worktree-first)", "worktree-ux (2026-07-21, GH #30/#31 — the ungranted-worktree notice and the containment-deny message)"]
  sources: [docs/specs/worktree-first.md, docs/history/worktree-session-routing/, "docs/specs/worktree-parallelism.md#S-routing-rule-d9-prose-not-a-hook"]
  authoritative_for: "worktree-parallelism: the worktree-first routing rule, its exemptions, and worktree visibility notices"
---

# Worktree Parallelism — Routing and Visibility

Everything else in this area answers "how does a feature worktree work?". This concept
answers the question that comes first: should there be one at all? Since worktree-first
(2026-07-31) the answer is yes by default — the rule still lives in prose and is enforced
by the guards that already existed — plus the notices that make sure a session never
believes it is isolated when it is not.

## Routing rule (D9 — prose, not a hook)

**Code-touching feature work branches at feature start (worktree-first —
docs/specs/worktree-first.md, 2026-07-31, owner-approved).** Recording a route with any
code-touching lane in the main checkout makes the worktree the loud, machine-named next action —
`bee worktree new --feature <slug>`, sibling dir `<repo>--wt--<slug>` on branch
`wt/<slug>` — and the session opens there. An occupied checkout is no longer the trigger;
the worktree is the default home for the feature, occupied or not. The MAIN checkout takes
only integration, docs-lane work, release machinery, merges, and reading (release always
runs in main). Docs-lane work and a `tiny` fix may stay in main only while no other live session is
present (heartbeat + non-idle phase, D9a); with one, each takes a worktree like any feature. The
explicit owner override is `--in-main` at feature start, recorded as a decision — never
silent. Landing stays `bee worktree merge` from main. The rule lives in bee-hive's Session
Scout and the AGENTS.md boundary list; the existing guards (holds, live-owner lanes,
gates, the main-checkout write guard) keep enforcing the hard parts.

**Visibility (worktree-ux, 2026-07-21, GH #30/#31):** `bee status` inside an UNGRANTED
linked worktree prints a loud notice (text + `worktree_notice` in JSON) that the tree
SHARES the main checkout's store — same feature/phase/claims, no isolation — naming both
remedies (`worktree new` from main, or `worktree register` for the existing tree); granted
worktrees and ordinary checkouts are byte-unchanged. `worktree new` success output carries
an explicit `next_step`: open a session with cwd at the created path; merge back later via
`worktree merge`. A write denied by containment that targets a granted sibling worktree
names that worktree and both remedies instead of the generic containment text (message
only — the deny itself is unchanged; any grants-read error falls back to the generic
message, never an allow).

## Contention is triage data, never a user question (worktree-first-enforcement, 2026-08-16)

Cross-worktree same-path leases are advisory by design — a hard deny needs the holder's
`workspace_id` to match the acting session's (`hooks/write_guard/checks.rs:366-374`) — so
two worktrees may edit the same file and queue at `bee worktree merge` instead of at
reserve time. What that leaves open is semantic overlap: a backlog item whose files an
in-flight cell already holds is likely to be swallowed by that cell, and doing it anyway
buys a merge conflict plus duplicated work. The doctrine (AGENTS block, Multi-session
etiquette): overlap is handled by the agent, not escalated — disjoint items first, a
natural scope split to the disjoint files second, and the overlapped remainder deferred
with a recorded reason and one report line. The user is asked only when the deferred set
is the entire explicit ask. Herding's dispatch role applies the same rule before spawning
(role-dispatch.md §7, "Rank overlap-aware"): candidates overlapping held files are skipped
for the iteration, never dispatched into a known collision.

## What the write guard actually judges (cells wtf-1, wtf-3, wtf-4, 2026-08-12)

The routing rule is prose, but the main-checkout write guard is what makes it bite, and it
now judges the record the acting session is really working under — the **lane record** for a
lane-bound session, which is the shape almost every live session has, not the default
pipeline's `state.json`, which routinely names a different feature entirely. Judging the
wrong record let a code-touching write pass because some unrelated feature happened to sit on
an exempt lane.

The guard has two arms with different reach:

- **The granted arm** — the acting feature already holds a worktree — is phase-independent and
  exempts only docs-lane work. A source write in main while the feature's own worktree stands
  is refused whatever the phase.
- **The no-grant arm** — the acting feature holds no worktree at all — is the arm the guard
  never had. It fires only during execution, only on a routed non-docs non-tiny lane, and only
  in a real git checkout, so exploring, planning, docs and solo-tiny work in main are untouched.

Both arms fail open rather than refuse with a remedy that would itself be refused: any grant
the guard cannot resolve produces an allow, never a deny. That has a known cost — a single
unresolvable entry anywhere in the grants registry disables the no-grant arm for **every**
feature, not just the broken one. This is deliberate: a stale registry must never manufacture
a false refusal on work that is legitimately in main.

**Lane-first refinement — superseded (worktree-first, 2026-07-31):** cross-worktree-holds D7
(2026-07-20) deferred the worktree grant to Gate 2's execution component (the old standalone
execution gate, folded into Gate 2 — validation-diet D2) and took it only on genuine file overlap,
on the argument that exploring and planning do not touch source. Worktree-first inverts
that: the grant is taken at feature start, before any gate and whether or not files overlap
— isolation is the default, not an escalation. The shared-store coordination the lane model
built (claims, reservations, holds all visible across checkouts) is unchanged and still
carries cross-worktree visibility; what is gone is the deferral itself.
