# Making it fast

Performance work fails in two directions. Teams optimize what is easy to
optimize rather than what is slow, and they treat "fast" as a direction
rather than a target, so the work never ends and never visibly succeeds.
Both are fixed by the same discipline: decide what fast enough means,
measure where the time actually goes, change one thing, measure again.

## Where to look

| Situation / goal | Entry |
|---|---|
| Someone says it is slow, or should be faster | No number, no optimization |
| Deciding how fast is fast enough | The budget comes from the user |
| A profile is in hand and several things look bad | Fix the bottleneck, not the list |
| Choosing what change would help | Do it less, in bulk, or later |
| It is fine now but the data is growing | Growth beats constants |
| A cache is being proposed | A cache is a copy with an owner |
| The average looks fine and users complain | The tail is the product |
| Timings vary wildly between runs | Measuring without lying to yourself |
| Adding threads, workers, or parallelism | Parallelism is not a speed setting |
| Memory or allocation is suspected | Memory is a performance property |
| The interface feels slow but the numbers are fine | Perceived speed is real speed |
| The optimization worked — now what | Know when to stop, and pin it |

## No number, no optimization

When anyone says something is slow → get a measurement before touching
code. Not a guess, not a reading of the code, not the profile from a
similar system — a measurement of *this* operation, in a realistic shape,
that you can repeat after the change.

Developer intuition about where time goes is unreliable in a specific
way: it is drawn to code that *looks* expensive — a nested loop, a
complicated function — while real time is usually spent waiting, on a
network call, a lock, a query, a filesystem, or a startup path nobody
reads. The profile routinely points somewhere nobody in the room
nominated.

Optimizing without a measurement costs more than the time it wastes: it
makes code more complex, less obvious, and harder to change, in exchange
for nothing verifiable. That trade is only worth making where the profile
says it matters. Everywhere else, the clear version is the correct
version.

## The budget comes from the user

When deciding how fast is fast enough → state a target before starting,
and derive it from what the person on the other end is doing rather than
from what the current implementation happens to achieve. "Faster" has no
completion condition; "this page must respond in under 300 ms at the 95th
percentile" can be met, verified, and defended.

Budgets are also how you allocate. A request budget divides among the
work it does — so many milliseconds for the query, so many for
rendering, so many left for the calls it makes — and a downstream
dependency that needs more than its share is a design conversation, not a
tuning problem. This is the same arithmetic as the timeout budget in
`apis.md` ("Every call out is a call that hangs"), and the two numbers
should agree.

Write the budget down where the work is described, and treat exceeding it
as a defect rather than as a fact of life. A performance target nobody
recorded becomes whatever the code does, one commit at a time.

## Fix the bottleneck, not the list

When a profile shows several expensive things → work on the largest one
until it is no longer the largest, then re-measure. This is not
tidiness; it is arithmetic. Halving something that accounts for 5% of the
time buys 2.5%, and no amount of effort on it can buy more. The ceiling
on any optimization is the share of total time it touches, and that
ceiling is why "we optimized everything" so often produces a system that
is no faster.

Two consequences. Re-measure after every change, because fixing the top
item usually reorders everything below it and the second-most-expensive
thing on the old profile is often irrelevant on the new one. And be
willing to conclude that the bottleneck is not in your code at all — a
dependency, a network hop, a store, a serialization boundary — because
that conclusion redirects the effort to where it can actually pay.

## Do it less, in bulk, or later

When choosing what change would help → try these in order. They are
ranked by how much they buy relative to the complexity they add.

1. **Don't do it.** The fastest work is the work that does not happen.
   Look for the request made twice, the value computed and discarded, the
   field fetched and never read, the loop that recomputes an invariant,
   the middleware that runs on a path that does not need it. Elimination
   costs no complexity and can never introduce a staleness bug.
2. **Do it once.** Compute a value a single time and reuse it within the
   scope where it cannot change. This is memoization at its smallest and
   safest — inside one request, one function, one pass — and it is
   distinct from caching across requests, which is a much bigger
   commitment.
3. **Do it in bulk.** Replace many small round trips with one larger one.
   The per-call overhead — connection, round trip, parse, lock — usually
   dominates the per-item cost, which is exactly why the N+1 shape hurts
   so much (`data.md`, "The N+1 is a shape"). Batching turns a linear
   number of expensive round trips into one.
4. **Do it later.** Move work out of the path the user is waiting on:
   enqueue it, defer it, run it on a schedule. The user's time is the
   scarce resource; the machine's is not.
5. **Do it faster.** Only now — a better algorithm, a better data
   structure, a tighter implementation. This is where most people start
   and where the least return usually lives.

Note that steps 1–4 do not require the code to become cleverer. Step 5
often does, which is why it is last.

## Growth beats constants

When performance is acceptable today but the data is growing → look at
the *shape* of the cost, not its current value. An operation whose cost
grows with the square of the input is fine on a hundred items and fatal
on ten thousand, and the change between those two states is not gradual —
it feels like a system that was fine yesterday.

The shapes worth spotting by eye: a loop inside a loop over the same
growing set; a lookup by scanning a list instead of a keyed structure; a
query without an index whose table keeps growing (`data.md`, "Index the
query you actually run"); anything that loads a whole collection into
memory to answer a question about part of it; a per-item operation that
issues a call.

The practical test is to measure at two sizes, not one. If ten times the
data costs ten times the time, the shape is linear and the constant is
the conversation. If it costs a hundred times, no constant-factor tuning
will save it — the approach has to change, and it is far cheaper to
change it now than after it becomes an incident.

## A cache is a copy with an owner

When a cache is proposed → understand what you are buying and what you
are signing up for. A cache trades correctness-over-time for speed: it
serves data that was true when it was stored. Every cache therefore needs
four answers before it is built, and a cache added without them will
eventually serve something wrong to someone who notices.

- **What is the staleness contract?** How out of date may this be, stated
  in units the product would accept? "Never stale" is not a cache; it is
  a copy with a synchronous invalidation problem.
- **Who invalidates it, and what happens if that fails?** Invalidation
  belongs with the write that makes the cached value wrong. A cache
  invalidated by nothing but expiry is honest; a cache whose invalidation
  is spread across four call sites is a bug with a schedule.
- **What is the key, exactly?** Caches leak across boundaries when the
  key omits something that varies the answer — a user, a tenant, a
  locale, a permission level. A cached response served to the wrong
  tenant is a data breach caused by a performance optimization.
- **What happens when it is empty?** On a cold start, an eviction, or an
  expiry of a popular entry, every waiting request recomputes the same
  value at once and lands on the origin simultaneously. Decide whether
  that stampede is survivable, and if not, let one request compute while
  the others wait or serve the stale value.

Cache as close to the consumer as the correctness allows — a value held
for the duration of one request is nearly free of these problems, and a
shared distributed cache has all of them.

## The tail is the product

When the average is fine and users complain → look at the distribution.
Averages hide the failures: a system where 99 requests take 50 ms and one
takes 8 seconds has an excellent average and one very unhappy user, and
that user is the one who writes the review.

Judge latency at a high percentile, and be aware that the tail is what a
*session* experiences, not what a request experiences. A page making
twenty calls, each with a 1-in-100 chance of being slow, is slow most of
the time. That arithmetic is why tail latency is worth more attention
than the median in almost every system with fan-out.

The usual sources of a bad tail are not "slow code" — they are queueing
behind something, a lock, a cold cache, garbage collection, a retry, a
connection pool exhausted by one slow dependency, or a resource near
saturation. Note the general rule: as utilization approaches capacity,
waiting time rises far faster than utilization does. A resource run at
90% is not 10% away from trouble; it is already in it.

## Measuring without lying to yourself

When timings vary between runs → fix the measurement before drawing
conclusions from it. The common ways a benchmark misleads:

- **Cold versus warm.** The first run pays for connections, caches,
  compilation, and page faults. Decide which one you care about — startup
  latency and steady-state throughput are different products — and
  measure that one deliberately.
- **Unrealistic data.** Ten rows of seed data will not reproduce a
  production plan, a cache miss rate, or a memory profile. Shape matters
  as much as size: skew, duplicates, and hot keys change everything.
- **Measuring the harness.** A loop that computes a value nobody uses can
  be optimized away entirely, and a timer around an asynchronous call
  that returns immediately measures nothing.
- **One run.** Repeat, and look at the spread as well as the middle. A
  change that moves the middle by 3% and the spread by nothing has
  probably not moved anything.
- **Changing two things.** One variable per measurement, or the result is
  unattributable.

For anything that must not regress, the durable form is a measurement in
the suite with a threshold, so the regression is caught by a machine and
not by a customer. Keep it stable enough not to flake — a performance
test that fails randomly gets disabled, and then it protects nothing.

## Parallelism is not a speed setting

When adding threads, workers, or concurrent calls → know which of two
problems you have. Parallelism helps when the work is genuinely divisible
and the resource is genuinely idle. It does nothing for a task that is
already waiting on one saturated resource — running four copies of a
query against a database at its limit produces four slow queries.

Three costs come with it and should be priced before the change:
coordination (locks, queues, and synchronization are work, and contention
can make the parallel version slower than the serial one), the
non-parallelizable remainder (the serial part sets a floor no amount of
workers goes below), and correctness (shared state under concurrency
produces defects that are timing-dependent, rare, and painful — see
`data.md`, "The anomalies worth knowing by name").

The version of concurrency with the best return is usually the simplest:
overlapping *waiting*. Issuing independent calls at once instead of in
sequence turns the sum of their latencies into the maximum, costs almost
no complexity, and is where most real wins in request-path code come
from.

## Memory is a performance property

When memory or allocation is suspected → note that memory shows up as
*time*, which is why it is often diagnosed late. Allocation pressure
becomes collection pauses, which become tail latency; a working set that
exceeds cache becomes stalls; a working set that exceeds available memory
becomes swapping or a process the system kills.

The shapes to look for: loading an entire collection to process it one
item at a time when a stream would do; accumulating results that are
never released; holding a large object alive because something small
still references it; copying structures at every layer boundary; and
per-item allocation inside a hot loop.

A leak is a growth curve, not a number, so measure over time and under
load rather than at one instant. And prefer bounding to tuning: a
structure with an explicit maximum size fails predictably, while an
unbounded one fails at the worst possible moment.

## Perceived speed is real speed

When an interface feels slow but the numbers look fine → the number you
are measuring is not the one the user experiences. What people perceive
is time-to-something-useful, and progress they can see. A response that
takes the same total time but shows meaningful content immediately is
experienced as faster, and that is not a trick — it is the metric that
matters being different from the one you measured.

The moves that buy perceived speed: show the part that is ready instead
of waiting for all of it; respond to input immediately even if the result
is not final; start the likely next work before it is requested; and make
waiting legible with real progress rather than an indefinite spinner. The
detail belongs to the interface (`frontend.md`, "Perceived speed is
designed"), but the measurement principle belongs here: instrument the
moment the user can *act*, not the moment the last byte arrives.

## Know when to stop, and pin it

When the target is met → stop, and leave two things behind. Record the
number that was achieved and the conditions it was measured under, so the
next person knows what "normal" is and can tell a regression from a bad
day. And add the check that keeps it — a threshold in the suite, a
dashboard, an alert — because an optimization with nothing defending it
decays back to baseline through ordinary changes, silently, over a few
months.

Then reverse the question: what did the speed cost? A faster
implementation that is harder to understand, a cache that can serve stale
data, a denormalized copy that can drift — each was a real trade, and it
should be written where the next reader will find it. Performance work
that leaves no explanation behind is indistinguishable from unnecessary
complexity to whoever inherits it, and it will be simplified away by
someone acting reasonably.
