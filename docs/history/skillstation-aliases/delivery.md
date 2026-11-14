---
type: bee.delivery
title: skillstation-aliases — delivery
description: "Delivery record for feature skillstation-aliases: the bee skillstation install and alias verbs, the README section and the onboarding concept."
timestamp: 2026-11-14
bee:
  id: skillstation-aliases-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [docs/history/skillstation-aliases/CONTEXT.md, docs/history/skillstation-aliases/plan.md]
  sources: [docs/history/skillstation-aliases/CONTEXT.md, docs/history/skillstation-aliases/plan.md, cell ssa-1, cell ssa-2]
---

# skillstation-aliases — Delivery

## Concepts changed

- **docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md** — New: why a hub skill cannot be started with `/<id>`, the install and alias verbs, the ownership rule for skill folders, and hub-owner approval (skillstation-aliases D1-D9).
- **docs/knowledge/patterns/20261114-a-fake-cli-test-that-resolves-anything-through-path-races-the-test-that-empties-path.md** — New pitfall from ssa-1: a fake-CLI test that resolves through PATH races the test that empties PATH.

## What shipped

- **ssa-1** — Added `bee skillstation install` and `alias` with registry entries and tests (commit efc1408fe).
- **ssa-2** — Added the README Skillstation section, the onboarding concept and this record.
