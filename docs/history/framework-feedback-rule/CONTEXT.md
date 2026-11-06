# Framework feedback rule — Context

**Feature slug:** framework-feedback-rule
**Date:** 2026-11-06
**Scope:** Small (one rule in four homes)
**Domain types:** CALL

## What was asked

The owner said on 2026-11-06: "Việc tôi nói ở dự án beehive thì nên xem và
chỉnh lại bộ beehive không nên chỉ ghi vào bộ nhớ riêng, vì beehive là bộ
khung cho nhiều project".

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | Owner feedback given inside the beehive repo is a change request to the framework itself: record it in the bee store, the backlog or the doctrine files and route it through the bee chain so every downstream project gets it on the next release; an agent's private memory may hold a pointer but is never the only home. (owner) | 7d5b60ab |

## Homes

- `CLAUDE.md` § Feedback here changes the framework.
- `packages/bee/AGENTS.block.md` § Capture what settles, rule
  `agents-feedback-changes-framework`, rendered into `AGENTS.md`.
- The rule index in
  `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`.
- `skills/bee-capturing/SKILL.md` § Capture the moment it settles.

## Deviation

The owner asked to fold this into the running context-mark docs feature.
That feature and handoff-flag-doctrine had already merged, so this is its
own small feature on the same release path.
