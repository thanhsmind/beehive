---
type: bee.area
title: "Feedback Digest — purpose, data model, ranking, and self-improvement"
description: "The feedback digest: rolling up repo-local friction, findings, and debt into a privacy-safe, six-field summary, scoring pain, and driving the gated evolving self-improvement loop."
timestamp: 2026-08-22
bee:
  id: feedback-digest-overview
  lifecycle: active
  areas: [feedback-digest]
  required_context: []
  decisions: ["D2 8cd4c84e", 9880542e, c45d0fb3, "D 9157d074", D1 backlog-auto-commit, D2 backlog-auto-commit]
  sources: ["docs/history/evolving-loop/ (cells evolving-1 … evolving-11, capped)", "docs/specs/feedback-digest.md#R1", "docs/specs/feedback-digest.md#R2"]
  authoritative_for: "feedback-digest: purpose, overview, and governance"
  owns.code: [packages/bee-rs/crates/bee/src/verbs/feedback.rs]
  owns.skills: ["skills/bee-evolving/*"]
  owns.tests: []
---

# Feedback Digest — Purpose, Data Model, Ranking, and Self-Improvement

## Purpose

The feedback digest rolls up repo-local friction, review findings, and technical debt
into a structured, privacy-safe summary. It enables the system to learn from past
pain and drive the gated self-improvement loop (`bee-evolving`) without leaking
proprietary source code or sensitive details across repository boundaries.

## How this area is split

- Digest structure and the six allowed fields: `data-model.md`.
- How a repository generates and refreshes its digest: `generation-and-refresh.md`.
- Safe cross-repository consumption and trust rules: `cross-repo-trust-boundary.md`.
- Pain scoring, ranking, and self-improvement: `ranking-and-self-improvement.md`.

## Entry Points & Triggers

- **Filing a record** (`bee feedback add`) — operators or agents record friction, findings, or debt at intake.
- **Generating a digest** (`bee feedback digest`) — builds the six-field, privacy-safe summary from local records.
- **Evolving self-improvement** (`bee-evolving`) — reads ranked feedback to propose targeted hive improvements.

## Data Dictionary

| Element | Meaning |
|---|---|
| feedback record | Raw intake record capturing friction, finding, debt, or learning. |
| feedback digest | Aggregated, privacy-safe summary containing exactly six fields per entry. |
| pain score | Weighted measure of friction frequency and impact computed during digest generation. |
| evolving loop | Two-gate human-supervised improvement cycle driven by digest rankings. |

## Actors & Access

- **Operators and agents** — file feedback records and generate digests.
- **Human owner** — approves self-improvement gates in the evolving loop.

## Business Rules

- Intake records are validated strictly against the closed kind vocabulary and severity scale.
- Digest entries never carry free-text descriptions or code snippets.
- Self-modification runs only on human explicit invocation behind human approval gates.

## Pointers (implementation)

- CLI verb and intake validation: `packages/bee-rs/crates/bee/src/verbs/feedback.rs`.
- Evolving workflow skill: `skills/bee-evolving/SKILL.md`.
