---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

How does a caller describe a scenario to the core?

## Answer

A Rust API — but the wave is a **value**, not a sequence of calls.

The real fork was never "API or file". It is whether a wave is a call
sequence or a struct. Built as a value — a `Wave` holding a list of
`WorkerSpec`, timeouts, and a failure policy — a file format later is
`serde` deriving onto types that already exist, and costs almost
nothing. Built as an imperative call sequence, extracting a format
later means designing the thing twice.

So: ship the API now, because D01 wants something running rather than a
schema, and keep the file format free for whenever it is wanted.

**The failure policy is an enum in the `Wave` value from day one**, even
with a single variant implemented. It is the axis that actually varies
between scenarios — wait for all, first success and cancel the rest,
best effort and report — and it is exactly the axis a recipe format
would have to grow a language for. Cheap to put in now, expensive to
retrofit.

Danger named: the source choreography has exactly ONE shape — fan out,
wait for all, aggregate. That is the only shape anyone has proven.
Inventing a recipe format now would freeze that single shape as if it
were the general case. Only a second real scenario shows which parts
are genuine parameters and which were incidental to the first.

Logged as D11.
