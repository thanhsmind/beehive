# Worktree entry routing — Context

Date: 2026-09-05
Lane: tiny; two instruction sources; solo main-checkout exemption

## Request

“theo nguyên tắc viẹc chuyển phải rất mượt tại sao bị những lỗi chặn ghi này, xem lại phần này fix cho hoạt động chuẩn hơn, tôi nghĩ luật ghi này có thể đang sai”

The user asks to fix worktree transition friction, including checking whether write restrictions are wrong.

## Decision

D1 — 4e2185b0-31e1-4412-874c-7cb4fdedc7d3: Keep canonical write containment. Select the runtime transport before requesting manual entry. Native workers inheriting the parent cwd need a worktree-rooted parent. External herding workers can start in the worktree using explicit cwd. Lane binding does not move a process. Verify the actual child cwd and delivery receipt. Manual entry is a fallback for an identified unavailable or failed capability.

## Evidence

The main session's attempted absolute-path Write into the granted sibling worktree was refused. This is expected containment, not an approval failure. A read-only evaluation of the same Write payload from the actual worktree directory returned hook_exit=0 and target_absent=true. Git reported the feature worktree root and branch wt/leader-completeness-check.

An external worker was launched from the feature cwd and wrote an acknowledgement. It stalled on its own hook subprocess and produced no completion result; the pane was closed after inspection. That proves startup placement only, not successful worker completion.

The old worktree-session-routing MVP explicitly left runtime relocation out of scope. Existing external herding transport now supplies a supported cwd at process creation. No child command can change the existing parent process cwd.

## Bounded correction

Update skills/bee-hive/references/routing-and-contracts.md and skills/bee-swarming/SKILL.md to separate native and external transport. Do not weaken the guard, modify Pi session internals, introduce new commands, or claim that a worker launch transfers leader ownership. Keep leader-authored decisions and gate writes with the leader. Identify missing leader-runtime transition honestly when one is necessary.

## Acceptance

- A supported external transport does not inherit a blanket manual-entry requirement intended for native workers.
- The routing instruction proves worker cwd and receipt before counting startup.
- Main never writes across the canonical worktree boundary.
- Lane binding never masquerades as a cwd change.
- A leader-only operation that cannot run from main uses an actual supported session handoff; no worker or lane metadata impersonates that transition.
- Existing guards, approval rules, and the paused leader-completeness-check feature remain unchanged.

## Proof and sizing

Pressure-test the current and corrected routing instructions. Regenerate managed skill copies and check their pointers and release manifest. Inspect the diff against each acceptance item. No runtime code changes; no full-suite rerun mandated by this instruction change.

Smallest Honest Shape keeps this repair to two instruction sources. Chesterton's Fence keeps the guard whose containment purpose still holds. Crash Site Versus Fault Site moves the correction to transport selection, not the refusal handler.
