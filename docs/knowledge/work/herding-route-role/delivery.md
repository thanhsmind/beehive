---
type: bee.delivery
title: herding-route-role — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-route-role: 7 capped cell(s), 15 recorded deviation(s)."
timestamp: 2026-09-09
bee:
  id: herding-route-role-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-route-role/CONTEXT.md, docs/history/herding-route-role/plan.md]
  sources: [docs/history/herding-route-role/CONTEXT.md, docs/history/herding-route-role/plan.md, .bee/cells/archive/herding-route-role/hrr-1.json, .bee/cells/archive/herding-route-role/hrr-2.json, .bee/cells/archive/herding-route-role/hrr-3.json, .bee/cells/archive/herding-route-role/hrr-5.json, .bee/cells/archive/herding-route-role/hrr-6.json, .bee/cells/archive/herding-route-role/hrr-7.json, .bee/cells/archive/herding-route-role/hrr-8.json]
---

# herding-route-role — Delivery

## What shipped

- **hrr-1** — route registered as the control loop's fourth role across all eight closed sites, with an enumerated read-and-announce tool surface (1 file(s) changed)
- **hrr-2** — route-prompt.md written as the route role's whole contract, 218 lines, slice 1 scope only (1 file(s) changed)
- **hrr-3** — bee-herding SKILL.md now describes four roles; route's paragraph, protocol pointer and never-list added, frontmatter corrected (1 file(s) changed)
- **hrr-5** — route tool surface widened to slice 2's five needs, verb by verb, with two new forbidden tokens (1 file(s) changed)
- **hrr-6** — route-prompt.md now carries the whole slice-2 contract: carve-out, producer resolution, different-agent rule, in-review marker, reviewer pane label, four verdict paths, D4 hand-off, BLOCKED stop (1 file(s) changed)
- **hrr-7** — merge role honors the in-review marker and closes the <slug>-review pane (1 file(s) changed)
- **hrr-8** — the review carve-out is named where the rule lives: one sentence in AGENTS.md, a full boundary in operational-invariants.md (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hrr-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee -- control_loop`
- **hrr-2** — `test -f skills/bee-herding/references/route-prompt.md && rg -q 'role-merge.md:73-76' skills/bee-herding/references/route-prompt.md && rg -q 'bee herding interlock' skills/bee-herding/references/route-prompt.md && rg -q 'lines 200' skills/bee-herding/references/route-prompt.md && ! rg -q '^1\. `phase` is' skills/bee-herding/references/route-prompt.md`
- **hrr-3** — `rg -q 'route' skills/bee-herding/SKILL.md && ! rg -q 'The three roles' skills/bee-herding/SKILL.md`
- **hrr-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee -- control_loop`
- **hrr-6** — `rg -q 'agents-review-user-invoked' skills/bee-herding/references/route-prompt.md && rg -q 'bee-herding.review' skills/bee-herding/references/route-prompt.md && rg -q -- '-review' skills/bee-herding/references/route-prompt.md && rg -q 'approved' skills/bee-herding/references/route-prompt.md && rg -q 'waiting-on' skills/bee-herding/references/route-prompt.md && ! rg -q 'SLICE 1 ONLY' skills/bee-herding/references/route-prompt.md`
- **hrr-7** — `rg -q 'bee-herding.review' skills/bee-herding/references/role-merge.md && rg -q -- '-review' skills/bee-herding/references/role-merge.md && rg -q 'bee-herding.red' skills/bee-herding/references/role-merge.md`
- **hrr-8** — `rg -q 'route' AGENTS.md && rg -q 'gate_bypass' AGENTS.md && rg -q 'route' skills/bee-herding/references/operational-invariants.md && rg -q 'bee-herding.review' skills/bee-herding/references/operational-invariants.md`

## Deviations

- **hrr-1** — Capped with --sync-ack: siblings in the same parallel wave touched skills/** in the shared worktree.
- **hrr-1** — The worker delivered hrr-4 scope (the shipped-route-prompt contract test) inside this cell; hrr-4 is dropped rather than re-done.
- **hrr-1** — sync-ack: parallel wave: siblings hrr-2 and hrr-3 landed skills/bee-herding/references/route-prompt.md and skills/bee-herding/SKILL.md in the same shared worktree. hrr-1's own commit 7275d515 carries control_loop.rs alone.
- **hrr-2** — Capped with --sync-ack: a sibling cell in the same parallel wave touched skills/bee-herding/SKILL.md in the shared worktree between this commit and this cap.
- **hrr-2** — sync-ack: parallel wave: hrr-3 landed skills/bee-herding/SKILL.md in the same shared worktree between this cell's commit and its cap, so the door sees a sibling's touched path. hrr-2's own commit 0efa2de4 carries route-prompt.md alone.
- **hrr-3** — Capped with --sync-ack: a sibling cell in the same parallel wave touched skills/bee-herding/references/route-prompt.md in the shared worktree.
- **hrr-3** — sync-ack: parallel wave: hrr-2 landed skills/bee-herding/references/route-prompt.md in the same shared worktree, so the door sees a sibling's touched path. hrr-3's own commit 45ecdce1 carries SKILL.md alone.
- **hrr-5** — Capped with --sync-ack: siblings in the same parallel wave touched skills/** in the shared worktree.
- **hrr-5** — sync-ack: parallel wave: siblings hrr-6/7/8 landed skills/** and AGENTS.md in the same shared worktree. hrr-5's own commit f3acc88f carries control_loop.rs alone.
- **hrr-6** — Capped with --sync-ack: siblings in the same parallel wave touched other skills/** paths and AGENTS.md in the shared worktree.
- **hrr-6** — sync-ack: parallel wave: siblings landed control_loop.rs, role-merge.md, AGENTS.md and operational-invariants.md in the same shared worktree. hrr-6's own commit 4951bb6d carries route-prompt.md alone.
- **hrr-7** — Capped with --sync-ack: siblings in the same parallel wave touched other skills/** paths and AGENTS.md in the shared worktree.
- **hrr-7** — sync-ack: parallel wave: siblings landed control_loop.rs, route-prompt.md, AGENTS.md and operational-invariants.md in the same shared worktree. hrr-7's own commit 9d76eed2 carries role-merge.md alone.
- **hrr-8** — Capped with --sync-ack: siblings in the same parallel wave touched other skills/** paths in the shared worktree.
- **hrr-8** — sync-ack: parallel wave: siblings landed control_loop.rs, route-prompt.md and role-merge.md in the same shared worktree. hrr-8's own commit 85436d0b carries AGENTS.md and operational-invariants.md alone.

## Provenance

Proposed by `bee knowledge promote --work herding-route-role` from 7 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-route-role/CONTEXT.md`, `docs/history/herding-route-role/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
