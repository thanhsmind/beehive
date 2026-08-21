# Herding Start Retry — Context

**Feature slug:** herding-start-retry
**Date:** 2026-08-20
**Shaping session:** complete (fix-first brief, tiny lane; backlog P3 finding)
**Scope:** Quick

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | execute_new retries agent_start when herdr refuses with agent_pane_busy — bounded (10 attempts, ~1s apart), same seam style as deliver_pointer; exhaustion keeps the spawn-failure behavior (close the just-split pane, typed error) | Live dogfood hee-1: pane split then start raced the shell's boot; a warm retry passed |
| D2 | This cell runs through the CONFIG route as the eval: models.claude.generation = {kind:herding, agent:agy-flash}; the standard dispatch prepare payload is executed as-is and every deviation is an eval finding | User asked to eval the claude executor over agy on the standard flow |
