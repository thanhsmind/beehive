---
type: bee.delivery
title: skillstation-principles — delivery
description: "Delivery record for feature skillstation-principles: the skillstation.principles key serves the bee-principle skills from the Skillstation hub to Claude Code."
timestamp: 2026-11-14
bee:
  id: skillstation-principles-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [docs/history/skillstation-principles/CONTEXT.md, docs/history/skillstation-principles/plan.md]
  sources: [docs/history/skillstation-principles/CONTEXT.md, docs/history/skillstation-principles/plan.md, cell ssp-1, cell ssp-2, cell ssp-3]
---

# skillstation-principles — Delivery

## Concepts changed

- **docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md** — Extended: hub-served principles, the agent file tool line, the orient header and the publish source (skillstation-principles D1-D5).
- **docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md** — Extended: the Principle homes section points at the onboarding concept for where a principle body is fetched from.

## What shipped

- **ssp-1** — One helper reads `skillstation.principles`; the orient header and agent files name `skill_get` in hub mode (commit 27ccf7af8).
- **ssp-2** — Onboard checks, drops and publishes the principles in hub mode, with a loud skip fallback (commits b9e332ecb, de099f736).
- **ssp-3** — Documented the key in `docs/config-reference.md`, README and the concepts, and wrote this record.

## Still to do

- Release with `scripts/release.sh` after merge.
