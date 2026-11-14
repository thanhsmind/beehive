---
type: bee.area
title: Doctrine Layer — per-task route, tester pairing and proof floors
description: "Why the leader routes the task in hand instead of a stored lane route, why a code cell is never certified by its own writer (separate tester, close and merge refusal, docs checker), and how the proof floor per class stops a too-narrow proof from reading green."
tags: [doctrine-layer, routing, tester, proof-floor, principles]
timestamp: 2026-11-14
bee:
  id: doctrine-layer-per-task-route-tester-pairing-and-proof-floors
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md]
  decisions: ["per-task-router D1-D5 (docs/history/per-task-router/CONTEXT.md, 2026-11-14; store 676a73b1, 70c3387f, eb8842b0)", "58d7b2a7 (planning choices after the hat wave)"]
  sources: [docs/history/per-task-router/CONTEXT.md, docs/history/per-task-router/plan.md, docs/history/per-task-router/reports/hat-wave.md, docs/history/per-task-router/delivery.md]
  authoritative_for: "doctrine-layer: per-task route, tester pairing and proof floors"
  anchors: [packages/bee-rs/crates/bee/src/principles.rs, packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs, packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs, packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs, packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs, packages/bee-rs/crates/bee/src/verbs/cells/proof.rs, packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs, packages/bee/LEADER.md, packages/bee/prompts/worker-cell.md, skills/bee-planning/SKILL.md, skills/bee-swarming/SKILL.md]
---

# Doctrine Layer — per-task route, tester pairing and proof floors

Read when: a session shows a hint line instead of a Route and principles, a worker prompt has or lacks a Principles block, a cap is refused for its proof kind or a missing principle answer, or `bee close` or merge names a code cell with no capped tester.

## Purpose

Three failures shared one root: the code writer chose its own proof, and the session inherited a route it never chose. A fresh unbound session fell back to the default state record, so a research task got a bugfix route's principles. A code cell capped itself on the narrowest proof it picked. The feature moves routing to the task in hand and moves certification to a second worker.

## Behaviors & Operations

**The per-task route.** The leader classifies the task in front of it and loads that class's playbook and principles; the lane record's stored route is not the source. The preamble prints Route and principles only when the route belongs to the session, meaning the session is bound by either bind path, or the repo has no lane records. Otherwise it prints one hint line naming `bee orient --class <c>`. That flag prints the playbook path and the class principles and writes nothing. The ownership test lives in one helper so the preamble and `bee orient` cannot disagree. The leader's todo list opens with the playbook steps; a skipped step stays with `skip:` and its reason.

**The tester pairing.** The implementer writes the code; a separate dispatched tester, with no shared context, verifies it and adds the new tests. This holds for every code-touching cell in every lane, `tiny` included, and an inline cap on a code cell is refused. A cell whose files are all `.md` or under `docs/` needs no tester; a second worker runs its parity and pointer checks. `bee cells add` stamps the cell and only warns when no dependent tester exists. Close and merge, through the one shared proof door, refuse a stamped cell without a capped tester or checker. A cell with no stamp is exempt, so work started before the rule still closes. An implementer owns the existing test edits its change forces; the tester adds new tests only.

**A red tester.** A tester never edits product code. On a defect it returns `[BLOCKED]` with the red output; the leader reopens the implementer cell and dispatches a fresh implementer whose prompt carries the tester's block as a prior round.

**Principles in the worker prompt, and the cap.** `bee dispatch prepare` attaches a Principles block (names and one-line rules) for the cell's route class, with a holdout share under its own config key. A tester hashes its implementer's id so a pair never lands in different arms; the prepare record names the arm. A cap from the principles arm must name each principle applied and the decision it changed, or say none, on the native and herded paths alike. A cap from the holdout arm, or from an older prepare record, is never refused for silence.

**Proof floors.** The cap refuses a proof line weaker than the floor of its cell. One Rust table holds the floors and the playbooks point to it. A bugfix tester must show red before the fix and green after, vocabulary `green:red-first`. Docs-only cells floor at static proof. Find the current rows in `class_proof_floor` and `cell_proof_floor`.

## Business Rules

- A code cell never certifies itself; this has no lane exemption (D1, D3).
- Implementer first, tester second, never the reverse (D2).
- The trigger index in `LEADER.md` points to the skill that owns a situation and never restates it.

## Rejected options

- A keyword classifier in Rust for the task class: it guesses wrong silently, and the leader's own classification is the signal.
- Floors written only in the playbooks: host repos do not carry skill files at a fixed path the cap could read.
- Refusing the pairing at `cells add`: breaks every fixture and every later single fix cell; close and merge already share one door.
- A same-worker identity check between implementer and tester: each cell is its own dispatch and under herding the leader caps both, so it fires falsely.
- Gating Route on a lane source alone: hides the route for a session bound through `session_binding` and for every lane-less repo.

## Pointers

- Principle homes and the duplication boundary: `areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`.
- How a worker consults on a red test: `areas/advisor-protocol/consult-loop.md`.
- Why the trigger index had to ride with the always-loaded layer: `patterns/20260713-promote-an-order-to-the-always-loaded-layer.md`.
- The hat wave that shaped these choices: `docs/history/per-task-router/reports/hat-wave.md`.
