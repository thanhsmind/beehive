# leader-completeness-check

## Problem

Worker reports are currently treated as completion evidence. They should
be navigation aids only — actual completion requires comparing every
approved requirement against actual artifacts, wiring, and execution
evidence.

## Decision

For every worker completion, compare every existing approved requirement
against actual artifacts before accepting the result.

- Worker reports = navigation aids, not completion evidence
- Reuse existing plan and cell requirements
- No new report schema
- No mandatory full-suite rerun
- Verification depth stays risk-based: deeper checks for high risk,
  missing evidence, or contradictions

## Scope

Instructional completeness check only:
- Additive instruction contract in three source files
- No Rust or schema changes
- Reuse existing must_haves and reports
- Pressure tests prove behavior

## Source

Decision f5e3c084-c91d-4dfa-825f-48175bc61fe5 (2026-09-05)
