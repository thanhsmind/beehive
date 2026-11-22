---
type: bee.pattern
title: "A dispatched stage that fails preflight gets run inline by the leader, and the preflight never gets fixed"
description: "Twelve consecutive releases saw the dispatched deploy worker stop before the release script ran and the leader ship inline under a fresh permit; because the release landed green, the same preflight failure returned each time. An inline run after a failed dispatch is allowed only with a same-turn fix-first record, and the second occurrence of the same refusal is the stop."
tags: [deploy, herding, release]
timestamp: 2026-10-27
bee:
  id: pattern-20261027-a-dispatched-stage-that-fails-preflight-gets-run-inline-by-the-leader-and-the-preflight-never-gets-fixed
  lifecycle: active
  areas: [bee-herding]
---

## The pitfall

A release is a dispatched deployment stage: the door prepares a deploy worker, the worker runs the release script under a one-time permit, and the leader waits. Across twelve consecutive releases (2.45.0 through 2.63.0) the dispatched path stopped before the script ran — a read-only payload that could not mutate, a permit minted for the wrong session, dirt on main from a broker heartbeat, a dirty-tree precondition — and each time the leader ran the release script inline under a fresh permit and shipped. The release landed, so nothing was red, and the same preflight failure returned at the next release.

## Why it repeats

The inline run is cheaper than the fix and leaves no red behind: the cap records a deviation, the delivery record lists it, and no reader counts deviations across releases. A pitfall that costs one line per occurrence never crosses the threshold that triggers a fix.

## What to do instead

- When a dispatched stage fails its preflight and the leader can finish inline, the inline run is allowed ONLY with a same-turn record of the preflight failure as a fix-first item: a backlog row or a reflection naming the exact refusal text, with `--fix-at check` or `architecture`.
- The second occurrence of the same refusal text is the stop: do not ship inline a second time without the fix cell claimed, or a decision that names the inline run as the accepted path.
- A deviation that names the same refusal across two features is a pattern candidate for the compound step, not a per-cell pitfall file.

## Evidence

Cells rel450-1, rel470-1, rel480-1, rel490-1, rel500-1 (claude deploy role returned a read-only gather payload); rel530-1, rel540-1 (fresh permit instead of the Paseo deploy worker); rel550-1 (`deploy_authorization_wrong_session`); rel560-1 (broker-heartbeat dirt on main); rel570-1 (ran inline under the permit); rel600-1 (dirty-tree precondition). Delivery records: `docs/history/knowledge-work/release-2-45-0/delivery.md` through `docs/history/knowledge-work/release-2-63-0/delivery.md`. Later fixes that each removed one cause: deploy-role-agy-flash, deploy-stage-execution, deploy-pane-authorization, release-clean-tree.

Recurrence, releases 2.66.0 through 2.72.0 (cells rel660-1 through rel720-1, delivery records `docs/history/knowledge-work/release-2-66-0/delivery.md` through `docs/history/knowledge-work/release-2-72-0/delivery.md`): every one ran inline again. The cause is now stable and by design, not a preflight bug — a herded worker may not push, and its 1800 s ceiling is shorter than the suite plus the CI wait. Releases 2.57.0 through 2.66.0 each logged a decision naming the inline run as the accepted path (d261d5ce through 997faaba), this pattern's own exit; 2.67.0 through 2.72.0 logged none and carried the inline run only as a cap deviation, so the stop above did not fire for six releases. Two first permits were refused before any change — 2.71.0's carried no release version, 2.72.0's was bound to an older main commit — each seen once, so neither is yet a second occurrence.

Recurrence, releases 2.73.0 through 2.91.1: the permit-bound-to-an-older-commit refusal came back four times, past the stop above. A commit landed after the permit was prepared — the plan (2.78.0, cell rel780-1), an older main commit after a dirty-tree refusal (2.86.0, rel860-1), lane bookkeeping (2.90.0, rel900-1, `deploy_authorization_wrong_commit`), the latest commit (2.91.1, rel911-1). A sibling cause, an uncommitted file at run time, stopped 2.79.0 (rel790-1). Each was fixed by a fresh permit on the current commit, and none filed a fix-first item. The rule that follows: commit every release artifact first, then prepare the permit as the last step before the run. No release cell after 2.91.1 records the refusal. Over the same span the inline run stayed the accepted path, named by decision `537d8a6d` in 2.88.0.

Beyond releases, early November 2026: the same shape recurred in feature cells. Three cells ran inline after `bee dispatch prepare` refused on `limits.concurrency` (cma-1, cma-2, hfd-1), and one more because herding concurrency was full (ffr-1). Workers that went idle were cancelled and the leader finished the work inline or dispatched it again: a 900 s idle stop (pw-1), a worker stalled for about 3 h (kfm-1), a judge idle twice (kfm-3). A worker committed and stopped before its cap, and the leader capped it (kdf-4). Each was recorded as a cap deviation only. The stop above covers any dispatched stage, not only deploy: a refusal or stall seen twice gets a fix-first item. Sources: the promote proposals of context-mark-advisory, handoff-flag-doctrine, framework-feedback-rule, paseo-worktree-workspace, knowledge-freshness-mechanic and knowledge-docs-in-the-flow under `docs/history/`.

## Pointers

- `docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` (B21–B23, the deploy authorization chain)
- `docs/knowledge/areas/onboarding/release-identity-and-version-parity.md` (the release script's gates and the deploy permit row)
- `docs/knowledge/areas/human-mailbox/overview.md` ("A mistake seen twice")
