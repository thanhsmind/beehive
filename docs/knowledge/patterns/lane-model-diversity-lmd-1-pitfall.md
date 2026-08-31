---
type: bee.pattern
title: A generic-sounding helper name is a scope promise — keep it honest
description: Naming a helper after its narrowest current caller (seat) when it answers a question about any role slot is a lie about its scope
timestamp: 2026-08-29
bee:
  id: lane-model-diversity-lmd-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/lmd-1.json]
  polarity: pitfall
---

# A generic-sounding helper name is a scope promise — keep it honest

## What happened

Cell lmd-1 built a helper that resolves per-seat models, but the underlying
question it answers ("does this role slot resolve to a model?") is not
seat-specific. The cell named it `role_slot_resolves` rather than a
seat-only name, since a seat-shaped name would have promised a narrower
scope than the code actually has. Separately: `requested_role` was carried
on the `economics` side of the dispatch envelope rather than the tool
payload, because the payload is the literal Agent/Bash/spawn_agent argument
map and an unknown key there would be handed straight to the tool. And the
doctor's undescribed-hat listing follows the CONFIG's own declared order,
not alphabetical — an assumption the first test got wrong.

## The lesson

Name a helper for what it actually answers, not for its first caller's
context — a seat-only name on a role-generic helper misleads the next
reader about where else it's safe to call. And metadata that must never
reach a downstream tool call belongs beside the call, never inside the
literal payload the tool receives.

## Status

Candidate only. Naming the pattern, generalizing it beyond this cell, and
moving `bee.lifecycle` to `active` are a human or agent decision.
