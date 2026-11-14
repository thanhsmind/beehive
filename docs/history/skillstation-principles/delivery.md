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
  sources: [docs/history/skillstation-principles/CONTEXT.md, docs/history/skillstation-principles/plan.md, cell ssp-1, cell ssp-2, cell ssp-3, cell ssp-6, cell ssp-7]
---

# skillstation-principles — Delivery

## Concepts changed

- **docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md** — Extended: hub-served principles, the agent file tool line, the orient header and the publish source (skillstation-principles D1-D5).
- **docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md** — Extended: the Principle homes section points at the onboarding concept for where a principle body is fetched from.

## What shipped

- **ssp-1** — One helper reads `skillstation.principles`; the orient header and agent files name `skill_get` in hub mode (commit 27ccf7af8).
- **ssp-2** — Onboard checks, drops and publishes the principles in hub mode, with a loud skip fallback (commits b9e332ecb, de099f736).
- **ssp-3** — Documented the key in `docs/config-reference.md`, README and the concepts, and wrote this record.

## Round 2: one-step switch

- **D8-D10** — The bee binary carries the principle index, so `bee orient` names principles in every project and the D7 index guard is gone. `onboard --skillstation` sets `skillstation.principles` to `hub`, publishes every principle and removes the local `.claude/skills` copies in one apply; a failed publish keeps them. `install.sh --skillstation` forwards the flag and rechecks without it.
- **ssp-6** — The switch in onboard plan and apply, and the install recheck (commit a323ec582).
- **ssp-7** — Rewrote the README, the config reference and the two onboarding concepts to match D8-D10 and dropped the D7 text.

## Still to do

- Release with `scripts/release.sh` after merge.
