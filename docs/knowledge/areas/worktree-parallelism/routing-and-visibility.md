---
type: bee.area
title: "Worktree Parallelism — when to take a worktree at all, and what an occupied checkout says out loud"
description: "The prose routing rule that sends new feature work into a worktree only when an occupied checkout makes it worth it, the lane-first refinement that defers the grant to Gate 3, and the notices an ungranted worktree and a denied write print so isolation is never silently absent."
timestamp: 2026-07-22
bee:
  id: worktree-parallelism-routing-and-visibility
  lifecycle: active
  areas: [worktree-parallelism]
  required_context: [areas/worktree-parallelism/entering-creating-and-registering.md]
  decisions: ["worktree-session-routing D9 (the routing rule is prose, not a hook)", D9a (a live cross-session heartbeat plus a non-idle phase in the shared store), "cross-worktree-holds D7 (lane-first: the grant is taken at Gate 3, and only on genuine file overlap)", "worktree-ux (2026-07-21, GH #30/#31 — the ungranted-worktree notice and the containment-deny message)"]
  sources: [docs/history/worktree-session-routing/, "docs/specs/worktree-parallelism.md#S-routing-rule-d9-prose-not-a-hook"]
  authoritative_for: "worktree-parallelism: the routing rule, its lane-first refinement, and worktree visibility notices"
---

# Worktree Parallelism — Routing and Visibility

Everything else in this area answers "how does a feature worktree work?". This concept
answers the question that comes first: should there be one at all? The answer is a rule
that lives in prose and is enforced by the guards that already existed — plus the notices
that make sure a session never believes it is isolated when it is not.

## Routing rule (D9 — prose, not a hook)

When a session is about to start NEW feature work in a checkout that already has another live
session's active work — a live cross-session heartbeat plus a non-idle phase in the shared
store (D9a), or active holds / live-owner lanes — the paved road is `worktree new` and opening
the next session in the printed path. Docs-lane work, tiny fixes, and release machinery stay
in the MAIN checkout (release always runs in main). The rule lives in bee-hive's Session Scout
and AGENTS.md critical rule 13; the existing guards (holds, live-owner lanes, gates) keep
enforcing the hard parts.

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

**Lane-first refinement (cross-worktree-holds D7, 2026-07-20):** exploring, planning, and
validating do not touch source — a new feature in an occupied checkout starts as a per-feature
LANE on the shared store (full live coordination: claims, reservations, holds all visible),
and a worktree grant is taken only at Gate 3, and only when the feature's execution genuinely
overlaps files with other in-flight work. Most parallel work never needs the worktree at all;
per-module test suites + suite auto-discovery (see `verify-pipeline.md`) removed the
artificial overlaps that used to force the choice early.
