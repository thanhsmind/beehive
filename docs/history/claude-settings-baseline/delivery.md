---
type: bee.delivery
title: claude-settings-baseline — delivery
description: "Delivery record for feature claude-settings-baseline: Claude settings baseline at install, add-only merge, opt-outs, and installer pass-through."
timestamp: 2026-11-10
bee:
  id: claude-settings-baseline-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [docs/history/claude-settings-baseline/CONTEXT.md, docs/history/claude-settings-baseline/plan.md]
  sources: [docs/history/claude-settings-baseline/CONTEXT.md, docs/history/claude-settings-baseline/plan.md, cell csb-1, cell csb-2]
---

# claude-settings-baseline — Delivery

## Concepts changed

- **docs/knowledge/areas/onboarding/host-project-artifacts.md** — Documents the Claude settings baseline projected add-only into `.claude/settings.json`, its permissions mode, denied tools, switches, notices, durable config opt-out, and installer pass-through (claude-settings-baseline D1-D7).

## What shipped

- **csb-1** — Projected the Claude settings baseline add-only in onboarding with `--no-claude-settings` and durable `"claude_settings": false` config opt-out (commit 471430df1).
- **csb-2** — Passed `--no-claude-settings` through `scripts/install.sh` and `scripts/install.ps1` and updated the release manifest (commits 26a6ee8c1, 32878a036).
- **csb-3** — Consolidated feature knowledge into host-project-artifacts and wrote delivery record (commit pending).

## Acceptance notes

- Denying `AskUserQuestion` makes gate questions plain chat text in hosts.
