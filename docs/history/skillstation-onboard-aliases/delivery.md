---
type: bee.delivery
title: skillstation-onboard-aliases — delivery
description: "Delivery record for feature skillstation-onboard-aliases: onboard and install write the skillstation /<id> aliases by default, and --skillstation adds missing hub skills as drafts."
timestamp: 2026-11-14
bee:
  id: skillstation-onboard-aliases-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [docs/history/skillstation-onboard-aliases/CONTEXT.md, docs/history/skillstation-onboard-aliases/plan.md]
  sources: [docs/history/skillstation-onboard-aliases/CONTEXT.md, docs/history/skillstation-onboard-aliases/plan.md, cell sooa-1, cell sooa-2]
---

# skillstation-onboard-aliases — Delivery

## Concepts changed

- **docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md** — Extended: onboard and install write the aliases by default, the skip rules and the `--skillstation` hub add (skillstation-onboard-aliases D1-D6).
- **docs/knowledge/areas/onboarding/host-project-artifacts.md** — Extended: the skillstation paragraph names the `skillstation_add` and `skillstation_alias` plan items.
- **docs/knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md** — Extended: `install.sh --skillstation` is forwarded to onboard (skillstation-onboard-aliases D5).

## What shipped

- **sooa-1** — Onboard plans and writes the alias skills, adds hub drafts under `--skillstation`, and `install.sh` passes the flag (commit 6c0c14e89).
- **sooa-2** — Updated the README Skillstation section, the three concepts and this record.

## Still to do

- Release with `scripts/release.sh` at the next minor version after merge (skillstation-onboard-aliases D8).
