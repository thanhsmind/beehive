---
type: bee.area
title: Decision Memory — the leader triages proposed PBIs; the owner gets only owner-only calls
description: "Who decides a proposed PBI: the project leader triages it (build stays proposed, park, drop) and records the verdict in the backlog and the owner brief; only a money, publishing or irreversible call goes to the owner, decision-ready."
tags: [backlog, triage, brief]
timestamp: 2026-11-17
bee:
  id: decision-memory-decision-memory-the-leader-triages-proposed-pbis-the-owner-gets-only-owner-only-calls
  lifecycle: active
  areas: [decision-memory]
  decisions: ["7942f87c (2026-11-17 — the project leader triages every proposed PBI; only an owner-only call goes to the owner)"]
---

# Decision Memory — The Leader Triages Proposed PBIs

## Purpose

The owner works with each project's leader directly. A proposed PBI that waits on the owner's
"approve, park or drop" waits forever: on 2026-11-17 the agent-works board held 55 beehive PBIs
in Backlog, many since late October, each asking the owner to choose. The owner said the leader
must handle them and must not make the human the bottleneck (spec drop sup-20261117-a7d2).

## Business Rules

- **The leader triages every proposed PBI itself**, under this repo's own rules, and records the
  verdict in the backlog with `bee backlog pbi status`:
  - **build** — the PBI stays `proposed`. bee has no `ready` status; in this repo `proposed`
    means "in the leader's queue, the leader schedules it". It is never a request to the owner.
  - **park** — `parked`. Use it for work that needs shaping first, has no current friction behind
    it, or waits on an outside fact. Name the wake-up fact in the reason.
  - **drop** — `declined`, or `done` when the tree already meets the CoS. Use it for a fixed,
    duplicate or stale item.
- **Only an owner-only call goes to the owner**: it spends money, publishes outside the machine,
  or cannot be undone. Such an item is decision-ready: the question, two options at most, and the
  leader's recommendation. Every other call is the leader's.
- **The owner brief follows the verdict.** A brief's `ask:` key matches the triage (`build`,
  `park`, `drop`), and its `**Anh cần làm:**` paragraph states the leader's verdict and reason,
  never an open choice for the owner. An owner-only item keeps `ask: answer` and the question.
- **New proposals get the same triage at add time.** The agent that adds a PBI triages it in the
  same turn; it never leaves the verdict for the owner.
- **The board side is not ours.** waggledance applies the agent-works board from the leader's
  report; the leader never edits the board.

## Pointers

- Decision: 7942f87c (2026-11-17).
- Origin: spec drop sup-20261117-a7d2 from waggledance, owner words of 2026-11-17.
- Brief format: areas/decision-memory/owner-brief-contract.md.
- Backlog store and status enum: areas/decision-memory/backlog-store.md.
