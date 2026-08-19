---
type: bee.pattern
title: A test that derives its fixture from the constant under test only proves the code agrees with itself
description: A test that derives its fixture from the constant under test only proves the code agrees with itself
tags: [failure, tests, review]
timestamp: 2026-08-19
bee:
  id: pattern-20260819-a-test-that-derives-its-fixture-from-the-constant-under-test-only-proves-the-code-agrees-with-itself
  lifecycle: active
  areas: [workflow-state, rust-runtime]
  sources: [".bee/cells/ddb-1.json", "original feature: doc-deferral-baseline"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs (doc_deferral_door_dry_run_detail_caps_the_sample_and_still_names_the_exact_count — every number in the fixture is a literal, and the doc comment records that deriving them from DOC_DEFERRAL_DRY_RUN_SAMPLE made the test pass with the cap raised to 100000)"
---

# A test that derives its fixture from the constant under test only proves the code agrees with itself

A dry-run detail line was joining every seed message into one string — 143 KB on this
repo, printed to the terminal and embedded in a JSON payload. The fix capped the sample
at a constant and summarised the remainder. The test written to pin it looked careful:

```rust
let total = DOC_DEFERRAL_DRY_RUN_SAMPLE + 7;
// ... create `total` files, each with one deferral line ...
assert!(detail.contains(&format!("and {} more", total - DOC_DEFERRAL_DRY_RUN_SAMPLE)));
assert_eq!(detail.matches("deferral-shaped prose").count(), DOC_DEFERRAL_DRY_RUN_SAMPLE);
```

Raising the constant to `100000` — deleting the cap in every way that matters — left the
test green. Both sides of every assertion move with the constant, so the equation holds
at any value. The test asserted that the code was internally consistent, which it always
is, and never that the output was bounded, which was the entire point.

The same numbers as literals fail immediately under that mutation:

```rust
for i in 0..27 { /* ... */ }
assert!(detail.contains("27 pre-existing deferral line(s)"));
assert!(detail.contains("and 7 more"));
assert_eq!(detail.matches("deferral-shaped prose").count(), 20);
assert!(detail.len() < 4096);
```

The rule generalises past constants. Whenever a test computes its expectation through
the same expression, helper, or configuration value the production path uses, it stops
being an independent check and becomes a restatement. Deriving a fixture from a config
default, building an expected string with the same formatter that produced it, or
reusing the production parser to read back what the production serializer wrote are all
the same failure wearing different clothes.

Two things make this one worth remembering.

**It survives careful review.** The derived version is what a diligent author writes:
it looks DRY, it looks like it will not rot when the constant changes, and it reads as
more rigorous than three bare numbers. The literal version looks lazier and is strictly
stronger. When a test and the code share a symbol, the test is only as strong as the
part that does not.

**Only mutation finds it.** The derived test passed a full green suite and would have
shipped. What exposed it was raising the constant to 100000 and expecting red. Any
assertion that pins a bound, a cap, a limit, or a count deserves that one check: break
the thing on purpose and confirm the test notices. This was caught one commit after an
independent judge had found the same shape of vacuity in six other tests in the same
file, which is the useful part — knowing the pattern by name did not stop it being
written again twenty minutes later.
