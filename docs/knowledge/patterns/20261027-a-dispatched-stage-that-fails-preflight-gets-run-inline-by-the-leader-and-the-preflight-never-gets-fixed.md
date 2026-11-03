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

Cells rel450-1, rel470-1, rel480-1, rel490-1, rel500-1 (claude deploy role returned a read-only gather payload); rel530-1, rel540-1 (fresh permit instead of the Paseo deploy worker); rel550-1 (`deploy_authorization_wrong_session`); rel560-1 (broker-heartbeat dirt on main); rel570-1 (ran inline under the permit); rel600-1 (dirty-tree precondition). Delivery records: `docs/knowledge/work/release-2-45-0/delivery.md` through `docs/knowledge/work/release-2-63-0/delivery.md`. Later fixes that each removed one cause: deploy-role-agy-flash, deploy-stage-execution, deploy-pane-authorization, release-clean-tree.

## Pointers

- `docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` (B21–B23, the deploy authorization chain)
- `docs/knowledge/areas/onboarding/release-identity-and-version-parity.md` (the release script's gates and the deploy permit row)
- `docs/knowledge/areas/human-mailbox/overview.md` ("A mistake seen twice")
