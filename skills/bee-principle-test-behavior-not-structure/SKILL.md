---
name: bee-principle-test-behavior-not-structure
description: "Apply when you write or repair a test. Assert what the code observably does — returns, emitted events, persisted effects — never how it is internally arranged."
---

# Test Behavior, Not Structure

Assert the contract: what came back, what was emitted, what was persisted.
Never the choreography — call counts, call order, private functions reached
directly, your own collaborators replaced by mocks. Mock at genuine boundaries
only: network, clock, filesystem, third-party services.

**The refactor litmus:** a refactor that preserves behavior must not break the
suite. If a rename, an inlined helper or a reordered internal call turns tests
red, those tests were pinned to structure.

One exception — when the algorithm IS the deliverable. A sort must be stable, a
cache must evict least-recently-used, a retry must back off exponentially:
those properties are the observable behavior. Assert the property (the output
order, the eviction victim, the delay sequence), still not the private call
graph.

**Why:** structure-coupled tests punish exactly the improvements they were
supposed to enable, so the improvement stops happening.

**Depth:** `.bee/expertise/tests.md` § Test behavior, not structure.
