# Pinning against a live oracle

## Context

When you reimplement behavior that something else owns: a platform API, a
collation order, a serializer's number formatting, a shell's argument
resolution, another runtime's string semantics. The rules are written down
somewhere, but the written rules and the shipped behavior are not the same
thing — and your test's expectations, if you author them from the docs or
from intuition, encode your reading rather than the reality you must match.

## Mechanism

Ask the real thing, then freeze its answers as the expectation.

1. **Query the oracle** with a spread of inputs, including the ones you
   believe you understand. Run it as a one-off script or probe; capture the
   answers verbatim.
2. **Write the captured answers into the test** as a table of pinned
   vectors, with a comment naming the oracle and how to regenerate them.
3. **Assert your implementation reproduces the table.** When a vector
   disagrees, the oracle is right by definition — your implementation
   changes, never the vector.

Include the cases you think are obvious. Those are the ones that carry
wrong beliefs, because you never thought to check them.

## Example

```bash
# Capture the oracle's real answers — never author them by hand.
node -e '
  const pairs = [["a01","a1"],["Ab","aC"],["x_1","x-1"],["r-5","r-23"]];
  for (const [a,b] of pairs)
    console.log(a, b, a.localeCompare(b, "en", { numeric: true }));
'
```

```rust
// Pinned from the oracle above (regenerate with scripts/probe_collation.js).
// Each row is what the platform ACTUALLY answers, not what the spec implies.
const VECTORS: &[(&str, &str, Ordering)] = &[
    ("a01", "a1",  Ordering::Equal),    // leading zeros compare equal
    ("Ab",  "aC",  Ordering::Less),     // case is a deferred tertiary
    ("x_1", "x-1", Ordering::Less),     // '_' sorts before '-'
    ("r-5", "r-23", Ordering::Less),    // numeric run, not byte order
];

#[test]
fn collation_matches_the_platform() {
    for (a, b, expected) in VECTORS {
        assert_eq!(compare(a, b), *expected, "{a} vs {b}");
    }
}
```

## Notes

Two of those rows contradict what a careful reading of the documentation
suggests. That is the point of the pattern: `"a01"` and `"a1"` comparing
*equal* — not merely adjacent — and case ranking below every letter
difference are behaviors you adopt only because you asked. A reimplementation
built on the documented rules alone passes its own tests and disagrees with
production.

Record where the vectors came from and how to regenerate them. A pinned
table with no provenance becomes unmaintainable the first time the oracle
legitimately changes, because no one can tell a stale expectation from a
real regression.

When the oracle is available at test time, prefer querying it in the test
over a frozen table — the table is for oracles you cannot invoke from the
suite (another language's runtime, a platform you are cross-compiling for).
