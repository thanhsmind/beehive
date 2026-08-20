# Herding Prompt Verify — Context

**Feature slug:** herding-prompt-verify
**Date:** 2026-08-20
**Shaping session:** complete (fix-first brief, tiny lane)
**Scope:** Quick

## Feature Boundary

After sending the pointer prompt, `bee herding run` verifies the pointer text is actually visible in the agent's pane and resends it (bounded) when it is not — because herdr reports idle/interactive_ready before the agent's input loop accepts injected text (live smoke 6: brief-1.txt written, prompt "sent", input empty; a later manual resend of the identical line landed). One file plus the trait seam.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | After each agent_prompt of the pointer, read the agent's recent pane text; the send counts as delivered only when the pointer's brief-file name is visible. Not visible after a short wait → resend, up to 5 attempts spaced by the poll interval; exhaustion is a typed failure keeping the pane. Applies to spawn and --continue | Status flags lie about input readiness; the pane text is the only honest delivery receipt. Duplicate delivery is harmless: the pointer is idempotent (read the same file) |
