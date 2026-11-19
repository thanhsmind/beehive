# Vendored prompt parity guard — Context

**Feature slug:** vendored-prompt-parity-guard
**Date:** 2026-11-19
**Scope:** Tiny
**Domain types:** RUN

## What was asked

The owner asked to work the fix-at debt. The top open group (8 findings, all
from run db52a73c): a worker narrowed the prompt skew assertion in
`verbs/drivers/tests.rs` so a stale vendored prompt in `.bee/bin/prompts/`
passed. The existing parity checks loop over hand-written name lists, and
`peer-boot.md` has no vendored parity check at all.

## What will be done

Add one test that takes its file set from the `packages/bee/prompts/`
directory and asserts that `.bee/bin/prompts/` holds the same files with the
same text. Narrowing a name list can no longer hide a stale vendored copy.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | A directory-driven test: every `.md` in `packages/bee/prompts/` has a twin in `.bee/bin/prompts/` with equal text after CRLF normalization, and the vendored tree holds no extra file. | 39cbf005 |
| C1 | Test contract vendored-prompt-parity: green on main; red, naming the file, on one changed or one extra vendored file. | a9aedf60 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/workflow-state/workflow-records-and-projections.md | suspect | names contract C4, the embedded-prompt parity pin |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/devtools/prompts.rs` | `c4_embedded_prompts_match_disk`, the home of the new test |
| `packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs` | `prompt_skew`, the runtime skew probe |

## Out of scope

The other open fix-at groups.
