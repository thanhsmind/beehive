---
type: bee.pattern
title: A seam added for testability can relocate the untested join instead of closing it
description: "Threading construction through a new seam makes the inner hop testable while pushing the real production argument one level up, out of every test's reach — a closure literal written at a call site is reachable by no test, so the untested join relocates instead of closing. Measured on the herding backend seam: the whole workspace stayed green under a mutation that made the production closure ignore both of its arguments and construct with constants, while the doc comment added alongside asserted the seam was the only place a backend is built from a resolved pair."
timestamp: 2026-08-19
bee:
  id: pattern-20260819-seam-relocates-the-untested-join
  lifecycle: active
  areas: [rust-runtime]
  sources: ["capture stub 01244180 (herding-orchestration, captured in its worktree)", packages/bee-rs/crates/bee/src/herding/wave.rs]
---

The fix for an untested join creates a new one, and the reason is
structural rather than careless. A closure literal written at a call
site is unreachable by any test: the test supplies its own closure to
the same function, so the two never meet. Threading backend
construction through such a seam in
packages/bee-rs/crates/bee/src/herding/wave.rs made the inner hop
testable — and pushed the real production argument one level up, out
of every test's reach. The whole workspace stayed green under a
mutation that made the production closure ignore both of its
arguments and construct with constants. The seam made the join
testable; it did not make it tested, and it silently relocated a
different join in the same move. The doc comment added alongside then
asserted that the seam was the only place a backend is built from a
resolved pair — the mutation falsified that sentence the day it was
written. This is the parent shape of
docs/knowledge/patterns/20260819-the-join-between-two-tested-parts-is-what-nobody-tests.md
reproducing itself inside its own remedy: the corrective move carries
the defect it corrects. What actually closes it is a name. A free
function the caller passes and a test calls directly is reachable
from both ends; a closure literal is reachable from neither.

**The rule:** when adding a seam to make something testable, ask what
the production side now passes into the seam and whether anything
observes that value. If the answer is a literal written at the call
site, the untested hop has moved rather than closed. Name the
production side — a free function that the caller passes and a test
calls directly — instead of a closure literal.
