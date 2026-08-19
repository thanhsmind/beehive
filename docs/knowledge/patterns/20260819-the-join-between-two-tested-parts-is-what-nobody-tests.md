---
type: bee.pattern
title: The join between two tested parts is what nobody tests
description: "When a producer has a test and a consumer has a test, the arrow between them is asserted by nobody — the reviewer's eye supplies it — and only mutating the join finds the gap. Nine measured instances in one feature: four joins where replacing the carried value with a constant survived the full suite (71 tests in one case, the entire workspace in another), a crossing test at the wrong altitude that stayed green at 1908 passed while two same-typed flags were swapped, and a ninth instance shipped in the test doubles by a worker whose dispatch prompt enumerated the previous eight."
timestamp: 2026-08-19
bee:
  id: pattern-20260819-untested-join-between-tested-parts
  lifecycle: active
  areas: [rust-runtime, bee-herding]
  sources: ["herding-orchestration cells ho-9, ho-12, ho-13 (.bee/cells/)", docs/history/herding-orchestration/CONTEXT.md, "capture stubs d3690b49, 020201b8, 00a35fcf"]
---

Two things are each tested, and the join between them is not. Four
instances in one feature: a constructed field reaches argv through a
call site, and replacing that field with an empty slice survives all
71 tests; an extracted pure function is tested in isolation while the
trait method's use of it is not, so disabling the call-site short
circuit survives the whole platform-portable set; a production
constructor carrying an entire documented obligation has zero
callers, so it can discard both parameters with the suite green; a
CLI verb resolves config correctly and constructs a backend
correctly, and discarding the resolved pair at the single call site
between them leaves the entire workspace green. Every one is
invisible to reading and to ordinary test-writing; every one is found
by mutating the join, not the parts. The shape is durable because it
looks finished from both ends — the producer has a test, the consumer
has a test, and the reviewer's eye supplies the arrow that no test
asserts.

A crossing test must cross at the same boundary production crosses.
Production ran argv → ledger row → occupancy; the crossing test ran
row → occupancy, exercising the two pure functions and skipping the
CLI wrapper between them. A judge swapped two same-typed arguments in
that wrapper — the path flag landed in the pane-id field and vice
versa — and the entire suite stayed green at 1908 passed, silently
reproducing the exact inert-ledger failure the test existed to
prevent: rows written, pane ids never matching the live list,
occupancy reporting a confident zero forever. Six string flags mapped
positionally onto six string parameters, where a mis-wire type-checks
perfectly. That was the seventh instance, and the first to appear
inside the cell written to fix the sixth. When someone reports a
crossing test, ask which two things it crosses and compare that with
which two things production crosses.

The lesson does not transfer as prose. A dispatch prompt enumerated
eight prior instances with measured consequences and stated three
distilled rules; the worker shipped a ninth on the exact path the
prompt named. The ninth lived in the test doubles, not the production
code: one spawner declared its argv parameter underscore-prefixed and
built its own command, discarding the value; the other recorded argv
into a field written once and read nowhere, while that field's own
doc comment claimed it existed so tests could assert the full value
crossing into the spawner. The double documented the exact intent and
did not fulfil it — worse than omitting the field, because a reader
checking coverage finds the field and the sentence and stops looking.

**The rule:** for any value constructed, stored, and later used,
write the test that fails when the stored value is replaced by a
constant at the point of use — the strongest form is an injected
closure that panics if called, which turns "this path is never taken"
into something a machine checks and which a value-equality assertion
would survive. A crossing test enters where production enters — the
same wrapper, the same door — or it is not crossing anything. On a
test double, an underscore-prefixed parameter is a red flag, because
the double exists to observe that value, and every field a double
records must have a reader — recorded-but-unread is dead weight
pretending to be coverage. Restating these rules in prose is measured
not to work — the ninth instance shipped after the eight before it
were named in the dispatch prompt — so the escalation is a check that
runs, never a warning that reads.
