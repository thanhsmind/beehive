# Working with data

Stored data outlives the code that writes it. A function can be rewritten
in an afternoon; a column that shipped wrong is still wrong three years
later, in every backup, in every export a customer already downloaded.
That asymmetry is the whole reason this guide exists — decisions about
data are made at code speed and paid for at data speed.

## Where to look

| Situation / goal | Entry |
|---|---|
| Designing a new table, collection, or document shape | The store is a contract |
| Deciding how to structure entities and their relations | Model the facts, not the screens |
| Tempted to enforce a rule only in application code | Constraints belong beside the data |
| Choosing a primary key, or "should this be unique?" | Keys are claims about the world |
| A query is slow, or someone proposes adding an index | Index the query you actually run |
| A list page issues one query per row | The N+1 is a shape |
| Two writes must both happen or neither | Name the transaction boundary |
| Two sessions can update the same record | The anomalies worth knowing by name |
| Changing a column, table, or field that is already live | Expand, migrate, contract |
| Rewriting rows in bulk | Backfills are jobs, not statements |
| A record must go away | Deletion is a decision, not a DELETE |
| Storing a timestamp, or comparing two of them | Time is not a number |
| A fix is believed to be faster | Read the plan before believing the fix |
| Data exists that no code could recreate | A backup is a restore you have performed |
| Judging a schema you did not design | Reading a schema you inherited |

## The store is a contract

When designing any persistent shape — a table, a collection, a document,
a file format, an event payload → treat it as a published interface, not
as an implementation detail of the code that happens to write it today.
Application code is the store's *current* client, never its only one:
reports, exports, backfills, the next service, and a human with a query
console all read it too. So the schema is judged by what a reader can
understand without the writer's source code beside them.

Two consequences. First, names carry meaning to strangers: `status` is
not a name, `payment_status` is; a boolean called `flag` is a puzzle
shipped to whoever reads it next. Second, a shape you would be
embarrassed to document is a shape to change now — while it holds a
thousand rows and one writer, not a billion rows and nine.

## Model the facts, not the screens

When the entities are being drawn from a mockup, an API response, or a
form → model what is *true about the world*, and let each view assemble
itself from those facts. A screen is a projection of the data and it will
be redesigned; if the schema mirrors the screen, every redesign becomes a
migration.

The test for whether something is a fact: could two different features
disagree about it? "The order total" is a fact. "The order summary card"
is a screen. Store one row per real-world occurrence and let the card be
a query.

**Denormalize deliberately, and name the owner.** Copying a value to
where it is read is a legitimate optimization — a cached count, a
snapshot of a price at purchase time, a materialized rollup. It stops
being legitimate when nobody can say which write is responsible for
keeping the copy true. Before duplicating a value, answer in one
sentence: *who updates this copy, and what happens when that update
fails?* An unanswerable question means the copy will drift, and drifted
copies are discovered by customers, not by tests.

Note that a *snapshot* and a *cache* look identical in the schema and are
opposites in meaning: the price stored on an order line is deliberately
frozen and must never be refreshed; a denormalized comment count is
deliberately live and must never go stale. Say which one it is in the
column's own name or comment, because the next person cannot tell by
looking.

## Constraints belong beside the data

When a rule is invariant — this column is never null, these two columns
are unique together, this reference must point at a real row → declare
it in the store, not only in the code path that happens to write it.
Application-level validation guards one door. The store guards every
door: the migration script, the admin console, the backfill job, the
incident-time manual fix, and the service someone writes next year.

The rule generalizes past databases. Any store with a schema mechanism —
a typed column, a check constraint, a foreign key, a unique index, a
schema validator on a document collection — should carry the invariants
that are genuinely invariant. What should *not* go there is policy that
legitimately varies: a discount ceiling, a rate limit, anything a product
decision could change next quarter. Constraints are for facts that cannot
be otherwise, not for rules that happen to be true right now.

**A constraint you cannot add is a finding.** When adding a NOT NULL or a
unique index fails on existing rows, the failure is not an obstacle to
route around — it is the store telling you the invariant was already
violated, in production, silently. Fix the rows first, then add the
constraint; skipping to "we'll enforce it in code" leaves the bad rows
there forever.

## Keys are claims about the world

When choosing a primary key → prefer an identifier with no meaning: a
generated id that says nothing about the row it names. A key built from
real-world attributes — an email, a phone number, a country-plus-code
tuple — asserts that those attributes never change, and they change.

Uniqueness constraints are the same kind of claim, and they are the
cheapest place to be wrong. Before declaring a unique index, ask what
happens when the world produces a duplicate: two people genuinely share
a phone number, one person legitimately holds two accounts, a
soft-deleted row still occupies its slot. If the answer is "the write
fails and a user is stuck", the constraint is either wrong or needs a
partial form that excludes the deleted rows.

**Ids that leave the system are part of the interface.** A sequential
integer in a URL tells every visitor how many records exist and lets them
walk the neighbors. When an id is exposed externally, prefer one that is
unguessable and carries no volume signal — and remember that changing
this later means changing every link anyone ever bookmarked.

## Index the query you actually run

When a read is slow → look at the query first and the index second. An
index is not a general speed setting; it is a sorted copy of specific
columns, useful only to queries whose filters and ordering match its
leading columns. Three things decide whether it helps:

- **Selectivity.** An index earns its keep when it eliminates most rows.
  Indexing a column with three distinct values across a million rows
  usually buys nothing — the store still reads a third of the table.
- **Leading column order.** A composite index on `(a, b)` serves filters
  on `a` and on `a, b`. It does not serve a filter on `b` alone. Order
  the columns by how queries actually filter, equality columns first.
- **What the query needs back.** When every column a query returns is in
  the index, the store answers from the index alone. That is a large win
  and the reason a slightly wider index sometimes beats a narrow one.

Indexes are not free: every one of them is extra work on every insert,
update, and delete, plus storage and memory. So the audit runs both ways
— find the queries with no supporting index, and find the indexes no
query uses. An unused index is a permanent tax collected for nothing.

## The N+1 is a shape

When code loops over a result set and touches the store inside the loop
→ recognize the shape before measuring it: one query for the list, then
one more per item. It is invisible at ten rows in development and fatal
at ten thousand in production, and it is the single most common cause of
a page that "got slow for no reason."

The fix is always the same in principle: fetch the related data in one
round trip keyed by the whole set, then join in memory. The shape hides
in more places than a literal `for` loop — a lazily-loaded relation
touched during serialization, a permission check inside a formatter, a
helper that looks pure and issues a query. When a request's query count
scales with its result count, you have it, whatever the code looks like.

## Name the transaction boundary

When two or more writes must not be observed half-done → wrap them in
one transaction, and be able to state in a sentence what invariant that
transaction protects. "Both writes are related" is not an invariant.
"The ledger must never show a debit without its credit" is.

Two rules keep transactions from becoming the problem they solve. Hold
them for as short a time as possible — a transaction is a lock on
concurrency, and its cost is paid by every other session waiting. And
never hold one across an operation you do not control: a network call, a
queue publish, a third-party API. That pattern couples the store's health
to a stranger's latency, and it makes the two-system consistency worse,
not better — the remote call can succeed while the local transaction
rolls back, and now the systems disagree with no record of why.

**Cross-system consistency needs a different tool.** When the work spans
a store and something else — another service, a payment provider, an
email — a transaction cannot span them. Make each step idempotent and
retryable, record intent before acting, and reconcile: write "we are
about to charge" durably, then charge, then record the outcome. On
recovery, the intent record tells you what to check.

## The anomalies worth knowing by name

When two sessions can touch the same data → know which of these you are
exposed to, because each has a different fix:

- **Lost update** — two sessions read a value, both compute from it, and
  the second write erases the first. Fix by making the update itself
  atomic (`set x = x - 1`, not read-then-write) or by version-checking
  the row on write.
- **Write skew** — two sessions each read a set, each individually
  satisfies a rule, and their combined writes break it. The classic is
  two "last" resources booked at once. Fix by locking the *set* being
  reasoned about, or by moving the rule into a constraint the store
  itself enforces.
- **Phantom read** — a query run twice inside one transaction sees rows
  that were not there before, because another session inserted them.

Isolation levels are a defense against these, and the defaults are
usually weaker than people assume. Rather than memorizing level names,
ask the local question: *if two copies of this request ran at the same
instant, what would be wrong afterward?* Then make that specific outcome
impossible — with an atomic operation, a constraint, or an explicit lock,
in that order of preference.

## Expand, migrate, contract

When changing a shape that is already live → never do it in one step.
Old and new code run simultaneously during any deploy, and a one-shot
rename breaks whichever half loses the race. Split every change into
three deployable phases:

1. **Expand.** Add the new column, table, or field. It is nullable or
   defaulted, nothing reads it yet, and old code is unaffected.
2. **Migrate.** Write to both shapes; backfill the existing rows; move
   readers to the new shape once it is complete and verified.
3. **Contract.** When nothing reads or writes the old shape, remove it.

Each phase ships and is reversible on its own. The rename that looks like
one line of migration is three deploys, and the discipline is what makes
each of them safe to roll back.

**Every migration is code, and code that has never been rehearsed is a
guess.** Run it against a copy shaped like production — same row counts,
same data skew, same indexes — before running it against production.
Locking behavior and duration are properties of scale, and a migration
that takes 40 ms on a development seed can hold a write lock for twenty
minutes on the real table.

## Backfills are jobs, not statements

When existing rows must be rewritten in bulk → treat it as a program
with four properties, never as a single statement left running in a
console:

- **Batched**, so no one statement locks a large range or exhausts
  memory.
- **Resumable**, recording its own progress, so an interruption costs one
  batch rather than the whole run.
- **Idempotent**, so re-running an already-processed batch changes
  nothing — it will be re-run, by a retry or by a human who lost the
  terminal.
- **Throttled and observable**, with a visible rate and a way to stop it,
  because the backfill shares its capacity with live traffic.

Verify by counting what remains, not by trusting the job's own log: the
honest completion check is a query that finds zero unmigrated rows.

## Deletion is a decision, not a DELETE

When a record must go away → decide explicitly which kind of gone it is,
because the choice has consequences no code review will surface later.
Hard deletion is honest and irreversible: it satisfies "erase my data"
obligations and it destroys the audit trail. Soft deletion keeps history
and quietly infects everything downstream — every query needs the filter,
every unique index needs a partial form, and one forgotten `where` clause
resurrects deleted records in a report.

Answer three questions before either: what must survive the deletion for
audit or legal reasons; what should happen to rows that reference this
one (cascade, orphan, or refuse); and how long the data is kept when the
answer is "not forever." Retention that is not written down is retention
forever, which is a decision made by accident.

## Time is not a number

When storing an instant → store it as an unambiguous absolute instant,
and store the *intent* separately when the intent is local. A meeting at
"9:00 in Berlin" is not the same fact as the instant that resolved to
last month, and storing only the instant loses the meeting when the
offset rules change — which they do, by legislation, several times a
year.

Related traps that surface as bugs in production and never in tests: a
day is not always 24 hours; a local time can occur twice or not at all
across an offset change; date arithmetic that adds 86400 seconds is
wrong at exactly the boundaries people notice; and "today" depends on
who is asking. Choose the user's timezone or the system's deliberately —
never by whichever the runtime defaulted to.

## Read the plan before believing the fix

When a query change is believed to be faster → ask the store how it will
execute the query, before and after. Every serious store can explain a
plan: which index it chose, whether it scanned, how many rows it expects
versus how many it found. That output turns "this should be faster" into
a fact.

Two things make the reading honest. Compare estimates against actuals —
a plan that expects 10 rows and processes 400,000 is running on stale
statistics, and the real fix is refreshing them, not rewriting the query.
And run the comparison at realistic scale: on a small table the store
scans because scanning is genuinely fastest, so a development plan tells
you nothing about the production one.

## A backup is a restore you have performed

When data exists that no code could recreate → the question is never
"are we backing up?" but "when did we last restore?" A backup job that
reports success proves that a file was written, not that it is complete,
not that it is readable, and not that anyone knows the procedure under
pressure.

Restore it, on a schedule, into a scratch environment, and check the data
is actually there. The two numbers worth stating out loud — how much data
you can afford to lose, and how long a restore takes — are properties of
the procedure you have rehearsed, not of the one you intend to write.

## Reading a schema you inherited

When judging a store you did not design → read it in this order, because
each step explains the next: the tables and their row counts (what is
large is what matters), then the constraints and indexes (what the
designers believed was invariant and what they optimized for), then the
queries in the hot paths (what is actually asked of it), and only then
the application code.

Look specifically for the tells: a column whose name no longer matches
its contents, two columns that must agree with no constraint saying so, a
nullable column that is never null in practice, an index nothing uses, an
enum with a value the code cannot produce. Each is a fossil of a change
made in code and not in the store, and each is where the next bug will
come from. Before removing any of them, find out why it was put there —
a schema is the place where an unexplained oddity is most likely to be
load-bearing.
