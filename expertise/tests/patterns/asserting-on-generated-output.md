# Asserting on generated output

## Context

When the subject produces text by combining data with a fixed template:
prompts, codegen, reports, formatted errors, log lines. The output has two
halves — the template's static prose, and the values the renderer
substituted into it — and only one of them is worth asserting on.

## Mechanism

Assert on what the renderer did with the data:

- every value that should have been substituted appears in the output;
- no placeholder syntax survives (`{{`, `${`, `<%`);
- values that must be *absent* — a secret, another session's id — are absent.

Do not assert on the static prose. Prose is edited constantly and carries no
contract, so a prose assertion fails on every wording change while catching
no defect. One renderer-level test that feeds known inputs and checks the
conditions above covers "did it render at all"; downstream tests assert the
data flow that produced the values, not the sentences around them.

**The exception — when the bytes themselves are the contract.** Sometimes a
rendered artifact is pinned: a prompt whose exact bytes are part of an
agreement, a generated file another tool parses. Then the test is a
byte-comparison against the checked-in artifact, and it is a *different*
test from the ones above — it exists to catch accidental edits, so it must
compare everything and normalize nothing.

## Example

```rust
// Good — the data reached the output, and nothing was left unrendered.
assert!(prompt.contains(&cell_id));
assert!(prompt.contains(&learned_context_block));
assert!(!prompt.contains("{{"));

// Good — the pinned-bytes case, kept deliberately separate.
assert_eq!(rendered, include_str!("../prompts/worker-cell.md"));

// Bad — asserts that nobody reworded the template.
assert!(prompt.contains("You are executing exactly one cell"));
```

## Notes

When the renderer assembles a block from several sources — a manifest, an
index, a list of patterns — assert the *composition rules* rather than the
finished paragraph: the block is present, it respects its line budget, and
each source contributed. Those are the properties that break when a source
goes missing; the exact sentence is not.
