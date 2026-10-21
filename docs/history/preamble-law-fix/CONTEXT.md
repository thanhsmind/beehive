# Preamble law fix — Context

**Feature slug:** preamble-law-fix
**Date:** 2026-10-21
**Scope:** Tiny (fix-first before release 2.57.0)

## What broke

`cargo test --test instruction_laws no_size_ceiling_on_instruction_text_survives_in_any_shipped_tooling_tree`
is red on `main` after token-thrift:

```
packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:66 (ceiling-shaped identifier "PREAMBLE_BUDGET_BYTES"): pub(crate) const PREAMBLE_BUDGET_BYTES: usize = 9500;
```

The law (decision `8f63adb4`) looks 6 lines above a ceiling-shaped constant
for an instruction-text subject word. token-thrift tt-1 removed the
`#[cfg_attr(not(test), allow(dead_code))]` line above the constant, so the
doc paragraph that says this budget is NOT on authored instruction text moved
into that window, and its own words ("INSTRUCTION TEXT", "skill", "AGENTS.md")
tripped the scan.

## Decision

D1: Reorder the two doc paragraphs above `PREAMBLE_BUDGET_BYTES` so the
paragraph about the generated payload sits right above the constant. No line
is added or removed. The budget stays a cap on a generated payload, as
token-thrift D6 and decision `8f63adb4` already state.
