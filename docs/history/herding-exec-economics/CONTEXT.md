# Herding Exec Economics — Context

**Feature slug:** herding-exec-economics
**Date:** 2026-08-20
**Shaping session:** complete (fix-first brief, tiny lane; backlog finding from the ht goal-check)
**Scope:** Quick

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | derive_economics treats channel "herding-exec" exactly like "cli-exec": enforcement "cli-command"-analogue (name it "herding-command"), effective_model_status "unverified", requested_model Null — an external pane executor names its own model; "prompt-budget" there is dishonest | Judge finding 2026-08-20; no behavior break, accounting honesty |
| D2 | This cell EXECUTES THROUGH THE HERDING FLOW ITSELF (bee herding run, agy worker in the feature worktree) — the first real write-work dogfood; the orchestrator does all bee bookkeeping per herding-executor D4 | The user asked for one concrete task through the flow |
