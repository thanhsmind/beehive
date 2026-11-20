# Shared target warning — Context

**Feature slug:** shared-target-warning
**Date:** 2026-11-20
**Scope:** Small
**Domain types:** RUN

## What was asked

The owner asked to work the fix-at debt. Open group (8 findings, 2026-11-03):
the full suite on main went red at output_spill because a wave-run worker
rebuilt target/release/bee from its own branch into the shared
`CARGO_TARGET_DIR` (`~/.cache/cargo-target`, exported in `~/.bashrc`) while
the suite ran. Offered two options, the owner chose option 2: keep the shared
target and warn.

## What will be done

`bee test` warns before it runs when `CARGO_TARGET_DIR` resolves outside the
checkout it tests and the wave ledger counts a live worker in another
checkout. The suite still runs and is recorded.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | Keep the shared target. bee test warns, and still runs, when CARGO_TARGET_DIR resolves outside the tested checkout while a live herding worker runs in a different checkout; the warning names the dir and those checkouts and says a red may be false. Per-worktree target dirs rejected. | 881a704e |
| C1 | Test contract shared-target-warning: warning on stderr and `shared_target_warning` in `--json` when the condition holds; none when the variable is unset, inside the checkout, or every live worker is in the same checkout. | 6fd09277 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/verify-pipeline/suite-result-cache.md | suspect | what bee test runs and records |
| docs/knowledge/areas/workflow-state/worktree-isolation.md | suspect | R37, shared build artifacts across worktrees |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/test_runner.rs` | `run`, the bee test verb |
| `packages/bee-rs/crates/bee/src/herding/wave_ledger.rs` | the ledger fold, `is_live`, worker `worktree` |

## Out of scope

Changing where cargo builds; serializing builds across worktrees.
