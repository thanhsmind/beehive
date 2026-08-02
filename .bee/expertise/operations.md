# Shipping and running it

Code that has never run in front of real traffic has not been finished;
it has been written. This guide covers the second half — getting a change
in front of users without risking everything, and knowing what the system
is doing once it is there.

Two properties dominate everything below, and they are worth building
before anything else: **the ability to see** what the system is doing, and
**the ability to undo** what you just did. A team with both can recover
from almost any mistake quickly. A team with neither is one bad deploy
away from a long night, no matter how good the change was.

## Where to look

| Situation / goal | Entry |
|---|---|
| About to ship anything | Reversibility is the first question |
| A change is risky but wanted | Deploying is not releasing |
| Choosing how a change reaches users | Pick a rollout by blast radius |
| A change spans code and stored data | Two clocks, never one deploy |
| Values differ between environments | The same artifact everywhere |
| Development works and production does not | Name every environment difference |
| Adding a health or readiness check | A health check makes a promise |
| Deciding what to instrument | Instrument what you would be paged for |
| Writing a log line for future you | Logs are evidence, not narration |
| Creating an alert | Alert on symptoms, and only when someone must act |
| Something is broken right now | Stabilize, then diagnose |
| The incident is over | The postmortem asks what let it through |
| A recurring job needs to run | Scheduled work is a system |
| Load is growing, or a spike is expected | Degrade on purpose |
| A procedure exists only in someone's head | Write the runbook before you need it |

## Reversibility is the first question

When about to ship anything → ask how it comes back out, and how long
that takes. The answer determines how much care the change needs and how
it should be rolled out. A change that reverts in two minutes with one
command can be shipped on a Friday afternoon by one person; a change that
cannot be reverted — because it rewrote data, sent messages, or moved a
one-way boundary — deserves a different level of scrutiny than its diff
size suggests.

Sort every change into three buckets and treat them differently:

- **Reversible** — code, configuration, flags. Ship freely, verify after.
- **Reversible with effort** — a schema addition, a new dependency, a
  rollout that has partially propagated. Ship deliberately, know the
  procedure.
- **Irreversible** — deleted data, sent email, charged money, a published
  version, an external contract. These get the full weight: a rehearsal,
  a second pair of eyes, and an explicit decision by someone who owns the
  consequence.

The most common way a reversible change becomes irreversible is that a
data migration rode along with it. See "Two clocks, never one deploy."

## Deploying is not releasing

When a change is wanted but risky → separate putting the code in
production from turning the behavior on. Ship the code inert, behind a
switch, then enable it separately — for yourself, then a few users, then
everyone. The two events have different risk profiles, and merging them
means every rollback is a deploy.

The benefit is not only safety. Inert code in production is code that has
already survived the build, the startup path, and the environment, so the
risky moment is a configuration change with an instant undo rather than a
release with a rebuild.

The cost is real and worth naming: every switch is a branch in the
system's behavior, and two switches make four states that in principle
need thinking about. So switches are temporary by default — a flag with
no removal plan becomes permanent complexity, and a codebase carrying
dozens of them has a combinatorial space nobody can test. Name the owner
and the removal condition when you add one, and delete it when the
rollout is done.

## Pick a rollout by blast radius

When choosing how a change reaches users → choose by how much damage the
change could do and how fast you would notice, not by what the pipeline
does by default.

- **All at once** is right for small, reversible, well-covered changes.
  Its virtue is that there is only one version live, which makes every
  subsequent question simpler.
- **Incremental exposure** — a small fraction of traffic first, widening
  as it holds — is right when the change is behavioral and the failure
  would be visible in metrics. Its whole value is in the *watching*: an
  incremental rollout that nobody monitors is a slow full rollout with
  extra steps. Decide before you start what signal would stop it.
- **Parallel environments** — bring up the new version alongside the old,
  switch traffic, keep the old one warm — is right when you need the undo
  to be instant and you can afford to run both. Note that it does not
  help with anything that shares state, which is usually the database.

Whatever the mechanism, two versions of your code will be live
simultaneously during the transition. Every change must therefore be
compatible with the version it is replacing — in both directions, because
a rollback runs the old code against whatever the new code wrote.

## Two clocks, never one deploy

When a change spans code and stored data → the deploy and the data move
on different clocks, and forcing them into one step is the classic way a
routine release becomes an outage with no rollback.

Split it: change the data shape in a way both code versions tolerate,
deploy the code, then remove what is no longer used — the expand,
migrate, contract sequence covered in `data.md` ("Expand, migrate,
contract"). Each step is separately deployable and separately
reversible.

The rule that catches the remaining cases: **a deploy must be safe to
roll back for as long as it might be rolled back.** If reverting the code
would leave rows the old version cannot read, the migration was in the
wrong phase, and the safest deploy in the world cannot save you from it.

## The same artifact everywhere

When a value differs between environments → the value moves, the build
does not. Build once, deploy that same artifact to every environment, and
let configuration supplied at run time be the only difference. A pipeline
that rebuilds per environment is testing one artifact and shipping
another.

Configuration follows three rules. It comes from the environment or a
config service, not the source. Secrets come from a secret store, never
from the same place as ordinary config (`security.md`, "Secrets live
outside the code"). And **the process refuses to start when required
configuration is missing or malformed** — validated at startup, with a
message naming what is missing. The alternative is a service that starts
healthy and fails on the first request that touches the missing value,
which is a much worse way to find out.

## Name every environment difference

When something works in development and fails in production → suspect the
differences before the code, and know them in advance rather than
discovering them under pressure. The recurring list: data volume and
skew, concurrency, latency between components, resource limits,
permissions, filesystem and path semantics, clock and timezone, network
egress rules, and the versions of everything underneath.

Parity is worth paying for where it is cheap — the same runtime version,
the same store engine, the same operating system family — precisely so
that the differences that remain are few and *named*. A team that can say
"production differs from staging in exactly these four ways" debugs a
production-only failure in minutes. A team that cannot has to consider
everything.

Treat any environment difference nobody chose as a defect: it will
eventually be the cause of a bug, and it will cost a day to find because
nobody suspected it.

## A health check makes a promise

When adding a health or readiness check → decide what it is actually
asserting, because the machinery will act on it. There are two different
questions and confusing them causes both outages and non-outages:

- **Am I alive?** — this process is not wedged, and restarting it would
  help. Keep it shallow. A liveness check that fails when a downstream
  dependency is slow will restart every healthy instance during someone
  else's outage.
- **Am I ready?** — this instance can serve traffic right now: warmed,
  configured, connected. Readiness may legitimately depend on
  dependencies. An instance that reports ready before it can serve
  produces errors that look like a bad deploy.

The failure to avoid is a check that only proves the web server can
answer a request. It reports green while every real request fails, which
means every automatic protection you built is disarmed exactly when you
need it. Make the check exercise at least the path that would break.

## Instrument what you would be paged for

When deciding what to measure → work backward from the questions you will
have at three in the morning: *is it broken, where, and why.* Each
question has a natural tool. Aggregate measurements answer whether
something is broken and give the shape over time. A request-scoped trace
answers where the time or the failure is, across components. Detailed
records answer why, for one specific occurrence.

For any user-facing surface, the durable set is: how much traffic, how
much of it fails, how long it takes, and how loaded the resource is.
Measure the timings as a distribution and judge them at the tail — the
average is a number no user experiences, and a healthy average routinely
conceals a slow percentile that is somebody's whole opinion of your
product.

Two rules keep the data honest. **Measure at the boundary the user cares
about** — an internal timing that excludes queueing and network is
measuring your comfort, not their experience. And **give every request an
identifier that travels with it** across services, into logs and into
downstream calls, because without it the three tools above are three
separate stories that cannot be joined.

## Logs are evidence, not narration

When writing a log line → write it for the person reconstructing an
incident, not for the person watching the console during development.
That person needs to find one occurrence among millions and understand
what happened around it.

So: emit structured records with stable field names rather than
interpolated sentences, because the questions asked later are filters and
groupings, not reading. Include the correlation identifier, the actor,
the object, and the outcome. Log at boundaries and decisions — what came
in, what was chosen, what went out, what failed — rather than at every
step, since a log that records everything is one nobody can search and a
bill nobody wants. Log the failure *with its cause attached*, not a
message announcing that a failure occurred somewhere.

And never log secrets or personal data that has no business in a
searchable store (`security.md`, "Log the event, never the secret").

## Alert on symptoms, and only when someone must act

When creating an alert → alert on the thing users experience — errors,
latency, work not getting done — not on the intermediate causes. Cause
alerts multiply (there are many ways to be broken) and they miss the
failure nobody predicted; a symptom alert catches every cause of the
symptom, including the new ones.

Every alert answers two questions before it exists: *what does the person
who receives this do about it*, and *what happens if it is ignored until
morning*. An alert with no action is a notification, and it belongs in a
dashboard or a digest. An alert that can wait is not a page. Anything
that fires regularly and is routinely dismissed is worse than nothing: it
trains the team to ignore the channel that will one day carry the real
one.

Route accordingly — urgent and actionable to a person now, everything
else to a place people look on purpose — and delete alerts that have
never once led to an action.

## Stabilize, then diagnose

When something is broken right now → restore service first. Understanding
comes second, and the instinct to find the root cause before acting is
the most expensive habit in incident response. Roll back the recent
change, fail over, disable the feature, add capacity, shed load — the
reversible action that makes users whole is correct even if it turns out
the change was innocent.

Three things keep the response from becoming its own incident. **One
person coordinates** and is not the same person with their hands in the
system; their job is to hold the picture, decide, and communicate.
**Changes are announced and made one at a time**, because three
simultaneous fixes make the recovery unattributable and can interact.
**Communication goes out on a clock**, at a stated interval, saying what
is known, what is being done, and when the next update comes — silence is
read as absence, and it generates a second wave of interruptions aimed at
the people fixing it.

Keep a timeline as you go. Writing down what was observed and done, with
times, costs almost nothing during the incident and is the entire input
to what follows.

## The postmortem asks what let it through

When the incident is over → the question is what made this possible and
what makes the next one visible sooner, never who typed the command.
People do reasonable things given what they knew at the time; a review
that concludes with "be more careful" has found nothing and changed
nothing.

Write down the timeline, the impact in terms someone outside can
understand, what actually caused it, and — most valuably — what made it
take as long as it did to detect and to fix. That last part is where the
real findings live: a missing signal, an alert that pointed at the wrong
thing, a runbook that was wrong, a rollback that took twenty minutes.

Come out with a small number of specific, owned actions. A list of
fifteen improvements is a list nobody will do; two that land are worth
more than the document.

## Scheduled work is a system

When something must run on a schedule → treat it with the same care as a
request path, because it fails in ways request paths do not and nobody is
watching when it does. Decide explicitly:

- **What happens when a run is still going when the next one starts** —
  overlap, skip, or queue. Silence here means overlap, and overlap on a
  job that was not designed for it corrupts things.
- **What happens when a run is missed** — because the system was down or
  the schedule did not fire. Catch up, or skip forward?
- **Whether a partial run is safe to repeat** — it will be. Make the work
  idempotent and resumable (`data.md`, "Backfills are jobs, not
  statements").
- **How anyone learns it stopped running.** A job that silently stops is
  the classic invisible outage: nothing breaks, data just quietly gets
  older. Alert on *absence of success*, not only on failure.

Pin the timezone deliberately, and remember that schedules interact with
offset changes: an hourly job runs twice or not at all on the days the
clocks move.

## Degrade on purpose

When load grows beyond what the system can serve → decide in advance what
gets sacrificed, because the default is that everything fails together.
A system that sheds the expensive optional work and keeps serving the
core is having a bad day; a system that tries to serve everything at once
is having an outage.

The moves worth designing before you need them: shed load at the edge
rather than accepting requests you cannot finish; serve a stale or
simplified response when the fresh one is unavailable; disable
non-essential features by switch; and queue what can be done later
instead of doing it inline. Each is a decision about what matters most,
and it should be made calmly, once, rather than under pressure.

Know one number: what happens at roughly ten times current load, and
which resource runs out first. You do not need a capacity model — you
need to have thought about it once, and to have a lever you can pull.

## Write the runbook before you need it

When a procedure exists only in one person's head → write it down while
things are calm. The candidates are the ones that matter under pressure:
how to roll back, how to fail over, how to rotate a credential, how to
restore from backup, how to disable a feature, how to reach whoever owns
the dependency.

A runbook that has never been executed is a hypothesis. Rehearse the ones
whose failure is expensive — a restore that has never been performed is
not a backup (`data.md`, "A backup is a restore you have performed"), and
a failover that has never been exercised is a plan. Fix the runbook every
time it turns out to be wrong during an incident, at the moment you
discover it, because that is the only time anyone knows exactly what was
missing.
