# Interfaces between systems

An interface is the part of a system that other people build on. Internals
can be rewritten whenever they stop serving; a published interface can
only be extended, deprecated, or broken — and breaking it breaks someone
else's working software. Everything here follows from that: design for
the caller, then keep the promise.

The rules apply to any contract crossing an ownership boundary — an HTTP
API, an RPC service, a message on a queue, a webhook, a library's public
functions, a command-line tool's flags and output. "Ours calls ours" does
not exempt a contract; it only means the person you break is a colleague.

## Where to look

| Situation / goal | Entry |
|---|---|
| Designing a new endpoint, method, or message | Design from the caller's job |
| Choosing between resource, RPC, or query styles | One shape per surface |
| Deciding what an operation returns when it fails | The error contract is half the interface |
| An operation creates, charges, sends, or mutates | Every unsafe operation needs a retry story |
| Calling something you do not control | Every call out is a call that hangs |
| A retry is being added | Retry the safe, back off, give up out loud |
| Changing a field, response, or parameter already in use | What you may add, what you may never change |
| A change cannot be made compatibly | Version when you break, not when you change |
| Returning a list | No unbounded response |
| A request does several things and some succeed | Partial failure is a state you must name |
| The operation takes longer than a request should | Long work gets a handle |
| Emitting events or webhooks to others | Events are at-least-once and out of order |
| Removing something callers still use | Deprecation is a schedule, not an announcement |
| Judging an interface you did not design | Reading an interface you inherited |

## Design from the caller's job

When designing any new surface → start from what the caller is trying to
accomplish, in their vocabulary, and work backward to the storage. The
failure mode is universal and easy to spot: an interface that mirrors the
provider's tables, its internal services, or its team boundaries, leaving
every caller to assemble a meaningful operation out of four calls and a
retry loop.

The test is a sentence: *what does one call let someone finish?* If the
answer needs "and then they call", the surface is a database with
network latency. Concretely, an interface that requires the caller to
know the provider's join order, its enum encodings, or which of its
services owns what has leaked its internals into someone else's code —
and now those internals cannot be changed.

Design the common case to be one call, and let the rare case be several.
Chattiness is not a performance problem you fix later with a cache; it is
a design property you fix now by naming the operation the caller actually
wanted.

## One shape per surface

When choosing between styles — resources with standard verbs, named
procedures, a query language, an event stream → pick by what the domain
actually is, then apply the choice consistently across the whole surface.
Resource-shaped works when the domain is genuinely things with lifecycles;
procedure-shaped works when it is genuinely actions; a query language
works when callers legitimately need to shape their own reads and you can
afford the cost of unpredictable queries.

The rule that matters more than the choice: **consistency beats
correctness at the margins.** A surface where half the operations are
resources and half are verbs, where some errors are HTTP codes and some
are `{"ok": false}` bodies, where one list paginates and another does
not, costs every caller a lookup at every call site. Pick the convention,
write it down once, and make the odd case conform or justify itself in
the docs.

Within a surface, keep naming, casing, date formats, id formats, null
semantics, and pagination identical everywhere. These are the details
that make an interface feel learnable, and inconsistency in them is the
most common reason a technically complete API is unpleasant to use.

## The error contract is half the interface

When defining what an operation returns → define its failures with the
same care as its successes. Callers spend most of their code on the
unhappy path, and an interface that returns a bare failure forces every
one of them to string-match a human message that was never meant to be
parsed.

Every error should answer four questions:

- **What kind of failure is this** — a stable, machine-readable code the
  caller can branch on, chosen from a documented set. Not the HTTP status
  alone; not the prose.
- **What exactly was wrong** — which field, which value, which
  constraint. "Invalid request" is a shrug; "`start_date` must be before
  `end_date`" is a fix.
- **Is retrying meaningful** — the single most useful bit in the whole
  response. A caller must be able to distinguish "your request is wrong,
  never send it again" from "we are briefly unavailable, try in a moment"
  without guessing from a status code.
- **How to find this occurrence** — a correlation id the caller can quote
  when they open a ticket, matching one you can search in your own logs.

Keep the codes stable. Once a caller branches on `insufficient_funds`,
renaming it is a breaking change even though nothing in the schema moved.
And keep the two audiences separate: a code for the program, a message
for the human, never one string trying to be both.

## Every unsafe operation needs a retry story

When an operation creates, charges, sends, or otherwise changes the world
→ assume it will be delivered more than once, and decide now what the
second delivery does. This is not a hypothetical: a caller whose
connection drops after the request arrives cannot tell success from
failure, and their only options are to retry or to lose the operation.

Reads and idempotent updates are safe by nature — reading twice, or
setting a field to the same value twice, changes nothing. Creation and
accumulation are not: two creates make two records, two increments move
the number twice.

The general fix is a **caller-supplied idempotency key**: the caller
generates a unique key per logical operation and sends it with the
request; you record the key with its outcome; a repeat of the same key
returns the original outcome instead of performing the work again. Three
details decide whether it actually works — the key is recorded in the
same transaction as the effect (otherwise a crash between them loses the
protection), a repeat with the *same* key but a *different* body is
rejected rather than silently ignored, and the retention window is
documented so callers know how long a retry stays safe.

Where a key is impractical, the fallback is a natural uniqueness
constraint in the store — the same order can only exist once — which
turns a duplicate into a constraint violation you can translate into the
original success.

## Every call out is a call that hangs

When calling anything you do not control → set an explicit timeout. A
call with no timeout does not fail; it occupies a worker, a connection,
and a caller who is waiting, until something further up gives up first.
This is how one slow dependency becomes an outage in a system that is
otherwise healthy: every request in flight is parked on the same stalled
call, and there is nothing left to serve the requests that would have
succeeded.

Choose the number from the caller's tolerance, not from the callee's
average — the whole point is to bound the worst case. And make the budget
add up: if the incoming request must answer in two seconds, the
downstream calls it makes cannot each be allowed three. When a request
has already spent its budget, stopping early is correct; continuing to
work for a caller who has left is pure cost.

**Isolate what can stall.** When one dependency is allowed to consume
every worker, its failure is your failure. Bound the concurrency spent on
each dependency so a stuck one degrades a feature instead of the system,
and let a dependency that is failing consistently be skipped for a while
rather than retried by every request in parallel.

## Retry the safe, back off, give up out loud

When adding a retry → answer three questions first, because a retry
implemented reflexively converts a small outage into a large one.

*Is this safe to repeat?* Only retry operations that are idempotent by
nature or protected by a key. Retrying a non-idempotent operation is
choosing duplicates over errors, silently.

*Is this worth repeating?* Retry only failures that plausibly resolve on
their own — timeouts, connection resets, explicit "busy" or "unavailable"
responses. Retrying a rejection for bad input just sends the same wrong
request again, four times, and reports the same error later than it could
have.

*What stops the retries?* A bounded count, growing delays, and random
jitter on each delay. Without growth you hammer a struggling service at
the exact moment it needs relief; without jitter every client that
noticed the outage retries in the same instant, and the recovery attempt
becomes a second outage. Beyond the bound, fail — and say which
dependency failed and how many attempts it got. A retry that swallows the
final error hides the cause of the incident inside a latency graph.

## What you may add, what you may never change

When changing an interface that already has callers → the safe set is
smaller than it feels, and the boundary is about what existing callers
already depend on:

**Safe:** adding a new optional field to a request; adding a new field to
a response; adding a new operation; accepting a value you previously
rejected; returning a new error code that only occurs for new inputs.

**Breaking:** removing or renaming anything; making an optional input
required; changing a type, format, or unit; narrowing what you accept;
changing the meaning of an existing value; adding a required field;
changing default behavior when a field is absent.

Two silent breakages deserve their own names because they pass review by
looking like nothing. **Never repurpose a field** — a column that meant
"cents" and now means "dollars", a status that gains a new meaning for an
old value, changes the contract while the schema stays identical, and
every caller keeps parsing successfully and computing wrongly. And
**never let a response shrink under a filter** — dropping a field that is
"usually null" breaks the caller who read it in the case where it was
not.

The mirror rule for consumers is to be **a tolerant reader**: ignore
fields you do not recognize rather than failing on them, and do not
assume the absence of a field is impossible. A consumer that rejects
unknown fields makes every provider's additive change a breaking one, and
turns the safe list above into an empty list.

## Version when you break, not when you change

When a change cannot be made compatibly → introduce a new version, and
understand what you have taken on: every live version is a code path to
maintain, test, and reason about during incidents. The cheapest version
is the one you did not need, so exhaust the additive path first — a new
optional parameter, a new field beside the old, a new operation with the
better shape.

When a version is genuinely needed, three rules keep it from multiplying.
Version the whole surface at a coarse grain, not each operation
separately, or callers end up assembling a matrix. Make the new version
the only place new work lands — a version you keep improving is a version
you will never retire. And decide the retirement date when you ship the
replacement, not when the old one becomes annoying.

## No unbounded response

When an operation returns a collection → cap it, always, with a documented
default and maximum. A list that returns "everything" is correct in
development and an incident in production, and it fails on the provider's
side (memory, timeout) at exactly the moment the caller most needs it.

Prefer a cursor — an opaque token meaning "continue from here" — over a
numeric offset. Offsets are wrong under concurrent writes: rows inserted
or deleted between pages cause callers to skip or repeat items, silently,
and the bug reports arrive as "some records are missing sometimes." A
cursor anchored to a stable sort does not have that failure. Keep the
token opaque so its encoding stays yours to change, and document what
happens when a caller holds one for a long time.

Say explicitly whether a total count is available. Counting is often the
most expensive part of a paginated read, and callers who need one will
build something worse if the interface silently makes them fetch every
page to get it.

## Partial failure is a state you must name

When one request does several things — a bulk create, a batch update, a
fan-out — decide and document which of two contracts it offers: all of it
happens or none of it does, or each item succeeds and fails on its own.
There is no third option, and leaving it unstated means every caller
guesses, most of them wrongly.

If it is all-or-nothing, the whole batch must be able to roll back —
which usually rules out fanning out to systems you do not control. If it
is per-item, the response must carry a result *per item*, keyed so the
caller can match it to what they sent, and the overall status must not
read as plain success. A `200 OK` over a body where three of ten items
failed is the shape that causes silent data loss downstream, because the
caller's error handling never ran.

## Long work gets a handle

When an operation takes longer than a caller should hold a connection —
a report, an import, a video encode → do not hold the request open.
Accept the work, return an identifier immediately, and give the caller a
way to learn the outcome: a status they can poll, or a callback you send.

Three things make the pattern usable rather than merely correct. The
status must distinguish *queued*, *running*, *succeeded*, and *failed
with a reason* — a caller polling a boolean cannot tell a slow job from a
dead one. The identifier must be usable after a restart on either side.
And accepting the work must itself be idempotent, or a retried submission
starts the expensive job twice.

## Events are at-least-once and out of order

When emitting events — webhooks, queue messages, streams → publish the
guarantees you actually provide, and design for the ones you cannot
avoid. In practice a consumer must assume every event may arrive more
than once, may arrive out of order relative to other events, and may
arrive late enough that the world has moved on.

That makes three things the producer's job. **Give every event a stable
id** so consumers can deduplicate. **Include the data the consumer needs
to act, plus enough to detect staleness** — a version or a timestamp of
the underlying entity — so a consumer receiving an old event can drop it
rather than overwriting newer state. And **let consumers verify the
sender**: an endpoint that acts on any POST it receives is an
unauthenticated write to your system, wearing a webhook's clothes.

On the receiving side the corresponding duty is to acknowledge only after
the work is durably recorded, and to make handling idempotent — the
delivery you already processed will be delivered again.

## Deprecation is a schedule, not an announcement

When removing something callers use → the announcement is the easy part.
A deprecation that works has four elements: a named replacement (never
"this is going away" alone), a date, a signal in the interface itself so
a caller learns from their own tooling rather than from a mailing list
they do not read, and **telemetry per caller** so you know who is still
on it.

That last one is what turns the schedule from a hope into a fact — you
cannot responsibly remove what you cannot see being used, and "we
announced it" is not evidence. When the date arrives and callers remain,
the choice is to extend or to break with warning; make it deliberately,
and make it once.

## Reading an interface you inherited

When judging a surface you did not design → read it in the order a caller
would meet it, because that is where the defects live. Take the three
most common operations and write out what a caller must do to accomplish
one real task; the number of round trips and the amount of provider
trivia they need is the interface's real quality score.

Then check the specifics that are cheap to see and expensive to fix
later: are errors machine-branchable and stable; do unsafe operations
have a retry story; does every list have a bound; are there fields whose
name no longer matches their contents; do two operations disagree about
casing, dates, or null; is anything documented that no longer exists.
Each mismatch between what the surface promises and what it does is a
finding — and the ones already published to callers are the ones to fix
first, because their cost grows with every new integration.
