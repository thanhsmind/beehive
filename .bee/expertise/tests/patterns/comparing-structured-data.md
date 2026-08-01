# Comparing structured data

## Context

When asserting on a structured artifact — a JSON store, a JSONL ledger, a
config file — or asserting that an operation left one unchanged. These
formats carry meaning independent of their serialization: whitespace, key
order, trailing newline, number formatting. Comparing the text compares the
writer as much as the data.

## Mechanism

Parse both sides and compare the parsed values. For "this file must be
unchanged," capture it before the action, then parse both and compare.

Two cases invert this rule, and both are common enough to name:

- **Key order is part of the contract.** When a consumer reads the file
  positionally, or when the artifact is diffed by humans or pinned by
  another tool, order matters and a parse-then-compare that ignores it will
  pass over a real regression. Use a parser that preserves insertion order
  and compare the serialized form.
- **The bytes are the deliverable.** Byte-identical output is sometimes the
  contract itself — a manifest another implementation must reproduce, a
  rendered artifact under a pin. Then compare bytes and normalize nothing.

Pick deliberately, and say which one you are in. The failure mode of getting
this wrong is silent in both directions: a text comparison that breaks on a
harmless reserialization, or a parsed comparison that sails past reordered
output some consumer depends on.

## Example

```rust
// Good — semantic comparison: robust to whitespace and serializer quirks.
let before: Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
run_operation_that_must_not_touch_state();
let after: Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
assert_eq!(after, before);

// Good — order-sensitive contract, stated as such.
// The store is read positionally, so key order IS behavior here.
assert_eq!(fs::read_to_string(&path)?, expected_serialization);

// Bad — text comparison used for a semantic question: fails when the writer
// changes indentation, passes when a value silently changes type.
assert_eq!(fs::read_to_string(&path)?, before_text);
```

For a JSONL ledger, parse line by line and compare the records; a whole-file
text comparison turns one appended entry into an unreadable diff, and hides
which record actually changed.

## Notes

When the artifact carries values that legitimately vary per run —
timestamps, generated ids, durations — mask those fields before comparing
rather than dropping the assertion. Compare everything else exactly; see the
differential-testing pattern for how to tell a genuinely varying token from
a defect you would be hiding.
