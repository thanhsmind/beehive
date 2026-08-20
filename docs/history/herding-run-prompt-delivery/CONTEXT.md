# Herding Run Prompt Delivery — Context

**Feature slug:** herding-run-prompt-delivery
**Date:** 2026-08-20
**Shaping session:** complete (fix-first brief, tiny lane)
**Scope:** Quick

## Feature Boundary

`bee herding run` delivers the round-1 brief through `herdr agent prompt` after a plain `agent start`, because a multi-line brief cannot ride `agent start`'s trailing argv (herdr refuses: "agent arguments cannot be encoded safely for the target shell" — live smoke smoke-agy-1, 2026-08-20). One file; the mailbox contract and every flag stay unchanged.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | execute_new starts the agent with agent_command args ONLY (no brief in argv), waits for start success, then sends the rendered brief via `herdr agent prompt` — the same delivery `--continue` already uses | The argv path is dead on arrival for real briefs; prompt delivery is live-proven by wave and continue |
