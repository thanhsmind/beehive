---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

Does the generic core drive only herdr panes, or is herdr one backend
behind a seam?

## Answer

A seam. The core talks to a worker-backend trait — start a worker, read
its status, send it a task, read its output — and herdr is the first
implementation.

The deciding argument was not future reuse but testability. The crate
has no test seam for an external binary today: every `git` call is a
bare `std::process::Command`, and the nearest precedent is the
`BEE_POSIX_SHELL` override in `src/shell.rs:28,133-142`. The backend
trait doubles as that seam, which is what makes the whole choreography
testable without a running herdr server. The source skill needed a fake
herdr harness for exactly this reason, and that harness had to solve
atomic write-then-rename because concurrent waiters read its state file
mid-write.

Design caution carried forward: a trait shaped around herdr's exact
status enum is not a seam, it is herdr with extra steps. The status
model the trait exposes has to be the one the choreography needs —
ready / working / blocked / finished / unverifiable — with each backend
mapping its own vocabulary onto it, and "unverifiable" a first-class
value rather than an error.

Logged as D07.
